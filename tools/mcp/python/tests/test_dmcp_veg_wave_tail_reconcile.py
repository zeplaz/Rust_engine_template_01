"""DMCP veg wave tail queue reconcile."""

from __future__ import annotations

from rust_engine_mcp.dmcp_veg_wave_tail_reconcile import (
    refresh_dmcp_veg_wave_tail_reconcile_witness,
    run_veg_wave_tail_reconcile,
)


def test_dmcp_veg_wave_tail_reconcile_green() -> None:
    body = refresh_dmcp_veg_wave_tail_reconcile_witness()
    assert body.get("green") is True
    assert body.get("done_count") == 8


def test_pilot_teach_annotation() -> None:
    audit = run_veg_wave_tail_reconcile()
    pilot = next(r for r in audit["rows"] if r["id"] == "DMCP-PILOT-TEACH-ANNOT-001")
    assert pilot["pilot_teach"]["green"] is True
