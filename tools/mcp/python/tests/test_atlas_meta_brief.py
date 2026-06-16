"""MCP-ATLAS-BRIEF-001 — atlas_meta_brief artist summary."""

from __future__ import annotations

import json

from rust_engine_mcp import atlas_meta_brief
from rust_engine_mcp.paths import repo_root


def test_atlas_meta_brief_pilot_v1_fails():
    body = atlas_meta_brief.atlas_meta_brief(atlas_meta_brief.PILOT_V1_FOLDER)
    assert body["schema"] == "atlas_meta_brief_v1"
    assert body["ok"] is False
    assert body["atlas_meta_schema"] == "v1"
    assert body["plain_language_count"] >= 1
    assert body["artist_messages"][0]["legend_code"] == "ATL_SCHEMA_V1_FROZEN"


def test_atlas_meta_brief_production_v2_passes():
    body = atlas_meta_brief.atlas_meta_brief(atlas_meta_brief.PRODUCTION_V2_FOLDER)
    assert body["ok"] is True
    assert body["atlas_meta_schema"] == "v2"
    assert body["facings"] == 8
    assert body["cells_present"] == 24
    assert body["missing_lookups"] == []


def test_atlas_meta_brief_missing_folder():
    body = atlas_meta_brief.atlas_meta_brief("no/such/atlas/folder")
    assert body["ok"] is False
    assert body["artist_messages"]


def test_refresh_mcp_atlas_brief_witness():
    assert atlas_meta_brief.refresh_mcp_atlas_brief_witness()
    data = json.loads(
        (repo_root() / atlas_meta_brief.MCP_ATLAS_BRIEF_WITNESS).read_text(encoding="utf-8")
    )
    assert data["green"] is True
    assert data["pilot_ok"] is False
    assert data["production_ok"] is True
