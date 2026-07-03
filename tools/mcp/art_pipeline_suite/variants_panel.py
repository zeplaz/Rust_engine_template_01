"""Variants workspace — variant_set_v1 layers, tags, agent patch."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

from rust_engine_mcp import variant_set
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import validate_variant_set
from rust_engine_mcp.variant_matrix_expand import variant_matrix_expand, variant_set_rows
from rust_engine_mcp.reaction_territory import (
    CATALOG_EVENT_IDS,
    P0_EVENT_IDS,
    TAG_FAMILIES,
    load_reaction_catalog,
    refresh_reaction_territory_witness,
)
from rust_engine_mcp.variants_sessions import (
    DEFAULT_MATRIX_REL,
    build_variant_set_from_assembly,
    refresh_variants_sessions_witness,
)

from . import aps_theme
from .aps_inline_feedback import set_inline_status
from .aps_onboarding_panel import empty_state_label
from .aps_collapsible import CollapsibleSection
from .aps_tk import themed_listbox, themed_text
from .aps_workflow_layout import workflow_file_row, workflow_intro, workflow_lane_banner, workflow_status_label
from .aps_theme import FONT_SMALL, FONT_UI
from .aps_tooltips import bind_aps_tooltip
from .generation_trace_strip import GenerationTraceStrip
from .job_controller import JobRecord, JobResult, JobState
from .state import ArtDomain, SuiteState
from .variants_layer_context import (
    build_layers_from_controls,
    compose_context_line,
    draft_is_dirty,
    merge_draft_into_entry,
    tags_from_vars,
)
from .variants_preview_panel import VariantsPreviewPanel


class VariantsPanel(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        start_job=None,
        on_go_assembly=None,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        self._start_job = start_job
        self._on_go_assembly = on_go_assembly
        self._data: dict | None = None
        self._display_indices: list[int] = []
        self._tag_vars: dict[str, tk.BooleanVar] = {}
        self._layer_focus: str | None = None
        self._reaction_event_labels = self._load_reaction_event_labels()
        self._build()

    def set_domain(self, lane: str) -> None:
        if lane == ArtDomain.LANDSCAPE.value:
            self._lane_banner.configure(text="Landscape lane — state axis scaffold (E3); building axes remain default.")
        else:
            self._lane_banner.configure(text="Buildings lane — damage / power / fill / lighting axes.")

    def _load_reaction_event_labels(self) -> dict[str, str]:
        try:
            catalog = load_reaction_catalog()
            events = catalog.get("events") or {}
            return {
                eid: str((events.get(eid) or {}).get("label") or eid)
                for eid in CATALOG_EVENT_IDS
                if eid in events
            }
        except (OSError, json.JSONDecodeError, KeyError):
            return {eid: eid for eid in CATALOG_EVENT_IDS}

    def _reaction_filter_value(self) -> str | None:
        raw = self._reaction_filter_var.get()
        if raw == "All sessions":
            return None
        if raw == "Base sessions":
            return "__base__"
        for eid, label in self._reaction_event_labels.items():
            if raw == label:
                return eid
        return None

    def _get_reaction_event_id(self) -> str | None:
        entry = self._selected_entry()
        if entry and entry.get("reaction_event_id"):
            return str(entry["reaction_event_id"])
        filt = self._reaction_filter_value()
        if filt and filt != "__base__":
            return filt
        return None

    def _build(self) -> None:
        workflow_intro(
            self,
            "Define visual states (lighting, damage, fill) for the same building — bake tiles from Atlas.",
        )
        self._lane_banner = workflow_lane_banner(self)

        primary = ttk.Frame(self)
        primary.pack(fill=tk.X, pady=4)
        ttk.Button(primary, text="New from assembly", command=self.on_new_from_assembly).pack(side=tk.LEFT, padx=2)
        ttk.Button(primary, text="Expand sessions…", command=self.on_expand_sessions).pack(side=tk.LEFT, padx=2)
        ttk.Button(primary, text="Load example", command=self.on_load_example).pack(side=tk.LEFT, padx=2)

        self._status_lbl, self.status_var = workflow_status_label(self, wraplength=720)

        paned = ttk.Panedwindow(self, orient=tk.HORIZONTAL)
        paned.pack(fill=tk.BOTH, expand=True, pady=4)

        left = ttk.Frame(paned, padding=4)
        paned.add(left, weight=1)
        filter_row = ttk.Frame(left)
        filter_row.pack(fill=tk.X, pady=(0, 4))
        ttk.Label(filter_row, text="Reaction event").pack(side=tk.LEFT)
        reaction_values = ["All sessions", "Base sessions"] + list(self._reaction_event_labels.values())
        self._reaction_filter_var = tk.StringVar(value="All sessions")
        reaction_combo = ttk.Combobox(
            filter_row,
            textvariable=self._reaction_filter_var,
            values=reaction_values,
            width=34,
            state="readonly",
        )
        reaction_combo.pack(side=tk.LEFT, padx=4, fill=tk.X, expand=True)
        reaction_combo.bind("<<ComboboxSelected>>", self._on_reaction_filter_change)
        bind_aps_tooltip(reaction_combo, "var_reaction_filter")
        self._suggested_tags_row = ttk.Frame(left)
        self._suggested_tags_row.pack(fill=tk.X, pady=(0, 4))
        self._suggested_tag_buttons: list[ttk.Button] = []
        ttk.Label(left, text="Variants").pack(anchor=tk.W)
        self._empty_state = empty_state_label(left, "variants")
        self._empty_state.pack(anchor=tk.W, pady=2)
        self.variant_list = themed_listbox(left, exportselection=False)
        self.variant_list.pack(fill=tk.BOTH, expand=True)
        self.variant_list.bind("<<ListboxSelect>>", self.on_variant_select)

        right = ttk.Frame(paned, padding=4)
        paned.add(right, weight=2)

        self._gen_trace = GenerationTraceStrip(
            right,
            self.state,
            get_snapshot=lambda: self.state.assembly_snapshot_data,
            on_go_assembly=self._on_go_assembly,
        )
        self._gen_trace.pack(fill=tk.X, pady=(0, 4))

        self._preview = VariantsPreviewPanel(
            right,
            on_log=self._on_log,
            get_snapshot=lambda: self.state.assembly_snapshot_data,
            get_variant_entry=self._preview_entry,
            get_reaction_event_id=self._get_reaction_event_id,
            get_context_line=self._layer_context_line,
            get_draft_dirty=self._layer_draft_dirty,
        )
        self._preview.pack(fill=tk.X, pady=(0, 4))

        layer_row = ttk.LabelFrame(right, text="Layers", padding=6)
        layer_row.pack(fill=tk.X, pady=4)

        ttk.Label(layer_row, text="Lighting").grid(row=0, column=0, sticky=tk.W)
        self.lighting_var = tk.StringVar(value="day")
        lighting_combo = ttk.Combobox(
            layer_row,
            textvariable=self.lighting_var,
            width=12,
            values=["day", "night_off", "night_on"],
            state="readonly",
        )
        lighting_combo.grid(row=0, column=1, padx=4)
        bind_aps_tooltip(lighting_combo, "var_lighting")
        ttk.Label(layer_row, text="Power").grid(row=0, column=2, sticky=tk.W)
        self.power_var = tk.StringVar(value="off")
        power_combo = ttk.Combobox(
            layer_row, textvariable=self.power_var, width=10, values=["off", "partial", "on"], state="readonly"
        )
        power_combo.grid(row=0, column=3, padx=4)
        bind_aps_tooltip(power_combo, "var_power")
        self.night_lights_var = tk.BooleanVar(value=False)
        night_cb = ttk.Checkbutton(layer_row, text="night_lights", variable=self.night_lights_var)
        night_cb.grid(row=1, column=1, sticky=tk.W)
        bind_aps_tooltip(night_cb, "var_lighting")

        ttk.Label(layer_row, text="Damage state").grid(row=2, column=0, sticky=tk.W)
        self.damage_state_var = tk.StringVar(value="clean")
        damage_combo = ttk.Combobox(
            layer_row,
            textvariable=self.damage_state_var,
            width=12,
            values=["clean", "dirty", "damaged", "ruined"],
            state="readonly",
        )
        damage_combo.grid(row=2, column=1, padx=4)
        bind_aps_tooltip(damage_combo, "var_damage")
        ttk.Label(layer_row, text="damage").grid(row=2, column=2, sticky=tk.W)
        self.damage_val_var = tk.DoubleVar(value=0.0)
        damage_scale = ttk.Scale(
            layer_row, from_=0, to=1, variable=self.damage_val_var, orient=tk.HORIZONTAL
        )
        damage_scale.grid(row=2, column=3, sticky=tk.EW, padx=4)
        bind_aps_tooltip(damage_scale, "var_damage")

        ttk.Label(layer_row, text="Fill").grid(row=3, column=0, sticky=tk.W)
        self.fill_var = tk.StringVar(value="empty")
        fill_combo = ttk.Combobox(
            layer_row,
            textvariable=self.fill_var,
            width=12,
            values=["empty", "quarter", "half", "full"],
            state="readonly",
        )
        fill_combo.grid(row=3, column=1, padx=4)
        bind_aps_tooltip(fill_combo, "var_fill")

        ttk.Label(layer_row, text="Material").grid(row=4, column=0, sticky=tk.W)
        self.material_var = tk.StringVar(value="")
        material_entry = ttk.Entry(layer_row, textvariable=self.material_var, width=28)
        material_entry.grid(row=4, column=1, columnspan=3, sticky=tk.EW, padx=4)

        ttk.Label(layer_row, text="Tags (mandate families)").grid(row=5, column=0, sticky=tk.NW, pady=(4, 0))
        tag_frame = ttk.Frame(layer_row)
        tag_frame.grid(row=5, column=1, columnspan=3, sticky=tk.EW, pady=(4, 0))
        self._tag_focus: str | None = None
        for col, (family, tags) in enumerate(TAG_FAMILIES.items()):
            box = ttk.LabelFrame(tag_frame, text=family.capitalize(), padding=4)
            box.grid(row=0, column=col, padx=2, sticky=tk.NW)
            for tag in tags:
                var = tk.BooleanVar(value=False)
                self._tag_vars[tag] = var
                from rust_engine_mcp.aps_tag_vocabulary import mandate_tag_label

                cb = ttk.Checkbutton(
                    box,
                    text=mandate_tag_label(tag),
                    variable=var,
                    command=lambda t=tag: self._on_tag_draft(t),
                )
                cb.pack(anchor=tk.W)
                bind_aps_tooltip(cb, f"var_mandate_tag:{tag}")

        self._tag_context_var = tk.StringVar(value="")
        ttk.Label(
            layer_row,
            textvariable=self._tag_context_var,
            wraplength=520,
            font=FONT_SMALL,
            foreground=aps_theme.COLOR_TEXT_SUBTLE,
        ).grid(row=7, column=0, columnspan=4, sticky=tk.W, pady=(0, 4))

        apply_btn = ttk.Button(layer_row, text="Apply layers to selected", command=self.on_apply_layers)
        apply_btn.grid(row=6, column=0, columnspan=2, pady=6, sticky=tk.W)
        ttk.Button(layer_row, text="Apply tag preset…", command=self._on_apply_tag_preset).grid(
            row=6, column=2, columnspan=2, pady=6, sticky=tk.W
        )
        bind_aps_tooltip(apply_btn, "var_apply_layers")
        bind_aps_tooltip(self._preview, "var_draft_preview")

        self._wire_layer_live_preview()

        self.bake_status = tk.StringVar(value="")
        self._bake_status_lbl = ttk.Label(right, textvariable=self.bake_status, font=FONT_SMALL)
        self._bake_status_lbl.pack(anchor=tk.W)

        file_row = workflow_file_row(self)
        ttk.Button(file_row, text="Load…", command=self.on_load).pack(side=tk.LEFT, padx=2)
        ttk.Button(file_row, text="Save (JSON)", command=lambda: self.on_save(ext=".json")).pack(side=tk.LEFT, padx=2)
        ttk.Button(file_row, text="Save (engine format)", command=lambda: self.on_save(ext=".ron")).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(file_row, text="Check schema", command=self.on_validate).pack(side=tk.LEFT, padx=2)
        self.path_var = tk.StringVar(value="(none)")
        ttk.Label(file_row, textvariable=self.path_var, foreground=aps_theme.COLOR_TEXT_SUBTLE).pack(
            side=tk.LEFT, padx=8
        )

        agent_section = CollapsibleSection(self, "Agent patch (advanced)", expanded=False, padding=4)
        agent_section.pack(fill=tk.X, pady=4)
        agent_row = ttk.Frame(agent_section.body, padding=2)
        agent_row.pack(fill=tk.BOTH, expand=True)
        ttk.Label(agent_row, text="Intent").pack(anchor=tk.W)
        self.intent_var = tk.StringVar(value="add_warm_window_lights")
        ttk.Entry(agent_row, textvariable=self.intent_var, width=48).pack(fill=tk.X, pady=2)
        btn_row = ttk.Frame(agent_row)
        btn_row.pack(fill=tk.X, pady=4)
        ttk.Button(btn_row, text="Request agent", command=self.on_request_agent).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Apply patch", command=self.on_apply_patch).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Bake selected", command=self.on_bake_selected).pack(side=tk.LEFT, padx=2)
        self.patch_text = themed_text(agent_row, height=8, wrap=tk.WORD, font=("Consolas", 9))
        self.patch_text.pack(fill=tk.BOTH, expand=True, pady=4)

    def _set_status(self, text: str, *, ok: bool | None = None) -> None:
        set_inline_status(self._status_lbl, self.status_var, text, ok=ok)

    def _set_empty_state_visible(self, visible: bool) -> None:
        if not hasattr(self, "_empty_state"):
            return
        if visible and not self._empty_state.winfo_ismapped():
            self._empty_state.pack(anchor=tk.W, pady=2, before=self.variant_list)
        elif not visible and self._empty_state.winfo_ismapped():
            self._empty_state.pack_forget()

    def _draft_layers_from_controls(self) -> dict:
        return build_layers_from_controls(
            lighting=self.lighting_var.get(),
            power=self.power_var.get(),
            night_lights=self.night_lights_var.get(),
            damage_state=self.damage_state_var.get(),
            damage=float(self.damage_val_var.get()),
            fill=self.fill_var.get(),
            wall_material=self.material_var.get(),
        )

    def _draft_tags_from_controls(self) -> list[str]:
        return tags_from_vars(self._tag_vars)

    def _preview_entry(self) -> dict | None:
        entry = self._selected_entry()
        if not entry:
            return None
        return merge_draft_into_entry(
            entry,
            self._draft_layers_from_controls(),
            self._draft_tags_from_controls(),
        )

    def _layer_context_line(self) -> str:
        return compose_context_line(
            lighting=self.lighting_var.get(),
            power=self.power_var.get(),
            night_lights=self.night_lights_var.get(),
            damage_state=self.damage_state_var.get(),
            damage=float(self.damage_val_var.get()),
            fill=self.fill_var.get(),
            wall_material=self.material_var.get(),
            focus=self._layer_focus,
        )

    def _layer_draft_dirty(self) -> bool:
        return draft_is_dirty(
            self._selected_entry(),
            self._draft_layers_from_controls(),
            self._draft_tags_from_controls(),
        )

    def _on_layer_draft(self, focus: str) -> None:
        self._layer_focus = focus
        self._refresh_tag_context()
        self.preview_selected_variant()

    def _on_tag_draft(self, focus: str) -> None:
        self._tag_focus = focus
        self._refresh_tag_context()
        self._on_layer_draft("tags")

    def _refresh_tag_context(self) -> None:
        if not hasattr(self, "_tag_context_var"):
            return
        from rust_engine_mcp.aps_tag_vocabulary import compose_mandate_tag_context

        active = self._draft_tags_from_controls()
        self._tag_context_var.set(compose_mandate_tag_context(active, focus=self._tag_focus))

    def _wire_layer_live_preview(self) -> None:
        focus_map = {
            self.lighting_var: "lighting",
            self.power_var: "power",
            self.damage_state_var: "damage_state",
            self.fill_var: "fill",
            self.material_var: "material",
        }
        for var, focus in focus_map.items():
            var.trace_add("write", lambda *_a, f=focus: self._on_layer_draft(f))
        self.damage_val_var.trace_add("write", lambda *_a: self._on_layer_draft("damage"))
        self.night_lights_var.trace_add("write", lambda *_a: self._on_layer_draft("night_lights"))

    def _selected_index(self) -> int | None:
        sel = self.variant_list.curselection()
        if not sel or not self._data or not self._display_indices:
            return None
        display_idx = int(sel[0])
        if display_idx >= len(self._display_indices):
            return None
        return self._display_indices[display_idx]

    def _selected_entry(self) -> dict | None:
        idx = self._selected_index()
        if idx is None or not self._data:
            return None
        variants = self._data.get("variants") or []
        if idx >= len(variants):
            return None
        return variants[idx]

    def preview_selected_variant(self, *, force: bool = False) -> None:
        if hasattr(self, "_preview"):
            self._preview.queue_preview(force=force)

    def _refresh_list(self) -> None:
        self.variant_list.delete(0, tk.END)
        self._display_indices = []
        has_variants = bool(self._data and (self._data.get("variants") or []))
        self._set_empty_state_visible(not has_variants)
        if not self._data:
            return
        filt = self._reaction_filter_value()
        for idx, entry in enumerate(self._data.get("variants") or []):
            event_id = entry.get("reaction_event_id")
            if filt == "__base__" and event_id:
                continue
            if filt and filt != "__base__" and str(event_id or "") != filt:
                continue
            key = entry.get("variant_key", "?")
            bake = (entry.get("bake") or {}).get("status")
            event = entry.get("reaction_event_id")
            suffix = f" [{bake}]" if bake else ""
            event_suffix = f" · {event}" if event else ""
            self._display_indices.append(idx)
            self.variant_list.insert(tk.END, f"{key}{event_suffix}{suffix}")

    def _on_reaction_filter_change(self, _event=None) -> None:
        from rust_engine_mcp.aps_tag_vocabulary import reaction_event_context
        from rust_engine_mcp.reaction_territory import load_reaction_catalog

        filt = self._reaction_filter_var.get()
        if filt not in ("All sessions", "Base sessions") and hasattr(self, "_tag_context_var"):
            try:
                catalog = load_reaction_catalog()
                events = catalog.get("events") or {}
                for eid, label in self._reaction_event_labels.items():
                    if label == filt:
                        event = events.get(eid) or {}
                        self._tag_context_var.set(reaction_event_context(event))
                        break
            except (OSError, json.JSONDecodeError, KeyError):
                pass
        self._refresh_suggested_tag_chips()
        prev_key = self.state.selected_variant_key
        self._refresh_list()
        if prev_key and self._data:
            for display_idx, src_idx in enumerate(self._display_indices):
                entry = self._data["variants"][src_idx]
                if str(entry.get("variant_key")) == prev_key:
                    self.variant_list.selection_set(display_idx)
                    self.on_variant_select()
                    return
        if self._display_indices:
            self.variant_list.selection_set(0)
            self.on_variant_select()
        else:
            self._set_status("No variant rows match this reaction filter.", ok=None)
        self.preview_selected_variant(force=True)

    def _current_archetype_id(self) -> str:
        snap = self.state.assembly_snapshot_data or {}
        return str(snap.get("archetype_id") or "IndustrialWarehouse")

    def _refresh_suggested_tag_chips(self) -> None:
        from rust_engine_mcp.aps_tag_tier2 import suggested_mandate_tags_for_event

        for child in self._suggested_tags_row.winfo_children():
            child.destroy()
        self._suggested_tag_buttons.clear()
        filt = self._reaction_filter_value()
        if not filt or filt == "__base__":
            return
        tags = suggested_mandate_tags_for_event(filt)
        if not tags:
            return
        ttk.Label(self._suggested_tags_row, text="Suggested tags:", font=FONT_SMALL).pack(side=tk.LEFT)
        from rust_engine_mcp.aps_tag_vocabulary import mandate_tag_label

        for tag in tags[:6]:
            btn = ttk.Button(
                self._suggested_tags_row,
                text=mandate_tag_label(tag),
                command=lambda t=tag: self._on_suggested_tag_chip(t),
            )
            btn.pack(side=tk.LEFT, padx=2)
            self._suggested_tag_buttons.append(btn)

    def _on_suggested_tag_chip(self, tag: str) -> None:
        var = self._tag_vars.get(tag)
        if var is not None:
            var.set(True)
            self._on_tag_draft(tag)
            self._on_log(f"suggested tag chip: {tag}")

    def _on_apply_tag_preset(self) -> None:
        from rust_engine_mcp.aps_tag_tier2 import preset_confirm_lines, preset_for_archetype

        archetype_id = self._current_archetype_id()
        row = preset_for_archetype(archetype_id)
        if not row:
            self._set_status(f"No tier-2 tag preset for {archetype_id}.", ok=None)
            return
        lines = "\n".join(preset_confirm_lines(archetype_id))
        if not messagebox.askyesno("Apply tag preset?", f"{lines}\n\nApply these mandate tags to the draft?"):
            return
        for tag in row.get("mandate_tags") or []:
            var = self._tag_vars.get(str(tag))
            if var is not None:
                var.set(True)
        self._tag_focus = None
        self._refresh_tag_context()
        self._on_log(f"tag preset {row.get('preset_name')} applied (draft) — Apply layers to save")

    def _load_data(self, data: dict, path: str | None) -> None:
        validate_variant_set(data)
        self._data = data
        self.state.variant_set_data = data
        self.state.variant_set_path = path
        self.path_var.set(path or "(memory)")
        self._refresh_list()
        if hasattr(self, "_gen_trace"):
            self._gen_trace.refresh()
        if self._data.get("variants"):
            self.variant_list.selection_set(0)
            self.on_variant_select()

    def on_load(self) -> None:
        path = filedialog.askopenfilename(
            title="Open variant set",
            filetypes=[("Variant set", "*.json *.ron"), ("All", "*.*")],
        )
        if not path:
            return
        try:
            data = variant_set.load_variant_set(path)
        except Exception as exc:  # noqa: BLE001
            self._set_status(f"Load failed: {exc}", ok=False)
            return
        self._load_data(data, path)
        self._set_status(f"Loaded {path}", ok=True)

    def on_load_example(self) -> None:
        path = variant_set.example_variant_set_path()
        if not path.is_file():
            self._set_status(f"Missing example: {path}", ok=False)
            return
        data = variant_set.load_variant_set(path)
        self._load_data(data, str(path))
        self._set_status(f"Loaded example {path}", ok=True)

    def on_new_from_assembly(self) -> None:
        aid = self.state.assembly_id
        if not aid:
            self._set_status("Generate an assembly snapshot first (Assembly tab).", ok=None)
            return
        snapshot = self.state.assembly_snapshot_data
        data = build_variant_set_from_assembly(
            assembly_id=aid,
            style_pack_id=self.state.style_pack_id,
            seed=self.state.seed,
            assembly_snapshot=snapshot,
            include_full_catalog=True,
        )
        out = variant_set.save_variant_set(data)
        self._load_data(data, str(out))
        refresh_variants_sessions_witness()
        refresh_reaction_territory_witness()
        count = len(data.get("variants") or [])
        base_count = sum(1 for v in data.get("variants") or [] if not v.get("reaction_event_id"))
        reaction_count = count - base_count
        self._on_log(f"New variant set {out} · {count} session rows ({base_count} base + {reaction_count} reaction)")
        self._set_status(
            f"Created {count} sessions ({base_count} base + {reaction_count} reaction territory).",
            ok=True,
        )
        self.preview_selected_variant(force=True)

    def on_expand_sessions(self) -> None:
        if not self._data:
            self._set_status("Create or load a variant set first.", ok=None)
            return
        default_dir = repo_root() / "debug_runs" / "art_pipeline"
        matrix_path = filedialog.askopenfilename(
            title="Expand variant sessions from matrix YAML",
            initialdir=str(default_dir) if default_dir.is_dir() else str(repo_root()),
            filetypes=[("Variant matrix YAML", "*.yaml *.yml"), ("All", "*.*")],
        )
        if not matrix_path:
            return
        matrix_path = Path(matrix_path)
        try:
            result = variant_matrix_expand(
                matrix_path,
                minimum_only=True,
                include_fire_row=True,
                write_batch=False,
            )
            rows = variant_set_rows(result.get("variant_keys") or [])
        except Exception as exc:  # noqa: BLE001
            self._set_status(f"Expand failed: {exc}", ok=False)
            return
        existing = {str(v.get("variant_key")) for v in self._data.get("variants") or []}
        added = 0
        for row in rows:
            key = str(row.get("variant_key"))
            if key in existing:
                continue
            self._data["variants"].append(row)
            existing.add(key)
            added += 1
        self._refresh_list()
        self._on_log(f"expand sessions +{added} from {matrix_path.name}")
        self._set_status(f"Expanded +{added} variant rows from matrix.", ok=True if added else None)
        refresh_variants_sessions_witness()

    def on_save(self, *, ext: str) -> None:
        if not self._data:
            self._set_status("Nothing to save.", ok=None)
            return
        path = self.state.variant_set_path
        if not path or not path.endswith(ext):
            path = str(variant_set.default_variant_set_path(self._data["variant_set_id"], ext=ext))
        out = variant_set.save_variant_set(self._data, path)
        self.state.variant_set_path = str(out)
        self.path_var.set(str(out))
        self._on_log(f"saved {out}")
        self._set_status(f"Saved: {out}", ok=True)

    def on_validate(self) -> None:
        if not self._data:
            self._set_status("Load a variant set first.", ok=None)
            return
        try:
            validate_variant_set(self._data)
        except Exception as exc:  # noqa: BLE001
            self._set_status(f"Schema check failed — {exc}", ok=False)
            return
        self._set_status("Schema check passed — variant set is valid.", ok=True)

    def on_variant_select(self, _event=None) -> None:
        idx = self._selected_index()
        if idx is None or not self._data:
            return
        self._layer_focus = None
        entry = self._data["variants"][idx]
        self.state.selected_variant_key = str(entry.get("variant_key"))
        layers = entry.get("layers") or {}
        lighting = layers.get("lighting") or {}
        damage = layers.get("damage") or {}
        fill = layers.get("fill") or {}
        material = layers.get("material") or {}
        self.lighting_var.set(str(lighting.get("lighting") or "day"))
        self.power_var.set(str(lighting.get("power") or "off"))
        self.night_lights_var.set(bool(lighting.get("night_lights")))
        self.damage_state_var.set(str(damage.get("state") or "clean"))
        self.damage_val_var.set(float(damage.get("damage") or 0.0))
        self.fill_var.set(str(fill.get("fill") or "empty"))
        mat = material.get("wall_material") or ""
        self.material_var.set(str(mat))
        tags = set(entry.get("tags") or [])
        for tag, var in self._tag_vars.items():
            var.set(tag in tags)
        self._tag_focus = None
        self._refresh_tag_context()
        bake = entry.get("bake") or {}
        bake_line = f"bake: {bake.get('status', 'pending')} · {bake.get('png') or '—'}"
        status = str(bake.get("status") or "").lower()
        if status in ("done", "ok", "pass", "passed"):
            set_inline_status(self._bake_status_lbl, self.bake_status, bake_line, ok=True)
        elif status:
            set_inline_status(self._bake_status_lbl, self.bake_status, bake_line, ok=False)
        else:
            set_inline_status(self._bake_status_lbl, self.bake_status, bake_line, ok=None)
        if hasattr(self, "_preview"):
            self._preview.sync_visual_state_from_entry(entry)
            self._preview.queue_preview()

    def on_apply_layers(self) -> None:
        idx = self._selected_index()
        if idx is None or not self._data:
            self._set_status("Select a variant row.", ok=None)
            return
        layers = self._draft_layers_from_controls()
        tags = self._draft_tags_from_controls()
        self._data["variants"][idx]["layers"] = layers
        if tags:
            self._data["variants"][idx]["tags"] = tags
        elif "tags" in self._data["variants"][idx]:
            self._data["variants"][idx]["tags"] = []
        self._layer_focus = None
        self._refresh_list()
        self.variant_list.selection_set(idx)
        key = self._data["variants"][idx]["variant_key"]
        self._on_log(f"layers updated {key}")
        self._set_status(f"Layers applied to {key}", ok=True)
        self.preview_selected_variant(force=True)

    def on_request_agent(self) -> None:
        if not self._data:
            self._set_status("Load a variant set first.", ok=None)
            return
        idx = self._selected_index() or 0
        entry = self._data["variants"][idx]
        body = {
            "assembly_id": self._data.get("assembly_id"),
            "variant_key": entry.get("variant_key"),
            "intent": self.intent_var.get().strip(),
            "current_layers": entry.get("layers") or {},
            "constraints": ["lod0_tier", f"deterministic_seed_{self._data.get('seed', 42)}"],
            "reference_tags": entry.get("tags") or [],
        }
        result = variant_set.variant_agent_request(body, write=True)
        self.patch_text.delete("1.0", tk.END)
        self.patch_text.insert("1.0", json.dumps(result, indent=2))
        written = result.get("written_path") or "debug_runs/art_pipeline/variant_agent_request.json"
        self._on_log(f"agent request → {written}")
        self._set_status(
            f"Wrote {written} · paste into Cursor; apply via variant_set_patch after review.",
            ok=True,
        )

    def on_apply_patch(self) -> None:
        if not self.state.variant_set_path:
            self._set_status("Save variant set to disk first.", ok=None)
            return
        try:
            raw = self.patch_text.get("1.0", tk.END).strip()
            payload = json.loads(raw) if raw else {}
        except json.JSONDecodeError as exc:
            self._set_status(f"Apply patch failed: {exc}", ok=False)
            return
        patch = payload.get("patch") if isinstance(payload, dict) else payload
        if not isinstance(patch, list):
            self._set_status("Patch JSON must be a list or {patch:[...]}", ok=False)
            return
        try:
            result = variant_set.variant_set_patch(self.state.variant_set_path, patch)
            self._data = result["document"]
            self.state.variant_set_data = self._data
            self._refresh_list()
        except Exception as exc:  # noqa: BLE001
            self._set_status(f"Apply patch failed: {exc}", ok=False)
            return
        self._on_log(f"patch applied ({len(patch)} ops)")
        self._set_status(f"Patch applied and saved ({len(patch)} ops).", ok=True)

    def on_bake_selected(self) -> None:
        if not self.state.variant_set_path or not self.state.selected_variant_key:
            self._set_status("Save variant set and select a variant.", ok=None)
            return
        vs_aid = str((self._data or {}).get("assembly_id") or "")
        cur_aid = str(self.state.assembly_id or "")
        if cur_aid and vs_aid and vs_aid != cur_aid:
            if not messagebox.askyesno(
                "Assembly mismatch",
                f"Variant set targets:\n  {vs_aid}\n\n"
                f"Current Assembly tab snapshot:\n  {cur_aid}\n\n"
                "Bake anyway? (PNG will land under the variant set assembly_id folder.)",
            ):
                self._on_log(f"Bake cancelled — variant set targets {vs_aid}, not the current {cur_aid}.")
                return
        if self._start_job:
            path = self.state.variant_set_path
            key = self.state.selected_variant_key

            def worker(_cancel) -> JobResult:
                if _cancel.is_set():
                    return JobResult(False, "Cancelled")
                try:
                    result = variant_set.variant_bake(path, key)
                except Exception as exc:  # noqa: BLE001
                    return JobResult(False, f"Bake failed: {exc}", detail=str(exc))
                if not result.get("ok"):
                    return JobResult(False, result.get("error") or "Bake failed")
                return JobResult(True, f"variant-bake {key} OK", data=result)

            def on_done(record: JobRecord) -> None:
                if record.result and record.result.ok:
                    self._finish_bake(record.result.data or {})

            self._start_job("Variant bake", worker, on_done=on_done)
            return
        self._bake_selected_sync()

    def _bake_selected_sync(self) -> None:
        vs_aid = str((self._data or {}).get("assembly_id") or "")
        cur_aid = str(self.state.assembly_id or "")
        if cur_aid and vs_aid and vs_aid != cur_aid:
            if not messagebox.askyesno(
                "Assembly mismatch",
                f"Variant set targets:\n  {vs_aid}\n\n"
                f"Current Assembly tab snapshot:\n  {cur_aid}\n\n"
                "Bake anyway? (PNG will land under the variant set assembly_id folder.)",
            ):
                self._on_log(f"Bake cancelled — variant set targets {vs_aid}, not the current {cur_aid}.")
                return
        self._on_log(f"variant-bake {self.state.selected_variant_key} → assembly {vs_aid}")
        try:
            result = variant_set.variant_bake(
                self.state.variant_set_path,
                self.state.selected_variant_key,
            )
        except Exception as exc:  # noqa: BLE001
            self._set_status(f"Bake failed: {exc}", ok=False)
            return
        self._finish_bake(result)

    def _finish_bake(self, result: dict) -> None:
        self._data = variant_set.load_variant_set(self.state.variant_set_path)
        self.state.variant_set_data = self._data
        self._refresh_list()
        self.on_variant_select()
        if result.get("ok"):
            png = result.get("png")
            rel = Path(str(png)).relative_to(repo_root()) if png else None
            self.state.atlas_folder = str((repo_root() / "assets/staging/tiles" / self._data["assembly_id"]).resolve())
            self._set_status(f"Bake OK · {rel}", ok=True)
        else:
            self._set_status(f"Bake failed: {result.get('error') or 'failed'}", ok=False)
