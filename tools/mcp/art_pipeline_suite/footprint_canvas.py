"""APS-UI-003b — footprint grid heatmap (W/D/C/R/Y tokens)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

TOKEN_COLORS = {
    "W": "#4a90d9",
    "D": "#6bbf59",
    "C": "#e6b422",
    "R": "#c45c5c",
    "Y": "#9a9a9a",
}

TOKEN_LABELS = {
    "W": "Wall",
    "D": "Door",
    "C": "Corner",
    "R": "Roof",
    "Y": "Yard / empty",
}

# GRAMMAR-ITER-001 footprint diff legend (designer wireframe)
DIFF_COLORS = {
    "added": "#6bbf59",
    "removed": "#c45c5c",
    "changed": "#e6b422",
}

DIFF_LABELS = {
    "added": "Added",
    "removed": "Removed",
    "changed": "Changed",
}


class FootprintCanvas(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        cell_px: int = 28,
        on_cell_select: Callable[[int, int, int], None] | None = None,
    ) -> None:
        super().__init__(master)
        self._cell_px = cell_px
        self._on_cell_select = on_cell_select
        self._cells: list[dict[str, Any]] = []
        self._placements: list[dict[str, Any]] = []
        self._floor = 0
        self._selected: tuple[int, int, int] | None = None
        self._cell_diff: dict[tuple[int, int, int], str] = {}
        self._removed_ghosts: list[tuple[int, int, int]] = []

        header = ttk.Frame(self)
        header.pack(fill=tk.X)
        ttk.Label(header, text="Footprint grid (plan view)").pack(side=tk.LEFT)
        ttk.Label(header, text="Floor").pack(side=tk.LEFT, padx=(12, 0))
        self.floor_var = tk.IntVar(value=0)
        self.floor_spin = ttk.Spinbox(
            header, from_=0, to=8, textvariable=self.floor_var, width=4, command=self.redraw
        )
        self.floor_spin.pack(side=tk.LEFT, padx=4)

        self._selection_var = tk.StringVar(value="Click a cell to select a placement")
        ttk.Label(header, textvariable=self._selection_var, foreground="#444").pack(
            side=tk.RIGHT, padx=4
        )

        workspace = ttk.Frame(self)
        workspace.pack(fill=tk.BOTH, expand=True, pady=4)

        canvas_wrap = ttk.Frame(workspace)
        canvas_wrap.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self.canvas = tk.Canvas(
            canvas_wrap,
            width=280,
            height=200,
            bg="#f8f8f8",
            highlightthickness=1,
            highlightbackground="#c8c8c8",
        )
        self.canvas.pack(anchor=tk.NW)
        self.canvas.bind("<Button-1>", self._on_click)

        legend = ttk.LabelFrame(workspace, text="Cell tokens", padding=6)
        legend.pack(side=tk.LEFT, fill=tk.Y, padx=(10, 0))
        for token in ("W", "D", "C", "R", "Y"):
            row = ttk.Frame(legend)
            row.pack(anchor=tk.W, pady=2)
            color = TOKEN_COLORS[token]
            swatch = tk.Label(
                row,
                text="  ",
                bg=color,
                width=2,
                relief=tk.RIDGE,
                borderwidth=1,
            )
            swatch.pack(side=tk.LEFT, padx=(0, 6))
            ttk.Label(
                row,
                text=f"{token} — {TOKEN_LABELS[token]}",
                font=("Segoe UI", 9),
            ).pack(side=tk.LEFT)

        diff_legend = ttk.LabelFrame(workspace, text="Iteration diff", padding=6)
        diff_legend.pack(side=tk.LEFT, fill=tk.Y, padx=(10, 0))
        for state in ("added", "removed", "changed"):
            row = ttk.Frame(diff_legend)
            row.pack(anchor=tk.W, pady=2)
            swatch = tk.Label(
                row,
                text="  ",
                bg=DIFF_COLORS[state],
                width=2,
                relief=tk.RIDGE,
                borderwidth=1,
            )
            swatch.pack(side=tk.LEFT, padx=(0, 6))
            ttk.Label(row, text=DIFF_LABELS[state], font=("Segoe UI", 9)).pack(side=tk.LEFT)

        ttk.Label(
            self,
            text="Colored squares match placement list tokens. Selected cell has a thick black outline.",
            foreground="#666",
            font=("Segoe UI", 8),
            wraplength=420,
        ).pack(anchor=tk.W, pady=(2, 0))

    def set_cells(
        self,
        cells: list[dict[str, Any]],
        placements: list[dict[str, Any]] | None = None,
        *,
        floor: int | None = None,
    ) -> None:
        self._cells = list(cells)
        self._placements = list(placements or [])
        if floor is not None:
            self._floor = floor
            self.floor_var.set(floor)
        if self._cells:
            max_floor = max(int(c.get("floor") or 0) for c in self._cells)
            self.floor_spin.configure(to=max_floor)
        self.redraw()

    def set_cell_diff(
        self,
        diff_map: dict[tuple[int, int, int], str] | None,
        *,
        removed_ghosts: list[tuple[int, int, int]] | None = None,
    ) -> None:
        """Highlight added/removed/changed cells after grammar iteration."""
        self._cell_diff = dict(diff_map or {})
        self._removed_ghosts = list(removed_ghosts or [])
        self.redraw()

    def clear_cell_diff(self) -> None:
        self._cell_diff = {}
        self._removed_ghosts = []
        self.redraw()

    def set_selection(self, grid_x: int, grid_y: int, floor: int) -> None:
        self._selected = (grid_x, grid_y, floor)
        self._floor = floor
        self.floor_var.set(floor)
        token = self._token_at(grid_x, grid_y, floor)
        label = TOKEN_LABELS.get(token, token)
        self._selection_var.set(f"Selected: floor {floor} · cell ({grid_x},{grid_y}) · {token} {label}")
        self.redraw()

    def _token_at(self, gx: int, gy: int, floor: int) -> str:
        for p in self._placements:
            if (
                int(p.get("floor") or 0) == floor
                and int(p.get("grid_x") or 0) == gx
                and int(p.get("grid_y") or 0) == gy
            ):
                return str(p.get("token") or "W")
        for cell in self._cells:
            if (
                int(cell.get("floor") or 0) == floor
                and int(cell["x"]) == gx
                and int(cell["y"]) == gy
            ):
                return str(cell.get("token") or "W")
        return "?"

    def redraw(self, _event=None) -> None:
        self._floor = int(self.floor_var.get())
        self.canvas.delete("all")
        if not self._cells:
            self.canvas.create_text(12, 12, anchor=tk.NW, text="Generate snapshot to show grid")
            self.canvas.configure(width=280, height=120)
            return
        width, depth = self._footprint_dims()
        if not width or not depth:
            return
        px = self._cell_px
        heat: dict[tuple[int, int], str] = {}
        for p in self._placements:
            if int(p.get("floor") or 0) != self._floor:
                continue
            gx, gy = int(p.get("grid_x") or 0), int(p.get("grid_y") or 0)
            heat[(gx, gy)] = str(p.get("token") or "W")
        for cell in self._cells:
            if int(cell.get("floor") or 0) != self._floor:
                continue
            x, y = int(cell["x"]), int(cell["y"])
            token = str(cell.get("token") or "W")
            color = TOKEN_COLORS.get(token, "#cccccc")
            if (x, y) in heat:
                color = TOKEN_COLORS.get(heat[(x, y)], color)
            key = (self._floor, x, y)
            diff_state = self._cell_diff.get(key)
            if diff_state:
                color = DIFF_COLORS.get(diff_state, color)
            x0, y0 = x * px + 6, y * px + 6
            x1, y1 = x0 + px - 3, y0 + px - 3
            selected = self._selected == (x, y, self._floor)
            outline = "#1a1a1a" if selected else "#555555"
            width_line = 3 if selected else 1
            self.canvas.create_rectangle(x0, y0, x1, y1, fill=color, outline=outline, width=width_line)
            if px >= 24:
                self.canvas.create_text(
                    (x0 + x1) // 2,
                    (y0 + y1) // 2,
                    text=token,
                    fill="white" if token != "Y" else "#222222",
                    font=("Consolas", 8, "bold"),
                )
        for key in self._removed_ghosts:
            if key[0] != self._floor:
                continue
            _, x, y = key
            x0, y0 = x * px + 6, y * px + 6
            x1, y1 = x0 + px - 3, y0 + px - 3
            self.canvas.create_rectangle(
                x0,
                y0,
                x1,
                y1,
                fill="",
                outline=DIFF_COLORS["removed"],
                width=2,
                dash=(3, 2),
            )
        pad = 12
        self.canvas.configure(width=width * px + pad, height=depth * px + pad)

    def _footprint_dims(self) -> tuple[int, int]:
        if not self._cells:
            return 0, 0
        max_x = max(int(c["x"]) for c in self._cells)
        max_y = max(int(c["y"]) for c in self._cells)
        return max_x + 1, max_y + 1

    def _on_click(self, event: tk.Event) -> None:
        if not self._cells or not self._on_cell_select:
            return
        px = self._cell_px
        gx = max(0, (event.x - 6) // px)
        gy = max(0, (event.y - 6) // px)
        self.set_selection(gx, gy, self._floor)
        self._on_cell_select(gx, gy, self._floor)
