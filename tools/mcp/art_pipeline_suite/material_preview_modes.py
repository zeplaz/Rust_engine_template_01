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

from .aps_theme import FONT_SMALL
from rust_engine_mcp.material_profiles import MaterialProfileEntry, ensure_profile_textures


class MaterialPreviewModesPanel(ttk.LabelFrame):
    def __init__(self, master: tk.Misc, *, on_log=None) -> None:
        super().__init__(master, text="Preview", padding=6)
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
            ("wall_strip", "Wall strip"),
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
        self._canvas = tk.Label(self, text="(select a profile)", bg="#e8e8e8", width=32, height=16)
        self._canvas.pack(fill=tk.BOTH, expand=True)
        self._hint_var = tk.StringVar(value="")
        ttk.Label(self, textvariable=self._hint_var, font=FONT_SMALL, foreground="#555").pack(anchor=tk.W)

    def set_profile(self, profile_id: str | None) -> None:
        if not profile_id:
            self._entry = None
            self._canvas.configure(image="", text="(select a profile)")
            self._hint_var.set("")
            return
        try:
            self._entry = ensure_profile_textures(profile_id, size=512)
        except Exception as exc:  # noqa: BLE001
            self._hint_var.set(str(exc)[:120])
            return
        self._refresh_mode()

    def _refresh_mode(self) -> None:
        if self._entry is None:
            return
        mode = self._mode_var.get()
        try:
            if mode == "sphere":
                img = render_sphere_preview(self._entry)
                self._hint_var.set("Sphere — albedo on lit ball")
            elif mode == "wall_strip":
                img = render_wall_strip_preview(self._entry)
                self._hint_var.set("Wall strip — tiled albedo")
            else:
                img, meta = render_building_section_preview(self._entry)
                src = meta.get("source") or meta.get("fallback", "degraded")
                self._hint_var.set(f"Building section — {src}")
            img.thumbnail((240, 240), Image.Resampling.LANCZOS)
            photo = ImageTk.PhotoImage(img)
            self._photos[mode] = photo
            self._canvas.configure(image=photo, text="")
        except Exception as exc:  # noqa: BLE001
            self._canvas.configure(image="", text=str(exc)[:80])
            self._on_log(f"preview {mode} failed: {exc}")
