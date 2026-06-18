"""APS-GRAMMAR-SET-UI-001 / APS-GRAMMAR-SWEEP-UI-001 — grammar set brief + eval sweep strip."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Callable

from rust_engine_mcp import grammar_build_set

from .aps_theme import COLOR_TEXT_SUBTLE


class GrammarBuildSetPanel(ttk.LabelFrame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        on_log: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(master, text="Building style set", padding=6)
        self._on_log = on_log or (lambda _m: None)
        self.brief_var = tk.StringVar(value="Loading…")
        ttk.Label(self, textvariable=self.brief_var, wraplength=720, justify=tk.LEFT).pack(
            anchor=tk.W, fill=tk.X
        )
        row = ttk.Frame(self)
        row.pack(fill=tk.X, pady=4)
        ttk.Button(row, text="Refresh brief", command=self.refresh_brief).pack(side=tk.LEFT, padx=(0, 6))
        ttk.Button(row, text="Eval sweep", command=self._run_sweep).pack(side=tk.LEFT)
        self.sweep_var = tk.StringVar(value="")
        ttk.Label(self, textvariable=self.sweep_var, foreground=COLOR_TEXT_SUBTLE, wraplength=720).pack(
            anchor=tk.W, fill=tk.X
        )
        self.refresh_brief()

    def refresh_brief(self) -> None:
        try:
            body = grammar_build_set.grammar_set_brief()
            self.brief_var.set(body.get("text") or "grammar set brief unavailable")
            self._on_log(f"grammar_set_brief green={body.get('green')}")
        except Exception as exc:  # noqa: BLE001
            self.brief_var.set(f"grammar_set_brief failed: {exc}")

    def _run_sweep(self) -> None:
        try:
            body = grammar_build_set.grammar_eval_sweep()
            lines = body.get("lines") or []
            self.sweep_var.set(" · ".join(lines[:6]) or body.get("text") or "sweep empty")
            self._on_log(f"grammar_eval_sweep seeds={body.get('seed_count')}")
        except Exception as exc:  # noqa: BLE001
            self.sweep_var.set(f"sweep failed: {exc}")
