"""APS-PREVIEW-001 — selected slot: module, material, combined, placement context."""

from __future__ import annotations

import tkinter as tk
from pathlib import Path
from tkinter import ttk
from typing import Any, Callable

from PIL import Image, ImageTk

from .aps_theme import FONT_SMALL, FONT_MONO_SMALL
from .slot_preview_render import (
    PREVIEW_SIZE,
    render_combined_preview,
    render_material_preview,
    render_module_isolated,
    render_placement_context_strip,
    write_aps_preview_001_witness,
)


class SlotPreviewPanel(ttk.LabelFrame):
    THUMB = PREVIEW_SIZE

    def __init__(
        self,
        master: tk.Misc,
        *,
        on_log: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(master, text="Selected piece previews", padding=6)
        self._on_log = on_log or (lambda _line: None)
        self._photos: dict[str, ImageTk.PhotoImage] = {}
        self._assembly_context_img: Image.Image | None = None
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Previews of the selected piece: module alone, its material, the two combined, and where it sits.",
            wraplength=520,
            font=FONT_SMALL,
            foreground="#555",
        ).pack(anchor=tk.W, pady=(0, 6))

        grid = ttk.Frame(self)
        grid.pack(fill=tk.X)

        self._module_title = tk.StringVar(value="Module preview")
        self._material_title = tk.StringVar(value="Material preview")
        self._combined_title = tk.StringVar(value="Combined")
        self._context_title = tk.StringVar(value="Placement context")

        self._module_label = self._thumb_cell(grid, 0, 0, self._module_title)
        self._material_label = self._thumb_cell(grid, 0, 1, self._material_title)
        self._combined_label = self._thumb_cell(grid, 1, 0, self._combined_title)
        self._context_label = self._thumb_cell(grid, 1, 1, self._context_title)

        meta = ttk.Frame(self)
        meta.pack(fill=tk.X, pady=(6, 0))
        self._mesh_var = tk.StringVar(value="")
        ttk.Label(meta, textvariable=self._mesh_var, font=FONT_MONO_SMALL, foreground="#444", wraplength=500).pack(
            anchor=tk.W
        )
        self._slot_why_var = tk.StringVar(value="")
        ttk.Label(meta, textvariable=self._slot_why_var, font=FONT_SMALL, foreground="#333", wraplength=500).pack(
            anchor=tk.W, pady=2
        )

    def _thumb_cell(self, parent: ttk.Frame, row: int, col: int, title_var: tk.StringVar) -> tk.Label:
        cell = ttk.Frame(parent, padding=4)
        cell.grid(row=row, column=col, sticky=tk.N)
        ttk.Label(cell, textvariable=title_var, font=(FONT_SMALL[0], FONT_SMALL[1], "bold")).pack(anchor=tk.CENTER)
        lbl = tk.Label(
            cell,
            text="(select piece)",
            width=self.THUMB // 8,
            height=self.THUMB // 16,
            bg="#ececec",
            relief=tk.SUNKEN,
        )
        lbl.pack(pady=4)
        return lbl

    def set_assembly_context_image(self, image: Image.Image | None) -> None:
        self._assembly_context_img = image

    def show_placement(
        self,
        placement: dict[str, Any] | None,
        *,
        snapshot: dict[str, Any] | None = None,
        grammar_chain: dict[str, str] | None = None,
    ) -> None:
        if not placement:
            self._clear()
            return
        module_id = str(placement.get("module_id") or "—")
        mat = str(placement.get("material_profile") or "—")
        glb_rel = str(placement.get("glb_path") or "")
        self._module_title.set(f"Module\n{module_id}")
        self._material_title.set(f"Material\n{mat}")
        self._combined_title.set(f"Combined\n{module_id[:12]}+\n{mat[:12]}")

        mod_img = render_module_isolated(glb_rel) if glb_rel else None
        mat_img = render_material_preview(mat) if mat and mat != "—" else None
        module_ok = mod_img is not None
        material_ok = mat_img is not None
        if mod_img is not None:
            self._set_thumb(self._module_label, "module", mod_img)
        if mat_img is not None:
            self._set_thumb(self._material_label, "material", mat_img)
        if mod_img is not None and mat_img is not None:
            comb = render_combined_preview(mod_img, mat_img)
            self._set_thumb(self._combined_label, "combined", comb)
            combined_ok = True
        else:
            combined_ok = False
            self._combined_label.configure(image="", text="(need module+material)")

        ctx = render_placement_context_strip(
            snapshot,
            selected=placement,
            assembly_thumb=self._assembly_context_img,
        )
        gx = int(placement.get("grid_x") or 0)
        gy = int(placement.get("grid_y") or 0)
        fl = int(placement.get("floor") or 0)
        self._context_title.set(f"Context\n({gx},{gy}) f{fl}")
        self._set_thumb(self._context_label, "context", ctx)

        self._mesh_var.set(f"Mesh: {glb_rel or '—'}")
        self._slot_why_var.set(_slot_generation_hint(placement, grammar_chain, snapshot))
        write_aps_preview_001_witness(
            module_ok=module_ok,
            material_ok=material_ok,
            combined_ok=combined_ok,
        )
        self._on_log(f"slot preview {module_id} · {mat}")

    def _set_thumb(self, label: tk.Label, key: str, image: Image.Image) -> None:
        img = image.copy()
        img.thumbnail((self.THUMB, self.THUMB), Image.Resampling.LANCZOS)
        photo = ImageTk.PhotoImage(img)
        self._photos[key] = photo
        label.configure(image=photo, text="")

    def _clear(self) -> None:
        for lbl in (self._module_label, self._material_label, self._combined_label, self._context_label):
            lbl.configure(image="", text="(select slot)")
        self._mesh_var.set("")
        self._slot_why_var.set("")


def _slot_generation_hint(
    placement: dict[str, Any],
    chain: dict[str, str] | None,
    snapshot: dict[str, Any] | None,
) -> str:
    from rust_engine_mcp.aps_grammar_labels import human_label

    chain = chain or (snapshot or {}).get("grammar_rule_chain") or {}
    if not isinstance(chain, dict):
        chain = {}
    gx = placement.get("grid_x")
    gy = placement.get("grid_y")
    parts = []
    if chain.get("massing"):
        parts.append(f"massing={human_label(str(chain['massing']))}")
    if chain.get("facade"):
        parts.append(f"facade={human_label(str(chain['facade']))}")
    if chain.get("roof"):
        parts.append(f"roof={human_label(str(chain['roof']))}")
    if chain.get("age"):
        parts.append(f"age={human_label(str(chain['age']))}")
    district = (snapshot or {}).get("district_style")
    if district:
        parts.append(f"district={human_label(str(district))}")
    seed = (snapshot or {}).get("seed")
    if seed is not None:
        parts.append(f"seed={seed}")
    base = f"Cell ({gx},{gy})"
    if parts:
        return f"{base} · " + " · ".join(parts)
    return f"{base} · placement fill (no grammar chain on snapshot)"
