"""APSR-P1 — AssemblyGrammarSectionMixin."""
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


class AssemblyGrammarSectionMixin:
    _TIER_STRIP_LABELS = {
        "G0": "G0 — pilot kit",
        "G1": "G1 — family seed",
        "G2": "G2 — axis coverage",
        "G3": "G3 — layer depth",
        "G4": "G4 — production set",
    }

    def _refresh_set_health_strip(self) -> None:
        """G2+ promoted strip — brief gaps + sweep entry point."""
        try:
            brief = grammar_build_set.grammar_set_brief()
            gaps = brief.get("gaps") or []
            if gaps:
                self._set_health_var.set(f"Set health: {gaps[0]}")
            elif brief.get("green"):
                self._set_health_var.set("Set health: OK — run sweep to verify massing spread")
            else:
                self._set_health_var.set(str(brief.get("text") or "Set health: refresh brief"))
        except Exception as exc:  # noqa: BLE001
            self._set_health_var.set(f"Set health: unavailable ({exc})")

    def _on_set_health_sweep(self) -> None:
        self.grammar_set_panel._run_sweep()
        self._refresh_set_health_strip()
        sweep = self.grammar_set_panel.sweep_var.get()
        if sweep:
            self._set_health_var.set(f"Set health: {sweep[:160]}")

    def refresh_grammar_tier_from_registry(self) -> None:
        body = grammar_build_set.grammar_set_tier()
        tier = str(body.get("tier") or "G0").upper()
        self.apply_grammar_tier(tier)

    def apply_grammar_tier(self, tier: str) -> None:
        """APS-GRAM-TIER-002 — show/hide grammar surfaces per exposure table."""
        tier = str(tier or "G0").upper()
        if tier not in grammar_build_set.TIER_ORDER:
            tier = "G0"
        self._grammar_set_tier = tier
        self._grammar_set_tier_var.set(self._TIER_STRIP_LABELS.get(tier, tier))
        self.facility_needs.set_grammar_tier(tier)
        self.site_layout.set_grammar_tier(tier)
        self._refresh_facility_needs()

        if tier == "G0":
            self._grammar_kit_var.set(
                "Only one building type in the kit — add grammar files to unlock the full family."
            )
            self._kit_hint_label.pack(anchor=tk.W, pady=(2, 0))
        else:
            self._grammar_kit_var.set("")
            self._kit_hint_label.pack_forget()

        if tier in ("G2", "G3", "G4"):
            self._refresh_set_health_strip()
            if not self._set_health_strip.winfo_ismapped():
                self._set_health_strip.pack(fill=tk.X, pady=(4, 0), before=self._next_step_frame)
        else:
            self._set_health_strip.pack_forget()

        archetypes = building_grammar.list_archetype_ids() or ["IndustrialWarehouse"]
        arch_labels, self._archetype_label_to_id = grammar_combo_maps(archetypes)
        self.archetype_combo.configure(values=arch_labels or [""])
        if arch_labels and self.archetype_var.get() not in arch_labels:
            self.archetype_var.set(arch_labels[0])
            self._on_archetype_change()

        dna_mode = "hidden"
        iterate_mode = "hidden"
        build_set_expanded = False
        if tier in ("G2", "G3", "G4"):
            dna_mode = "collapsed"
            iterate_mode = "collapsed"
        if tier in ("G3", "G4"):
            dna_mode = "visible"
            iterate_mode = "visible"
        if tier in ("G2", "G3", "G4"):
            build_set_expanded = tier in ("G2", "G3", "G4")

        self._apply_tier_section(self.iterate_section, iterate_mode)
        self._apply_tier_section(self.grammar_dna_section, dna_mode)
        if build_set_expanded and not self._grammar_set_section.is_expanded:
            self._grammar_set_section._expanded = True
            self._grammar_set_section._head_btn.configure(text=self._grammar_set_section._header_text())
            self._grammar_set_section._sync_body()
        elif tier in ("G0", "G1") and self._grammar_set_section.is_expanded:
            self._grammar_set_section._expanded = False
            self._grammar_set_section._head_btn.configure(text=self._grammar_set_section._header_text())
            self._grammar_set_section._sync_body()

        self._refresh_assembly_empty_label()

    def _refresh_assembly_empty_label(self) -> None:
        """DES-APS-ASSEMBLY-EMPTY-G2-001 — tier-aware empty label on footprint pane."""
        if not hasattr(self, "_empty_state"):
            return
        from rust_engine_mcp.aps_uiux_onboard import assembly_empty_state_text

        self._empty_state.configure(text=assembly_empty_state_text(self._grammar_set_tier))

    def _apply_tier_section(
        section: CollapsibleSection,
        mode: str,
        *,
        before: tk.Misc | None = None,
    ) -> None:
        if mode == "hidden":
            section.pack_forget()
            return
        if not section.winfo_ismapped():
            if before is not None:
                section.pack(fill=tk.X, pady=4, before=before)
            else:
                section.pack(fill=tk.X, pady=4)
        want_expanded = mode == "visible"
        if section.is_expanded != want_expanded:
            section._expanded = want_expanded
            section._head_btn.configure(text=section._header_text())
            section._sync_body()

    def _widget_packed(widget: tk.Misc) -> bool:
        try:
            return bool(widget.winfo_manager())
        except tk.TclError:
            return False

    def grammar_tier_gate_snapshot(self) -> dict[str, Any]:
        """Scanner payload for tier gate witnesses."""
        values = self.archetype_combo.cget("values")
        combo_count = len(values) if isinstance(values, (list, tuple)) else 0
        kit_text = self._grammar_kit_var.get().strip()
        return {
            "tier": self._grammar_set_tier,
            "dna_panel_visible": self._widget_packed(self.grammar_dna_section),
            "iterate_panel_visible": self._widget_packed(self.iterate_section),
            "build_set_expanded_default": self._grammar_set_section.is_expanded,
            "kit_hint_visible": self._widget_packed(self._kit_hint_label) and bool(kit_text),
            "archetype_combo_count": combo_count,
        }

    def _grammar_section_title(self) -> str:
        base = "Grammar inspector"
        if not self._snapshot:
            return base
        arch = human_label(str(self._snapshot.get("archetype_id") or ""))
        if arch and arch not in ("—", ""):
            return f"{base} — {arch}"
        return base

    def _on_grammar_toggle(self) -> None:
        use = self.use_grammar_var.get()
        state = "readonly" if use else "disabled"
        self.archetype_combo.configure(state=state)
        self.district_combo.configure(state=state)
        fp_state = "disabled" if use else "normal"
        self.footprint_entry.configure(state=fp_state)
        self.floors_spin.configure(state=fp_state)
        if use:
            self.style_combo.configure(state="disabled")
        else:
            self.style_combo.configure(state="normal")
        self._refresh_facility_needs()

    def _refresh_facility_needs(self) -> None:
        archetype_id = None
        site_template_id = None
        if self.use_grammar_var.get():
            label = self.archetype_var.get()
            archetype_id = self._archetype_label_to_id.get(label, label)
            try:
                grammar = building_grammar.load_building_grammar_by_archetype(archetype_id)
                binding = grammar.get("facility_binding") or {}
                site_template_id = binding.get("site_template_id")
            except (KeyError, FileNotFoundError, NotImplementedError):
                site_template_id = None
        self.facility_needs.refresh(archetype_id=archetype_id, lane=self.state.art_domain)
        inset: list[tuple[int, int]] = []
        if self._snapshot:
            for p in self._sorted_placements():
                if int(p.get("floor") or 0) == int(self.footprint_canvas.floor_var.get()):
                    inset.append((int(p.get("grid_x") or 0), int(p.get("grid_y") or 0)))
        self.site_layout.refresh(
            archetype_id=archetype_id,
            lane=self.state.art_domain,
            site_template_id=str(site_template_id) if site_template_id else None,
            footprint_cells=inset,
        )

    def _on_archetype_change(self, _event=None) -> None:
        archetype = self._resolve_archetype_id()
        if not archetype:
            return
        districts = building_grammar.list_district_styles(archetype)
        labels, self._district_label_to_id = grammar_combo_maps(districts or [])
        self.district_combo.configure(values=labels or [""])
        if labels:
            self.district_var.set(labels[0])
        self._refresh_facility_needs()
        self.style_var.set(self.state.style_pack_id)
        self.footprint_var.set(self.state.footprint)
        self.floors_var.set(self.state.floors)
        self.seed_var.set(self.state.seed)

    def _resolve_archetype_id(self) -> str:
        raw = self.archetype_var.get().strip()
        return self._archetype_label_to_id.get(raw, raw)

    def _resolve_district_id(self) -> str:
        raw = self.district_var.get().strip()
        return self._district_label_to_id.get(raw, raw)

    def _on_grammar_inspector_rule_select(self, layer: str, rule_id: str) -> None:
        count = self.footprint_canvas.highlight_for_rule(rule_id)
        self._on_log(f"grammar-inspector {layer}/{rule_id} → {count} cells highlighted")

    def _on_iterate_applied(
        self,
        before: dict[str, Any],
        after: dict[str, Any],
        result: dict[str, Any],
    ) -> None:
        from rust_engine_mcp.grammar_iterate import compute_cell_diff_map

        self._load_snapshot_into_ui(after, path_hint="(iterated)")
        diff_map = compute_cell_diff_map(before, after)
        removed = [k for k, v in diff_map.items() if v == "removed"]
        self.footprint_canvas.set_cell_diff(
            {k: v for k, v in diff_map.items() if v != "removed"},
            removed_ghosts=removed,
        )
        self._on_log(
            f"iterate ok mode={result.get('mode')} child={result.get('lineage', {}).get('child_id')}"
        )

    def _on_grammar_dna_change(self) -> None:
        if not self._snapshot:
            return
        self._commit_snapshot(self._apply_grammar_dna_from_ui(self._snapshot))

    def _apply_grammar_dna_from_ui(self, snap: dict) -> dict:
        state = self.grammar_dna_panel.get_state()
        return arch_build_grammar.apply_to_snapshot(
            snap,
            preset_id=str(state.get("preset_id") or arch_build_grammar.default_preset_id()),
            pressure_field=state.get("pressure_field"),
            include=bool(state.get("include")),
        )
