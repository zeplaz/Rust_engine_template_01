"""AUTO-011 — tile pipeline automation tests."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from rust_engine_mcp import assembly
from rust_engine_mcp.paths import repo_root, schemas_dir
from rust_engine_mcp.schemas import (
    load_json_file,
    validate_assembly_build_job,
    validate_assembly_snapshot,
    validate_tile_variant_bake_job,
)
from rust_engine_mcp.tile_pipeline import tile_batch_run, tile_dry_run_enabled
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.witness import write_tile_batch_witness


def test_assembly_snapshot_schema_example(tmp_path: Path) -> None:
    snap = assembly.generate_assembly_snapshot(
        style_pack_id="style_victorian",
        width=4,
        depth=3,
        floors=2,
        seed=42,
        write=False,
    )
    validate_assembly_snapshot(snap)
    assert snap["style_pack_id"] == "style_victorian"
    assert len(snap["module_placements"]) >= 3


def test_style_packs_load_count() -> None:
    packs = assembly.list_style_packs()
    assert len(packs) == 7


def test_assembly_build_job_schema() -> None:
    job = {
        "schema_version": 1,
        "job_id": "asm_test",
        "operation": "assembly_build",
        "assembly_snapshot": "assets/staging/assemblies/test.json",
        "output": {"blend": "assets/staging/assemblies/test.blend"},
    }
    validate_assembly_build_job(job)


def test_tile_variant_bake_job_schema() -> None:
    job = {
        "schema_version": 1,
        "job_id": "tile_test",
        "operation": "tile_variant_bake",
        "mode": "terrain",
        "terrain_base": "concrete",
        "variant": {
            "state": "clean",
            "damage": 0.0,
            "power": "off",
            "fill": "empty",
            "lighting": "day",
        },
        "render": {"method": "blender_orthographic_iso", "seed": 42, "tile_size_px": 128},
        "output": {"png": "assets/staging/tiles/test/clean.png"},
    }
    validate_tile_variant_bake_job(job)


def test_tile_batch_validate_factory_floor() -> None:
    path = schemas_dir() / "examples" / "tile_batch_factory_floor_v1.json"
    report = run_validator("tile_batch", str(path))
    assert report.status in ("passed", "warning")


def test_remap_rowhouse_snapshot_to_production() -> None:
    from rust_engine_mcp.assembly import (
        example_snapshot_path,
        load_assembly_snapshot,
        remap_assembly_snapshot_to_production,
    )

    lod0 = load_assembly_snapshot(example_snapshot_path())
    prod = remap_assembly_snapshot_to_production(lod0)
    assert prod["source_tier"] == "production"
    assert prod["reference_tags"]
    assert prod["assembly_id"] == lod0["assembly_id"]
    for p in prod["module_placements"]:
        assert p["job_id"].endswith("_production_run001")
        assert "kit_production" in p["glb_path"] or "_production_run001" in p["glb_path"]


def test_tile_batch_ship_rejects_smoke_ortho() -> None:
    path = schemas_dir() / "examples" / "tile_batch_rowhouse_victorian_pilot_v1.json"
    data = load_json_file(path)
    data["ship"] = True
    data["source_tier"] = "production"
    bad = repo_root() / "assets" / "staging" / "tiles" / "_test_ship_bad.json"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    try:
        report = run_validator("tile_batch", str(bad))
        assert report.status == "failed"
        sigs = {i.signature for i in report.errors}
        assert "tile_batch_ship_requires_keyframe_pack" in sigs
    finally:
        bad.unlink(missing_ok=True)


@pytest.fixture
def tile_dry_run(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("RUST_ENGINE_TILE_DRY_RUN", "1")


def test_tile_batch_run_dry_factory_floor(tile_dry_run: None) -> None:
    assert tile_dry_run_enabled()
    path = schemas_dir() / "examples" / "tile_batch_factory_floor_v1.json"
    result = tile_batch_run(path)
    assert result.get("ok"), result
    assert result.get("status") == "done"
    assert "not_implemented" not in json.dumps(result)
    witness = result.get("witness") or {}
    assert witness.get("gates", {}).get("G3") == "pass"
    assert witness.get("green") is True


def test_assembly_snapshot_generate_cli_shape() -> None:
    """CLI must emit assembly_snapshot_v1 JSON with written_path (AUTO-011 contract)."""
    proc = subprocess.run(
        [
            sys.executable,
            "-m",
            "rust_engine_mcp.cli",
            "assembly-snapshot-generate",
            "--style-pack",
            "style_victorian",
            "--footprint",
            "4x3",
            "--floors",
            "2",
            "--seed",
            "4242",
        ],
        cwd=str(repo_root() / "tools/mcp/python"),
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, (proc.stderr or proc.stdout or "cli failed")
    snap = json.loads(proc.stdout)
    assert snap.get("schema_version") == 1
    assert isinstance(snap.get("assembly_id"), str) and snap["assembly_id"]
    assert isinstance(snap.get("module_placements"), list) and snap["module_placements"]
    assert snap.get("style_pack_id") == "style_victorian"
    assert snap.get("footprint", {}).get("width") == 4
    assert snap.get("footprint", {}).get("depth") == 3
    assert "written_path" in snap
    written = repo_root() / str(snap["written_path"])
    try:
        assert written.is_file()
        validate_assembly_snapshot(load_json_file(written))
    finally:
        written.unlink(missing_ok=True)


def test_write_tile_batch_witness_g3() -> None:
    batch = load_json_file(schemas_dir() / "examples" / "tile_batch_factory_floor_v1.json")
    w = write_tile_batch_witness(
        "tile_factory_floor_greybox_001",
        batch=batch,
        png_count=6,
        atlas_path=str(repo_root() / "assets/staging/tiles/x/atlas.png"),
        meta_path=str(repo_root() / "assets/staging/tiles/x/atlas_meta.json"),
        dry_run=True,
    )
    assert w["gates"]["G3"] == "pass"


def test_witness_g3_fails_stub_png_when_real_bake(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    from rust_engine_mcp.witness import write_tile_batch_witness
    from rust_engine_mcp.tile_pipeline import _MINIMAL_PNG

    batch_id = "tile_stub_test"
    staging = tmp_path / "assets" / "staging" / "tiles" / batch_id
    staging.mkdir(parents=True)
    (staging / "a.png").write_bytes(_MINIMAL_PNG)
    (staging / "b.png").write_bytes(_MINIMAL_PNG)
    (staging / "batch_status.json").write_text(
        json.dumps({"status": "done", "dry_run": False}),
        encoding="utf-8",
    )
    monkeypatch.setattr("rust_engine_mcp.witness.repo_root", lambda: tmp_path)

    atlas = staging / "atlas.png"
    meta = staging / "meta.json"
    atlas.write_bytes(b"x")
    meta.write_text("{}", encoding="utf-8")

    batch = {
        "tile_id": "test",
        "variants": [{"state": "clean"}, {"state": "damaged"}],
    }
    w = write_tile_batch_witness(
        batch_id,
        batch=batch,
        png_count=2,
        atlas_path=str(atlas),
        meta_path=str(meta),
        dry_run=False,
    )
    assert w["gates"]["G3"] == "fail"
    assert w["real_bake"] is True


def test_register_tile_atlas_from_meta_pilot() -> None:
    from rust_engine_mcp.tile_index import load_tile_atlas_index, register_tile_atlas_from_batch

    meta = repo_root() / "assets/staging/tiles/tile_rowhouse_victorian_pilot_v1/atlas_meta.json"
    if not meta.is_file():
        pytest.skip("pilot atlas_meta missing — run real tile-batch-run first")
    result = register_tile_atlas_from_batch("tile_rowhouse_victorian_pilot_v1")
    assert result.get("ok")
    entries = load_tile_atlas_index()
    assert any(e.get("atlas_id") == "rowhouse_victorian_pilot_v1" for e in entries)
