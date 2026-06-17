"""GRAMMAR-ITER-001 — Iterate grammar panel for Assembly tab."""

from __future__ import annotations

import json
import tkinter as tk
from tkinter import ttk
from typing import Any, Callable

from rust_engine_mcp import building_grammar
from rust_engine_mcp.aps_grammar_labels import human_label
from rust_engine_mcp.grammar_iterate import compute_cell_diff_map, iterate_grammar
from rust_engine_mcp.paths import repo_root

from .aps_theme import FONT_SMALL


def _ui_labels() -> dict[str, str]:
    path = repo_root() / "assets/configs/buildings/grammars/grammar_labels_v1.json"
    if not path.is_file():
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    ui = data.get("ui") if isinstance(data.get("ui"), dict) else {}
    modes = data.get("iteration_modes") if isinstance(data.get("iteration_modes"), dict) else {}
    return {**{str(k): str(v) for k, v in ui.items()}, **{f"mode.{k}": str(v) for k, v in modes.items()}}


class GrammarIteratePanel(ttk.LabelFrame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        on_applied: Callable[[dict[str, Any], dict[str, Any], dict[str, Any]], None] | None = None,
        on_log: Callable[[str], None] | None = None,
    ) -> None:
        ui = _ui_labels()
        super().__init__(master, text=ui.get("panel_title", "Iterate grammar"), padding=6)
        self._on_applied = on_applied
        self._on_log = on_log or (lambda _m: None)
        self._before_snapshot: dict[str, Any] | None = None

        top = ttk.Frame(self)
        top.pack(fill=tk.X, pady=2)
        ttk.Label(top, text="Mode").pack(side=tk.LEFT)
        self.mode_var = tk.StringVar(value="massing")
        mode_values = ["massing", "material_strategy", "placement", "full"]
        self.mode_combo = ttk.Combobox(
            top, textvariable=self.mode_var, width=18, values=mode_values, state="readonly"
        )
        self.mode_combo.pack(side=tk.LEFT, padx=4)
        self.mode_combo.bind("<<ComboboxSelected>>", self._on_mode_change)
        ttk.Label(top, text="Seed").pack(side=tk.LEFT, padx=(8, 0))
        self.seed_var = tk.IntVar(value=43)
        ttk.Spinbox(top, from_=0, to=999999, textvariable=self.seed_var, width=8).pack(side=tk.LEFT, padx=4)
        apply_btn = ttk.Button(top, text=ui.get("apply_iteration", "Apply iteration"), command=self._on_apply)
        apply_btn.pack(side=tk.LEFT, padx=8)
        from .aps_tooltips import bind_aps_tooltip

        bind_aps_tooltip(apply_btn, "asm_iterate")
        bind_aps_tooltip(self.mode_combo, "asm_iterate")

        self.massing_frame = ttk.Frame(self)
        self.massing_frame.pack(fill=tk.X, pady=4)
        ttk.Label(self.massing_frame, text="Massing strategy").grid(row=0, column=0, sticky=tk.W)
        self.strategy_var = tk.StringVar(value="double_hall")
        strategies = ["long_hall", "double_hall", "l_shape", "yard_complex"]
        for i, sid in enumerate(strategies):
            ttk.Radiobutton(
                self.massing_frame,
                text=human_label(sid),
                variable=self.strategy_var,
                value=sid,
            ).grid(row=1 + i // 2, column=i % 2, sticky=tk.W, padx=4)
        fp = ttk.Frame(self.massing_frame)
        fp.grid(row=3, column=0, columnspan=2, sticky=tk.W, pady=4)
        ttk.Label(fp, text="Footprint W×D").pack(side=tk.LEFT)
        self.width_var = tk.IntVar(value=10)
        self.depth_var = tk.IntVar(value=6)
        self.iter_floors_var = tk.IntVar(value=2)
        ttk.Spinbox(fp, from_=2, to=32, textvariable=self.width_var, width=4).pack(side=tk.LEFT, padx=2)
        ttk.Label(fp, text="×").pack(side=tk.LEFT)
        ttk.Spinbox(fp, from_=2, to=32, textvariable=self.depth_var, width=4).pack(side=tk.LEFT, padx=2)
        ttk.Label(fp, text="Floors").pack(side=tk.LEFT, padx=(8, 0))
        ttk.Spinbox(fp, from_=1, to=8, textvariable=self.iter_floors_var, width=4).pack(side=tk.LEFT, padx=2)

        pin_row = ttk.Frame(self)
        pin_row.pack(fill=tk.X, pady=2)
        self.pin_district_var = tk.BooleanVar(value=True)
        self.pin_age_var = tk.BooleanVar(value=False)
        ttk.Checkbutton(
            pin_row, text=ui.get("pin_district_style", "Pin district style"), variable=self.pin_district_var
        ).pack(side=tk.LEFT, padx=4)
        ttk.Checkbutton(
            pin_row, text=ui.get("pin_age_band", "Pin age band"), variable=self.pin_age_var
        ).pack(side=tk.LEFT, padx=4)

        self.diff_var = tk.StringVar(value="")
        self._diff_lbl = ttk.Label(self, textvariable=self.diff_var, foreground="#444", wraplength=520)
        self._diff_lbl.pack(anchor=tk.W, pady=4)
        ttk.Label(
            self,
            text=ui.get("generate_vs_iterate_hint", ""),
            foreground="#666",
            font=FONT_SMALL,
            wraplength=520,
        ).pack(anchor=tk.W)

        self.material_hint = ttk.Label(
            self,
            text=ui.get("open_materials_tab", "Open Materials tab"),
            foreground="#0a4a7a",
        )
        self.placement_hint = ttk.Label(
            self,
            text=ui.get("placement_defer_hint", ""),
            foreground="#666",
            wraplength=520,
        )
        self._on_mode_change()

    def set_base_snapshot(self, snapshot: dict[str, Any] | None) -> None:
        self._before_snapshot = dict(snapshot) if snapshot else None
        if snapshot and snapshot.get("seed") is not None:
            self.seed_var.set(int(snapshot["seed"]))
        fp = (snapshot or {}).get("footprint") or {}
        if fp.get("width"):
            self.width_var.set(int(fp["width"]))
        if fp.get("depth"):
            self.depth_var.set(int(fp["depth"]))
        if fp.get("floors"):
            self.iter_floors_var.set(int(fp["floors"]))
        chain = (snapshot or {}).get("grammar_rule_chain") or {}
        if isinstance(chain, dict) and chain.get("massing"):
            self.strategy_var.set(str(chain["massing"]))

    def _on_mode_change(self, _event=None) -> None:
        mode = self.mode_var.get()
        if mode == "massing" or mode == "full":
            self.massing_frame.pack(fill=tk.X, pady=4)
            self.material_hint.pack_forget()
            self.placement_hint.pack_forget()
        elif mode == "material_strategy":
            self.massing_frame.pack_forget()
            self.material_hint.pack(anchor=tk.W, pady=4)
            self.placement_hint.pack_forget()
        elif mode == "placement":
            self.massing_frame.pack_forget()
            self.material_hint.pack_forget()
            self.placement_hint.pack(anchor=tk.W, pady=4)
        else:
            self.massing_frame.pack(fill=tk.X, pady=4)
            self.material_hint.pack_forget()
            self.placement_hint.pack_forget()

    def _build_request(self, base: dict[str, Any]) -> dict[str, Any]:
        mode = self.mode_var.get()
        preserve: list[str] = []
        if self.pin_district_var.get():
            preserve.append("district_style")
        if self.pin_age_var.get():
            preserve.append("age")
        req: dict[str, Any] = {
            "schema": "grammar_iterate_request_v1",
            "mode": mode,
            "seed": int(self.seed_var.get()),
            "archetype_id": str(base.get("archetype_id") or "IndustrialWarehouse"),
            "district_style": str(base.get("district_style") or "industrial_west"),
            "base_snapshot": base,
            "preserve_layers": preserve,
            "parent_lineage_id": str(base.get("assembly_id") or ""),
        }
        if mode in ("massing", "full"):
            req["overrides"] = {
                "massing_strategy": self.strategy_var.get(),
                "footprint": {
                    "width": int(self.width_var.get()),
                    "depth": int(self.depth_var.get()),
                    "floors": int(self.iter_floors_var.get()),
                },
            }
        elif mode == "material_strategy":
            req["overrides"] = {"district_material_profiles": {}}
        elif mode == "placement":
            req["overrides"] = {}
        return req

    def _set_status(self, text: str, *, ok: bool | None = None) -> None:
        self.diff_var.set(text)
        if ok is True:
            fg = "#006400"
        elif ok is False:
            fg = "#8b0000"
        else:
            fg = "#444444"
        self._diff_lbl.configure(foreground=fg)

    def _on_apply(self) -> None:
        if not self._before_snapshot:
            self._set_status("Load or generate a snapshot first.", ok=False)
            return
        req = self._build_request(self._before_snapshot)
        self._on_log(f"grammar-iterate mode={req['mode']} seed={req['seed']}")
        result = iterate_grammar(req)
        if not result.get("ok"):
            err = result.get("errors") or [{}]
            msg = str(err[0].get("message") if err else result)
            self._set_status(f"Iterate failed: {msg}", ok=False)
            self._on_log(f"grammar-iterate error: {msg}")
            return
        after = result["snapshot"]
        diff = result.get("diff") or {}
        ui = _ui_labels()
        tpl = ui.get("last_change_template", "+{added} −{removed} ~{changed}")
        self._set_status(
            f"Last diff: {tpl.format(added=diff.get('cells_added', 0), removed=diff.get('cells_removed', 0), changed=diff.get('cells_changed', 0))} · "
            f"{ui.get('layers_prefix', 'layers')}: {', '.join(diff.get('layers_touched') or [])}",
            ok=True,
        )
        diff_map = compute_cell_diff_map(self._before_snapshot, after)
        removed = [k for k, v in diff_map.items() if v == "removed"]
        if self._on_applied:
            self._on_applied(self._before_snapshot, after, result)
        self._before_snapshot = after

    def massing_strategies_for_archetype(self, archetype_id: str) -> list[str]:
        try:
            grammar = building_grammar.load_building_grammar_by_archetype(archetype_id)
            return [str(s["id"]) for s in grammar.get("massing", {}).get("strategies") or []]
        except (KeyError, FileNotFoundError, NotImplementedError):
            return ["long_hall", "double_hall"]
