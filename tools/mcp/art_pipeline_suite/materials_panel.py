"""APS-MAT-001 — Materials tab (library studio + link to Assembly)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_paned import add_pane, vertical_paned
from .aps_tooltips import bind_aps_tooltip
from .aps_theme import track_wraplength
from .material_library_widget import MaterialLibraryWidget
from .material_preview_modes import MaterialPreviewModesPanel
from .metadata_flow_panel import MetadataFlowPanel
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
        intro = ttk.Label(
            self,
            text="Material Studio — browse, generate, and edit profiles. "
            "Drop authored PNGs into each profile folder, then Reload preview. "
            "Assign on the Assembly tab.",
            wraplength=900,
            justify=tk.LEFT,
        )
        intro.pack(anchor=tk.W, pady=(0, 4))
        track_wraplength(self, intro, minimum=480)
        self._lane_banner = ttk.Label(self, text="", font=("Segoe UI", 9), foreground="#555")
        self._lane_banner.pack(anchor=tk.W, pady=(0, 4))
        self.metadata_flow = MetadataFlowPanel(self, context="materials")
        self.metadata_flow.pack(fill=tk.X, pady=(0, 6))
        body = vertical_paned(self)
        body.pack(fill=tk.BOTH, expand=True)
        lib_wrap = ttk.Frame(body, padding=2)
        add_pane(body, lib_wrap, weight=3, minsize=280)
        self.library = MaterialLibraryWidget(
            lib_wrap,
            mode="studio",
            on_log=on_log,
            layout="studio_tree",
            on_open_in_assembly=on_open_in_assembly,
            on_profile_selected=self._on_profile_selected,
            start_job=start_job,
        )
        self.library.pack(fill=tk.BOTH, expand=True)
        preview_wrap = ttk.Frame(body, padding=2)
        add_pane(body, preview_wrap, weight=1, minsize=180)
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
            self._lane_banner.configure(text="Buildings lane — material_profile authority for assembly.")

    def _on_profile_selected(self, profile_id: str) -> None:
        self.preview_modes.set_profile(profile_id)

    def highlight_profile(self, profile_id: str | None) -> None:
        self.library.highlight_profile(profile_id)
        if profile_id:
            self.preview_modes.set_profile(profile_id)
