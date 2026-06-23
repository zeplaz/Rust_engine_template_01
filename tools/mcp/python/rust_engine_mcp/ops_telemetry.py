"""OPS telemetry — processes, drift instances, run_events rollup, dashboard bundle."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import time
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from uuid import uuid4

from .paths import repo_root

RUN_EVENTS_REL = "debug_runs/agent_ops/run_events.jsonl"
AGENT_MARKERS_REL = "debug_runs/agent_ops/agent_markers.jsonl"
OPS_DASHBOARD_REL = "debug_runs/agent_ops/ops_dashboard_live.json"
OPS_REPORT_REL = "debug_runs/agent_ops/ops_report_latest.json"
UNIFIED_INDEX_REL = "debug_runs/unified_witness_index.json"
VIEWPORT_DRIFT_REL = "debug_runs/viewport_drift.json"

_PROCESS_NAME_HINTS = (
    "python",
    "blender",
    "cargo",
    "rustc",
    "proc_a_dine",
    "bevy",
    "pytest",
)

_DRIFT_PATH_HINTS = (
    "viewport_drift",
    "authority_drift",
    "dual_write",
    "drift",
    "honest_gate",
    "dishonest",
    "inflated_green",
    "rollup_inflated",
    "queue_contradiction",
)


def _load_json(path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    return data if isinstance(data, dict) else None


def _read_jsonl(rel: str, *, limit: int = 5000) -> list[dict[str, Any]]:
    path = repo_root() / rel
    if not path.is_file():
        return []
    rows: list[dict[str, Any]] = []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return []
    for line in lines[-limit:]:
        line = line.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(row, dict):
            rows.append(row)
    return rows


def _parse_ts(raw: Any) -> datetime | None:
    if raw is None:
        return None
    text = str(raw).strip()
    if not text:
        return None
    try:
        if text.endswith("Z"):
            text = text[:-1] + "+00:00"
        return datetime.fromisoformat(text)
    except ValueError:
        return None


def scan_run_events(*, window_hours: int = 168) -> dict[str, Any]:
    """Roll up agent_run_event_v1 from run_events.jsonl — slip-ups and tier-1 proxies."""
    rows = _read_jsonl(RUN_EVENTS_REL)
    now = datetime.now(timezone.utc)
    cutoff = now.timestamp() - (window_hours * 3600)

    in_window: list[dict[str, Any]] = []
    for row in rows:
        ts = _parse_ts(row.get("ts"))
        if ts and ts.timestamp() >= cutoff:
            in_window.append(row)
        elif not ts:
            in_window.append(row)

    by_agent: Counter[str] = Counter()
    by_slice: Counter[str] = Counter()
    by_status: Counter[str] = Counter()
    demo_count = 0
    pytest_count = 0
    done_count = 0
    tools_flat: Counter[str] = Counter()

    for row in in_window:
        agent = str(row.get("agent") or "unknown")
        by_agent[agent] += 1
        slice_id = str(row.get("slice_id") or row.get("task_id") or "unknown")
        by_slice[slice_id] += 1
        status = str(row.get("status") or "unknown")
        by_status[status] += 1
        if row.get("demo"):
            demo_count += 1
        if agent == "pytest" or slice_id == "test-slice":
            pytest_count += 1
        if status == "done":
            done_count += 1
        for tool in row.get("tools_called") or []:
            tools_flat[str(tool)] += 1

    total = len(in_window)
    non_test = [r for r in in_window if str(r.get("agent")) not in ("pytest", "witness") and r.get("slice_id") != "test-slice"]
    non_test_total = len(non_test)

    repeat_slices = [
        {"slice_id": sid, "count": cnt}
        for sid, cnt in by_slice.most_common(12)
        if cnt >= 4 and sid not in ("test-slice", "MCP-DOC-READ-002-IMPL", "AGENT-LANG-DEMO-001")
    ]

    slip_ups: list[dict[str, Any]] = []
    for item in repeat_slices:
        slip_ups.append(
            {
                "kind": "repeat_slice",
                "severity": "warn" if item["count"] < 8 else "alert",
                "detail": f"slice {item['slice_id']} logged {item['count']}x in {window_hours}h",
                "slice_id": item["slice_id"],
                "count": item["count"],
            }
        )
    if demo_count > 5 and non_test_total > 0:
        slip_ups.append(
            {
                "kind": "demo_noise",
                "severity": "warn",
                "detail": f"{demo_count} demo events in window — filter for production KPIs",
                "count": demo_count,
            }
        )

    ftr = round(done_count / non_test_total, 3) if non_test_total else None
    rtr = round(1.0 - (done_count / non_test_total), 3) if non_test_total and done_count else None

    return {
        "task_id": "OPS-TEL-RUN-EVENTS-001",
        "schema": "ops_run_events_rollup_v1",
        "ok": True,
        "window_hours": window_hours,
        "total_events": total,
        "non_test_events": non_test_total,
        "by_agent": dict(by_agent.most_common(12)),
        "by_status": dict(by_status),
        "top_slices": [{"slice_id": s, "count": c} for s, c in by_slice.most_common(10)],
        "top_tools": [{"tool": t, "count": c} for t, c in tools_flat.most_common(12)],
        "demo_event_count": demo_count,
        "pytest_event_count": pytest_count,
        "done_count": done_count,
        "metrics_tier1": {
            "ftr": ftr,
            "rtr": rtr,
            "events_per_day": round(total / max(window_hours / 24, 1), 2),
            "status": "measured" if non_test_total else "sparse",
            "note": "FTR/RTR from status=done on non-test events; sparse if HANDOFF -OpsEvent unused",
        },
        "slip_ups": slip_ups,
        "ledger_path": RUN_EVENTS_REL,
    }


def scan_processes(*, max_rows: int = 40) -> dict[str, Any]:
    """Best-effort scan for engine-related OS processes (no psutil dependency)."""
    rows: list[dict[str, Any]] = []
    error: str | None = None
    try:
        if sys.platform == "win32":
            proc = subprocess.run(
                ["tasklist", "/FO", "CSV", "/NH"],
                capture_output=True,
                text=True,
                timeout=15,
                check=False,
            )
            raw_lines = (proc.stdout or "").splitlines()
            for line in raw_lines:
                if not line.strip():
                    continue
                parts = [p.strip('"') for p in re.findall(r'"[^"]*"|[^,]+', line)]
                if len(parts) < 2:
                    continue
                name, pid = parts[0].lower(), parts[1]
                if not any(h in name for h in _PROCESS_NAME_HINTS):
                    continue
                mem = parts[4] if len(parts) > 4 else ""
                rows.append({"name": parts[0], "pid": pid, "mem": mem})
        else:
            proc = subprocess.run(
                ["ps", "-eo", "pid,comm,rss"],
                capture_output=True,
                text=True,
                timeout=15,
                check=False,
            )
            for line in (proc.stdout or "").splitlines()[1:]:
                bits = line.split(None, 2)
                if len(bits) < 2:
                    continue
                pid, comm = bits[0], bits[1].lower()
                if not any(h in comm for h in _PROCESS_NAME_HINTS):
                    continue
                rows.append({"name": bits[1], "pid": pid, "mem": bits[2] if len(bits) > 2 else ""})
    except (OSError, subprocess.TimeoutExpired) as exc:
        error = str(exc)

    stale_hint = len([r for r in rows if "blender" in r.get("name", "").lower()]) > 2

    return {
        "task_id": "OPS-TEL-PROCESS-001",
        "schema": "ops_process_scan_v1",
        "ok": error is None,
        "error": error,
        "process_count": len(rows),
        "processes": rows[:max_rows],
        "slip_ups": (
            [
                {
                    "kind": "many_blender",
                    "severity": "warn",
                    "detail": "Multiple Blender processes — stale headless bakes?",
                }
            ]
            if stale_hint
            else []
        ),
        "platform": sys.platform,
    }


def scan_drift_instances() -> dict[str, Any]:
    """Collect drift / dishonesty signals from witnesses and index."""
    root = repo_root()
    instances: list[dict[str, Any]] = []

    vp = _load_json(root / VIEWPORT_DRIFT_REL)
    if vp:
        instances.append(
            {
                "kind": "viewport_authority",
                "path": VIEWPORT_DRIFT_REL,
                "status": vp.get("status"),
                "severity": "info" if vp.get("status") == "closed" else "alert",
                "detail": vp.get("witness") or "viewport drift witness",
            }
        )

    index = _load_json(root / UNIFIED_INDEX_REL) or {}
    proofs = index.get("proofs") if isinstance(index.get("proofs"), list) else []
    for proof in proofs:
        if not isinstance(proof, dict):
            continue
        summary = proof.get("summary") if isinstance(proof.get("summary"), dict) else {}
        gate = summary.get("honest_gate")
        if gate in ("dishonest_gate", "inflated_green", "rollup_inflated"):
            instances.append(
                {
                    "kind": "witness_honesty",
                    "path": proof.get("path") or proof.get("witness_path"),
                    "honest_gate": gate,
                    "program_id": proof.get("program_id"),
                    "severity": "alert" if gate == "dishonest_gate" else "warn",
                    "green": summary.get("green"),
                }
            )

    integrity = index.get("integrity_cache") if isinstance(index.get("integrity_cache"), dict) else {}
    q_contra = int(integrity.get("queue_contradiction_count") or 0)
    if q_contra:
        instances.append(
            {
                "kind": "queue_contradiction",
                "severity": "alert",
                "count": q_contra,
                "detail": "Queue rows contradict witness rollup",
            }
        )

    report = _load_json(root / OPS_REPORT_REL) or {}
    for row in report.get("delta_wf") or []:
        if not isinstance(row, dict):
            continue
        finding = str(row.get("finding") or "")
        if any(h in finding.lower() for h in ("blocked", "red", "dishonest", "open")):
            instances.append(
                {
                    "kind": "delta_wf",
                    "severity": "warn",
                    "finding": finding[:120],
                    "owner": row.get("owner"),
                    "program_id": row.get("program_id"),
                }
            )

    markers = _read_jsonl(AGENT_MARKERS_REL, limit=200)
    recent_markers = markers[-20:]
    for m in recent_markers:
        if m.get("delta_wf") or m.get("prior_writer"):
            instances.append(
                {
                    "kind": "agent_marker",
                    "severity": "info",
                    "agent": m.get("agent"),
                    "slice_id": m.get("slice_id"),
                    "delta_wf": m.get("delta_wf"),
                    "ts": m.get("ts"),
                }
            )

    alert_count = sum(1 for i in instances if i.get("severity") == "alert")
    warn_count = sum(1 for i in instances if i.get("severity") == "warn")

    return {
        "task_id": "OPS-TEL-DRIFT-001",
        "schema": "ops_drift_scan_v1",
        "ok": True,
        "instance_count": len(instances),
        "alert_count": alert_count,
        "warn_count": warn_count,
        "instances": instances[:40],
        "unified_index_path": UNIFIED_INDEX_REL,
    }


def build_ops_dashboard(*, window_hours: int = 168) -> dict[str, Any]:
    """Unified oversight bundle for agents, Grafana Infinity, and local HTML dashboard."""
    run_rollup = scan_run_events(window_hours=window_hours)
    processes = scan_processes()
    drift = scan_drift_instances()
    report = _load_json(repo_root() / OPS_REPORT_REL) or {}

    slip_ups: list[dict[str, Any]] = []
    slip_ups.extend(run_rollup.get("slip_ups") or [])
    slip_ups.extend(processes.get("slip_ups") or [])
    crash_slip: list[dict[str, Any]] = []
    crash_metrics: dict[str, Any] = {}
    try:
        from .ops_crash_exporter import run_crash_scan

        crash = run_crash_scan(record_events=False)
        crash_slip = crash.get("slip_ups") or []
        crash_metrics = {
            "crash_alert_count": crash.get("crash_alert_count"),
            "dcc_process_count": crash.get("dcc_process_count"),
            "glyph_summary": crash.get("glyph_summary"),
        }
        slip_ups.extend(crash_slip)
    except Exception:  # noqa: BLE001
        pass

    for inst in drift.get("instances") or []:
        if inst.get("severity") in ("alert", "warn"):
            slip_ups.append(
                {
                    "kind": inst.get("kind"),
                    "severity": inst.get("severity"),
                    "detail": inst.get("detail") or inst.get("finding") or inst.get("honest_gate"),
                    "path": inst.get("path"),
                }
            )

    severity_rank = {"alert": 0, "warn": 1, "info": 2}
    slip_ups.sort(key=lambda x: severity_rank.get(str(x.get("severity")), 9))

    qce = report.get("qce") if isinstance(report.get("qce"), dict) else {}
    tier1 = run_rollup.get("metrics_tier1") or {}
    tier1["drift_alerts"] = drift.get("alert_count")
    tier1["drift_warns"] = drift.get("warn_count")
    tier1["active_processes"] = processes.get("process_count")
    tier1.update(crash_metrics)

    return {
        "schema": "ops_dashboard_v1",
        "task_id": "OPS-DASHBOARD-001",
        "ok": True,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "run_id": str(uuid4()),
        "auth_spine": report.get("dsm_snapshot", [""])[0] if report.get("dsm_snapshot") else None,
        "qce": qce,
        "metrics_tier1": tier1,
        "run_events": run_rollup,
        "processes": processes,
        "drift": drift,
        "slip_ups": slip_ups[:24],
        "slip_up_count": len(slip_ups),
        "program_summary": report.get("program_summary"),
        "delta_wf": (report.get("delta_wf") or [])[:8],
        "sources": {
            "ops_report": OPS_REPORT_REL,
            "run_events": RUN_EVENTS_REL,
            "unified_index": UNIFIED_INDEX_REL,
            "triage_live": "debug_runs/agent_ops/triage_live.json",
            "prometheus": "debug_runs/agent_ops/prometheus/rust_engine_ops.prom",
        },
        "grafana": {
            "panel_hints": [
                "slip_ups[] — alert table",
                "metrics_tier1.ftr / rtr — stat panels",
                "processes.process_count — gauge",
                "drift.alert_count — gauge",
                "run_events.by_agent — bar chart",
                "metrics_tier1.crash_alert_count — DCC/crash gauge",
            ],
            "refresh_cli": "python -m rust_engine_mcp.cli ops-dashboard-refresh",
            "triage_dashboard": "tools/orchestrator/dashboard/grafana_triage_overview.json",
            "provision_path": "tools/orchestrator/dashboard/grafana_ops_overview.json",
        },
    }


def write_ops_dashboard_witness(*, window_hours: int = 168) -> dict[str, Any]:
    body = build_ops_dashboard(window_hours=window_hours)
    out = repo_root() / OPS_DASHBOARD_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    meta = {
        "schema": "ops_dashboard_v1",
        "written_at_epoch_secs": int(time.time()),
        "profile": "OPS_DASHBOARD",
        "source_system": "ops_telemetry",
        "relative_path": OPS_DASHBOARD_REL,
        "agent": "operations-intelligence",
    }
    body["_agent_meta"] = meta
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    body["written"] = OPS_DASHBOARD_REL
    return body
