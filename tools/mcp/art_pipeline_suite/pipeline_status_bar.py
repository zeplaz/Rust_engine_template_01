"""APS-UX-PIPELINE-001 — artist workflow step indicators."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .aps_theme import FONT_HINT
from .aps_tooltips import bind_aps_tooltip
from .state import SuiteState


class PipelineStatusBar(ttk.Frame):
    STEPS = (
        ("catalog", "Catalog"),
        ("assembly", "Assembly"),
        ("materials", "Materials"),
        ("variants", "Variants"),
        ("atlas", "Atlas"),
    )

    def __init__(self, master: tk.Misc, state: SuiteState) -> None:
        super().__init__(master, padding=(0, 4))
        self.state = state
        self._vars: dict[str, tk.StringVar] = {}
        self._labels: dict[str, ttk.Label] = {}
        ttk.Label(self, text="Pipeline:", font=("Segoe UI", 9, "bold")).pack(side=tk.LEFT, padx=(0, 6))
        for key, label in self.STEPS:
            var = tk.StringVar(value=f"○ {label} pending")
            self._vars[key] = var
            lbl = ttk.Label(self, textvariable=var, font=("Segoe UI", 9))
            lbl.pack(side=tk.LEFT, padx=6)
            self._labels[key] = lbl
            bind_aps_tooltip(lbl, f"pipeline_{key}")
        ttk.Label(
            self,
            text="Keyframe bake is behind Atlas — Assembly/Materials/Preview work without ship proof.",
            font=FONT_HINT,
            foreground="#555",
        ).pack(side=tk.LEFT, padx=(12, 0))

    def refresh(self) -> None:
        s = self.state
        self._set(
            "catalog",
            bool(s.selected_module_id or s.selected_module_ids),
        )
        self._set(
            "assembly",
            bool(s.assembly_snapshot_path or s.assembly_snapshot_data),
        )
        self._set(
            "materials",
            bool(s.assembly_snapshot_data and _has_material_profiles(s.assembly_snapshot_data)),
        )
        self._set("variants", bool(s.variant_set_data or s.variant_set_path))
        self._set("atlas", bool(s.atlas_folder or s.tile_batch_path))

    def _set(self, key: str, ok: bool) -> None:
        label = next(l for k, l in self.STEPS if k == key)
        state = "complete" if ok else "pending"
        mark = "✓" if ok else "○"
        self._vars[key].set(f"{mark} {label} {state}")


def _has_material_profiles(snapshot: dict) -> bool:
    for row in snapshot.get("module_placements") or []:
        if isinstance(row, dict) and row.get("material_profile"):
            return True
    return False
