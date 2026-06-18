"""OVR-P1-TOKENS-001 — aps_design_system_v1.md §3 token contract."""

from __future__ import annotations

import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

import art_pipeline_suite.aps_theme as theme

theme.apply_theme("light")

import pytest

from art_pipeline_suite import aps_inline_feedback

_LIGHT = theme._LIGHT_TOKENS


@pytest.fixture(autouse=True)
def _light_theme_for_token_tests() -> None:
    theme.apply_theme("light")


def test_typography_ramp() -> None:
    assert theme.FONT_CAPTION == ("Segoe UI", 8)
    assert theme.FONT_UI == ("Segoe UI", 9)
    assert theme.FONT_SMALL == ("Segoe UI", 9)
    assert theme.FONT_SECTION == ("Segoe UI", 10, "bold")
    assert theme.FONT_MONO_SMALL == ("Consolas", 9)
    assert theme.FONT_MONO == ("Consolas", 10)
    assert theme.FONT_TITLE == ("Segoe UI", 13, "bold")
    assert theme.FONT_SMALL[1] >= 9
    assert theme.FONT_MONO_SMALL[1] >= 9


def test_color_roles() -> None:
    assert theme.COLOR_PASS == _LIGHT["COLOR_PASS"]
    assert theme.COLOR_FAIL == _LIGHT["COLOR_FAIL"]
    assert theme.COLOR_WARN == _LIGHT["COLOR_WARN"]
    assert theme.COLOR_PASS_BG == _LIGHT["COLOR_PASS_BG"]
    assert theme.COLOR_WARN_BG == _LIGHT["COLOR_WARN_BG"]
    assert theme.COLOR_FAIL_BG == _LIGHT["COLOR_FAIL_BG"]
    assert theme.COLOR_SELECT_BG == _LIGHT["COLOR_SELECT_BG"]
    assert theme.COLOR_OUTLINE == _LIGHT["COLOR_OUTLINE"]
    assert theme.COLOR_LANE_LANDSCAPE != theme.COLOR_PASS


def test_spacing_scale() -> None:
    assert theme.GAP_XS == 2
    assert theme.GAP_SM == 4
    assert theme.GAP_MD == 8
    assert theme.GAP_LG == 12
    assert theme.GAP_XL == 16
    assert theme.INSET_PANEL == 8
    assert theme.PANE_MIN_LIST == 220
    assert theme.PANE_MIN_DETAIL == 280
    assert theme.PANE_MIN_CANVAS == 320
    assert theme.ROW_HEIGHT == 24
    assert theme.SASH_WIDTH == 7


def test_status_atom_canonical() -> None:
    glyph, word, fg, bg = aps_inline_feedback.status_atom("pass")
    assert glyph == "✓"
    assert word == "valid"
    assert fg == theme.COLOR_PASS
    assert bg == theme.COLOR_PASS_BG

    glyph, word, fg, bg = aps_inline_feedback.status_atom("fail", word="blocked", detail="missing preset")
    assert glyph == "✗"
    assert word == "blocked — missing preset"
    assert fg == theme.COLOR_FAIL
    assert bg == theme.COLOR_FAIL_BG

    glyph, word, fg, bg = aps_inline_feedback.status_atom("warn")
    assert glyph == "◐"
    assert fg == theme.COLOR_WARN
    assert bg == theme.COLOR_WARN_BG

    glyph, word, fg, bg = aps_inline_feedback.status_atom("pending")
    assert glyph == "○"

    glyph, word, fg, bg = aps_inline_feedback.status_atom("working")
    assert glyph == "⟳"
    assert fg == theme.COLOR_ACCENT


@pytest.mark.aps_gui
def test_set_inline_status_strips_legacy_pass_prefix(tk_root) -> None:
    import tkinter as tk

    var = tk.StringVar()
    lbl = tk.Label(tk_root)
    aps_inline_feedback.set_inline_status(lbl, var, "PASS: schema OK", ok=True)
    assert var.get().startswith("✓")
    assert "valid" in var.get()
    assert lbl.cget("foreground") == theme.COLOR_PASS


def test_apply_status_atom_fail_word() -> None:
    glyph, word, fg, bg = aps_inline_feedback.status_atom("fail")
    assert glyph == "✗"
    assert word == "blocked"
    assert fg == theme.COLOR_FAIL
