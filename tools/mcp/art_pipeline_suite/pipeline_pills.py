"""APS-E1-PIPELINE-LANE-001 — pipeline pill formatting per design_aps_pipeline_pills_v1.md."""

from __future__ import annotations

import tkinter as tk

from . import aps_theme
from .aps_inline_feedback import format_status_line, status_atom


def pill_bg_map() -> dict[str, str]:
    return {
        "pending": aps_theme.COLOR_INPUT_BG,
        "saved_qc_not_run": aps_theme.COLOR_WARN_BG,
        "valid": aps_theme.COLOR_PASS_BG,
        "fail": aps_theme.COLOR_FAIL_BG,
    }


def pill_fg_map() -> dict[str, str]:
    return {
        "pending": aps_theme.COLOR_MUTED,
        "saved_qc_not_run": aps_theme.COLOR_WARN,
        "valid": aps_theme.COLOR_PASS,
        "fail": aps_theme.COLOR_FAIL,
    }


# Back-compat alias — refreshed on each read via pill_bg_map().
PILL_BG = pill_bg_map()
PILL_FG = pill_fg_map()


def format_pill(label: str, state_key: str) -> tuple[str, str, str]:
    """Return (display_text, bg, fg) for a pipeline pill."""
    if state_key == "pending" or state_key == "stamp_pending":
        text = format_status_line("pending", word=f"{label} pending")
        style_key = "pending"
    elif state_key == "valid" or state_key == "stamp_done":
        text = format_status_line("pass", word=f"{label} valid")
        style_key = "valid"
    elif state_key == "fail":
        text = format_status_line("fail", word=f"{label} blocked")
        style_key = "fail"
    elif state_key in ("saved_qc_not_run", "presets_loaded", "grammar_saved", "atlas_packed"):
        text = format_status_line("warn", word=f"{label} saved (not checked)")
        style_key = "saved_qc_not_run"
    else:
        text = format_status_line("pending", word=f"{label} pending")
        style_key = "pending"
    _glyph, _word, fg, bg = status_atom(
        "pass" if style_key == "valid" else "fail" if style_key == "fail" else "warn" if style_key == "saved_qc_not_run" else "pending"
    )
    bg_map, fg_map = pill_bg_map(), pill_fg_map()
    return text, bg_map.get(style_key, aps_theme.COLOR_INPUT_BG), fg_map.get(style_key, aps_theme.COLOR_MUTED)


def apply_pill(widget: tk.Frame, label_widget: tk.Label, label: str, state_key: str) -> None:
    text, bg, fg = format_pill(label, state_key)
    widget.configure(background=bg)
    label_widget.configure(text=text, foreground=fg, background=bg)
