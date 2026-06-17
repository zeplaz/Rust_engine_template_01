"""APS-EVO-E0-RELAUNCH-001 → APS-EVO-E2-PRESET-BROWSE-001 maintain bundle."""

from __future__ import annotations

from rust_engine_mcp.aps_artist_tool_e2e import (
    APS_ARTIST_TOOL_E2E_WITNESS,
    refresh_aps_e0_relaunch,
    run_pytest_aps_gate,
)
from rust_engine_mcp.aps_landscape_preset_browse import (
    APS_LANDSCAPE_PRESET_BROWSE_WITNESS,
    refresh_aps_landscape_preset_browse_witness,
)
from rust_engine_mcp.landscape_grammar_presets import landscape_grammar_presets_batch
from rust_engine_mcp.paths import repo_root


def test_auxiliary_display_strings_excluded_from_preset_batch() -> None:
    body = landscape_grammar_presets_batch()
    assert body.get("green") is True
    orphans = (body.get("index") or {}).get("orphan_preset_files") or []
    assert "_display_strings_v1" not in orphans
    failed = [r for r in (body.get("preset_validation") or {}).get("results") or [] if r.get("status") != "passed"]
    assert failed == []


def test_pytest_aps_gate_green() -> None:
    gate = run_pytest_aps_gate()
    assert gate.get("ok") is True, gate.get("summary")


def test_e2_preset_browse_witness_green() -> None:
    body = refresh_aps_landscape_preset_browse_witness()
    assert body.get("green") is True
    assert body.get("presets_listed", 0) >= 10
    assert body.get("validate_inline_green") is True
    assert (repo_root() / APS_LANDSCAPE_PRESET_BROWSE_WITNESS).is_file()


def test_e0_relaunch_bundle_green() -> None:
    bundle = refresh_aps_e0_relaunch(include_e2=True)
    assert bundle.get("green") is True
    assert bundle.get("pytest_aps", {}).get("ok") is True
    assert bundle.get("e0_witness_green") is True
    assert bundle.get("e2_witness_green") is True
    assert (repo_root() / APS_ARTIST_TOOL_E2E_WITNESS).is_file()
