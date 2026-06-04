"""Variants workspace — variant_set_v1 layers, tags, agent patch."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

from rust_engine_mcp import variant_set
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import validate_variant_set

from .state import SuiteState


class VariantsPanel(ttk.Frame):
    def __init__(self, master: tk.Misc, state: SuiteState, *, on_log) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        self._data: dict | None = None
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="variant_set_v1 — declarative layers (lighting, damage, material, fill). "
            "Bake via MCP variant_bake / tile_batch_run — no manual Blender.",
            wraplength=720,
            justify=tk.LEFT,
        ).pack(anchor=tk.W, pady=(0, 8))

        top = ttk.Frame(self)
        top.pack(fill=tk.X, pady=4)
        ttk.Button(top, text="Load…", command=self.on_load).pack(side=tk.LEFT, padx=2)
        ttk.Button(top, text="Load example", command=self.on_load_example).pack(side=tk.LEFT, padx=2)
        ttk.Button(top, text="New from assembly", command=self.on_new_from_assembly).pack(side=tk.LEFT, padx=2)
        ttk.Button(top, text="Save JSON", command=lambda: self.on_save(ext=".json")).pack(side=tk.LEFT, padx=2)
        ttk.Button(top, text="Save RON", command=lambda: self.on_save(ext=".ron")).pack(side=tk.LEFT, padx=2)
        ttk.Button(top, text="Validate", command=self.on_validate).pack(side=tk.LEFT, padx=2)

        self.path_var = tk.StringVar(value="(none)")
        ttk.Label(top, textvariable=self.path_var, foreground="#444").pack(side=tk.LEFT, padx=8)

        paned = ttk.Panedwindow(self, orient=tk.HORIZONTAL)
        paned.pack(fill=tk.BOTH, expand=True, pady=8)

        left = ttk.Frame(paned, padding=4)
        paned.add(left, weight=1)
        ttk.Label(left, text="Variants").pack(anchor=tk.W)
        self.variant_list = tk.Listbox(left, exportselection=False)
        self.variant_list.pack(fill=tk.BOTH, expand=True)
        self.variant_list.bind("<<ListboxSelect>>", self.on_variant_select)

        right = ttk.Frame(paned, padding=4)
        paned.add(right, weight=2)

        layer_row = ttk.LabelFrame(right, text="Layers", padding=6)
        layer_row.pack(fill=tk.X, pady=4)

        ttk.Label(layer_row, text="Lighting").grid(row=0, column=0, sticky=tk.W)
        self.lighting_var = tk.StringVar(value="day")
        ttk.Combobox(
            layer_row,
            textvariable=self.lighting_var,
            width=12,
            values=["day", "night_off", "night_on"],
        ).grid(row=0, column=1, padx=4)
        ttk.Label(layer_row, text="Power").grid(row=0, column=2, sticky=tk.W)
        self.power_var = tk.StringVar(value="off")
        ttk.Combobox(
            layer_row, textvariable=self.power_var, width=10, values=["off", "partial", "on"]
        ).grid(row=0, column=3, padx=4)
        self.night_lights_var = tk.BooleanVar(value=False)
        ttk.Checkbutton(layer_row, text="night_lights", variable=self.night_lights_var).grid(
            row=1, column=1, sticky=tk.W
        )

        ttk.Label(layer_row, text="Damage state").grid(row=2, column=0, sticky=tk.W)
        self.damage_state_var = tk.StringVar(value="clean")
        ttk.Combobox(
            layer_row,
            textvariable=self.damage_state_var,
            width=12,
            values=["clean", "dirty", "damaged", "ruined"],
        ).grid(row=2, column=1, padx=4)
        ttk.Label(layer_row, text="damage").grid(row=2, column=2, sticky=tk.W)
        self.damage_val_var = tk.DoubleVar(value=0.0)
        ttk.Scale(layer_row, from_=0, to=1, variable=self.damage_val_var, orient=tk.HORIZONTAL).grid(
            row=2, column=3, sticky=tk.EW, padx=4
        )

        ttk.Label(layer_row, text="Fill").grid(row=3, column=0, sticky=tk.W)
        self.fill_var = tk.StringVar(value="empty")
        ttk.Combobox(
            layer_row, textvariable=self.fill_var, width=12, values=["empty", "quarter", "half", "full"]
        ).grid(row=3, column=1, padx=4)

        ttk.Label(layer_row, text="Material").grid(row=4, column=0, sticky=tk.W)
        self.material_var = tk.StringVar(value="")
        ttk.Entry(layer_row, textvariable=self.material_var, width=28).grid(
            row=4, column=1, columnspan=3, sticky=tk.EW, padx=4
        )

        ttk.Label(layer_row, text="Tags (comma)").grid(row=5, column=0, sticky=tk.W)
        self.tags_var = tk.StringVar(value="")
        ttk.Entry(layer_row, textvariable=self.tags_var, width=40).grid(
            row=5, column=1, columnspan=3, sticky=tk.EW, padx=4
        )

        ttk.Button(layer_row, text="Apply layers to selected", command=self.on_apply_layers).grid(
            row=6, column=0, columnspan=4, pady=6, sticky=tk.W
        )

        agent_row = ttk.LabelFrame(right, text="Agent patch strip", padding=6)
        agent_row.pack(fill=tk.X, pady=8)
        ttk.Label(agent_row, text="Intent").pack(anchor=tk.W)
        self.intent_var = tk.StringVar(value="add_warm_window_lights")
        ttk.Entry(agent_row, textvariable=self.intent_var, width=48).pack(fill=tk.X, pady=2)
        btn_row = ttk.Frame(agent_row)
        btn_row.pack(fill=tk.X, pady=4)
        ttk.Button(btn_row, text="Request agent", command=self.on_request_agent).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Apply patch", command=self.on_apply_patch).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Bake selected", command=self.on_bake_selected).pack(side=tk.LEFT, padx=2)

        self.patch_text = tk.Text(agent_row, height=8, wrap=tk.WORD, font=("Consolas", 9))
        self.patch_text.pack(fill=tk.BOTH, expand=True, pady=4)

        self.bake_status = tk.StringVar(value="")
        ttk.Label(right, textvariable=self.bake_status, foreground="#006400").pack(anchor=tk.W)

    def _selected_index(self) -> int | None:
        sel = self.variant_list.curselection()
        if not sel or not self._data:
            return None
        return int(sel[0])

    def _refresh_list(self) -> None:
        self.variant_list.delete(0, tk.END)
        if not self._data:
            return
        for entry in self._data.get("variants") or []:
            key = entry.get("variant_key", "?")
            bake = (entry.get("bake") or {}).get("status")
            suffix = f" [{bake}]" if bake else ""
            self.variant_list.insert(tk.END, f"{key}{suffix}")

    def _load_data(self, data: dict, path: str | None) -> None:
        validate_variant_set(data)
        self._data = data
        self.state.variant_set_data = data
        self.state.variant_set_path = path
        self.path_var.set(path or "(memory)")
        self._refresh_list()
        if self._data.get("variants"):
            self.variant_list.selection_set(0)
            self.on_variant_select()

    def on_load(self) -> None:
        path = filedialog.askopenfilename(
            title="variant_set_v1",
            filetypes=[("Variant set", "*.json *.ron"), ("All", "*.*")],
        )
        if not path:
            return
        try:
            data = variant_set.load_variant_set(path)
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Variants", str(exc))
            return
        self._load_data(data, path)

    def on_load_example(self) -> None:
        path = variant_set.example_variant_set_path()
        if not path.is_file():
            messagebox.showerror("Variants", f"Missing example: {path}")
            return
        data = variant_set.load_variant_set(path)
        self._load_data(data, str(path))

    def on_new_from_assembly(self) -> None:
        aid = self.state.assembly_id
        if not aid:
            messagebox.showinfo("Variants", "Generate an assembly snapshot first (Assembly tab).")
            return
        vsid = f"{aid.replace('-', '_')}_variants"[:64]
        data = {
            "schema_version": 1,
            "variant_set_id": vsid,
            "assembly_id": aid,
            "style_pack_id": self.state.style_pack_id,
            "seed": self.state.seed,
            "axes": {
                "state": ["clean", "dirty", "damaged", "ruined"],
                "power": ["off", "partial", "on"],
                "fill": ["empty", "half", "full"],
                "lighting": ["day", "night_off", "night_on"],
            },
            "variants": [
                {
                    "variant_key": "clean_day",
                    "tags": ["default", f"stylepack_{self.state.style_pack_id.removeprefix('style_')}"],
                    "layers": {
                        "lighting": {"lighting": "day", "power": "off"},
                        "damage": {"state": "clean", "damage": 0.0},
                        "fill": {"fill": "empty"},
                    },
                }
            ],
        }
        out = variant_set.save_variant_set(data)
        self._load_data(data, str(out))
        self._on_log(f"new variant_set {out}")

    def on_save(self, *, ext: str) -> None:
        if not self._data:
            messagebox.showinfo("Variants", "Nothing to save.")
            return
        path = self.state.variant_set_path
        if not path or not path.endswith(ext):
            path = str(variant_set.default_variant_set_path(self._data["variant_set_id"], ext=ext))
        out = variant_set.save_variant_set(self._data, path)
        self.state.variant_set_path = str(out)
        self.path_var.set(str(out))
        self._on_log(f"saved {out}")
        messagebox.showinfo("Variants", f"Saved:\n{out}")

    def on_validate(self) -> None:
        if not self._data:
            messagebox.showinfo("Variants", "Load a variant set first.")
            return
        try:
            validate_variant_set(self._data)
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Validate", str(exc))
            return
        messagebox.showinfo("Validate", "variant_set_v1 OK")

    def on_variant_select(self, _event=None) -> None:
        idx = self._selected_index()
        if idx is None or not self._data:
            return
        entry = self._data["variants"][idx]
        self.state.selected_variant_key = str(entry.get("variant_key"))
        layers = entry.get("layers") or {}
        lighting = layers.get("lighting") or {}
        damage = layers.get("damage") or {}
        fill = layers.get("fill") or {}
        material = layers.get("material") or {}
        self.lighting_var.set(str(lighting.get("lighting") or "day"))
        self.power_var.set(str(lighting.get("power") or "off"))
        self.night_lights_var.set(bool(lighting.get("night_lights")))
        self.damage_state_var.set(str(damage.get("state") or "clean"))
        self.damage_val_var.set(float(damage.get("damage") or 0.0))
        self.fill_var.set(str(fill.get("fill") or "empty"))
        mat = material.get("wall_material") or ""
        self.material_var.set(str(mat))
        tags = entry.get("tags") or []
        self.tags_var.set(", ".join(tags))
        bake = entry.get("bake") or {}
        self.bake_status.set(
            f"bake: {bake.get('status', 'pending')} · {bake.get('png') or '—'}"
        )

    def on_apply_layers(self) -> None:
        idx = self._selected_index()
        if idx is None or not self._data:
            messagebox.showinfo("Variants", "Select a variant row.")
            return
        layers: dict = {
            "lighting": {
                "lighting": self.lighting_var.get(),
                "power": self.power_var.get(),
                "night_lights": self.night_lights_var.get(),
            },
            "damage": {
                "state": self.damage_state_var.get(),
                "damage": round(float(self.damage_val_var.get()), 3),
            },
            "fill": {"fill": self.fill_var.get()},
        }
        mat = self.material_var.get().strip()
        if mat:
            layers["material"] = {"wall_material": mat}
        tags = [t.strip() for t in self.tags_var.get().split(",") if t.strip()]
        self._data["variants"][idx]["layers"] = layers
        if tags:
            self._data["variants"][idx]["tags"] = tags
        self._refresh_list()
        self.variant_list.selection_set(idx)
        self._on_log(f"layers updated {self._data['variants'][idx]['variant_key']}")

    def on_request_agent(self) -> None:
        if not self._data:
            messagebox.showinfo("Variants", "Load a variant set first.")
            return
        idx = self._selected_index() or 0
        entry = self._data["variants"][idx]
        body = {
            "assembly_id": self._data.get("assembly_id"),
            "variant_key": entry.get("variant_key"),
            "intent": self.intent_var.get().strip(),
            "current_layers": entry.get("layers") or {},
            "constraints": ["lod0_tier", f"deterministic_seed_{self._data.get('seed', 42)}"],
            "reference_tags": entry.get("tags") or [],
        }
        result = variant_set.variant_agent_request(body, write=True)
        self.patch_text.delete("1.0", tk.END)
        self.patch_text.insert("1.0", json.dumps(result, indent=2))
        written = result.get("written_path") or "debug_runs/art_pipeline/variant_agent_request.json"
        self._on_log(f"agent request → {written}")
        messagebox.showinfo(
            "Request agent",
            f"Wrote {written}\nPaste into Cursor; apply via variant_set_patch after review.",
        )

    def on_apply_patch(self) -> None:
        if not self.state.variant_set_path:
            messagebox.showinfo("Variants", "Save variant set to disk first.")
            return
        try:
            raw = self.patch_text.get("1.0", tk.END).strip()
            payload = json.loads(raw) if raw else {}
        except json.JSONDecodeError as exc:
            messagebox.showerror("Apply patch", str(exc))
            return
        patch = payload.get("patch") if isinstance(payload, dict) else payload
        if not isinstance(patch, list):
            messagebox.showerror("Apply patch", "Patch JSON must be a list or {patch:[...]}")
            return
        try:
            result = variant_set.variant_set_patch(self.state.variant_set_path, patch)
            self._data = result["document"]
            self.state.variant_set_data = self._data
            self._refresh_list()
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Apply patch", str(exc))
            return
        self._on_log(f"patch applied ({len(patch)} ops)")
        messagebox.showinfo("Apply patch", "Patch applied and saved.")

    def on_bake_selected(self) -> None:
        if not self.state.variant_set_path or not self.state.selected_variant_key:
            messagebox.showinfo("Variants", "Save variant set and select a variant.")
            return
        vs_aid = str((self._data or {}).get("assembly_id") or "")
        cur_aid = str(self.state.assembly_id or "")
        if cur_aid and vs_aid and vs_aid != cur_aid:
            if not messagebox.askyesno(
                "Assembly mismatch",
                f"Variant set targets:\n  {vs_aid}\n\n"
                f"Current Assembly tab snapshot:\n  {cur_aid}\n\n"
                "Bake anyway? (PNG will land under the variant set assembly_id folder.)",
            ):
                self._on_log(f"bake cancelled — variant_set assembly_id={vs_aid} != current {cur_aid}")
                return
        self._on_log(f"variant-bake {self.state.selected_variant_key} → assembly {vs_aid}")
        try:
            result = variant_set.variant_bake(
                self.state.variant_set_path,
                self.state.selected_variant_key,
            )
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Bake", str(exc))
            return
        self._data = variant_set.load_variant_set(self.state.variant_set_path)
        self.state.variant_set_data = self._data
        self._refresh_list()
        self.on_variant_select()
        if result.get("ok"):
            png = result.get("png")
            rel = Path(str(png)).relative_to(repo_root()) if png else None
            self.state.atlas_folder = str((repo_root() / "assets/staging/tiles" / self._data["assembly_id"]).resolve())
            messagebox.showinfo("Bake", f"OK\n{rel}")
        else:
            messagebox.showerror("Bake", result.get("error") or "failed")
