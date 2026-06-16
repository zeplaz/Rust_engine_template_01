"""Grammar / building-set MCP tools (MCP-GRAMMAR-SET + MCP-BUILD-SET)."""

from rust_engine_mcp import grammar_build_set


def test_grammar_set_brief_four_grammar_pilots() -> None:
    body = grammar_build_set.grammar_set_brief()
    assert body["ok"] is True
    assert body["green"] is True
    assert body["counts"]["grammar_pilots"] >= 4
    assert body["counts"]["arch_dna_presets"] >= 4
    f_axis = set(body["f_axis_values"])
    assert {"logistics", "manufacturing", "power", "fuel"}.issubset(f_axis)


def test_grammar_preset_pair_validate_warehouse() -> None:
    body = grammar_build_set.grammar_preset_pair_validate(
        preset_id="logistics_rail_warehouse_v0"
    )
    assert body["green"] is True
    assert body["pilot_id"] == "logistics_rail_warehouse_v0"


def test_grammar_eval_sweep_histogram() -> None:
    body = grammar_build_set.grammar_eval_sweep(seeds=[42, 43, 44])
    assert body["ok"] is True
    assert body["seed_count"] == 3
    assert len(body["massing_histogram"]) >= 1


def test_grammar_pilot_parity_green() -> None:
    body = grammar_build_set.grammar_pilot_parity()
    assert body["green"] is True
    assert body["grammar_pilot_count"] >= 4


def test_building_set_coverage_green() -> None:
    body = grammar_build_set.building_set_coverage_report()
    assert body["green"] is True
    assert body["preset_count"] >= 4
    set_ids = {row["set_id"] for row in body["rows"]}
    assert "industrial_west_v0" in set_ids
    assert "infrastructure_v0" in set_ids


def test_building_set_health_brief() -> None:
    body = grammar_build_set.building_set_health_brief()
    assert body["green"] is True
    assert body["grammar_pilot_count"] >= 4


def test_building_set_manifest_validate_industrial_west() -> None:
    body = grammar_build_set.building_set_manifest_validate(
        path="tools/mcp/schemas/examples/building_set_industrial_west_v0.json"
    )
    assert body["green"] is True


def test_building_set_manifest_validate_infrastructure() -> None:
    body = grammar_build_set.building_set_manifest_validate(
        path="tools/mcp/schemas/examples/building_set_infrastructure_v0.json"
    )
    assert body["green"] is True
    assert body["set_id"] == "infrastructure_v0"
