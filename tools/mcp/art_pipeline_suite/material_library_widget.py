"""Shared APS material library — browse, preview, generate, register (APS-MAT-006)."""

from __future__ import annotations

import threading
import tkinter as tk
from tkinter import messagebox, simpledialog, ttk
from typing import Callable

from PIL import Image, ImageTk

from rust_engine_mcp.material_thumb_cache import LIST_THUMB, get_cached_thumb, warm_thumbnail_cache
from rust_engine_mcp.material_category_tree import category_label, tree_roots
from rust_engine_mcp.material_profiles import (
    CATEGORY_ORDER,
    MaterialProfileEntry,
    ensure_profile_textures,
    generate_all_missing,
    infer_category,
    infer_generator,
    load_material_profile_catalog,
    open_profile_folder,
    open_registry_in_editor,
    register_material_profile,
)
from rust_engine_mcp.material_textures import PILOT_PROFILES, generate_profile

from .aps_paned import add_pane, horizontal_paned
from .aps_scroll import attach_wheel_area, bind_debounced_scrollregion, canvas_yscroll
from .aps_tooltips import bind_aps_tooltip, bind_many
from .job_controller import JobRecord, JobResult, JobState

StartJobFn = Callable[..., str | None]


class MaterialLibraryWidget(ttk.Frame):
    """Categorized library with search, map status, and iterative edit workflow."""

    THUMB_SIZE = 72
    PREVIEW_SIZE = 168

    def __init__(
        self,
        master: tk.Misc,
        *,
        mode: str = "assign",
        on_apply_material=None,
        on_log=None,
        layout: str = "vertical",
        on_open_in_assembly=None,
        on_profile_selected=None,
        start_job: StartJobFn | None = None,
    ) -> None:
        super().__init__(master, padding=4)
        self._mode = mode
        self._on_apply = on_apply_material or (lambda _pid: None)
        self._on_log = on_log or (lambda _line: None)
        self._on_open_in_assembly = on_open_in_assembly
        self._on_profile_selected = on_profile_selected
        self._start_job = start_job
        self._layout = layout if layout in ("vertical", "horizontal", "studio_tree") else "vertical"
        self._tree_category: str | None = None
        self._entries: list[MaterialProfileEntry] = []
        self._filtered: list[MaterialProfileEntry] = []
        self._selected_id: str | None = None
        self._thumb_photos: dict[str, ImageTk.PhotoImage] = {}
        self._row_photos: dict[str, ImageTk.PhotoImage] = {}
        self._preview_photo: ImageTk.PhotoImage | None = None
        self._apply_btn = None
        self._gen_selected_btn = None
        self._search_entry = None
        self._build()

    def _build(self) -> None:
        toolbar = ttk.Frame(self)
        toolbar.pack(fill=tk.X, pady=(0, 4))
        left_tools = ttk.Frame(toolbar)
        left_tools.pack(side=tk.LEFT, fill=tk.X, expand=True)
        self._add_btn = ttk.Button(left_tools, text="Add profile…", command=self._add_profile_dialog)
        self._add_btn.pack(side=tk.LEFT, padx=2)
        self._gen_selected_btn = ttk.Button(left_tools, text="Generate selected", command=self._generate_selected)
        self._gen_selected_btn.pack(side=tk.LEFT, padx=2)
        self._gen_all_btn = ttk.Button(left_tools, text="Generate all missing", command=self._generate_all_missing)
        self._gen_all_btn.pack(side=tk.LEFT, padx=2)
        self._open_folder_btn = ttk.Button(left_tools, text="Open texture folder", command=self._open_folder)
        self._open_folder_btn.pack(side=tk.LEFT, padx=2)
        self._open_registry_btn = ttk.Button(left_tools, text="Open registry JSON", command=self._open_registry)
        self._open_registry_btn.pack(side=tk.LEFT, padx=2)
        self._use_asm_btn = None
        if self._on_open_in_assembly:
            self._use_asm_btn = ttk.Button(toolbar, text="Use in Assembly", command=self._open_in_assembly)
            self._use_asm_btn.pack(side=tk.RIGHT, padx=6)

        filter_row = ttk.Frame(self)
        filter_row.pack(fill=tk.X, pady=2)
        ttk.Label(filter_row, text="Search").pack(side=tk.LEFT)
        self._search_var = tk.StringVar(value="")
        self._search_entry = ttk.Entry(filter_row, textvariable=self._search_var, width=18)
        self._search_entry.pack(side=tk.LEFT, padx=4)
        self._search_var.trace_add("write", lambda *_: self._apply_filters())
        ttk.Label(filter_row, text="Category").pack(side=tk.LEFT, padx=(8, 0))
        self._category_var = tk.StringVar(value="all")
        self._category_combo = ttk.Combobox(
            filter_row,
            textvariable=self._category_var,
            values=list(CATEGORY_ORDER),
            width=18,
            state="readonly",
        )
        self._category_combo.pack(side=tk.LEFT, padx=4)
        self._category_var.trace_add("write", lambda *_: self._apply_filters())

        hint = (
            "Iterative workflow: Add profile → Generate → drop PNGs in texture folder → Reload. "
            "Double-click a card to open its folder."
        )
        if self._mode == "assign":
            hint = "Select a footprint cell first, then pick a profile. " + hint
        ttk.Label(self, text=hint, wraplength=520, justify=tk.LEFT, font=("Segoe UI", 8), foreground="#555").pack(
            anchor=tk.W, pady=(0, 4)
        )

        if self._layout == "studio_tree":
            self._build_studio_tree()
        elif self._layout == "horizontal":
            self._build_horizontal()
        else:
            self._build_vertical()

        self._status_var = tk.StringVar(value="")
        ttk.Label(self, textvariable=self._status_var, foreground="#444", font=("Segoe UI", 8)).pack(
            anchor=tk.W, pady=2
        )
        self.reload_catalog()

    def bind_tooltips(self) -> None:
        bind_many(
            [
                (self._add_btn, "mat_add_profile"),
                (self._gen_selected_btn, "mat_generate"),
                (self._gen_all_btn, "mat_generate_all"),
                (self._open_folder_btn, "mat_open_folder"),
                (self._open_registry_btn, "mat_open_registry"),
                (self._use_asm_btn, "mat_use_in_assembly"),
                (self._search_entry, "mat_search"),
                (self._category_combo, "mat_category"),
                (getattr(self, "_category_tree", None), "mat_category_tree"),
                (self._apply_btn, "mat_apply"),
                (getattr(self, "_reload_btn", None), "mat_reload_preview"),
            ]
        )

    def _build_preview_strip(self, parent: tk.Misc) -> None:
        preview = ttk.LabelFrame(parent, text="Preview & maps", padding=6)
        preview.pack(fill=tk.X, pady=(0, 6))

        row = ttk.Frame(preview)
        row.pack(fill=tk.X)
        self._preview_image = tk.Label(
            row,
            text="(select profile)",
            width=self.PREVIEW_SIZE // 8,
            height=self.PREVIEW_SIZE // 16,
            bg="#e8e8e8",
            relief=tk.SUNKEN,
        )
        self._preview_image.pack(side=tk.LEFT, padx=(0, 8))
        meta_col = ttk.Frame(row)
        meta_col.pack(side=tk.LEFT, fill=tk.X, expand=True)
        self._preview_meta = tk.StringVar(value="")
        ttk.Label(meta_col, textvariable=self._preview_meta, wraplength=240, justify=tk.LEFT).pack(anchor=tk.W)
        self._maps_var = tk.StringVar(value="")
        ttk.Label(meta_col, textvariable=self._maps_var, wraplength=240, foreground="#333", font=("Consolas", 9)).pack(
            anchor=tk.W, pady=2
        )

        btn_row = ttk.Frame(meta_col)
        btn_row.pack(anchor=tk.W, pady=4)
        if self._mode == "assign":
            self._apply_btn = ttk.Button(btn_row, text="Apply to selected slot", command=self._apply_selected)
            self._apply_btn.pack(side=tk.LEFT, padx=2)
        self._reload_btn = ttk.Button(btn_row, text="Reload preview", command=self._reload_selected_preview)
        self._reload_btn.pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Regenerate all pilots", command=self._refresh_pilot_textures).pack(side=tk.LEFT, padx=2)

    def _build_grid(self, parent: tk.Misc) -> tk.Canvas:
        grid_wrap = ttk.Frame(parent)
        grid_wrap.pack(fill=tk.BOTH, expand=True)
        canvas = tk.Canvas(grid_wrap, highlightthickness=0, height=200)
        scroll_y = ttk.Scrollbar(grid_wrap, orient=tk.VERTICAL, command=canvas.yview)
        self._grid_inner = ttk.Frame(canvas)
        self._grid_win = canvas.create_window((0, 0), window=self._grid_inner, anchor=tk.NW)

        def _on_canvas_configure(event) -> None:
            canvas.itemconfigure(self._grid_win, width=max(event.width, 140))

        canvas.bind("<Configure>", _on_canvas_configure)
        canvas.configure(yscrollcommand=scroll_y.set)
        canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll_y.pack(side=tk.RIGHT, fill=tk.Y)
        bind_debounced_scrollregion(canvas, self._grid_inner)
        attach_wheel_area(
            canvas,
            self._grid_inner,
            on_scroll_y=canvas_yscroll(canvas),
            area_id=f"aps-mat-grid-{id(self)}",
        )
        return canvas

    def _build_vertical(self) -> None:
        self._build_preview_strip(self)
        self._build_grid(self)

    def _build_horizontal(self) -> None:
        body = horizontal_paned(self)
        body.pack(fill=tk.BOTH, expand=True)
        grid_wrap = ttk.Frame(body, padding=2)
        add_pane(body, grid_wrap, weight=3, minsize=280)
        self._build_grid(grid_wrap)
        preview_wrap = ttk.Frame(body, padding=2)
        add_pane(body, preview_wrap, weight=1, minsize=180)
        self._build_preview_strip(preview_wrap)

    def _build_studio_tree(self) -> None:
        """APS-MAT-002 at scale — category tree + profile list (not card grid)."""
        body = horizontal_paned(self)
        body.pack(fill=tk.BOTH, expand=True)
        nav = ttk.Frame(body, padding=2)
        add_pane(body, nav, weight=1, minsize=240)
        preview_wrap = ttk.Frame(body, padding=2)
        add_pane(body, preview_wrap, weight=2, minsize=320)
        self._build_preview_strip(preview_wrap)

        nav_row = horizontal_paned(nav)
        nav_row.pack(fill=tk.BOTH, expand=True)
        tree_wrap = ttk.LabelFrame(nav_row, text="Categories", padding=4)
        add_pane(nav_row, tree_wrap, weight=1, minsize=140)
        list_wrap = ttk.LabelFrame(nav_row, text="Profiles", padding=4)
        add_pane(nav_row, list_wrap, weight=2, minsize=180)

        self._category_tree = ttk.Treeview(tree_wrap, show="tree", height=14)
        tree_scroll = ttk.Scrollbar(tree_wrap, orient=tk.VERTICAL, command=self._category_tree.yview)
        self._category_tree.configure(yscrollcommand=tree_scroll.set)
        self._category_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        tree_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._category_tree.bind("<<TreeviewSelect>>", self._on_tree_category_select)
        attach_wheel_area(
            self._category_tree,
            on_scroll_y=lambda delta: self._category_tree.yview_scroll(int(-delta * 3), "units"),
            area_id=f"aps-mat-tree-{id(self)}",
        )

        list_scroll = ttk.Scrollbar(list_wrap, orient=tk.VERTICAL)
        list_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        self._profile_canvas = tk.Canvas(
            list_wrap,
            highlightthickness=0,
            yscrollcommand=list_scroll.set,
            height=280,
        )
        self._profile_canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        list_scroll.configure(command=self._profile_canvas.yview)
        self._profile_rows_inner = ttk.Frame(self._profile_canvas)
        self._profile_rows_win = self._profile_canvas.create_window(
            (0, 0), window=self._profile_rows_inner, anchor=tk.NW
        )

        def _on_canvas_configure(event) -> None:
            self._profile_canvas.itemconfigure(self._profile_rows_win, width=event.width)

        self._profile_canvas.bind("<Configure>", _on_canvas_configure)
        bind_debounced_scrollregion(self._profile_canvas, self._profile_rows_inner)
        attach_wheel_area(
            self._profile_canvas,
            self._profile_rows_inner,
            on_scroll_y=canvas_yscroll(self._profile_canvas),
            area_id=f"aps-mat-profiles-{id(self)}",
        )

    def _on_tree_category_select(self, _event=None) -> None:
        sel = self._category_tree.selection()
        if not sel:
            return
        iid = sel[0]
        if iid == "all":
            self._tree_category = None
            self._category_var.set("all")
        else:
            self._tree_category = iid.replace("cat_", "")
            self._category_var.set(self._tree_category)
        self._apply_filters()

    def _category_matches_filter(self, entry_category: str) -> bool:
        if self._layout == "studio_tree" and self._tree_category:
            tc = self._tree_category
            ec = entry_category.lower()
            tc_l = tc.lower()
            if ec == tc_l:
                return True
            if ec.startswith(tc_l + "/"):
                return True
            # parent-only selection (e.g. industrial)
            if "/" not in tc_l and ec.split("/")[0].lower() == tc_l:
                return True
            return False
        cat = self._category_var.get().strip().lower()
        if cat == "all":
            return True
        return entry.category.lower() == cat

    def _on_list_profile_select(self, profile_id: str) -> None:
        self._select_profile(profile_id, apply=False)

    def _render_tree_and_list(self) -> None:
        if self._layout != "studio_tree":
            return
        for item in self._category_tree.get_children():
            self._category_tree.delete(item)
        self._category_tree.insert("", tk.END, iid="all", text=f"All ({len(self._entries)})", open=True)
        by_path: dict[str, list[MaterialProfileEntry]] = {}
        for entry in self._entries:
            path = entry.category or infer_category(entry.profile_id)
            by_path.setdefault(path, []).append(entry)
        # APS-MAT-003 — tree from material_category_tree_v1.json
        for root in tree_roots():
            root_id = str(root.get("id") or "")
            root_label = str(root.get("label") or root_id.replace("_", " ").title())
            children = list(root.get("children") or [])
            root_paths: list[str] = []
            if children:
                for child in sorted(children, key=lambda c: int(c.get("sort_order") or 0)):
                    leaf_id = str(child.get("id") or "")
                    root_paths.append(f"{root_id}/{leaf_id}")
            else:
                root_paths.append(root_id)
            parent_count = sum(len(by_path.get(p, [])) for p in root_paths)
            if parent_count == 0 and not children:
                continue
            parent_iid = f"cat_{root_id}"
            self._category_tree.insert(
                "",
                tk.END,
                iid=parent_iid,
                text=f"{root_label} ({parent_count})",
                open=True,
            )
            for path in root_paths:
                count = len(by_path.get(path, []))
                if count == 0 and "/" in path:
                    continue
                leaf_label = category_label(path)
                if path == root_id:
                    continue
                self._category_tree.insert(
                    parent_iid,
                    tk.END,
                    iid=f"cat_{path}",
                    text=f"{leaf_label} ({count})",
                )

        for w in self._profile_rows_inner.winfo_children():
            w.destroy()
        self._row_photos.clear()
        warm_thumbnail_cache(self._filtered, size=LIST_THUMB, limit=120)
        for entry in self._filtered:
            row = ttk.Frame(self._profile_rows_inner, padding=2)
            row.pack(fill=tk.X, anchor=tk.W)
            thumb_img = get_cached_thumb(entry, size=LIST_THUMB)
            if thumb_img is not None:
                thumb_img = thumb_img.copy()
                thumb_img.thumbnail((LIST_THUMB, LIST_THUMB), Image.Resampling.LANCZOS)
                photo = ImageTk.PhotoImage(thumb_img)
                self._row_photos[entry.profile_id] = photo
                lbl_img = tk.Label(row, image=photo, bg="#f0f0f0", cursor="hand2")
                lbl_img.image = photo
                lbl_img.pack(side=tk.LEFT, padx=(0, 6))
                lbl_img.bind(
                    "<Button-1>",
                    lambda _e, pid=entry.profile_id: self._on_list_profile_select(pid),
                )
                lbl_img.bind(
                    "<Double-Button-1>",
                    lambda _e, pid=entry.profile_id: self._open_folder(pid),
                )
            stxt = self._status_text(entry.texture_status())
            text = ttk.Label(
                row,
                text=f"{self._status_label(entry.texture_status(), entry.profile_id)}\n{entry.category}",
                font=("Segoe UI", 9),
                cursor="hand2",
            )
            text.pack(side=tk.LEFT, fill=tk.X, expand=True)
            text.bind("<Button-1>", lambda _e, pid=entry.profile_id: self._on_list_profile_select(pid))
            text.bind("<Double-Button-1>", lambda _e, pid=entry.profile_id: self._open_folder(pid))
            if self._selected_id == entry.profile_id:
                row.configure(relief=tk.RIDGE)

    def reload_catalog(self) -> None:
        self._entries = load_material_profile_catalog()
        self._apply_filters()

    def _apply_filters(self) -> None:
        q = self._search_var.get().strip().lower()
        cat = self._category_var.get().strip().lower()
        filtered: list[MaterialProfileEntry] = []
        for entry in self._entries:
            if not self._category_matches_filter(entry):
                continue
            blob = f"{entry.profile_id} {entry.label} {entry.generator} {entry.category}".lower()
            if q and q not in blob:
                continue
            filtered.append(entry)
        self._filtered = filtered
        if self._layout == "studio_tree":
            self._render_tree_and_list()
        else:
            self._render_grid()

    def _render_grid(self) -> None:
        for w in self._grid_inner.winfo_children():
            w.destroy()
        self._thumb_photos.clear()
        cols = 3 if self._layout == "vertical" else 4
        for i, entry in enumerate(self._filtered):
            card = self._make_card(entry)
            card.grid(row=i // cols, column=i % cols, padx=3, pady=3, sticky=tk.NW)
        ready = sum(1 for e in self._filtered if e.texture_status() == "ready")
        self._status_var.set(f"{len(self._filtered)} shown · {ready} ready · {len(self._entries)} total")
        if self._filtered and self._selected_id not in {e.profile_id for e in self._filtered}:
            self._select_profile(self._filtered[0].profile_id, apply=False)
        elif self._filtered and not self._selected_id:
            self._select_profile(self._filtered[0].profile_id, apply=False)
        self._status_var.set(
            f"{len(self._filtered)} shown · cache {LIST_THUMB}px · {len(self._entries)} total"
        )

    def _status_glyph(self, status: str) -> str:
        return {"ready": "●", "partial": "◐", "missing": "○"}.get(status, "?")

    def _status_text(self, status: str) -> str:
        return {"ready": "Ready", "partial": "Partial", "missing": "Missing"}.get(status, status.title())

    def _status_label(self, status: str, profile_id: str | None = None) -> str:
        """APS-UX-POLISH-001 — word-first status (glyph optional suffix)."""
        word = self._status_text(status)
        glyph = self._status_glyph(status)
        if profile_id:
            return f"{word} · {profile_id} · {glyph}"
        return f"{word} · {glyph}"

    def _load_thumb(self, entry: MaterialProfileEntry, *, force_reload: bool = False) -> ImageTk.PhotoImage:
        if not force_reload and entry.profile_id in self._thumb_photos:
            return self._thumb_photos[entry.profile_id]
        path = entry.albedo_path
        if path is None or not path.is_file():
            try:
                fresh = ensure_profile_textures(entry.profile_id, size=256)
                path = fresh.albedo_path
            except Exception:
                path = None
        if path and path.is_file():
            try:
                img = Image.open(path).convert("RGB")
                img.thumbnail((self.THUMB_SIZE, self.THUMB_SIZE), Image.Resampling.LANCZOS)
                photo = ImageTk.PhotoImage(img)
            except Exception:
                photo = self._placeholder_thumb((180, 60, 60), "ERR")
        else:
            photo = self._placeholder_thumb((96, 104, 118), "GEN")
        self._thumb_photos[entry.profile_id] = photo
        return photo

    def _placeholder_thumb(self, rgb: tuple[int, int, int], label: str) -> ImageTk.PhotoImage:
        img = Image.new("RGB", (self.THUMB_SIZE, self.THUMB_SIZE), rgb)
        try:
            from PIL import ImageDraw

            draw = ImageDraw.Draw(img)
            draw.text((8, self.THUMB_SIZE // 2 - 6), label, fill=(255, 255, 255))
        except Exception:
            pass
        return ImageTk.PhotoImage(img)

    def _make_card(self, entry: MaterialProfileEntry) -> ttk.Frame:
        frame = ttk.Frame(self._grid_inner, relief=tk.RIDGE, borderwidth=1, padding=3)
        status = entry.texture_status()
        stxt = self._status_text(status)
        top = ttk.Frame(frame)
        top.pack(fill=tk.X)
        status_lbl = ttk.Label(
            top,
            text=self._status_label(status),
            font=("Segoe UI", 9),
            foreground={"ready": "#0a6b0a", "partial": "#a66b00", "missing": "#888"}.get(status, "#888"),
        )
        status_lbl.pack(side=tk.LEFT)
        bind_aps_tooltip(status_lbl, "mat_status")
        photo = self._load_thumb(entry)
        selected = self._selected_id == entry.profile_id
        btn = tk.Button(
            frame,
            image=photo,
            text=entry.profile_id.replace("_", " "),
            compound=tk.TOP,
            command=lambda pid=entry.profile_id: self._on_card_click(pid),
            width=self.THUMB_SIZE + 8,
            bg="#e8eef5" if selected else "#f4f4f4",
            activebackground="#cce0ff",
            relief=tk.SOLID if selected else tk.FLAT,
            borderwidth=2 if selected else 1,
            font=("Segoe UI", 7),
            wraplength=self.THUMB_SIZE + 4,
        )
        btn.image = photo
        btn.pack()
        btn.bind("<Double-Button-1>", lambda _e, pid=entry.profile_id: self._open_folder(pid))
        frame._profile_btn = btn  # type: ignore[attr-defined]
        frame._profile_id = entry.profile_id  # type: ignore[attr-defined]
        return frame

    def _refresh_card_highlights(self) -> None:
        for child in self._grid_inner.winfo_children():
            btn = getattr(child, "_profile_btn", None)
            pid = getattr(child, "_profile_id", None)
            if btn is None or pid is None:
                continue
            selected = pid == self._selected_id
            btn.configure(
                bg="#e8eef5" if selected else "#f4f4f4",
                relief=tk.SOLID if selected else tk.FLAT,
                borderwidth=2 if selected else 1,
            )

    def _on_card_click(self, profile_id: str) -> None:
        self._select_profile(profile_id, apply=self._mode == "assign")

    def _maps_line(self, entry: MaterialProfileEntry) -> str:
        def _ok(p) -> str:
            return "yes" if p and p.is_file() else "no"

        return (
            f"albedo: {_ok(entry.albedo_path)}  "
            f"normal: {_ok(entry.normal_path)}  "
            f"roughness: {_ok(entry.roughness_path)}"
        )

    def _select_profile(self, profile_id: str, *, apply: bool = False) -> None:
        self._selected_id = profile_id
        err: str | None = None
        try:
            entry = ensure_profile_textures(profile_id, size=512)
        except Exception as exc:  # noqa: BLE001
            entry = next((e for e in load_material_profile_catalog() if e.profile_id == profile_id), None)
            err = str(exc)
        if entry is None:
            self._preview_meta.set(err or f"Unknown profile {profile_id}")
            self._maps_var.set("")
            return

        self._show_preview_image(entry, err=err)
        meta = (
            f"{entry.display_label()}\n"
            f"id: {entry.profile_id}\n"
            f"category: {entry.category}\n"
            f"generator: {entry.generator}  "
            f"registry: {'yes' if entry.in_registry else 'inferred'}\n"
            f"metallic: {entry.metallic:.2f}  roughness: {entry.roughness_base:.2f}"
        )
        self._preview_meta.set(meta)
        self._maps_var.set(self._maps_line(entry))
        self._status_var.set(f"Selected {profile_id} · {entry.texture_status()}")
        self._refresh_card_highlights()
        if self._on_profile_selected:
            self._on_profile_selected(profile_id)
        if apply:
            self._apply_selected()

    def _show_preview_image(self, entry: MaterialProfileEntry, *, err: str | None = None) -> None:
        path = entry.albedo_path
        if path and path.is_file():
            try:
                img = Image.open(path).convert("RGB")
                img.thumbnail((self.PREVIEW_SIZE, self.PREVIEW_SIZE), Image.Resampling.LANCZOS)
                self._preview_photo = ImageTk.PhotoImage(img)
                self._preview_image.configure(image=self._preview_photo, text="", bg="#f0f0f0")
                return
            except Exception as exc:  # noqa: BLE001
                err = err or str(exc)
        self._preview_photo = None
        msg = "No albedo — click Generate selected"
        if err:
            msg = f"{msg}\n{err[:120]}"
        self._preview_image.configure(image="", text=msg, bg="#f5e6e6" if err else "#eee8d5")

    def _reload_selected_preview(self) -> None:
        if not self._selected_id:
            return
        self._thumb_photos.pop(self._selected_id, None)
        self.reload_catalog()
        self._select_profile(self._selected_id, apply=False)

    def highlight_profile(self, profile_id: str | None) -> None:
        if profile_id:
            self._search_var.set("")
            self._category_var.set("all")
            self.reload_catalog()
            self._select_profile(profile_id, apply=False)

    def _apply_selected(self) -> None:
        if not self._selected_id:
            self._preview_meta.set("Select a material profile first.")
            return
        self._on_apply(self._selected_id)

    def _generate_selected(self) -> None:
        if not self._selected_id:
            self._preview_meta.set("Select a profile first.")
            return
        pid = self._selected_id
        if not self._start_job:
            try:
                ensure_profile_textures(pid, size=512, force=True)
            except Exception as exc:  # noqa: BLE001
                self._preview_meta.set(f"Generate failed: {exc}")
                return
            self._on_log(f"generated textures for {pid}")
            self._thumb_photos.pop(pid, None)
            self.reload_catalog()
            self._select_profile(pid, apply=False)
            return

        def worker(cancel: threading.Event) -> JobResult:
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            try:
                ensure_profile_textures(pid, size=512, force=True)
            except Exception as exc:  # noqa: BLE001
                return JobResult(False, f"Generate failed: {exc}", detail=str(exc))
            return JobResult(True, f"Generated textures for {pid}")

        def on_done(record: JobRecord) -> None:
            if record.result and record.result.ok:
                self._thumb_photos.pop(pid, None)
                self.reload_catalog()
                self._select_profile(pid, apply=False)
            elif record.result:
                self._preview_meta.set(record.result.summary)

        self._start_job("Generate profile", worker, on_done=on_done)

    def _generate_all_missing(self) -> None:
        if not self._start_job:
            self._generate_all_sync()
            return

        def worker(cancel: threading.Event) -> JobResult:
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            try:
                ids = generate_all_missing(size=512)
            except Exception as exc:  # noqa: BLE001
                return JobResult(False, f"Generate failed: {exc}", detail=str(exc))
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            return JobResult(True, f"Generated {len(ids)} profile(s)", data={"ids": ids})

        def on_done(record: JobRecord) -> None:
            if record.state == JobState.CANCELLED:
                self._preview_meta.set("Generate cancelled.")
                return
            if record.result and record.result.ok:
                self.reload_catalog()
                self._preview_meta.set(record.result.summary)
            elif record.result:
                self._preview_meta.set(record.result.summary)

        self._start_job(
            "Generate materials",
            worker,
            on_done=on_done,
            button=self._gen_all_btn,
            button_label="Generate all missing",
        )

    def _generate_all_sync(self) -> None:
        try:
            ids = generate_all_missing(size=512)
        except Exception as exc:  # noqa: BLE001
            self._preview_meta.set(f"Generate failed: {exc}")
            return
        self._on_log(f"generated {len(ids)} profiles")
        self.reload_catalog()
        self._preview_meta.set(f"Generated or refreshed {len(ids)} profile(s).")

    def _refresh_pilot_textures(self) -> None:
        for pid, defn in PILOT_PROFILES.items():
            try:
                generate_profile(defn, size=512)
            except Exception as exc:  # noqa: BLE001
                self._on_log(f"pilot regen failed {pid}: {exc}")
        self._thumb_photos.clear()
        self.reload_catalog()
        self._on_log("regenerated pilot material textures")

    def _open_folder(self, profile_id: str | None = None) -> None:
        pid = profile_id or self._selected_id
        if not pid:
            self._preview_meta.set("Select a profile first.")
            return
        folder = open_profile_folder(pid)
        self._on_log(f"opened {folder}")

    def _open_registry(self) -> None:
        path = open_registry_in_editor()
        self._on_log(f"opened registry {path.name}")
        self._preview_meta.set(f"Opened registry — {path.name}")

    def _add_profile_dialog(self) -> None:
        pid = simpledialog.askstring("Add material profile", "Profile id (e.g. steel_panel_02):", parent=self)
        if not pid:
            return
        pid = pid.strip()
        gen = infer_generator(pid)
        cat = infer_category(pid)
        dlg = tk.Toplevel(self)
        dlg.title("Register profile")
        dlg.transient(self.winfo_toplevel())
        dlg.grab_set()
        ttk.Label(dlg, text=f"Profile: {pid}", font=("Segoe UI", 10, "bold")).grid(row=0, column=0, columnspan=2, pady=6)
        ttk.Label(dlg, text="Generator").grid(row=1, column=0, sticky=tk.W, padx=8)
        gen_var = tk.StringVar(value=gen)
        ttk.Combobox(dlg, textvariable=gen_var, values=["steel", "brick", "concrete", "wood"], state="readonly").grid(
            row=1, column=1, sticky=tk.W, padx=8, pady=4
        )
        ttk.Label(dlg, text="Category").grid(row=2, column=0, sticky=tk.W, padx=8)
        cat_var = tk.StringVar(value=cat)
        ttk.Combobox(dlg, textvariable=cat_var, values=[c for c in CATEGORY_ORDER if c != "all"], state="readonly").grid(
            row=2, column=1, sticky=tk.W, padx=8, pady=4
        )
        gen_now = tk.BooleanVar(value=True)
        ttk.Checkbutton(dlg, text="Generate textures now", variable=gen_now).grid(
            row=3, column=0, columnspan=2, sticky=tk.W, padx=8, pady=4
        )

        def _ok() -> None:
            try:
                register_material_profile(pid, generator=gen_var.get(), category=cat_var.get())
                if gen_now.get():
                    ensure_profile_textures(pid, size=512, force=True)
            except Exception as exc:  # noqa: BLE001
                messagebox.showerror("Add profile", str(exc), parent=dlg)
                return
            dlg.destroy()
            self.reload_catalog()
            self._select_profile(pid, apply=False)
            self._on_log(f"registered profile {pid}")

        ttk.Button(dlg, text="Register", command=_ok).grid(row=4, column=0, padx=8, pady=8)
        ttk.Button(dlg, text="Cancel", command=dlg.destroy).grid(row=4, column=1, padx=8, pady=8)

    def _open_in_assembly(self) -> None:
        if not self._selected_id:
            self._preview_meta.set("Select a profile first.")
            return
        if self._on_open_in_assembly:
            self._on_open_in_assembly(self._selected_id)
