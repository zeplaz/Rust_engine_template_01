"""APS Voxel / Massing subsection — presentation-only height columns (DR-VOXEL-OCCUPATION).

Edits a massing JSON for Bevy ghost paint. Never writes FootprintMatrix / sim occupation.
"""

from __future__ import annotations

import json
from pathlib import Path
from tkinter import ttk, messagebox
from typing import Callable

from .aps_theme import FONT_UI_BOLD, GAP_MD


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def default_massing_path() -> Path:
    p = _repo_root() / "debug_runs" / "studio_dash" / "massing_preview.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    return p


class VoxelMassingPanel(ttk.Frame):
    """3×3 presentation massing grid; floors 1..4 per cell."""

    def __init__(
        self,
        master,
        *,
        state=None,
        on_log: Callable[[str], None] | None = None,
        **_kwargs,
    ) -> None:
        super().__init__(master)
        self._log = on_log or (lambda _m: None)
        self._seed = ttk.Spinbox(self, from_=0, to=2**31 - 1, width=12)
        self._seed.delete(0, "end")
        self._seed.insert(0, "1")
        self._cells: list[ttk.Spinbox] = []

        ttk.Label(self, text="Voxel / Massing (presentation)", font=FONT_UI_BOLD).pack(
            anchor="w", padx=GAP_MD, pady=(GAP_MD, 4)
        )
        ttk.Label(
            self,
            text="Preview ≠ gameplay. Columns are disposable. Sim occupation is DR-VOXEL-OCCUPATION.",
            wraplength=520,
        ).pack(anchor="w", padx=GAP_MD)

        seed_row = ttk.Frame(self, padding=GAP_MD)
        seed_row.pack(fill="x")
        ttk.Label(seed_row, text="Seed").pack(side="left")
        self._seed.pack(side="left", padx=8)

        grid = ttk.Frame(self, padding=GAP_MD)
        grid.pack()
        for y in range(3):
            for x in range(3):
                sp = ttk.Spinbox(grid, from_=0, to=4, width=4)
                sp.delete(0, "end")
                sp.insert(0, "1" if (x, y) != (1, 1) else "2")
                sp.grid(row=y, column=x, padx=4, pady=4)
                self._cells.append(sp)

        btns = ttk.Frame(self, padding=GAP_MD)
        btns.pack(fill="x")
        ttk.Button(btns, text="Export massing JSON", command=self._export).pack(side="left", padx=4)
        ttk.Button(btns, text="Load massing JSON", command=self._load).pack(side="left", padx=4)
        ttk.Button(btns, text="Enqueue massing_export task", command=self._enqueue).pack(
            side="left", padx=4
        )

    def _grid_values(self) -> list[dict]:
        cols = []
        for i, sp in enumerate(self._cells):
            floors = int(sp.get() or 0)
            if floors <= 0:
                continue
            x, y = i % 3, i // 3
            cols.append(
                {
                    "tile": [x - 1, y - 1],
                    "floors": min(4, max(1, floors)),
                    "color_kind": "Valid",
                    "weight": 1.0,
                }
            )
        return cols

    def _payload(self) -> dict:
        return {
            "schema_version": 1,
            "title": "massing_preview",
            "presentation_only": True,
            "sim_occupation": False,
            "seed": int(self._seed.get() or 0),
            "massing_columns": self._grid_values(),
            "note": "VOXEL-MASSING-PRES-001 — never commit to occupation",
        }

    def _export(self) -> None:
        path = default_massing_path()
        data = self._payload()
        path.write_text(json.dumps(data, indent=2), encoding="utf-8")
        self._log(f"Massing: exported {path} ({len(data['massing_columns'])} columns)")

    def _load(self) -> None:
        path = default_massing_path()
        if not path.is_file():
            messagebox.showinfo("Massing", f"No file at {path}")
            return
        data = json.loads(path.read_text(encoding="utf-8"))
        self._seed.delete(0, "end")
        self._seed.insert(0, str(data.get("seed", 1)))
        for sp in self._cells:
            sp.delete(0, "end")
            sp.insert(0, "0")
        for col in data.get("massing_columns") or []:
            tile = col.get("tile") or [0, 0]
            lx, ly = int(tile[0]) + 1, int(tile[1]) + 1
            if 0 <= lx < 3 and 0 <= ly < 3:
                idx = ly * 3 + lx
                self._cells[idx].delete(0, "end")
                self._cells[idx].insert(0, str(int(col.get("floors", 1))))
        self._log(f"Massing: loaded {path}")

    def _enqueue(self) -> None:
        from rust_engine_mcp import studio_task

        self._export()
        path = default_massing_path()
        task = studio_task.enqueue(
            kind="massing_export",
            seed=int(self._seed.get() or 0),
            payload_path=str(path.relative_to(_repo_root()))
            if path.is_relative_to(_repo_root())
            else str(path),
        )
        self._log(f"Massing: StudioTask {task['task_id']} queued")
