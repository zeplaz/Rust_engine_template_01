"""Shared APS tab workflow chrome — intro · primary · work · file row · advanced."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_theme import COLOR_MUTED, COLOR_TEXT_SUBTLE, FONT_SMALL, FONT_UI, track_wraplength


def workflow_intro(parent: tk.Misc, text: str) -> ttk.Label:
    """One-line workflow hint at tab top."""
    lbl = ttk.Label(
        parent,
        text=text,
        wraplength=900,
        justify=tk.LEFT,
        foreground=COLOR_TEXT_SUBTLE,
        font=FONT_SMALL,
    )
    lbl.pack(anchor=tk.W, pady=(0, 4))
    track_wraplength(parent, lbl, minimum=480)
    return lbl


def workflow_lane_banner(parent: tk.Misc, *, initial: str = "") -> ttk.Label:
    lbl = ttk.Label(parent, text=initial, font=FONT_UI, foreground=COLOR_MUTED)
    lbl.pack(anchor=tk.W, pady=(0, 4))
    return lbl


def workflow_primary_row(parent: tk.Misc) -> ttk.Frame:
    """Primary actions / filters — always visible."""
    row = ttk.Frame(parent)
    row.pack(fill=tk.X, pady=(0, 4))
    return row


def workflow_file_row(parent: tk.Misc) -> ttk.Frame:
    """Load / save / validate / ship checks."""
    row = ttk.Frame(parent)
    row.pack(fill=tk.X, pady=4)
    return row


def workflow_status_label(parent: tk.Misc, *, wraplength: int = 720) -> tuple[ttk.Label, tk.StringVar]:
    var = tk.StringVar(value="")
    lbl = ttk.Label(parent, textvariable=var, wraplength=wraplength, justify=tk.LEFT, font=FONT_SMALL)
    lbl.pack(anchor=tk.W, pady=(0, 4))
    return lbl, var
