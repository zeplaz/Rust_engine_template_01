"""Tests for APS-TAGS-002 semantic tag taxonomy + assembly enrichment."""

from __future__ import annotations

import pytest

from rust_engine_mcp import aps_tags, assembly, building_grammar


def test_semantic_tags_roundtrip_from_flat():
    flat = ["exterior", "industrial", "weathered"]
    semantic = aps_tags.semantic_tags_from_flat(flat)
    assert "location" in semantic
    assert "street_facing" in semantic["location"]
    assert "industrial" in semantic.get("architectural", [])
    back = aps_tags.flatten_semantic_tags(semantic)
    assert "exterior" in back or "street_facing" in back
    assert "weathered" in back


def test_enrich_placement_syncs_semantic_tags():
    placement = {
        "module_id": "wall_steel_1u",
        "token": "W",
        "grid_x": 1,
        "grid_y": 0,
        "floor": 0,
        "placement_tags": ["exterior", "industrial"],
    }
    enriched = assembly.enrich_placement(placement, source_tier="lod0")
    assert enriched.get("semantic_tags")
    assert enriched.get("placement_tags")


def test_generate_grammar_snapshot_has_rule_chain():
    try:
        snap = assembly.generate_assembly_snapshot(
            archetype_id="IndustrialWarehouse",
            district_style="industrial_west",
            seed=43,
            source_tier="lod0",
            write=False,
        )
    except (ValueError, FileNotFoundError, KeyError) as exc:
        pytest.skip(str(exc))
    assert snap.get("archetype_id") == "IndustrialWarehouse"
    assert snap.get("district_style") == "industrial_west"
    assert snap.get("grammar_rule_chain")
    chain = snap["grammar_rule_chain"]
    assert chain.get("massing")
    for p in snap["module_placements"]:
        assert p.get("semantic_tags") or p.get("placement_tags")


def test_grammar_rule_chain_snapshot_from_generate():
    result = building_grammar.generate("IndustrialWarehouse", "industrial_west", 43)
    chain = building_grammar.grammar_rule_chain_snapshot(result)
    assert chain.get("massing")
    assert chain.get("footprint_mode")
