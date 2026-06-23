"""DMCP-SPEC-TRANSFORMER-PAD-001 witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_transformer_pad_spec import (
    EXPECTED_ASSET_ID,
    EXPECTED_GRID,
    refresh_dmcp_transformer_pad_spec_witness,
    run_transformer_pad_spec_audit,
)


def test_dmcp_transformer_pad_spec_witness() -> None:
    body = refresh_dmcp_transformer_pad_spec_witness()
    assert body.get("green") is True
    assert body.get("asset_id") == EXPECTED_ASSET_ID
    assert body.get("grid_units") == EXPECTED_GRID
    assert body.get("spec_only") is True
    assert body.get("bpy_pending") is True


def test_transformer_pad_geometry_brief() -> None:
    audit = run_transformer_pad_spec_audit()
    assert audit["checks"]["bushings_three"] is True
    assert audit["checks"]["grid_units_2x2"] is True
