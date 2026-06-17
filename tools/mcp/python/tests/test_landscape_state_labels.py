"""DES-APS-STATE-AXIS-LABELS-001 — v2 label table tests."""

from __future__ import annotations

import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite.landscape_state_labels import (
    REGROWTH_MACRO_ENUMS,
    SUCCESSION_STAGE_ENUMS,
    SUCCESSION_STAGE_ROWS,
    burn_frame_enum,
    burn_preview_label,
    combobox_enum_values,
    combobox_display_values,
    enum_from_ui_label,
    resolver_plain_label,
    ui_label_for_enum,
)


def test_succession_combobox_enum_display_split() -> None:
    enums = combobox_enum_values(SUCCESSION_STAGE_ROWS)
    labels = combobox_display_values(SUCCESSION_STAGE_ROWS)
    assert enums == list(SUCCESSION_STAGE_ENUMS)
    assert labels == [ui_label_for_enum(e) for e in enums]
    assert "Grass" not in labels
    assert "Pioneer grass" in labels


def test_enum_from_ui_label_roundtrip() -> None:
    for row in SUCCESSION_STAGE_ROWS:
        assert enum_from_ui_label(row.ui_label, rows=SUCCESSION_STAGE_ROWS) == row.enum
        assert enum_from_ui_label(row.enum, rows=SUCCESSION_STAGE_ROWS) == row.enum


def test_burn_preview_labels() -> None:
    assert burn_frame_enum(3) == "veg_burn_03"
    assert "Fire mid" in burn_preview_label(3)
    assert "Fire start" in burn_preview_label(0)
    assert "Fire end" in burn_preview_label(7)


def test_resolver_plain_label_templates() -> None:
    assert resolver_plain_label(
        {"variant_key": "topology_patch", "resolver": {"kind": "topology_kind", "topology_kind": "Patch"}}
    ) == "Patch topology sprite"
    assert resolver_plain_label(
        {"variant_key": "veg_burn_03", "resolver": {"kind": "active_burn_frame", "frame_index": 3}}
    ) == "Active fire · frame 3"
    assert "Regrowth" in resolver_plain_label(
        {
            "variant_key": "veg_regrowth_nuclei",
            "resolver": {"kind": "regrowth_macro", "regrowth_macro_phase": "Nuclei"},
        }
    )


def test_states_panel_uses_v2_labels() -> None:
    from art_pipeline_suite import landscape_state_labels
    from art_pipeline_suite.landscape_states_panel import LandscapeStatesPanel

    assert hasattr(LandscapeStatesPanel, "selected_burn_preview_enum")
    assert landscape_state_labels.SUCCESSION_STAGE_ROWS
