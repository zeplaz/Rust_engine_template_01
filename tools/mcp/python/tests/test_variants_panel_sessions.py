"""APS P0-A — variant session rows + preview patch merge."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite.aps_preview_variant_state import (  # noqa: E402
    merge_variant_patch,
    variant_entry_to_visual_state,
)
from rust_engine_mcp.paths import repo_root  # noqa: E402
from rust_engine_mcp.variants_sessions import (  # noqa: E402
    SESSION_VARIANT_KEYS,
    WITNESS_REL,
    build_variant_set_from_assembly,
    refresh_variants_sessions_witness,
)


def test_new_from_assembly_yields_four_session_variants() -> None:
    data = build_variant_set_from_assembly(
        assembly_id="warehouse_industrial_west_production_v1",
        style_pack_id="style_industrial_west",
        seed=42,
    )
    keys = [v["variant_key"] for v in data["variants"]]
    assert len(keys) >= 4
    for key in SESSION_VARIANT_KEYS:
        assert key in keys
    assert len(keys) >= 8


def test_session_keys_deterministic_for_assembly_and_seed() -> None:
    a = build_variant_set_from_assembly(
        assembly_id="shopfront_v1",
        style_pack_id="style_victorian",
        seed=7,
    )
    b = build_variant_set_from_assembly(
        assembly_id="shopfront_v1",
        style_pack_id="style_victorian",
        seed=7,
    )
    keys_a = [v["variant_key"] for v in a["variants"]]
    keys_b = [v["variant_key"] for v in b["variants"]]
    assert keys_a == keys_b
    assert a["variant_set_id"] == b["variant_set_id"]


def test_preview_patch_merge_night_state() -> None:
    base = {
        "assembly_id": "warehouse_industrial_west_production_v1",
        "module_placements": [{"module_id": "wall_concrete_2u", "glb_path": "assets/models/modules/x/model.glb"}],
        "variants": {"seed": 42},
    }
    entry = {
        "variant_key": "clean_night_on",
        "layers": {
            "lighting": {"lighting": "night_on", "power": "on", "night_lights": True},
            "damage": {"state": "clean", "damage": 0.0},
        },
    }
    visual = variant_entry_to_visual_state(entry)
    assert visual == "night"
    merged = merge_variant_patch(base, visual)
    assert merged["preview_variant_state"] == "night"
    assert merged["variants"]["lighting"] == "night_on"
    assert merged["variants"]["night_lights"] is True
    assert merged["assembly_id"] == base["assembly_id"]


def test_variants_sessions_witness_green() -> None:
    body = refresh_variants_sessions_witness()
    assert body["green"] is True
    assert body["session_count"] >= 4
    path = repo_root() / WITNESS_REL
    assert path.is_file()
    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert loaded["gate"] == "APS-P0-VARIANTS-SESSIONS-001"


@pytest.mark.aps_gui
def test_variants_panel_has_preview_panel(aps_app) -> None:
    aps_app._apply_lane("buildings", log=False)
    assert hasattr(aps_app.variants, "_preview")
    assert aps_app.variants._preview.winfo_exists()
