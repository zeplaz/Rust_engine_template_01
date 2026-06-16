"""APS paned layouts — visible sash dividers + stretch-friendly pane adds."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

HORIZONTAL_STYLE = "Aps.Horizontal.TPanedwindow"
VERTICAL_STYLE = "Aps.Vertical.TPanedwindow"


def horizontal_paned(master: tk.Misc, **kwargs) -> ttk.Panedwindow:
    return ttk.Panedwindow(master, orient=tk.HORIZONTAL, style=HORIZONTAL_STYLE, **kwargs)


def vertical_paned(master: tk.Misc, **kwargs) -> ttk.Panedwindow:
    return ttk.Panedwindow(master, orient=tk.VERTICAL, style=VERTICAL_STYLE, **kwargs)


def add_pane(
    paned: ttk.Panedwindow,
    child: tk.Widget,
    *,
    weight: int = 1,
    minsize: int = 160,
) -> None:
    """Add a pane with proportional stretch and a sensible drag floor."""
    paned.add(child, weight=weight)
    try:
        paned.paneconfigure(child, minsize=minsize)
    except tk.TclError:
        pass


def set_initial_pane_widths(
    paned: ttk.Panedwindow,
    specs: list[tuple[tk.Widget, float]],
    *,
    min_total: int = 400,
) -> None:
    """Set sash positions once from leading pane fractions (does not fight user drags on resize)."""

    def _once(_event=None) -> None:
        if getattr(paned, "_aps_initial_sash_done", False):
            return
        try:
            total = paned.winfo_width()
            if total < min_total:
                return
            cumulative = 0
            for sash_index, (_pane, fraction) in enumerate(specs):
                cumulative += max(int(total * fraction), 160)
                paned.sashpos(sash_index, cumulative)
            paned._aps_initial_sash_done = True  # type: ignore[attr-defined]
        except tk.TclError:
            pass

    paned.bind("<Configure>", _once, add="+")
