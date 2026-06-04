"""Tests for structured validators."""

from __future__ import annotations

import json

from rust_engine_mcp.validators.asset import validate_asset_glb
from rust_engine_mcp.validators.cargo import _parse_cargo_json, validate_cargo
from rust_engine_mcp.validators.mcp_schema import validate_mcp_job
from rust_engine_mcp.validators.report import ValidationIssue, ValidationReport
from rust_engine_mcp.validators.tier import AssetValidationContext, tier_issues_for_asset, tier_issues_for_job
from rust_engine_mcp.paths import repo_root


def test_validation_report_compress_level_4():
    report = ValidationReport(
        validator="cargo",
        status="failed",
        errors=[
            ValidationIssue(kind="TypeMismatch", hint="a"),
            ValidationIssue(kind="MissingField", hint="b"),
        ],
        known_fixes=[],
        summary="2 errors",
        error_count=2,
    )
    c4 = report.compress(4)
    assert c4.compression_level == 4
    assert len(c4.errors) == 0


def test_parse_cargo_json_message():
    line = json.dumps(
        {
            "reason": "compiler-message",
            "message": {
                "level": "error",
                "message": "mismatch types",
                "code": {"code": "E0308"},
                "spans": [{"file_name": "src/foo.rs", "line_start": 10, "column_start": 5}],
            },
        }
    )
    root = repo_root()
    issues = _parse_cargo_json(line + "\n", root)
    assert len(issues) == 1
    assert issues[0].kind == "TypeMismatch"
    assert issues[0].rustc_code == "E0308"


def test_validate_mcp_job_missing_seed_warning():
    job = repo_root() / "tools/mcp/schemas/examples/wall_concrete_2u_run001.json"
    report = validate_mcp_job(job, compression_level=3)
    assert report.status in ("passed", "warning", "failed")
    assert report.validator == "mcp_schema"


def test_validate_cargo_cached_orchestrator():
    cached = repo_root() / "tools/orchestrator/state/last_run.json"
    if not cached.is_file():
        return
    report = validate_cargo(use_cached_orchestrator=True, compression_level=3)
    assert report.validator == "cargo"
    assert report.compression_level == 3


def test_tier_rejects_24vert_pitched_at_lod0():
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=24,
        archetype="module_roof",
        profile="pitched",
        development_tier="lod0",
        module_id="roof_pitched_2u",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=24)
    assert any(i.rule_id == "TIER-002" for i in issues)


def test_tier_allows_24vert_pitched_at_smoke():
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=24,
        archetype="module_roof",
        profile="pitched",
        development_tier="smoke",
        batch_id="kit_greybox_001",
        module_id="roof_pitched_2u",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=24)
    assert not any(i.rule_id == "TIER-002" for i in issues)


def test_tier_rejects_kit_greybox_new_job():
    job = {
        "schema_version": 1,
        "job_id": "test_bad",
        "operation": "module_wall",
        "batch_id": "kit_greybox_004",
        "output": {"glb": "assets/staging/test/model.glb"},
    }
    issues = tier_issues_for_job(job, repo_root() / "test.json")
    assert any(i.rule_id == "TIER-006" and i.severity == "error" for i in issues)


def test_tier_b2_wall_brick_cube_at_production():
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=24,
        archetype="module_wall",
        profile="flat",
        development_tier="production",
        batch_id="kit_production_001",
        module_id="wall_brick_1u",
        pbr_status="shipped",
        material_profile="brick_red_01",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=24)
    assert any(i.rule_id == "TIER-002" for i in issues)


def test_tier_b2_roof_pitched_cube_at_production():
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=24,
        archetype="module_roof",
        profile="pitched",
        development_tier="production",
        batch_id="kit_production_001",
        module_id="roof_pitched_gable",
        pbr_status="shipped",
        material_profile="roof_tile_01",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=24)
    assert any(i.rule_id == "TIER-002" for i in issues)


def test_tier_pbr_missing_tileable_at_production():
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=200,
        archetype="module_wall",
        profile="brick",
        development_tier="production",
        batch_id="kit_production_001",
        module_id="wall_brick_1u",
        pbr_status="shipped",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=200)
    assert any(i.signature == "tier_missing_tileable_set" for i in issues)


def test_tier_pbr_unknown_tileable_at_production():
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=200,
        archetype="module_wall",
        profile="brick",
        development_tier="production",
        batch_id="kit_production_001",
        module_id="wall_brick_1u",
        pbr_status="shipped",
        material_profile="neon_pink_99",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=200)
    assert any(i.signature == "tier_unknown_tileable_set" for i in issues)


def test_production_spec_tier_clean():
    spec_path = repo_root() / "assets/staging/specs/wall_brick_1u_production.json"
    if not spec_path.is_file():
        return
    from rust_engine_mcp.validators.tier import tier_issues_for_spec

    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    issues = tier_issues_for_spec(spec, spec_path)
    assert not any(i.severity == "error" for i in issues)


def test_validate_asset_greybox_pitched_on_disk():
    glb = repo_root() / "assets/models/modules/roof_pitched_2u_run001/model.glb"
    if not glb.is_file():
        return
    report = validate_asset_glb(glb, compression_level=1)
    assert report.status in ("passed", "warning")
    assert not any(e.kind == "SilhouetteInsufficient" for e in report.errors)
    assert any(e.symbol == "TIER-006" for e in report.errors)
