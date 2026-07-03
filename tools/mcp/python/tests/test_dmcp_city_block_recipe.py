"""DES-CITY-BLOCK-RECIPE-001 charter witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_city_block_recipe import (
    GATE_ID,
    refresh_city_block_recipe_charter_witness,
    run_city_block_recipe_charter_audit,
)


def test_city_block_recipe_charter_audit_green() -> None:
    audit = run_city_block_recipe_charter_audit()
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["acceptance"]["B3_ron_on_disk"] is True


def test_city_block_recipe_witness_writes() -> None:
    body = refresh_city_block_recipe_charter_witness()
    assert body["gate"] == GATE_ID
    assert body["green"] is True
