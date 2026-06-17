"""APS-EVO-E3-VEG-STATE-AXIS-001 — States tab axis + catalog validator witness."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from rust_engine_mcp.aps_witness_honesty import write_aps_live_witness
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.veg_catalog_loader import (
    burn_variant_keys,
    catalog_axis_summary,
    catalog_validator_report,
    state_axis_rows,
)

APS_VEG_STATE_AXIS_WITNESS = "debug_runs/aps_veg_state_axis_live.json"
DESIGN_LABELS_REF = "src/dev/design_aps_veg_state_labels_v1.md"
DESIGN_MATRIX_REF = "src/dev/design_landscape_lg5_expansion_matrix_v1.md"
MIN_BURN_VARIANTS = 8
MIN_CATALOG_ROWS = 16


def _ensure_aps_suite_path() -> None:
    suite_root = repo_root() / "tools/mcp"
    if str(suite_root) not in sys.path:
        sys.path.insert(0, str(suite_root))


def _verify_v2_labels_wired() -> bool:
    _ensure_aps_suite_path()
    try:
        from art_pipeline_suite.landscape_state_labels import (
            REGROWTH_MACRO_ENUMS,
            REGROWTH_MACRO_ROWS,
            SUCCESSION_STAGE_ENUMS,
            SUCCESSION_STAGE_ROWS,
            combobox_display_values,
            combobox_enum_values,
            resolver_plain_label,
        )
        from art_pipeline_suite.landscape_states_panel import LandscapeStatesPanel
    except ImportError:
        return False
    succession_ok = combobox_enum_values(SUCCESSION_STAGE_ROWS) == list(SUCCESSION_STAGE_ENUMS)
    regrowth_ok = combobox_enum_values(REGROWTH_MACRO_ROWS) == list(REGROWTH_MACRO_ENUMS)
    labels_ok = (
        len(combobox_display_values(SUCCESSION_STAGE_ROWS)) == len(SUCCESSION_STAGE_ENUMS)
        and "Pioneer grass" in combobox_display_values(SUCCESSION_STAGE_ROWS)
    )
    panel_ok = hasattr(LandscapeStatesPanel, "selected_burn_preview_enum") and hasattr(
        LandscapeStatesPanel, "mark_states_ready"
    )
    resolver_ok = "topology sprite" in resolver_plain_label(
        {"variant_key": "topology_patch", "resolver": {"kind": "topology_kind", "topology_kind": "Patch"}}
    )
    return succession_ok and regrowth_ok and labels_ok and panel_ok and resolver_ok


def verify_veg_state_axis(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    validator = catalog_validator_report(repo=root)
    axis = catalog_axis_summary(repo=root)
    burns = burn_variant_keys(repo=root)
    rows = state_axis_rows(repo=root)
    burn_count = len(burns)
    burn_variants_ok = burn_count >= MIN_BURN_VARIANTS
    catalog_rows_ok = len(rows) >= MIN_CATALOG_ROWS
    v2_labels_wired = _verify_v2_labels_wired()
    _ensure_aps_suite_path()
    panel_wired = False
    try:
        from art_pipeline_suite.landscape_states_panel import LandscapeStatesPanel

        panel_wired = hasattr(LandscapeStatesPanel, "selected_burn_preview_enum") and v2_labels_wired
    except ImportError:
        panel_wired = False
    return {
        "slice_id": "APS-EVO-E3-VEG-STATE-AXIS-001",
        "catalog_validator_green": bool(validator.get("green")),
        "catalog_validator": validator,
        "burn_variants_authored": burn_count,
        "burn_variants_ok": burn_variants_ok,
        "burn_frame_count": axis.get("burn_frame_count"),
        "succession_stages": axis.get("succession_stages"),
        "regrowth_macro_phases": axis.get("regrowth_macro_phases"),
        "state_axis_row_count": len(rows),
        "catalog_rows_ok": catalog_rows_ok,
        "topology_state_rows": axis.get("topology_rows"),
        "states_panel_catalog_wired": panel_wired,
        "v2_labels_wired": v2_labels_wired,
        "catalog_path": validator.get("catalog_path"),
        "design_labels_ref": DESIGN_LABELS_REF,
        "design_matrix_ref": DESIGN_MATRIX_REF,
    }


def refresh_aps_veg_state_axis_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    check = verify_veg_state_axis(repo=root)
    green = bool(
        check.get("catalog_validator_green")
        and check.get("burn_variants_ok")
        and check.get("catalog_rows_ok")
        and check.get("states_panel_catalog_wired")
        and check.get("v2_labels_wired")
    )
    body: dict[str, Any] = {
        "gate": "APS-EVO-E3-VEG-STATE-AXIS-001",
        "program_id": "APS-E3",
        "green": green,
        **check,
    }
    return write_aps_live_witness(
        body,
        APS_VEG_STATE_AXIS_WITNESS,
        schema="aps_veg_state_axis_live_v1",
        profile="APS_E3_VEG_STATE_AXIS",
        source_system="aps_veg_state_axis",
        ritual="BLANG:WIT-HON APS-EVO-E3-VEG-STATE-AXIS-001" if green else None,
        exit_predicate_must=[
            {"path": "catalog_validator_green", "eq": True},
            {"path": "burn_variants_ok", "eq": True},
            {"path": "catalog_rows_ok", "eq": True},
            {"path": "states_panel_catalog_wired", "eq": True},
            {"path": "v2_labels_wired", "eq": True},
        ],
        repo=root,
    )
