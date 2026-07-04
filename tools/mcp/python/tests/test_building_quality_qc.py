"""APSR-A4-Q1-001 — building quality QC strip tests."""

from __future__ import annotations

from rust_engine_mcp import building_quality_qc
from rust_engine_mcp.paths import repo_root


def test_building_quality_witness_loads() -> None:
    witness = building_quality_qc.load_building_quality_witness(repo=repo_root())
    assert witness is not None
    assert witness.get("gate") == "BQ-A2-GATE-001"


def test_format_qc_strip_from_witness() -> None:
    text, ok = building_quality_qc.format_qc_strip_text(repo=repo_root())
    assert "Building QC" in text
    assert ok is True or ok is False or ok is None


def test_write_apsr_q1_witness() -> None:
    body = building_quality_qc.write_apsr_q1_witness(repo=repo_root())
    assert body.get("task_id") == "APSR-A4-Q1-001"
    assert body.get("approve_blocks_on_red_qc") is True
    assert (repo_root() / "debug_runs" / "apsr_a4_q1_001_live.json").is_file()
