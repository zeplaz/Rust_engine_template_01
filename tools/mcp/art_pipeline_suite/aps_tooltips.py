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
    # Pipeline steps
    "pipeline_catalog": "Done = module selected. Pending = pick a module in Catalog.",
    "pipeline_assembly": "Done = snapshot loaded or generated. Pending = Assembly tab.",
    "pipeline_materials": "Done = placements have material_profile. Pending = assign on Assembly.",
    "pipeline_variants": "Done = variant_set loaded. Pending = Variants tab.",
    "pipeline_atlas": "Done = atlas folder or tile_batch set. Pending = Atlas tab.",
    "pipeline_step": "Valid = step passed its check. Saved = data exists, not yet validated. Pending = not started.",
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


def bind_aps_tooltip(widget: tk.Misc, key: str) -> None:
    text = _wrap_tooltip_text(_tooltip_text(key))

    def _enter(_event: tk.Event) -> None:
        widget.tooltip = _Tooltip(widget, text)

    def _leave(_event: tk.Event) -> None:
        tip = getattr(widget, "tooltip", None)
        if tip is not None:
            tip.destroy()
            widget.tooltip = None

    widget.bind("<Enter>", _enter, add="+")
    widget.bind("<Leave>", _leave, add="+")


def bind_many(pairs: list[tuple[Any, str]]) -> None:
    for widget, key in pairs:
        if widget is not None:
            bind_aps_tooltip(widget, key)


class _Tooltip:
    def __init__(self, widget: tk.Misc, text: str) -> None:
        self._top = tk.Toplevel(widget)
        self._top.wm_overrideredirect(True)
        self._top.wm_attributes("-topmost", True)
        lbl = ttk.Label(self._top, text=text, relief=tk.SOLID, padding=4, font=("Segoe UI", 9))
        lbl.pack()
        self._top.update_idletasks()
        x = widget.winfo_rootx() + 8
        y = widget.winfo_rooty() + widget.winfo_height() + 4
        self._top.wm_geometry(f"+{x}+{y}")

    def destroy(self) -> None:
        self._top.destroy()
