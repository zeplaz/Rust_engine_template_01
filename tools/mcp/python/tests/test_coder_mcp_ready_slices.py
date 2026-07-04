"""Tests for BQ-C2/C3 and CITY-C8 coder-mcp slices."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp import building_quality_bq_c2, building_quality_bq_c3, city_c8_pipeline
from rust_engine_mcp.paths import repo_root


def test_bq_c2_bounds_audit_nonempty() -> None:
    audit = building_quality_bq_c2.run_bounds_audit()
    assert audit["promoted_count"] > 10
    assert audit["ok_count"] >= 1


def test_bq_c2_witness_writes() -> None:
    body = building_quality_bq_c2.write_bq_c2_witness()
    assert body["task_id"] == "BQ-C2-BOUNDS-001"
    assert (repo_root() / building_quality_bq_c2.WITNESS_REL).is_file()


def test_bq_c3_seam_audit() -> None:
    audit = building_quality_bq_c3.run_seam_audit()
    assert audit["style_pack_count"] >= 1


def test_bq_c3_witness_writes() -> None:
    body = building_quality_bq_c3.write_bq_c3_witness()
    assert body["task_id"] == "BQ-C3-SEAM-001"
    assert (repo_root() / building_quality_bq_c3.WITNESS_REL).is_file()


def test_city_c8_pilot_job_validates() -> None:
    job = city_c8_pipeline.pilot_job()
    assert job["operation"] == "module_variant_merge"
    assert len(job["params"]["parts"]) >= 2


def test_city_c8_witness_schema_only() -> None:
    body = city_c8_pipeline.write_city_c8_witness(run_merge=False)
    assert body["schema_ok"] is True
    assert body["op_registered"] is True


def test_bq_f1_tail_tier_unblocked() -> None:
    body = city_c8_pipeline.write_bq_f1_tail_witness()
    assert body["task_id"] == "BQ-F1-TAIL-001"
