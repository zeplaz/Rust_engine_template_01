"""DMCP-OVR P2/P3 designer-mcp audit witnesses."""

from __future__ import annotations

from rust_engine_mcp.aps_uiux_p2_impl_audit import refresh_dmcp_ovr_p2_impl_audit_witness, run_p2_impl_audit
from rust_engine_mcp.aps_uiux_p3_accept_rubric import refresh_dmcp_ovr_p3_accept_rubric_witness


def test_dmcp_ovr_p2_impl_audit_witness() -> None:
    body = refresh_dmcp_ovr_p2_impl_audit_witness()
    assert body.get("audit_complete") is True
    assert body.get("gate") == "DMCP-OVR-P2-IMPL-AUDIT-001"
    assert body.get("ban_list", {}).get("clean") is True
    assert int(body.get("copy_pack", {}).get("pass") or 0) >= 20


def test_p2_impl_audit_ban_list_clean() -> None:
    audit = run_p2_impl_audit()
    assert audit["ban_list"]["clean"] is True


def test_dmcp_ovr_p3_accept_rubric_witness() -> None:
    body = refresh_dmcp_ovr_p3_accept_rubric_witness()
    assert body.get("green") is True
    assert body.get("gate") == "DMCP-OVR-P3-ACCEPT-RUBRIC-001"
