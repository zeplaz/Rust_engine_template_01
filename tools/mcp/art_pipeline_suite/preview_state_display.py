"""APSR-P2 — shared preview validity, fidelity chips, thumbnail surfaces, status colors.

Single presentation module for assembly / atlas / variants preview panels.
``aps_preview_state`` re-exports this module for backward compatibility.
"""

from __future__ import annotations

import tkinter as tk
from typing import Callable, Literal

from PIL import Image, ImageTk

from rust_engine_mcp.paths import repo_root

from . import aps_theme
from .aps_inline_feedback import set_inline_status
from .aps_preview_variant_state import VARIANT_STATES, variant_state_label
from .aps_theme import FONT_CAPTION, FONT_SMALL, PREVIEW_THUMB_SM

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


def set_preview_status(
    status_label: tk.Misc,
    status_var: tk.StringVar,
    text: str,
    *,
    ok: bool | None = None,
) -> None:
    set_inline_status(status_label, status_var, text, ok=ok)


def build_variant_state_chip_row(parent: tk.Misc) -> tk.Frame:
    """Static variant-state legend row (assembly preview)."""
    row = tk.Frame(parent)
    for state in VARIANT_STATES:
        tk.Label(row, text=variant_state_label(state), font=FONT_SMALL).pack(side=tk.LEFT, padx=2)
    return row


def load_png_thumbnail(
    label: tk.Label,
    png_rel: str,
    *,
    width: int,
    height: int,
    on_log: Callable[[str], None] | None = None,
    empty_detail: str = "No thumbnail",
    error_detail: str = "Thumbnail unavailable",
    near_black_detail: str = "Preview blank",
    near_black_hint: str | None = "use Open in browser",
) -> ImageTk.PhotoImage | None:
    """Load repo-relative PNG into label; returns PhotoImage when shown."""
    log = on_log or (lambda _m: None)
    if not png_rel:
        configure_preview_label(label, "empty", detail=empty_detail, width=width, height=height)
        return None
    path = repo_root() / str(png_rel).replace("\\", "/")
    if not path.is_file():
        log(f"preview thumb missing {path.name}")
        configure_preview_label(
            label,
            "error",
            detail=error_detail,
            hint="use Open in browser",
            width=width,
            height=height,
        )
        return None
    try:
        img = Image.open(path).convert("RGB")
    except Exception as exc:  # noqa: BLE001
        log(f"preview thumb unreadable {path.name}: {exc}")
        configure_preview_label(
            label,
            "error",
            detail="Thumbnail unreadable",
            hint="use Open in browser",
            width=width,
            height=height,
        )
        return None
    if image_is_near_black(img):
        configure_preview_label(
            label,
            "error" if near_black_hint else "empty",
            detail=near_black_detail,
            hint=near_black_hint,
            width=width,
            height=height,
        )
        log(f"preview thumb blank/black {path.name}")
        return None
    img.thumbnail((width, height), Image.Resampling.LANCZOS)
    photo = ImageTk.PhotoImage(img)
    apply_preview_photo(label, photo)
    return photo


def show_image_file_thumbnail(
    label: tk.Label,
    path,
    *,
    max_size: int,
    on_log: Callable[[str], None] | None = None,
    photos_cache: dict[str, ImageTk.PhotoImage] | None = None,
    cache_key: str | None = None,
) -> ImageTk.PhotoImage | None:
    """Load absolute PNG path into label (atlas cell strip)."""
    from pathlib import Path

    log = on_log or (lambda _m: None)
    p = Path(path)
    try:
        img = Image.open(p).convert("RGB")
        if image_is_near_black(img):
            configure_preview_label(
                label,
                "error",
                detail="Cell image blank",
                hint=p.name[:16],
                width=max_size,
                height=max_size,
            )
            return None
        img.thumbnail((max_size, max_size), Image.Resampling.LANCZOS)
        photo = ImageTk.PhotoImage(img)
        if photos_cache is not None and cache_key:
            photos_cache[cache_key] = photo
        apply_preview_photo(label, photo)
        return photo
    except Exception as exc:  # noqa: BLE001
        configure_preview_label(
            label,
            "error",
            detail="Thumb failed",
            hint=p.name[:16],
            width=max_size,
            height=max_size,
        )
        log(f"thumb fail {p.name}: {exc}")
        return None
