"""APSR-A2-P1-001 — assembly_panel split witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

TASK_ID = "APSR-A2-P1-001"
WITNESS_REL = "debug_runs/apsr_a2_p1_001_live.json"
PANEL_REL = "tools/mcp/art_pipeline_suite/assembly_panel.py"
MAX_PANEL_LOC = 400
SECTION_MODULES = (
    "tools/mcp/art_pipeline_suite/assembly_grammar_section.py",
    "tools/mcp/art_pipeline_suite/assembly_validation_section.py",
    "tools/mcp/art_pipeline_suite/assembly_preview_section.py",
    "tools/mcp/art_pipeline_suite/assembly_metadata_section.py",
    "tools/mcp/art_pipeline_suite/assembly_footprint_section.py",
    "tools/mcp/art_pipeline_suite/assembly_panel_layout.py",
    "tools/mcp/art_pipeline_suite/assembly_panel_common.py",
)


def write_apsr_a2_p1_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    panel_path = root / PANEL_REL
    loc = len(panel_path.read_text(encoding="utf-8").splitlines())
    sections_present = all((root / rel).is_file() for rel in SECTION_MODULES)
    green = loc <= MAX_PANEL_LOC and sections_present
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "assembly_panel_loc": loc,
        "assembly_panel_loc_max": MAX_PANEL_LOC,
        "section_modules": list(SECTION_MODULES),
        "sections_present": sections_present,
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-P1",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_a2_p1_live_v1",
        profile="APSR_A2_P1",
        source_system="apsr_a2_p1",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
