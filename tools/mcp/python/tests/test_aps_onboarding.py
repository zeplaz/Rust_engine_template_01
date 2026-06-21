"""OVR-P56 onboarding guards."""

from __future__ import annotations

import pytest

from rust_engine_mcp.aps_uiux_onboard import (
    EMPTY_STATES,
    ONBOARDING_PREFS_KEY,
    ONBOARDING_STEPS,
    empty_state_text,
    load_onboarding_seen,
    mark_onboarding_seen,
    onboarding_greeting_lines,
    refresh_aps_onboard_witness,
)
from rust_engine_mcp.paths import repo_root


def test_metadata_flow_collapsed_by_default() -> None:
    text = (repo_root() / "tools/mcp/art_pipeline_suite/assembly_onboard_strip.py").read_text(
        encoding="utf-8"
    )
    assert "AssemblyOnboardStrip" in text
    assert "value=False" in text


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


# --- P5.6 first-run content (real, not just the seen-flag) ---

EXPECTED_STEP_NAMES = ("Catalog", "Materials", "Assembly", "Variants", "Atlas")


def test_onboarding_greeting_covers_five_named_steps() -> None:
    names = tuple(name for name, _blurb in ONBOARDING_STEPS)
    assert names == EXPECTED_STEP_NAMES
    lines = onboarding_greeting_lines()
    # Title + intro + one line per step.
    assert len(lines) == 2 + len(EXPECTED_STEP_NAMES)
    assert lines[0] == "How this works"
    body = "\n".join(lines)
    for i, name in enumerate(EXPECTED_STEP_NAMES, start=1):
        assert f"{i}. {name} —" in body


def test_empty_states_exist_for_primary_surfaces() -> None:
    for surface in ("catalog", "materials", "assembly", "variants", "atlas"):
        text = empty_state_text(surface)
        assert text and text in EMPTY_STATES.values()
    # The assembly empty state matches the plan's example phrasing.
    assert empty_state_text("assembly") == "No assembly yet — Generate one to begin."


@pytest.mark.aps_gui
def test_onboarding_panel_renders_title_steps_and_dismiss(tk_root) -> None:
    """The first-run card renders the title, all 5 step names, and a working dismiss."""
    from art_pipeline_suite.aps_onboarding_panel import OnboardingPanel

    dismissed = {"v": False}
    panel = OnboardingPanel(tk_root, on_dismiss=lambda: dismissed.__setitem__("v", True))
    panel.place(x=-9000, y=-9000, width=600, height=480)
    tk_root.update_idletasks()

    seen_text: list[str] = []

    def _collect(widget) -> None:
        if "text" in widget.keys():
            seen_text.append(str(widget.cget("text")))
        for child in widget.winfo_children():
            _collect(child)

    _collect(panel)
    blob = "\n".join(seen_text)
    assert "How this works" in blob
    for name in EXPECTED_STEP_NAMES:
        assert name in blob
    # Dismiss invokes the callback and destroys the panel.
    panel.dismiss()
    tk_root.update_idletasks()
    assert dismissed["v"] is True
    assert not panel.winfo_exists()


@pytest.mark.aps_gui
def test_app_first_run_decision_shows_panel_once(tk_root, monkeypatch) -> None:
    """_maybe_onboarding shows the panel when unseen and skips it when seen."""
    seen = {"v": False}
    monkeypatch.setattr(
        "rust_engine_mcp.aps_uiux_onboard.load_onboarding_seen", lambda **_k: seen["v"]
    )
    monkeypatch.setattr(
        "rust_engine_mcp.aps_uiux_onboard.mark_onboarding_seen",
        lambda **_k: seen.__setitem__("v", True),
    )
    # Clear any existing panel from app construction.
    if getattr(tk_root, "_onboarding_panel", None) is not None:
        tk_root._dismiss_onboarding()
    tk_root._onboarding_panel = None

    tk_root._maybe_onboarding()
    tk_root.update_idletasks()
    assert tk_root._onboarding_panel is not None  # first run → shown
    assert seen["v"] is True
    tk_root._dismiss_onboarding()
    tk_root.update_idletasks()
    assert tk_root._onboarding_panel is None

    # Second run (now seen) → no panel.
    tk_root._maybe_onboarding()
    tk_root.update_idletasks()
    assert tk_root._onboarding_panel is None


@pytest.mark.aps_gui
def test_app_renders_empty_states_on_primary_surfaces(tk_root) -> None:
    # Variants: no data → friendly empty state visible.
    tk_root.variants._data = None
    tk_root.variants._refresh_list()
    tk_root.update_idletasks()
    assert tk_root.variants._empty_state.winfo_ismapped()
    assert tk_root.variants._empty_state.cget("text") == empty_state_text("variants")
    # Loading content hides it.
    tk_root.variants._data = {"variants": [{"variant_key": "clean_day"}]}
    tk_root.variants._refresh_list()
    tk_root.update_idletasks()
    assert not tk_root.variants._empty_state.winfo_ismapped()
    tk_root.variants._data = None
    tk_root.variants._refresh_list()
    # Assembly: no snapshot → empty state visible.
    tk_root.assembly._snapshot = None
    tk_root.assembly._refresh_placement_list()
    tk_root.update_idletasks()
    assert tk_root.assembly._empty_state.winfo_ismapped()
    assert tk_root.assembly._empty_state.cget("text") == empty_state_text("assembly")
