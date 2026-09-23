"""Tests for agent_flow_route / agent_flow_policy."""

from __future__ import annotations

from rust_engine_mcp import agent_flow


def test_agent_flow_policy_tiers() -> None:
    p = agent_flow.agent_flow_policy()
    assert p["ok"] is True
    assert "L0_CHEAP" in p["tiers"]
    assert p["tiers"]["L0_CHEAP"]["model"] == agent_flow.MODEL_CHEAP
    assert p["tiers"]["L1_HARD"]["model"] == agent_flow.MODEL_HARD


def test_route_fire_skips_hard_without_triggers() -> None:
    out = agent_flow.agent_flow_route("refresh fire ecology witness sparks", domain="fire")
    assert out["ok"] is True
    assert out["domain"] == "fire"
    assert out["hard_gate"] is False
    assert out["phases"][1].get("skipped") is True
    assert out["phases"][2]["chat_agent"] == "@coder"


def test_route_hard_gate_on_authority() -> None:
    out = agent_flow.agent_flow_route(
        "resolve fire dual writer authority conflict on overlay",
        domain="auto",
    )
    assert out["hard_gate"] is True
    assert out["domain"] == "fire"
    hard = out["phases"][1]
    assert hard.get("skipped") is not True
    assert hard["model"] == agent_flow.MODEL_HARD
    assert hard["chat_agent"] == "@debug-intelligence"


def test_route_art_domain() -> None:
    out = agent_flow.agent_flow_route("Track B warehouse G4 keyframe ship", domain="auto")
    assert out["domain"] == "art"
    assert out["phases"][2]["chat_agent"] == "@coder-mcp"


def test_route_force_hard() -> None:
    out = agent_flow.agent_flow_route("simple ui tray tweak", domain="ui", force_hard=True)
    assert out["hard_gate"] is True


def test_token_savings_lists_flow() -> None:
    from rust_engine_mcp import agent_queue

    g = agent_queue.token_savings_guide()
    assert "agent_flow" in g["briefs"]
    assert "BLANG:FLOW" in g["blang"]["tokens"]
