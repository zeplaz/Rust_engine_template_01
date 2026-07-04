"""APSR-D4 — density/polish witness (smoothness charter adoption)."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

CHARTER_REL = "src/dev/design_aps_smoothness_charter_v1.md"
THEME_REL = "tools/mcp/art_pipeline_suite/aps_theme.py"
SITES = (
    "tools/mcp/art_pipeline_suite/assembly_panel_layout.py",
    "tools/mcp/art_pipeline_suite/catalog.py",
    "tools/mcp/art_pipeline_suite/catalog_kit_coverage_strip.py",
    "tools/mcp/art_pipeline_suite/golden_seed_review_panel.py",
)


def density_polish_audit(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    theme_text = (root / THEME_REL).read_text(encoding="utf-8")
    has_min_banner = "VALIDATION_BANNER_MIN_PX" in theme_text
    sites = []
    for rel in SITES:
        text = (root / rel).read_text(encoding="utf-8") if (root / rel).is_file() else ""
        sites.append(
            {
                "file": rel,
                "uses_validation_min": "VALIDATION_BANNER_MIN_PX" in text,
            }
        )
    all_banner = all(s["uses_validation_min"] for s in sites if s["file"].endswith((".py",)))
    green = has_min_banner and all_banner and (root / CHARTER_REL).is_file()
    return {
        "charter": CHARTER_REL,
        "validation_banner_min_px": has_min_banner,
        "sites": sites,
        "green": green,
    }


def write_apsr_d4_witness(*, repo: Path | None = None) -> dict[str, Any]:
    from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness

    audit = density_polish_audit(repo=repo)
    body: dict[str, Any] = {
        "task_id": "APSR-A3-D4-001",
        "gate": "APSR-A3-D4-001",
        "green": audit["green"],
        "validation_banner_min_px": audit["validation_banner_min_px"],
        "sites": audit["sites"],
        "plan_ref": "src/dev/plan_aps_refactor_v1.md#APSR-D4",
    }
    return write_aps_live_witness(
        body,
        "debug_runs/apsr_a3_d4_001_live.json",
        schema="apsr_a3_d4_live_v1",
        profile="APSR_A3_D4",
        source_system="apsr_a3_d4",
        ritual="BLANG:WIT-HON APSR-A3-D4-001" if audit["green"] else None,
        repo=repo,
    )
