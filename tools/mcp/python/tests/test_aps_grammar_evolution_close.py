"""APS-GRAM-CLOSE-001 — program rollup witness."""

from rust_engine_mcp import grammar_build_set


def test_write_grammar_labels_g1_witness() -> None:
    body = grammar_build_set.write_grammar_labels_g1_witness()
    assert body["human_labels_for_new_archetypes"] is True
    assert body["archetype_labels"]["FactoryCluster"] == "Factory Cluster"


def test_write_aps_grammar_evolution_close_witness() -> None:
    grammar_build_set.write_grammar_labels_g1_witness()
    body = grammar_build_set.write_aps_grammar_evolution_close_witness(
        pytest_aps={"passed": 17, "failed": 0, "exit_code": 0},
    )
    assert body["status"] == "pass"
    assert body["tier"] in ("G1", "G2", "G3", "G4")
    assert body["rows_closed"] >= 8
    assert body["green"] is True
