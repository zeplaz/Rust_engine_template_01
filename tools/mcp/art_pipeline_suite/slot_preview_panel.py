"""APS-PREVIEW-001 — selected slot: module, material, combined, placement context."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

from PIL import Image, ImageTk

from .aps_preview_state import (
    apply_preview_photo,
    configure_preview_label,
    image_is_near_black,
    make_fidelity_chip,
)
from .aps_theme import COLOR_MUTED, FONT_SMALL, FONT_MONO_SMALL, PREVIEW_THUMB_SM
from .job_controller import JobRecord, JobResult
from .slot_preview_render import (
    render_combined_preview,
    render_material_preview,
    render_module_isolated,
    render_placement_context_strip,
    write_aps_preview_001_witness,
)

try:
    import trimesh  # noqa: F401

    _TRIMESH_OK = True
except ImportError:
    _TRIMESH_OK = False


class SlotPreviewPanel(ttk.LabelFrame):
    THUMB = PREVIEW_THUMB_SM

    def __init__(
        self,
        master: tk.Misc,
        *,
        on_log: Callable[[str], None] | None = None,
        start_job: Callable | None = None,
    ) -> None:
        super().__init__(master, text="Piece previews", padding=6)
        self._on_log = on_log or (lambda _line: None)
        self._start_job = start_job
        self._photos: dict[str, ImageTk.PhotoImage] = {}
        self._assembly_context_img: Image.Image | None = None
        self._thumb_labels: dict[str, tk.Label] = {}
        self._pending: dict[str, Any] | None = None
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Module · material · combined · placement context for the selected grid cell.",
            wraplength=520,
            font=FONT_SMALL,
            foreground=COLOR_MUTED,
        ).pack(anchor=tk.W, pady=(0, 6))

        grid = ttk.Frame(self)
        grid.pack(fill=tk.X)

        self._module_title = tk.StringVar(value="Module preview")
        self._material_title = tk.StringVar(value="Material preview")
        self._combined_title = tk.StringVar(value="Combined")
        self._context_title = tk.StringVar(value="Placement context")

        self._thumb_labels["module"] = self._thumb_cell(grid, 0, 0, self._module_title, "quick")
        self._thumb_labels["material"] = self._thumb_cell(grid, 0, 1, self._material_title, "quick")
        self._thumb_labels["combined"] = self._thumb_cell(grid, 1, 0, self._combined_title, "quick")
        self._thumb_labels["context"] = self._thumb_cell(grid, 1, 1, self._context_title, "layout")

        meta = ttk.Frame(self)
        meta.pack(fill=tk.X, pady=(6, 0))
        self._mesh_var = tk.StringVar(value="")
        ttk.Label(meta, textvariable=self._mesh_var, font=FONT_MONO_SMALL, foreground=COLOR_MUTED, wraplength=500).pack(
            anchor=tk.W
        )
        self._slot_why_var = tk.StringVar(value="")
        ttk.Label(meta, textvariable=self._slot_why_var, font=FONT_SMALL, wraplength=500).pack(anchor=tk.W, pady=2)
        self._show_empty_all()

    def _thumb_cell(self, parent: ttk.Frame, row: int, col: int, title_var: tk.StringVar, fidelity: str) -> tk.Label:
        cell = ttk.Frame(parent, padding=4)
        cell.grid(row=row, column=col, sticky=tk.N)
        make_fidelity_chip(cell, fidelity).pack(anchor=tk.CENTER)
        ttk.Label(cell, textvariable=title_var, font=(FONT_SMALL[0], FONT_SMALL[1], "bold")).pack(anchor=tk.CENTER)
        lbl = tk.Label(cell, relief=tk.SUNKEN)
        lbl.pack(pady=4)
        configure_preview_label(
            lbl,
            "empty",
            detail="No piece selected",
            width=self.THUMB,
            height=self.THUMB,
        )
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
        self._pending = {
            "placement": placement,
            "snapshot": snapshot,
            "grammar_chain": grammar_chain,
        }
        self._show_loading_all(placement)
        if self._start_job is not None:

            def worker(_cancel) -> JobResult:
                return JobResult(True, "ok", data=dict(self._pending or {}))

            def on_done(record: JobRecord) -> None:
                if record.result and record.result.ok and record.result.data:
                    self._render_placement(record.result.data)

            self._start_job("Slot previews", worker, on_done=on_done)
            return
        self._render_placement(self._pending)

    def _show_loading_all(self, placement: dict[str, Any] | None = None) -> None:
        module_id = str((placement or {}).get("module_id") or "—")
        mat = str((placement or {}).get("material_profile") or "—")
        self._module_title.set(f"Module\n{module_id}")
        self._material_title.set(f"Material\n{mat}")
        self._combined_title.set(f"Combined\n{module_id[:12]}+\n{mat[:12]}")
        for key in self._thumb_labels:
            configure_preview_label(
                self._thumb_labels[key],
                "loading",
                detail="Rendering…",
                width=self.THUMB,
                height=self.THUMB,
            )

    def _show_empty_all(self) -> None:
        empty_copy = {
            "module": "No piece selected",
            "material": "No material",
            "combined": "Select a piece",
            "context": "No placement",
        }
        for key, lbl in self._thumb_labels.items():
            configure_preview_label(
                lbl,
                "empty",
                detail=empty_copy[key],
                width=self.THUMB,
                height=self.THUMB,
            )

    def _render_placement(self, payload: dict[str, Any]) -> None:
        placement = payload.get("placement")
        snapshot = payload.get("snapshot")
        grammar_chain = payload.get("grammar_chain")
        if not placement:
            self._clear()
            return

        if not _TRIMESH_OK:
            hint = "layout view still works"
            detail = "Quick preview needs optional 3D library"
            for lbl in self._thumb_labels.values():
                configure_preview_label(
                    lbl,
                    "error",
                    detail=detail,
                    hint=hint,
                    width=self.THUMB,
                    height=self.THUMB,
                )
            self._mesh_var.set("")
            self._slot_why_var.set(detail)
            return

        module_id = str(placement.get("module_id") or "—")
        mat = str(placement.get("material_profile") or "—")
        glb_rel = str(placement.get("glb_path") or "")

        mod_img = render_module_isolated(glb_rel) if glb_rel else None
        mat_img = render_material_preview(mat) if mat and mat != "—" else None
        module_ok = mod_img is not None and not image_is_near_black(mod_img)
        material_ok = mat_img is not None and not image_is_near_black(mat_img)

        if mod_img is not None and module_ok:
            self._set_thumb(self._thumb_labels["module"], "module", mod_img)
        else:
            configure_preview_label(
                self._thumb_labels["module"],
                "error" if glb_rel else "empty",
                detail="No 3D file" if glb_rel else "No piece selected",
                hint="validate or pick another module" if glb_rel else None,
                width=self.THUMB,
                height=self.THUMB,
            )

        if mat_img is not None and material_ok:
            self._set_thumb(self._thumb_labels["material"], "material", mat_img)
        else:
            configure_preview_label(
                self._thumb_labels["material"],
                "empty" if not mat or mat == "—" else "error",
                detail="No material" if not mat or mat == "—" else "Material preview unavailable",
                width=self.THUMB,
                height=self.THUMB,
            )

        if mod_img is not None and mat_img is not None and module_ok and material_ok:
            comb = render_combined_preview(mod_img, mat_img)
            if image_is_near_black(comb):
                configure_preview_label(
                    self._thumb_labels["combined"],
                    "error",
                    detail="Combined preview unavailable",
                    hint="check module and material",
                    width=self.THUMB,
                    height=self.THUMB,
                )
                combined_ok = False
            else:
                self._set_thumb(self._thumb_labels["combined"], "combined", comb)
                combined_ok = True
        else:
            configure_preview_label(
                self._thumb_labels["combined"],
                "empty",
                detail="Select a piece",
                width=self.THUMB,
                height=self.THUMB,
            )
            combined_ok = False

        ctx = render_placement_context_strip(
            snapshot,
            selected=placement,
            assembly_thumb=self._assembly_context_img,
        )
        gx = int(placement.get("grid_x") or 0)
        gy = int(placement.get("grid_y") or 0)
        fl = int(placement.get("floor") or 0)
        self._context_title.set(f"Context\n({gx},{gy}) f{fl}")
        self._set_thumb(self._thumb_labels["context"], "context", ctx)

        self._mesh_var.set(f"Mesh: {glb_rel or '—'}")
        self._slot_why_var.set(_slot_generation_hint(placement, grammar_chain, snapshot))
        write_aps_preview_001_witness(
            module_ok=module_ok,
            material_ok=material_ok,
            combined_ok=combined_ok,
        )
        self._on_log(f"slot preview {module_id} · {mat}")

    def _set_thumb(self, label: tk.Label, key: str, image: Image.Image) -> None:
        if image_is_near_black(image):
            configure_preview_label(
                label,
                "error",
                detail="Thumbnail unavailable",
                hint="use layout view",
                width=self.THUMB,
                height=self.THUMB,
            )
            return
        img = image.copy()
        img.thumbnail((self.THUMB, self.THUMB), Image.Resampling.LANCZOS)
        photo = ImageTk.PhotoImage(img)
        self._photos[key] = photo
        apply_preview_photo(label, photo)

    def _clear(self) -> None:
        self._pending = None
        self._show_empty_all()
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
