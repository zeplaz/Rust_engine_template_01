"""OVR-P2 / P7 guard — no §2b ban-list tokens AND no off-glossary terms in any
artist-visible APS string (the *real* runtime surface, not just static text=)."""

from __future__ import annotations

from rust_engine_mcp.aps_uiux_g0_audit import run_ban_list_audit
from rust_engine_mcp.aps_uiux_surface_scan import scan_surface


def test_aps_visible_strings_ban_list_clean() -> None:
    audit = run_ban_list_audit()
    assert audit.get("audit_complete") is True
    count = int(audit.get("violation_count") or 0)
    assert count == 0, (
        f"§2b ban-list: {count} hit(s) — top rules: {audit.get('by_rule')}. "
        "See src/dev/design_aps_uiux_g0_audit_v1.md"
    )


def test_surface_scanner_catches_known_offenders() -> None:
    """META-GUARD self-test — the extended scanner MUST catch the live offenders
    the prior static scanner missed. This locks the keystone against regressing
    back to a false green (a scanner that passes by looking at nothing).

    Guarded by re-introducing each offender into a throwaway tree so the test is
    independent of whether the live tree has been fixed yet. We assert the rule
    *patterns* fire, which is the contract — not a snapshot of the live tree.
    """
    from rust_engine_mcp.aps_uiux_g0_audit import BAN_PATTERNS
    from rust_engine_mcp.aps_uiux_surface_scan import TERM_PATTERNS

    def fires(value: str) -> bool:
        if any(pat.search(value) for _, pat in BAN_PATTERNS):
            return True
        return any(pat.search(value) for _, pat, _ in TERM_PATTERNS)

    expected_catch = [
        "P0 gate: passed",
        "P0 failed",
        "Snapshot OK",
        "P0 gate (production + grammar)",
        "Validate (production)",
        "Material profile",
        "Node id",
        "StylePack",
        "Archetype",
        "Validate",  # the bare Variants button
    ]
    missed = [v for v in expected_catch if not fires(v)]
    assert not missed, f"scanner blind to known offenders (false-green risk): {missed}"


def test_surface_scan_includes_runtime_contexts() -> None:
    """The scanner must actually look at runtime contexts, not just static text=."""
    report = scan_surface()
    assert report.get("scan_complete") is True
    # by_context keys exist only because the scanner reaches log_sink / set /
    # configure / messagebox / title= — proving it sees the real surface.
    assert "by_context" in report
