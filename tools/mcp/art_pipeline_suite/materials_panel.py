"""APS-MAT-001 — Materials tab (library studio + link to Assembly)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .material_library_widget import MaterialLibraryWidget
from .state import SuiteState


class MaterialsPanel(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        on_open_in_assembly=None,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        ttk.Label(
            self,
            text="Material Studio — browse, generate, and edit profiles. "
            "Drop authored PNGs into each profile folder, then Reload preview. "
            "Assign on the Assembly tab.",
            wraplength=900,
            justify=tk.LEFT,
        ).pack(anchor=tk.W, pady=(0, 8))
        self.library = MaterialLibraryWidget(
            self,
            mode="studio",
            on_log=on_log,
            layout="horizontal",
            on_open_in_assembly=on_open_in_assembly,
        )
        self.library.pack(fill=tk.BOTH, expand=True)

    def highlight_profile(self, profile_id: str | None) -> None:
        self.library.highlight_profile(profile_id)
