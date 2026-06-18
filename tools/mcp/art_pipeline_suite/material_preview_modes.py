"""APS-MAT-002 — Tk preview mode strip for Materials tab."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from PIL import Image, ImageTk

from rust_engine_mcp.material_studio_preview import (
    render_building_section_preview,
    render_sphere_preview,
    render_wall_strip_preview,
)

from .aps_preview_state import apply_preview_photo, configure_preview_label, image_is_near_black, make_fidelity_chip
from .aps_theme import COLOR_MUTED, FONT_SMALL, PREVIEW_THUMB_MD
from rust_engine_mcp.material_profiles import MaterialProfileEntry, ensure_profile_textures


class MaterialPreviewModesPanel(ttk.LabelFrame):
    def __init__(self, master: tk.Misc, *, on_log=None) -> None:
        super().__init__(master, text="Material preview", padding=6)
        self._on_log = on_log or (lambda _s: None)
        self._photos: dict[str, ImageTk.PhotoImage] = {}
        self._entry: MaterialProfileEntry | None = None
        self._build()

    def _build(self) -> None:
        self._mode_var = tk.StringVar(value="sphere")
        row = ttk.Frame(self)
        row.pack(fill=tk.X, pady=(0, 6))
        for mode, label in (
            ("sphere", "Sphere"),
            ("wall_strip", "Wall"),
            ("building_section", "Building section"),
        ):
            ttk.Radiobutton(
                row,
                text=label,
                value=mode,
                variable=self._mode_var,
                command=self._refresh_mode,
            ).pack(side=tk.LEFT, padx=4)
        ttk.Button(row, text="Refresh", command=self._refresh_mode).pack(side=tk.LEFT, padx=8)
        make_fidelity_chip(self, "quick").pack(anchor=tk.W)
        self._canvas = tk.Label(self, relief=tk.SUNKEN)
        self._canvas.pack(fill=tk.BOTH, expand=True)
        configure_preview_label(
            self._canvas,
            "empty",
            detail="Select a material",
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )
        self._hint_var = tk.StringVar(value="")
        ttk.Label(self, textvariable=self._hint_var, font=FONT_SMALL, foreground=COLOR_MUTED).pack(anchor=tk.W)

    def set_profile(self, profile_id: str | None) -> None:
        if not profile_id:
            self._entry = None
            configure_preview_label(
                self._canvas,
                "empty",
                detail="Select a material",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._hint_var.set("")
            return
        configure_preview_label(
            self._canvas,
            "loading",
            detail="Generating preview…",
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )
        try:
            self._entry = ensure_profile_textures(profile_id, size=512)
        except Exception as exc:  # noqa: BLE001
            configure_preview_label(
                self._canvas,
                "error",
                detail="Profile load failed",
                hint=str(exc)[:60],
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._hint_var.set(str(exc)[:120])
            return
        self._refresh_mode()

    def _refresh_mode(self) -> None:
        if self._entry is None:
            return
        mode = self._mode_var.get()
        configure_preview_label(
            self._canvas,
            "loading",
            detail="Generating preview…",
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )
        try:
            if mode == "sphere":
                img = render_sphere_preview(self._entry)
                self._hint_var.set("Sphere — color on lit ball")
            elif mode == "wall_strip":
                img = render_wall_strip_preview(self._entry)
                self._hint_var.set("Wall — tiled color map")
            else:
                img, meta = render_building_section_preview(self._entry)
                src = meta.get("source") or meta.get("fallback", "degraded")
                self._hint_var.set(f"Building section — {src}")
            if image_is_near_black(img):
                configure_preview_label(
                    self._canvas,
                    "error",
                    detail="No color map yet",
                    hint="click Generate selected",
                    width=PREVIEW_THUMB_MD,
                    height=PREVIEW_THUMB_MD,
                )
                return
            img.thumbnail((240, 240), Image.Resampling.LANCZOS)
            photo = ImageTk.PhotoImage(img)
            self._photos[mode] = photo
            apply_preview_photo(self._canvas, photo)
        except Exception as exc:  # noqa: BLE001
            configure_preview_label(
                self._canvas,
                "error",
                detail="Preview failed",
                hint=str(exc)[:60],
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._on_log(f"preview {mode} failed: {exc}")
