#!/usr/bin/env python3
"""PLAN-LEDGER-REFRESH-007 — fleet truth sync after planner P1 prep + queue drain."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

REQUIRED_PLANS = [
    "src/dev/planner_status_audit_v9.md",
    "src/dev/plan_construction_hydro_coupling_001_v1.md",
    "src/dev/plan_wss_slab_pr3_exec_001_v1.md",
    "src/dev/plan_ledger_refresh_007_checklist_v1.md",
]


def load_json(rel: str) -> dict:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> None:
    missing = [p for p in REQUIRED_PLANS if not (ROOT / p).is_file()]
    if missing:
        raise SystemExit(f"missing plans: {missing}")

    pq = load_json("tools/orchestrator/queues/planner_active_queue.json")
    if pq.get("active"):
        raise SystemExit("planner_active_queue: active must be empty after drain")

    wss = load_json("debug_runs/wss_substrate_live.json")
    construction = load_json("debug_runs/construction_stage_live.json")

    slab_green = wss.get("green") is True and wss.get("gate") == "WSS-CHUNK-SLAB-001"
    param = construction.get("construction_parametric_placement_001") or {}
    r4_corridor = construction.get("construction_r4_corridor_001") or {}
    r4_mv = construction.get("construction_r4_mv_ghost_001") or {}

    checks: list[tuple[str, bool, str]] = [
        ("wss_chunk_slab_001.green", slab_green, "substrate"),
        ("construction_parametric_placement_001.green", param.get("green") is True, "parametric"),
        ("construction_r4_corridor_001.green", r4_corridor.get("green") is True, "R4 corridor"),
        ("construction_r4_mv_ghost_001.green", r4_mv.get("green") is True, "R4 MV"),
        ("planner active empty", not pq.get("active"), "planner"),
    ]

    failed = [c for c in checks if not c[1]]
    if failed:
        raise SystemExit(f"failed: {failed}")

    audit = ROOT / "src/dev/planner_status_audit_v9.md"
    text = audit.read_text(encoding="utf-8")
    if "**SIGNED**" not in text:
        raise SystemExit("planner_status_audit_v9.md must be SIGNED")

    archive_ids = {e.get("id") for e in pq.get("wave6_archive", [])}
    for required in (
        "PLAN-CONSTRUCTION-HYDRO-COUPLING-001",
        "PLAN-WSS-SLAB-PR-3-EXEC-001",
        "PLAN-LEDGER-REFRESH-007",
    ):
        if required not in archive_ids:
            raise SystemExit(f"missing wave6_archive entry: {required}")

    print("PLAN-LEDGER-REFRESH-007: OK")
    print(f"  audit: {audit.relative_to(ROOT)}")
    print(f"  wss slab green={slab_green}")
    print(f"  parametric green={param.get('green')}")
    print(f"  r4 corridor green={r4_corridor.get('green')} r4 mv green={r4_mv.get('green')}")


if __name__ == "__main__":
    main()
