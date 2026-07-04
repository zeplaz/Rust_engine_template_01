"""Design tokens (aps_design_system_v1.md §3)."""

from __future__ import annotations

import json
import os
import tkinter as tk
from tkinter import ttk
from typing import Literal

from rust_engine_mcp.paths import repo_root

ThemeMode = Literal["light", "dark"]

_PREFS_PATH = repo_root() / "debug_runs/aps_ui_prefs.json"

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
# Semantic text/surface aliases — panels must use these, not raw hex literals.
COLOR_TEXT_SUBTLE = "#444444"
COLOR_TEXT_HINT = "#666666"
COLOR_TEXT_BODY = "#333333"
COLOR_PREVIEW_PLACEHOLDER = "#e8e8e8"
COLOR_CARD_BG = "#f4f4f4"
COLOR_GRID_MUTED = "#888888"
COLOR_EXPLAINER_BG = "#f8f8f8"
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
# DES-APS-SMOOTHNESS-001 §3 — reserve validation banner row (no layout jump).
VALIDATION_BANNER_MIN_PX = 24

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

# --- §3.7 Preview sizing (OVR-P55-PREVIEW-001) ---
PREVIEW_THUMB_SM = 96
PREVIEW_THUMB_MD = 128
PREVIEW_THUMB_LG = 192
PREVIEW_MIN_H = 120

_THEME_MODE: ThemeMode = "light"

_LIGHT_TOKENS: dict[str, str] = {
    "COLOR_PASS": "#0a6b0a",
    "COLOR_FAIL": "#a00000",
    "COLOR_WARN": "#a66b00",
    "COLOR_MUTED": "#555555",
    "COLOR_ACCENT": "#0a4a7a",
    "COLOR_PASS_BG": "#f0faf0",
    "COLOR_WARN_BG": "#fff8ee",
    "COLOR_FAIL_BG": "#fff0f0",
    "COLOR_PANEL_BG": "#f6f7f9",
    "COLOR_INPUT_BG": "#ffffff",
    "COLOR_SELECT_BG": "#e8eef5",
    "COLOR_SELECT_ACTIVE": "#cce0ff",
    "COLOR_OUTLINE": "#c8ccd4",
    "COLOR_LANE_LANDSCAPE": "#1f6b54",
    "COLOR_TEXT_SUBTLE": "#444444",
    "COLOR_TEXT_HINT": "#666666",
    "COLOR_TEXT_BODY": "#333333",
    "COLOR_PREVIEW_PLACEHOLDER": "#e8e8e8",
    "COLOR_CARD_BG": "#f4f4f4",
    "COLOR_GRID_MUTED": "#888888",
    "COLOR_EXPLAINER_BG": "#f8f8f8",
    "COLOR_PANE_BG": "#eceff3",
    "COLOR_SASH": "#6b8299",
    "COLOR_SASH_LIGHT": "#94a3b4",
    "COLOR_SASH_DARK": "#4a5568",
    "COLOR_TREE_SELECT_FG": "#000000",
}

_DARK_TOKENS: dict[str, str] = {
    "COLOR_PASS": "#6ecf6e",
    "COLOR_FAIL": "#f07070",
    "COLOR_WARN": "#e0b050",
    "COLOR_MUTED": "#9aa0ad",
    "COLOR_ACCENT": "#7eb8ff",
    "COLOR_PASS_BG": "#1e3324",
    "COLOR_WARN_BG": "#3a3020",
    "COLOR_FAIL_BG": "#3a2020",
    "COLOR_PANEL_BG": "#1c1c22",
    "COLOR_INPUT_BG": "#282830",
    "COLOR_SELECT_BG": "#364458",
    "COLOR_SELECT_ACTIVE": "#4a6080",
    "COLOR_OUTLINE": "#454550",
    "COLOR_LANE_LANDSCAPE": "#4cc9a0",
    "COLOR_TEXT_SUBTLE": "#b4b8c4",
    "COLOR_TEXT_HINT": "#9098a8",
    "COLOR_TEXT_BODY": "#e8eaf0",
    "COLOR_PREVIEW_PLACEHOLDER": "#32323a",
    "COLOR_CARD_BG": "#2e2e36",
    "COLOR_GRID_MUTED": "#707880",
    "COLOR_EXPLAINER_BG": "#24242c",
    "COLOR_PANE_BG": "#222228",
    "COLOR_SASH": "#5a6478",
    "COLOR_SASH_LIGHT": "#7888a0",
    "COLOR_SASH_DARK": "#3a4250",
    "COLOR_TREE_SELECT_FG": "#e8eaf0",
}


def load_theme_mode() -> ThemeMode:
    """Artist default: dark. Override with APS_THEME=light|dark or aps_ui_prefs.json theme."""
    env = os.environ.get("APS_THEME", "").strip().lower()
    if env in ("light", "dark"):
        return env  # type: ignore[return-value]
    if _PREFS_PATH.is_file():
        try:
            data = json.loads(_PREFS_PATH.read_text(encoding="utf-8"))
            mode = str((data or {}).get("theme") or "").lower()
            if mode in ("light", "dark"):
                return mode  # type: ignore[return-value]
        except (OSError, json.JSONDecodeError):
            pass
    return "dark"


def save_theme_mode(mode: ThemeMode) -> None:
    prefs: dict = {}
    if _PREFS_PATH.is_file():
        try:
            raw = json.loads(_PREFS_PATH.read_text(encoding="utf-8"))
            if isinstance(raw, dict):
                prefs = raw
        except (OSError, json.JSONDecodeError):
            prefs = {}
    prefs["theme"] = mode
    _PREFS_PATH.parent.mkdir(parents=True, exist_ok=True)
    _PREFS_PATH.write_text(json.dumps(prefs, indent=2) + "\n", encoding="utf-8")


def theme_mode() -> ThemeMode:
    return _THEME_MODE


def apply_theme(mode: ThemeMode | None = None) -> ThemeMode:
    """Swap module-level COLOR_* tokens (call before init_aps_ttk)."""
    global _THEME_MODE
    resolved: ThemeMode = mode or load_theme_mode()
    _THEME_MODE = resolved
    tokens = _DARK_TOKENS if resolved == "dark" else _LIGHT_TOKENS
    g = globals()
    for key, value in tokens.items():
        g[key] = value
    g["COLOR_LANE_BUILDING"] = g["COLOR_ACCENT"]
    return resolved


def wrap_for_widget(widget: tk.Misc, *, fraction: float = 0.92, minimum: int = 280) -> int:
    """Dynamic wraplength from parent width (call on Configure)."""
    try:
        w = int(widget.winfo_width() * fraction)
    except tk.TclError:
        w = minimum
    return max(minimum, w)


def init_tk_options(root: tk.Misc) -> None:
    """Global tk option database — classic widgets without explicit fg/bg."""
    opts = [
        ("*Foreground", COLOR_TEXT_BODY),
        ("*Background", COLOR_PANEL_BG),
        ("*selectForeground", COLOR_TREE_SELECT_FG),
        ("*selectBackground", COLOR_SELECT_BG),
        ("*insertBackground", COLOR_TEXT_BODY),
        ("*disabledForeground", COLOR_TEXT_HINT),
        ("*Listbox*Background", COLOR_INPUT_BG),
        ("*Listbox*Foreground", COLOR_TEXT_BODY),
        ("*Listbox*selectBackground", COLOR_SELECT_BG),
        ("*Listbox*selectForeground", COLOR_TREE_SELECT_FG),
        ("*Text*Background", COLOR_EXPLAINER_BG),
        ("*Text*Foreground", COLOR_TEXT_BODY),
        ("*Text*insertBackground", COLOR_TEXT_BODY),
        ("*Text*selectBackground", COLOR_SELECT_BG),
        ("*Text*selectForeground", COLOR_TREE_SELECT_FG),
        ("*Entry*Background", COLOR_INPUT_BG),
        ("*Entry*Foreground", COLOR_TEXT_BODY),
        ("*Canvas*Background", COLOR_CARD_BG),
        # Combobox popdown list (Windows clam).
        ("*TCombobox*Listbox*Background", COLOR_INPUT_BG),
        ("*TCombobox*Listbox*Foreground", COLOR_TEXT_BODY),
        ("*TCombobox*Listbox*selectBackground", COLOR_SELECT_BG),
        ("*TCombobox*Listbox*selectForeground", COLOR_TREE_SELECT_FG),
    ]
    for pattern, value in opts:
        try:
            root.option_add(pattern, value, 80)
        except tk.TclError:
            pass


def init_aps_ttk(root: tk.Misc) -> ttk.Style:
    """Apply APS ttk theme — visible paned sash dividers + consistent chrome."""
    apply_theme()
    try:
        root.configure(background=COLOR_PANEL_BG)
    except tk.TclError:
        pass
    init_tk_options(root)
    style = ttk.Style(root)
    if "clam" in style.theme_names():
        style.theme_use("clam")

    style.configure(".", background=COLOR_PANEL_BG, font=FONT_UI)
    style.configure("TFrame", background=COLOR_PANEL_BG)
    style.configure("TNotebook", padding=(GAP_SM, GAP_XS), background=COLOR_PANEL_BG)
    style.configure("TNotebook.Tab", padding=(GAP_LG, 6), font=FONT_UI)
    style.map(
        "TNotebook.Tab",
        background=[("selected", COLOR_INPUT_BG), ("!selected", COLOR_PANEL_BG)],
        foreground=[("selected", COLOR_TEXT_BODY), ("!selected", COLOR_MUTED)],
    )

    style.configure(
        "TLabelframe",
        background=COLOR_PANEL_BG,
        borderwidth=1,
        relief=tk.GROOVE,
        bordercolor=COLOR_OUTLINE,
    )
    style.configure("TLabelframe.Label", background=COLOR_PANEL_BG, font=FONT_SECTION, foreground=COLOR_ACCENT)

    style.configure(
        "TButton",
        padding=(10, GAP_SM),
        background=COLOR_INPUT_BG,
        foreground=COLOR_TEXT_BODY,
        bordercolor=COLOR_OUTLINE,
    )
    style.map(
        "TButton",
        background=[("active", COLOR_SELECT_BG), ("disabled", COLOR_CARD_BG)],
        foreground=[("disabled", COLOR_TEXT_HINT)],
    )
    style.configure(
        "TCombobox",
        padding=GAP_XS,
        fieldbackground=COLOR_INPUT_BG,
        background=COLOR_INPUT_BG,
        foreground=COLOR_TEXT_BODY,
        arrowcolor=COLOR_TEXT_BODY,
        bordercolor=COLOR_OUTLINE,
    )
    style.map(
        "TCombobox",
        fieldbackground=[
            ("readonly", COLOR_INPUT_BG),
            ("disabled", COLOR_CARD_BG),
        ],
        foreground=[
            ("readonly", COLOR_TEXT_BODY),
            ("disabled", COLOR_TEXT_HINT),
        ],
        arrowcolor=[("disabled", COLOR_TEXT_HINT)],
    )
    style.configure(
        "TEntry",
        fieldbackground=COLOR_INPUT_BG,
        foreground=COLOR_TEXT_BODY,
        bordercolor=COLOR_OUTLINE,
        insertcolor=COLOR_TEXT_BODY,
    )
    style.map(
        "TEntry",
        fieldbackground=[("disabled", COLOR_CARD_BG), ("readonly", COLOR_INPUT_BG)],
        foreground=[("disabled", COLOR_TEXT_HINT), ("readonly", COLOR_TEXT_BODY)],
    )
    style.configure(
        "TSpinbox",
        fieldbackground=COLOR_INPUT_BG,
        foreground=COLOR_TEXT_BODY,
        background=COLOR_PANEL_BG,
        arrowcolor=COLOR_TEXT_BODY,
        bordercolor=COLOR_OUTLINE,
        insertcolor=COLOR_TEXT_BODY,
    )
    style.map(
        "TSpinbox",
        fieldbackground=[("disabled", COLOR_CARD_BG)],
        foreground=[("disabled", COLOR_TEXT_HINT)],
        arrowcolor=[("disabled", COLOR_TEXT_HINT)],
    )
    style.configure("TLabel", background=COLOR_PANEL_BG, foreground=COLOR_TEXT_BODY)
    style.configure("TRadiobutton", background=COLOR_PANEL_BG, foreground=COLOR_TEXT_BODY)
    style.map("TRadiobutton", foreground=[("disabled", COLOR_TEXT_HINT)])
    style.configure("TCheckbutton", background=COLOR_PANEL_BG, foreground=COLOR_TEXT_BODY)
    style.map("TCheckbutton", foreground=[("disabled", COLOR_TEXT_HINT)])
    style.configure(
        "TScrollbar",
        background=COLOR_PANEL_BG,
        troughcolor=COLOR_CARD_BG,
        arrowcolor=COLOR_TEXT_BODY,
        bordercolor=COLOR_OUTLINE,
    )
    style.configure(
        "Vertical.TScrollbar",
        background=COLOR_PANEL_BG,
        troughcolor=COLOR_CARD_BG,
        arrowcolor=COLOR_TEXT_BODY,
    )
    style.configure(
        "Horizontal.TScrollbar",
        background=COLOR_PANEL_BG,
        troughcolor=COLOR_CARD_BG,
        arrowcolor=COLOR_TEXT_BODY,
    )
    style.configure(
        "Treeview",
        rowheight=ROW_HEIGHT,
        font=FONT_UI,
        background=COLOR_INPUT_BG,
        fieldbackground=COLOR_INPUT_BG,
        foreground=COLOR_TEXT_BODY,
    )
    style.configure("Treeview.Heading", font=FONT_UI_BOLD, background=COLOR_PANEL_BG, foreground=COLOR_TEXT_BODY)
    style.map(
        "Treeview",
        background=[("selected", COLOR_SELECT_BG)],
        foreground=[("selected", COLOR_TREE_SELECT_FG)],
    )

    style.configure("Aps.Toolbar.TButton", padding=(6, GAP_XS), font=FONT_UI, foreground=COLOR_TEXT_BODY)
    style.configure("Aps.Lane.TRadiobutton", padding=(GAP_LG, GAP_SM), font=FONT_UI, foreground=COLOR_TEXT_BODY)
    style.map(
        "Aps.Lane.TRadiobutton",
        background=[("selected", COLOR_INPUT_BG), ("!selected", COLOR_PANEL_BG)],
        foreground=[("selected", COLOR_TEXT_BODY), ("!selected", COLOR_MUTED)],
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
