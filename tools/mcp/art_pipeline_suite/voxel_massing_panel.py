"""APS Voxel / Massing subsection — presentation-only height columns (DR-VOXEL-OCCUPATION).

Edits a massing JSON for Bevy ghost paint. Never writes FootprintMatrix / sim occupation.
"""

from __future__ import annotations

import json
from pathlib import Path
from tkinter import ttk, messagebox
from typing import Any, Callable

from .aps_theme import FONT_UI_BOLD, GAP_MD

_COLOR_KINDS = frozenset({"Valid", "Risky", "Invalid"})


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def default_massing_path() -> Path:
    p = _repo_root() / "debug_runs" / "studio_dash" / "massing_preview.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    return p


def default_param_frame_path() -> Path:
    return (
        _repo_root()
        / "tools"
        / "mcp"
        / "schemas"
        / "examples"
        / "param_frame_rail_warehouse_v1.json"
    )


def normalize_massing_columns(raw_columns: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Omit floors <= 0; clamp floors to 1..4; coerce tile/color/weight."""
    out: list[dict[str, Any]] = []
    for col in raw_columns:
        try:
            floors = int(col.get("floors", 0))
        except (TypeError, ValueError):
            continue
        if floors <= 0:
            continue
        floors = min(4, max(1, floors))
        tile = col.get("tile") or [0, 0]
        if not isinstance(tile, (list, tuple)) or len(tile) < 2:
            continue
        color = str(col.get("color_kind") or "Valid")
        if color not in _COLOR_KINDS:
            color = "Valid"
        try:
            weight = float(col.get("weight", 1.0))
        except (TypeError, ValueError):
            weight = 1.0
        weight = min(1.0, max(0.0, weight))
        out.append(
            {
                "tile": [int(tile[0]), int(tile[1])],
                "floors": floors,
                "color_kind": color,
                "weight": weight,
            }
        )
    return out


def build_massing_preview(
    *,
    seed: int,
    columns: list[dict[str, Any]],
    title: str = "massing_preview",
    pilot_id: str | None = None,
    grammar_rule_chain: dict[str, str] | None = None,
    arch_build_grammar_preset_id: str | None = None,
    material_strategy: str | None = None,
) -> dict[str, Any]:
    """Build a schema-ready presentation-only payload (floors already normalized)."""
    seed_i = max(0, int(seed))
    payload: dict[str, Any] = {
        "schema_version": 1,
        "title": title,
        "presentation_only": True,
        "sim_occupation": False,
        "seed": seed_i,
        "massing_columns": normalize_massing_columns(columns),
        "note": "VOXEL-MASSING-PRES-001 — never commit to occupation",
    }
    if pilot_id:
        payload["pilot_id"] = pilot_id
    if grammar_rule_chain:
        payload["grammar_rule_chain"] = dict(grammar_rule_chain)
    if arch_build_grammar_preset_id:
        payload["arch_build_grammar_preset_id"] = arch_build_grammar_preset_id
    if material_strategy:
        payload["material_strategy"] = material_strategy
    return payload


def export_massing_preview(path: Path, data: dict[str, Any]) -> dict[str, Any]:
    """Normalize columns, validate against massing_preview_v1, then write JSON."""
    from rust_engine_mcp.schemas import validate_massing_preview

    payload = dict(data)
    payload["massing_columns"] = normalize_massing_columns(
        list(payload.get("massing_columns") or [])
    )
    payload["presentation_only"] = True
    payload["sim_occupation"] = False
    payload["schema_version"] = 1
    payload["seed"] = max(0, int(payload.get("seed", 0)))
    validate_massing_preview(payload)
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return payload


# CQ-76: sole implementation lives in rust_engine_mcp.massing_qc.
from rust_engine_mcp.massing_qc import floors_histogram, massing_qc_view


class VoxelMassingPanel(ttk.Frame):
    """Pilot/grammar export + debug 3×3 grid; floors 1..4 per cell."""

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
        self._pilot_ids: list[str] = []
        try:
            from rust_engine_mcp.grammar_build_set import load_pilot_catalog

            self._pilot_ids = [str(p["id"]) for p in load_pilot_catalog()]
        except Exception:
            self._pilot_ids = ["shape_l_3x2", "logistics_rail_warehouse_v0"]

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
        ttk.Label(seed_row, text="Pilot").pack(side="left", padx=(16, 4))
        self._pilot = ttk.Combobox(
            seed_row,
            values=self._pilot_ids,
            width=36,
            state="readonly",
        )
        default_pilot = (
            "logistics_rail_warehouse_v0"
            if "logistics_rail_warehouse_v0" in self._pilot_ids
            else (self._pilot_ids[0] if self._pilot_ids else "")
        )
        if default_pilot:
            self._pilot.set(default_pilot)
        self._pilot.pack(side="left", padx=4)

        grid = ttk.Frame(self, padding=GAP_MD)
        grid.pack()
        ttk.Label(grid, text="Debug 3×3 (optional)").grid(row=0, column=0, columnspan=3, sticky="w")
        for y in range(3):
            for x in range(3):
                sp = ttk.Spinbox(grid, from_=0, to=4, width=4)
                sp.delete(0, "end")
                sp.insert(0, "1" if (x, y) != (1, 1) else "2")
                sp.grid(row=y + 1, column=x, padx=4, pady=4)
                self._cells.append(sp)

        btns = ttk.Frame(self, padding=GAP_MD)
        btns.pack(fill="x")
        ttk.Button(
            btns, text="Export from pilot/grammar", command=self._export_pilot
        ).pack(side="left", padx=4)
        ttk.Button(
            btns, text="Export from ParamFrame", command=self._export_param_frame
        ).pack(side="left", padx=4)
        ttk.Button(btns, text="Export 3×3 grid", command=self._export).pack(side="left", padx=4)
        ttk.Button(btns, text="Load massing JSON", command=self._load).pack(side="left", padx=4)
        ttk.Button(btns, text="Enqueue massing_export task", command=self._enqueue).pack(
            side="left", padx=4
        )
        ttk.Button(btns, text="Enqueue param_frame task", command=self._enqueue_param_frame).pack(
            side="left", padx=4
        )

    def _grid_values(self) -> list[dict]:
        cols = []
        for i, sp in enumerate(self._cells):
            try:
                floors = int(sp.get() or 0)
            except (TypeError, ValueError):
                floors = 0
            if floors <= 0:
                continue
            x, y = i % 3, i // 3
            cols.append(
                {
                    "tile": [x - 1, y - 1],
                    "floors": floors,
                    "color_kind": "Valid",
                    "weight": 1.0,
                }
            )
        return cols

    def _payload(self) -> dict:
        return build_massing_preview(
            seed=int(self._seed.get() or 0),
            columns=self._grid_values(),
        )

    def _export(self) -> None:
        path = default_massing_path()
        try:
            data = export_massing_preview(path, self._payload())
        except Exception as exc:  # jsonschema.ValidationError or coerce errors
            messagebox.showerror("Massing export", f"Validation failed:\n{exc}")
            self._log(f"Massing: export rejected — {exc}")
            return
        self._log(f"Massing: exported {path} ({len(data['massing_columns'])} columns)")

    def _export_param_frame(self) -> None:
        from rust_engine_mcp.massing_from_param_frame import export_massing_preview_from_param_frame

        frame_path = default_param_frame_path()
        if not frame_path.is_file():
            messagebox.showerror("Massing export", f"ParamFrame missing:\n{frame_path}")
            return
        path = default_massing_path()
        try:
            data = export_massing_preview_from_param_frame(path, frame_path)
        except Exception as exc:
            messagebox.showerror("Massing export", f"ParamFrame export failed:\n{exc}")
            self._log(f"Massing: ParamFrame export rejected — {exc}")
            return
        self._seed.delete(0, "end")
        self._seed.insert(0, str(data.get("seed", 0)))
        pid = data.get("pilot_id")
        if pid and pid in self._pilot_ids:
            self._pilot.set(pid)
        self._log(
            f"Massing: ParamFrame → {path} ({len(data['massing_columns'])} columns, "
            f"frame={data.get('param_frame_id')})"
        )

    def _export_pilot(self) -> None:
        from rust_engine_mcp.massing_from_pilot import export_massing_preview_from_pilot

        pilot_id = (self._pilot.get() or "").strip()
        if not pilot_id:
            messagebox.showerror("Massing export", "Select a catalog pilot.")
            return
        path = default_massing_path()
        try:
            data = export_massing_preview_from_pilot(
                path,
                pilot_id,
                seed=int(self._seed.get() or 0),
            )
        except Exception as exc:
            messagebox.showerror("Massing export", f"Pilot export failed:\n{exc}")
            self._log(f"Massing: pilot export rejected — {exc}")
            return
        self._log(
            f"Massing: exported pilot {pilot_id} → {path} "
            f"({len(data['massing_columns'])} columns)"
        )

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
                floors = int(col.get("floors", 1))
                if floors <= 0:
                    continue
                self._cells[idx].delete(0, "end")
                self._cells[idx].insert(0, str(min(4, max(1, floors))))
        self._log(f"Massing: loaded {path}")

    def _enqueue(self) -> None:
        from rust_engine_mcp import studio_task

        pilot_id = (self._pilot.get() or "").strip()
        if pilot_id:
            self._export_pilot()
        else:
            self._export()
        path = default_massing_path()
        if not path.is_file():
            return
        task = studio_task.enqueue(
            kind="massing_export",
            seed=int(self._seed.get() or 0),
            payload_path=str(path.relative_to(_repo_root()))
            if path.is_relative_to(_repo_root())
            else str(path),
        )
        self._log(f"Massing: StudioTask {task['task_id']} queued")

    def _enqueue_param_frame(self) -> None:
        from rust_engine_mcp import studio_task

        frame_path = default_param_frame_path()
        if not frame_path.is_file():
            messagebox.showerror("Massing", f"ParamFrame missing:\n{frame_path}")
            return
        self._export_param_frame()
        root = _repo_root()
        rel = (
            str(frame_path.relative_to(root)).replace("\\", "/")
            if frame_path.is_relative_to(root)
            else str(frame_path)
        )
        task = studio_task.enqueue(
            kind="param_frame",
            seed=int(self._seed.get() or 0),
            payload_path=rel,
        )
        self._log(f"Massing: StudioTask param_frame {task['task_id']} queued")
