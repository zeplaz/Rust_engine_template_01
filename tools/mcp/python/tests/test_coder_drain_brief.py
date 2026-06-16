"""MCP-CODER-DRAIN-001 — coder_drain_brief stale dispatch detection."""

from __future__ import annotations

from rust_engine_mcp import agent_queue


def test_coder_drain_brief_c_stale_witness_closed():
    body = agent_queue.coder_drain_brief("c")
    assert body["ok"] is True
    assert body["schema"] == "coder_drain_brief_v1"
    assert body["recommend_next"] is None or body["recommend_next"] == "WEATHER-REGIONAL-001"
    if body["program_next"]:
        assert "WEATHER-REGIONAL-001" in body["program_next"][0]


def test_simulation_queue_brief():
    body = agent_queue.simulation_queue_brief()
    assert body["ok"] is True
    assert body["recommend_next"] is None or body["recommend_next"] == "WEATHER-REGIONAL-001"
    assert "WEATHER-CLIMATE-001" in body["done"]


def test_coder_mcp_drain_brief():
    body = agent_queue.coder_mcp_drain_brief()
    assert body["ok"] is True
    assert body["schema"] == "coder_mcp_drain_brief_v1"
    ready_ids = {r["id"] for r in body.get("ready") or []}
    assert "MCP-P2-RUN-EVENT-001" not in ready_ids
    assert "MCP-P2-HONEST-BAKE-001" not in ready_ids
    assert len(body["drain_todos"]) >= 1
    deferred = {t["id"] for t in body["drain_todos"] if t["status"] == "deferred"}
    assert "MCP-OPS-REPORT-001" in deferred



def test_coder_drain_brief_a_horizon_infra():
    body = agent_queue.coder_drain_brief("a")
    assert body["ok"] is True
    assert body["active_open"] == [] or isinstance(body["active_open"], list)
    closed = body.get("construction_closed") or []
    program = body.get("program_next") or []
    assert (
        "INFRA-E4-002" in program
        or "CON-P7-LOGISTICS-001" in program
        or "INFRA-E4-002" in closed
        or "CON-P7-LOGISTICS-001" in closed
    )


def test_coder_drain_brief_b_construction_closed():
    body = agent_queue.coder_drain_brief("b")
    assert body["ok"] is True
    assert "ECON-OG-SAVE" in body["construction_closed"] or "ECON-OG-SAVE-001" in str(
        body["construction_closed"]
    )
