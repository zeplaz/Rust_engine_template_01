"""MCP productivity P0 — preflight, snapshot_digest, validate_p0_gate_plain."""

from __future__ import annotations

import json

from rust_engine_mcp import mcp_productivity_p0
from rust_engine_mcp.paths import repo_root

EXAMPLE = (
    "tools/mcp/schemas/examples/"
    "assembly_snapshot_warehouse_industrial_west_production_v1.json"
)


def test_pipeline_preflight_schema():
    body = mcp_productivity_p0.pipeline_preflight()
    assert body["schema"] == "pipeline_preflight_v1"
    assert body["repo_root"]
    assert "assembly_snapshot_v1" in body["schemas"]
    assert body["paths"]["grammars_dir"] is True


def test_snapshot_digest_example():
    body = mcp_productivity_p0.snapshot_digest(EXAMPLE)
    assert body["ok"] is True
    assert body["placements"] > 10
    assert body["material_profiles"]["missing"] == 0
    assert body["grammar"]["archetype"] == "IndustrialWarehouse"
    assert "hint" in body


def test_validate_p0_gate_plain_example():
    body = mcp_productivity_p0.validate_p0_gate_plain(EXAMPLE)
    assert body["schema"] == "validate_p0_gate_plain_v1"
    assert body["status"] in ("passed", "failed")
    assert isinstance(body["artist_messages"], list)
    for msg in body["artist_messages"]:
        assert msg["sentence"]
        assert msg["fix"]


def test_validate_p0_gate_plain_missing():
    body = mcp_productivity_p0.validate_p0_gate_plain("no/such/snapshot.json")
    assert body["status"] == "failed"
    assert body["artist_messages"][0]["signature"] == "assembly_production_snapshot_missing"


def test_refresh_witness():
    assert mcp_productivity_p0.refresh_mcp_productivity_p0_witness()
    data = json.loads(
        (repo_root() / mcp_productivity_p0.MCP_PRODUCTIVITY_P0_WITNESS).read_text(encoding="utf-8")
    )
    assert data["green"] is True
    assert data["snapshot_digest_ok"] is True
