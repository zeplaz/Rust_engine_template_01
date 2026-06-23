"""APS P0-C — reaction-territory variant sessions + deterministic keys."""

from __future__ import annotations

import json
import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite.aps_preview_variant_state import (  # noqa: E402
    merge_variant_entry_layers,
    variant_entry_to_visual_state,
)
from rust_engine_mcp.paths import repo_root  # noqa: E402
from rust_engine_mcp.reaction_territory import (  # noqa: E402
    P0_EVENT_IDS,
    WITNESS_REL,
    load_reaction_catalog,
    reaction_key_hash,
    reaction_session_variant_key,
    refresh_reaction_territory_witness,
    resolve_reaction_territory_variant,
)
from rust_engine_mcp.variants_sessions import (  # noqa: E402
    SESSION_VARIANT_KEYS,
    build_variant_set_from_assembly,
)


def test_reaction_event_catalog_loads_three_p0_events() -> None:
    catalog = load_reaction_catalog()
    events = catalog.get("events") or {}
    for eid in P0_EVENT_IDS:
        assert eid in events
        assert events[eid].get("variant_keys")
        assert events[eid].get("metric_deltas")


def test_new_from_assembly_includes_base_and_reaction_sessions() -> None:
    data = build_variant_set_from_assembly(
        assembly_id="warehouse_industrial_west_production_v1",
        style_pack_id="style_industrial_west",
        seed=42,
    )
    keys = [v["variant_key"] for v in data["variants"]]
    assert len(keys) >= 4
    for key in SESSION_VARIANT_KEYS:
        assert key in keys
    reaction_rows = [v for v in data["variants"] if v.get("reaction_event_id")]
    assert len(reaction_rows) >= 4
    assert any(v.get("reaction_event_id") == "heritage_site_destruction" for v in reaction_rows)
    assert any(v.get("reaction_event_id") == "language_ban" for v in reaction_rows)


def test_reaction_keys_deterministic_for_assembly_event_seed() -> None:
    a = reaction_key_hash("shopfront_v1", "language_ban", 7)
    b = reaction_key_hash("shopfront_v1", "language_ban", 7)
    c = reaction_key_hash("shopfront_v1", "language_ban", 8)
    assert a == b
    assert a != c
    assert len(a) == 8
    vkey = reaction_session_variant_key("language_ban", "clean_night_off", a)
    assert vkey.startswith("language_ban__clean_night_off__")


def test_build_variant_set_reaction_keys_stable() -> None:
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


def test_reaction_preview_merge_heritage_burning() -> None:
    base = {
        "assembly_id": "warehouse_industrial_west_production_v1",
        "module_placements": [{"module_id": "wall_concrete_2u", "glb_path": "assets/models/modules/x/model.glb"}],
        "variants": {"seed": 42},
    }
    entry = {
        "variant_key": "heritage_site_destruction__burning_00__deadbeef",
        "reaction_event_id": "heritage_site_destruction",
        "reaction_key": "deadbeef",
        "tag_anchor": {"cell_x": 3, "cell_y": 2, "anchor_kind": "cell_center_v1"},
        "layers": {
            "lighting": {"lighting": "night_on", "power": "on", "night_lights": True},
            "damage": {"state": "damaged", "damage": 0.55},
        },
        "tags": ["sim_fire", "heritage_integrity"],
    }
    visual = variant_entry_to_visual_state(entry)
    assert visual == "burning"
    merged = merge_variant_entry_layers(base, entry, visual)
    assert merged["preview_variant_state"] == "burning"
    assert merged["reaction_event_id"] == "heritage_site_destruction"
    assert merged["tag_anchor"]["cell_x"] == 3
    assert merged["variants"]["damage"] == 0.55


def test_resolve_reaction_territory_variant_heritage_civic() -> None:
    body = resolve_reaction_territory_variant("heritage_site_destruction", "heritage_civic")
    assert body["concrete_variant_keys"]
    assert body["preview_visual_states"] == ["damaged", "burning", "clean"]
    assert "burn_origin" in str(body["tag_anchors"])


def test_full_catalog_session_count() -> None:
    data = build_variant_set_from_assembly(
        assembly_id="warehouse_industrial_west_production_v1",
        style_pack_id="style_industrial_west",
        seed=42,
        include_full_catalog=True,
    )
    reaction_rows = [v for v in data["variants"] if v.get("reaction_event_id")]
    assert len(reaction_rows) >= 8
    event_ids = {v.get("reaction_event_id") for v in reaction_rows}
    assert "forced_assimilation_in_schools" in event_ids
    assert "archive_seizure_or_censorship" in event_ids


def test_preview_visual_states_language_ban() -> None:
    from rust_engine_mcp.reaction_territory import preview_visual_states_for_catalog_states

    assert preview_visual_states_for_catalog_states(["night"]) == ["night"]
    assert preview_visual_states_for_catalog_states(["damaged", "night"]) == ["damaged", "night"]


def test_reaction_territory_witness_green() -> None:
    body = refresh_reaction_territory_witness()
    assert body["green"] is True
    assert body["cmcp_resolve_001_green"] is True
    assert body["cmcp_preview_001_green"] is True
    assert body["reaction_session_count_full_catalog"] >= 8
    path = repo_root() / WITNESS_REL
    assert path.is_file()
    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert loaded["gate"] == "APS-P0-REACTION-TERRITORY-001"
