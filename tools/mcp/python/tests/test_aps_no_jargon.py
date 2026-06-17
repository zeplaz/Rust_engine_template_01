"""OVR-P2 guard — no §2b ban-list tokens in visible APS strings (pre-P2: expected FAIL)."""

from __future__ import annotations

from rust_engine_mcp.aps_uiux_g0_audit import run_ban_list_audit


def test_aps_visible_strings_ban_list_clean() -> None:
    audit = run_ban_list_audit()
    assert audit.get("audit_complete") is True
    count = int(audit.get("violation_count") or 0)
    assert count == 0, (
        f"§2b ban-list: {count} hit(s) — top rules: {audit.get('by_rule')}. "
        "See src/dev/design_aps_uiux_g0_audit_v1.md"
    )
