"""MCP-P2-SIM-VALIDATORS — review_order_brief, slice_exec_brief, witness profiles."""

from __future__ import annotations

import json

from rust_engine_mcp import agent_queue, ops_intelligence
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator


def test_review_order_brief_lists_four_p0_rows() -> None:
    body = ops_intelligence.review_order_brief()
    assert body["ok"] is True
    assert len(body["p0_rows"]) == 4
    keys = {row["key"] for row in body["p0_rows"]}
    assert keys == {"P0-A", "P0-B", "P0-C", "P0-D"}
    build = next(r for r in body["p0_rows"] if r["key"] == "P0-A")
    assert build["task_id"] == "TRIAGE-MAP-PICK-CLOSURE-001"
    assert build["label"] == "P0-BUILD-FOOTPRINT-001"
    assert build["status"] == "done"


def test_slice_exec_brief_map_pick_closure() -> None:
    body = agent_queue.slice_exec_brief("TRIAGE-MAP-PICK-CLOSURE-001", queue="phase4")
    assert body["ok"] is True
    assert body["queue"] == "phase4"
    assert "footprint_projection_ok" in body["exit"]
    assert body["witness"] == "debug_runs/construction_stage_live.json"
    assert body["do_not_pick"] is True


def test_phase4_queue_registry_and_load() -> None:
    assert "phase4" in agent_queue.QUEUE_REGISTRY
    tasks = agent_queue.load_queue("phase4")
    assert len(tasks) >= 10
    assert any(t.get("id") == "G-PLAY-01" for t in tasks)


def test_agent_queue_next_coder_prefers_phase4_when_gplay_open() -> None:
    out = agent_queue.agent_queue_next("coder", queue="auto")
    assert out["queue"] == "phase4"
    assert out["queue_requested"] == "auto"


def test_agent_queue_next_phase4_explicit() -> None:
    out = agent_queue.agent_queue_next("coder", queue="phase4")
    assert out["queue"] == "phase4"
    assert "drain_reason" in out


def test_witness_brief_construction_profile_compact() -> None:
    body = agent_queue.witness_brief(
        "debug_runs/construction_stage_live.json",
        profile="construction",
    )
    assert body["ok"] is True
    brief = body["brief"]
    assert brief["profile"] == "construction"
    assert brief["construction_parametric_placement_001"]["green"] is True
    line_count = len(json.dumps(body, indent=2).splitlines())
    assert line_count <= 40


def test_witness_brief_map_pick_from_map_zoom() -> None:
    body = agent_queue.witness_brief(
        "debug_runs/map_zoom_coherence_live.json",
        profile="map_pick",
    )
    assert body["ok"] is True
    assert body["brief"]["profile"] == "map_pick"
    assert body["brief"]["footprint_projection_ok"] is True


def test_validate_report_construction_placement() -> None:
    report = run_validator(
        "construction",
        target="debug_runs/construction_placement_live.json",
        compression_level=3,
    )
    assert report.status == "passed"


def test_write_mcp_phase4_queue_witness() -> None:
    body = ops_intelligence.write_mcp_phase4_queue_live_witness()
    assert body["green"] is True
    assert (repo_root() / ops_intelligence.MCP_PHASE4_QUEUE_WITNESS_REL).is_file()


def test_write_mcp_valid_construction_witness() -> None:
    body = ops_intelligence.write_mcp_valid_construction_live_witness()
    assert body["green"] is True
    assert (repo_root() / ops_intelligence.MCP_VALID_CONSTRUCTION_WITNESS_REL).is_file()


def test_token_savings_guide_blang_review_slice_place() -> None:
    g = agent_queue.token_savings_guide()
    tokens = g["blang"]["tokens"]
    assert "BLANG:REVIEW" in tokens
    assert "BLANG:SLICE" in tokens
    assert "BLANG:PLACE" in tokens
    assert "BLANG:WIT-HON" in tokens
    wit = g["witness_integrity"]
    assert wit["blang"] == "BLANG:WIT-HON"
    assert "validate-report witness_honesty" in wit["validate_witness"]


def test_witness_brief_honesty_profile_failed_rule_ids() -> None:
    bad = agent_queue.witness_brief(
        "tools/mcp/schemas/examples/witness_honesty_fixtures/bad_exit_predicate_live.json",
        profile="honesty",
    )
    assert bad["ok"] is True
    brief = bad["brief"]
    assert brief["profile"] == "honesty"
    assert brief["status"] != "passed"
    assert "WIT-EXIT-PREDICATE" in brief["failed_rule_ids"]
    assert brief["q_forbidden"] is True
    assert brief["blang"] == "BLANG:WIT-HON"

    good = agent_queue.witness_brief(
        "tools/mcp/schemas/examples/witness_honesty_fixtures/good_minimal_live.json",
        profile="honesty",
    )
    assert good["ok"] is True
    assert good["brief"]["status"] == "passed"
    assert good["brief"]["failed_rule_ids"] == []
    assert good["brief"]["q_forbidden"] is False


def test_slice_exec_brief_includes_witness_honesty_and_exit_predicate() -> None:
    body = agent_queue.slice_exec_brief("TRIAGE-MAP-PICK-CLOSURE-001", queue="phase4")
    assert body["ok"] is True
    wh = body["witness_honesty"]
    assert "last_scan" in wh
    assert "fail_count" in wh["last_scan"] or wh["last_scan"].get("source")
    assert "witness" in wh or wh.get("q_forbidden") is not None
