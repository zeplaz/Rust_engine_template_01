"""OVR-P56-ONBOARD-001 — assembly tab onboarding strip (replaces MetadataFlowPanel)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from rust_engine_mcp.aps_uiux_onboard import ONBOARDING_INTRO, ONBOARDING_STEPS, ONBOARDING_TITLE

from . import aps_theme
from .aps_theme import FONT_SMALL, FONT_UI, FONT_UI_BOLD
from .aps_tooltips import bind_aps_tooltip


class AssemblyOnboardStrip(ttk.LabelFrame):
    """Collapsible five-step pipeline guide — collapsed by default per onboard spec v2."""

    def __init__(self, master: tk.Misc) -> None:
        super().__init__(master, text=ONBOARDING_TITLE, padding=6)
        self._expanded = tk.BooleanVar(value=False)
        head = ttk.Frame(self)
        head.pack(fill=tk.X)
        chk = ttk.Checkbutton(
            head,
            text="Show the five-step pipeline",
            variable=self._expanded,
            command=self._toggle,
        )
        chk.pack(side=tk.LEFT)
        bind_aps_tooltip(chk, "onboard_steps")
        self._collapsed_hint = ttk.Label(
            head,
            text=ONBOARDING_INTRO,
            font=FONT_UI,
            foreground=aps_theme.COLOR_ACCENT,
            wraplength=680,
        )
        self._body = ttk.Frame(self)
        steps = tk.Frame(self._body, background=aps_theme.COLOR_EXPLAINER_BG)
        steps.pack(fill=tk.X, anchor=tk.W, padx=4, pady=4)
        for i, (name, blurb) in enumerate(ONBOARDING_STEPS, start=1):
            row = tk.Frame(steps, background=aps_theme.COLOR_EXPLAINER_BG)
            row.pack(fill=tk.X, anchor=tk.W, pady=2)
            tk.Label(
                row,
                text=f"{i}. {name}",
                font=FONT_UI_BOLD,
                background=aps_theme.COLOR_EXPLAINER_BG,
                foreground=aps_theme.COLOR_TEXT_BODY,
            ).pack(side=tk.LEFT, padx=(0, 6))
            tk.Label(
                row,
                text=blurb,
                font=FONT_SMALL,
                background=aps_theme.COLOR_EXPLAINER_BG,
                foreground=aps_theme.COLOR_TEXT_HINT,
                wraplength=560,
                justify=tk.LEFT,
            ).pack(side=tk.LEFT, fill=tk.X, expand=True)
        self._sync_collapsed_hint()

    def _sync_collapsed_hint(self) -> None:
        if self._expanded.get():
            self._collapsed_hint.pack_forget()
        else:
            self._collapsed_hint.pack(side=tk.LEFT, padx=(12, 0), fill=tk.X, expand=True)

    def _toggle(self) -> None:
        if self._expanded.get():
            self._body.pack(fill=tk.BOTH, expand=True, pady=(6, 0))
        else:
            self._body.pack_forget()
        self._sync_collapsed_hint()
