#!/usr/bin/env python3
"""PLAN-LEDGER-REFRESH-005 — machine-state sync after planner wave 4."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

WAVE4_PLANS = [
    "PLAN-F7-A-EXIT-001",
    "PLAN-F7-B-STREAM-001",
    "PLAN-F7-C-LOD-001",
    "PLAN-CONSTRUCTION-MV-001",
    "PLAN-IND-E02-PLAY-001",
    "PLAN-LOG-E01-VISUAL-001",
    "PLAN-VISUAL-RUN-GATE-001",
    "PLAN-M3-MINMAP-001",
    "PLAN-REPLAY-PARITY-001",
    "PLAN-S7B-M4-SIM-001",
    "PLAN-PHASE-D-PARITY-001",
    "PLAN-LEDGER-REFRESH-005",
]


def load_json(rel: str) -> dict:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> None:
    infra = load_json("debug_runs/infrastructure_view_isolation_live.json")
    fire_stream = load_json("debug_runs/fire_streaming_live.json")
    construction = load_json("debug_runs/construction_stage_live.json")
    industrial = load_json("debug_runs/industrial_activation_live.json")
    minimap = load_json("debug_runs/minimap_compositor_live.json")
    replay = load_json("debug_runs/replay_editor_parity_live.json")
    stage7 = load_json("debug_runs/stage7_behavioral_live.json")
    stage5 = load_json("debug_runs/stage5_full_app_live.json")

    checks: list[dict] = []

    def add(witness: str, field: str, ok: bool, verdict: str, note: str = "") -> None:
        checks.append(
            {"witness": witness, "field": field, "ok": ok, "verdict": verdict, "note": note}
        )

    f7_exit = infra.get("fire7_f7_a_exit_001") or {}
    add(
        "infrastructure_view_isolation_live.json",
        "fire7_f7_a_exit_001.green",
        f7_exit.get("green") is True,
        "CURRENT",
        "PLAN-F7-A-EXIT-001",
    )
    add(
        "fire_streaming_live.json",
        "green",
        fire_stream.get("green") is True,
        "CURRENT",
        "PLAN-F7-B-STREAM-001",
    )
    mv = construction.get("construction_mv_001") or {}
    add(
        "construction_stage_live.json",
        "construction_mv_001.green",
        mv.get("green") is True,
        "CURRENT",
        "PLAN-CONSTRUCTION-MV-001",
    )
    chain = industrial.get("concrete_chain_e2e") or {}
    add(
        "industrial_activation_live.json",
        "concrete_chain_e2e.ind_e02_green",
        chain.get("ind_e02_green") is True,
        "CURRENT",
        "PLAN-IND-E02-PLAY-001",
    )
    log_rows = (stage5.get("projection_graph") or {}).get("logistics_active_rows", 0)
    add(
        "stage5_full_app_live.json",
        "projection_graph.logistics_active_rows",
        isinstance(log_rows, int) and log_rows > 0,
        "STALE" if log_rows == 0 else "CURRENT",
        "PLAN-LOG-E01-VISUAL-001",
    )
    add(
        "minimap_compositor_live.json",
        "ui_p3_m3_units_001_green",
        minimap.get("ui_p3_m3_units_001_green") is True,
        "CURRENT",
        "PLAN-M3-MINMAP-001",
    )
    add(
        "replay_editor_parity_live.json",
        "parity_green",
        replay.get("parity_green") is True,
        "CURRENT",
        "PLAN-REPLAY-PARITY-001",
    )
    m4 = stage7.get("s7b_m4_play_001") or {}
    add(
        "stage7_behavioral_live.json",
        "s7b_m4_play_001.green",
        m4.get("green") is True,
        "CURRENT",
        "PLAN-S7B-M4-SIM-001",
    )
    vm08 = infra.get("vm_08") or {}
    add(
        "infrastructure_view_isolation_live.json",
        "vm_08.overlay_masks_aligned",
        vm08.get("overlay_masks_aligned") is True,
        "CURRENT",
        "PLAN-PHASE-D-PARITY-001",
    )

    plan_docs = [
        "docs/archive/2026-06-src-dev/plans/fire7_f7_a_exit_acceptance_v1.md",
        "docs/archive/2026-06-src-dev/plans/fire7_f7_b_streaming_impl_plan_v1.md",
        "docs/archive/2026-06-src-dev/plans/fire7_f7_c_lod_impl_plan_v1.md",
        "docs/archive/2026-06-src-dev/plans/construction_multiview_sim_spec_v1.md",
        "docs/archive/2026-06-src-dev/plans/ind_e02_default_play_spec_v1.md",
        "docs/archive/2026-06-src-dev/plans/log_e01_visual_acceptance_v1.md",
        "docs/archive/2026-06-src-dev/plans/visual_run_acceptance_matrix_v1.md",
        "docs/archive/2026-06-src-dev/plans/minimap_m3_units_replay_impl_plan_v1.md",
        "docs/archive/2026-06-src-dev/plans/replay_editor_parity_impl_plan_v1.md",
        "docs/archive/2026-06-src-dev/plans/s7b_m4_sim_playtest_spec_v1.md",
        "docs/archive/2026-06-src-dev/plans/overlay_parity_stress_plan_v1.md",
        "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_005_checklist_v1.md",
    ]
    missing = [p for p in plan_docs if not (ROOT / p).is_file()]
    if missing:
        raise SystemExit(f"missing plan docs: {missing}")

    audit_path = ROOT / "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v7.md"
    print(f"audit: {audit_path.relative_to(ROOT)}")
    print(f"wave4 plans on disk: {len(plan_docs)}/{len(plan_docs)}")
    print(f"witness checks: {len(checks)}")
    stale = [c for c in checks if c["verdict"] == "STALE"]
    if stale:
        print(f"STALE ({len(stale)}): " + ", ".join(c["field"] for c in stale))
    failed = [c for c in checks if not c["ok"] and c["verdict"] != "STALE"]
    if failed:
        raise SystemExit(f"failed checks: {failed}")
    print("PLAN-LEDGER-REFRESH-005: OK")


if __name__ == "__main__":
    main()
