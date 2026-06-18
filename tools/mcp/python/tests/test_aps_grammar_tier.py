"""APS-GRAM-TIER-001 — grammar_set_tier() API + honest tier from registry."""

from rust_engine_mcp import grammar_build_set


def test_grammar_set_tier_g1_after_g1_content() -> None:
    body = grammar_build_set.grammar_set_tier()
    assert body["source"] == "grammar_set_tier()"
    assert body["tier"] == "G1"
    assert body["archetype_count"] >= 3
    assert "FactoryCluster" in body.get("grammar_files", []) or body["archetype_count"] >= 3


def test_write_grammar_set_tier_witness() -> None:
    body = grammar_build_set.write_grammar_set_tier_witness()
    assert body["tier"] == "G1"
    assert body["archetype_count"] >= 3
