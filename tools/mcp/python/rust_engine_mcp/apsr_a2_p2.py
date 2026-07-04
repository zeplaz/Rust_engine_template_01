"""APSR-A2-P2-001 — shared preview_state_display witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

TASK_ID = "APSR-A2-P2-001"
WITNESS_REL = "debug_runs/apsr_a2_p2_001_live.json"
MODULE_REL = "tools/mcp/art_pipeline_suite/preview_state_display.py"
PANELS = (
    "tools/mcp/art_pipeline_suite/assembly_preview_panel.py",
    "tools/mcp/art_pipeline_suite/atlas_preview_panel.py",
    "tools/mcp/art_pipeline_suite/variants_preview_panel.py",
)


def write_apsr_a2_p2_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    module_ok = (root / MODULE_REL).is_file()
    consumers = []
    for rel in PANELS:
        text = (root / rel).read_text(encoding="utf-8")
        consumers.append({"file": rel, "uses_preview_state_display": "preview_state_display" in text})
    all_consume = all(c["uses_preview_state_display"] for c in consumers)
    green = module_ok and all_consume
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "preview_state_display_module": MODULE_REL,
        "panel_consumers": consumers,
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-P2",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_a2_p2_live_v1",
        profile="APSR_A2_P2",
        source_system="apsr_a2_p2",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
