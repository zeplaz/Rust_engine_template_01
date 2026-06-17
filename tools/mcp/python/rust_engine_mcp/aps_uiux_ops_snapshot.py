"""OPS-OVR-PROGRAM-SNAPSHOT-001 — compress overhaul queue into ops report."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/agent_ops/aps_uiux_overhaul_ops_snapshot_live.json"
QUEUE_REL = "tools/orchestrator/queues/aps_uiux_overhaul_queue.json"
PARALLEL_REL = "tools/orchestrator/queues/aps_uiux_overhaul_parallel_drain_v1.json"
DISPATCH_REL = "debug_runs/agent_ops/aps_uiux_overhaul_dispatch_live.json"
CLOSE_REL = "debug_runs/aps_uiux_overhaul_close_live.json"


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _summarize_rows(rows: list[dict[str, Any]]) -> dict[str, Any]:
    by_status: dict[str, list[str]] = {}
    for row in rows:
        status = str(row.get("status", "unknown"))
        by_status.setdefault(status, []).append(str(row.get("id", "")))
    done = len(by_status.get("done", []))
    total = len(rows)
    return {
        "done": done,
        "total": total,
        "percent": round(100 * done / total) if total else 0,
        "by_status": {k: len(v) for k, v in sorted(by_status.items())},
        "ready": by_status.get("ready", []),
        "blocked": by_status.get("blocked", []),
    }


def refresh_aps_uiux_ops_snapshot(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    queue = _load_json(root / QUEUE_REL)
    parallel = _load_json(root / PARALLEL_REL)
    main_rows = queue.get("drain", queue.get("rows", []))
    par_rows = parallel.get("parallel", [])
    main_sum = _summarize_rows(main_rows)
    par_sum = _summarize_rows(par_rows)
    close: dict[str, Any] = {}
    close_path = root / CLOSE_REL
    if close_path.is_file():
        close = _load_json(close_path)
    human_pending = close.get("human_gates_pending", [])
    body: dict[str, Any] = {
        "gate_id": "OPS-OVR-PROGRAM-SNAPSHOT-001",
        "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
        "program_status": queue.get("_meta", {}).get("program_status", "ACTIVE"),
        "snapshot_at": time.strftime("%Y-%m-%d"),
        "progress": {
            "main_queue": main_sum,
            "parallel_queue": par_sum,
            "phases_complete": close.get("phases_complete", []),
            "pytest_aps_ok": close.get("pytest_aps_ok"),
            "ban_list_clean": bool((close.get("ban_list") or {}).get("ui_clean")),
        },
        "blockers": {
            "human_gates_pending": human_pending,
            "ready_now": main_sum.get("ready", []) + par_sum.get("ready", []),
        },
        "routing": {
            "next_coder_mcp": "idle — P1–P6 machine phases done",
            "next_operator": "OVR-P6-OPERATOR-EYEBALL-001" if human_pending else None,
            "next_designer": "OVR-P6-DESIGN-SIGN-001" if "OVR-P6-DESIGN-SIGN-001" in human_pending else None,
            "next_designer_mcp": "DMCP-OVR-ARTIST-ACCEPT-001"
            if "DMCP-OVR-ARTIST-ACCEPT-001" in human_pending
            else None,
        },
        "sources": {
            "queue": QUEUE_REL,
            "parallel_queue": PARALLEL_REL,
            "dispatch": DISPATCH_REL,
            "close_witness": CLOSE_REL,
        },
        "_agent_meta": {
            "schema": "aps_uiux_overhaul_ops_snapshot_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "OPS_OVR_PROGRAM_SNAPSHOT",
            "relative_path": WITNESS_REL,
            "agent": "operations-intelligence",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
