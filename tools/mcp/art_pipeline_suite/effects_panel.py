"""VSS-T4-005 — Effects tab shell (browse / preview ladder / assign)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_tooltips import bind_aps_tooltip
from .aps_theme import COLOR_MUTED, FONT_SMALL
from .aps_workflow_layout import workflow_intro, workflow_lane_banner, workflow_primary_row
from .effect_browser import mount_effect_library
from .state import ArtDomain, SuiteState


class EffectsPanel(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        on_assign_effect=None,
        **_kwargs,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log

        workflow_intro(
            self,
            "Browse registry effects, check preview honesty, then assign to a scenario marker — "
            "this panel does not draw particles in the game.",
        )
        self._lane_banner = workflow_lane_banner(self)

        primary = workflow_primary_row(self)
        ttk.Label(
            primary,
            text="Effect library",
            font=("Segoe UI", 9, "bold"),
        ).pack(side=tk.LEFT)
        ttk.Label(
            primary,
            text="  ·  honest_gate gates Assign — thumbs/captures only, no live fire_vfx preview",
            foreground=COLOR_MUTED,
            font=FONT_SMALL,
        ).pack(side=tk.LEFT)

        self.library = mount_effect_library(
            self,
            mount="browse",
            on_log=on_log,
            on_assign_effect=on_assign_effect or (lambda _e, _m: None),
        )
        self.library.pack(fill=tk.BOTH, expand=True, pady=4)
        self.library.bind_tooltips()
        bind_aps_tooltip(self.library, "tab_effects")

    def set_domain(self, lane: str) -> None:
        if lane == ArtDomain.LANDSCAPE.value:
            self._lane_banner.configure(
                text="Landscape lane — Effects remain buildings-notebook (scenario markers shared)."
            )
        else:
            self._lane_banner.configure(
                text="Buildings lane — assign EffectSpecs to scenario trigger markers when honest."
            )
