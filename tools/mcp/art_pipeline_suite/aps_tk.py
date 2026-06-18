"""Themed classic tk widgets — use for Listbox, Text, Canvas chrome."""

from __future__ import annotations

import tkinter as tk
from typing import Any

from . import aps_theme


def themed_listbox(master: tk.Misc, **kwargs: Any) -> tk.Listbox:
    defaults: dict[str, Any] = {
        "bg": aps_theme.COLOR_INPUT_BG,
        "fg": aps_theme.COLOR_TEXT_BODY,
        "selectbackground": aps_theme.COLOR_SELECT_BG,
        "selectforeground": aps_theme.COLOR_TREE_SELECT_FG,
        "highlightbackground": aps_theme.COLOR_OUTLINE,
        "highlightcolor": aps_theme.COLOR_ACCENT,
        "activestyle": "none",
    }
    defaults.update(kwargs)
    return tk.Listbox(master, **defaults)


def themed_text(master: tk.Misc, **kwargs: Any) -> tk.Text:
    defaults: dict[str, Any] = {
        "bg": aps_theme.COLOR_EXPLAINER_BG,
        "fg": aps_theme.COLOR_TEXT_BODY,
        "insertbackground": aps_theme.COLOR_TEXT_BODY,
        "selectbackground": aps_theme.COLOR_SELECT_BG,
        "selectforeground": aps_theme.COLOR_TREE_SELECT_FG,
        "highlightbackground": aps_theme.COLOR_OUTLINE,
        "highlightcolor": aps_theme.COLOR_ACCENT,
        "relief": tk.FLAT,
    }
    defaults.update(kwargs)
    return tk.Text(master, **defaults)


def themed_label(master: tk.Misc, **kwargs: Any) -> tk.Label:
    defaults: dict[str, Any] = {
        "bg": aps_theme.COLOR_PANEL_BG,
        "fg": aps_theme.COLOR_TEXT_BODY,
    }
    if "background" in kwargs:
        defaults["bg"] = kwargs.pop("background")
    if "foreground" in kwargs:
        defaults["fg"] = kwargs.pop("foreground")
    defaults.update(kwargs)
    return tk.Label(master, **defaults)
