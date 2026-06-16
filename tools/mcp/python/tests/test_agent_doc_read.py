"""MCP agent-lang Tier 1 — doc touch, run append, snapshot diff, grammar_iterate MCP."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from rust_engine_mcp import agent_doc_read, agent_queue
from rust_engine_mcp.grammar_iterate import iterate_grammar
from rust_engine_mcp.paths import repo_root

EXAMPLE_SNAPSHOT = (
    "tools/mcp/schemas/examples/"
    "assembly_snapshot_warehouse_industrial_west_production_v1.json"
)
PLAN_DOC = "docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md"


def test_agent_doc_touch_schema_and_ledger():
    result = agent_doc_read.agent_doc_touch(PLAN_DOC, intent="ref", agent="pytest")
    assert result["schema"] == "agent_doc_touch_v1"
    assert result["ok"] is True
    assert result["intent"] == "ref"
    assert result["digest"]["total_lines"] > 0
    assert result["ledger_appended"] is True
    ledger = repo_root() / agent_doc_read.DOC_READS_LEDGER
    assert ledger.is_file()
    last = ledger.read_text(encoding="utf-8").strip().splitlines()[-1]
    row = json.loads(last)
    assert row["path"] == PLAN_DOC
    assert row["agent"] == "pytest"


def test_agent_doc_touch_bad_intent():
    result = agent_doc_read.agent_doc_touch(PLAN_DOC, intent="read-all")
    assert result["ok"] is False


def test_agent_run_append():
    result = agent_doc_read.agent_run_append(
        {"slice_id": "test-slice", "tools_called": ["agent_doc_touch"]},
        agent="pytest",
    )
    assert result["ok"] is True
    assert result["appended"]["slice_id"] == "test-slice"
    ledger = repo_root() / agent_doc_read.RUN_EVENTS_LEDGER
    assert ledger.is_file()


def test_snapshot_diff_brief_same_file():
    result = agent_doc_read.snapshot_diff_brief(EXAMPLE_SNAPSHOT, EXAMPLE_SNAPSHOT)
    assert result["ok"] is True
    assert result["schema"] == "snapshot_diff_brief_v1"
    assert result["diff"]["cells_added"] == 0
    assert result["diff"]["cells_removed"] == 0


def test_snapshot_diff_brief_after_iterate():
    req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "massing",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": EXAMPLE_SNAPSHOT,
        "overrides": {
            "massing_strategy": "double_hall",
            "footprint": {"width": 10, "depth": 6, "floors": 2},
        },
        "preserve_layers": ["district_style", "age"],
        "parent_lineage_id": "industrial_west_8x9_s43_f75a",
    }
    iterated = iterate_grammar(req)
    assert iterated["ok"] is True
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False, encoding="utf-8") as fh:
        json.dump(iterated["snapshot"], fh)
        after_path = fh.name
    try:
        result = agent_doc_read.snapshot_diff_brief(EXAMPLE_SNAPSHOT, after_path)
        assert result["ok"] is True
        assert result["diff"]["cells_added"] >= 0
        assert "massing" in result["diff"]["layers_touched"]
    finally:
        Path(after_path).unlink(missing_ok=True)


def test_grammar_iterate_mcp_wrapper():
    req = {
        "schema": "grammar_iterate_request_v1",
        "mode": "roof",
        "seed": 43,
        "archetype_id": "IndustrialWarehouse",
        "district_style": "industrial_west",
        "base_snapshot_path": EXAMPLE_SNAPSHOT,
        "overrides": {"roof_rule_id": "roof_flat"},
        "preserve_layers": ["district_style", "age"],
        "parent_lineage_id": "industrial_west_8x9_s43_f75a",
    }
    with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False, encoding="utf-8") as fh:
        json.dump(req, fh)
        req_path = fh.name
    try:
        result = agent_doc_read.grammar_iterate_mcp(req_path)
        assert result.get("ok") is True
        assert "roof" in (result.get("diff") or {}).get("layers_touched", [])
    finally:
        Path(req_path).unlink(missing_ok=True)


def test_token_savings_guide_blang():
    guide = agent_queue.token_savings_guide()
    assert "blang_session_loop" in guide
    assert "agent_session_bootstrap" in guide["session_start"][1]
    assert any("agent_doc_touch" in row for row in guide["artifact_touch"])
    assert "grammar_iterate" in guide["grammar_tools"][0]


def test_refresh_witnesses():
    assert agent_doc_read.refresh_agent_doc_read_witness()
    assert agent_doc_read.refresh_agent_run_append_witness()
    doc_w = json.loads(
        (repo_root() / agent_doc_read.AGENT_DOC_READ_WITNESS).read_text(encoding="utf-8")
    )
    run_w = json.loads(
        (repo_root() / agent_doc_read.AGENT_RUN_APPEND_WITNESS).read_text(encoding="utf-8")
    )
    assert doc_w["green"] is True
    assert run_w["green"] is True


def test_agent_marker_append_and_brief():
    result = agent_doc_read.agent_marker_append(
        agent="pytest",
        slice_id="AGENT-LANG-DEMO-001",
        mirror="@coder-mcp landed WRK — extend only",
        scan="BLANG:P0 footprint 4x3 🟢",
        why="Review stop for next agent",
        joint="Suggest designer-mcp G4 before promote",
        dim=["🟡", "🧩"],
        delta_wf="ΔWF→@designer-mcp",
    )
    assert result["ok"] is True
    brief = agent_doc_read.agent_markers_brief(tail=3)
    assert brief["ok"] is True
    assert brief["count"] >= 1
    assert brief["markers"][-1]["agent"] == "pytest"


def test_agent_doc_reads_brief():
    agent_doc_read.agent_doc_touch(PLAN_DOC, intent="ref", agent="pytest-stats")
    agent_doc_read.agent_doc_touch(PLAN_DOC, intent="ref", agent="pytest-stats")
    brief = agent_doc_read.agent_doc_reads_brief(min_reads=2, write_witness=True)
    assert brief["ok"] is True
    assert brief["schema"] == "agent_doc_reads_brief_v1"
    assert brief["total_reads_in_window"] >= 2
    witness = repo_root() / agent_doc_read.DOC_READS_BRIEF_WITNESS
    assert witness.is_file()


def test_agent_doc_promote_and_cache(monkeypatch):
    fixture_dir = repo_root() / "tools/mcp/cache/_pytest_promote_fixture"
    fixture_dir.mkdir(parents=True, exist_ok=True)
    isolated_doc = fixture_dir / "promote_fixture.md"
    isolated_doc.write_text("# promote fixture\nline two\n", encoding="utf-8")
    rel = isolated_doc.relative_to(repo_root()).as_posix()
    cache_rel = "tools/mcp/cache/agent_doc_digests_pytest_promote"
    (repo_root() / cache_rel).mkdir(parents=True, exist_ok=True)
    ledger_rel = "tools/mcp/cache/_pytest_promote_fixture/doc_reads.jsonl"
    (repo_root() / ledger_rel).unlink(missing_ok=True)
    monkeypatch.setattr(agent_doc_read, "DIGEST_CACHE_DIR", cache_rel)
    monkeypatch.setattr(agent_doc_read, "DOC_READS_LEDGER", ledger_rel)

    for _ in range(4):
        agent_doc_read.agent_doc_touch(rel, intent="ref", agent="pytest-promote-isolated")
    promote = agent_doc_read.agent_doc_promote_hot_reads(min_reads=3, max_promote=2)
    assert promote["ok"] is True
    assert promote["promoted_count"] >= 1
    assert any(row.get("path") == rel for row in promote.get("promoted") or [])
    cached = agent_doc_read.agent_doc_digest_cached(rel)
    assert cached["ok"] is True
    assert cached.get("cache_hit") is True


def test_agent_session_bootstrap():
    result = agent_doc_read.agent_session_bootstrap("pytest", session_hint="PYTEST-BOOT")
    assert result["ok"] is True
    assert result["schema"] == "agent_session_bootstrap_v1"
    assert len(result["canonical_touches"]) == len(agent_doc_read.CANONICAL_SESSION_PATHS)
    assert "prompts/llm_agent_brief.md" in result["role_reads"] or any(
        t["path"] == "prompts/llm_agent_brief.md" for t in result["canonical_touches"]
    )
