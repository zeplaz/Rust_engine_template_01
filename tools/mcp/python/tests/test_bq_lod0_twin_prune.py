"""BQ-LOD0-TWIN-PRUNE-001 — archive unused lod0 twins + index rebuild."""

from __future__ import annotations

from rust_engine_mcp import bq_lod0_twin_prune
from rust_engine_mcp.paths import repo_root


def test_list_lod0_dirs_or_archived() -> None:
    root = repo_root()
    live = bq_lod0_twin_prune.list_lod0_job_dirs(repo=root)
    archive = root / bq_lod0_twin_prune.ARCHIVE_REL / "modules"
    archived = len([p for p in archive.iterdir() if p.is_dir()]) if archive.is_dir() else 0
    # Either still on disk (pre-run) or already quarantined.
    assert len(live) + archived >= 50 or archived >= 50 or len(live) == 50


def test_run_lod0_twin_prune_green() -> None:
    body = bq_lod0_twin_prune.run_lod0_twin_prune(register=True, retire=True)
    assert body["rules_check"]["passed"] is True
    assert body["rules_check"]["no_q3_golden_seed_theater"] is True
    assert body["exit_check"]["lod0_count"] == 0
    assert body["exit_check"]["residual_lod0_dirs_in_modules"] == []
    assert body["operator_pass"] is False
    assert not str(body["next_residual"]).startswith("BQ-Q3")
    assert "063286c3" in str(body["next_residual"])
    assert "Visual Concept" in str(body["next_residual"])
    # First run clears ~50; re-runs may clear 0 if already archived.
    archive = repo_root() / bq_lod0_twin_prune.ARCHIVE_REL / "modules"
    archived = len([p for p in archive.iterdir() if p.is_dir()]) if archive.is_dir() else 0
    assert body["pruned_count"] + archived >= 50
    assert body["green"] is True


def test_write_witness() -> None:
    body = bq_lod0_twin_prune.write_bq_lod0_twin_prune_witness()
    assert body["written"] == bq_lod0_twin_prune.WITNESS_REL
    path = repo_root() / bq_lod0_twin_prune.WITNESS_REL
    assert path.is_file()
    assert body["_agent_meta"]["task_id"] == "BQ-LOD0-TWIN-PRUNE-001"
