"""APS-UX-TOKENS-001 — shared fonts, colors, wraplength helpers."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

FONT_UI = ("Segoe UI", 9)
FONT_UI_SM = ("Segoe UI", 9)
FONT_UI_BOLD = ("Segoe UI", 9, "bold")
FONT_SECTION = ("Segoe UI", 9, "bold")
FONT_MONO = ("Consolas", 10)
FONT_HINT = ("Segoe UI", 9)
# APS-UX-FONT-FLOOR — smallest font allowed on primary content labels.
# Never use a literal ("Segoe UI", 8) / ("Consolas", 8) on a content label; use this token.
FONT_SMALL = ("Segoe UI", 9)
FONT_MONO_SMALL = ("Consolas", 9)

PAD_SM = 4
PAD_MD = 8
PAD_LG = 12

COLOR_PASS = "#0a6b0a"
COLOR_FAIL = "#a00000"
COLOR_WARN = "#a66b00"
COLOR_MUTED = "#555555"
COLOR_ACCENT = "#0a4a7a"
COLOR_LANE_BUILDING = COLOR_ACCENT
COLOR_LANE_LANDSCAPE = "#1f6b54"
COLOR_PANE_BG = "#eceff3"
COLOR_PANEL_BG = "#f6f7f9"
COLOR_INPUT_BG = "#ffffff"
COLOR_SASH = "#6b8299"
COLOR_SASH_LIGHT = "#94a3b4"
COLOR_SASH_DARK = "#4a5568"
SASH_WIDTH = 7

AUTHORITY_STRIP = (
    "Ship truth: assembly_snapshot (materials + tags). Sidecar and atlas are inputs only."
)

# Viewport policy — 1080p-class production; default one step below full comfortable width.
DISPLAY_CLASS_1080P = (1920, 1080)
DESIGN_TARGET_WINDOW = (1280, 800)  # primary sign-off + default launch (720p-class height)
DEFAULT_WINDOW_SIZE = DESIGN_TARGET_WINDOW
COMFORTABLE_MAX_WINDOW = (1440, 900)  # also supported on same display when undocked / maximized-ish
MIN_WINDOW_SIZE = (960, 600)  # regression floor only — must not break


def wrap_for_widget(widget: tk.Misc, *, fraction: float = 0.92, minimum: int = 280) -> int:
    """Dynamic wraplength from parent width (call on Configure)."""
    try:
        w = int(widget.winfo_width() * fraction)
    except tk.TclError:
        w = minimum
    return max(minimum, w)


def init_aps_ttk(root: tk.Misc) -> ttk.Style:
    """Apply APS ttk theme — visible paned sash dividers + consistent chrome."""
    style = ttk.Style(root)
    if "clam" in style.theme_names():
        style.theme_use("clam")

    style.configure(".", background=COLOR_PANEL_BG, font=FONT_UI)
    style.configure("TFrame", background=COLOR_PANEL_BG)
    style.configure("TNotebook", padding=(4, 2), background=COLOR_PANEL_BG)
    style.configure("TNotebook.Tab", padding=(12, 6), font=FONT_UI)
    style.map("TNotebook.Tab", background=[("selected", COLOR_INPUT_BG)])

    style.configure("TLabelframe", background=COLOR_PANEL_BG, borderwidth=1, relief=tk.GROOVE)
    style.configure("TLabelframe.Label", background=COLOR_PANEL_BG, font=FONT_SECTION, foreground=COLOR_ACCENT)

    style.configure("TButton", padding=(10, 5))
    style.configure("TCombobox", padding=2)
    style.configure("Treeview", rowheight=24, font=FONT_UI, background=COLOR_INPUT_BG, fieldbackground=COLOR_INPUT_BG)
    style.configure("Treeview.Heading", font=FONT_UI_BOLD)

    style.configure("Aps.Toolbar.TButton", padding=(6, 2), font=FONT_UI)
    style.configure("Aps.Lane.TRadiobutton", padding=(12, 4), font=FONT_UI)
    style.map(
        "Aps.Lane.TRadiobutton",
        background=[("selected", COLOR_INPUT_BG), ("!selected", COLOR_PANEL_BG)],
    )

    for pane_style in ("Aps.Horizontal.TPanedwindow", "Aps.Vertical.TPanedwindow"):
        style.configure(pane_style, background=COLOR_PANE_BG)
        style.configure(
            f"{pane_style}.Sash",
            sashwidth=SASH_WIDTH,
            sashpad=3,
            background=COLOR_SASH,
            bordercolor=COLOR_SASH,
            lightcolor=COLOR_SASH_LIGHT,
            darkcolor=COLOR_SASH_DARK,
            gripcount=8,
        )
    return style


def track_wraplength(
    container: tk.Misc,
    *widgets: tk.Misc,
    fraction: float = 0.92,
    minimum: int = 280,
) -> None:
    """Keep label wraplength in sync when a pane or panel grows wider."""

    def _update(_event=None) -> None:
        width = wrap_for_widget(container, fraction=fraction, minimum=minimum)
        for widget in widgets:
            try:
                widget.configure(wraplength=width)
            except tk.TclError:
                pass

    container.bind("<Configure>", _update, add="+")
    container.after_idle(_update)
