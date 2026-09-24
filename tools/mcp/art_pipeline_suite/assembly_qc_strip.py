"""APSR-A4-Q1-001 — Assembly tab QC strip (BQ-A2 / BQ-F3 + BQ-Q2 screen consumer)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from rust_engine_mcp import bq_q2_screen, building_quality_qc

from .aps_inline_feedback import set_inline_status
from .aps_theme import FONT_SMALL


class AssemblyQcStrip(ttk.LabelFrame):
    """Read-only QC summary from `building_quality_live` + Q2 screenshot lane."""

    def __init__(self, master: tk.Misc) -> None:
        super().__init__(master, text="Assembly QC (BQ-A2 / Q2)", padding=6)
        self._var = tk.StringVar(value="Building QC: load or generate an assembly to refresh.")
        self._lbl = ttk.Label(
            self,
            textvariable=self._var,
            wraplength=880,
            justify=tk.LEFT,
            font=FONT_SMALL,
        )
        self._lbl.pack(anchor=tk.W, fill=tk.X)
        self._q2_var = tk.StringVar(value="Q2 screen: run bq-q2-screen for preview paths.")
        self._q2_lbl = ttk.Label(
            self,
            textvariable=self._q2_var,
            wraplength=880,
            justify=tk.LEFT,
            font=FONT_SMALL,
        )
        self._q2_lbl.pack(anchor=tk.W, fill=tk.X, pady=(4, 0))

    def refresh(
        self,
        assembly_id: str | None = None,
        *,
        snapshot: dict | None = None,
    ) -> None:
        text, ok = building_quality_qc.format_qc_strip_text(
            assembly_id,
            snapshot=snapshot,
        )
        set_inline_status(self._lbl, self._var, text, ok=ok)
        q2_text, q2_ok = bq_q2_screen.format_q2_strip_text(assembly_id)
        set_inline_status(self._q2_lbl, self._q2_var, q2_text, ok=q2_ok)
