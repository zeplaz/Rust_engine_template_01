"""OVR-P1-TOKENS-001 — aps_design_system_v1.md §3 token contract."""

from __future__ import annotations

import sys
from pathlib import Path

APS_ROOT = Path(__file__).resolve().parents[2]
if str(APS_ROOT) not in sys.path:
    sys.path.insert(0, str(APS_ROOT))

from art_pipeline_suite import aps_inline_feedback
from art_pipeline_suite.aps_theme import (
    COLOR_ACCENT,
    COLOR_FAIL,
    COLOR_FAIL_BG,
    COLOR_LANE_LANDSCAPE,
    COLOR_OUTLINE,
    COLOR_PASS,
    COLOR_PASS_BG,
    COLOR_SELECT_BG,
    COLOR_WARN,
    COLOR_WARN_BG,
    FONT_CAPTION,
    FONT_MONO,
    FONT_MONO_SMALL,
    FONT_SECTION,
    FONT_SMALL,
    FONT_TITLE,
    FONT_UI,
    GAP_LG,
    GAP_MD,
    GAP_SM,
    GAP_XL,
    GAP_XS,
    INSET_PANEL,
    PANE_MIN_CANVAS,
    PANE_MIN_DETAIL,
    PANE_MIN_LIST,
    ROW_HEIGHT,
    SASH_WIDTH,
)


def test_typography_ramp() -> None:
    assert FONT_CAPTION == ("Segoe UI", 8)
    assert FONT_UI == ("Segoe UI", 9)
    assert FONT_SMALL == ("Segoe UI", 9)
    assert FONT_SECTION == ("Segoe UI", 10, "bold")
    assert FONT_MONO_SMALL == ("Consolas", 9)
    assert FONT_MONO == ("Consolas", 10)
    assert FONT_TITLE == ("Segoe UI", 13, "bold")
    assert FONT_SMALL[1] >= 9
    assert FONT_MONO_SMALL[1] >= 9


def test_color_roles() -> None:
    assert COLOR_PASS == "#0a6b0a"
    assert COLOR_FAIL == "#a00000"
    assert COLOR_WARN == "#a66b00"
    assert COLOR_PASS_BG == "#f0faf0"
    assert COLOR_WARN_BG == "#fff8ee"
    assert COLOR_FAIL_BG == "#fff0f0"
    assert COLOR_SELECT_BG == "#e8eef5"
    assert COLOR_OUTLINE == "#c8ccd4"
    assert COLOR_LANE_LANDSCAPE != COLOR_PASS


def test_spacing_scale() -> None:
    assert GAP_XS == 2
    assert GAP_SM == 4
    assert GAP_MD == 8
    assert GAP_LG == 12
    assert GAP_XL == 16
    assert INSET_PANEL == 8
    assert PANE_MIN_LIST == 220
    assert PANE_MIN_DETAIL == 280
    assert PANE_MIN_CANVAS == 320
    assert ROW_HEIGHT == 24
    assert SASH_WIDTH == 7


def test_status_atom_canonical() -> None:
    glyph, word, fg, bg = aps_inline_feedback.status_atom("pass")
    assert glyph == "✓"
    assert word == "valid"
    assert fg == COLOR_PASS
    assert bg == COLOR_PASS_BG

    glyph, word, fg, bg = aps_inline_feedback.status_atom("fail", word="blocked", detail="missing preset")
    assert glyph == "✗"
    assert word == "blocked — missing preset"
    assert fg == COLOR_FAIL
    assert bg == COLOR_FAIL_BG

    glyph, word, fg, bg = aps_inline_feedback.status_atom("warn")
    assert glyph == "◐"
    assert fg == COLOR_WARN
    assert bg == COLOR_WARN_BG

    glyph, word, fg, bg = aps_inline_feedback.status_atom("pending")
    assert glyph == "○"

    glyph, word, fg, bg = aps_inline_feedback.status_atom("working")
    assert glyph == "⟳"
    assert fg == COLOR_ACCENT
