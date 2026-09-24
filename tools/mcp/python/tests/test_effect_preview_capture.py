"""VSS-T4 residual — effect preview capture → honest_gate."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

import pytest

from rust_engine_mcp import effect_preview_capture as epc
from rust_engine_mcp import effect_promote as ep
from rust_engine_mcp.effect_registry_browse import list_effect_entries
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.scenario_trigger_write import assign_effect_to_marker, can_assign

SPARK = repo_root() / "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json"
SMOKE = repo_root() / "tools/mcp/schemas/examples/effect_spec_smoke_column_v1.json"
RAIN = repo_root() / "tools/mcp/schemas/examples/effect_spec_rain_streaks_v1.json"
REFERENCE = (SPARK, SMOKE, RAIN)


@pytest.fixture()
def isolated_staging(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    staging = tmp_path / "assets" / "staging"
    registry = tmp_path / "assets" / "effects" / "registry"
    staging.mkdir(parents=True)
    registry.mkdir(parents=True)

    monkeypatch.setattr(ep, "staging_root", lambda: staging)
    monkeypatch.setattr(ep, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(ep, "WITNESS_REL", "debug_runs/artist_vfx_pipeline_live.json")
    monkeypatch.setattr(ep, "REGISTRY_REL", "assets/effects/registry")
    monkeypatch.setattr(epc, "staging_root", lambda: staging)
    monkeypatch.setattr(epc, "repo_root", lambda: tmp_path)
    monkeypatch.setattr(epc, "WITNESS_REL", "debug_runs/artist_vfx_pipeline_live.json")
    monkeypatch.setattr(epc, "REGISTRY_REL", "assets/effects/registry")

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


def test_capture_writes_digest_frames_and_honest_gate(isolated_staging: Path) -> None:
    rel = "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json"
    body = epc.capture_effect_preview(rel, pack_first=True, write_registry=False)
    assert body["honest_gate"] == "honest"
    assert body["frames_captured"] >= 1
    assert body["capture_kind"] == "staging_pack_digest"
    assert len(body["capture_hash"]) == 64

    frames = isolated_staging / "assets" / "staging" / "spark_shower" / "preview_frames"
    assert (frames / "capture_000.png").is_file()
    assert (frames / "frames_manifest.json").is_file()
    assert epc.verify_capture_hash(isolated_staging / "assets" / "staging" / "spark_shower")

    pw = body["preview_witness"]
    assert pw["honest_gate"] == "honest"
    assert pw["frames_captured"] == body["frames_captured"]
    assert pw["capture_hash"] == body["capture_hash"]


def test_capture_then_promote_preserves_honesty(isolated_staging: Path) -> None:
    rel = "tools/mcp/schemas/examples/effect_spec_rain_streaks_v1.json"
    epc.capture_effect_preview(rel, pack_first=True, write_registry=False)
    promoted = ep.promote_effect(rel, pack_first=False)
    assert promoted["preview_witness"]["honest_gate"] == "honest"
    assert promoted["preview_witness"]["frames_captured"] >= 1
    reg = isolated_staging / "assets" / "effects" / "registry" / "rain_streaks"
    assert (reg / "preview_frames" / "capture_000.png").is_file()
    manifest = json.loads((reg / ep.PROMOTE_MANIFEST).read_text(encoding="utf-8"))
    assert manifest["preview_witness"]["honest_gate"] == "honest"


def test_batch_capture_unblocks_assign(isolated_staging: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        "rust_engine_mcp.effect_registry_browse.repo_root", lambda: isolated_staging
    )
    monkeypatch.setattr(
        "rust_engine_mcp.effect_registry_browse.staging_root",
        lambda: isolated_staging / "assets" / "staging",
    )
    batch = epc.capture_reference_batch(ep.REFERENCE_BATCH_ID, pack_first=True)
    assert batch["count"] == 3
    assert batch["all_honest"] is True

    entries = list_effect_entries(repo=isolated_staging, batch_id=ep.REFERENCE_BATCH_ID)
    assert len(entries) >= 3
    for e in entries:
        assert e.honest_gate == "honest"
        assert can_assign(e) is True

    spark = next(e for e in entries if e.effect_id == "spark_shower")
    result = assign_effect_to_marker(
        "spark_shower",
        "test_spark_shower_capture",
        repo=isolated_staging,
        entry=spark,
    )
    assert (isolated_staging / result["path"]).is_file()


def test_capture_is_deterministic(isolated_staging: Path) -> None:
    rel = "tools/mcp/schemas/examples/effect_spec_smoke_column_v1.json"
    a = epc.capture_effect_preview(rel, pack_first=True, write_registry=False)
    b = epc.capture_effect_preview(rel, pack_first=False, write_registry=False)
    assert a["capture_hash"] == b["capture_hash"]
    assert a["frames_captured"] == b["frames_captured"]
