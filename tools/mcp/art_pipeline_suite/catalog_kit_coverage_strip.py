"""APSR-Q2 — Catalog tab kit-coverage strip (BQ-K2 audit consumer)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from rust_engine_mcp import kit_coverage_audit

from .aps_inline_feedback import set_inline_status
from .aps_theme import FONT_SMALL, VALIDATION_BANNER_MIN_PX
from .aps_tooltips import bind_aps_tooltip


class CatalogKitCoverageStrip(ttk.LabelFrame):
    """Per-pack slot completeness from style-pack RON vs promoted GLB index."""

    def __init__(self, master: tk.Misc) -> None:
        super().__init__(master, text="Kit coverage (BQ-K2)", padding=6)
        self._var = tk.StringVar(value="Kit coverage: refresh to scan style packs.")
        holder = ttk.Frame(self, height=VALIDATION_BANNER_MIN_PX)
        holder.pack(fill=tk.X)
        holder.pack_propagate(False)
        self._lbl = ttk.Label(holder, textvariable=self._var, wraplength=880, font=FONT_SMALL)
        self._lbl.pack(anchor=tk.W, fill=tk.X)
        row = ttk.Frame(self)
        row.pack(anchor=tk.W, pady=(4, 0))
        refresh = ttk.Button(row, text="Refresh audit", command=self.refresh)
        refresh.pack(side=tk.LEFT)
        bind_aps_tooltip(refresh, "cat_refresh")

    def refresh(self) -> None:
        text, ok = kit_coverage_audit.format_kit_coverage_summary()
        set_inline_status(self._lbl, self._var, text, ok=ok)
