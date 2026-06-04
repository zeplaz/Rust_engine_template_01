"""Agent queue MCP — drain logic."""

from __future__ import annotations

import json

import pytest

from rust_engine_mcp import agent_queue


def test_grammar_queue_next_planner_idle_when_lane_done() -> None:
    out = agent_queue.agent_queue_next("planner", queue="grammar")
    assert out["action"] in ("work", "idle")
    if out["action"] == "work":
        assert out["slice"]["id"].startswith("PLAN-")


def test_grammar_queue_coder_mcp_idle_after_material_browser() -> None:
    out = agent_queue.agent_queue_next("coder-mcp", queue="grammar")
    assert out["action"] == "idle"


def test_grammar_queue_coder_idle_after_pg_material() -> None:
    out = agent_queue.agent_queue_next("coder", queue="grammar")
    assert out["action"] == "idle"


def test_grammar_queue_pilot_blocked_on_material_authority() -> None:
    out = agent_queue.agent_queue_next("designer-mcp", queue="grammar")
    assert out["action"] in ("work", "idle")
    if out["action"] == "work":
        assert out["slice"]["id"] == "MCP-PILOT-GRAMMAR-001"
        assert out["slice"]["status"] == "ready"


def test_agent_queue_update_roundtrip(tmp_path, monkeypatch) -> None:
    qpath = tmp_path / "q.json"
    qpath.write_text(
        json.dumps(
            [{"id": "TEST-1", "agent": "planner", "priority": 1, "status": "ready"}]
        ),
        encoding="utf-8",
    )
    monkeypatch.setitem(agent_queue.QUEUE_REGISTRY, "test", "test/q.json")

    def _qpath(q: str) -> __import__("pathlib").Path:
        if q == "test":
            return qpath
        return agent_queue.queue_path(q)

    monkeypatch.setattr(agent_queue, "queue_path", _qpath)

    agent_queue.agent_queue_update("TEST-1", "done", note="pytest", queue="test")
    items = json.loads(qpath.read_text())
    assert items[0]["status"] == "done"
    assert items[0]["note"] == "pytest"


def test_token_savings_guide_keys() -> None:
    g = agent_queue.token_savings_guide()
    assert "validators" in g
    assert "never" in g
