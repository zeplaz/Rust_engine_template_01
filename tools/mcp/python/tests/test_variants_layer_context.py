# APS variants layer context — unit tests

from __future__ import annotations

import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite.variants_layer_context import (  # noqa: E402
    build_layers_from_controls,
    compose_context_line,
    draft_is_dirty,
    merge_draft_into_entry,
)


def test_build_layers_from_controls_material_optional() -> None:
    layers = build_layers_from_controls(
        lighting="night_on",
        power="on",
        night_lights=True,
        damage_state="damaged",
        damage=0.4,
        fill="half",
        wall_material="",
    )
    assert layers["lighting"]["lighting"] == "night_on"
    assert "material" not in layers


def test_draft_is_dirty_when_lighting_changes() -> None:
    entry = {
        "variant_key": "clean_day",
        "layers": {
            "lighting": {"lighting": "day", "power": "off", "night_lights": False},
            "damage": {"state": "clean", "damage": 0.0},
            "fill": {"fill": "empty"},
        },
        "tags": ["default"],
    }
    draft = build_layers_from_controls(
        lighting="night_on",
        power="off",
        night_lights=False,
        damage_state="clean",
        damage=0.0,
        fill="empty",
        wall_material="",
    )
    assert draft_is_dirty(entry, draft, ["default"])


def test_merge_draft_into_entry_for_preview() -> None:
    entry = {"variant_key": "x", "layers": {"lighting": {"lighting": "day"}}}
    draft = build_layers_from_controls(
        lighting="night_off",
        power="partial",
        night_lights=False,
        damage_state="dirty",
        damage=0.1,
        fill="quarter",
        wall_material="brick_red",
    )
    merged = merge_draft_into_entry(entry, draft, ["sim_night"])
    assert merged["layers"]["lighting"]["lighting"] == "night_off"
    assert merged["tags"] == ["sim_night"]


def test_compose_context_line_focus_lighting() -> None:
    line = compose_context_line(
        lighting="night_on",
        power="on",
        night_lights=True,
        damage_state="clean",
        damage=0.0,
        fill="empty",
        wall_material="",
        focus="lighting",
    )
    assert "Night still" in line
