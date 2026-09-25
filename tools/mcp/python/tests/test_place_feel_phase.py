"""TIER-PLACE-001 — kit phase is not a place-feel phase."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.place_feel_phase import claim_issue_rows, phase_note
from rust_engine_mcp.schemas import validate_geometry_job
from rust_engine_mcp.validators.tier import tier_issues_for_job, tier_issues_for_spec


def test_omitted_claim_is_silent() -> None:
    job = {
        "schema_version": 1,
        "job_id": "wall_brick_2u_production_run001",
        "batch_id": "kit_production_001",
        "development_tier": "production",
        "operation": "module_wall",
        "params": {"profile": "brick"},
        "output": {"glb": "x.glb"},
    }
    assert claim_issue_rows("production", None) == []
    assert tier_issues_for_job(job, Path("job.json")) == []
    note = phase_note("lod0")
    assert note["ok"] is True
    assert note["allowed_claims"] == ["preview_ghost_stand_in"]
    assert "site_phase" in note["not_a"]


def test_lod0_cannot_claim_operational_envelope() -> None:
    rows = claim_issue_rows("lod0", "operational_envelope")
    assert rows and rows[0]["rule_id"] == "TIER-PLACE-001"
    assert rows[0]["severity"] == "error"
    job = {
        "batch_id": "kit_lod0_004",
        "development_tier": "lod0",
        "place_feel_claim": "operational_envelope",
        "operation": "module_wall",
        "params": {},
    }
    hits = [i for i in tier_issues_for_job(job, Path("j.json")) if i.rule_id == "TIER-PLACE-001"]
    assert hits and hits[0].severity == "error"


def test_production_envelope_and_same_mesh_are_legal() -> None:
    assert claim_issue_rows("production", "operational_envelope") == []
    assert claim_issue_rows("production", "preview_ghost_same_mesh") == []
    assert claim_issue_rows("smoke", "harness_only") == []
    assert claim_issue_rows("lod0", "preview_ghost_stand_in") == []


def test_module_cannot_claim_picker_icon() -> None:
    for tier in ("smoke", "lod0", "production"):
        rows = claim_issue_rows(tier, "picker_icon")
        assert rows and rows[0]["kind"] == "PlaceFeelPhaseLie"


def test_spec_infers_lod0_batch_against_operational_claim() -> None:
    spec = {
        "asset_id": "wall_brick_1u",
        "batch_id": "kit_lod0_004",
        "place_feel_claim": "operational_envelope",
    }
    hits = [i for i in tier_issues_for_spec(spec, Path("s.json")) if i.rule_id == "TIER-PLACE-001"]
    assert hits


def test_existing_job_schema_still_accepts_jobs_without_claim() -> None:
    job = {
        "schema_version": 1,
        "job_id": "wall_probe",
        "development_tier": "production",
        "operation": "module_wall",
        "output": {"glb": "assets/staging/wall_probe/model.glb"},
    }
    validate_geometry_job(job)
    job["place_feel_claim"] = "operational_envelope"
    validate_geometry_job(job)
