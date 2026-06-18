"""OVR-P55-PREVIEW-001 — shared four-state preview contract."""

from __future__ import annotations

import tkinter as tk
from typing import Literal

from PIL import Image, ImageTk

from . import aps_theme
from .aps_theme import FONT_CAPTION, PREVIEW_THUMB_SM

PreviewState = Literal["empty", "loading", "error", "result"]

_FIDELITY_LABELS = {
    "quick": "Quick preview",
    "interactive": "Interactive 3D",
    "ship": "Ship render",
    "layout": "Layout view",
}

_EMPTY_DEFAULT = "Nothing selected"


def preview_surface_state(
    state: PreviewState,
    *,
    detail: str | None = None,
    hint: str | None = None,
) -> tuple[str, str, str]:
    """Return (display_text, fg, bg) for a preview surface."""
    if state == "loading":
        line = detail or "Rendering…"
        return f"⟳ {line}", aps_theme.COLOR_MUTED, aps_theme.COLOR_PANEL_BG
    if state == "empty":
        line = detail or _EMPTY_DEFAULT
        return f"○ {line}", aps_theme.COLOR_MUTED, aps_theme.COLOR_INPUT_BG
    if state == "error":
        reason = detail or "Preview unavailable"
        tail = f" — {hint}" if hint else ""
        return f"◐ {reason}{tail}", aps_theme.COLOR_WARN, aps_theme.COLOR_WARN_BG
    if state == "result":
        line = detail or "Ready"
        return f"✓ {line}", aps_theme.COLOR_PASS, aps_theme.COLOR_INPUT_BG
    return f"○ {_EMPTY_DEFAULT}", aps_theme.COLOR_MUTED, aps_theme.COLOR_INPUT_BG


def fidelity_label(kind: str) -> str:
    return _FIDELITY_LABELS.get(kind, kind)


def configure_preview_label(
    label: tk.Label,
    state: PreviewState,
    *,
    detail: str | None = None,
    hint: str | None = None,
    width: int | None = None,
    height: int | None = None,
) -> None:
    text, fg, bg = preview_surface_state(state, detail=detail, hint=hint)
    thumb = width or PREVIEW_THUMB_SM
    kw: dict = {
        "image": "",
        "text": text,
        "fg": fg,
        "bg": bg,
        "wraplength": max(48, thumb - 12),
        "justify": tk.CENTER,
    }
    if width is not None:
        kw["width"] = max(8, width // 8)
    if height is not None:
        kw["height"] = max(4, height // 16)
    label.configure(**kw)


def apply_preview_photo(
    label: tk.Label,
    photo: ImageTk.PhotoImage,
    *,
    bg: str | None = None,
) -> None:
    label.configure(image=photo, text="", bg=bg or aps_theme.COLOR_INPUT_BG, fg=aps_theme.COLOR_MUTED)


def image_is_near_black(img: Image.Image) -> bool:
    lo, hi = img.convert("L").getextrema()
    return hi - lo < 16 and hi < 24


def make_fidelity_chip(parent: tk.Misc, kind: str) -> tk.Label:
    return tk.Label(
        parent,
        text=fidelity_label(kind),
        font=FONT_CAPTION,
        fg=aps_theme.COLOR_MUTED,
        bg=aps_theme.COLOR_PANEL_BG,
        padx=2,
        pady=0,
    )
