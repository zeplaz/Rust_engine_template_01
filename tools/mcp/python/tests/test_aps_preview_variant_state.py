"""OVR-P55-PREVIEW-002 — variant visual state strip contract."""

from __future__ import annotations

import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite.aps_preview_variant_state import (  # noqa: E402
    VARIANT_STATES,
    merge_variant_patch,
    variant_axis_patch,
    variant_state_label,
)


def test_variant_states_canonical_four() -> None:
    assert VARIANT_STATES == ("clean", "night", "damaged", "burning")
    for state in VARIANT_STATES:
        assert variant_state_label(state)


def test_variant_axis_patch_burning_emissive() -> None:
    patch = variant_axis_patch("burning")
    assert patch.get("emissive_overlay") is True
    assert patch.get("damage_state") == "damaged"


def test_merge_variant_patch_preserves_base() -> None:
    base = {"module_id": "wall_brick_1u", "variants": {"seed": 42}}
    merged = merge_variant_patch(base, "night")
    assert merged["module_id"] == "wall_brick_1u"
    assert merged["variants"]["seed"] == 42
    assert merged["variants"]["lighting"] == "night_on"
    assert merged["preview_variant_state"] == "night"
