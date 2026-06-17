"""MCP-P2-KIT002-G2 — roof_industrial_shed_2u production bpy profile."""

from __future__ import annotations

from rust_engine_mcp import kit_production_002
from rust_engine_mcp.validators.mcp_schema import validate_mcp_job
from rust_engine_mcp.validators.tier import tier_issues_for_job


def test_roof_production_job_profile_shed() -> None:
    job = kit_production_002._load_job(kit_production_002.ROOF_JOB_ID)
    assert job.get("batch_id") == "kit_production_002"
    assert job.get("development_tier") == "production"
    assert kit_production_002._profile_ok(job, "module_roof", "shed")
    params = job.get("params") or {}
    assert params.get("material_profile") == "metal_roof_01"
    assert float(params.get("pitch_height_m", 0)) > 0


def test_roof_production_job_tier_and_schema() -> None:
    job_path = kit_production_002._job_path(kit_production_002.ROOF_JOB_ID)
    report = validate_mcp_job(job_path, compression_level=1)
    assert report.status == "passed"
    job = kit_production_002._load_job(kit_production_002.ROOF_JOB_ID)
    issues = tier_issues_for_job(job, job_path)
    assert not any(i.severity == "error" for i in issues)


def test_g2_witness_green_after_promote() -> None:
    body = kit_production_002.refresh_kit_production_002_g2_witness()
    assert body.get("green") is True
    assert body.get("bpy_profile") == "shed"
