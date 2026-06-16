"""RT-REG-001 — rowhouse production register + lookup stamp witness."""

from __future__ import annotations

import json

from rust_engine_mcp import rt_registry
from rust_engine_mcp.paths import repo_root


def test_lookup_stamp_rowhouse_production_v1():
    meta_path = (
        repo_root()
        / "assets/staging/tiles/tile_rowhouse_victorian_production_v1/atlas_meta.json"
    )
    meta = json.loads(meta_path.read_text(encoding="utf-8"))
    stamp = rt_registry.lookup_stamp_from_meta(meta)
    assert stamp["schema_version"] == 1
    assert stamp["lookup_count"] == 14
    assert len(stamp["stamp"]) == 16
    assert len(stamp["variant_keys_sample"]) == 8


def test_rt_registry_register_rowhouse_production():
    body = rt_registry.rt_registry_register_rowhouse_production()
    assert body["ok"] is True
    assert body["atlas_id"] == rt_registry.ROWHOUSE_PRODUCTION_ATLAS_ID
    assert body["lookup_stamp"]["lookup_count"] == 14
    assert body["ship_allowed"] is True
    assert body["atlas_png_present"] is True


def test_refresh_rt_registry_001_witness():
    assert rt_registry.refresh_rt_registry_001_witness()
    data = json.loads(
        (repo_root() / rt_registry.RT_REGISTRY_WITNESS).read_text(encoding="utf-8")
    )
    assert data["green"] is True
    assert data["gate_id"] == "RT-REG-001"
    assert data["lookup_stamp"]["lookup_count"] == 14
