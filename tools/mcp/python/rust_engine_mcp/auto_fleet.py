"""Auto fleet drain brief — no-operator wave status."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

QUEUE_REL = "tools/orchestrator/queues/auto_fleet_drain_queue_v1.json"
INVENTORY_REL = "debug_runs/agent_ops/auto_fleet_inventory_live.json"


def auto_fleet_brief() -> dict[str, Any]:
    root = repo_root()
    qpath = root / QUEUE_REL
    if not qpath.is_file():
        return {"ok": False, "error": f"missing {QUEUE_REL}"}
    data = json.loads(qpath.read_text(encoding="utf-8"))
    waves = data.get("waves") or []
    counts = {"dispatch": 0, "done": 0, "queued": 0, "blocked": 0, "other": 0}
    by_agent: dict[str, list[str]] = {}
    for w in waves:
        for s in w.get("slices") or []:
            st = str(s.get("status") or "other")
            key = st if st in counts else "other"
            counts[key] = counts.get(key, 0) + 1
            ag = str(s.get("agent") or "?")
            by_agent.setdefault(ag, []).append(f"{s.get('id')}:{st}")
    inv = None
    ip = root / INVENTORY_REL
    if ip.is_file():
        inv = json.loads(ip.read_text(encoding="utf-8"))
    return {
        "schema": "auto_fleet_brief_v1",
        "ok": True,
        "queue": QUEUE_REL,
        "current_wave": (data.get("_meta") or {}).get("current_wave"),
        "operator_excluded": (data.get("_meta") or {}).get("operator_excluded"),
        "counts": counts,
        "by_agent": by_agent,
        "inventory_auto": (inv or {}).get("auto_count"),
        "inventory_blocked": (inv or {}).get("blocked_non_op_count"),
        "hint": "W1 parallel dispatch → W2 coder serial RPC-1-003..005 → W3 planner RPC-2",
    }
