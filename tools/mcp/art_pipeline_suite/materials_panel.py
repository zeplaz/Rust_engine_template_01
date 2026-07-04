"""APS-MAT-001 — Materials tab (library studio + preview modes)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_paned import add_pane, horizontal_paned
from .aps_tooltips import bind_aps_tooltip
from .aps_theme import COLOR_MUTED, FONT_SMALL
from .aps_workflow_layout import workflow_intro, workflow_lane_banner, workflow_primary_row
from .material_browser import mount_material_library
from .material_preview_modes import MaterialPreviewModesPanel
from .state import ArtDomain, SuiteState


class MaterialsPanel(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        on_open_in_assembly=None,
        start_job=None,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log

        workflow_intro(
            self,
            "Pick a material, preview it on the right, then assign on Assembly — that assignment is what ships.",
        )
        self._lane_banner = workflow_lane_banner(self)

        primary = workflow_primary_row(self)
        ttk.Label(
            primary,
            text="Studio library",
            font=("Segoe UI", 9, "bold"),
        ).pack(side=tk.LEFT)
        ttk.Label(
            primary,
            text="  ·  Drop PNGs in each profile folder, Reload preview, Use in Assembly when ready",
            foreground=COLOR_MUTED,
            font=FONT_SMALL,
        ).pack(side=tk.LEFT)

        body = horizontal_paned(self)
        body.pack(fill=tk.BOTH, expand=True, pady=4)
        lib_wrap = ttk.Frame(body, padding=2)
        add_pane(body, lib_wrap, weight=3, minsize=320)
        self.library = mount_material_library(
            lib_wrap,
            mount="studio",
            on_log=on_log,
            on_open_in_assembly=on_open_in_assembly,
            on_profile_selected=self._on_profile_selected,
            start_job=start_job,
        )
        self.library.pack(fill=tk.BOTH, expand=True)

        preview_wrap = ttk.Frame(body, padding=2)
        add_pane(body, preview_wrap, weight=2, minsize=240)
        self.preview_modes = MaterialPreviewModesPanel(preview_wrap, on_log=on_log)
        self.preview_modes.pack(fill=tk.BOTH, expand=True)
        bind_aps_tooltip(self.preview_modes, "mat_preview_modes")
        self.library.bind_tooltips()

    def set_domain(self, lane: str) -> None:
        if lane == ArtDomain.LANDSCAPE.value:
            self._lane_banner.configure(
                text="Landscape lane — material profiles remain buildings-only until E3 cross-lane profiles."
            )
        else:
            self._lane_banner.configure(text="Buildings lane — materials you assign on Assembly are what ships.")

    def _on_profile_selected(self, profile_id: str) -> None:
        self.preview_modes.set_profile(profile_id)

    def highlight_profile(self, profile_id: str | None) -> None:
        self.library.highlight_profile(profile_id)
        if profile_id:
            self.preview_modes.set_profile(profile_id)
