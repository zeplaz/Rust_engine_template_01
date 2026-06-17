"""MCP-APS-ATLAS-LAND-REGISTER-001 tests."""

from __future__ import annotations

from rust_engine_mcp.aps_atlas_land_register import (
    APS_ATLAS_LAND_REGISTER_WITNESS,
    EXPANDED_ATLAS_ID,
    check_atlas_land_register,
    refresh_aps_atlas_land_register_witness,
)
from rust_engine_mcp.landscape_atlas_index import landscape_expanded_atlas_registered
from rust_engine_mcp.paths import repo_root


def test_expanded_landscape_register_ensure() -> None:
    body = check_atlas_land_register()
    assert body.get("expanded_registered") is True
    assert landscape_expanded_atlas_registered()
    assert EXPANDED_ATLAS_ID in (body.get("atlas_ids") or [])


def test_atlas_land_register_witness_green() -> None:
    body = refresh_aps_atlas_land_register_witness()
    assert body.get("green") is True
    assert body.get("register_green") is True
    assert body.get("pilot_registered") is True
    assert body.get("panel_wired") is True
    assert body.get("witness_honesty", {}).get("status") == "passed"
    out = repo_root() / APS_ATLAS_LAND_REGISTER_WITNESS
    assert out.is_file()
