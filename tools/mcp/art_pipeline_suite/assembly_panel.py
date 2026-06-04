"""Assembly Editor (APS-UI-003b) — footprint grid, grammar, categorized semantic tags."""

from __future__ import annotations

import json
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

from rust_engine_mcp import aps_tags, assembly, building_grammar
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.assembly_production import validate_assembly_snapshot_path
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_p0_gate

from .footprint_canvas import FootprintCanvas
from .grammar_inspector import GrammarInspectorPanel
from .material_browser import MaterialBrowserPanel
from .assembly_preview_panel import AssemblyPreviewPanel
from .state import SuiteState


class AssemblyPanel(ttk.Frame):
    def __init__(self, master: tk.Misc, state: SuiteState, *, on_log) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        self._snapshot: dict | None = None
        self._selected_node_id: str | None = None
        self._semantic_tag_vars: dict[str, dict[str, tk.BooleanVar]] = {}
        self._variant_tag_vars: dict[str, tk.BooleanVar] = {}
        self._tag_category_frames: dict[str, ttk.LabelFrame] = {}
        self._material_profiles: list[str] = []
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Assembly — pick a footprint cell, assign material from the library beside the grid, "
            "edit tags on the right, then Save. P0 gate before ship/bake.",
            wraplength=900,
            justify=tk.LEFT,
        ).pack(anchor=tk.W, pady=(0, 6))

        gen = ttk.LabelFrame(self, text="Generate", padding=6)
        gen.pack(fill=tk.X, pady=4)

        gram_row = ttk.Frame(gen)
        gram_row.pack(fill=tk.X, pady=2)
        self.use_grammar_var = tk.BooleanVar(value=False)
        ttk.Checkbutton(
            gram_row, text="Use building grammar", variable=self.use_grammar_var, command=self._on_grammar_toggle
        ).pack(side=tk.LEFT)
        ttk.Label(gram_row, text="Archetype").pack(side=tk.LEFT, padx=(12, 0))
        archetypes = building_grammar.list_archetype_ids() or ["IndustrialWarehouse"]
        self.archetype_var = tk.StringVar(value=archetypes[0])
        self.archetype_combo = ttk.Combobox(
            gram_row, textvariable=self.archetype_var, width=20, values=archetypes, state="readonly"
        )
        self.archetype_combo.pack(side=tk.LEFT, padx=4)
        self.archetype_combo.bind("<<ComboboxSelected>>", self._on_archetype_change)
        ttk.Label(gram_row, text="District").pack(side=tk.LEFT, padx=(8, 0))
        districts = building_grammar.list_district_styles(archetypes[0]) or ["industrial_west"]
        self.district_var = tk.StringVar(value=districts[0] if districts else "")
        self.district_combo = ttk.Combobox(
            gram_row, textvariable=self.district_var, width=16, values=districts, state="readonly"
        )
        self.district_combo.pack(side=tk.LEFT, padx=4)

        row = ttk.Frame(gen)
        row.pack(fill=tk.X, pady=2)
        ttk.Label(row, text="StylePack").pack(side=tk.LEFT)
        packs = assembly.list_style_packs()
        self.style_var = tk.StringVar(value=self.state.style_pack_id)
        self.style_combo = ttk.Combobox(
            row, textvariable=self.style_var, width=22, values=packs or ["style_victorian"]
        )
        self.style_combo.pack(side=tk.LEFT, padx=4)
        ttk.Label(row, text="Tier").pack(side=tk.LEFT, padx=(8, 0))
        self.tier_var = tk.StringVar(value="production")
        ttk.Combobox(
            row, textvariable=self.tier_var, width=12, values=["production", "lod0"], state="readonly"
        ).pack(side=tk.LEFT, padx=4)

        row2 = ttk.Frame(gen)
        row2.pack(fill=tk.X, pady=2)
        ttk.Label(row2, text="Footprint W×D").pack(side=tk.LEFT)
        self.footprint_var = tk.StringVar(value=self.state.footprint)
        self.footprint_entry = ttk.Entry(row2, textvariable=self.footprint_var, width=8)
        self.footprint_entry.pack(side=tk.LEFT, padx=4)
        ttk.Label(row2, text="Floors").pack(side=tk.LEFT, padx=(8, 0))
        self.floors_var = tk.IntVar(value=self.state.floors)
        self.floors_spin = ttk.Spinbox(row2, from_=1, to=8, textvariable=self.floors_var, width=4)
        self.floors_spin.pack(side=tk.LEFT, padx=4)
        ttk.Label(row2, text="Seed").pack(side=tk.LEFT, padx=(8, 0))
        self.seed_var = tk.IntVar(value=self.state.seed)
        ttk.Spinbox(row2, from_=0, to=999999, textvariable=self.seed_var, width=8).pack(side=tk.LEFT, padx=4)
        ttk.Button(row2, text="Generate snapshot", command=self.on_generate).pack(side=tk.LEFT, padx=8)

        file_row = ttk.Frame(self)
        file_row.pack(fill=tk.X, pady=4)
        ttk.Button(file_row, text="Load…", command=self.on_load).pack(side=tk.LEFT, padx=2)
        ttk.Button(file_row, text="Save", command=self.on_save).pack(side=tk.LEFT, padx=2)
        ttk.Button(file_row, text="Validate", command=self.on_validate).pack(side=tk.LEFT, padx=2)
        ttk.Button(file_row, text="P0 gate", command=self.on_validate_p0).pack(side=tk.LEFT, padx=2)
        self.path_var = tk.StringVar(value="(no snapshot)")
        ttk.Label(file_row, textvariable=self.path_var, foreground="#444").pack(side=tk.LEFT, padx=8)
        ttk.Button(file_row, text="Preview assembly", command=self.on_preview_assembly).pack(side=tk.LEFT, padx=2)

        workspace = ttk.Panedwindow(self, orient=tk.HORIZONTAL)
        workspace.pack(fill=tk.BOTH, expand=True, pady=8)

        footprint_pane = ttk.Frame(workspace, padding=4)
        materials_pane = ttk.Frame(workspace, padding=4)
        inspector_pane = ttk.Frame(workspace, padding=4)
        workspace.add(footprint_pane, weight=2)
        workspace.add(materials_pane, weight=2)
        workspace.add(inspector_pane, weight=3)

        ttk.Label(footprint_pane, text="Footprint & placements", font=("Segoe UI", 9, "bold")).pack(
            anchor=tk.W
        )
        self.placement_list = tk.Listbox(
            footprint_pane, exportselection=False, font=("Consolas", 9), height=5
        )
        self.placement_list.pack(fill=tk.X, pady=(4, 6))
        self.placement_list.bind("<<ListboxSelect>>", self.on_placement_select)
        self.footprint_canvas = FootprintCanvas(footprint_pane, on_cell_select=self._on_grid_cell_select)
        self.footprint_canvas.pack(fill=tk.BOTH, expand=True)

        mat_frame = ttk.LabelFrame(materials_pane, text="Material library", padding=4)
        mat_frame.pack(fill=tk.BOTH, expand=True)
        self.material_browser = MaterialBrowserPanel(
            mat_frame,
            on_apply_material=self._apply_material_profile,
            on_log=self._on_log,
            layout="vertical",
        )
        self.material_browser.pack(fill=tk.BOTH, expand=True)

        self.assembly_preview = AssemblyPreviewPanel(inspector_pane, on_log=self._on_log)
        self.assembly_preview.pack(fill=tk.X, pady=(0, 8))

        slot = ttk.LabelFrame(inspector_pane, text="Selected slot", padding=8)
        slot.pack(fill=tk.BOTH, expand=True)

        ttk.Label(slot, text="Node id").grid(row=0, column=0, sticky=tk.W)
        self.node_id_var = tk.StringVar(value="—")
        ttk.Label(slot, textvariable=self.node_id_var, font=("Consolas", 9)).grid(
            row=0, column=1, sticky=tk.W, padx=4
        )

        ttk.Label(slot, text="Module").grid(row=1, column=0, sticky=tk.W, pady=4)
        self.module_var = tk.StringVar(value="")
        ttk.Entry(slot, textvariable=self.module_var, width=28, state="readonly").grid(
            row=1, column=1, sticky=tk.W, padx=4
        )

        ttk.Label(slot, text="Material profile").grid(row=2, column=0, sticky=tk.W, pady=4)
        self.material_var = tk.StringVar(value="—")
        mat_row = ttk.Frame(slot)
        mat_row.grid(row=2, column=1, sticky=tk.W, padx=4)
        self._material_swatch = tk.Label(mat_row, text="  ", width=2, bg="#dddddd", relief=tk.RIDGE)
        self._material_swatch.pack(side=tk.LEFT, padx=(0, 6))
        ttk.Label(mat_row, textvariable=self.material_var, font=("Consolas", 9)).pack(side=tk.LEFT)

        ttk.Label(slot, text="LOD policy").grid(row=3, column=0, sticky=tk.W, pady=4)
        self.lod_var = tk.StringVar(value="production")
        ttk.Combobox(
            slot,
            textvariable=self.lod_var,
            width=14,
            values=["lod0", "production", "hero"],
            state="readonly",
        ).grid(row=3, column=1, sticky=tk.W, padx=4)

        tag_filter_row = ttk.Frame(slot)
        tag_filter_row.grid(row=5, column=0, columnspan=2, sticky=tk.EW, pady=(8, 2))
        ttk.Label(tag_filter_row, text="Tag category filter").pack(side=tk.LEFT)
        self.tag_filter_var = tk.StringVar(value="all")
        ttk.Combobox(
            tag_filter_row,
            textvariable=self.tag_filter_var,
            width=14,
            values=["all"] + list(aps_tags.CATEGORY_ORDER),
            state="readonly",
        ).pack(side=tk.LEFT, padx=4)
        self.tag_filter_var.trace_add("write", lambda *_: self._apply_tag_category_filter())

        tags_outer = ttk.Frame(slot)
        tags_outer.grid(row=6, column=0, columnspan=2, sticky=tk.NSEW)
        self._build_semantic_tag_pickers(tags_outer)

        var_frame = ttk.LabelFrame(slot, text="Variant tags", padding=4)
        var_frame.grid(row=7, column=0, columnspan=2, sticky=tk.EW, pady=4)
        var_grid = ttk.Frame(var_frame)
        var_grid.pack(anchor=tk.W)
        for i, tag in enumerate(assembly.COMMON_VARIANT_TAGS):
            var = tk.BooleanVar(value=False)
            self._variant_tag_vars[tag] = var
            ttk.Checkbutton(var_grid, text=tag, variable=var).grid(row=0, column=i, sticky=tk.W, padx=4)

        btn_row = ttk.Frame(slot)
        btn_row.grid(row=8, column=0, columnspan=2, sticky=tk.W, pady=8)
        ttk.Button(btn_row, text="Apply tags to slot", command=self.on_apply_slot).pack(side=tk.LEFT, padx=2)

        self.grammar_inspector = GrammarInspectorPanel(inspector_pane)
        self.grammar_inspector.pack(fill=tk.X, pady=4)

        self.validation_var = tk.StringVar(value="")
        ttk.Label(inspector_pane, textvariable=self.validation_var, wraplength=420, foreground="#006400").pack(
            anchor=tk.W, pady=4
        )

        def _workspace_minsizes(_event=None) -> None:
            try:
                w = workspace.winfo_width()
                if w < 400:
                    return
                workspace.paneconfigure(footprint_pane, width=max(260, int(w * 0.30)))
                workspace.paneconfigure(materials_pane, width=max(240, int(w * 0.28)))
            except tk.TclError:
                pass

        workspace.bind("<Configure>", _workspace_minsizes)
        slot.columnconfigure(1, weight=1)
        self._on_grammar_toggle()

    def _build_semantic_tag_pickers(self, parent: ttk.Frame) -> None:
        labels = aps_tags.category_labels()
        for cat in aps_tags.CATEGORY_ORDER:
            frame = ttk.LabelFrame(parent, text=labels.get(cat, cat.title()), padding=4)
            frame.pack(fill=tk.X, pady=2)
            self._tag_category_frames[cat] = frame
            grid = ttk.Frame(frame)
            grid.pack(anchor=tk.W)
            self._semantic_tag_vars[cat] = {}
            for i, row in enumerate(aps_tags.tags_for_category(cat)):
                tag_id = str(row.get("id") or "")
                label = str(row.get("label") or tag_id)
                var = tk.BooleanVar(value=False)
                self._semantic_tag_vars[cat][tag_id] = var
                ttk.Checkbutton(grid, text=label, variable=var).grid(
                    row=i // 3, column=i % 3, sticky=tk.W, padx=4
                )

    def _apply_tag_category_filter(self) -> None:
        filt = self.tag_filter_var.get().strip().lower()
        for cat, frame in self._tag_category_frames.items():
            if filt == "all" or filt == cat:
                frame.pack(fill=tk.X, pady=2)
            else:
                frame.pack_forget()

    def _on_material_browser_apply(self, profile_id: str) -> None:
        self.material_var.set(profile_id)
        if self._snapshot and self._selected_node_id:
            self.on_apply_slot()
        else:
            self._on_log(f"material selected: {profile_id} (pick a slot to apply)")

    def _on_grammar_toggle(self) -> None:
        use = self.use_grammar_var.get()
        state = "readonly" if use else "disabled"
        self.archetype_combo.configure(state=state)
        self.district_combo.configure(state=state)
        fp_state = "disabled" if use else "normal"
        self.footprint_entry.configure(state=fp_state)
        self.floors_spin.configure(state=fp_state)
        if use:
            self.style_combo.configure(state="disabled")
        else:
            self.style_combo.configure(state="normal")

    def _on_archetype_change(self, _event=None) -> None:
        archetype = self.archetype_var.get().strip()
        if not archetype:
            return
        districts = building_grammar.list_district_styles(archetype)
        self.district_combo.configure(values=districts or [""])
        if districts:
            self.district_var.set(districts[0])

    def sync_from_state(self) -> None:
        self.style_var.set(self.state.style_pack_id)
        self.footprint_var.set(self.state.footprint)
        self.floors_var.set(self.state.floors)
        self.seed_var.set(self.state.seed)

    def _sync_state_from_snapshot(self, snap: dict) -> None:
        self.state.assembly_id = str(snap.get("assembly_id"))
        rel = snap.get("written_path")
        if rel:
            self.state.assembly_snapshot_path = str(rel)
        self.state.style_pack_id = str(snap.get("style_pack_id") or self.state.style_pack_id)
        fp = snap.get("footprint") or {}
        w, d, f = fp.get("width"), fp.get("depth"), fp.get("floors")
        if w and d:
            self.state.footprint = f"{w}x{d}"
            self.footprint_var.set(self.state.footprint)
        if f:
            self.state.floors = int(f)
            self.floors_var.set(int(f))
        if snap.get("seed") is not None:
            self.state.seed = int(snap["seed"])
            self.seed_var.set(int(snap["seed"]))
        if snap.get("archetype_id"):
            self.use_grammar_var.set(True)
            self.archetype_var.set(str(snap["archetype_id"]))
            self._on_archetype_change()
            if snap.get("district_style"):
                self.district_var.set(str(snap["district_style"]))
            self._on_grammar_toggle()
        self.state.module_ids_in_assembly = sorted(
            {str(p.get("module_id")) for p in snap.get("module_placements") or []}
        )
        self.state.assembly_snapshot_data = snap

    def _placement_label(self, p: dict) -> str:
        token = p.get("token", "?")
        gx, gy, fl = p.get("grid_x"), p.get("grid_y"), p.get("floor")
        mat = p.get("material_profile") or "—"
        return f"f{fl} ({gx},{gy}) {token}  {p.get('module_id')}  [{mat}]"

    def _sorted_placements(self) -> list[dict]:
        if not self._snapshot:
            return []
        return sorted(
            self._snapshot.get("module_placements") or [],
            key=lambda p: (int(p.get("floor") or 0), int(p.get("grid_y") or 0), int(p.get("grid_x") or 0)),
        )

    def _refresh_placement_list(self) -> None:
        self.placement_list.delete(0, tk.END)
        for p in self._sorted_placements():
            self.placement_list.insert(tk.END, self._placement_label(p))

    def _refresh_footprint_grid(self) -> None:
        if not self._snapshot:
            self.footprint_canvas.set_cells([], [])
            return
        cells = assembly.footprint_cells_for_snapshot(self._snapshot)
        self.footprint_canvas.set_cells(cells, self._sorted_placements())

    def _load_snapshot_into_ui(self, snap: dict, *, path_hint: str = "") -> None:
        self._snapshot = assembly.enrich_snapshot(snap)
        self._sync_state_from_snapshot(self._snapshot)
        self.path_var.set(path_hint or self.state.assembly_snapshot_path or "(memory)")
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self.grammar_inspector.load_snapshot(self._snapshot)
        self.assembly_preview.set_snapshot(self._snapshot)
        if self.placement_list.size():
            self.placement_list.selection_set(0)
            self.on_placement_select()

    def on_generate(self) -> None:
        seed = int(self.seed_var.get())
        tier = self.tier_var.get().strip() or "lod0"
        try:
            if self.use_grammar_var.get():
                archetype = self.archetype_var.get().strip()
                district = self.district_var.get().strip()
                self._on_log(f"assembly-snapshot-generate grammar {archetype}/{district} seed={seed}")
                snap = assembly.generate_assembly_snapshot(
                    archetype_id=archetype,
                    district_style=district,
                    seed=seed,
                    source_tier=tier,
                )
            else:
                style = self.style_var.get().strip()
                fp = self.footprint_var.get().strip().lower()
                w, d = fp.split("x")
                width, depth = int(w), int(d)
                floors = int(self.floors_var.get())
                self._on_log(f"assembly-snapshot-generate {style} {width}x{depth} tier={tier}")
                snap = assembly.generate_assembly_snapshot(
                    style_pack_id=style,
                    width=width,
                    depth=depth,
                    floors=floors,
                    seed=seed,
                    source_tier=tier,
                )
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Assembly", str(exc))
            return
        self._load_snapshot_into_ui(snap, path_hint=str(snap.get("written_path") or ""))
        self._on_log(f"wrote {self.state.assembly_snapshot_path}")
        rep = self._p0_report()
        if rep.status == "passed":
            messagebox.showinfo("Assembly", f"Snapshot OK\n{self.state.assembly_id}\n\nP0 gate: passed")
        else:
            hints = self._format_validation_hints(rep)
            self.validation_var.set(f"P0 failed: {hints[:200]}")
            messagebox.showwarning(
                "Assembly — P0 failed",
                f"Snapshot generated but P0 gate failed (fix before save/bake):\n\n{hints}",
            )

    def _p0_report(self):
        import tempfile

        from rust_engine_mcp.validators.report import ValidationReport

        if not self._snapshot:
            return ValidationReport(
                validator="assembly_p0",
                status="failed",
                compression_level=3,
                summary="no snapshot",
                error_count=1,
                errors=[],
            )
        path = self.state.assembly_snapshot_path
        if path and (repo_root() / path).is_file():
            snap_path = str(path)
        else:
            tmp = Path(tempfile.gettempdir()) / "_aps_assembly_p0_validate.json"
            tmp.write_text(json.dumps(self._snapshot, indent=2), encoding="utf-8")
            snap_path = str(tmp)
        return validate_assembly_p0_gate(
            self._snapshot,
            snapshot_path=snap_path.replace("\\", "/"),
            ship=True,
        )

    @staticmethod
    def _format_validation_hints(rep) -> str:
        return "\n".join(e.hint or e.kind for e in rep.errors if e.severity == "error")[:1200]

    def _run_p0_or_block(self, action: str) -> bool:
        rep = self._p0_report()
        if rep.status == "passed":
            self.validation_var.set(f"P0 gate: passed — {action} OK")
            return True
        hints = self._format_validation_hints(rep)
        self.validation_var.set(f"P0 failed: {hints[:200]}")
        return messagebox.askyesno(
            f"P0 gate failed — {action} anyway?",
            f"{hints}\n\nProceed anyway? (Not recommended for ship/bake.)",
        )

    def on_load(self) -> None:
        initial = repo_root() / "assets" / "staging" / "assemblies"
        path = filedialog.askopenfilename(
            title="Load assembly snapshot",
            initialdir=str(initial) if initial.is_dir() else str(repo_root()),
            filetypes=[("JSON", "*.json"), ("All", "*.*")],
        )
        if not path:
            return
        try:
            snap = assembly.load_assembly_snapshot(path)
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Load", str(exc))
            return
        rel = Path(path).resolve().relative_to(repo_root()).as_posix()
        snap["written_path"] = rel
        self._load_snapshot_into_ui(snap, path_hint=rel)
        self._on_log(f"loaded {rel}")

    def on_save(self) -> None:
        if not self._snapshot:
            messagebox.showinfo("Save", "Generate or load a snapshot first.")
            return
        if not self._run_p0_or_block("Save"):
            self._on_log("save cancelled — P0 gate failed")
            return
        try:
            out = assembly.save_assembly_snapshot(self._snapshot)
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Save", str(exc))
            return
        rel = str(out.relative_to(repo_root())).replace("\\", "/")
        self.state.assembly_snapshot_path = rel
        self.path_var.set(rel)
        self._snapshot["written_path"] = rel
        self._on_log(f"saved {rel}")
        messagebox.showinfo("Save", f"Saved\n{rel}")

    def on_preview_assembly(self) -> None:
        if not self._run_p0_or_block("Preview"):
            self._on_log("preview cancelled — P0 gate failed")
            return
        self.assembly_preview.on_preview()

    def on_validate(self) -> None:
        if not self._snapshot:
            messagebox.showinfo("Validate", "No snapshot loaded.")
            return
        path = self.state.assembly_snapshot_path
        if path:
            rep = validate_assembly_snapshot_path(repo_root() / path, ship=True)
        else:
            import tempfile

            tmp = Path(tempfile.gettempdir()) / "_aps_assembly_validate.json"
            tmp.write_text(json.dumps(self._snapshot, indent=2), encoding="utf-8")
            rep = validate_assembly_snapshot_path(tmp, ship=True)
        self._show_validation_report(rep, title="Validate (production)")

    def on_validate_p0(self) -> None:
        if not self._snapshot:
            messagebox.showinfo("P0 gate", "No snapshot loaded.")
            return
        rep = self._p0_report()
        self._show_validation_report(rep, title="P0 gate (production + grammar)")

    def _show_validation_report(self, rep, *, title: str) -> None:
        if rep.status == "passed":
            self.validation_var.set(f"{title}: passed")
            messagebox.showinfo(title, "Passed.")
        else:
            hints = self._format_validation_hints(rep)
            self.validation_var.set(f"{title} failed: {hints[:200]}")
            messagebox.showwarning(title, hints or rep.summary)

    def _select_placement_at(self, gx: int, gy: int, floor: int) -> None:
        placements = self._sorted_placements()
        for idx, p in enumerate(placements):
            if (
                int(p.get("grid_x") or 0) == gx
                and int(p.get("grid_y") or 0) == gy
                and int(p.get("floor") or 0) == floor
            ):
                self.placement_list.selection_clear(0, tk.END)
                self.placement_list.selection_set(idx)
                self.placement_list.see(idx)
                self.on_placement_select()
                return

    def _on_grid_cell_select(self, gx: int, gy: int, floor: int) -> None:
        self._select_placement_at(gx, gy, floor)

    def on_placement_select(self, _event=None) -> None:
        if not self._snapshot:
            return
        sel = self.placement_list.curselection()
        if not sel:
            return
        placements = self._sorted_placements()
        idx = int(sel[0])
        if idx >= len(placements):
            return
        p = placements[idx]
        self._selected_node_id = assembly.placement_node_id(p)
        self.node_id_var.set(self._selected_node_id)
        self.module_var.set(str(p.get("module_id") or ""))
        mat = str(p.get("material_profile") or "")
        self.material_var.set(mat or "—")
        self._update_material_swatch(mat)
        self.lod_var.set(str(p.get("lod_policy") or "production"))
        semantic = p.get("semantic_tags") or aps_tags.semantic_tags_from_flat(p.get("placement_tags") or [])
        for cat, tag_map in self._semantic_tag_vars.items():
            active = set(semantic.get(cat) or [])
            for tag_id, var in tag_map.items():
                var.set(tag_id in active)
        vtags = set(p.get("variant_tags") or [])
        for tag, var in self._variant_tag_vars.items():
            var.set(tag in vtags)
        self.footprint_canvas.set_selection(
            int(p.get("grid_x") or 0), int(p.get("grid_y") or 0), int(p.get("floor") or 0)
        )
        if mat:
            self.material_browser.highlight_profile(mat)

    def _update_material_swatch(self, profile_id: str) -> None:
        color = "#dddddd"
        if profile_id:
            try:
                from rust_engine_mcp.material_profiles import ensure_profile_textures

                entry = ensure_profile_textures(profile_id, size=64)
                path = entry.albedo_path
                if path and path.is_file():
                    from PIL import Image

                    img = Image.open(path).convert("RGB")
                    img.thumbnail((16, 16))
                    r, g, b = img.getpixel((4, 4))
                    color = f"#{r:02x}{g:02x}{b:02x}"
            except Exception:
                if "steel" in profile_id:
                    color = "#6a7f94"
                elif "brick" in profile_id:
                    color = "#9e4a38"
                elif "wood" in profile_id:
                    color = "#7a5a3a"
        self._material_swatch.configure(bg=color)

    def _apply_material_profile(self, profile_id: str) -> None:
        if not self._snapshot or not self._selected_node_id:
            messagebox.showinfo("Material", "Select a placement row or grid cell first.")
            return
        self.material_var.set(profile_id)
        self._update_material_swatch(profile_id)
        try:
            self._snapshot = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                material_profile=profile_id,
            )
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Material", str(exc))
            return
        self.state.assembly_snapshot_data = self._snapshot
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._on_log(f"material {profile_id} → {self._selected_node_id}")
        self.validation_var.set(f"Material {profile_id} applied — Save snapshot before bake")

    def _collect_semantic_tags(self) -> dict[str, list[str]]:
        out: dict[str, list[str]] = {}
        for cat, tag_map in self._semantic_tag_vars.items():
            picked = [tid for tid, var in tag_map.items() if var.get()]
            if picked:
                out[cat] = picked
        return out

    def on_apply_slot(self) -> None:
        if not self._snapshot or not self._selected_node_id:
            messagebox.showinfo("Apply", "Select a placement row first.")
            return
        semantic_tags = self._collect_semantic_tags()
        variant_tags = [t for t, v in self._variant_tag_vars.items() if v.get()]
        try:
            self._snapshot = assembly.update_placement(
                self._snapshot,
                self._selected_node_id,
                material_profile=self.material_var.get().strip(),
                semantic_tags=semantic_tags,
                variant_tags=variant_tags,
                lod_policy=self.lod_var.get().strip(),
            )
        except Exception as exc:  # noqa: BLE001
            messagebox.showerror("Apply", str(exc))
            return
        self.state.assembly_snapshot_data = self._snapshot
        self._refresh_placement_list()
        self._refresh_footprint_grid()
        self._on_log(f"updated {self._selected_node_id} material={self.material_var.get()}")
        self.validation_var.set("Slot updated — run Validate before bake")
