"""DMCP-OVR-G0-AUDIT-001 witness + ban-list scanner tests."""

from __future__ import annotations

from rust_engine_mcp.aps_uiux_g0_audit import refresh_dmcp_ovr_g0_audit_witness, run_ban_list_audit


def test_dmcp_ovr_g0_audit_witness_complete() -> None:
    body = refresh_dmcp_ovr_g0_audit_witness()
    assert body.get("audit_complete") is True
    assert body.get("green") is True
    assert body.get("gate") == "DMCP-OVR-G0-AUDIT-001"
    assert int(body.get("violation_count") or 0) == 0
    assert body.get("ui_clean") is True


def test_ban_list_audit_returns_structure() -> None:
    audit = run_ban_list_audit()
    assert "by_rule" in audit
    assert "by_file" in audit
    assert "violations" in audit
