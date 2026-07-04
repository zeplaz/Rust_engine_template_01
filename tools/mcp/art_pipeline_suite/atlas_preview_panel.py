"""APS-ATLAS-PREVIEW-001/002 — packed atlas + UV grid + cell strip."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import ttk
from typing import Any

from PIL import Image, ImageDraw, ImageTk

from module_viewer.pipeline_runner import find_latest_atlas_in
from rust_engine_mcp.paths import repo_root

from .aps_paned import add_pane, horizontal_paned
from .preview_state_display import (
    apply_preview_photo,
    configure_preview_label,
    image_is_near_black,
    make_fidelity_chip,
    set_preview_status,
    show_image_file_thumbnail,
)
from .aps_scroll import attach_wheel_area, bind_debounced_scrollregion, canvas_xscroll, canvas_yscroll, text_yscroll
from .aps_theme import (
    COLOR_INPUT_BG,
    COLOR_MUTED,
    COLOR_TEXT_BODY,
    COLOR_TEXT_SUBTLE,
    FONT_SMALL,
    FONT_UI,
    FONT_UI_BOLD,
    PREVIEW_THUMB_SM,
    track_wraplength,
)


def _resolve_path(raw: str) -> Path:
    p = Path(raw.replace("\\", "/"))
    if p.is_absolute():
        return p
    return (repo_root() / p).resolve()


def _source_pngs(folder: Path) -> list[Path]:
    if not folder.is_dir():
        return []
    out: list[Path] = []
    for p in sorted(folder.glob("*.png")):
        name = p.name.lower()
        if name.startswith("tile_map"):
            continue
        out.append(p)
    return out


def _load_atlas_meta(folder: Path) -> dict[str, Any] | None:
    meta = folder / "atlas_meta.json"
    if not meta.is_file():
        return None
    try:
        data = json.loads(meta.read_text(encoding="utf-8"))
        return data if isinstance(data, dict) else None
    except (OSError, json.JSONDecodeError):
        return None


def _draw_uv_grid(
    img: Image.Image,
    *,
    columns: int,
    rows: int,
    highlight: tuple[int, int] | None = None,
) -> Image.Image:
    """Overlay columns×rows grid; optional highlight cell (col, row)."""
    out = img.copy()
    draw = ImageDraw.Draw(out)
    w, h = out.size
    if columns < 1 or rows < 1:
        return out
    cw, rh = w / columns, h / rows
    for c in range(columns + 1):
        x = int(c * cw)
        draw.line([(x, 0), (x, h)], fill="#888888", width=1)
    for r in range(rows + 1):
        y = int(r * rh)
        draw.line([(0, y), (w, y)], fill="#888888", width=1)
    if highlight is not None:
        col, row = highlight
        if 0 <= col < columns and 0 <= row < rows:
            x0, y0 = int(col * cw), int(row * rh)
            x1, y1 = int((col + 1) * cw), int((row + 1) * rh)
            draw.rectangle((x0, y0, x1 - 1, y1 - 1), outline="#0066cc", width=2)
    return out


class AtlasPreviewPanel(ttk.LabelFrame):
    THUMB = 96
    ATLAS_MAX = 320

    def __init__(self, master: tk.Misc, *, on_log=None) -> None:
        super().__init__(master, text="Tile preview (packed atlas + cells)", padding=6)
        self._on_log = on_log or (lambda _s: None)
        self._photos: dict[str, ImageTk.PhotoImage] = {}
        self._folder: Path | None = None
        self._meta: dict[str, Any] | None = None
        self._atlas_path: Path | None = None
        self._grid_cols = 0
        self._grid_rows = 0
        self._highlight: tuple[int, int] | None = None
        self._build()

    def _build(self) -> None:
        top = horizontal_paned(self)
        top.pack(fill=tk.BOTH, expand=True)

        atlas_col = ttk.Frame(top, padding=4)
        add_pane(top, atlas_col, weight=1, minsize=280)
        head = ttk.Frame(atlas_col)
        head.pack(fill=tk.X)
        ttk.Label(head, text="Packed atlas (UV grid)", font=FONT_UI_BOLD).pack(side=tk.LEFT)
        make_fidelity_chip(head, "ship").pack(side=tk.LEFT, padx=(8, 0))
        self._atlas_label = tk.Label(atlas_col, relief=tk.SUNKEN)
        self._atlas_label.pack(fill=tk.BOTH, expand=True, pady=4)
        configure_preview_label(
            self._atlas_label,
            "empty",
            detail="No packed tile sheet yet — run Pack atlas",
            width=self.ATLAS_MAX,
            height=self.ATLAS_MAX // 2,
        )
        self._atlas_path_var = tk.StringVar(value="")
        ttk.Label(atlas_col, textvariable=self._atlas_path_var, font=FONT_UI, foreground=COLOR_TEXT_SUBTLE).pack(
            anchor=tk.W
        )
        self._grid_legend_var = tk.StringVar(value="")
        ttk.Label(atlas_col, textvariable=self._grid_legend_var, font=FONT_SMALL, foreground=COLOR_MUTED).pack(
            anchor=tk.W
        )

        detail_col = ttk.Frame(top, padding=4)
        add_pane(top, detail_col, weight=1, minsize=240)
        ttk.Label(detail_col, text="Selected cell", font=FONT_UI_BOLD).pack(anchor=tk.W)
        self._cell_label = tk.Label(detail_col, relief=tk.SUNKEN)
        self._cell_label.pack(fill=tk.BOTH, expand=True, pady=4)
        configure_preview_label(
            self._cell_label,
            "empty",
            detail="Click a cell below",
            width=PREVIEW_THUMB_SM * 2,
            height=PREVIEW_THUMB_SM * 2,
        )
        self._cell_meta_var = tk.StringVar(value="")
        cell_meta_lbl = ttk.Label(detail_col, textvariable=self._cell_meta_var, font=FONT_UI, wraplength=280)
        cell_meta_lbl.pack(anchor=tk.W)
        track_wraplength(detail_col, cell_meta_lbl, minimum=200)

        ttk.Label(self, text="Source PNG cells", font=FONT_UI_BOLD).pack(anchor=tk.W, pady=(8, 2))
        cells_wrap = ttk.Frame(self)
        cells_wrap.pack(fill=tk.BOTH, expand=True)
        canvas = tk.Canvas(cells_wrap, highlightthickness=0, height=120)
        scroll_x = ttk.Scrollbar(cells_wrap, orient=tk.HORIZONTAL, command=canvas.xview)
        scroll_y = ttk.Scrollbar(cells_wrap, orient=tk.VERTICAL, command=canvas.yview)
        self._cells_inner = ttk.Frame(canvas)
        self._cells_win = canvas.create_window((0, 0), window=self._cells_inner, anchor=tk.NW)

        def _on_canvas_configure(event) -> None:
            canvas.itemconfigure(self._cells_win, width=max(event.width, 120))

        canvas.bind("<Configure>", _on_canvas_configure)
        bind_debounced_scrollregion(canvas, self._cells_inner)
        canvas.configure(xscrollcommand=scroll_x.set, yscrollcommand=scroll_y.set)
        canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll_y.pack(side=tk.RIGHT, fill=tk.Y)
        scroll_x.pack(side=tk.BOTTOM, fill=tk.X)
        attach_wheel_area(
            canvas,
            self._cells_inner,
            on_scroll_y=canvas_yscroll(canvas),
            on_scroll_x=canvas_xscroll(canvas),
            area_id=f"aps-atlas-cells-{id(self)}",
        )
        self._cells_canvas = canvas

        self._meta_var = tk.StringVar(value="atlas_meta: —")
        meta_lbl = ttk.Label(self, textvariable=self._meta_var, font=FONT_UI, foreground=COLOR_TEXT_BODY, wraplength=720)
        meta_lbl.pack(anchor=tk.W, pady=4)
        track_wraplength(self, meta_lbl, minimum=480)

    def load_folder(self, folder: str | Path | None) -> None:
        if not folder:
            self._folder = None
            self._clear()
            return
        path = Path(folder)
        if not path.is_dir():
            path = _resolve_path(str(folder))
        self._folder = path if path.is_dir() else None
        if not self._folder:
            self._clear()
            return
        configure_preview_label(
            self._atlas_label,
            "loading",
            detail="Loading atlas…",
            width=self.ATLAS_MAX,
            height=self.ATLAS_MAX // 2,
        )
        self._highlight = None
        self._refresh()
        self._on_log(f"atlas preview · {self._folder.name}")

    def _clear(self) -> None:
        configure_preview_label(
            self._atlas_label,
            "empty",
            detail="No packed tile sheet yet — run Pack atlas",
            width=self.ATLAS_MAX,
            height=self.ATLAS_MAX // 2,
        )
        configure_preview_label(
            self._cell_label,
            "empty",
            detail="Click a cell below",
            width=PREVIEW_THUMB_SM * 2,
            height=PREVIEW_THUMB_SM * 2,
        )
        self._atlas_path_var.set("")
        self._cell_meta_var.set("")
        self._meta_var.set("atlas_meta: —")
        self._grid_legend_var.set("")
        for w in self._cells_inner.winfo_children():
            w.destroy()

    def _refresh(self) -> None:
        assert self._folder is not None
        folder = self._folder
        for w in self._cells_inner.winfo_children():
            w.destroy()
        self._photos.clear()

        self._atlas_path = find_latest_atlas_in(folder)
        meta = _load_atlas_meta(folder)
        self._meta = meta
        self._grid_cols = int(meta.get("columns") or 0) if meta else 0
        self._grid_rows = int(meta.get("rows") or 0) if meta else 0

        if self._atlas_path and self._atlas_path.is_file():
            self._show_atlas_with_grid()
            self._atlas_path_var.set(self._atlas_path.name)
        else:
            configure_preview_label(
                self._atlas_label,
                "empty",
                detail="No tile_map yet — run Pack atlas",
                width=self.ATLAS_MAX,
                height=self.ATLAS_MAX // 2,
            )
            self._atlas_path_var.set("")

        tiles_by_png: dict[str, dict[str, Any]] = {}
        if meta:
            for row in meta.get("tiles") or []:
                if not isinstance(row, dict):
                    continue
                png = str(row.get("png") or "")
                if png:
                    tiles_by_png[_resolve_path(png).name] = row
            cols = meta.get("columns", "?")
            rows = meta.get("rows", "?")
            tid = meta.get("tile_id") or meta.get("atlas_id") or "?"
            n = len(meta.get("tiles") or [])
            self._meta_var.set(
                f"Atlas: {n} tiles · grid {cols}×{rows} · Next: register this atlas for the map"
            )
            if self._grid_cols and self._grid_rows:
                self._grid_legend_var.set(
                    f"UV overlay: {self._grid_cols}×{self._grid_rows} cells · "
                    "Legend: Grid lines = UV cells · Blue outline = selected cell"
                )
            elif meta:
                self._grid_legend_var.set("UV overlay unavailable — fix atlas_meta columns/rows")
            else:
                self._grid_legend_var.set("")
        else:
            self._meta_var.set("atlas_meta: (missing) — pack/register writes atlas_meta.json")
            self._grid_legend_var.set("")

        pngs = _source_pngs(folder)
        if not pngs:
            ttk.Label(self._cells_inner, text="○ No source PNGs in folder").pack(padx=8)
            return

        for i, png in enumerate(pngs):
            cell = ttk.Frame(self._cells_inner, padding=2)
            cell.grid(row=0, column=i, padx=2, pady=2)
            lbl = tk.Label(cell, bg=COLOR_INPUT_BG, cursor="hand2", relief=tk.RIDGE)
            lbl.pack()
            self._show_image(lbl, f"cell_{i}", png, max_size=self.THUMB)
            name = png.stem
            ttk.Label(cell, text=name[:14], font=FONT_SMALL).pack()
            row = tiles_by_png.get(png.name) or {}
            grid = row.get("grid")
            hint = f"grid {grid}" if grid else "no meta row"
            lbl.bind(
                "<Button-1>",
                lambda _e, p=png, h=hint, r=row: self._on_cell_click(p, h, r),
            )

        if pngs:
            self._on_cell_click(pngs[0], pngs[0].stem, tiles_by_png.get(pngs[0].name) or {})

    def _show_atlas_with_grid(self) -> None:
        assert self._atlas_path is not None
        try:
            img = Image.open(self._atlas_path).convert("RGB")
            img.thumbnail((self.ATLAS_MAX, self.ATLAS_MAX), Image.Resampling.LANCZOS)
            if self._grid_cols > 0 and self._grid_rows > 0:
                img = _draw_uv_grid(
                    img,
                    columns=self._grid_cols,
                    rows=self._grid_rows,
                    highlight=self._highlight,
                )
            photo = ImageTk.PhotoImage(img)
            self._photos["atlas"] = photo
            apply_preview_photo(self._atlas_label, photo)
        except Exception as exc:  # noqa: BLE001
            configure_preview_label(
                self._atlas_label,
                "error",
                detail="Atlas image unreadable",
                hint=str(self._atlas_path.name)[:24],
                width=self.ATLAS_MAX,
                height=self.ATLAS_MAX // 2,
            )
            self._on_log(f"atlas thumb fail: {exc}")

    def _on_cell_click(self, path: Path, hint: str, meta_row: dict[str, Any]) -> None:
        self._show_image(self._cell_label, "selected", path, max_size=240)
        uv = meta_row.get("uv")
        vk = meta_row.get("variant_key") or path.stem
        grid = meta_row.get("grid")
        parts = [f"variant: {vk}"]
        if grid:
            parts.append(f"grid: {grid}")
            if isinstance(grid, (list, tuple)) and len(grid) >= 2:
                try:
                    self._highlight = (int(grid[0]), int(grid[1]))
                    self._show_atlas_with_grid()
                except (TypeError, ValueError):
                    pass
        if uv:
            parts.append(f"uv: {uv}")
        self._cell_meta_var.set(" · ".join(parts))

    def _show_image(self, label: tk.Label, key: str, path: Path, *, max_size: int) -> None:
        show_image_file_thumbnail(
            label,
            path,
            max_size=max_size,
            on_log=self._on_log,
            photos_cache=self._photos,
            cache_key=key,
        )
