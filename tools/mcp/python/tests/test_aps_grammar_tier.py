"""APS-GRAM-TIER-001 — grammar_set_tier() API + honest tier from registry."""

from rust_engine_mcp import grammar_build_set

_VALID_TIERS = frozenset({"G0", "G1", "G2", "G3", "G4"})


def test_grammar_set_tier_at_least_g1() -> None:
    body = grammar_build_set.grammar_set_tier()
    assert body["source"] == "grammar_set_tier()"
    assert body["tier"] in _VALID_TIERS
    assert body["tier"] != "G0"
    assert body["archetype_count"] >= 3
    assert "FactoryCluster" in body.get("grammar_files", []) or body["archetype_count"] >= 3


def test_write_grammar_set_tier_witness() -> None:
    body = grammar_build_set.write_grammar_set_tier_witness()
    assert body["tier"] in _VALID_TIERS
    assert body["tier"] != "G0"
    assert body["archetype_count"] >= 3
