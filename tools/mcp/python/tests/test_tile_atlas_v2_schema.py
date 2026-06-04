"""TILE-FIX-002 — atlas meta v2 + visual_config schema tests."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import validate_atlas_meta_v2, validate_visual_config
from rust_engine_mcp.validators import run_validator


def test_visual_config_warehouse_example_passes() -> None:
    path = (
        repo_root()
        / "assets/configs/buildings/visual_config_warehouse_industrial_west_v2.example.json"
    )
    validate_visual_config(json.loads(path.read_text(encoding="utf-8")))
    report = run_validator("visual_config", str(path))
    assert report.status == "passed"


def test_atlas_meta_v2_example_schema_only_fails_lookup_complete() -> None:
    path = (
        repo_root()
        / "tools/mcp/schemas/examples/atlas_meta_warehouse_industrial_west_v2.example.json"
    )
    validate_atlas_meta_v2(json.loads(path.read_text(encoding="utf-8")))
    report = run_validator("atlas_meta_v2", str(path))
    assert report.status == "failed"
    assert any(i.signature == "atlas_meta_v2_lookup_incomplete" for i in report.errors)


def test_frozen_production_batch_rejects_ship() -> None:
    path = (
        repo_root()
        / "tools/mcp/schemas/examples/tile_batch_warehouse_industrial_west_production_v1.json"
    )
    report = run_validator("tile_batch", str(path))
    assert report.status == "failed"
    assert any(i.signature == "tile_batch_frozen" for i in report.errors)


def test_tile_batch_ship_requires_v2_fields() -> None:
    batch = {
        "schema_version": 1,
        "batch_id": "tile_test_v2",
        "tile_id": "test",
        "base": "concrete",
        "ship": True,
        "bake_source": "keyframe_pack",
        "source_tier": "production",
        "rules_applied": [
            "no_ai_generated_images",
            "deterministic_output",
            "batch_processing",
            "grid_alignment",
        ],
        "render": {
            "method": "blender_keyframe_light_rig",
            "seed": 1,
            "tile_size_px": 128,
        },
        "variants": [
            {"variant_key": "a", "state": "clean", "lighting": "day"},
            {"variant_key": "b", "state": "clean", "lighting": "night_on"},
        ],
    }
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        json.dump(batch, f)
        tmp = f.name
    report = run_validator("tile_batch", tmp)
    Path(tmp).unlink(missing_ok=True)
    assert report.status == "failed"
    sigs = {i.signature for i in report.errors}
    assert "tile_batch_ship_requires_atlas_v2" in sigs
    assert "tile_batch_ship_visual_config" in sigs
