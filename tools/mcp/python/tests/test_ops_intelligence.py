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
    tier1 = brief["metrics_tier1"]
    assert tier1["q_per_token"] is None
    assert tier1["status"] in ("not_measured", "measured", "sparse")
    assert "ftr" in tier1


def test_ops_get_project_brief_ok_true():
    brief = ops_intelligence.ops_get_project_brief()
    assert brief.get("ok") is True
    assert brief["schema"] == "ops_project_brief_v1"


def test_ops_get_retry_guidance_known_task():
    guidance = ops_intelligence.ops_get_retry_guidance("G-PLAY-01")
    assert guidance.get("ok") is True
    assert guidance["task_id"] == "G-PLAY-01"
    assert guidance.get("schema") == "ops_retry_guidance_v2"
    assert guidance.get("status") in ("ready", "blocked")
    assert "witness" in guidance
    assert guidance.get("exec_doc") == "src/dev/plan_g_play_close_001_checklist_v1.md"


def test_ops_get_retry_guidance_triage_map_pick_hotfix():
    guidance = ops_intelligence.ops_get_retry_guidance("TRIAGE-MAP-PICK-CLOSURE-001")
    assert guidance.get("ok") is True
    assert guidance.get("phase4_row") is True
    assert guidance.get("exec_doc") == "src/dev/plan_build_footprint_vm09_exec_v1.md"
    steps = guidance.get("hotfix_steps") or []
    assert len(steps) >= 2
    assert steps[0].get("phase") == "A"
    assert "visual_authority.rs" in steps[0].get("file", "")


def test_ops_get_active_blockers_g_play_open():
    blockers = ops_intelligence.ops_get_active_blockers()
    assert blockers.get("ok") is True
    open_ids = [g["id"] for g in blockers.get("open_gates") or []]
    assert "G-PLAY-01" in open_ids
    gplay = next(g for g in blockers["open_gates"] if g["id"] == "G-PLAY-01")
    assert any(s.get("id") == "G-PLAY-OPERATOR-01" for s in gplay.get("open_sub_gates") or [])


def test_ops_build_project_brief_delta_wf_composed():
    brief = ops_intelligence.ops_build_project_brief()
    assert "delta_wf" in brief
    assert isinstance(brief["delta_wf"], list)
    assert "active_blockers" in brief
    assert brief["active_blockers"].get("ok") is True


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


def test_ops_intelligence_scan_mcp_ops_report_001():
    """MCP-OPS-REPORT-001 — shared CLI/MCP scan writes slice witness + spine artifacts."""
    body = ops_intelligence.run_ops_intelligence_scan(window_hours=24, write_slice_witness=True)
    assert body.get("gate") == "MCP-OPS-REPORT-001"
    assert body.get("green") is True, body.get("summary") or body.get("steps")
    assert body.get("written") == ops_intelligence.MCP_OPS_REPORT_001_WITNESS_REL
    root = repo_root()
    for rel in (
        ops_intelligence.UNIFIED_INDEX_REL,
        ops_intelligence.OPS_REPORT_REL,
        ops_intelligence.OPS_BRIEF_REL,
        ops_intelligence.OPS_DASHBOARD_REL,
        ops_intelligence.OPS_TRIAGE_REL,
        ops_intelligence.MCP_OPS_REPORT_001_WITNESS_REL,
    ):
        assert (root / rel).is_file(), rel
    disk = json.loads((root / ops_intelligence.MCP_OPS_REPORT_001_WITNESS_REL).read_text(encoding="utf-8"))
    assert disk.get("green") is True
    assert disk.get("_agent_meta", {}).get("task_id") == "MCP-OPS-REPORT-001"


def test_cli_ops_intelligence_scan():
    proc = subprocess.run(
        [
            sys.executable,
            "-m",
            "rust_engine_mcp.cli",
            "ops-intelligence-scan",
            "--window-hours",
            "24",
        ],
        cwd=repo_root() / "tools/mcp/python",
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    body = json.loads(proc.stdout)
    assert body.get("ok") is True
    assert body.get("gate") == "MCP-OPS-REPORT-001"