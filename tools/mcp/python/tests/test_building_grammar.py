"""ARCH-BUILD-GRAMMAR-001/003 — grammar schema + generate parity."""

from __future__ import annotations

from pathlib import Path

import pytest

from rust_engine_mcp import assembly, building_grammar
from rust_engine_mcp.paths import schemas_dir
from rust_engine_mcp.schemas import load_json_file, validate_building_grammar


def test_building_grammar_schema_example() -> None:
    path = schemas_dir() / "examples" / "building_grammar_industrial_warehouse_v1.json"
    data = load_json_file(path)
    validate_building_grammar(data)
    assert data["archetype"]["id"] == "IndustrialWarehouse"


def test_generate_deterministic() -> None:
    a = building_grammar.generate("IndustrialWarehouse", "industrial_west", 99)
    b = building_grammar.generate("IndustrialWarehouse", "industrial_west", 99)
    assert a["width"] == b["width"]
    assert a["massing_strategy"] == b["massing_strategy"]
    assert a["style_pack_id"] == "style_industrial_west"


def test_assembly_snapshot_from_grammar() -> None:
    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=7,
        source_tier="lod0",
        write=False,
    )
    assert snap["style_pack_id"] == "style_industrial_west"
    assert snap["procedural_rules_version"] == building_grammar.GRAMMAR_RULES_VERSION
    tags = snap.get("reference_tags") or []
    assert any(t.startswith("grammar:") for t in tags)
    assert snap["module_placements"]


def test_grammar_massings_vary() -> None:
    strategies = {
        building_grammar.generate("IndustrialWarehouse", "industrial_west", s)["massing_strategy"]
        for s in range(48)
    }
    assert len(strategies) >= 2


def test_grammar_emits_material_profile_and_weathering() -> None:
    """PG-MATERIAL-GENERATION-001 — district material_profiles → placement authority."""
    result = building_grammar.generate("IndustrialWarehouse", "industrial_west", 43)
    assert result.get("material_profiles")
    assert result.get("weathering")
    snap = assembly.generate_assembly_snapshot(
        archetype_id="IndustrialWarehouse",
        district_style="industrial_west",
        seed=43,
        write=False,
    )
    for p in snap["module_placements"]:
        assert str(p.get("material_profile") or "").strip()
        assert str(p.get("weathering") or "").strip()
