"""Catalog workspace — module browser (former Module Kit Viewer body)."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import messagebox, ttk

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

from .state import SuiteState


class CatalogPanel(ttk.Frame):
    def __init__(self, master: tk.Misc, state: SuiteState, *, on_select) -> None:
        super().__init__(master, padding=4)
        self.state = state
        self._on_select = on_select
        self._records: list[ModuleRecord] = []
        self._current: ModuleRecord | None = None
        self._build()
        self.refresh_list()

    def _build(self) -> None:
        bar = ttk.Frame(self)
        bar.pack(fill=tk.X, pady=(0, 4))
        ttk.Label(bar, text="Batch").pack(side=tk.LEFT)
        self.batch_var = tk.StringVar(value="(all)")
        self.batch_combo = ttk.Combobox(
            bar, textvariable=self.batch_var, width=18, state="readonly", values=["(all)"]
        )
        self.batch_combo.pack(side=tk.LEFT, padx=(4, 12))
        self.batch_combo.bind("<<ComboboxSelected>>", lambda _e: self.refresh_list())

        ttk.Label(bar, text="Category").pack(side=tk.LEFT)
        self.category_var = tk.StringVar(value="(all)")
        self.category_combo = ttk.Combobox(
            bar, textvariable=self.category_var, width=14, state="readonly", values=["(all)"]
        )
        self.category_combo.pack(side=tk.LEFT, padx=(4, 12))
        self.category_combo.bind("<<ComboboxSelected>>", lambda _e: self.refresh_list())
        ttk.Button(bar, text="Refresh", command=self.refresh_list).pack(side=tk.RIGHT)

        paned = ttk.Panedwindow(self, orient=tk.HORIZONTAL)
        paned.pack(fill=tk.BOTH, expand=True)

        left = ttk.Frame(paned, padding=4)
        paned.add(left, weight=1)
        ttk.Label(left, text="Modules").pack(anchor=tk.W)
        self.listbox = tk.Listbox(left, exportselection=False, activestyle="none")
        self.listbox.pack(fill=tk.BOTH, expand=True, pady=4)
        self.listbox.bind("<<ListboxSelect>>", self.on_select)

        right = ttk.Frame(paned, padding=4)
        paned.add(right, weight=3)
        self.summary = tk.StringVar(value="Select a module")
        ttk.Label(right, textvariable=self.summary, wraplength=680, justify=tk.LEFT).pack(
            anchor=tk.W, fill=tk.X
        )
        self.validation = tk.StringVar(value="")
        ttk.Label(
            right, textvariable=self.validation, foreground="#006400", wraplength=680, justify=tk.LEFT
        ).pack(anchor=tk.W, fill=tk.X, pady=(4, 0))

        notebook = ttk.Notebook(right)
        notebook.pack(fill=tk.BOTH, expand=True, pady=8)

        meta_frame = ttk.Frame(notebook, padding=4)
        notebook.add(meta_frame, text="AssetSpec sidecar")
        self.meta_text = tk.Text(meta_frame, wrap=tk.NONE, undo=True, font=("Consolas", 10))
        meta_scroll = ttk.Scrollbar(meta_frame, orient=tk.VERTICAL, command=self.meta_text.yview)
        self.meta_text.configure(yscrollcommand=meta_scroll.set)
        self.meta_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        meta_scroll.pack(side=tk.RIGHT, fill=tk.Y)

        index_frame = ttk.Frame(notebook, padding=4)
        notebook.add(index_frame, text="Index entry")
        self.index_text = tk.Text(index_frame, wrap=tk.NONE, state=tk.DISABLED, font=("Consolas", 10))
        idx_scroll = ttk.Scrollbar(index_frame, orient=tk.VERTICAL, command=self.index_text.yview)
        self.index_text.configure(yscrollcommand=idx_scroll.set)
        self.index_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        idx_scroll.pack(side=tk.RIGHT, fill=tk.Y)

        actions = ttk.Frame(right)
        actions.pack(fill=tk.X, pady=4)
        ttk.Button(actions, text="Validate GLB", command=self.on_validate).pack(side=tk.LEFT, padx=2)
        ttk.Button(actions, text="Save metadata", command=self.on_save).pack(side=tk.LEFT, padx=2)
        ttk.Button(actions, text="Reindex library", command=self.on_reindex).pack(side=tk.LEFT, padx=2)
        ttk.Button(actions, text="Preview in browser", command=self.on_browser_preview).pack(
            side=tk.LEFT, padx=2
        )
        ttk.Button(actions, text="3D preview (trimesh)", command=self.on_trimesh).pack(side=tk.LEFT, padx=2)

    def refresh_list(self) -> None:
        batch = self.batch_var.get()
        category = self.category_var.get()
        batch_filter = None if batch == "(all)" else batch
        category_filter = None if category == "(all)" else category
        self._records = list_modules(batch_id=batch_filter, category=category_filter)
        self.listbox.delete(0, tk.END)
        for rec in self._records:
            label = f"{rec.module_id}  ({rec.index_row.get('category', '?')})"
            self.listbox.insert(tk.END, label)
        batches = sorted({str(r.index_row.get("batch_id") or "") for r in self._records if r.index_row.get("batch_id")})
        categories = sorted(
            {str(r.index_row.get("category") or "") for r in self._records if r.index_row.get("category")}
        )
        self.batch_combo["values"] = ["(all)", *batches]
        self.category_combo["values"] = ["(all)", *categories]
        if self._records:
            self.listbox.selection_set(0)
            self.on_select()

    def on_select(self, _event=None) -> None:
        sel = self.listbox.curselection()
        if not sel:
            return
        self._current = self._records[sel[0]]
        rec = self._current
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
            messagebox.showinfo("Catalog", "Select a module first.")
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
        self.validation.set(f"Validation {status} · {verts} verts · {issues}")

    def on_save(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        try:
            data = json.loads(self.meta_text.get("1.0", tk.END))
        except json.JSONDecodeError as exc:
            messagebox.showerror("Save metadata", f"Invalid JSON:\n{exc}")
            return
        path = save_sidecar(rec, data)
        messagebox.showinfo("Save metadata", f"Saved:\n{path}")
        self.on_select()

    def on_reindex(self) -> None:
        try:
            result = reindex_library()
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Reindex", str(exc))
            return
        messagebox.showinfo("Reindex", f"Updated {result.get('entry_count', 0)} entries")
        self.refresh_list()

    def on_browser_preview(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        try:
            url = preview_in_browser(rec.glb_path, title=rec.module_id)
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Browser preview", str(exc))
            return
        if url.startswith("http"):
            self.validation.set(f"Browser preview: {url}")

    def on_trimesh(self) -> None:
        rec = self._require_current()
        if rec is None:
            return
        err = preview_trimesh(rec.glb_path)
        if err:
            messagebox.showwarning("3D preview", err)
