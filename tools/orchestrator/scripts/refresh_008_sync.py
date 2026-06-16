#!/usr/bin/env python3
"""PLAN-LEDGER-REFRESH-008 — wave6_archive witness reconcile."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

REQUIRED = [
    "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v10.md",
    "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_008_checklist_v1.md",
    "docs/archive/2026-06-src-dev/plans/plan_elemental_wave2_index_001_v1.md",
    "docs/archive/2026-06-src-dev/plans/plan_wss_hybrid_retire_pr4_001_v1.md",
    "docs/archive/2026-06-src-dev/plans/plan_bq128_apply_exec_001_v1.md",
]


def load_json(rel: str) -> dict:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> None:
    missing = [p for p in REQUIRED if not (ROOT / p).is_file()]
    if missing:
        raise SystemExit(f"missing: {missing}")

    pq = load_json("tools/orchestrator/queues/planner_active_queue.json")
    if pq.get("active"):
        raise SystemExit("planner active must be empty")

    wss = load_json("debug_runs/wss_substrate_live.json")
    construction = load_json("debug_runs/construction_stage_live.json")
    minimap = load_json("debug_runs/minimap_compositor_live.json")
    replay = load_json("debug_runs/replay_editor_parity_live.json")
    stage5 = load_json("debug_runs/stage5_full_app_live.json")
    wave_s = load_json("debug_runs/wave_s_blueprint_roundtrip.json")

    hydro = (wss.get("wss_hydro_runtime_001") or {})
    checks: list[tuple[str, bool]] = [
        ("wss.green", wss.get("green") is True),
        ("dual_write_shim_enabled", wss.get("dual_write_shim_enabled") is True),
        ("active_runtime_wired", wss.get("active_runtime_wired") is True),
        ("construction_hydro_coupling_wired", hydro.get("construction_hydro_coupling_wired") is True),
        ("parametric.green", (construction.get("construction_parametric_placement_001") or {}).get("green") is True),
        ("r4_corridor.green", (construction.get("construction_r4_corridor_001") or {}).get("green") is True),
        ("r4_mv.green", (construction.get("construction_r4_mv_ghost_001") or {}).get("green") is True),
        ("m3_units.green", minimap.get("ui_p3_m3_units_001_green") is True),
        ("m3_replay.green", minimap.get("ui_p3_m3_replay_001_green") is True),
        ("parity_green", replay.get("parity_green") is True),
        ("replay_ring_len>=2", (replay.get("replay_ring_len") or 0) >= 2),
        ("stage5.passes", (stage5.get("readiness") or {}).get("passes") is True),
        ("wave_s.roundtrip_ok", wave_s.get("roundtrip_ok") is True),
        ("planner active empty", not pq.get("active")),
    ]

    failed = [c for c in checks if not c[1]]
    if failed:
        raise SystemExit(f"witness reconcile failed: {failed}")

    audit = (ROOT / "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v10.md").read_text(encoding="utf-8")
    if "**SIGNED**" not in audit:
        raise SystemExit("audit v10 must be SIGNED")

    archive_ids = {e.get("id") for e in pq.get("wave6_archive", [])}
    for req in (
        "PLAN-LEDGER-REFRESH-008",
        "PLAN-ELEMENTAL-WAVE2-INDEX-001",
        "PLAN-WSS-HYBRID-RETIRE-PR4-001",
        "PLAN-BQ128-APPLY-EXEC-001",
    ):
        if req not in archive_ids:
            raise SystemExit(f"missing archive: {req}")

    print("PLAN-LEDGER-REFRESH-008: OK")
    print(f"  closed witness bundle: {len([c for c in checks if c[1]])}/{len(checks)} checks")


if __name__ == "__main__":
    main()
