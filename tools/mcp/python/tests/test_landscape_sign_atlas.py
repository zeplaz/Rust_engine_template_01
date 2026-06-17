"""MCP landscape sign + atlas rollup witness."""

from __future__ import annotations

from rust_engine_mcp import landscape_sign_atlas


def test_landscape_sign_atlas_status_green() -> None:
    status = landscape_sign_atlas.landscape_sign_atlas_status()
    assert status["sign"]["green"] is True
    assert status["preset_batch"]["green"] is True
    assert status["atlas"]["green"] is True


def test_landscape_sign_atlas_witness() -> None:
    body = landscape_sign_atlas.refresh_mcp_landscape_sign_atlas_witness()
    assert body.get("green") is True
    assert body.get("written") == landscape_sign_atlas.ROLLUP_WITNESS_REL
