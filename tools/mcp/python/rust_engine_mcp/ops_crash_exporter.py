"""OPS crash exporter + DCC process monitor — daemon-friendly, not cron."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from uuid import uuid4

from .ops_atomic_witness import append_jsonl_atomic, write_witness_atomic
from .paths import repo_root

CRASH_EVENTS_REL = "debug_runs/agent_ops/crash_events.jsonl"
CRASH_STATE_REL = "debug_runs/agent_ops/crash_daemon_state.json"
TRIAGE_LIVE_REL = "debug_runs/agent_ops/triage_live.json"
PROMETHEUS_REL = "debug_runs/agent_ops/prometheus/rust_engine_ops.prom"
PREVIEW_JOBS_DIR = "debug_runs/preview_jobs"

_DCC_NAMES = ("blender", "bpy")
_ENGINE_NAMES = ("python", "cargo", "rustc", "proc_a_dine", "bevy", "pytest")

_STALE_WITNESS_HOURS = 24
_DATA_DROP_PATHS = (
    "debug_runs/unified_witness_index.json",
    "debug_runs/agent_ops/ops_report_latest.json",
    "debug_runs/agent_ops/ops_dashboard_live.json",
)

_GLYPH = {
    "crash": "⛔",
    "data_drop": "🧊",
    "stale_witness": "⚠",
    "dcc_exit": "🔴",
    "process_surge": "⚡",
    "ok": "✅",
}


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _load_state() -> dict[str, Any]:
    path = repo_root() / CRASH_STATE_REL
    if not path.is_file():
        return {"tracked_pids": {}, "last_scan_epoch": 0}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        return data if isinstance(data, dict) else {"tracked_pids": {}, "last_scan_epoch": 0}
    except (json.JSONDecodeError, OSError):
        return {"tracked_pids": {}, "last_scan_epoch": 0}


def _save_state(state: dict[str, Any]) -> None:
    write_witness_atomic(
        CRASH_STATE_REL,
        state,
        actor="ops_crash_exporter",
        profile="CRASH-DAEMON-STATE",
        source_system="ops_crash_exporter",
        glyph=_GLYPH["ok"],
    )


def _scan_processes() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    if sys.platform == "win32":
        proc = subprocess.run(
            ["tasklist", "/FO", "CSV", "/NH"],
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        for line in (proc.stdout or "").splitlines():
            if not line.strip():
                continue
            parts = [p.strip('"') for p in re.findall(r'"[^"]*"|[^,]+', line)]
            if len(parts) < 2:
                continue
            name_lower = parts[0].lower()
            if not any(h in name_lower for h in _DCC_NAMES + _ENGINE_NAMES):
                continue
            rows.append(
                {
                    "name": parts[0],
                    "pid": parts[1],
                    "mem": parts[4] if len(parts) > 4 else "",
                    "is_dcc": any(d in name_lower for d in _DCC_NAMES),
                }
            )
    else:
        proc = subprocess.run(
            ["ps", "-eo", "pid,comm"],
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        for line in (proc.stdout or "").splitlines()[1:]:
            bits = line.split(None, 1)
            if len(bits) < 2:
                continue
            comm = bits[1].lower()
            if not any(h in comm for h in _DCC_NAMES + _ENGINE_NAMES):
                continue
            rows.append({"name": bits[1], "pid": bits[0], "mem": "", "is_dcc": any(d in comm for d in _DCC_NAMES)})
    return rows


def _scan_preview_failures() -> list[dict[str, Any]]:
    root = repo_root() / PREVIEW_JOBS_DIR
    if not root.is_dir():
        return []
    failures: list[dict[str, Any]] = []
    for path in root.glob("*.status.json"):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            continue
        if not isinstance(data, dict):
            continue
        status = str(data.get("status") or "").lower()
        exit_code = data.get("exit_code")
        if status in ("failed", "error", "crash") or (exit_code is not None and int(exit_code) != 0):
            failures.append(
                {
                    "kind": "preview_job_failure",
                    "path": str(path.relative_to(repo_root())).replace("\\", "/"),
                    "status": status,
                    "exit_code": exit_code,
                    "glyph": _GLYPH["dcc_exit"],
                }
            )
    return failures


def _scan_data_drops() -> list[dict[str, Any]]:
    drops: list[dict[str, Any]] = []
    now = time.time()
    for rel in _DATA_DROP_PATHS:
        path = repo_root() / rel
        if not path.is_file():
            drops.append(
                {
                    "kind": "data_drop",
                    "path": rel,
                    "detail": "expected witness missing",
                    "severity": "alert",
                    "glyph": _GLYPH["data_drop"],
                }
            )
            continue
        age_h = (now - path.stat().st_mtime) / 3600
        if age_h > _STALE_WITNESS_HOURS:
            drops.append(
                {
                    "kind": "stale_witness",
                    "path": rel,
                    "age_hours": round(age_h, 1),
                    "severity": "warn",
                    "glyph": _GLYPH["stale_witness"],
                }
            )
    return drops


def _detect_pid_exits(state: dict[str, Any], current: list[dict[str, Any]]) -> list[dict[str, Any]]:
    tracked: dict[str, Any] = dict(state.get("tracked_pids") or {})
    live_pids = {str(r["pid"]) for r in current}
    events: list[dict[str, Any]] = []
    for pid, meta in list(tracked.items()):
        if pid not in live_pids and meta.get("is_dcc"):
            events.append(
                {
                    "kind": "dcc_process_exit",
                    "pid": pid,
                    "name": meta.get("name"),
                    "severity": "alert",
                    "detail": f"DCC process {meta.get('name')} pid={pid} no longer running",
                    "glyph": _GLYPH["crash"],
                }
            )
            del tracked[pid]
    for row in current:
        pid = str(row["pid"])
        tracked[pid] = {"name": row["name"], "is_dcc": row.get("is_dcc"), "seen_at": _now_iso()}
    state["tracked_pids"] = tracked
    return events


def _blender_surge(current: list[dict[str, Any]]) -> list[dict[str, Any]]:
    blenders = [r for r in current if r.get("is_dcc")]
    if len(blenders) > 2:
        return [
            {
                "kind": "many_blender",
                "count": len(blenders),
                "severity": "warn",
                "detail": "Multiple Blender/DCC processes — stale headless bakes?",
                "glyph": _GLYPH["process_surge"],
            }
        ]
    return []


def run_crash_scan(*, record_events: bool = True) -> dict[str, Any]:
    """Single scan cycle — process poll, preview failures, data drops."""
    state = _load_state()
    processes = _scan_processes()
    pid_events = _detect_pid_exits(state, processes)
    preview_failures = _scan_preview_failures()
    data_drops = _scan_data_drops()
    surge = _blender_surge(processes)

    slip_ups: list[dict[str, Any]] = []
    slip_ups.extend(pid_events)
    slip_ups.extend(preview_failures)
    slip_ups.extend(data_drops)
    slip_ups.extend(surge)

    crash_total = len([e for e in slip_ups if e.get("severity") == "alert" or e.get("kind") == "dcc_process_exit"])
    dcc_count = len([p for p in processes if p.get("is_dcc")])

    if record_events:
        for ev in slip_ups:
            row = {"ts": _now_iso(), "run_id": str(uuid4()), **ev}
            append_jsonl_atomic(CRASH_EVENTS_REL, row, actor="ops_crash_exporter")
        state["last_scan_epoch"] = int(time.time())
        _save_state(state)

    return {
        "task_id": "OPS-CRASH-EXPORT-001",
        "schema": "ops_crash_scan_v1",
        "ok": True,
        "scanned_at": _now_iso(),
        "process_count": len(processes),
        "dcc_process_count": dcc_count,
        "crash_alert_count": crash_total,
        "slip_up_count": len(slip_ups),
        "slip_ups": slip_ups[:32],
        "processes": processes[:40],
        "glyph_summary": "".join(dict.fromkeys(e.get("glyph", "") for e in slip_ups if e.get("glyph"))),
        "prometheus_path": PROMETHEUS_REL,
    }


def write_prometheus_metrics(scan: dict[str, Any]) -> str:
    """Write Prometheus textfile exposition (node_exporter compatible)."""
    lines = [
        "# HELP rust_engine_ops_crash_alerts_total Crash/DCC alert count this scan",
        "# TYPE rust_engine_ops_crash_alerts_total gauge",
        f"rust_engine_ops_crash_alerts_total {scan.get('crash_alert_count', 0)}",
        "# HELP rust_engine_ops_slip_ups_total Slip-ups this scan",
        "# TYPE rust_engine_ops_slip_ups_total gauge",
        f"rust_engine_ops_slip_ups_total {scan.get('slip_up_count', 0)}",
        "# HELP rust_engine_dcc_process_count Live DCC processes",
        "# TYPE rust_engine_dcc_process_count gauge",
        f"rust_engine_dcc_process_count {scan.get('dcc_process_count', 0)}",
        "# HELP rust_engine_engine_process_count Live engine-related processes",
        "# TYPE rust_engine_engine_process_count gauge",
        f"rust_engine_engine_process_count {scan.get('process_count', 0)}",
    ]
    text = "\n".join(lines) + "\n"
    out = repo_root() / PROMETHEUS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    tmp = out.with_suffix(".tmp")
    tmp.write_text(text, encoding="utf-8")
    os.replace(tmp, out)
    return str(out.relative_to(repo_root())).replace("\\", "/")


def build_triage_live(*, window_hours: int = 168) -> dict[str, Any]:
    """Merge crash scan + ops blockers into triage dashboard witness."""
    from .ops_intelligence import ops_get_active_blockers
    from .ops_telemetry import build_ops_dashboard, scan_run_events

    crash = run_crash_scan(record_events=False)
    ops = build_ops_dashboard(window_hours=window_hours)
    blockers = ops_get_active_blockers()
    run_rollup = scan_run_events(window_hours=window_hours)

    open_gates = blockers.get("open_gates") or []
    blocker_rows = [
        {
            "id": g.get("id"),
            "severity": "warn",
            "glyph": "⚠",
            "detail": g.get("title") or g.get("id"),
        }
        for g in open_gates[:16]
        if isinstance(g, dict)
    ]

    all_slip = list(crash.get("slip_ups") or [])
    all_slip.extend(ops.get("slip_ups") or [])
    severity_rank = {"alert": 0, "warn": 1, "info": 2}
    all_slip.sort(key=lambda x: severity_rank.get(str(x.get("severity")), 9))

    body: dict[str, Any] = {
        "schema": "ops_triage_live_v1",
        "task_id": "OPS-TRIAGE-DASH-001",
        "ok": True,
        "generated_at": _now_iso(),
        "ignore_dcc_status_bar": True,
        "metrics": {
            "crash_alerts": crash.get("crash_alert_count"),
            "slip_ups": len(all_slip),
            "open_gates": len(open_gates),
            "ftr": (run_rollup.get("metrics_tier1") or {}).get("ftr"),
            "dcc_processes": crash.get("dcc_process_count"),
        },
        "glyph_chain": (crash.get("glyph_summary") or "") + "⚠⛔",
        "blockers": blocker_rows,
        "slip_ups": all_slip[:24],
        "crash": crash,
        "ops_dashboard": {
            "path": "debug_runs/agent_ops/ops_dashboard_live.json",
            "slip_up_count": ops.get("slip_up_count"),
        },
        "prometheus": {"path": PROMETHEUS_REL},
        "grafana": {
            "triage_dashboard": "tools/orchestrator/dashboard/grafana_triage_overview.json",
            "alert_rules": "tools/orchestrator/dashboard/prometheus_alert_rules.yml",
        },
    }
    return body


def write_triage_witness(*, window_hours: int = 168) -> dict[str, Any]:
    body = build_triage_live(window_hours=window_hours)
    scan = body["crash"]
    prom_path = write_prometheus_metrics(scan)
    body["prometheus"]["written"] = prom_path
    result = write_witness_atomic(
        TRIAGE_LIVE_REL,
        body,
        actor="ops_crash_exporter",
        profile="OPS-TRIAGE-LIVE",
        source_system="ops_crash_exporter",
        glyph=body.get("glyph_chain"),
    )
    body["written"] = TRIAGE_LIVE_REL
    body["content_hash"] = result.get("content_hash")
    return body


def daemon_loop(*, interval_sec: int = 30, max_cycles: int | None = None) -> None:
    """Background monitor — poll processes, export metrics, refresh triage witness."""
    cycles = 0
    while True:
        write_triage_witness()
        cycles += 1
        if max_cycles is not None and cycles >= max_cycles:
            break
        time.sleep(max(5, interval_sec))
