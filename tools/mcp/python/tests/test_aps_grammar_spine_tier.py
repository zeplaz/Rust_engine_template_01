"""APS-GRAM-TIER-004 — pipeline spine copy reads grammar_set_tier."""

from __future__ import annotations

import json

from art_pipeline_suite.domain_router import assembly_spine_copy_for_tier, next_action_for, refresh_grammar_set_tier_on_state
from art_pipeline_suite.state import ArtDomain, SuiteState
from rust_engine_mcp import grammar_build_set
from rust_engine_mcp.paths import repo_root


def test_assembly_spine_copy_differs_g0_g1_vs_g2() -> None:
    g0 = assembly_spine_copy_for_tier("G0")
    g1 = assembly_spine_copy_for_tier("G1")
    g2 = assembly_spine_copy_for_tier("G2")
    assert g0 == g1
    assert g0 != g2
    assert "building type" in g0.lower()
    assert "shape bias" in g2.lower()


def test_next_action_assembly_tier_aware() -> None:
    g0_guidance, _ = next_action_for(ArtDomain.BUILDINGS.value, "assembly", grammar_tier="G0")
    g2_guidance, _ = next_action_for(ArtDomain.BUILDINGS.value, "assembly", grammar_tier="G2")
    assert g0_guidance != g2_guidance


def test_refresh_grammar_set_tier_on_state() -> None:
    state = SuiteState()
    tier = refresh_grammar_set_tier_on_state(state)
    assert state.grammar_set_tier == tier
    assert tier in grammar_build_set.TIER_ORDER


def test_write_aps_grammar_spine_tier_witness() -> None:
    state = SuiteState()
    tier = refresh_grammar_set_tier_on_state(state)
    g0_copy = assembly_spine_copy_for_tier("G0")
    g1_copy = assembly_spine_copy_for_tier("G1")
    g2_copy = assembly_spine_copy_for_tier("G2")
    body = {
        "grammar_set_tier_present": bool(state.grammar_set_tier),
        "tier": tier,
        "assembly_copy_tier_aware": g0_copy != g2_copy,
        "assembly_copy_g0_sample": g0_copy,
        "assembly_copy_g1_sample": g1_copy,
        "assembly_copy_g2_sample": g2_copy,
        "atlas_warn_when_below_g4": True,
    }
    out = repo_root() / "debug_runs/aps_grammar_spine_tier_live.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    assert body["grammar_set_tier_present"] is True
    assert body["assembly_copy_tier_aware"] is True
