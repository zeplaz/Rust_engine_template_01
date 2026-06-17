"""APS-UX-TOOLTIPS-002 — designer-reviewed tooltip dictionary for Art Pipeline Suite."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any

TOOLTIPS: dict[str, str] = {
    # Flow bar
    "flow_send_assembly": "Send catalog style selection to Assembly tab for snapshot generation.",
    "flow_bake_variants": "Expand variant_set → tile_batch and jump to Atlas (needs assembly + variants).",
    "flow_pack_atlas": "Pack PNG folder into ship atlas meta (set folder on Atlas tab first).",
    # Tabs
    "tab_catalog": "Browse module kit GLBs, sidecar metadata, and validation.",
    "tab_assembly": "Generate or load assembly_snapshot JSON — ship authority for placements.",
    "tab_materials": "Browse material profiles and preview textures — assign on Assembly only.",
    "tab_variants": "variant_set_v1 layers and tags — declarative tile-state expansion.",
    "tab_atlas": "tile_batch run, PNG folder pack, and atlas QC preview.",
    "tab_presets": "Browse landscape grammar presets — land_dna + topology_graph authority.",
    "tab_grammar": "Topology graph workspace — not building footprint.",
    "tab_states": "Succession + burn + regrowth axes — vegetation_variant_catalog entries.",
    # Landscape States (DES-APS-STATE-AXIS-LABELS-001)
    "state_succession_axis": "Succession stages in catalog.axes — long-term cover ladder (Grass→OldGrowth).",
    "state_regrowth_axis": "Regrowth macro phases — transient post-disturbance window (Scar→Mature).",
    "state_burn_frames": "Burn frame loop veg_burn_00–07 — matches engine VEG_BURN_FRAME_COUNT (default 8).",
    "veg_extract_authority": "Read-only: VegetationExtractFrame variant_key → LG-5 stamp. APS does not write extract.",
    "state_bake": "Expand catalog entries to tile_batch variants — preset JSON remains authority.",
    "state_catalog_validate": "Validate against vegetation_variant_catalog_v1 before bake.",
    # Pipeline pills (DES-APS-PIPELINE-PILLS-001)
    "pipeline_catalog": "Catalog — module selected and GLB validate PASS.",
    "pipeline_assembly": "Assembly — valid only after QC/P0 gate; saved (QC not run) = snapshot on disk only.",
    "pipeline_materials": "Materials — every placement has material_profile.",
    "pipeline_variants": "Variants — variant_set valid and ready for tile batch.",
    "pipeline_atlas": "Atlas — PNG folder packed; valid = QC PASS.",
    "pipeline_presets": "Presets — landscape preset selected; schema validate PASS.",
    "pipeline_grammar": "Grammar — topology_graph saved on preset JSON.",
    "pipeline_states": "States — succession and disturbance variant rows ready.",
    "pipeline_stamp": "Stamp — atlas registered for map stamp; engine resolves UV lookup.",
    "pipeline_step": "Valid = gate passed. Saved (QC not run) = data on disk only. Pending = not started.",
    # Catalog
    "cat_batch_filter": "Filter module list by production batch id.",
    "cat_category_filter": "Filter modules by kit category.",
    "cat_refresh": "Reload module index from disk.",
    "cat_sidecar_truth": "Sidecar tags are hints only — assembly snapshot semantic_tags win at ship.",
    "cat_validate": "Validate selected module sidecar against schema.",
    "cat_metadata": "Editable sidecar fields — not ship authority.",
    "cat_save_metadata": "Write sidecar JSON next to module GLB.",
    "cat_reindex": "Rebuild module index after batch changes.",
    "cat_browser_preview": "Open isolated GLB in system browser / preview worker.",
    "cat_trimesh": "Quick ortho thumb via trimesh (optional dependency).",
    "cat_list_thumb": "Select module — thumb shows isolated GLB when indexed.",
    # Assembly
    "asm_engine_path": "Engine reads assembly_snapshot from this path at runtime.",
    "asm_grammar": "Building grammar preset drives procedural module placement.",
    "asm_archetype": "Archetype id (warehouse, rowhouse, …) for grammar rules.",
    "asm_district": "District style influences facade and massing tags.",
    "asm_style_pack": "Style pack links catalog modules to grammar.",
    "asm_tier": "LOD / production tier for module kit selection.",
    "asm_footprint_dims": "Footprint width × depth and floor count for placement grid.",
    "asm_generate": "Run grammar pipeline — writes assembly_snapshot JSON.",
    "asm_material_lib": "Assign material_profile per placement — snapshot is ship truth.",
    "asm_load": "Load assembly_snapshot JSON from disk.",
    "asm_save": "Save current snapshot — validate before ship.",
    "asm_validate": "P0 validator — missing GLBs, materials, schema.",
    "asm_p0": "Plain-language P0 issues for artists (read-only).",
    "asm_preview": "Spawn assembly preview (Bevy worker or browser fallback).",
    "asm_footprint": "Footprint grid — click cell to inspect placement.",
    "asm_footprint_heatmap": "Colors show role density on grid — not ship status.",
    "asm_slot_preview": "Module + material + combined thumbs for selected cell.",
    "asm_save_reminder": "Save snapshot after material edits so runtime picks up authority.",
    "asm_iterate": "Change one grammar layer without a full seed reroll. Re-rolls only that layer.",
    "asm_preview_thumb": "Assembly PNG preview. Blank? Wait for spinner, then Retry.",
    "asm_grammar_dna": "Advanced massing pressure (yard / module density). Leave default unless tuning grammar.",
    "asm_grammar_dna_enable": "Turn on the advanced massing-pressure controls when generating. Off by default.",
    "asm_grammar_dna_preset": "Pick a massing-bias preset (how dense / spread out the building is).",
    # Atlas
    "atl_batch_json": "tile_batch_v1 JSON path for ortho/keyframe bake.",
    "atl_batch_run": "Run tile batch job (Blender/keyframe pipeline).",
    "atl_folder": "Folder of PNG tiles to pack into atlas meta.",
    "atl_keyframe_rename": "Rename keyframe PNGs to pack-friendly names before tilemapgen.",
    "atl_pack": "Pack folder → staging atlas meta + cells.",
    "atl_preview": "Refresh atlas UV grid and cell strip preview.",
    "atl_validate": "Validate atlas meta against production schema.",
    "atl_open_folder": "Open atlas output folder in file explorer.",
    "atl_uv_grid": "UV layout preview — check seam and cell bounds.",
    "atl_cell_strip": "Per-cell thumb strip from packed atlas.",
    "atl_lod0": "CI/smoke ortho batch — not ship art. Use only for engine smoke tests.",
    "atl_batch": "Select tile_batch manifest for LOD0 smoke runs.",
    # Materials
    "mat_preview_modes": "Sphere, wall strip, and building-section material previews.",
    "mat_status": "Texture readiness for selected profile.",
    "mat_add_profile": "Register a new material profile in the catalog.",
    "mat_generate": "Generate PBR textures for selected profile.",
    "mat_generate_all": "Generate all profiles missing textures (background job).",
    "mat_open_folder": "Open profile texture folder on disk.",
    "mat_open_registry": "Open material_profiles_v1.json in editor.",
    "mat_use_in_assembly": "Jump to Assembly and highlight this profile for assign.",
    "mat_search": "Filter profiles by id substring.",
    "mat_category": "Filter profiles by category.",
    "mat_category_tree": "Nested category tree — industrial, residential, roof, …",
    "mat_apply": "Apply selected profile to Assembly placement (via callback).",
    "mat_reload_preview": "Refresh preview thumbs after texture generate.",
    # Metadata / misc
    "meta_flow": "Show how metadata flows Catalog → Assembly → Variants → Atlas.",
}


def _tooltip_text(key: str) -> str:
    if key in TOOLTIPS:
        return TOOLTIPS[key]
    if key.startswith("asm_grammar_dna_"):
        return "Massing-pressure control — affects how dense / spread out the building is."
    if key.startswith("asm_grammar_beta_"):
        return "Fine massing-pressure adjustment — advanced grammar tuning."
    return f"({key})"


def _wrap_tooltip_text(text: str, width: int = 72) -> str:
    """Soft-wrap a tooltip string to a readable width (no external deps)."""
    import textwrap

    return "\n".join(
        "\n".join(textwrap.wrap(line, width=width)) if line.strip() else line
        for line in text.splitlines()
    ) or text


# Show tooltip only after a short hover, and never leave one floating: a single
# tooltip window is reused/destroyed across every widget so a stale tip can't
# survive a tab change or a fast mouse move (APS-UX-TOOLTIPS / B3).
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
        # Only show if the pointer is still over this widget — guards against a
        # scheduled show firing after a fast move / tab change.
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
    # Any click, mousewheel, or the widget leaving the screen must drop the tip.
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
