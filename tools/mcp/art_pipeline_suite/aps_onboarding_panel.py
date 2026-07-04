"""P5.6 — first-run "How this works" panel + per-tab empty states.

Content lives in :mod:`rust_engine_mcp.aps_uiux_onboard` (display-free, testable);
this module only renders it. The panel is dismissible and shown once — the seen
flag is owned by the same onboarding prefs the brief already tracks.
"""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Callable

from rust_engine_mcp.aps_uiux_onboard import (
    ONBOARDING_DISMISS,
    ONBOARDING_INTRO,
    ONBOARDING_STEPS,
    ONBOARDING_TITLE,
    empty_state_text,
)

from .aps_theme import (
    COLOR_ACCENT,
    COLOR_EXPLAINER_BG,
    COLOR_OUTLINE,
    COLOR_TEXT_BODY,
    COLOR_TEXT_HINT,
    COLOR_TEXT_SUBTLE,
    FONT_SECTION,
    FONT_SMALL,
    FONT_TITLE,
    FONT_UI_BOLD,
    GAP_LG,
    GAP_MD,
    GAP_SM,
    GAP_XS,
)
from .aps_tooltips import bind_aps_tooltip


class OnboardingPanel(ttk.Frame):
    """Dismissible first-run card — the 5-step pipeline in plain artist words."""

    def __init__(self, master: tk.Misc, *, on_dismiss: Callable[[], None] | None = None) -> None:
        super().__init__(master, padding=GAP_LG)
        self._on_dismiss = on_dismiss
        self._build()

    def _build(self) -> None:
        card = tk.Frame(
            self,
            background=COLOR_EXPLAINER_BG,
            highlightbackground=COLOR_OUTLINE,
            highlightthickness=1,
            padx=GAP_LG,
            pady=GAP_LG,
        )
        card.pack(fill=tk.BOTH, expand=True)

        tk.Label(
            card,
            text=ONBOARDING_TITLE,
            font=FONT_TITLE,
            background=COLOR_EXPLAINER_BG,
            foreground=COLOR_ACCENT,
        ).pack(anchor=tk.W)
        tk.Label(
            card,
            text=ONBOARDING_INTRO,
            font=FONT_SMALL,
            wraplength=560,
            justify=tk.LEFT,
            background=COLOR_EXPLAINER_BG,
            foreground=COLOR_TEXT_SUBTLE,
        ).pack(anchor=tk.W, pady=(GAP_SM, GAP_MD))

        steps = tk.Frame(card, background=COLOR_EXPLAINER_BG)
        steps.pack(fill=tk.X, anchor=tk.W)
        for i, (name, blurb) in enumerate(ONBOARDING_STEPS, start=1):
            row = tk.Frame(steps, background=COLOR_EXPLAINER_BG)
            row.pack(fill=tk.X, anchor=tk.W, pady=GAP_XS)
            tk.Label(
                row,
                text=f"{i}",
                font=FONT_UI_BOLD,
                width=2,
                background=COLOR_EXPLAINER_BG,
                foreground=COLOR_ACCENT,
            ).pack(side=tk.LEFT, anchor=tk.N)
            text = tk.Frame(row, background=COLOR_EXPLAINER_BG)
            text.pack(side=tk.LEFT, fill=tk.X, expand=True)
            tk.Label(
                text,
                text=name,
                font=FONT_SECTION,
                background=COLOR_EXPLAINER_BG,
                foreground=COLOR_TEXT_BODY,
            ).pack(anchor=tk.W)
            tk.Label(
                text,
                text=blurb,
                font=FONT_SMALL,
                wraplength=520,
                justify=tk.LEFT,
                background=COLOR_EXPLAINER_BG,
                foreground=COLOR_TEXT_HINT,
            ).pack(anchor=tk.W)

        btn_row = ttk.Frame(card)
        btn_row.pack(anchor=tk.E, pady=(GAP_MD, 0))
        self._dismiss_btn = ttk.Button(btn_row, text=ONBOARDING_DISMISS, command=self.dismiss)
        self._dismiss_btn.pack(side=tk.RIGHT)
        bind_aps_tooltip(self._dismiss_btn, "onboard_dismiss")

    def dismiss(self) -> None:
        if self._on_dismiss is not None:
            self._on_dismiss()
        self.destroy()


def empty_state_label(parent: tk.Misc, surface: str, *, wraplength: int = 480) -> ttk.Label:
    """Friendly per-tab empty state on a primary surface (plain artist words)."""
    lbl = ttk.Label(
        parent,
        text=empty_state_text(surface),
        font=FONT_SMALL,
        wraplength=wraplength,
        justify=tk.LEFT,
        foreground=COLOR_TEXT_HINT,
    )
    return lbl
