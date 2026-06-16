"""OPS-CYCLE-2-001 — ops_project_brief_v1 + MCP function layer."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from rust_engine_mcp import ops_intelligence
from rust_engine_mcp.paths import repo_root

REQUIRED_KEYS = {
    "schema",
    "project",
    "quality_score",
    "utility_score",
    "auth_spine",
    "known_failures",
    "top_failures_ranked",
    "recent_improvements",
    "suggested_focus",
    "active_picks",
    "last_20_runs_summary",
    "metrics_tier1",
}


def test_ops_build_project_brief_schema_keys():
    brief = ops_intelligence.ops_build_project_brief()
    assert ops_intelligence.brief_schema_keys_present(brief)
    assert REQUIRED_KEYS.issubset(brief.keys())
    assert brief["schema"] == "ops_project_brief_v1"
    assert isinstance(brief["quality_score"], int)
    assert isinstance(brief["utility_score"], (int, float))
    assert isinstance(brief["active_picks"], dict)
    assert brief["metrics_tier1"]["q_per_token"] is None
    assert brief["metrics_tier1"]["ftr"] is None


def test_ops_get_project_brief_ok_true():
    brief = ops_intelligence.ops_get_project_brief()
    assert brief.get("ok") is True
    assert brief["schema"] == "ops_project_brief_v1"


def test_ops_get_retry_guidance_known_task():
    guidance = ops_intelligence.ops_get_retry_guidance("G-PLAY-01")
    assert guidance.get("ok") is True
    assert guidance["task_id"] == "G-PLAY-01"
    assert guidance.get("status") == "ready"
    assert "witness" in guidance


def test_ops_get_retry_guidance_missing_task():
    guidance = ops_intelligence.ops_get_retry_guidance("NOT-A-REAL-SLICE-999")
    assert guidance.get("ok") is False


def test_ops_project_brief_written_by_scan():
    script = repo_root() / "tools/orchestrator/scripts/ops_witness_index.py"
    proc = subprocess.run(
        [sys.executable, str(script)],
        cwd=repo_root(),
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    brief_path = repo_root() / ops_intelligence.OPS_BRIEF_REL
    witness_path = repo_root() / ops_intelligence.OPS_MCP_LAYER_WITNESS_REL
    assert brief_path.is_file()
    assert witness_path.is_file()
    brief = json.loads(brief_path.read_text(encoding="utf-8"))
    witness = json.loads(witness_path.read_text(encoding="utf-8"))
    assert brief["schema"] == "ops_project_brief_v1"
    assert witness["green"] is True
    assert witness["ops_get_project_brief"] is True
    assert witness["ops_project_brief_v1_path"] is True
    report_path = repo_root() / ops_intelligence.OPS_REPORT_REL
    report = json.loads(report_path.read_text(encoding="utf-8"))
    assert "utility_score" in report
    assert "metrics_tier1" in report
    assert "ops_project_brief" in report["_agent_meta"]["source_system"]
