"""DMCP-SPEC-NUCLEAR-PWR-001 witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_nuclear_pwr_spec import (
    EXPECTED_ASSET_ID,
    EXPECTED_GRID,
    HERO_MODULE,
    MODULE_WHITELIST,
    refresh_dmcp_nuclear_pwr_spec_witness,
    run_nuclear_pwr_spec_audit,
)


def test_dmcp_nuclear_pwr_spec_witness() -> None:
    body = refresh_dmcp_nuclear_pwr_spec_witness()
    assert body.get("green") is True
    assert body.get("asset_id") == EXPECTED_ASSET_ID
    assert body.get("grid_units") == EXPECTED_GRID
    assert body.get("spec_only") is True
    assert body.get("bpy_blocked") is True


def test_hero_containment_dome_3x3() -> None:
    audit = run_nuclear_pwr_spec_audit()
    assert audit["hero_module"]["module_id"] == HERO_MODULE
    assert audit["hero_module"]["grid"] == [3, 3]


def test_module_whitelist_from_massing() -> None:
    audit = run_nuclear_pwr_spec_audit()
    assert set(audit["module_whitelist"]) == set(MODULE_WHITELIST)
