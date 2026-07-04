"""APSR-P1 — AssemblyPreviewSectionMixin."""
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


class AssemblyPreviewSectionMixin:
    def _on_assembly_preview_thumb(self, image, _result: dict) -> None:
        """P1 — assembly-level Bevy/browser PNG → slot placement context thumb."""
        self.slot_preview.set_assembly_context_image(image)
        sel = self.placement_list.curselection()
        if sel and self._snapshot:
            placements = self._sorted_placements()
            idx = int(sel[0])
            if idx < len(placements):
                chain = (self._snapshot or {}).get("grammar_rule_chain")
                self.slot_preview.show_placement(
                    placements[idx],
                    snapshot=self._snapshot,
                    grammar_chain=chain if isinstance(chain, dict) else None,
                )
        self._on_log("assembly preview thumb → slot context")

    def on_preview_assembly(self) -> None:
        if not self._run_p0_or_block("Preview"):
            self._on_log("preview cancelled — ship check failed")
            return
        self.assembly_preview.on_preview()

    def _update_material_category(self, profile_id: str) -> None:
        if not profile_id or profile_id == "—":
            self.material_category_var.set("")
            return
        try:
            from rust_engine_mcp.material_profiles import load_material_profile_catalog

            entry = next(
                (e for e in load_material_profile_catalog() if e.profile_id == profile_id),
                None,
            )
            if entry:
                self.material_category_var.set(f"category: {entry.category}")
            else:
                from rust_engine_mcp.material_profiles import infer_category

                self.material_category_var.set(f"category: {infer_category(profile_id)}")
        except Exception:
            self.material_category_var.set("")

    def _update_material_swatch(self, profile_id: str) -> None:
        color = "#dddddd"
        if profile_id:
            try:
                from rust_engine_mcp.material_profiles import ensure_profile_textures

                entry = ensure_profile_textures(profile_id, size=64)
                path = entry.albedo_path
                if path and path.is_file():
                    from PIL import Image

                    img = Image.open(path).convert("RGB")
                    img.thumbnail((16, 16))
                    r, g, b = img.getpixel((4, 4))
                    color = f"#{r:02x}{g:02x}{b:02x}"
            except Exception:
                if "steel" in profile_id:
                    color = "#6a7f94"
                elif "brick" in profile_id:
                    color = "#9e4a38"
                elif "wood" in profile_id:
                    color = "#7a5a3a"
        # APS-UX-NONCOLOR — carry identity as text on the swatch so material is
        # distinguishable in grayscale / colorblind, not by color block alone.
        swatch_text = "—"
        if profile_id:
            head = profile_id.split("_")[0]
            swatch_text = head[:3].upper() if head else profile_id[:3].upper()
        fg = "#ffffff" if is_dark_color(color) else "#111111"
        self._material_swatch.configure(bg=color, text=swatch_text, fg=fg)

    def _apply_material_profile(self, profile_id: str) -> None:
        if not self._snapshot or not self._selected_node_id:
            self._set_validation_result("Select a placement row or grid cell first.", ok=False)
            return
        self.material_var.set(profile_id)
        self._update_material_category(profile_id)
        self._update_material_swatch(profile_id)
        try:
            updated = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                material_profile=profile_id,
            )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Material apply failed: {exc}", ok=False)
            return
        self._commit_snapshot(updated)
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._on_log(f"material {profile_id} → {self._selected_node_id}")
        self._set_validation_result(f"Material {profile_id} applied — Save snapshot before bake", ok=None)

    def show_material_assign_callout(self, profile_id: str) -> None:
        if self._snapshot:
            self.next_step_var.set(
                f"Material {profile_id} highlighted — select a footprint cell, then Apply to selected piece."
            )
        else:
            self.next_step_var.set("Generate or load an Assembly first, then select a footprint cell.")
        bind_aps_tooltip(self._next_step_lbl, "asm_material_lib")

    def _on_material_browser_apply(self, profile_id: str) -> None:
        self.material_var.set(profile_id)
        if self._snapshot and self._selected_node_id:
            self.on_apply_slot()
        else:
            self._on_log(f"material selected: {profile_id} (pick a slot to apply)")
