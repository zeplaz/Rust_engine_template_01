"""Assembly Editor (APS-UI-003b) — footprint grid, grammar, categorized semantic tags."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .assembly_footprint_section import AssemblyFootprintSectionMixin
from .assembly_grammar_section import AssemblyGrammarSectionMixin
from .assembly_metadata_section import AssemblyMetadataSectionMixin
from .assembly_panel_layout import AssemblyPanelLayoutMixin
from .assembly_preview_section import AssemblyPreviewSectionMixin
from .assembly_validation_section import AssemblyValidationSectionMixin
from .state import ArtDomain, SuiteState


class AssemblyPanel(
    AssemblyFootprintSectionMixin,
    AssemblyValidationSectionMixin,
    AssemblyPreviewSectionMixin,
    AssemblyMetadataSectionMixin,
    AssemblyGrammarSectionMixin,
    AssemblyPanelLayoutMixin,
    ttk.Frame,
):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        on_open_in_materials=None,
        start_job=None,
        assembly_service,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        self._on_open_in_materials = on_open_in_materials
        self._start_job = start_job
        if assembly_service is None:
            raise ValueError("AssemblyPanel requires assembly_service (APSR-S2)")
        self._assembly = assembly_service
        self._selected_node_id: str | None = None
        self._semantic_tag_vars: dict[str, dict[str, tk.BooleanVar]] = {}
        self._variant_tag_vars: dict[str, tk.BooleanVar] = {}
        self._tag_category_frames: dict[str, ttk.LabelFrame] = {}
        self._material_profiles: list[str] = []
        self._grammar_set_tier = "G0"
        self._build()

    @property
    def _snapshot(self) -> dict | None:
        return self._assembly.snapshot

    def _commit_snapshot(self, snap: dict) -> None:
        self._assembly.set_snapshot_data(snap)

    def set_domain(self, lane: str) -> None:
        if not hasattr(self, "_lane_banner"):
            from .aps_theme import COLOR_MUTED, FONT_UI

            self._lane_banner = ttk.Label(self, text="", font=FONT_UI, foreground=COLOR_MUTED)
            self._lane_banner.pack(anchor=tk.W, before=self.metadata_flow, pady=(0, 4))
        if lane == ArtDomain.LANDSCAPE.value:
            self._lane_banner.configure(text="Landscape lane — grammar preset authority.")
        else:
            self._lane_banner.configure(text="Buildings lane — Assembly authority.")
