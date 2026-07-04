"""APSR-P1 — AssemblyMetadataSectionMixin."""
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


class AssemblyMetadataSectionMixin:
    def _build_semantic_tag_pickers(self, parent: ttk.Frame) -> None:
        labels = aps_tags.category_labels()
        for cat in aps_tags.CATEGORY_ORDER:
            frame = ttk.LabelFrame(parent, text=labels.get(cat, cat.title()), padding=4)
            frame.pack(fill=tk.X, pady=2)
            self._tag_category_frames[cat] = frame
            grid = ttk.Frame(frame)
            grid.pack(anchor=tk.W)
            self._semantic_tag_vars[cat] = {}
            for i, row in enumerate(aps_tags.tags_for_category(cat)):
                tag_id = str(row.get("id") or "")
                label = str(row.get("label") or tag_id)
                var = tk.BooleanVar(value=False)
                self._semantic_tag_vars[cat][tag_id] = var
                cb = ttk.Checkbutton(grid, text=label, variable=var)
                cb.grid(row=i // 3, column=i % 3, sticky=tk.W, padx=4)
                grammar_use = str(row.get("grammar_use") or "")
                from rust_engine_mcp.aps_tag_vocabulary import semantic_tag_hint

                bind_aps_tooltip(cb, f"asm_semantic_tag:{tag_id}")
                cb.bind(
                    "<Enter>",
                    lambda _e, tid=tag_id, gu=grammar_use: self._show_tag_hint(
                        semantic_tag_hint(tid, grammar_use=gu)
                    ),
                    add="+",
                )

    def _show_tag_hint(self, text: str) -> None:
        if hasattr(self, "next_step_var"):
            self.next_step_var.set(text[:220])

    def _apply_tag_category_filter(self) -> None:
        filt = self.tag_filter_var.get().strip().lower()
        for cat, frame in self._tag_category_frames.items():
            if filt == "all" or filt == cat:
                frame.pack(fill=tk.X, pady=2)
            else:
                frame.pack_forget()

    def _on_apply_semantic_preset(self) -> None:
        from rust_engine_mcp.aps_tag_tier2 import preset_confirm_lines, preset_for_archetype

        archetype_id = self._resolve_archetype_id()
        row = preset_for_archetype(archetype_id)
        if not row:
            self._on_log(f"Tag preset: no tier-2 preset for {archetype_id}")
            return
        lines = "\n".join(preset_confirm_lines(archetype_id))
        if not messagebox.askyesno("Apply tag preset?", f"{lines}\n\nApply semantic tags to the picker?"):
            return
        semantic = row.get("semantic_tags") or {}
        for cat, tag_ids in semantic.items():
            tag_map = self._semantic_tag_vars.get(str(cat)) or {}
            for tid in tag_ids or []:
                var = tag_map.get(str(tid))
                if var is not None:
                    var.set(True)
        self._on_log(f"semantic tag preset {row.get('preset_name')} applied — Save tags to this piece to commit")

    def _count_active_tags(self) -> int:
        n = sum(1 for tag_map in self._semantic_tag_vars.values() for var in tag_map.values() if var.get())
        n += sum(1 for var in self._variant_tag_vars.values() if var.get())
        return n

    def _refresh_collapsible_titles(self) -> None:
        tag_title = "Tags (look & state)"
        n = self._count_active_tags()
        if n:
            tag_title = f"{tag_title} ({n} selected)"
        self._tag_section.set_title(tag_title)
        self._grammar_section.set_title(self._grammar_section_title())

    def _open_in_materials_tab(self) -> None:
        mat = self.material_var.get().strip()
        if mat and mat != "—" and self._on_open_in_materials:
            self._on_open_in_materials(mat)

    def _collect_semantic_tags(self) -> dict[str, list[str]]:
        out: dict[str, list[str]] = {}
        for cat, tag_map in self._semantic_tag_vars.items():
            picked = [tid for tid, var in tag_map.items() if var.get()]
            if picked:
                out[cat] = picked
        return out

    def on_apply_slot(self) -> None:
        if not self._snapshot or not self._selected_node_id:
            self._set_validation_result("Select a placement row first.", ok=False)
            return
        semantic_tags = self._collect_semantic_tags()
        variant_tags = [t for t, v in self._variant_tag_vars.items() if v.get()]
        try:
            updated = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                material_profile=self.material_var.get().strip(),
                semantic_tags=semantic_tags,
                variant_tags=variant_tags,
                lod_policy=self.lod_var.get().strip(),
            )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Apply failed: {exc}", ok=False)
            return
        self._commit_snapshot(updated)
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._on_log(f"updated {self._selected_node_id} material={self.material_var.get()}")
        self._set_validation_result("Slot updated — run Validate before bake", ok=None)
