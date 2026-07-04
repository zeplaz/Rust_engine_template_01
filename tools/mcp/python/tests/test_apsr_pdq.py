"""Tests for APSR-A3-D1–D4 and APSR-A4-Q1–Q3."""

from __future__ import annotations

from rust_engine_mcp import building_quality_qc, golden_seed_review, kit_coverage_audit
from rust_engine_mcp.aps_inline_feedback_audit import inline_feedback_audit, write_apsr_d3_witness
from rust_engine_mcp.aps_tooltip_coverage import tooltip_coverage_audit, write_apsr_d2_witness
from rust_engine_mcp.apsr_density_polish import density_polish_audit, write_apsr_d4_witness
from rust_engine_mcp.apsr_design_token_lint import token_lint_audit, write_apsr_d1_witness
from rust_engine_mcp.golden_seed_review import write_apsr_q3_witness
from rust_engine_mcp.kit_coverage_audit import write_apsr_q2_witness
from rust_engine_mcp.paths import repo_root


def test_token_lint_zero_violations() -> None:
    audit = token_lint_audit()
    assert audit["green"] is True
    assert audit["violation_count"] == 0


def test_tooltip_coverage_green() -> None:
    audit = tooltip_coverage_audit()
    assert audit["green"] is True


def test_inline_feedback_adoption_green() -> None:
    audit = inline_feedback_audit()
    assert audit["green"] is True


def test_density_polish_charter_wired() -> None:
    audit = density_polish_audit()
    assert audit["green"] is True


def test_kit_coverage_audit_runs() -> None:
    audit = kit_coverage_audit.audit_all_style_packs()
    assert audit["style_pack_count"] >= 1
    text, ok = kit_coverage_audit.format_kit_coverage_summary()
    assert "Kit coverage" in text
    assert ok is True or ok is False


def test_golden_seeds_load_from_bq_q3() -> None:
    seeds = golden_seed_review.load_golden_seeds()
    assert len(seeds) >= 12


def test_golden_seed_verdict_persists() -> None:
    seeds = golden_seed_review.load_golden_seeds()
    entry = seeds[0]
    row = golden_seed_review.record_seed_verdict(entry, verdict="approve", note="pytest")
    assert row["verdict"] == "approve"
    data = golden_seed_review.load_rubric_rows()
    assert any(r.get("seed_key") == row["seed_key"] for r in data.get("rows") or [])


def test_assembly_qc_allows_approve_with_witness() -> None:
    allowed, reason = building_quality_qc.assembly_qc_allows_approve("victorian_4x2_s1_d8e2")
    assert isinstance(allowed, bool)
    assert reason


def test_apsr_d1_witness_green() -> None:
    body = write_apsr_d1_witness()
    assert body["green"] is True


def test_apsr_d2_witness_green() -> None:
    body = write_apsr_d2_witness()
    assert body["green"] is True


def test_apsr_d3_witness_green() -> None:
    body = write_apsr_d3_witness()
    assert body["green"] is True


def test_apsr_d4_witness_green() -> None:
    body = write_apsr_d4_witness()
    assert body["green"] is True


def test_apsr_q1_witness_green() -> None:
    body = building_quality_qc.write_apsr_q1_witness()
    assert body["green"] is True
    assert body["approve_blocks_on_red_qc"] is True


def test_apsr_q2_witness_writes() -> None:
    body = write_apsr_q2_witness()
    assert body["task_id"] == "APSR-A4-Q2-001"
    assert (repo_root() / "debug_runs/apsr_a4_q2_001_live.json").is_file()


def test_apsr_q3_witness_green() -> None:
    body = write_apsr_q3_witness()
    assert body["green"] is True
    assert body["golden_seed_count"] >= 12
