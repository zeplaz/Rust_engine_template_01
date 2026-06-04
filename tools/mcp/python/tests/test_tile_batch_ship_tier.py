"""PT-2-003 — ship batches reject lod0 tier."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from rust_engine_mcp.validators.tile_batch import validate_tile_batch


def test_ship_lod0_rejected():
    doc = {
        "schema_version": 1,
        "batch_id": "tile_test_ship_lod0",
        "tile_id": "test",
        "base": "stone",
        "ship": True,
        "bake_source": "keyframe_pack",
        "source_tier": "lod0",
        "development_tier": "lod0",
        "variants": [
            {"variant_key": "clean_day"},
            {"variant_key": "clean_night_on"},
        ],
        "render": {"method": "blender_keyframe_light_rig"},
    }
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(doc, f)
        path = f.name
    report = validate_tile_batch(Path(path))
    Path(path).unlink(missing_ok=True)
    assert report.status == "failed"
    assert any(
        e.signature == "tile_batch_ship_lod0_rejected" or "lod0" in (e.hint or "").lower()
        for e in report.errors
    )
