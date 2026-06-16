"""RT-BRIEF-001 — runtime lookup brief from index + meta."""

from __future__ import annotations

import json

from rust_engine_mcp import rt_registry, runtime_lookup_brief
from rust_engine_mcp.paths import repo_root


def test_runtime_lookup_brief_rowhouse_production():
    rt_registry.refresh_rt_registry_001_witness()
    body = runtime_lookup_brief.runtime_lookup_brief()
    assert body["schema"] == "runtime_lookup_brief_v1"
    assert body["ok"] is True
    assert body["atlas_meta_schema"] == "v1"
    assert body["lookup_stamp"]["lookup_count"] == 14
    assert body["missing_lookups"] == []
    assert body["plain_language_count"] >= 1


def test_runtime_lookup_brief_missing_atlas():
    body = runtime_lookup_brief.runtime_lookup_brief("no_such_atlas_id")
    assert body["ok"] is False
    assert body["artist_messages"][0]["legend_code"] == "RT_INDEX_MISSING"


def test_refresh_rt_lookup_brief_001_witness():
    rt_registry.refresh_rt_registry_001_witness()
    assert runtime_lookup_brief.refresh_rt_lookup_brief_001_witness()
    data = json.loads(
        (repo_root() / runtime_lookup_brief.RT_LOOKUP_BRIEF_WITNESS).read_text(encoding="utf-8")
    )
    assert data["green"] is True
    assert data["gate_id"] == "RT-BRIEF-001"
    assert data["registry_witness_green"] is True
