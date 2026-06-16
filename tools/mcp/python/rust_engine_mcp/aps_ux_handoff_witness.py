"""APS UX handoff witnesses — tooltips merge, atlas legend, materials IA."""

from __future__ import annotations

import json
import re
from pathlib import Path

from .aps_atlas_qc import write_aps_atlas_preview_witness
from .paths import repo_root

APS_ROOT = repo_root() / "tools/mcp/art_pipeline_suite"
TOOLTIPS_WITNESS = "debug_runs/aps_ux_tooltips_002_live.json"
ATLAS_LEGEND_WITNESS = "debug_runs/aps_atlas_legend_001_live.json"
MAT_IA_WITNESS = "debug_runs/aps_mat_ia_001_live.json"

_EXPECTED_TOOLTIP_KEYS = 78
_NEW_TOOLTIP_KEYS = (
    "pipeline_catalog",
    "cat_batch_filter",
    "asm_archetype",
    "mat_use_in_assembly",
    "var_load",
    "atl_validate",
)


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _bind_count(source: str) -> int:
    return len(re.findall(r"bind_aps_tooltip\s*\(", source))


def refresh_aps_ux_tooltips_witness() -> bool:
    from art_pipeline_suite import aps_tooltips  # type: ignore[import-not-found]

    tooltips_path = APS_ROOT / "aps_tooltips.py"
    src = _read(tooltips_path)
    key_count = len(aps_tooltips.TOOLTIPS)
    bind_total = sum(_bind_count(_read(p)) for p in APS_ROOT.glob("*.py"))
    missing_new = [k for k in _NEW_TOOLTIP_KEYS if k not in aps_tooltips.TOOLTIPS]
    panel_src = _read(APS_ROOT / "pipeline_status_bar.py")
    per_step = 'f"pipeline_{key}"' in panel_src or "pipeline_catalog" in panel_src
    green = key_count >= 70 and not missing_new and per_step and bind_total >= 60
    payload = {
        "program_id": "APS-UX-TOOLTIPS-002",
        "gate": "APS-UX-TOOLTIPS-002-MERGE",
        "green": green,
        "tooltip_key_count": key_count,
        "expected_keys": _EXPECTED_TOOLTIP_KEYS,
        "bind_call_count": bind_total,
        "per_step_pipeline_tooltips": per_step,
        "missing_sample_keys": missing_new,
    }
    out = repo_root() / TOOLTIPS_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_aps_atlas_legend_witness() -> bool:
    preview_src = _read(APS_ROOT / "atlas_preview_panel.py")
    panel_src = _read(APS_ROOT / "atlas_panel.py")
    qc_src = _read(repo_root() / "tools/mcp/python/rust_engine_mcp/aps_atlas_qc.py")
    has_legend = "Legend: Grid lines = UV cells" in preview_src
    has_unavailable = "UV overlay unavailable" in preview_src
    has_format = "format_atlas_qc_display" in qc_src and "format_atlas_qc_display" in panel_src
    atlas_witness = write_aps_atlas_preview_witness()
    atlas_body = json.loads(atlas_witness.read_text(encoding="utf-8"))
    green = has_legend and has_unavailable and has_format and bool(atlas_body.get("uv_grid_overlay"))
    payload = {
        "program_id": "APS-ATLAS-LEGEND-001",
        "gate": "APS-ATLAS-LEGEND-001-IMPL",
        "green": green,
        "legend_line": has_legend,
        "plain_validate": has_format,
        "uv_grid_overlay": atlas_body.get("uv_grid_overlay"),
        "pilot_validation": atlas_body.get("validation_status"),
    }
    out = repo_root() / ATLAS_LEGEND_WITNESS
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_aps_mat_ia_witness() -> bool:
    mat_src = _read(APS_ROOT / "material_library_widget.py")
    panel_src = _read(APS_ROOT / "materials_panel.py")
    has_cta = 'text="Use in Assembly"' in mat_src and "side=tk.RIGHT" in mat_src
    no_apply_studio = 'mode="studio"' in panel_src and "Apply to selected slot" not in mat_src.split("studio")[0]
    has_tree = "Categories" in mat_src and "_render_tree_and_list" in mat_src
    row_copy = "entry.profile_id" in mat_src and "entry.category" in mat_src
    green = has_cta and has_tree and row_copy
    payload = {
        "program_id": "APS-MAT-IA-001",
        "gate": "APS-MAT-IA-001-IMPL",
        "green": green,
        "use_in_assembly_primary_cta": has_cta,
        "category_tree": has_tree,
        "profile_row_status_text": row_copy,
        "studio_mode_no_apply": no_apply_studio,
    }
    out = repo_root() / MAT_IA_WITNESS
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_aps_validation_colors_witness() -> bool:
    asm_src = _read(APS_ROOT / "assembly_panel.py")
    cat_src = _read(APS_ROOT / "catalog.py")
    var_src = _read(APS_ROOT / "variants_panel.py")
    green = (
        "_set_validation_result" in asm_src
        and "_set_validation_result" in cat_src
        and "_set_bake_status" in var_src
    )
    payload = {
        "program_id": "APS-UX-POLISH-001",
        "gate": "validation_fail_not_green",
        "green": green,
        "assembly_panel": "_set_validation_result" in asm_src,
        "catalog_panel": "_set_validation_result" in cat_src,
        "variants_panel": "_set_bake_status" in var_src,
    }
    out = repo_root() / "debug_runs/aps_validation_colors_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green


def refresh_all_aps_ux_handoff_witnesses() -> dict[str, bool]:
    return {
        "tooltips": refresh_aps_ux_tooltips_witness(),
        "atlas_legend": refresh_aps_atlas_legend_witness(),
        "mat_ia": refresh_aps_mat_ia_witness(),
        "validation_colors": refresh_aps_validation_colors_witness(),
    }
