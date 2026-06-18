"""OVR-P55-PREVIEW-001 — preview surface state contract."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite.aps_preview_state import fidelity_label, preview_surface_state
import art_pipeline_suite.aps_theme as aps_theme
from art_pipeline_suite.aps_theme import (
    PREVIEW_MIN_H,
    PREVIEW_THUMB_LG,
    PREVIEW_THUMB_MD,
    PREVIEW_THUMB_SM,
    apply_theme,
)

apply_theme("light")


def test_preview_surface_states_non_black_backgrounds() -> None:
    for state in ("empty", "loading", "error", "result"):
        _text, _fg, bg = preview_surface_state(state)
        assert bg.lower() not in ("#000000", "#000", "black")
        assert bg != ""


def test_preview_surface_state_copy() -> None:
    text, _fg, bg = preview_surface_state("empty", detail="No piece selected")
    assert text.startswith("○")
    assert "No piece selected" in text
    assert bg == aps_theme.COLOR_INPUT_BG

    text, _fg, bg = preview_surface_state("loading")
    assert text.startswith("⟳")
    assert bg == aps_theme.COLOR_PANEL_BG

    text, _fg, bg = preview_surface_state("error", detail="Thumb failed", hint="retry")
    assert text.startswith("◐")
    assert "Thumb failed" in text
    assert bg == aps_theme.COLOR_WARN_BG


def test_fidelity_labels() -> None:
    assert fidelity_label("quick") == "Quick preview"
    assert fidelity_label("ship") == "Ship render"


def test_preview_sizing_tokens() -> None:
    assert PREVIEW_THUMB_SM == 96
    assert PREVIEW_THUMB_MD == 128
    assert PREVIEW_THUMB_LG == 192
    assert PREVIEW_MIN_H == 120


@pytest.mark.aps_gui
def test_configure_preview_label_never_black(tk_root) -> None:
    import tkinter as tk

    from art_pipeline_suite.aps_preview_state import configure_preview_label

    lbl = tk.Label(tk_root)
    configure_preview_label(lbl, "empty", detail="Nothing selected", width=96, height=96)
    assert lbl.cget("bg").lower() not in ("#000000", "#000", "black")
