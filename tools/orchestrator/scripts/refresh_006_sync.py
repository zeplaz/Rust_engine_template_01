#!/usr/bin/env python3
"""PLAN-LEDGER-REFRESH-006 — fleet truth sync after wave 3/4/5 closure."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

REQUIRED_PLANS = [
    "src/dev/planner_status_audit_v8.md",
    "src/dev/planner_delivery_signoff_matrix_v1.md",
    "src/dev/operator_visual_signoff_bundle_plan_v1.md",
    "src/dev/construction_round4_corridor_phase_spec_v1.md",
    "src/dev/construction_round4_multiview_ghost_presets_v1.md",
]


def load_json(rel: str) -> dict:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> None:
    missing = [p for p in REQUIRED_PLANS if not (ROOT / p).is_file()]
    if missing:
        raise SystemExit(f"missing plans: {missing}")

    cq = load_json("tools/orchestrator/queues/coder_active_queue.json")
    if cq.get("coder_a", {}).get("active") or cq.get("coder_b", {}).get("active"):
        raise SystemExit("coder_active_queue: active lanes must be empty for fleet-closed audit")

    s5 = load_json("debug_runs/stage5_full_app_live.json")
    readiness = (s5.get("readiness") or {}).get("passes")
    log_rows = (s5.get("projection_graph") or {}).get("logistics_active_rows", 0)
    vfx = s5.get("vfx_visual_signoff_001") or {}

    checks: list[tuple[str, bool, str]] = [
        ("readiness.passes", readiness is True, "stage5"),
        ("logistics_active_rows>0", isinstance(log_rows, int) and log_rows > 0, "LOG-E01 qualified"),
        ("vfx_visual_signoff_001.green", vfx.get("green") is True, "VFX qualified"),
        ("coder_a.active empty", not cq.get("coder_a", {}).get("active"), "fleet"),
        ("coder_b.active empty", not cq.get("coder_b", {}).get("active"), "fleet"),
    ]

    failed = [c for c in checks if not c[1]]
    if failed:
        raise SystemExit(f"failed: {failed}")

    audit = ROOT / "src/dev/planner_status_audit_v8.md"
    text = audit.read_text(encoding="utf-8")
    if "**SIGNED**" not in text or "QUEUED" in text.split("Status")[1][:80]:
        raise SystemExit("planner_status_audit_v8.md must be SIGNED (not QUEUED)")

    print("PLAN-LEDGER-REFRESH-006: OK")
    print(f"  audit: {audit.relative_to(ROOT)}")
    print(f"  stage5 readiness.passes={readiness} logistics_rows={log_rows}")
    print(f"  vfx green={vfx.get('green')} visual_run_pending={vfx.get('visual_run_pending')}")


if __name__ == "__main__":
    main()
