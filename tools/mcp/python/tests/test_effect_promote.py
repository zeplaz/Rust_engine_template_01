"""VSS-T4-003 — EffectSpec pack + promote path tests."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

import pytest

from rust_engine_mcp import effect_promote as ep
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator

SPARK = repo_root() / "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json"
SMOKE = repo_root() / "tools/mcp/schemas/examples/effect_spec_smoke_column_v1.json"
RAIN = repo_root() / "tools/mcp/schemas/examples/effect_spec_rain_streaks_v1.json"
REFERENCE = (SPARK, SMOKE, RAIN)


@pytest.fixture()
def isolated_staging(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Pack/promote under tmp_path — no writes to repo assets/."""
    staging = tmp_path / "assets" / "staging"
    registry = tmp_path / "assets" / "effects" / "registry"
    witness = tmp_path / "debug_runs" / "artist_vfx_pipeline_live.json"
    staging.mkdir(parents=True)
    registry.mkdir(parents=True)

    monkeypatch.setattr(ep, "staging_root", lambda: staging)
    monkeypatch.setattr(ep, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(ep, "WITNESS_REL", "debug_runs/artist_vfx_pipeline_live.json")
    monkeypatch.setattr(ep, "REGISTRY_REL", "assets/effects/registry")

    # Copy shader sources referenced by reference specs into tmp repo layout.
    for rel in (
        "assets/shaders/fire/fire_spark_compute.wgsl",
        "assets/shaders/fire/fire_particle_draw.wgsl",
        "assets/shaders/fire/fire_particle.wgsl",
        "assets/shaders/post/weather_fire_field.wgsl",
        "assets/shaders/experiments/atmosphere/smoke_column.wgsl",
        "assets/shaders/weather/weather_precip_draw.wgsl",
    ):
        src = repo_root() / rel
        dest = tmp_path / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)

    for spec in REFERENCE:
        rel = spec.relative_to(repo_root())
        dest = tmp_path / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(spec, dest)

    yield tmp_path


def test_validate_effect_spec_all_reference_examples() -> None:
    for path in REFERENCE:
        report = run_validator("effect_spec", str(path.relative_to(repo_root())), compression_level=4)
        assert report.status == "passed", (path.name, report.to_dict())


def test_pack_effect_spec_writes_manifest(isolated_staging: Path) -> None:
    rel = "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json"
    body = ep.pack_effect_spec(rel)
    assert body["effect_id"] == "spark_shower"
    assert body["shader_count"] == 3
    staging_dir = isolated_staging / "assets" / "staging" / "spark_shower"
    manifest_path = staging_dir / ep.PACK_MANIFEST
    assert manifest_path.is_file()
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert manifest["schema"] == "effect_pack_manifest_v1"
    assert manifest["pack_hash"]
    assert set(manifest["shader_hashes"]) == {"compute", "draw", "particle"}
    assert (staging_dir / "shaders" / "fire_spark_compute.wgsl").is_file()
    pw = body["preview_witness"]
    assert pw["honest_gate"] == "pending"
    assert pw["witness_schema"] == "artist_vfx_preview_witness_v1"


def test_promote_effect_copies_registry(isolated_staging: Path) -> None:
    rel = "tools/mcp/schemas/examples/effect_spec_rain_streaks_v1.json"
    promoted = ep.promote_effect(rel, pack_first=True)
    assert promoted["effect_id"] == "rain_streaks"
    registry = isolated_staging / "assets" / "effects" / "registry" / "rain_streaks"
    assert (registry / "weather_precip_draw.wgsl").is_file()
    assert (registry / ep.EFFECT_SPEC_NAME).is_file()
    assert (registry / ep.PROMOTE_MANIFEST).is_file()
    assert promoted["preview_witness"]["honest_gate"] == "pending"


def test_reference_batch_pack_and_promote(isolated_staging: Path) -> None:
    pack = ep.pack_reference_batch(ep.REFERENCE_BATCH_ID)
    assert pack["count"] == 3
    promote = ep.promote_reference_batch(ep.REFERENCE_BATCH_ID, pack_first=False)
    assert promote["count"] == 3
    for effect_id in ("spark_shower", "smoke_column", "rain_streaks"):
        reg = isolated_staging / "assets" / "effects" / "registry" / effect_id
        assert (reg / ep.PROMOTE_MANIFEST).is_file()


def test_refresh_witness_partial_ship_before_promote(isolated_staging: Path) -> None:
    ep.pack_reference_batch(ep.REFERENCE_BATCH_ID)
    witness = ep.refresh_artist_vfx_pipeline_witness()
    assert witness["slice_id"] == "VSS-T4-003"
    assert witness["green"] is False
    assert witness["reference_specs"]["staging"]["spark_shower"]["pack_hash"]
    gaps = witness["reference_specs"]["gaps"]
    assert gaps
    assert any("promote" in g or "capture" in g or "pending" in g for g in gaps)


def test_refresh_witness_green_after_batch_promote(isolated_staging: Path) -> None:
    ep.effect_promote(batch_id=ep.REFERENCE_BATCH_ID, phase="full", write_witness=False)
    witness = ep.refresh_artist_vfx_pipeline_witness()
    assert witness["green"] is True
    assert witness["status"] == "shipped"
    assert len(witness["reference_specs"]["promoted"]) == 3
