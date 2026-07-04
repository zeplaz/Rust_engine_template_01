"""APSR-P1 — AssemblyFootprintSectionMixin."""
from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk
from typing import Any

from rust_engine_mcp import aps_tags, arch_build_grammar, assembly, building_grammar, grammar_build_set, library
from rust_engine_mcp.aps_grammar_labels import human_label
from rust_engine_mcp.aps_mat_auth_ui import save_hint
from rust_engine_mcp.aps_validator_plain import format_p0_display
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate

from .aps_collapsible import CollapsibleSection
from .aps_inline_feedback import set_inline_status
from .aps_tooltips import bind_aps_tooltip
from .assembly_panel_common import MATERIAL_AUTHORITY_COPY, grammar_combo_maps, is_dark_color


class AssemblyFootprintSectionMixin:
    def _placement_label(self, p: dict) -> str:
        token = p.get("token", "?")
        gx, gy, fl = p.get("grid_x"), p.get("grid_y"), p.get("floor")
        mat = p.get("material_profile") or "—"
        return f"f{fl} ({gx},{gy}) {token}  {p.get('module_id')}  [{mat}]"

    def _sorted_placements(self) -> list[dict]:
        if not self._snapshot:
            return []
        return sorted(
            self._snapshot.get("module_placements") or [],
            key=lambda p: (int(p.get("floor") or 0), int(p.get("grid_y") or 0), int(p.get("grid_x") or 0)),
        )

    def _refresh_placement_list(self) -> None:
        self.placement_list.delete(0, tk.END)
        self._set_empty_state_visible(self._snapshot is None)
        for p in self._sorted_placements():
            self.placement_list.insert(tk.END, self._placement_label(p))

    def _set_empty_state_visible(self, visible: bool) -> None:
        if not hasattr(self, "_empty_state"):
            return
        if visible and not self._empty_state.winfo_ismapped():
            self._empty_state.pack(anchor=tk.W, pady=2, before=self.placement_list)
        elif not visible and self._empty_state.winfo_ismapped():
            self._empty_state.pack_forget()

    def _refresh_footprint_grid(self) -> None:
        if not self._snapshot:
            self.footprint_canvas.set_cells([], [])
            return
        cells = assembly.footprint_cells_for_snapshot(self._snapshot)
        self.footprint_canvas.clear_rule_highlight()
        self.footprint_canvas.set_cells(cells, self._sorted_placements())

    def _refresh_snapshot_presentation(self, snap: dict, *, path_hint: str = "") -> None:
        self.iterate_panel.set_base_snapshot(snap)
        self.grammar_dna_panel.set_from_snapshot(snap)
        self.path_var.set(path_hint or self.state.assembly_snapshot_path or "(memory)")
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self.grammar_inspector.load_snapshot(snap)
        self.assembly_preview.set_snapshot(snap)
        self.save_hint_var.set(save_hint(snap))
        self._refresh_collapsible_titles()
        if self.placement_list.size():
            self.placement_list.selection_set(0)
            self.on_placement_select()
        self.refresh_generation_trace()

    def _load_snapshot_into_ui(self, snap: dict, *, path_hint: str = "") -> None:
        enriched = self._assembly.set_snapshot_data(snap)
        self._assembly.reset_p0_verdict()
        if hasattr(self, "_gen_trace"):
            self._gen_trace.reset_approval()
        self._sync_state_from_snapshot(enriched)
        self._refresh_snapshot_presentation(enriched, path_hint=path_hint)
        if hasattr(self, "_qc_strip"):
            aid = str(enriched.get("assembly_id") or self.state.assembly_id or "")
            self._qc_strip.refresh(aid or None)

    def refresh_generation_trace(self) -> None:
        if hasattr(self, "_gen_trace"):
            self._gen_trace.refresh()

    def sync_from_state(self) -> None:
        """Mirror shell/catalog ``SuiteState`` into panel chrome and live snapshot."""
        self.style_var.set(self.state.style_pack_id)
        self.footprint_var.set(self.state.footprint)
        self.floors_var.set(self.state.floors)
        self.seed_var.set(self.state.seed)
        if self._assembly.snapshot:
            patched = self._assembly.patch_snapshot_from_shell()
            if patched:
                self._refresh_snapshot_presentation(patched)

    def _sync_state_from_snapshot(self, snap: dict) -> None:
        self._assembly.sync_shell_from_snapshot(snap)
        self.style_var.set(self.state.style_pack_id)
        fp = snap.get("footprint") or {}
        w, d, f = fp.get("width"), fp.get("depth"), fp.get("floors")
        if w and d:
            self.footprint_var.set(f"{w}x{d}")
        if f:
            self.floors_var.set(int(f))
        if snap.get("seed") is not None:
            self.seed_var.set(int(snap["seed"]))
        if snap.get("archetype_id"):
            self.use_grammar_var.set(True)
            self.archetype_var.set(human_label(str(snap["archetype_id"])))
            self._on_archetype_change()
            if snap.get("district_style"):
                self.district_var.set(human_label(str(snap["district_style"])))
            self._on_grammar_toggle()

    def _refresh_module_picker_values(self) -> None:
        style_pack = str((self._snapshot or {}).get("style_pack_id") or self.state.style_pack_id or "")
        rows = library.search_modules(style_pack=style_pack or None)
        if not rows:
            rows = library.load_index_json()
        ids = sorted({str(r.get("module_id") or "") for r in rows if r.get("module_id")})
        self.module_combo.configure(values=ids)

    def _refresh_module_resolve_label(self, module_id: str) -> None:
        if not module_id:
            self._module_resolve_var.set("")
            return
        snap = self._snapshot or {}
        body = assembly.explain_module_resolve(
            module_id,
            style_pack_id=str(snap.get("style_pack_id") or self.state.style_pack_id or ""),
            source_tier=str(snap.get("source_tier") or self.tier_var.get() or "production"),
        )
        self._module_resolve_var.set(str(body.get("label") or ""))

    def _on_module_picked(self, _event=None) -> None:
        if not self._snapshot or not self._selected_node_id:
            self._set_validation_result("Select a placement row first.", ok=False)
            return
        module_id = self.module_var.get().strip()
        if not module_id:
            return
        try:
            updated = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                module_id=module_id,
            )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Module apply failed: {exc}", ok=False)
            return
        self._commit_snapshot(updated)
        self._refresh_module_resolve_label(module_id)
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._refresh_facility_needs()
        self.on_placement_select()
        self._on_log(f"module {module_id} → {self._selected_node_id}")
        self._set_validation_result(f"Module {module_id} applied — Save snapshot before bake", ok=None)

    def _select_placement_at(self, gx: int, gy: int, floor: int) -> None:
        placements = self._sorted_placements()
        for idx, p in enumerate(placements):
            if (
                int(p.get("grid_x") or 0) == gx
                and int(p.get("grid_y") or 0) == gy
                and int(p.get("floor") or 0) == floor
            ):
                self.placement_list.selection_clear(0, tk.END)
                self.placement_list.selection_set(idx)
                self.placement_list.see(idx)
                self.on_placement_select()
                return

    def _on_grid_cell_select(self, gx: int, gy: int, floor: int) -> None:
        self._select_placement_at(gx, gy, floor)

    def on_placement_select(self, _event=None) -> None:
        if not self._snapshot:
            return
        sel = self.placement_list.curselection()
        if not sel:
            return
        placements = self._sorted_placements()
        idx = int(sel[0])
        if idx >= len(placements):
            return
        p = placements[idx]
        self._selected_node_id = assembly.placement_node_id(p)
        self.node_id_var.set(self._selected_node_id)
        self.module_var.set(str(p.get("module_id") or ""))
        self._refresh_module_picker_values()
        self._refresh_module_resolve_label(str(p.get("module_id") or ""))
        mat = str(p.get("material_profile") or "")
        self.material_var.set(mat or "—")
        self._update_material_category(mat)
        self._update_material_swatch(mat)
        self.lod_var.set(str(p.get("lod_policy") or "production"))
        semantic = p.get("semantic_tags") or aps_tags.semantic_tags_from_flat(p.get("placement_tags") or [])
        for cat, tag_map in self._semantic_tag_vars.items():
            active = set(semantic.get(cat) or [])
            for tag_id, var in tag_map.items():
                var.set(tag_id in active)
        vtags = set(p.get("variant_tags") or [])
        for tag, var in self._variant_tag_vars.items():
            var.set(tag in vtags)
        self.footprint_canvas.set_selection(
            int(p.get("grid_x") or 0), int(p.get("grid_y") or 0), int(p.get("floor") or 0)
        )
        if mat:
            self.material_browser.highlight_profile(mat)
        chain = self._snapshot.get("grammar_rule_chain") if self._snapshot else None
        if isinstance(chain, dict):
            self.slot_preview.show_placement(p, snapshot=self._snapshot, grammar_chain=chain)
        else:
            self.slot_preview.show_placement(p, snapshot=self._snapshot)
        self._refresh_collapsible_titles()
