"""APS-E1-PIPELINE-LANE-001 — validity-aware pipeline pills (lane-scoped)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from typing import Callable

from .aps_theme import FONT_HINT
from .aps_tooltips import bind_aps_tooltip
from .domain_router import pipeline_steps_for
from .pipeline_pills import apply_pill
from .state import ArtDomain, SuiteState


class PipelineStatusBar(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_step_click: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(master, padding=(0, 4))
        self.state = state
        self._on_step_click = on_step_click
        self._current_key: str | None = None
        self._lane = state.art_domain
        self._steps: list[tuple[str, str]] = list(pipeline_steps_for(state.art_domain))
        self._pills: dict[str, tuple[tk.Frame, tk.Label]] = {}
        ttk.Label(self, text="Pipeline:", font=("Segoe UI", 9, "bold")).pack(side=tk.LEFT, padx=(0, 6))
        self._step_frame = ttk.Frame(self)
        self._step_frame.pack(side=tk.LEFT)
        self._hint = ttk.Label(self, text="", font=FONT_HINT, foreground="#555")
        self._hint.pack(side=tk.LEFT, padx=(12, 0))
        self._rebuild_step_widgets()
        self._set_lane_hint()

    def _set_lane_hint(self) -> None:
        if self.state.art_domain == ArtDomain.LANDSCAPE.value:
            self._hint.configure(
                text="Final landscape tile art is signed off separately from passing the schema and bake checks."
            )
        else:
            self._hint.configure(
                text="You can build, assign materials, and preview without baking tiles. Tile bake happens on the Atlas step."
            )

    def _rebuild_step_widgets(self) -> None:
        for child in self._step_frame.winfo_children():
            child.destroy()
        self._pills.clear()
        self._steps = list(pipeline_steps_for(self._lane))
        for key, label in self._steps:
            pill = tk.Frame(self._step_frame, relief=tk.RIDGE, borderwidth=1, padx=6, pady=2)
            pill.pack(side=tk.LEFT, padx=4)
            lbl = tk.Label(pill, text=f"○ {label} pending", font=("Segoe UI", 9))
            lbl.pack()
            self._pills[key] = (pill, lbl)
            bind_aps_tooltip(lbl, f"pipeline_{key}")
            if self._on_step_click is not None:
                pill.configure(cursor="hand2")
                lbl.configure(cursor="hand2")
                pill.bind("<Button-1>", lambda _e, k=key: self._on_step_click(k))
                lbl.bind("<Button-1>", lambda _e, k=key: self._on_step_click(k))

    def set_domain(self, lane: str) -> None:
        self._lane = lane
        self._rebuild_step_widgets()
        self._set_lane_hint()
        self.refresh()

    def _step_label(self, key: str, default: str) -> str:
        for step_key, label in self._steps:
            if step_key == key:
                return label
        return default

    def set_current(self, key: str | None) -> None:
        self._current_key = key
        self._sync_current_markers()

    def _sync_current_markers(self) -> None:
        for step_key, (pill, lbl) in self._pills.items():
            text = lbl.cget("text")
            if step_key == self._current_key and not text.startswith("▣"):
                lbl.configure(text=f"▣ {text}")
            elif step_key != self._current_key and text.startswith("▣ "):
                lbl.configure(text=text[2:])

    def _apply(self, key: str, state_key: str) -> None:
        if key not in self._pills:
            return
        pill, lbl = self._pills[key]
        apply_pill(pill, lbl, self._step_label(key, key.title()), state_key)
        if key == self._current_key:
            lbl.configure(text=f"▣ {lbl.cget('text').lstrip('▣ ')}")

    def refresh(self) -> None:
        if self.state.art_domain == ArtDomain.LANDSCAPE.value:
            self._refresh_landscape()
            return
        self._refresh_buildings()

    def _refresh_buildings(self) -> None:
        s = self.state
        if s.selected_module_id or s.selected_module_ids:
            self._apply("catalog", "valid")
        else:
            self._apply("catalog", "pending")
        has_snapshot = bool(s.assembly_snapshot_path or s.assembly_snapshot_data)
        if not has_snapshot:
            self._apply("assembly", "pending")
        elif s.assembly_p0_passed is True:
            self._apply("assembly", "valid")
        elif s.assembly_p0_passed is False:
            self._apply("assembly", "fail")
        else:
            self._apply("assembly", "saved_qc_not_run")
        if s.assembly_snapshot_data and _has_material_profiles(s.assembly_snapshot_data):
            self._apply("materials", "valid")
        elif s.assembly_snapshot_data:
            self._apply("materials", "saved_qc_not_run")
        else:
            self._apply("materials", "pending")
        if s.variant_set_data or s.variant_set_path:
            self._apply("variants", "valid")
        else:
            self._apply("variants", "pending")
        if s.atlas_folder or s.tile_batch_path:
            self._apply("atlas", "atlas_packed")
        else:
            self._apply("atlas", "pending")

    def _refresh_landscape(self) -> None:
        s = self.state
        if s.landscape_preset_validate_ok is True:
            self._apply("presets", "valid")
        elif s.landscape_preset_validate_ok is False:
            self._apply("presets", "fail")
        elif s.selected_landscape_preset_id:
            self._apply("presets", "presets_loaded")
        else:
            self._apply("presets", "pending")
        if s.landscape_preset_validate_ok is True and s.landscape_grammar_saved:
            self._apply("grammar", "valid")
        elif s.landscape_grammar_saved:
            self._apply("grammar", "grammar_saved")
        elif s.selected_landscape_preset_id:
            self._apply("grammar", "pending")
        else:
            self._apply("grammar", "pending")
        if s.landscape_states_ready:
            self._apply("states", "valid")
        elif s.landscape_grammar_saved:
            self._apply("states", "saved_qc_not_run")
        else:
            self._apply("states", "pending")
        if s.atlas_folder or s.tile_batch_path:
            state_key = "valid" if s.landscape_stamp_registered else "atlas_packed"
            self._apply("atlas", state_key)
        else:
            self._apply("atlas", "pending")


def _has_material_profiles(snapshot: dict) -> bool:
    for row in snapshot.get("module_placements") or []:
        if isinstance(row, dict) and row.get("material_profile"):
            return True
    return False
