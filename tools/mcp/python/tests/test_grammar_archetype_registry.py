"""GRAM-CONTENT-002 — archetype registry after G1 content drop."""

from rust_engine_mcp import building_grammar, grammar_build_set


def test_list_archetype_ids_g1_family() -> None:
    ids = building_grammar.list_archetype_ids()
    assert len(ids) >= 3
    for expected in ("IndustrialWarehouse", "FactoryCluster", "RailEdge"):
        assert expected in ids


def test_write_grammar_archetype_g1_witness() -> None:
    body = grammar_build_set.write_grammar_archetype_g1_witness()
    assert body["archetype_count"] >= 3
    assert body["ron_files_added"] >= 2
    assert body["json_mirrors_added"] >= 2


def test_grammar_set_tier_g1_after_content() -> None:
    body = grammar_build_set.grammar_set_tier()
    assert body["tier"] == "G1"
    assert body["archetype_count"] >= 3


def test_write_grammar_set_tier_g1_witness() -> None:
    body = grammar_build_set.write_grammar_set_tier_g1_witness()
    assert body["tier"] == "G1"
    assert body["kit_hint_downgraded"] is True
    assert len(body["archetype_ids"]) >= 3
