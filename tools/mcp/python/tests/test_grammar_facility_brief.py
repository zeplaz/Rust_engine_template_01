"""CMCP-GRAMMAR-FACILITY-BRIEF-001 + DES-POWER-TIER-001 tests."""

from __future__ import annotations

import json

from rust_engine_mcp import grammar_facility_brief
from rust_engine_mcp.power_tier_bands import power_tier_from_units


def test_power_tier_from_chain_json() -> None:
    from rust_engine_mcp.paths import repo_root

    chains = json.loads(
        (repo_root() / "assets/configs/industrial_supply_chains.json").read_text(encoding="utf-8")
    )["chains"]
    expected = {
        ("concrete_portland", "concrete_cement_kiln"): "medium",
        ("concrete_portland", "concrete_mixer_plant"): "light",
        ("aluminum_primary", "aluminum_smelter1"): "heavy",
    }
    for chain_id, catalog_id, tier in (
        ("concrete_portland", "concrete_cement_kiln", "medium"),
        ("concrete_portland", "concrete_mixer_plant", "light"),
        ("aluminum_primary", "aluminum_smelter1", "heavy"),
    ):
        steps = chains[chain_id]["steps"]
        step = next(s for s in steps if s["catalog_id"] == catalog_id)
        assert power_tier_from_units(float(step["power_consumption"])) == tier
        assert expected[(chain_id, catalog_id)] == tier


def test_substation_is_grid_tier() -> None:
    assert power_tier_from_units(8, utility_role="substation") == "grid"


def test_grammar_facility_brief_rail_edge() -> None:
    body = grammar_facility_brief.grammar_facility_brief(grammar_id="rail_edge_v1")
    brief = body["brief"]
    assert brief is not None
    assert brief["green"] is True
    assert brief["catalog"]["catalog_id"] == "logistics_rail_warehouse"
    assert brief["chain"]["chain_id"] == "logistics_storage"
    assert brief["derived"]["power_tier_from_catalog"] == "light"


def test_grammar_facility_brief_industrial_warehouse() -> None:
    body = grammar_facility_brief.grammar_facility_brief(grammar_id="industrial_warehouse_v1")
    brief = body["brief"]
    assert brief is not None
    assert brief["green"] is True
    assert brief["catalog"]["catalog_id"] == "logistics_storage_warehouse"


def test_grammar_facility_brief_factory_cluster() -> None:
    body = grammar_facility_brief.grammar_facility_brief(grammar_id="factory_cluster_v1")
    brief = body["brief"]
    assert brief is not None
    assert brief["green"] is True
    assert brief["catalog"]["catalog_id"] == "concrete_mixer_plant"
    assert brief["chain"]["chain_id"] == "concrete_portland"
    assert brief["derived"]["power_tier_from_catalog"] == "light"
    assert brief["io_summary"]["produces_top3"] == ["Concrete"]


def test_grammar_facility_brief_inventory() -> None:
    body = grammar_facility_brief.grammar_facility_brief()
    assert body["grammar_count"] >= 3
    assert body["binding_count"] == 3
    assert body["green_binding_count"] == 3
    assert body["green"] is True


def test_write_grammar_facility_brief_witness() -> None:
    body = grammar_facility_brief.write_grammar_facility_brief_witness(grammar_id="factory_cluster_v1")
    assert body.get("written") == grammar_facility_brief.WITNESS_REL
    assert body.get("brief", {}).get("green") is True
