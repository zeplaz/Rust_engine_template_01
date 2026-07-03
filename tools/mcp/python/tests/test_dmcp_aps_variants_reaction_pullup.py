"""DMCP-APS-VARIANTS-REACTION-PULLUP-001 witness tests."""

from __future__ import annotations

from rust_engine_mcp.dmcp_aps_variants_reaction_pullup import (
    GATE_ID,
    refresh_dmcp_aps_variants_reaction_pullup_witness,
    run_pullup_critique_audit,
)


def test_pullup_critique_audit_green() -> None:
    audit = run_pullup_critique_audit()
    assert audit["green"] is True
    assert audit["verdict"] == "PASS"
    assert audit["event_count"] == 11
    assert audit["confront_summary"]["horror_show"] is False


def test_pullup_witness_writes() -> None:
    body = refresh_dmcp_aps_variants_reaction_pullup_witness()
    assert body["gate"] == GATE_ID
    assert body["green"] is True
