"""APSR-P1 — AssemblyPanel layout (_build)."""
from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from rust_engine_mcp import aps_tags, assembly, building_grammar
from rust_engine_mcp.aps_grammar_labels import human_label

from .aps_collapsible import CollapsibleSection
from .aps_onboarding_panel import empty_state_label
from .aps_paned import add_pane, horizontal_paned, set_initial_pane_widths
from .aps_scroll import attach_wheel_area
from .aps_tooltips import bind_aps_tooltip
from .aps_theme import (
    COLOR_ACCENT,
    COLOR_MUTED,
    COLOR_PREVIEW_PLACEHOLDER,
    COLOR_TEXT_HINT,
    COLOR_TEXT_SUBTLE,
    FONT_MONO,
    FONT_MONO_SMALL,
    FONT_SMALL,
    FONT_UI,
    FONT_UI_BOLD,
    MIN_WINDOW_SIZE,
    VALIDATION_BANNER_MIN_PX,
    track_wraplength,
    wrap_for_widget,
)
from .aps_tk import themed_listbox
from .assembly_onboard_strip import AssemblyOnboardStrip
from .golden_seed_review_panel import GoldenSeedReviewPanel
from .assembly_qc_strip import AssemblyQcStrip
from .assembly_panel_common import MATERIAL_AUTHORITY_COPY, grammar_combo_maps
from .assembly_preview_panel import AssemblyPreviewPanel
from .facility_needs_strip import FacilityNeedsStrip
from .footprint_canvas import FootprintCanvas
from .generation_trace_strip import GenerationTraceStrip
from .grammar_build_set_panel import GrammarBuildSetPanel
from .grammar_dna_panel import GrammarDnaPanel
from .grammar_inspector import GrammarInspectorPanel
from .grammar_iterate_panel import GrammarIteratePanel
from .material_browser import mount_material_library
from .site_preview_panel import SiteLayoutPreviewSection
from .slot_preview_panel import SlotPreviewPanel


class AssemblyPanelLayoutMixin:
    def _build(self) -> None:
        intro = ttk.Label(
            self,
            text="Pick type and district, generate, assign materials, then ship check.",
            wraplength=900,
            justify=tk.LEFT,
            foreground=COLOR_TEXT_SUBTLE,
            font=FONT_SMALL,
        )
        intro.pack(anchor=tk.W, pady=(0, 4))
        track_wraplength(self, intro, minimum=480)
        self.metadata_flow = AssemblyOnboardStrip(self)
        self.metadata_flow.pack(fill=tk.X, pady=(0, 6))

        self._qc_strip = AssemblyQcStrip(self)
        self._qc_strip.pack(fill=tk.X, pady=(0, 6))
        self._qc_strip.refresh()

        self._gen_trace = GenerationTraceStrip(
            self,
            self.state,
            get_snapshot=lambda: self._snapshot,
            get_assembly_id=lambda: str((self._snapshot or {}).get("assembly_id") or self.state.assembly_id or ""),
        )
        self._gen_trace.pack(fill=tk.X, pady=(0, 6))

        self._golden_seed_panel = GoldenSeedReviewPanel(
            self,
            on_load_snapshot=self._load_snapshot_into_ui,
            on_log=self._on_log,
        )
        self._golden_seed_panel.pack(fill=tk.X, pady=(0, 6))

        # --- Step 1: Generate (primary workflow — always visible) ---
        gen = ttk.Frame(self)
        gen.pack(fill=tk.X, pady=(0, 4))

        self._grammar_set_tier_var = tk.StringVar(value="G0 — pilot kit")
        self._grammar_tier_strip = ttk.Label(
            gen,
            textvariable=self._grammar_set_tier_var,
            font=FONT_UI_BOLD,
            foreground=COLOR_ACCENT,
        )
        self._grammar_tier_strip.pack(anchor=tk.W, pady=(0, 4))

        archetypes = building_grammar.list_archetype_ids() or ["IndustrialWarehouse"]
        arch_labels, self._archetype_label_to_id = grammar_combo_maps(archetypes)
        districts = building_grammar.list_district_styles(archetypes[0]) or ["industrial_west"]
        dist_labels, self._district_label_to_id = grammar_combo_maps(districts)

        primary = ttk.Frame(gen)
        primary.pack(fill=tk.X, pady=2)
        ttk.Label(primary, text="Building type").pack(side=tk.LEFT)
        self.archetype_var = tk.StringVar(value=arch_labels[0] if arch_labels else "")
        self.archetype_combo = ttk.Combobox(
            primary, textvariable=self.archetype_var, width=26, values=arch_labels, state="readonly"
        )
        self.archetype_combo.pack(side=tk.LEFT, padx=4)
        self.archetype_combo.bind("<<ComboboxSelected>>", self._on_archetype_change)
        bind_aps_tooltip(self.archetype_combo, "asm_archetype")
        ttk.Label(primary, text="District").pack(side=tk.LEFT, padx=(8, 0))
        self.district_var = tk.StringVar(value=dist_labels[0] if dist_labels else "")
        self.district_combo = ttk.Combobox(
            primary, textvariable=self.district_var, width=22, values=dist_labels, state="readonly"
        )
        self.district_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.district_combo, "asm_district")
        ttk.Label(primary, text="Seed").pack(side=tk.LEFT, padx=(8, 0))
        self.seed_var = tk.IntVar(value=self.state.seed)
        ttk.Spinbox(primary, from_=0, to=999999, textvariable=self.seed_var, width=8).pack(side=tk.LEFT, padx=4)
        gen_btn = ttk.Button(primary, text="Generate Assembly", command=self.on_generate)
        gen_btn.pack(side=tk.LEFT, padx=(12, 0))
        bind_aps_tooltip(gen_btn, "asm_generate")

        self._grammar_kit_var = tk.StringVar(value="")
        kit_hint = ttk.Label(
            gen,
            textvariable=self._grammar_kit_var,
            wraplength=880,
            justify=tk.LEFT,
            foreground=COLOR_TEXT_HINT,
            font=FONT_UI,
        )
        kit_hint.pack(anchor=tk.W, pady=(2, 0))
        track_wraplength(gen, kit_hint, minimum=480)
        self._kit_hint_label = kit_hint

        self._set_health_var = tk.StringVar(value="")
        self._set_health_strip = ttk.Frame(gen)
        ttk.Label(
            self._set_health_strip,
            textvariable=self._set_health_var,
            wraplength=720,
            justify=tk.LEFT,
            font=FONT_SMALL,
            foreground=COLOR_TEXT_SUBTLE,
        ).pack(side=tk.LEFT, fill=tk.X, expand=True)
        ttk.Button(
            self._set_health_strip,
            text="Run sweep",
            command=self._on_set_health_sweep,
        ).pack(side=tk.RIGHT, padx=(8, 0))

        self.next_step_var = tk.StringVar(value="")
        self._next_step_frame = ttk.Frame(gen)
        self._next_step_frame.pack(fill=tk.X, pady=(4, 0))
        self._next_step_lbl = ttk.Label(
            self._next_step_frame,
            textvariable=self.next_step_var,
            wraplength=880,
            justify=tk.LEFT,
            foreground=COLOR_ACCENT,
            font=FONT_UI,
        )
        self._next_step_lbl.pack(anchor=tk.W)
        bind_aps_tooltip(self._next_step_lbl, "asm_material_lib")

        self.save_hint_var = tk.StringVar(value="")
        ttk.Label(
            gen,
            textvariable=self.save_hint_var,
            wraplength=880,
            justify=tk.LEFT,
            foreground=COLOR_ACCENT,
            font=FONT_SMALL,
        ).pack(anchor=tk.W, pady=(2, 0))

        self.use_grammar_var = tk.BooleanVar(value=bool(archetypes))

        # --- Step 2–4: Footprint · Materials · Inspector (main work area) ---
        workspace = horizontal_paned(self)
        workspace.pack(fill=tk.BOTH, expand=True, pady=4)

        footprint_pane = ttk.Frame(workspace, padding=4)
        materials_pane = ttk.Frame(workspace, padding=4)
        inspector_pane = ttk.Frame(workspace, padding=4)
        _min_w = MIN_WINDOW_SIZE[0]
        _fp_min, _mat_min, _insp_min = (215, 195, 215) if _min_w <= 1024 else (240, 220, 260)
        add_pane(workspace, footprint_pane, weight=2, minsize=_fp_min)
        add_pane(workspace, materials_pane, weight=2, minsize=_mat_min)
        add_pane(workspace, inspector_pane, weight=3, minsize=_insp_min)
        set_initial_pane_widths(
            workspace,
            [(footprint_pane, 0.30), (materials_pane, 0.28)],
        )

        ttk.Label(footprint_pane, text="Footprint & placements", font=FONT_UI_BOLD).pack(
            anchor=tk.W
        )
        self._empty_state = empty_state_label(footprint_pane, "assembly")
        self._empty_state.pack(anchor=tk.W, pady=2)
        self.placement_list = themed_listbox(
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
        self.site_layout = SiteLayoutPreviewSection(footprint_pane, on_log=self._on_log)
        bind_aps_tooltip(self.site_layout, "asm_site_layout")
        bind_aps_tooltip(self.placement_list, "asm_footprint_heatmap")

        mat_frame = ttk.LabelFrame(
            materials_pane,
            text="Material library — assign here to ship",
            padding=4,
        )
        mat_frame.pack(fill=tk.BOTH, expand=True)
        self.engine_path_var = tk.StringVar(value=MATERIAL_AUTHORITY_COPY)
        auth_lbl = ttk.Label(
            mat_frame,
            textvariable=self.engine_path_var,
            wraplength=420,
            justify=tk.LEFT,
            font=FONT_SMALL,
            foreground=COLOR_MUTED,
        )
        auth_lbl.pack(anchor=tk.W, pady=(0, 4))
        bind_aps_tooltip(auth_lbl, "asm_engine_path")
        self.material_browser = mount_material_library(
            mat_frame,
            mount="assign",
            on_apply_material=self._apply_material_profile,
            on_log=self._on_log,
        )
        self.material_browser.pack(fill=tk.BOTH, expand=True)
        bind_aps_tooltip(mat_frame, "asm_material_lib")

        preview_section = CollapsibleSection(inspector_pane, "Previews", expanded=True, padding=4)
        preview_section.pack(fill=tk.X, pady=(0, 6))
        preview_body = preview_section.body

        self.slot_preview = SlotPreviewPanel(preview_body, on_log=self._on_log, start_job=self._start_job)
        bind_aps_tooltip(self.slot_preview, "asm_slot_preview")
        self.slot_preview.pack(fill=tk.X, pady=(0, 6))

        self.assembly_preview = AssemblyPreviewPanel(
            preview_body,
            on_log=self._on_log,
            on_preview_thumb=self._on_assembly_preview_thumb,
            start_job=self._start_job,
        )
        self.assembly_preview.pack(fill=tk.X, pady=(0, 4))

        slot = ttk.LabelFrame(inspector_pane, text="Selected piece — edit", padding=8)
        slot.pack(fill=tk.BOTH, expand=True)

        ttk.Label(slot, text="Piece id").grid(row=0, column=0, sticky=tk.W)
        self.node_id_var = tk.StringVar(value="—")
        ttk.Label(slot, textvariable=self.node_id_var, font=("Consolas", 9)).grid(
            row=0, column=1, sticky=tk.W, padx=4
        )

        ttk.Label(slot, text="Module").grid(row=1, column=0, sticky=tk.W, pady=4)
        self.module_var = tk.StringVar(value="")
        mod_row = ttk.Frame(slot)
        mod_row.grid(row=1, column=1, sticky=tk.W, padx=4)
        self.module_combo = ttk.Combobox(mod_row, textvariable=self.module_var, width=26, values=[])
        self.module_combo.pack(side=tk.LEFT)
        self.module_combo.bind("<<ComboboxSelected>>", self._on_module_picked)
        self.module_combo.bind("<Return>", self._on_module_picked)
        bind_aps_tooltip(self.module_combo, "asm_module_picker")
        self._module_resolve_var = tk.StringVar(value="")
        ttk.Label(
            mod_row, textvariable=self._module_resolve_var, font=FONT_SMALL, foreground=COLOR_TEXT_SUBTLE
        ).pack(side=tk.LEFT, padx=(8, 0))

        ttk.Label(slot, text="Material").grid(row=2, column=0, sticky=tk.W, pady=4)
        self.material_var = tk.StringVar(value="—")
        self.material_category_var = tk.StringVar(value="")
        mat_row = ttk.Frame(slot)
        mat_row.grid(row=2, column=1, sticky=tk.W, padx=4)
        self._material_swatch = tk.Label(
            mat_row, text="—", width=3, bg=COLOR_PREVIEW_PLACEHOLDER, relief=tk.RIDGE, font=FONT_SMALL
        )
        self._material_swatch.pack(side=tk.LEFT, padx=(0, 6))
        mat_col = ttk.Frame(mat_row)
        mat_col.pack(side=tk.LEFT)
        ttk.Label(mat_col, textvariable=self.material_var, font=FONT_MONO_SMALL).pack(anchor=tk.W)
        ttk.Label(
            mat_col, textvariable=self.material_category_var, font=FONT_SMALL, foreground=COLOR_MUTED
        ).pack(anchor=tk.W)
        mat_btn_row = ttk.Frame(slot)
        mat_btn_row.grid(row=2, column=2, sticky=tk.W, padx=4)
        if self._on_open_in_materials:
            ttk.Button(mat_btn_row, text="Open in Materials tab", command=self._open_in_materials_tab).pack(
                anchor=tk.W
            )

        ttk.Label(slot, text="Detail level").grid(row=3, column=0, sticky=tk.W, pady=4)
        self.lod_var = tk.StringVar(value="production")
        ttk.Combobox(
            slot,
            textvariable=self.lod_var,
            width=14,
            values=["rough", "production", "hero"],
            state="readonly",
        ).grid(row=3, column=1, sticky=tk.W, padx=4)

        tag_section = CollapsibleSection(slot, "Tags (look & state)", expanded=False, padding=4)
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
            from rust_engine_mcp.aps_tag_vocabulary import assembly_variant_tag_label

            cb = ttk.Checkbutton(var_grid, text=assembly_variant_tag_label(tag), variable=var)
            cb.grid(row=0, column=i, sticky=tk.W, padx=4)
            bind_aps_tooltip(cb, f"asm_variant_tag:{tag}")

        btn_row = ttk.Frame(tag_body)
        btn_row.pack(fill=tk.X, pady=(4, 0))
        ttk.Button(btn_row, text="Save tags to this piece", command=self.on_apply_slot).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Apply tag preset…", command=self._on_apply_semantic_preset).pack(
            side=tk.LEFT, padx=2
        )

        grammar_section = CollapsibleSection(
            inspector_pane, "Grammar inspector", expanded=False, padding=2
        )
        grammar_section.pack(fill=tk.X, pady=4)
        self._grammar_section = grammar_section
        self.grammar_inspector = GrammarInspectorPanel(
            grammar_section.body,
            on_rule_select=self._on_grammar_inspector_rule_select,
        )
        self.grammar_inspector.pack(fill=tk.X)

        self.validation_var = tk.StringVar(value="")
        validation_holder = ttk.Frame(inspector_pane, height=VALIDATION_BANNER_MIN_PX)
        validation_holder.pack(anchor=tk.W, fill=tk.X, pady=4)
        validation_holder.pack_propagate(False)
        self._validation_lbl = ttk.Label(
            validation_holder,
            textvariable=self.validation_var,
            wraplength=420,
            font=FONT_UI,
        )
        self._validation_lbl.pack(anchor=tk.W, fill=tk.X)

        def _validation_wrap(_event=None) -> None:
            self._validation_lbl.configure(wraplength=wrap_for_widget(inspector_pane, minimum=320))

        inspector_pane.bind("<Configure>", _validation_wrap)

        slot.columnconfigure(1, weight=1)

        # --- File / ship actions ---
        file_row = ttk.Frame(self)
        file_row.pack(fill=tk.X, pady=4)
        load_btn = ttk.Button(file_row, text="Load…", command=self.on_load)
        load_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(load_btn, "asm_load")
        save_btn = ttk.Button(file_row, text="Save", command=self.on_save)
        save_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(save_btn, "asm_save")
        val_btn = ttk.Button(file_row, text="Check schema", command=self.on_validate)
        val_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(val_btn, "asm_validate")
        p0_btn = ttk.Button(file_row, text="Run ship check", command=self.on_validate_p0)
        p0_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(p0_btn, "asm_p0")
        self.path_var = tk.StringVar(value="(no snapshot)")
        ttk.Label(file_row, textvariable=self.path_var, foreground=COLOR_TEXT_SUBTLE).pack(side=tk.LEFT, padx=8)
        prev_btn = ttk.Button(file_row, text="Preview assembly", command=self.on_preview_assembly)
        prev_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(prev_btn, "asm_preview")

        # --- Setup & manual fallback (collapsed by default) ---
        self._setup_section = CollapsibleSection(
            self, "Setup & manual fallback", expanded=False, padding=4
        )
        self._setup_section.pack(fill=tk.X, pady=(0, 4))
        setup_body = self._setup_section.body

        gram_row = ttk.Frame(setup_body)
        gram_row.pack(fill=tk.X, pady=2)
        gram_cb = ttk.Checkbutton(
            gram_row,
            text="Use building style rules (recommended)",
            variable=self.use_grammar_var,
            command=self._on_grammar_toggle,
        )
        gram_cb.pack(side=tk.LEFT)
        bind_aps_tooltip(gram_cb, "asm_grammar")

        self.facility_needs = FacilityNeedsStrip(setup_body)
        self.facility_needs.pack(fill=tk.X, pady=(4, 4))

        manual = ttk.LabelFrame(setup_body, text="Manual override (when grammar off)", padding=4)
        manual.pack(fill=tk.X, pady=2)
        row = ttk.Frame(manual)
        row.pack(fill=tk.X, pady=2)
        ttk.Label(row, text="Style pack").pack(side=tk.LEFT)
        packs = assembly.list_style_packs()
        self.style_var = tk.StringVar(value=self.state.style_pack_id)
        self.style_combo = ttk.Combobox(
            row, textvariable=self.style_var, width=22, values=packs or ["style_victorian"]
        )
        self.style_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.style_combo, "asm_style_pack")
        ttk.Label(row, text="Source tier").pack(side=tk.LEFT, padx=(8, 0))
        self.tier_var = tk.StringVar(value="production")
        self.tier_combo = ttk.Combobox(
            row, textvariable=self.tier_var, width=12, values=["production", "lod0"], state="readonly"
        )
        self.tier_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.tier_combo, "asm_tier")
        row2 = ttk.Frame(manual)
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

        self.iterate_section = CollapsibleSection(
            setup_body, "Tweak one style layer (advanced)", expanded=False, padding=2
        )
        self.iterate_section.pack(fill=tk.X, pady=4)
        self.iterate_panel = GrammarIteratePanel(
            self.iterate_section.body,
            on_applied=self._on_iterate_applied,
            on_log=self._on_log,
        )
        self.iterate_panel.pack(fill=tk.X)

        self.grammar_dna_section = CollapsibleSection(
            setup_body, "Building shape bias (advanced)", expanded=False, padding=2
        )
        self.grammar_dna_section.pack(fill=tk.X, pady=4)
        bind_aps_tooltip(self.grammar_dna_section._head_btn, "asm_grammar_dna")
        self.grammar_dna_panel = GrammarDnaPanel(
            self.grammar_dna_section.body,
            on_change=self._on_grammar_dna_change,
        )
        self.grammar_dna_panel.pack(fill=tk.X)

        # --- Kit grammar reference (bottom, advanced) ---
        self._grammar_set_section = CollapsibleSection(
            self, "Kit grammar reference (advanced)", expanded=False, padding=4
        )
        self._grammar_set_section.pack(fill=tk.X, pady=(0, 4))
        self.grammar_set_panel = GrammarBuildSetPanel(self._grammar_set_section.body, on_log=self._on_log)
        self.grammar_set_panel.pack(fill=tk.X)

        self._on_grammar_toggle()
        self.refresh_grammar_tier_from_registry()
        self._refresh_facility_needs()
