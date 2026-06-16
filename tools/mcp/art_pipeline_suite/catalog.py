"""Catalog workspace — module browser (former Module Kit Viewer body)."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import ttk

from PIL import ImageTk

from module_viewer.model_store import (
    ModuleRecord,
    list_modules,
    open_in_blender,
    open_path,
    preview_trimesh,
    reindex_library,
    save_sidecar,
    validate_record,
)
from module_viewer.preview_browser import preview_in_browser
from rust_engine_mcp.aps_catalog_preview import render_module_list_thumb
from rust_engine_mcp.paths import repo_root

from .aps_tooltips import bind_aps_tooltip
from .aps_inline_feedback import set_inline_status
from .aps_paned import add_pane, horizontal_paned
from .aps_scroll import attach_wheel_area, bind_debounced_scrollregion, canvas_yscroll, text_yscroll
from .aps_theme import track_wraplength
from .metadata_flow_panel import MetadataFlowPanel
from .state import SuiteState

SIDECAR_TRUTH = (
    "Sidecar tags ≠ ship truth — assembly snapshot semantic_tags and material_profile win at runtime."
)


class CatalogPanel(ttk.Frame):
    def __init__(self, master: tk.Misc, state: SuiteState, *, on_select) -> None:
        super().__init__(master, padding=4)
        self.state = state
        self._on_select = on_select
        self._records: list[ModuleRecord] = []
        self._current: ModuleRecord | None = None
        self._row_photos: dict[str, ImageTk.PhotoImage] = {}
        self._build()
        self.refresh_list()

    def _build(self) -> None:
        self.metadata_flow = MetadataFlowPanel(self, context="catalog")
        self.metadata_flow.pack(fill=tk.X, pady=(0, 6))
        bar = ttk.Frame(self)
        bar.pack(fill=tk.X, pady=(0, 4))
        ttk.Label(bar, text="Batch").pack(side=tk.LEFT)
        self.batch_var = tk.StringVar(value="(all)")
        self.batch_combo = ttk.Combobox(
            bar, textvariable=self.batch_var, width=18, state="readonly", values=["(all)"]
        )
        self.batch_combo.pack(side=tk.LEFT, padx=(4, 12))
        self.batch_combo.bind("<<ComboboxSelected>>", lambda _e: self.refresh_list())
        bind_aps_tooltip(self.batch_combo, "cat_batch_filter")

        ttk.Label(bar, text="Category").pack(side=tk.LEFT)
        self.category_var = tk.StringVar(value="(all)")
        self.category_combo = ttk.Combobox(
            bar, textvariable=self.category_var, width=14, state="readonly", values=["(all)"]
        )
        self.category_combo.pack(side=tk.LEFT, padx=(4, 12))
        self.category_combo.bind("<<ComboboxSelected>>", lambda _e: self.refresh_list())
        bind_aps_tooltip(self.category_combo, "cat_category_filter")
        refresh_btn = ttk.Button(bar, text="Refresh", command=self.refresh_list)
        refresh_btn.pack(side=tk.RIGHT)
        bind_aps_tooltip(refresh_btn, "cat_refresh")

        paned = horizontal_paned(self)
        paned.pack(fill=tk.BOTH, expand=True)

        left = ttk.Frame(paned, padding=4)
        add_pane(paned, left, weight=1, minsize=220)
        ttk.Label(left, text="Modules").pack(anchor=tk.W)
        list_wrap = ttk.Frame(left)
        list_wrap.pack(fill=tk.BOTH, expand=True, pady=4)
        list_scroll = ttk.Scrollbar(list_wrap, orient=tk.VERTICAL)
        list_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._list_canvas = tk.Canvas(
            list_wrap,
            highlightthickness=0,
            yscrollcommand=list_scroll.set,
        )
        self._list_canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        list_scroll.configure(command=self._list_canvas.yview)
        self._list_inner = ttk.Frame(self._list_canvas)
        self._list_win = self._list_canvas.create_window((0, 0), window=self._list_inner, anchor=tk.NW)

        def _on_canvas_configure(event) -> None:
            self._list_canvas.itemconfigure(self._list_win, width=event.width)

        self._list_canvas.bind("<Configure>", _on_canvas_configure)
        bind_debounced_scrollregion(self._list_canvas, self._list_inner)
        attach_wheel_area(
            self._list_canvas,
            self._list_inner,
            on_scroll_y=canvas_yscroll(self._list_canvas),
            area_id=f"aps-catalog-list-{id(self)}",
        )

        right = ttk.Frame(paned, padding=4)
        add_pane(paned, right, weight=3, minsize=360)
        self.summary = tk.StringVar(value="Select a module")
        summary_lbl = ttk.Label(right, textvariable=self.summary, wraplength=680, justify=tk.LEFT)
        summary_lbl.pack(anchor=tk.W, fill=tk.X)
        self.sidecar_truth_var = tk.StringVar(value=SIDECAR_TRUTH)
        sidecar_lbl = ttk.Label(
            right,
            textvariable=self.sidecar_truth_var,
            wraplength=680,
            justify=tk.LEFT,
            foreground="#555",
            font=("Segoe UI", 9),
        )
        sidecar_lbl.pack(anchor=tk.W, fill=tk.X, pady=(2, 0))
        bind_aps_tooltip(sidecar_lbl, "cat_sidecar_truth")
        self.validation = tk.StringVar(value="")
        self._validation_lbl = tk.Label(
            right,
            textvariable=self.validation,
            foreground="#444444",
            wraplength=680,
            justify=tk.LEFT,
            font=("Segoe UI", 9),
        )
        self._validation_lbl.pack(anchor=tk.W, fill=tk.X, pady=(4, 0))
        track_wraplength(right, summary_lbl, sidecar_lbl, self._validation_lbl, minimum=320)

        notebook = ttk.Notebook(right)
        notebook.pack(fill=tk.BOTH, expand=True, pady=8)

        meta_frame = ttk.Frame(notebook, padding=4)
        notebook.add(meta_frame, text="AssetSpec sidecar")
        self.meta_text = tk.Text(meta_frame, wrap=tk.NONE, undo=True, font=("Consolas", 10))
        meta_scroll = ttk.Scrollbar(meta_frame, orient=tk.VERTICAL, command=self.meta_text.yview)
        self.meta_text.configure(yscrollcommand=meta_scroll.set)
        self.meta_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        meta_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        attach_wheel_area(
            self.meta_text,
            on_scroll_y=text_yscroll(self.meta_text),
            area_id=f"aps-catalog-meta-{id(self)}",
        )

        index_frame = ttk.Frame(notebook, padding=4)
        notebook.add(index_frame, text="Index entry")
        self.index_text = tk.Text(index_frame, wrap=tk.NONE, state=tk.DISABLED, font=("Consolas", 10))
        idx_scroll = ttk.Scrollbar(index_frame, orient=tk.VERTICAL, command=self.index_text.yview)
        self.index_text.configure(yscrollcommand=idx_scroll.set)
        self.index_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        idx_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        attach_wheel_area(
            self.index_text,
            on_scroll_y=text_yscroll(self.index_text),
            area_id=f"aps-catalog-index-{id(self)}",
        )

        actions = ttk.Frame(right)
        actions.pack(fill=tk.X, pady=4)
        val_btn = ttk.Button(actions, text="Validate GLB", command=self.on_validate)
        val_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(val_btn, "cat_validate")
        bind_aps_tooltip(notebook, "cat_metadata")
        save_btn = ttk.Button(actions, text="Save metadata", command=self.on_save)
        save_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(save_btn, "cat_save_metadata")
        reindex_btn = ttk.Button(actions, text="Reindex library", command=self.on_reindex)
        reindex_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(reindex_btn, "cat_reindex")
        browser_btn = ttk.Button(actions, text="Preview in browser", command=self.on_browser_preview)
        browser_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(browser_btn, "cat_browser_preview")
        trimesh_btn = ttk.Button(actions, text="3D preview (trimesh)", command=self.on_trimesh)
        trimesh_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(trimesh_btn, "cat_trimesh")

    def refresh_list(self) -> None:
        batch = self.batch_var.get()
        category = self.category_var.get()
        batch_filter = None if batch == "(all)" else batch
        category_filter = None if category == "(all)" else category
        self._records = list_modules(batch_id=batch_filter, category=category_filter)
        for w in self._list_inner.winfo_children():
            w.destroy()
        self._row_photos.clear()
        for rec in self._records:
            self._add_list_row(rec)
        batches = sorted({str(r.index_row.get("batch_id") or "") for r in self._records if r.index_row.get("batch_id")})
        categories = sorted(
            {str(r.index_row.get("category") or "") for r in self._records if r.index_row.get("category")}
        )
        self.batch_combo["values"] = ["(all)", *batches]
        self.category_combo["values"] = ["(all)", *categories]
        if self._records:
            self._select_record(self._records[0])

    def _add_list_row(self, rec: ModuleRecord) -> None:
        row = ttk.Frame(self._list_inner, padding=2)
        row.pack(fill=tk.X, anchor=tk.W)
        glb = rec.glb_path
        if not glb.is_absolute():
            glb = repo_root() / glb
        thumb = render_module_list_thumb(glb, module_id=rec.module_id)
        img_lbl: tk.Label | None = None
        if thumb is not None:
            photo = ImageTk.PhotoImage(thumb)
            self._row_photos[rec.module_id] = photo
            img_lbl = tk.Label(row, image=photo, bg="#f0f0f0", cursor="hand2")
            img_lbl.image = photo
            img_lbl.pack(side=tk.LEFT, padx=(0, 6))
            img_lbl.bind("<Button-1>", lambda _e, r=rec: self._select_record(r))
            bind_aps_tooltip(img_lbl, "cat_list_thumb")
        cat = rec.index_row.get("category", "?")
        text = ttk.Label(
            row,
            text=f"{rec.module_id}\n{cat}",
            font=("Segoe UI", 8),
            cursor="hand2",
        )
        text.pack(side=tk.LEFT, fill=tk.X, expand=True)
        text.bind("<Button-1>", lambda _e, r=rec: self._select_record(r))
        bind_aps_tooltip(text, "cat_list_thumb")

    def _set_validation_result(self, text: str, *, ok: bool | None = None) -> None:
        if ok is True and text:
            text = f"Validation: PASS — {text}" if not text.startswith("Validation:") else text
        elif ok is False and text:
            text = f"Validation: FAIL — {text}" if not text.startswith("Validation:") else text
        set_inline_status(self._validation_lbl, self.validation, text, ok=ok)

    def _select_record(self, rec: ModuleRecord) -> None:
        self._current = rec
        self.on_select()

    def on_select(self, _event=None) -> None:
        rec = self._current
        if rec is None:
            return
        self.state.selected_module_id = rec.module_id
        self.state.selected_module_ids = [rec.module_id]
        dims = (rec.sidecar or {}).get("dimensions_m") or {}
        dim_txt = f"{dims.get('w', '?')}×{dims.get('h', '?')}×{dims.get('d', '?')} m"
        grid = rec.index_row.get("grid_units") or ["?", "?"]
        self.summary.set(
            f"{rec.module_id} · job {rec.job_id} · {rec.index_row.get('archetype', '')}\n"
            f"GLB: {rec.glb_path}\n"
            f"Grid {grid} · dims {dim_txt} · batch {rec.index_row.get('batch_id', '—')}"
        )
        self.validation.set("")
        self._validation_lbl.configure(foreground="#444444")
        sidecar_json = json.dumps(rec.sidecar or {}, indent=2)
        self.meta_text.configure(state=tk.NORMAL)
        self.meta_text.delete("1.0", tk.END)
        self.meta_text.insert("1.0", sidecar_json if rec.sidecar else '{\n  "schema_version": 1\n}')
        self._set_readonly(self.index_text, json.dumps(rec.index_row, indent=2))
        style = rec.index_row.get("style_pack") or rec.index_row.get("stylepack")
        if style:
            self.state.style_pack_id = str(style)
        self._on_select(rec)

    def _set_readonly(self, widget: tk.Text, text: str) -> None:
        widget.configure(state=tk.NORMAL)
        widget.delete("1.0", tk.END)
        widget.insert("1.0", text)
        widget.configure(state=tk.DISABLED)

    def _require_current(self) -> ModuleRecord | None:
        if self._current is None:
            self._set_validation_result("Select a module first.", ok=False)
            return None
        return self._current

    def on_validate(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        report = validate_record(rec)
        status = "PASS" if report.get("valid") else "FAIL"
        verts = report.get("vertex_count", "?")
        issues = "; ".join(report.get("issues") or []) or "none"
        ok = bool(report.get("valid"))
        self._set_validation_result(f"Validation {status} · {verts} verts · {issues}", ok=ok)

    def on_save(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        try:
            data = json.loads(self.meta_text.get("1.0", tk.END))
        except json.JSONDecodeError as exc:
            self._set_validation_result(f"Invalid JSON: {exc}", ok=False)
            return
        path = save_sidecar(rec, data)
        self._set_validation_result(f"Saved metadata — {path.name}", ok=True)
        self.on_select()

    def on_reindex(self) -> None:
        try:
            result = reindex_library()
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Reindex failed: {exc}", ok=False)
            return
        count = result.get("entry_count", 0)
        self._set_validation_result(f"Reindex OK — {count} entries", ok=True)
        self.refresh_list()

    def on_browser_preview(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        try:
            url = preview_in_browser(rec.glb_path, title=rec.module_id)
        except Exception as exc:  # noqa: BLE001
            self._set_validation_result(f"Browser preview failed: {exc}", ok=False)
            return
        if url.startswith("http"):
            self._set_validation_result(f"Browser preview: {url}", ok=True)

    def on_trimesh(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        err = preview_trimesh(rec.glb_path)
        if err:
            self._set_validation_result(f"3D preview: {err}", ok=False)
            return
        self._set_validation_result("3D preview opened.", ok=True)
