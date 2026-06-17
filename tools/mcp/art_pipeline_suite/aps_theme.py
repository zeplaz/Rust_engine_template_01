"""Design tokens (aps_design_system_v1.md §3)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

# --- §3.1 Typography ramp ---
FONT_CAPTION = ("Segoe UI", 8)  # decorative only — allowlisted in font-floor guard
FONT_UI = ("Segoe UI", 9)
FONT_UI_SM = ("Segoe UI", 9)
FONT_HINT = ("Segoe UI", 9)
FONT_SMALL = ("Segoe UI", 9)
FONT_UI_BOLD = ("Segoe UI", 9, "bold")
FONT_SECTION = ("Segoe UI", 10, "bold")
FONT_MONO_SMALL = ("Consolas", 9)
FONT_MONO = ("Consolas", 10)
FONT_TITLE = ("Segoe UI", 13, "bold")

# --- §3.2 Color roles ---
COLOR_PASS = "#0a6b0a"
COLOR_FAIL = "#a00000"
COLOR_WARN = "#a66b00"
COLOR_MUTED = "#555555"
COLOR_ACCENT = "#0a4a7a"
COLOR_PASS_BG = "#f0faf0"
COLOR_WARN_BG = "#fff8ee"
COLOR_FAIL_BG = "#fff0f0"
COLOR_PANEL_BG = "#f6f7f9"
COLOR_INPUT_BG = "#ffffff"
COLOR_SELECT_BG = "#e8eef5"
COLOR_SELECT_ACTIVE = "#cce0ff"
COLOR_OUTLINE = "#c8ccd4"
COLOR_LANE_BUILDING = COLOR_ACCENT
COLOR_LANE_LANDSCAPE = "#1f6b54"
COLOR_PANE_BG = "#eceff3"
COLOR_SASH = "#6b8299"
COLOR_SASH_LIGHT = "#94a3b4"
COLOR_SASH_DARK = "#4a5568"

# --- §3.3 Spacing scale (4px grid) ---
GAP_XS = 2
GAP_SM = 4
GAP_MD = 8
GAP_LG = 12
GAP_XL = 16
INSET_PANE = 8
INSET_PANEL = 8
PANE_MIN_LIST = 220
PANE_MIN_DETAIL = 280
PANE_MIN_CANVAS = 320
ROW_HEIGHT = 24
SASH_WIDTH = 7

# Back-compat aliases (prefer GAP_* in new code)
PAD_SM = GAP_SM
PAD_MD = GAP_MD
PAD_LG = GAP_LG

AUTHORITY_STRIP = (
    "What ships: the Assembly you save here (its materials + tags). "
    "Catalog data and atlas tiles only feed into it."
)

DISPLAY_CLASS_1080P = (1920, 1080)
DESIGN_TARGET_WINDOW = (1280, 800)
DEFAULT_WINDOW_SIZE = DESIGN_TARGET_WINDOW
COMFORTABLE_MAX_WINDOW = (1440, 900)
MIN_WINDOW_SIZE = (960, 600)


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
    style.configure("TNotebook", padding=(GAP_SM, GAP_XS), background=COLOR_PANEL_BG)
    style.configure("TNotebook.Tab", padding=(GAP_LG, 6), font=FONT_UI)
    style.map("TNotebook.Tab", background=[("selected", COLOR_INPUT_BG)])

    style.configure(
        "TLabelframe",
        background=COLOR_PANEL_BG,
        borderwidth=1,
        relief=tk.GROOVE,
        bordercolor=COLOR_OUTLINE,
    )
    style.configure("TLabelframe.Label", background=COLOR_PANEL_BG, font=FONT_SECTION, foreground=COLOR_ACCENT)

    style.configure("TButton", padding=(10, GAP_SM))
    style.configure("TCombobox", padding=GAP_XS)
    style.configure(
        "Treeview",
        rowheight=ROW_HEIGHT,
        font=FONT_UI,
        background=COLOR_INPUT_BG,
        fieldbackground=COLOR_INPUT_BG,
    )
    style.configure("Treeview.Heading", font=FONT_UI_BOLD)
    style.map(
        "Treeview",
        background=[("selected", COLOR_SELECT_BG)],
        foreground=[("selected", "#000000")],
    )

    style.configure("Aps.Toolbar.TButton", padding=(6, GAP_XS), font=FONT_UI)
    style.configure("Aps.Lane.TRadiobutton", padding=(GAP_LG, GAP_SM), font=FONT_UI)
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
