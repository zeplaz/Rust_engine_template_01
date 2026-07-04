"""APSR-A2-P3-001 — material_browser single mount witness."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

TASK_ID = "APSR-A2-P3-001"
WITNESS_REL = "debug_runs/apsr_a2_p3_001_live.json"
BROWSER_REL = "tools/mcp/art_pipeline_suite/material_browser.py"
MOUNT_SITES = (
    "tools/mcp/art_pipeline_suite/assembly_panel_layout.py",
    "tools/mcp/art_pipeline_suite/materials_panel.py",
)


def write_apsr_a2_p3_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    root = repo or repo_root()
    browser_text = (root / BROWSER_REL).read_text(encoding="utf-8")
    factory_ok = "def mount_material_library" in browser_text and "MATERIAL_MOUNT_CONFIG" in browser_text
    sites = []
    for rel in MOUNT_SITES:
        text = (root / rel).read_text(encoding="utf-8")
        sites.append(
            {
                "file": rel,
                "uses_mount_factory": "mount_material_library" in text,
                "direct_widget_import": "MaterialLibraryWidget" in text,
            }
        )
    no_direct = all(not s["direct_widget_import"] for s in sites)
    all_mount = all(s["uses_mount_factory"] for s in sites)
    green = factory_ok and all_mount and no_direct
    body: dict[str, Any] = {
        "task_id": TASK_ID,
        "gate": TASK_ID,
        "green": green,
        "material_browser_module": BROWSER_REL,
        "mount_sites": sites,
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-P3",
    }
    return write_aps_live_witness(
        body,
        WITNESS_REL,
        schema="apsr_a2_p3_live_v1",
        profile="APSR_A2_P3",
        source_system="apsr_a2_p3",
        ritual=f"BLANG:WIT-HON {TASK_ID}" if green else None,
        repo=root,
    )
