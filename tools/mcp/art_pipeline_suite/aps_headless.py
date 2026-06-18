"""Headless Tk for APS pytest — no visible windows during ``pytest -k aps``."""

from __future__ import annotations

import os
import tkinter as tk


def headless_tests_enabled() -> bool:
    raw = os.environ.get("APS_TEST_HEADLESS", "").strip().lower()
    return raw in ("1", "true", "yes", "on")


def apply_headless_root(root: tk.Misc) -> None:
    """Hide and shrink a Tk root before/after widget build (Windows-safe)."""
    if not headless_tests_enabled():
        return
    try:
        root.withdraw()
        # Off-screen 1×1 — avoids taskbar flash while keeping layout measurable.
        root.geometry("1x1+-20000+-20000")
        try:
            root.wm_attributes("-alpha", 0.0)
        except tk.TclError:
            pass
        root.update_idletasks()
    except tk.TclError:
        pass


def layout_widget_visible(widget: tk.Misc, *, min_height: int = 1) -> bool:
    """Headless-safe visibility: withdrawn roots make ``winfo_viewable`` always 0."""
    if not widget.winfo_exists():
        return False
    widget.update_idletasks()
    if headless_tests_enabled():
        h = max(int(widget.winfo_height()), int(widget.winfo_reqheight()))
        return h >= min_height
    return bool(widget.winfo_viewable())
