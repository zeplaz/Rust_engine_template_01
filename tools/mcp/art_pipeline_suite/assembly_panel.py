"""Assembly Editor (APS-UI-003b) — footprint grid, grammar, categorized semantic tags."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk
from typing import Any

from rust_engine_mcp import aps_tags, assembly, arch_build_grammar, building_grammar
from rust_engine_mcp.aps_grammar_labels import human_label
from rust_engine_mcp.aps_mat_auth_ui import ENGINE_READ_PATH, save_hint
from rust_engine_mcp.aps_validator_plain import format_p0_display
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate

from .aps_tooltips import bind_aps_tooltip
from .aps_collapsible import CollapsibleSection
from .aps_inline_feedback import set_inline_status
from .aps_paned import add_pane, horizontal_paned, set_initial_pane_widths
from .aps_scroll import attach_wheel_area
from .aps_theme import FONT_UI, FONT_UI_BOLD, FONT_MONO, track_wraplength, wrap_for_widget
from .footprint_canvas import FootprintCanvas
from .metadata_flow_panel import MetadataFlowPanel
from .grammar_inspector import GrammarInspectorPanel
from .grammar_iterate_panel import GrammarIteratePanel
from .grammar_dna_panel import GrammarDnaPanel
from .grammar_build_set_panel import GrammarBuildSetPanel
from .material_browser import MaterialBrowserPanel
from .assembly_preview_panel import AssemblyPreviewPanel
from .slot_preview_panel import SlotPreviewPanel
from .state import SuiteState


class AssemblyPanel(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        on_open_in_materials=None,
        start_job=None,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        self._on_open_in_materials = on_open_in_materials
        self._start_job = start_job
        self._snapshot: dict | None = None
        self._selected_node_id: str | None = None
        self._semantic_tag_vars: dict[str, dict[str, tk.BooleanVar]] = {}
        self._variant_tag_vars: dict[str, tk.BooleanVar] = {}
        self._tag_category_frames: dict[str, ttk.LabelFrame] = {}
        self._material_profiles: list[str] = []
        self._build()

    def _build(self) -> None:
        intro = ttk.Label(
            self,
            text="Assembly — snapshot is authority for materials & tags (not Blender). "
            "Preview slots + full assembly here; keyframe tile bake is on Atlas tab.",
            wraplength=900,
            justify=tk.LEFT,
        )
        intro.pack(anchor=tk.W, pady=(0, 4))
        track_wraplength(self, intro, minimum=480)
        self.metadata_flow = MetadataFlowPanel(self, context="assembly")
        self.metadata_flow.pack(fill=tk.X, pady=(0, 6))

        self.grammar_set_panel = GrammarBuildSetPanel(self, on_log=self._on_log)
        self.grammar_set_panel.pack(fill=tk.X, pady=(0, 6))

        auth = ttk.LabelFrame(self, text="Material authority (APS-MAT-AUTH-UI-001)", padding=6)
        auth.pack(fill=tk.X, pady=(0, 6))
        self.engine_path_var = tk.StringVar(value=ENGINE_READ_PATH)
        self._engine_path_lbl = ttk.Label(
            auth, textvariable=self.engine_path_var, wraplength=900, justify=tk.LEFT, font=("Segoe UI", 9)
        )
        self._engine_path_lbl.pack(anchor=tk.W)
        bind_aps_tooltip(self._engine_path_lbl, "asm_engine_path")
        self.save_hint_var = tk.StringVar(value="")
        ttk.Label(auth, textvariable=self.save_hint_var, wraplength=900, justify=tk.LEFT, foreground="#0a4a7a").pack(
            anchor=tk.W, pady=(4, 0)
        )

        gen = ttk.LabelFrame(self, text="Generate", padding=6)
        gen.pack(fill=tk.X, pady=4)

        gram_row = ttk.Frame(gen)
        gram_row.pack(fill=tk.X, pady=2)
        self.use_grammar_var = tk.BooleanVar(value=False)
        gram_cb = ttk.Checkbutton(
            gram_row, text="Use building grammar", variable=self.use_grammar_var, command=self._on_grammar_toggle
        )
        gram_cb.pack(side=tk.LEFT)
        bind_aps_tooltip(gram_cb, "asm_grammar")
        ttk.Label(gram_row, text="Archetype").pack(side=tk.LEFT, padx=(12, 0))
        archetypes = building_grammar.list_archetype_ids() or ["IndustrialWarehouse"]
        self.archetype_var = tk.StringVar(value=archetypes[0])
        self.archetype_combo = ttk.Combobox(
            gram_row, textvariable=self.archetype_var, width=20, values=archetypes, state="readonly"
        )
        self.archetype_combo.pack(side=tk.LEFT, padx=4)
        self.archetype_combo.bind("<<ComboboxSelected>>", self._on_archetype_change)
        bind_aps_tooltip(self.archetype_combo, "asm_archetype")
        ttk.Label(gram_row, text="District").pack(side=tk.LEFT, padx=(8, 0))
        districts = building_grammar.list_district_styles(archetypes[0]) or ["industrial_west"]
        self.district_var = tk.StringVar(value=districts[0] if districts else "")
        self.district_combo = ttk.Combobox(
            gram_row, textvariable=self.district_var, width=16, values=districts, state="readonly"
        )
        self.district_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.district_combo, "asm_district")

        row = ttk.Frame(gen)
        row.pack(fill=tk.X, pady=2)
        ttk.Label(row, text="StylePack").pack(side=tk.LEFT)
        packs = assembly.list_style_packs()
        self.style_var = tk.StringVar(value=self.state.style_pack_id)
        self.style_combo = ttk.Combobox(
            row, textvariable=self.style_var, width=22, values=packs or ["style_victorian"]
        )
        self.style_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.style_combo, "asm_style_pack")
        ttk.Label(row, text="Tier").pack(side=tk.LEFT, padx=(8, 0))
        self.tier_var = tk.StringVar(value="production")
        self.tier_combo = ttk.Combobox(
            row, textvariable=self.tier_var, width=12, values=["production", "lod0"], state="readonly"
        )
        self.tier_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.tier_combo, "asm_tier")

        row2 = ttk.Frame(gen)
        row2.pack(fill=tk.X, pady=2)
        ttk.Label(row2, text="Footprint W×D").pack(side=tk.LEFT)
        self.footprint_var = tk.StringVar(value=self.state.footprint)
        self.footprint_entry = ttk.Entry(row2, textvariable=self.footprint_var, width=8)
        self.footprint_entry.pack(side=tk.LEFT, padx=4)
        ttk.Label(row2, text="Floors").pack(side=tk.LEFT, padx=(8, 0))
        self.floors_var = tk.IntVar(value=self.state.floors)
        self.floors_spin = ttk.Spinbox(row2, from_=1, to=8, textvariable=self.floors_var, width=4)
        self.floors_spin.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.footprint_entry, "asm_footprint_dims")
        bind_aps_tooltip(self.floors_spin, "asm_footprint_dims")
        ttk.Label(row2, text="Seed").pack(side=tk.LEFT, padx=(8, 0))
        self.seed_var = tk.IntVar(value=self.state.seed)
        ttk.Spinbox(row2, from_=0, to=999999, textvariable=self.seed_var, width=8).pack(side=tk.LEFT, padx=4)
        gen_btn = ttk.Button(row2, text="Generate snapshot", command=self.on_generate)
        gen_btn.pack(side=tk.LEFT, padx=8)
        bind_aps_tooltip(gen_btn, "asm_generate")

        self.next_step_var = tk.StringVar(value="")
        self._next_step_frame = ttk.Frame(gen)
        self._next_step_frame.pack(fill=tk.X, pady=(4, 0))
        self._next_step_lbl = ttk.Label(
            self._next_step_frame,
            textvariable=self.next_step_var,
            wraplength=880,
            justify=tk.LEFT,
            foreground="#0a4a7a",
            font=("Segoe UI", 9),
        )
        self._next_step_lbl.pack(anchor=tk.W)
        bind_aps_tooltip(self._next_step_lbl, "asm_material_lib")

        self.iterate_section = CollapsibleSection(
            gen, "Iterate grammar (advanced)", expanded=False, padding=2
        )
        self.iterate_section.pack(fill=tk.X, pady=4)
        self.iterate_panel = GrammarIteratePanel(
            self.iterate_section.body,
            on_applied=self._on_iterate_applied,
            on_log=self._on_log,
        )
        self.iterate_panel.pack(fill=tk.X)

        self.grammar_dna_section = CollapsibleSection(
            gen, "ARCH-DNA + β v0", expanded=True, padding=2
        )
        self.grammar_dna_section.pack(fill=tk.X, pady=4)
        self.grammar_dna_panel = GrammarDnaPanel(
            self.grammar_dna_section.body,
            on_change=self._on_grammar_dna_change,
        )
        self.grammar_dna_panel.pack(fill=tk.X)

        file_row = ttk.Frame(self)
        file_row.pack(fill=tk.X, pady=4)
        load_btn = ttk.Button(file_row, text="Load…", command=self.on_load)
        load_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(load_btn, "asm_load")
        save_btn = ttk.Button(file_row, text="Save", command=self.on_save)
        save_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(save_btn, "asm_save")
        val_btn = ttk.Button(file_row, text="Validate", command=self.on_validate)
        val_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(val_btn, "asm_validate")
        p0_btn = ttk.Button(file_row, text="P0 gate", command=self.on_validate_p0)
        p0_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(p0_btn, "asm_p0")
        self.path_var = tk.StringVar(value="(no snapshot)")
        ttk.Label(file_row, textvariable=self.path_var, foreground="#444").pack(side=tk.LEFT, padx=8)
        prev_btn = ttk.Button(file_row, text="Preview assembly", command=self.on_preview_assembly)
        prev_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(prev_btn, "asm_preview")

        workspace = horizontal_paned(self)
        workspace.pack(fill=tk.BOTH, expand=True, pady=8)

        footprint_pane = ttk.Frame(workspace, padding=4)
        materials_pane = ttk.Frame(workspace, padding=4)
        inspector_pane = ttk.Frame(workspace, padding=4)
        add_pane(workspace, footprint_pane, weight=2, minsize=240)
        add_pane(workspace, materials_pane, weight=2, minsize=220)
        add_pane(workspace, inspector_pane, weight=3, minsize=260)
        set_initial_pane_widths(
            workspace,
            [(footprint_pane, 0.30), (materials_pane, 0.28)],
        )

        ttk.Label(footprint_pane, text="Footprint & placements", font=("Segoe UI", 9, "bold")).pack(
            anchor=tk.W
        )
        self.placement_list = tk.Listbox(
            footprint_pane, exportselection=False, font=("Consolas", 9), height=5
        )
        self.placement_list.pack(fill=tk.X, pady=(4, 6))
        self.placement_list.bind("<<ListboxSelect>>", self.on_placement_select)
        attach_wheel_area(
            self.placement_list,
            on_scroll_y=lambda delta: self.placement_list.yview_scroll(int(-delta * 3), "units"),
            area_id=f"aps-asm-placements-{id(self)}",
        )
        self.footprint_canvas = FootprintCanvas(footprint_pane, on_cell_select=self._on_grid_cell_select)
        self.footprint_canvas.pack(fill=tk.BOTH, expand=True)
        bind_aps_tooltip(self.footprint_canvas, "asm_footprint")
        bind_aps_tooltip(self.placement_list, "asm_footprint_heatmap")

        mat_frame = ttk.LabelFrame(materials_pane, text="Material library", padding=4)
        mat_frame.pack(fill=tk.BOTH, expand=True)
        self.material_browser = MaterialBrowserPanel(
            mat_frame,
            on_apply_material=self._apply_material_profile,
            on_log=self._on_log,
            layout="vertical",
        )
        self.material_browser.pack(fill=tk.BOTH, expand=True)
        bind_aps_tooltip(mat_frame, "asm_material_lib")

        self.slot_preview = SlotPreviewPanel(inspector_pane, on_log=self._on_log)
        bind_aps_tooltip(self.slot_preview, "asm_slot_preview")
        self.slot_preview.pack(fill=tk.X, pady=(0, 6))

        self.assembly_preview = AssemblyPreviewPanel(
            inspector_pane,
            on_log=self._on_log,
            on_preview_thumb=self._on_assembly_preview_thumb,
            start_job=self._start_job,
        )
        self.assembly_preview.pack(fill=tk.X, pady=(0, 8))

        slot = ttk.LabelFrame(inspector_pane, text="Selected slot — edit", padding=8)
        slot.pack(fill=tk.BOTH, expand=True)

        ttk.Label(slot, text="Node id").grid(row=0, column=0, sticky=tk.W)
        self.node_id_var = tk.StringVar(value="—")
        ttk.Label(slot, textvariable=self.node_id_var, font=("Consolas", 9)).grid(
            row=0, column=1, sticky=tk.W, padx=4
        )

        ttk.Label(slot, text="Module").grid(row=1, column=0, sticky=tk.W, pady=4)
        self.module_var = tk.StringVar(value="")
        ttk.Entry(slot, textvariable=self.module_var, width=28, state="readonly").grid(
            row=1, column=1, sticky=tk.W, padx=4
        )

        ttk.Label(slot, text="Material profile").grid(row=2, column=0, sticky=tk.W, pady=4)
        self.material_var = tk.StringVar(value="—")
        self.material_category_var = tk.StringVar(value="")
        mat_row = ttk.Frame(slot)
        mat_row.grid(row=2, column=1, sticky=tk.W, padx=4)
        self._material_swatch = tk.Label(mat_row, text="  ", width=2, bg="#dddddd", relief=tk.RIDGE)
        self._material_swatch.pack(side=tk.LEFT, padx=(0, 6))
        mat_col = ttk.Frame(mat_row)
        mat_col.pack(side=tk.LEFT)
        ttk.Label(mat_col, textvariable=self.material_var, font=("Consolas", 9)).pack(anchor=tk.W)
        ttk.Label(
            mat_col, textvariable=self.material_category_var, font=("Segoe UI", 8), foreground="#555"
        ).pack(anchor=tk.W)
        mat_btn_row = ttk.Frame(slot)
        mat_btn_row.grid(row=2, column=2, sticky=tk.W, padx=4)
        if self._on_open_in_materials:
            ttk.Button(mat_btn_row, text="Open in Materials tab", command=self._open_in_materials_tab).pack(
                anchor=tk.W
            )

        ttk.Label(slot, text="LOD policy").grid(row=3, column=0, sticky=tk.W, pady=4)
        self.lod_var = tk.StringVar(value="production")
        ttk.Combobox(
            slot,
            textvariable=self.lod_var,
            width=14,
            values=["lod0", "production", "hero"],
            state="readonly",
        ).grid(row=3, column=1, sticky=tk.W, padx=4)

        tag_section = CollapsibleSection(slot, "Semantic & variant tags", expanded=False, padding=4)
        tag_section.grid(row=5, column=0, columnspan=3, sticky=tk.EW, pady=(8, 0))
        self._tag_section = tag_section
        tag_body = tag_section.body

        tag_filter_row = ttk.Frame(tag_body)
        tag_filter_row.pack(fill=tk.X, pady=(0, 2))
        ttk.Label(tag_filter_row, text="Tag category filter").pack(side=tk.LEFT)
        self.tag_filter_var = tk.StringVar(value="all")
        ttk.Combobox(
            tag_filter_row,
            textvariable=self.tag_filter_var,
            width=14,
            values=["all"] + list(aps_tags.CATEGORY_ORDER),
            state="readonly",
        ).pack(side=tk.LEFT, padx=4)
        self.tag_filter_var.trace_add("write", lambda *_: self._apply_tag_category_filter())

        tags_outer = ttk.Frame(tag_body)
        tags_outer.pack(fill=tk.X)
        self._build_semantic_tag_pickers(tags_outer)

        var_frame = ttk.LabelFrame(tag_body, text="Variant tags", padding=4)
        var_frame.pack(fill=tk.X, pady=4)
        var_grid = ttk.Frame(var_frame)
        var_grid.pack(anchor=tk.W)
        for i, tag in enumerate(assembly.COMMON_VARIANT_TAGS):
            var = tk.BooleanVar(value=False)
            self._variant_tag_vars[tag] = var
            ttk.Checkbutton(var_grid, text=tag, variable=var).grid(row=0, column=i, sticky=tk.W, padx=4)

        btn_row = ttk.Frame(tag_body)
        btn_row.pack(fill=tk.X, pady=(4, 0))
        ttk.Button(btn_row, text="Apply tags to slot", command=self.on_apply_slot).pack(side=tk.LEFT, padx=2)

        grammar_section = CollapsibleSection(
            inspector_pane, "Grammar inspector", expanded=False, padding=2
        )
        grammar_section.pack(fill=tk.X, pady=4)
        self._grammar_section = grammar_section
        self.grammar_inspector = GrammarInspectorPanel(grammar_section.body)
        self.grammar_inspector.pack(fill=tk.X)

        self.validation_var = tk.StringVar(value="")
        self._validation_lbl = tk.Label(
            inspector_pane,
            textvariable=self.validation_var,
            wraplength=420,
            foreground="#444444",
            font=FONT_UI,
        )
        self._validation_lbl.pack(anchor=tk.W, pady=4)

        def _validation_wrap(_event=None) -> None:
            self._validation_lbl.configure(wraplength=wrap_for_widget(inspector_pane, minimum=320))

        inspector_pane.bind("<Configure>", _validation_wrap)

        slot.columnconfigure(1, weight=1)
        self._on_grammar_toggle()

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
                ttk.Checkbutton(grid, text=label, variable=var).grid(
                    row=i // 3, column=i % 3, sticky=tk.W, padx=4
                )

    def _set_validation_result(self, text: str, *, ok: bool | None = None) -> None:
        if ok is False and text and not text.lower().startswith("validation"):
            text = f"Validation: FAIL — {text}"
        elif ok is True and text and "passed" not in text.lower():
            text = f"Validation: PASS — {text}"
        set_inline_status(self._validation_lbl, self.validation_var, text, ok=ok)

    def show_material_assign_callout(self, profile_id: str) -> None:
        if self._snapshot:
            self.next_step_var.set(
                f"Profile {profile_id} highlighted — select a footprint cell, then Apply to selected slot."
            )
        else:
            self.next_step_var.set("Generate or load an assembly snapshot first, then select a footprint cell.")
        bind_aps_tooltip(self._next_step_lbl, "asm_material_lib")

    def _apply_tag_category_filter(self) -> None:
        filt = self.tag_filter_var.get().strip().lower()
        for cat, frame in self._tag_category_frames.items():
            if filt == "all" or filt == cat:
                frame.pack(fill=tk.X, pady=2)
            else:
                frame.pack_forget()

    def _count_active_tags(self) -> int:
        n = sum(1 for tag_map in self._semantic_tag_vars.values() for var in tag_map.values() if var.get())
        n += sum(1 for var in self._variant_tag_vars.values() if var.get())
        return n

    def _grammar_section_title(self) -> str:
        base = "Grammar inspector"
        if not self._snapshot:
            return base
        arch = human_label(str(self._snapshot.get("archetype_id") or ""))
        if arch and arch not in ("—", ""):
            return f"{base} — {arch}"
        return base

    def _refresh_collapsible_titles(self) -> None:
        tag_title = "Semantic & variant tags"
        n = self._count_active_tags()
        if n:
            tag_title = f"{tag_title} ({n} selected)"
        self._tag_section.set_title(tag_title)
        self._grammar_section.set_title(self._grammar_section_title())

    def _on_material_browser_apply(self, profile_id: str) -> None:
        self.material_var.set(profile_id)
        if self._snapshot and self._selected_node_id:
            self.on_apply_slot()
        else:
            self._on_log(f"material selected: {profile_id} (pick a slot to apply)")

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

    def _on_archetype_change(self, _event=None) -> None:
        archetype = self.archetype_var.get().strip()
        if not archetype:
            return
        districts = building_grammar.list_district_styles(archetype)
        self.district_combo.configure(values=districts or [""])
        if districts:
            self.district_var.set(districts[0])

    def sync_from_state(self) -> None:
        self.style_var.set(self.state.style_pack_id)
        self.footprint_var.set(self.state.footprint)
        self.floors_var.set(self.state.floors)
        self.seed_var.set(self.state.seed)

    def _sync_state_from_snapshot(self, snap: dict) -> None:
        self.state.assembly_id = str(snap.get("assembly_id"))
        rel = snap.get("written_path")
        if rel:
            self.state.assembly_snapshot_path = str(rel)
        self.state.style_pack_id = str(snap.get("style_pack_id") or self.state.style_pack_id)
        fp = snap.get("footprint") or {}
        w, d, f = fp.get("width"), fp.get("depth"), fp.get("floors")
        if w and d:
            self.state.footprint = f"{w}x{d}"
            self.footprint_var.set(self.state.footprint)
        if f:
            self.state.floors = int(f)
            self.floors_var.set(int(f))
        if snap.get("seed") is not None:
            self.state.seed = int(snap["seed"])
            self.seed_var.set(int(snap["seed"]))
        if snap.get("archetype_id"):
            self.use_grammar_var.set(True)
            self.archetype_var.set(str(snap["archetype_id"]))
            self._on_archetype_change()
            if snap.get("district_style"):
                self.district_var.set(str(snap["district_style"]))
            self._on_grammar_toggle()
        self.state.module_ids_in_assembly = sorted(
            {str(p.get("module_id")) for p in snap.get("module_placements") or []}
        )
        self.state.assembly_snapshot_data = snap

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
        for p in self._sorted_placements():
            self.placement_list.insert(tk.END, self._placement_label(p))

    def _refresh_footprint_grid(self) -> None:
        if not self._snapshot:
            self.footprint_canvas.set_cells([], [])
            return
        cells = assembly.footprint_cells_for_snapshot(self._snapshot)
        self.footprint_canvas.set_cells(cells, self._sorted_placements())

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
        self._snapshot = self._apply_grammar_dna_from_ui(self._snapshot)

    def _apply_grammar_dna_from_ui(self, snap: dict) -> dict:
        state = self.grammar_dna_panel.get_state()
        return arch_build_grammar.apply_to_snapshot(
            snap,
            preset_id=str(state.get("preset_id") or arch_build_grammar.default_preset_id()),
            pressure_field=state.get("pressure_field"),
            include=bool(state.get("include")),
        )

    def _load_snapshot_into_ui(self, snap: dict, *, path_hint: str = "") -> None:
        self._snapshot = assembly.enrich_snapshot(snap)
        self._sync_state_from_snapshot(self._snapshot)
        self.iterate_panel.set_base_snapshot(self._snapshot)
        self.grammar_dna_panel.set_from_snapshot(self._snapshot)
        self.path_var.set(path_hint or self.state.assembly_snapshot_path or "(memory)")
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self.grammar_inspector.load_snapshot(self._snapshot)
        self.assembly_preview.set_snapshot(self._snapshot)
        self.save_hint_var.set(save_hint(self._snapshot))
        self._refresh_collapsible_titles()
        if self.placement_list.size():
            self.placement_list.selection_set(0)
            self.on_placement_select()

    def on_generate(self) -> None:
        seed = int(self.seed_var.get())
        tier = self.tier_var.get().strip() or "lod0"
        try:
            if self.use_grammar_var.get():
                archetype = self.archetype_var.get().strip()
                district = self.district_var.get().strip()
                self._on_log(f"assembly-snapshot-generate grammar {archetype}/{district} seed={seed}")
                snap = assembly.generate_assembly_snapshot(
                    archetype_id=archetype,
                    district_style=district,
                    seed=seed,
                    source_tier=tier,
                )
            else:
                style = self.style_var.get().strip()
                fp = self.footprint_var.get().strip().lower()
                w, d = fp.split("x")
                width, depth = int(w), int(d)
                floors = int(self.floors_var.get())
                self._on_log(f"assembly-snapshot-generate {style} {width}x{depth} tier={tier}")
                snap = assembly.generate_assembly_snapshot(
                    style_pack_id=style,
                    width=width,
                    depth=depth,
                    floors=floors,
                    seed=seed,
                    source_tier=tier,
                )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Generate failed: {exc}", ok=False)
            return
        snap = self._apply_grammar_dna_from_ui(snap)
        self._load_snapshot_into_ui(snap, path_hint=str(snap.get("written_path") or ""))
        self._on_log(f"wrote {self.state.assembly_snapshot_path}")
        self.next_step_var.set(
            "Next: Select a footprint cell → Materials tab → Apply profile → Save snapshot "
            "(sidecar tags in Catalog are hints only)."
        )
        bind_aps_tooltip(self._next_step_lbl, "asm_save_reminder")
        rep = self._p0_report()
        if rep.status == "passed":
            self._set_validation_result(
                f"Snapshot OK · {self.state.assembly_id} · P0 gate passed",
                ok=True,
            )
        else:
            hints = self._format_validation_hints(rep)
            self._set_validation_result(f"P0 failed: {hints[:200]}", ok=False)
            self._on_log(f"generate P0 failed: {hints[:400]}")

    def _p0_report(self):
        import tempfile

        from rust_engine_mcp.validators.report import ValidationReport

        if not self._snapshot:
            return ValidationReport(
                validator="assembly_p0",
                status="failed",
                compression_level=3,
                summary="no snapshot",
                error_count=1,
                errors=[],
            )
        path = self.state.assembly_snapshot_path
        if path and (repo_root() / path).is_file():
            snap_path = str(path)
        else:
            tmp = Path(tempfile.gettempdir()) / "_aps_assembly_p0_validate.json"
            tmp.write_text(json.dumps(self._snapshot, indent=2), encoding="utf-8")
            snap_path = str(tmp)
        return validate_assembly_p0_gate(
            self._snapshot,
            snapshot_path=snap_path.replace("\\", "/"),
            ship=True,
        )

    @staticmethod
    def _format_validation_hints(rep) -> str:
        return format_p0_display(rep, limit=20)[:1200]

    def _run_p0_or_block(self, action: str) -> bool:
        rep = self._p0_report()
        if rep.status == "passed":
            self._set_validation_result(f"P0 gate: passed — {action} OK", ok=True)
            return True
        hints = self._format_validation_hints(rep)
        self._set_validation_result(f"P0 failed: {hints[:200]}", ok=False)
        return messagebox.askyesno(
            f"P0 gate failed — {action} anyway?",
            f"{hints}\n\nProceed anyway? (Not recommended for ship/bake.)",
        )

    def on_load(self) -> None:
        initial = repo_root() / "assets" / "staging" / "assemblies"
        path = filedialog.askopenfilename(
            title="Load assembly snapshot",
            initialdir=str(initial) if initial.is_dir() else str(repo_root()),
            filetypes=[("JSON", "*.json"), ("All", "*.*")],
        )
        if not path:
            return
        try:
            snap = assembly.load_assembly_snapshot(path)
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Load failed: {exc}", ok=False)
            return
        rel = Path(path).resolve().relative_to(repo_root()).as_posix()
        snap["written_path"] = rel
        self._load_snapshot_into_ui(snap, path_hint=rel)
        self._on_log(f"loaded {rel}")

    def on_save(self) -> None:
        if not self._snapshot:
            self._set_validation_result("Generate or load a snapshot first.", ok=False)
            return
        if not self._run_p0_or_block("Save"):
            self._on_log("save cancelled — P0 gate failed")
            return
        try:
            self._snapshot = self._apply_grammar_dna_from_ui(self._snapshot)
            out = assembly.save_assembly_snapshot(self._snapshot)
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Save failed: {exc}", ok=False)
            return
        rel = str(out.relative_to(repo_root())).replace("\\", "/")
        self.state.assembly_snapshot_path = rel
        self.path_var.set(rel)
        self._snapshot["written_path"] = rel
        self._on_log(f"saved {rel}")
        self.save_hint_var.set(save_hint(self._snapshot))
        hint = save_hint(self._snapshot)
        msg = f"Saved {rel}"
        if "missing material_profile" in hint:
            msg += f" · {hint}"
        self._set_validation_result(msg, ok=True)

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
            self._on_log("preview cancelled — P0 gate failed")
            return
        self.assembly_preview.on_preview()

    def on_validate(self) -> None:
        if not self._snapshot:
            self._set_validation_result("No snapshot loaded.", ok=False)
            return
        path = self.state.assembly_snapshot_path
        if path:
            rep = validate_assembly_snapshot_path(repo_root() / path, ship=True)
        else:
            import tempfile

            tmp = Path(tempfile.gettempdir()) / "_aps_assembly_validate.json"
            tmp.write_text(json.dumps(self._snapshot, indent=2), encoding="utf-8")
            rep = validate_assembly_snapshot_path(tmp, ship=True)
        self._show_validation_report(rep, title="Validate (production)")

    def on_validate_p0(self) -> None:
        if not self._snapshot:
            self._set_validation_result("No snapshot loaded.", ok=False)
            return
        rep = self._p0_report()
        self._show_validation_report(rep, title="P0 gate (production + grammar)")

    def _show_validation_report(self, rep, *, title: str) -> None:
        if rep.status == "passed":
            self._set_validation_result(f"{title}: passed", ok=True)
            self._on_log(f"{title}: passed")
        else:
            hints = self._format_validation_hints(rep)
            self._set_validation_result(f"{title} failed: {hints[:200]}", ok=False)
            self._on_log(f"{title} failed: {hints[:800]}")

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

    def _open_in_materials_tab(self) -> None:
        mat = self.material_var.get().strip()
        if mat and mat != "—" and self._on_open_in_materials:
            self._on_open_in_materials(mat)

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
        self._material_swatch.configure(bg=color)

    def _apply_material_profile(self, profile_id: str) -> None:
        if not self._snapshot or not self._selected_node_id:
            self._set_validation_result("Select a placement row or grid cell first.", ok=False)
            return
        self.material_var.set(profile_id)
        self._update_material_category(profile_id)
        self._update_material_swatch(profile_id)
        try:
            self._snapshot = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                material_profile=profile_id,
            )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Material apply failed: {exc}", ok=False)
            return
        self.state.assembly_snapshot_data = self._snapshot
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._on_log(f"material {profile_id} → {self._selected_node_id}")
        self._set_validation_result(f"Material {profile_id} applied — Save snapshot before bake", ok=None)

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
            self._snapshot = assembly.update_placement(
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
        self.state.assembly_snapshot_data = self._snapshot
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._on_log(f"updated {self._selected_node_id} material={self.material_var.get()}")
        self._set_validation_result("Slot updated — run Validate before bake", ok=None)
