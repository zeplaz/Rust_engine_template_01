"""APS-E1-PIPELINE-LANE-001 — pipeline pill formatting per design_aps_pipeline_pills_v1.md."""

from __future__ import annotations

import tkinter as tk

from .aps_theme import COLOR_FAIL, COLOR_INPUT_BG, COLOR_MUTED, COLOR_PASS, COLOR_WARN

PILL_BG = {
    "pending": COLOR_INPUT_BG,
    "saved_qc_not_run": "#fff8ee",
    "valid": "#f0faf0",
    "fail": "#fff0f0",
}

PILL_FG = {
    "pending": COLOR_MUTED,
    "saved_qc_not_run": COLOR_WARN,
    "valid": COLOR_PASS,
    "fail": COLOR_FAIL,
}


def format_pill(label: str, state_key: str) -> tuple[str, str, str]:
    """Return (display_text, bg, fg) for a pipeline pill."""
    templates = {
        "pending": f"○ {label} pending",
        "saved_qc_not_run": f"◐ {label} saved (QC not run)",
        "valid": f"✓ {label} valid",
        "fail": f"✗ {label} FAIL",
        "stamp_pending": f"○ {label} pending",
        "stamp_done": f"✓ {label} registered",
        "presets_loaded": f"◐ {label} loaded (QC not run)",
        "grammar_saved": f"◐ {label} saved (QC not run)",
        "atlas_packed": f"◐ {label} packed (QC not run)",
    }
    text = templates.get(state_key, templates["pending"])
    style_key = state_key
    if state_key in ("stamp_done",):
        style_key = "valid"
    elif state_key in ("stamp_pending",):
        style_key = "pending"
    elif state_key in ("presets_loaded", "grammar_saved", "atlas_packed"):
        style_key = "saved_qc_not_run"
    return text, PILL_BG.get(style_key, COLOR_INPUT_BG), PILL_FG.get(style_key, COLOR_MUTED)


def apply_pill(widget: tk.Frame, label_widget: tk.Label, label: str, state_key: str) -> None:
    text, bg, fg = format_pill(label, state_key)
    widget.configure(background=bg)
    label_widget.configure(text=text, foreground=fg, background=bg)
