"""APSR-A4-Q1-001 — Assembly tab QC strip (BQ-A2 / BQ-F3 witness consumer)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from rust_engine_mcp import building_quality_qc

from .aps_inline_feedback import set_inline_status
from .aps_theme import FONT_SMALL, FONT_UI_BOLD


class AssemblyQcStrip(ttk.LabelFrame):
    """Read-only QC summary from `debug_runs/building_quality_live.json`."""

    def __init__(self, master: tk.Misc) -> None:
        super().__init__(master, text="Assembly QC (BQ-A2)", padding=6)
        self._var = tk.StringVar(value="Building QC: load or generate an assembly to refresh.")
        self._lbl = ttk.Label(
            self,
            textvariable=self._var,
            wraplength=880,
            justify=tk.LEFT,
            font=FONT_SMALL,
        )
        self._lbl.pack(anchor=tk.W, fill=tk.X)

    def refresh(self, assembly_id: str | None = None) -> None:
        text, ok = building_quality_qc.format_qc_strip_text(assembly_id)
        set_inline_status(self._lbl, self._var, text, ok=ok)
