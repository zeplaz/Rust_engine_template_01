"""Shared inline status helpers for APS-UX-NONBLOCK-001 / APS-UX-POLISH-001."""

from __future__ import annotations

import tkinter as tk
from typing import Literal

from .aps_theme import (
    COLOR_ACCENT,
    COLOR_FAIL,
    COLOR_FAIL_BG,
    COLOR_INPUT_BG,
    COLOR_MUTED,
    COLOR_PASS,
    COLOR_PASS_BG,
    COLOR_WARN,
    COLOR_WARN_BG,
)
from .state import ArtDomain

StatusState = Literal["pass", "fail", "warn", "pending", "working"]

_STATUS_DEFAULT_WORD: dict[StatusState, str] = {
    "pass": "valid",
    "fail": "FAIL",
    "warn": "partial",
    "pending": "pending",
    "working": "working",
}


def status_atom(
    state: StatusState,
    *,
    word: str | None = None,
    detail: str | None = None,
) -> tuple[str, str, str, str]:
    """§3.4 canonical status atom — (glyph, word, fg, bg)."""
    glyphs = {
        "pass": "✓",
        "fail": "✗",
        "warn": "◐",
        "pending": "○",
        "working": "⟳",
    }
    fg = {
        "pass": COLOR_PASS,
        "fail": COLOR_FAIL,
        "warn": COLOR_WARN,
        "pending": COLOR_MUTED,
        "working": COLOR_ACCENT,
    }
    bg = {
        "pass": COLOR_PASS_BG,
        "fail": COLOR_FAIL_BG,
        "warn": COLOR_WARN_BG,
        "pending": COLOR_INPUT_BG,
        "working": "",
    }
    label = (word or _STATUS_DEFAULT_WORD[state]).strip()
    if detail:
        label = f"{label} — {detail.strip()}"
    return glyphs[state], label, fg[state], bg[state]


def format_status_line(state: StatusState, *, word: str | None = None, detail: str | None = None) -> str:
    glyph, label, _fg, _bg = status_atom(state, word=word, detail=detail)
    return f"{glyph} {label}"


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
            return "Switch to Landscape lane before Pack landscape atlas."
        if not getattr(state, "atlas_folder", None) and not getattr(state, "tile_batch_path", None):
            return "Bake states or set atlas folder on Landscape Atlas before Pack landscape atlas."
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
            return "Pack atlas is a Buildings-lane verb — use Pack landscape atlas in Landscape lane."
        if not getattr(state, "atlas_folder", None) and not getattr(state, "tile_batch_path", None):
            return "Prepare a tile batch or set an atlas PNG folder on Atlas before Pack atlas."
        return None

    return None
