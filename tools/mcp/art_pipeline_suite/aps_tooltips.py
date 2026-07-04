"""Designer-reviewed tooltip dictionary for Art Pipeline Suite."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any

TOOLTIPS: dict[str, str] = {
    # Flow bar
    "flow_send_assembly": "Send catalog style selection to the Assembly step for generation.",
    "flow_bake_variants": (
        "Turn your variant set into a tile job and open the Atlas step (needs an Assembly + a variant set)."
    ),
    "flow_pack_atlas": "Pack PNG folder into the ship atlas (set folder on Atlas step first).",
    # Tabs
    "tab_catalog": "Browse module kit models, sidecar metadata, and validation.",
    "tab_assembly": "Generate or load the Assembly — ship authority for materials and tags.",
    "tab_materials": "Browse materials and preview textures — assign on Assembly only.",
    "tab_variants": "Variant layers and tags — declarative tile-state expansion.",
    "tab_atlas": "Run tile jobs, pack PNG folders, and preview atlas QC.",
    "tab_presets": "Browse landscape presets — layout and disturbance authority.",
    "tab_grammar": "Edit the landscape layout graph (roads, rings, patches). Not the building footprint grid.",
    "tab_states": "Vegetation states over time — growth stages, fire, and regrowth.",
    # Landscape States
    "state_succession_axis": "Growth stages in the catalog — long-term cover ladder (Grass to Old Growth).",
    "state_regrowth_axis": "Regrowth phases — transient post-disturbance window (Scar to Mature).",
    "state_burn_frames": "Burn frame loop — matches engine burn frame count (default 8).",
    "veg_extract_authority": (
        "Read-only: engine maps vegetation variant keys to landscape atlas tiles. APS does not write extract."
    ),
    "state_bake": "Expand catalog entries to tile variants — preset JSON remains authority.",
    "state_catalog_validate": "Validate against the vegetation catalog before bake.",
    # Pipeline pills
    "pipeline_catalog": "Catalog — module selected and model validate pass.",
    "pipeline_assembly": "Assembly — valid only after ship check; saved (not checked) = file on disk only.",
    "pipeline_materials": "Materials — every piece has a material assigned.",
    "pipeline_variants": "Variants — variant set valid and ready for tile batch.",
    "pipeline_atlas": "Atlas — PNG folder packed; valid = QC pass.",
    "pipeline_presets": "Presets — landscape preset selected; schema validate pass.",
    "pipeline_grammar": "Grammar — layout graph saved on preset JSON.",
    "pipeline_states": "States — growth and disturbance variant rows ready.",
    "pipeline_stamp": "Stamp — atlas registered for map stamp; engine resolves UV lookup.",
    "pipeline_step": "Valid = check passed. Saved (not checked) = data on disk only. Pending = not started.",
    # Catalog
    "cat_batch_filter": "Filter module list by production batch id.",
    "cat_category_filter": "Filter modules by kit category.",
    "cat_refresh": "Reload module index from disk.",
    "cat_sidecar_truth": "Sidecar tags are hints only — Assembly tags and materials win at ship.",
    "cat_validate": "Validate selected module sidecar against schema.",
    "cat_metadata": "Editable sidecar fields — not ship authority.",
    "cat_save_metadata": "Write sidecar JSON next to module model.",
    "cat_reindex": "Rebuild module index after batch changes.",
    "cat_browser_preview": "Open isolated model in system browser / preview worker.",
    "cat_trimesh": "Quick ortho thumb (optional 3D preview dependency).",
    "cat_list_thumb": "Select module — thumb shows isolated model when indexed.",
    # Assembly
    "asm_engine_path": "Engine reads the Assembly you save here at runtime.",
    "asm_grammar": "Building style preset drives procedural module placement.",
    "asm_archetype": "Archetype id (warehouse, rowhouse, …) for grammar rules.",
    "asm_district": "District style influences facade and massing tags.",
    "asm_style_pack": "Style pack links catalog modules to grammar.",
    "asm_tier": "Detail / production tier for module kit selection.",
    "asm_footprint_dims": "Footprint width × depth and floor count for placement grid.",
    "asm_generate": "Run grammar pipeline — writes Assembly JSON.",
    "asm_material_lib": "Assign material per piece — Assembly is ship authority.",
    "asm_load": "Load Assembly JSON from disk.",
    "asm_save": "Save current Assembly — run ship check before bake.",
    "asm_validate": "Check schema — missing models, materials, schema.",
    "asm_p0": "Plain-language ship issues for artists (read-only).",
    "asm_preview": "Spawn assembly preview (worker or browser fallback).",
    "asm_footprint": "Footprint grid — click cell to inspect piece.",
    "asm_footprint_heatmap": "Colors show role density on grid — not ship status.",
    "asm_slot_preview": "Module + material + combined thumbs for selected cell.",
    "asm_save_reminder": "Save Assembly after material edits so runtime picks up authority.",
    "asm_iterate": "Change one grammar layer without a full seed reroll. Re-rolls only that layer.",
    "asm_preview_thumb": "Assembly PNG preview. Blank? Wait for spinner, then Retry.",
    "asm_grammar_dna": "Advanced building shape bias (yard / module density). Leave default unless tuning style.",
    "asm_grammar_dna_enable": "Turn on advanced shape controls when generating. Off by default.",
    "asm_grammar_dna_preset": "Pick a shape-bias preset (how dense / spread out the building is).",
    # Atlas
    "atl_batch_json": "Tile job file path for ortho/keyframe bake.",
    "atl_batch_run": "Run tile batch job (Blender/keyframe pipeline).",
    "atl_folder": "Folder of PNG tiles to pack into atlas meta.",
    "atl_keyframe_rename": "Rename keyframe PNGs to pack-friendly names before packing.",
    "atl_pack": "Pack folder → staging atlas meta + cells.",
    "atl_preview": "Refresh atlas UV grid and cell strip preview.",
    "atl_validate": "Validate atlas meta against production schema.",
    "atl_open_folder": "Open atlas output folder in file explorer.",
    "atl_uv_grid": "UV layout preview — check seam and cell bounds.",
    "atl_cell_strip": "Per-cell thumb strip from packed atlas.",
    "atl_lod0": "Smoke ortho batch — not ship art. Use only for engine smoke tests.",
    "atl_batch": "Select tile job manifest for smoke-test runs.",
    # Materials
    "mat_preview_modes": "Sphere, wall strip, and building-section material previews.",
    "mat_status": "Texture readiness for selected material.",
    "mat_add_profile": "Register a new material in the catalog.",
    "mat_generate": "Generate PBR textures for selected material.",
    "mat_generate_all": "Generate all materials missing textures (background job).",
    "mat_open_folder": "Open material texture folder on disk.",
    "mat_open_registry": "Open material registry JSON in editor.",
    "mat_use_in_assembly": "Jump to Assembly and highlight this material for assign.",
    "mat_search": "Filter materials by id substring.",
    "mat_category": "Filter materials by category.",
    "mat_category_tree": "Nested category tree — industrial, residential, roof, …",
    "mat_apply": "Apply selected material to Assembly piece (via callback).",
    "mat_reload_preview": "Refresh preview thumbs after texture generate.",
    # Variants — DES-APS-VARIANTS-LIVE-PREVIEW-001
    "var_apply_layers": (
        "Save layer dropdowns onto the selected variant row. Preview updates live while you "
        "edit; Apply commits the row before Save / tile batch."
    ),
    "var_lighting": "Day · night_off · night_on — drives tile lighting layer and Night preview chip.",
    "var_power": "Grid story for reaction sessions — off · partial · on.",
    "var_damage": "clean · dirty · damaged · ruined — wear read for tile stills.",
    "var_fill": "Occupancy overlay for sim tiles — not a geometry swap.",
    "var_draft_preview": "Preview shows your current controls. Apply layers commits them to the variant row.",
    "var_layers": "Lighting, damage, fill, and tags become variant_key data. Apply, then Save.",
    "var_reaction_filter": "Filter sessions by reaction event — shows suggested tag anchors when selected.",
    "gen_trace_approve": (
        "Artist sign-off that this assembly snapshot is the parent for variant rows and tile bake."
    ),
    "gen_trace_edit_assembly": "Switch to Assembly tab to change archetype, district, seed, or regenerate.",
    # Metadata / misc
    "meta_flow": "Show how metadata flows Catalog → Assembly → Variants → Atlas.",
    "onboard_dismiss": "Hide the welcome card — reopen from Help → getting started.",
    "onboard_steps": "Expand the five-step pipeline guide for this tab.",
    "job_cancel": "Cancel the running background job.",
    "grammar_set_refresh": "Reload building style set brief from disk.",
    "grammar_eval_sweep": "Run grammar evaluation sweep across seeds.",
    "landscape_validate_schema": "Validate landscape grammar schema for the selected preset.",
    "landscape_preset_refresh": "Reload landscape preset list.",
    "landscape_preset_validate": "Validate selected landscape preset JSON.",
    "landscape_preset_pick": "Select this landscape preset for Grammar and States tabs.",
    "preview_open_url": "Open assembly preview URL in the system browser.",
    "preview_copy_url": "Copy assembly preview URL to clipboard.",
}


def _tooltip_text(key: str) -> str:
    if key in TOOLTIPS:
        return TOOLTIPS[key]
    if key.startswith("var_mandate_tag:"):
        tag_id = key.split(":", 1)[1]
        try:
            from rust_engine_mcp.aps_tag_vocabulary import mandate_tag_hint

            return mandate_tag_hint(tag_id)
        except ImportError:
            return tag_id
    if key.startswith("asm_semantic_tag:"):
        tag_id = key.split(":", 1)[1]
        try:
            from rust_engine_mcp.aps_tag_vocabulary import semantic_tag_hint

            return semantic_tag_hint(tag_id)
        except ImportError:
            return tag_id
    if key.startswith("asm_variant_tag:"):
        tag_id = key.split(":", 1)[1]
        try:
            from rust_engine_mcp.aps_tag_vocabulary import assembly_variant_tag_hint

            return assembly_variant_tag_hint(tag_id)
        except ImportError:
            return tag_id
    if key.startswith("asm_grammar_dna_"):
        return "Shape control — affects how dense / spread out the building is."
    if key.startswith("asm_grammar_beta_"):
        return "Fine shape adjustment — advanced grammar tuning."
    return f"({key})"


def _wrap_tooltip_text(text: str, width: int = 72) -> str:
    """Soft-wrap a tooltip string to a readable width (no external deps)."""
    import textwrap

    return "\n".join(
        "\n".join(textwrap.wrap(line, width=width)) if line.strip() else line
        for line in text.splitlines()
    ) or text


_HOVER_DELAY_MS = 450


def bind_aps_tooltip(widget: tk.Misc, key: str) -> None:
    text = _wrap_tooltip_text(_tooltip_text(key))
    state: dict[str, Any] = {"after_id": None}

    def _cancel_pending() -> None:
        after_id = state.get("after_id")
        if after_id is not None:
            try:
                widget.after_cancel(after_id)
            except (tk.TclError, ValueError):
                pass
            state["after_id"] = None

    def _show() -> None:
        state["after_id"] = None
        try:
            under = widget.winfo_containing(
                widget.winfo_pointerx(), widget.winfo_pointery()
            )
        except tk.TclError:
            return
        if under is not widget and not _is_descendant(widget, under):
            return
        _Tooltip.show_for(widget, text)

    def _enter(_event: tk.Event) -> None:
        _cancel_pending()
        try:
            state["after_id"] = widget.after(_HOVER_DELAY_MS, _show)
        except tk.TclError:
            state["after_id"] = None

    def _hide(_event: tk.Event | None = None) -> None:
        _cancel_pending()
        _Tooltip.hide()

    widget.bind("<Enter>", _enter, add="+")
    widget.bind("<Leave>", _hide, add="+")
    widget.bind("<ButtonPress>", _hide, add="+")
    widget.bind("<MouseWheel>", _hide, add="+")
    widget.bind("<Unmap>", _hide, add="+")
    widget.bind("<Destroy>", _hide, add="+")


def _is_descendant(ancestor: tk.Misc, widget: tk.Misc | None) -> bool:
    node = widget
    while node is not None:
        if node is ancestor:
            return True
        node = getattr(node, "master", None)
    return False


def bind_many(pairs: list[tuple[Any, str]]) -> None:
    for widget, key in pairs:
        if widget is not None:
            bind_aps_tooltip(widget, key)


def hide_all_tooltips(_event: tk.Event | None = None) -> None:
    """Public hook — call on tab change / focus loss to drop any floating tip."""
    _Tooltip.hide()


class _Tooltip:
    """Single shared tooltip window — at most one is visible at any time."""

    _current: _Tooltip | None = None

    def __init__(self, widget: tk.Misc, text: str) -> None:
        self._top = tk.Toplevel(widget)
        self._top.wm_overrideredirect(True)
        try:
            self._top.wm_attributes("-topmost", True)
        except tk.TclError:
            pass
        lbl = ttk.Label(self._top, text=text, relief=tk.SOLID, padding=4, font=("Segoe UI", 9))
        lbl.pack()
        self._top.update_idletasks()
        try:
            x = widget.winfo_rootx() + 8
            y = widget.winfo_rooty() + widget.winfo_height() + 4
        except tk.TclError:
            x, y = 0, 0
        self._top.wm_geometry(f"+{x}+{y}")

    @classmethod
    def show_for(cls, widget: tk.Misc, text: str) -> None:
        cls.hide()
        try:
            cls._current = cls(widget, text)
        except tk.TclError:
            cls._current = None

    @classmethod
    def hide(cls) -> None:
        tip = cls._current
        cls._current = None
        if tip is not None:
            tip.destroy()

    def destroy(self) -> None:
        try:
            self._top.destroy()
        except tk.TclError:
            pass
