"""Assembly Editor (APS-UI-003b) — footprint grid, grammar, categorized semantic tags."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk
from typing import Any

from rust_engine_mcp import aps_tags, assembly, arch_build_grammar, building_grammar, grammar_build_set, library
from rust_engine_mcp.aps_grammar_labels import human_label
from rust_engine_mcp.aps_mat_auth_ui import save_hint
from rust_engine_mcp.aps_validator_plain import format_p0_display
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate

from .aps_tooltips import bind_aps_tooltip
from .aps_collapsible import CollapsibleSection
from .aps_inline_feedback import set_inline_status
from .aps_paned import add_pane, horizontal_paned, set_initial_pane_widths
from .aps_scroll import attach_wheel_area
from .aps_theme import (
    COLOR_ACCENT,
    COLOR_MUTED,
    COLOR_PREVIEW_PLACEHOLDER,
    COLOR_TEXT_HINT,
    COLOR_TEXT_SUBTLE,
    FONT_UI,
    FONT_UI_BOLD,
    FONT_MONO,
    FONT_MONO_SMALL,
    FONT_SMALL,
    MIN_WINDOW_SIZE,
    track_wraplength,
    wrap_for_widget,
)
from .aps_tk import themed_listbox
from .aps_onboarding_panel import empty_state_label
from .facility_needs_strip import FacilityNeedsStrip
from .footprint_canvas import FootprintCanvas
from .site_preview_panel import SiteLayoutPreviewSection
from .assembly_onboard_strip import AssemblyOnboardStrip
from .grammar_inspector import GrammarInspectorPanel
from .grammar_iterate_panel import GrammarIteratePanel
from .grammar_dna_panel import GrammarDnaPanel
from .grammar_build_set_panel import GrammarBuildSetPanel
from .generation_trace_strip import GenerationTraceStrip
from .material_browser import MaterialBrowserPanel
from .assembly_preview_panel import AssemblyPreviewPanel
from .slot_preview_panel import SlotPreviewPanel
from .state import ArtDomain, SuiteState

_MATERIAL_AUTHORITY_COPY = (
    "The material you assign here is saved on each piece. The game and the preview both read it "
    "from this Assembly — not from Catalog tags or the Blender viewport. So: assign here, save, "
    "and it shows up everywhere."
)


def _is_dark_color(hex_color: str) -> bool:
    """True if a #rrggbb color is dark enough to need light text on top."""
    try:
        h = hex_color.lstrip("#")
        if len(h) != 6:
            return False
        r, g, b = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)
        return (0.299 * r + 0.587 * g + 0.114 * b) < 140
    except (ValueError, TypeError):
        return False


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
        self._grammar_set_tier = "G0"
        self._build()

    def set_domain(self, lane: str) -> None:
        if not hasattr(self, "_lane_banner"):
            self._lane_banner = ttk.Label(self, text="", font=FONT_UI, foreground=COLOR_MUTED)
            self._lane_banner.pack(anchor=tk.W, before=self.metadata_flow, pady=(0, 4))
        if lane == ArtDomain.LANDSCAPE.value:
            self._lane_banner.configure(text="Landscape lane — grammar preset authority.")
        else:
            self._lane_banner.configure(text="Buildings lane — Assembly authority.")

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

        self._gen_trace = GenerationTraceStrip(
            self,
            self.state,
            get_snapshot=lambda: self._snapshot or self.state.assembly_snapshot_data,
        )
        self._gen_trace.pack(fill=tk.X, pady=(0, 6))

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
        arch_labels, self._archetype_label_to_id = self._grammar_combo_maps(archetypes)
        districts = building_grammar.list_district_styles(archetypes[0]) or ["industrial_west"]
        dist_labels, self._district_label_to_id = self._grammar_combo_maps(districts)

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
        self.engine_path_var = tk.StringVar(value=_MATERIAL_AUTHORITY_COPY)
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
        self.material_browser = MaterialBrowserPanel(
            mat_frame,
            on_apply_material=self._apply_material_profile,
            on_log=self._on_log,
            layout="vertical",
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
        self._validation_lbl = ttk.Label(
            inspector_pane,
            textvariable=self.validation_var,
            wraplength=420,
            font=FONT_UI,
        )
        self._validation_lbl.pack(anchor=tk.W, pady=4)

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
        arch_labels, self._archetype_label_to_id = self._grammar_combo_maps(archetypes)
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

    @staticmethod
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

    @staticmethod
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

    def _set_validation_result(self, text: str, *, ok: bool | None = None) -> None:
        set_inline_status(self._validation_lbl, self.validation_var, text, ok=ok)

    def _show_tag_hint(self, text: str) -> None:
        if hasattr(self, "next_step_var"):
            self.next_step_var.set(text[:220])

    def show_material_assign_callout(self, profile_id: str) -> None:
        if self._snapshot:
            self.next_step_var.set(
                f"Material {profile_id} highlighted — select a footprint cell, then Apply to selected piece."
            )
        else:
            self.next_step_var.set("Generate or load an Assembly first, then select a footprint cell.")
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

    @staticmethod
    def _grammar_combo_maps(ids: list[str]) -> tuple[list[str], dict[str, str]]:
        labels = [human_label(i) for i in ids if i]
        label_to_id = {human_label(i): i for i in ids if i}
        return labels, label_to_id

    def _resolve_archetype_id(self) -> str:
        raw = self.archetype_var.get().strip()
        return self._archetype_label_to_id.get(raw, raw)

    def _resolve_district_id(self) -> str:
        raw = self.district_var.get().strip()
        return self._district_label_to_id.get(raw, raw)

    def _grammar_section_title(self) -> str:
        base = "Grammar inspector"
        if not self._snapshot:
            return base
        arch = human_label(str(self._snapshot.get("archetype_id") or ""))
        if arch and arch not in ("—", ""):
            return f"{base} — {arch}"
        return base

    def _refresh_collapsible_titles(self) -> None:
        tag_title = "Tags (look & state)"
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
            self._snapshot = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                module_id=module_id,
            )
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Module apply failed: {exc}", ok=False)
            return
        self.state.assembly_snapshot_data = self._snapshot
        self._refresh_module_resolve_label(module_id)
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._refresh_facility_needs()
        self.on_placement_select()
        self._on_log(f"module {module_id} → {self._selected_node_id}")
        self._set_validation_result(f"Module {module_id} applied — Save snapshot before bake", ok=None)

    def _on_archetype_change(self, _event=None) -> None:
        archetype = self._resolve_archetype_id()
        if not archetype:
            return
        districts = building_grammar.list_district_styles(archetype)
        labels, self._district_label_to_id = self._grammar_combo_maps(districts or [])
        self.district_combo.configure(values=labels or [""])
        if labels:
            self.district_var.set(labels[0])
        self._refresh_facility_needs()
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
            self.archetype_var.set(human_label(str(snap["archetype_id"])))
            self._on_archetype_change()
            if snap.get("district_style"):
                self.district_var.set(human_label(str(snap["district_style"])))
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

    def _on_grammar_inspector_rule_select(self, layer: str, rule_id: str) -> None:
        count = self.footprint_canvas.highlight_for_rule(rule_id)
        self._on_log(f"grammar-inspector {layer}/{rule_id} → {count} cells highlighted")

    def _refresh_footprint_grid(self) -> None:
        if not self._snapshot:
            self.footprint_canvas.set_cells([], [])
            return
        cells = assembly.footprint_cells_for_snapshot(self._snapshot)
        self.footprint_canvas.clear_rule_highlight()
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
        # APS-UX-PIPELINE-VALIDITY-001 — a freshly generated/loaded snapshot is
        # unvalidated; clear any stale P0 verdict so the pipeline bar shows "saved
        # (P0 not run)" until the gate is actually run.
        self.state.assembly_p0_passed = None
        if hasattr(self, "_gen_trace"):
            self._gen_trace.reset_approval()
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
        self.refresh_generation_trace()

    def refresh_generation_trace(self) -> None:
        if hasattr(self, "_gen_trace"):
            self._gen_trace.refresh()

    def on_generate(self) -> None:
        seed = int(self.seed_var.get())
        tier = self.tier_var.get().strip() or "lod0"
        try:
            if self.use_grammar_var.get():
                archetype = self._resolve_archetype_id()
                district = self._resolve_district_id()
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
        # P7 Slice B — the pipeline spine owns the "what's next" walkthrough; this
        # stays a short in-context hint about the work area, not a second pipeline nav.
        self.next_step_var.set(
            "Select a footprint cell to assign a material (Catalog tags are hints only)."
        )
        bind_aps_tooltip(self._next_step_lbl, "asm_save_reminder")
        rep = self._p0_report()
        if rep.status == "passed":
            self._set_validation_result(
                f"Assembly saved · {self.state.assembly_id} · ship check passed",
                ok=True,
            )
        else:
            hints = self._format_validation_hints(rep)
            self._set_validation_result(f"Ship check failed: {hints[:200]}", ok=False)
            self._on_log(f"generate ship check failed: {hints[:400]}")

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
        rep = validate_assembly_p0_gate(
            self._snapshot,
            snapshot_path=snap_path.replace("\\", "/"),
            ship=True,
        )
        # APS-UX-PIPELINE-VALIDITY-001 — record the live P0 verdict so the pipeline
        # bar can show ✓ only when the gate actually passed for this snapshot.
        self.state.assembly_p0_passed = rep.status == "passed"
        return rep

    @staticmethod
    def _format_validation_hints(rep) -> str:
        return format_p0_display(rep, limit=20)[:1200]

    def _run_p0_or_block(self, action: str) -> bool:
        rep = self._p0_report()
        if rep.status == "passed":
            self._set_validation_result(f"Ship check passed — {action} OK", ok=True)
            return True
        hints = self._format_validation_hints(rep)
        self._set_validation_result(f"Ship check failed: {hints[:200]}", ok=False)
        return messagebox.askyesno(
            f"Ship check failed — {action} anyway?",
            f"{hints}\n\nProceed anyway? (Not recommended before you ship.)",
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
            self._on_log("save cancelled — ship check failed")
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
            self._on_log("preview cancelled — ship check failed")
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
        self._show_validation_report(rep, title="Check schema")

    def on_validate_p0(self) -> None:
        if not self._snapshot:
            self._set_validation_result("No snapshot loaded.", ok=False)
            return
        rep = self._p0_report()
        self._show_validation_report(rep, title="Ship check")

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
        # APS-UX-NONCOLOR — carry identity as text on the swatch so material is
        # distinguishable in grayscale / colorblind, not by color block alone.
        swatch_text = "—"
        if profile_id:
            head = profile_id.split("_")[0]
            swatch_text = head[:3].upper() if head else profile_id[:3].upper()
        fg = "#ffffff" if _is_dark_color(color) else "#111111"
        self._material_swatch.configure(bg=color, text=swatch_text, fg=fg)

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
