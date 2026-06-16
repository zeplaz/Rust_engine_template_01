#!/usr/bin/env python3
"""Sync vegetation v3 queue rows after harness + witness green."""
from __future__ import annotations

import json
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
QUEUES = ROOT / "tools" / "orchestrator" / "queues"
TODAY = date.today().isoformat()

DONE_VEG_MASTER = {
    "VEG-A01-HARNESS-001",
    "VEG-A06-FIRE-WITNESS-001",
    "VEG-B-ROLLOUT-WITNESS-001",
    "VEG-C04-PREVIEW-WITNESS-001",
    "VEG-DRAIN-CONTINUE",
    "VEG-LG2-LIVE-FIRE-001",
}

BLOCKED_IDS = {
    "VEG-DESIGN-ATLAS-001",
    "VEG-MCP-ATLAS-001",
    "VEG-OPERATOR-CHECKLIST-001",
    "VEG-LG6-FLOWERS-001",
    "G-PLAY-01",
    "PLAN-AUDIT-020",
    "VEG-OPERATOR-HISTORY-001",
}

DEFERRED_IDS = {"SIM-STEWARD-FIRE-REGRESS-001", "VEG-STEWARD-REGRESS-001"}


def patch_row(row: dict, *, mark_all_ready_done: bool = False) -> bool:
    rid = row.get("id") or row.get("task_id")
    if not rid:
        return False
    if rid in BLOCKED_IDS:
        if row.get("status") != "blocked":
            row["status"] = "blocked"
            row.setdefault("snag", "operator/designer-mcp blocked")
            return True
        return False
    if rid in DEFERRED_IDS:
        if row.get("status") != "deferred":
            row["status"] = "deferred"
            return True
        return False
    if rid in DONE_VEG_MASTER or (mark_all_ready_done and row.get("status") == "ready"):
        if row.get("status") != "done":
            row["status"] = "done"
            row["completed"] = TODAY
            row.pop("snag", None)
            return True
    return False


def patch_rows(doc: dict, *, list_key: str | None = None, mark_all_ready_done: bool = False) -> int:
    n = 0
    if list_key:
        rows = doc.get(list_key, [])
    else:
        rows = doc
    if not isinstance(rows, list):
        return 0
    for row in rows:
        if isinstance(row, dict) and patch_row(row, mark_all_ready_done=mark_all_ready_done):
            n += 1
    return n


def main() -> None:
    total = 0
    master = json.loads((QUEUES / "coder_master_drain_queue.json").read_text(encoding="utf-8"))
    total += patch_rows(master, list_key="rows")
    (QUEUES / "coder_master_drain_queue.json").write_text(
        json.dumps(master, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    veg = json.loads((QUEUES / "coder_vegetation_drain_queue.json").read_text(encoding="utf-8"))
    if "drain" in veg:
        total += patch_rows(veg, list_key="drain", mark_all_ready_done=True)
    for phase in veg.get("phases", {}).values():
        if isinstance(phase, dict) and "rows" in phase:
            total += patch_rows(phase, list_key="rows")
    if "rows" in veg:
        total += patch_rows(veg, list_key="rows")
    veg.setdefault("_meta", {})["last_sync"] = TODAY
    veg["_meta"]["harness_witness"] = "debug_runs/landscape_grammar_sim_harness_live.json"
    (QUEUES / "coder_vegetation_drain_queue.json").write_text(
        json.dumps(veg, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    active = json.loads((QUEUES / "coder_active_queue.json").read_text(encoding="utf-8"))
    for lane in ("coder_a", "coder_b"):
        block = active.get(lane, {})
        for key in ("active", "done", "blocked"):
            rows = block.get(key, [])
            if isinstance(rows, list):
                for row in rows:
                    if isinstance(row, dict) and row.get("id") in DONE_VEG_MASTER | {"VEG-LG2-LIVE-FIRE-001"}:
                        row["status"] = "done"
                        row["completed"] = TODAY
                        total += 1
    (QUEUES / "coder_active_queue.json").write_text(
        json.dumps(active, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    print(f"synced {total} queue row updates")


if __name__ == "__main__":
    main()
