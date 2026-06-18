"""Shared inline status helpers for APS-UX-NONBLOCK-001 / APS-UX-POLISH-001."""

from __future__ import annotations

import tkinter as tk
from typing import Literal

from . import aps_theme
from .state import ArtDomain

StatusState = Literal["pass", "fail", "warn", "pending", "working"]

_STATUS_DEFAULT_WORD: dict[StatusState, str] = {
    "pass": "valid",
    "fail": "blocked",
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
        "pass": aps_theme.COLOR_PASS,
        "fail": aps_theme.COLOR_FAIL,
        "warn": aps_theme.COLOR_WARN,
        "pending": aps_theme.COLOR_MUTED,
        "working": aps_theme.COLOR_ACCENT,
    }
    bg = {
        "pass": aps_theme.COLOR_PASS_BG,
        "fail": aps_theme.COLOR_FAIL_BG,
        "warn": aps_theme.COLOR_WARN_BG,
        "pending": aps_theme.COLOR_INPUT_BG,
        "working": "",
    }
    label = (word or _STATUS_DEFAULT_WORD[state]).strip()
    if detail:
        label = f"{label} — {detail.strip()}"
    return glyphs[state], label, fg[state], bg[state]


def format_status_line(state: StatusState, *, word: str | None = None, detail: str | None = None) -> str:
    glyph, label, _fg, _bg = status_atom(state, word=word, detail=detail)
    return f"{glyph} {label}"


def power_tier_atom(
    tier: str,
    *,
    detail: str | None = None,
) -> tuple[str, str, str, str]:
    """DES-POWER-TIER-001 — bolt glyph + tier word for Facility Needs strip."""
    norm = str(tier or "light").lower()
    glyphs = {
        "light": "⚡",
        "medium": "⚡⚡",
        "heavy": "⚡⚡⚡",
        "grid": "⊞",
    }
    fg = {
        "light": aps_theme.COLOR_MUTED,
        "medium": aps_theme.COLOR_WARN,
        "heavy": aps_theme.COLOR_FAIL,
        "grid": aps_theme.COLOR_ACCENT,
    }
    bg = {
        "light": aps_theme.COLOR_INPUT_BG,
        "medium": aps_theme.COLOR_WARN_BG,
        "heavy": aps_theme.COLOR_FAIL_BG,
        "grid": aps_theme.COLOR_PASS_BG,
    }
    glyph = glyphs.get(norm, "⚡")
    word = f"{norm} power"
    if detail:
        word = f"{word} — {detail.strip()}"
    return glyph, word, fg.get(norm, aps_theme.COLOR_MUTED), bg.get(norm, aps_theme.COLOR_INPUT_BG)


def format_power_tier_line(tier: str, *, detail: str | None = None) -> str:
    glyph, word, _fg, _bg = power_tier_atom(tier, detail=detail)
    return f"{glyph} {word}"


def apply_status_atom(
    label: tk.Widget,
    var: tk.StringVar,
    state: StatusState,
    *,
    word: str | None = None,
    detail: str | None = None,
) -> None:
    """Apply canonical §3.4 status atom to a label + string var."""
    glyph, label_word, fg, bg = status_atom(state, word=word, detail=detail)
    var.set(f"{glyph} {label_word}")
    label.configure(foreground=fg)
    if bg and isinstance(label, tk.Label):
        label.configure(background=bg)


def _detail_from_legacy(text: str) -> str:
    t = text.strip()
    low = t.lower()
    for prefix in (
        "validation: pass — ",
        "validation: fail — ",
        "validation: pass - ",
        "validation: fail - ",
        "validation: pass:",
        "validation: fail:",
    ):
        if low.startswith(prefix):
            return t[len(prefix) :].strip()
    if low.startswith("pass:"):
        return t[5:].strip()
    if low.startswith("fail:"):
        return t[5:].strip()
    if low.startswith("register pass"):
        return t[len("register pass") :].strip().lstrip("—- ").strip()
    return t


def material_texture_status(
    status: str,
) -> tuple[StatusState, str, str]:
    """Map material card texture status → status_atom tuple (glyph, word, fg)."""
    mapping: dict[str, tuple[StatusState, str]] = {
        "ready": ("pass", "ready"),
        "partial": ("warn", "partial"),
        "missing": ("pending", "missing"),
    }
    state, word = mapping.get(status, ("pending", status))
    glyph, label, fg, _bg = status_atom(state, word=word)
    return glyph, label, fg


def format_material_texture_status(status: str, *, profile_id: str | None = None) -> str:
    glyph, label, _fg = material_texture_status(status)
    line = f"{glyph} {label}"
    if profile_id:
        return f"{line} · {profile_id}"
    return line


def validation_foreground(ok: bool | None) -> str:
    if ok is True:
        return aps_theme.COLOR_PASS
    if ok is False:
        return aps_theme.COLOR_FAIL
    return aps_theme.COLOR_WARN if ok is None else aps_theme.COLOR_MUTED


def set_inline_status(
    label: tk.Widget,
    var: tk.StringVar,
    text: str,
    *,
    ok: bool | None = None,
) -> None:
    if ok is None:
        var.set(text)
        _glyph, _word, fg, _bg = status_atom("pending")
        label.configure(foreground=fg)
        return
    state: StatusState = "pass" if ok else "fail"
    detail = _detail_from_legacy(text)
    word = "valid" if ok else "blocked"
    apply_status_atom(label, var, state, word=word, detail=detail or None)


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
