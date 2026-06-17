"""Shared inline status helpers for APS-UX-NONBLOCK-001 / APS-UX-POLISH-001."""

from __future__ import annotations

import tkinter as tk

from .aps_theme import COLOR_ACCENT, COLOR_FAIL, COLOR_MUTED, COLOR_PASS, COLOR_WARN
from .state import ArtDomain


def validation_foreground(ok: bool | None) -> str:
    if ok is True:
        return COLOR_PASS
    if ok is False:
        return COLOR_FAIL
    return COLOR_WARN if ok is None else COLOR_MUTED


def set_inline_status(
    label: tk.Label | tk.Widget,
    var: tk.StringVar,
    text: str,
    *,
    ok: bool | None = None,
) -> None:
    var.set(text)
    label.configure(foreground=validation_foreground(ok))


def flow_prerequisite_message(action: str, state) -> str | None:
    """MCP-APS-INLINE-FEEDBACK-002 — lane-scoped flow-bar prerequisites."""
    action_key = action.strip().lower().replace(" ", "_")
    domain = str(getattr(state, "art_domain", ArtDomain.BUILDINGS.value) or ArtDomain.BUILDINGS.value)

    if action_key == "generate_grammar":
        if domain != ArtDomain.LANDSCAPE.value:
            return "Switch to Landscape lane before Generate grammar."
        if not getattr(state, "selected_landscape_preset_id", None):
            return "Select a landscape preset on Presets before Generate grammar."
        if getattr(state, "landscape_preset_validate_ok", None) is False:
            return "Fix landscape_grammar validator errors on Presets before Generate grammar."
        return None

    if action_key == "bake_states":
        if domain != ArtDomain.LANDSCAPE.value:
            return "Switch to Landscape lane before Bake states."
        if not getattr(state, "landscape_grammar_saved", False) and not getattr(
            state, "landscape_preset_validate_ok", None
        ):
            return "Generate grammar on Grammar tab before Bake states."
        return None

    if action_key == "pack_lg5_atlas":
        if domain != ArtDomain.LANDSCAPE.value:
            return "Switch to Landscape lane before Pack LG-5 atlas."
        if not getattr(state, "atlas_folder", None) and not getattr(state, "tile_batch_path", None):
            return "Bake states or set atlas folder on Landscape Atlas before Pack LG-5 atlas."
        return None

    if action_key in ("send_to_assembly",):
        if domain == ArtDomain.LANDSCAPE.value:
            return "Send to Assembly is a Buildings-lane verb — switch lane or use Generate grammar."
        if not (getattr(state, "selected_module_id", None) or getattr(state, "selected_module_ids", None)):
            return "Select a module in Catalog before Send to Assembly."
        return None

    if action_key == "bake_variants":
        if domain == ArtDomain.LANDSCAPE.value:
            return "Bake variants is a Buildings-lane verb — use Bake states in Landscape lane."
        if not getattr(state, "assembly_snapshot_data", None) and not getattr(state, "assembly_snapshot_path", None):
            return "Generate or load an Assembly snapshot before Bake variants."
        if not getattr(state, "variant_set_data", None) and not getattr(state, "variant_set_path", None):
            return "Create a variant set on the Variants tab before Bake variants."
        return None

    if action_key == "pack_atlas":
        if domain == ArtDomain.LANDSCAPE.value:
            return "Pack atlas is a Buildings-lane verb — use Pack LG-5 atlas in Landscape lane."
        if not getattr(state, "atlas_folder", None) and not getattr(state, "tile_batch_path", None):
            return "Prepare a tile batch or set an atlas PNG folder on Atlas before Pack atlas."
        return None

    return None
