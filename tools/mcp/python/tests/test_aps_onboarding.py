"""OVR-P56 onboarding guards."""

from __future__ import annotations

from rust_engine_mcp.aps_uiux_onboard import (
    ONBOARDING_PREFS_KEY,
    load_onboarding_seen,
    mark_onboarding_seen,
    refresh_aps_onboard_witness,
)
from rust_engine_mcp.paths import repo_root


def test_metadata_flow_collapsed_by_default() -> None:
    text = (repo_root() / "tools/mcp/art_pipeline_suite/metadata_flow_panel.py").read_text(
        encoding="utf-8"
    )
    assert "def _initial_expanded" in text
    assert "return False" in text.split("def _initial_expanded", 1)[1].split("def ", 1)[0]


def test_onboarding_prefs_roundtrip(tmp_path, monkeypatch) -> None:
    prefs = tmp_path / "aps_ui_prefs.json"
    monkeypatch.setattr(
        "rust_engine_mcp.aps_uiux_onboard.onboarding_prefs_path",
        lambda repo=None: prefs,
    )
    assert load_onboarding_seen() is False
    mark_onboarding_seen()
    assert load_onboarding_seen() is True
    assert ONBOARDING_PREFS_KEY in prefs.read_text(encoding="utf-8")


def test_refresh_onboard_witness() -> None:
    body = refresh_aps_onboard_witness()
    assert body.get("green") is True
    assert body.get("metadata_collapsed_default") is True
