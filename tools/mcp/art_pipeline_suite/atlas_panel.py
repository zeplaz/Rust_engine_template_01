"""Atlas workspace — tile_batch_run + tile_atlas_pack."""

from __future__ import annotations

import json
import tempfile
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, messagebox, ttk

from rust_engine_mcp import variant_set
from rust_engine_mcp.paths import repo_root

from module_viewer.pipeline_runner import (
    art_debug_gui_enabled,
    find_latest_atlas_in,
    open_light_blend,
    open_keyframe_render_addon,
    pack_tile_folder,
    run_lod0_batch,
    run_tile_batch,
)

from .state import SuiteState


class AtlasPanel(ttk.Frame):
    def __init__(self, master: tk.Misc, state: SuiteState, *, on_log) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._on_log = on_log
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Atlas — PRODUCTION: keyframe PNG folder → Pack atlas (tilemapgen). tile_batch_run = CI/smoke ortho OR keyframe_pack register-only.",
            wraplength=720,
            justify=tk.LEFT,
        ).pack(anchor=tk.W, pady=(0, 8))

        batch_row = ttk.Frame(self)
        batch_row.pack(fill=tk.X, pady=4)
        ttk.Label(batch_row, text="tile_batch_v1").pack(side=tk.LEFT)
        self.batch_json_var = tk.StringVar()
        ttk.Entry(batch_row, textvariable=self.batch_json_var, width=52).pack(
            side=tk.LEFT, padx=4, fill=tk.X, expand=True
        )
        ttk.Button(batch_row, text="Browse…", command=self.on_browse_batch).pack(side=tk.LEFT)
        ttk.Button(batch_row, text="From variant set", command=self.on_batch_from_variant_set).pack(
            side=tk.LEFT, padx=4
        )
        ttk.Button(batch_row, text="Run tile batch", command=self.on_run_batch).pack(anchor=tk.W, pady=4)

        tile_row = ttk.Frame(self)
        tile_row.pack(fill=tk.X, pady=4)
        ttk.Label(tile_row, text="PNG folder").pack(side=tk.LEFT)
        self.folder_var = tk.StringVar()
        ttk.Entry(tile_row, textvariable=self.folder_var, width=52).pack(
            side=tk.LEFT, padx=4, fill=tk.X, expand=True
        )
        ttk.Button(tile_row, text="Browse…", command=self.on_browse_folder).pack(side=tk.LEFT)
        self.keyframe_rename_var = tk.BooleanVar(value=False)
        ttk.Checkbutton(tile_row, text="-pk rename", variable=self.keyframe_rename_var).pack(side=tk.LEFT)
        ttk.Button(self, text="Pack atlas (tilemapgen)", command=self.on_pack).pack(anchor=tk.W, pady=4)

        lod_row = ttk.Frame(self)
        lod_row.pack(fill=tk.X, pady=8)
        ttk.Label(lod_row, text="lod0 batch").pack(side=tk.LEFT)
        self.lod_batch_var = tk.StringVar(value="kit_lod0_003")
        ttk.Combobox(
            lod_row,
            textvariable=self.lod_batch_var,
            width=16,
            values=[f"kit_lod0_{i:03d}" for i in range(3, 11)],
        ).pack(side=tk.LEFT, padx=4)
        self.lod_phase_var = tk.StringVar(value="g0g1")
        ttk.Combobox(
            lod_row,
            textvariable=self.lod_phase_var,
            width=10,
            state="readonly",
            values=["g0g1", "geometry", "promote", "full"],
        ).pack(side=tk.LEFT, padx=4)
        ttk.Button(lod_row, text="Run lod0 batch", command=self.on_lod0).pack(side=tk.LEFT, padx=4)

        if art_debug_gui_enabled():
            dbg = ttk.Frame(self)
            dbg.pack(fill=tk.X, pady=4)
            ttk.Button(dbg, text="Open light setup (.blend)", command=self.on_light_blend).pack(
                side=tk.LEFT, padx=2
            )
            ttk.Button(dbg, text="Keyframe addon", command=self.on_keyframe_addon).pack(side=tk.LEFT, padx=2)
        else:
            ttk.Label(
                self,
                text="Blender GUI hidden — RUST_ENGINE_ART_DEBUG_GUI=1 for legacy debug buttons.",
                foreground="#666",
            ).pack(anchor=tk.W)

        ttk.Label(self, text="Log").pack(anchor=tk.W, pady=(8, 0))
        self.log_text = tk.Text(self, height=12, wrap=tk.WORD, font=("Consolas", 9))
        scroll = ttk.Scrollbar(self, orient=tk.VERTICAL, command=self.log_text.yview)
        self.log_text.configure(yscrollcommand=scroll.set)
        self.log_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll.pack(side=tk.RIGHT, fill=tk.Y)

    def _log(self, text: str) -> None:
        self.log_text.insert(tk.END, text + "\n")
        self.log_text.see(tk.END)
        self._on_log(text)

    def sync_folder_from_state(self) -> None:
        if self.state.atlas_folder:
            self.folder_var.set(self.state.atlas_folder)

    def on_browse_batch(self) -> None:
        path = filedialog.askopenfilename(filetypes=[("JSON", "*.json")])
        if path:
            self.batch_json_var.set(path)
            self.state.tile_batch_path = path

    def on_browse_folder(self) -> None:
        path = filedialog.askdirectory()
        if path:
            self.folder_var.set(path)
            self.state.atlas_folder = path

    def on_batch_from_variant_set(self) -> None:
        data = self.state.variant_set_data
        if not data:
            messagebox.showinfo("Atlas", "Load or create a variant set first (Variants tab).")
            return
        batch = variant_set.expand_variant_set_to_tile_batch(data)
        tmp = Path(tempfile.gettempdir()) / f"{batch['batch_id']}.json"
        tmp.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
        self.batch_json_var.set(str(tmp))
        self.state.tile_batch_path = str(tmp)
        self._log(f"expanded variant_set → {tmp}")
        messagebox.showinfo("Atlas", f"Wrote temp tile_batch:\n{tmp}")

    def on_run_batch(self) -> None:
        path = self.batch_json_var.get().strip()
        if not path:
            messagebox.showinfo("Atlas", "Choose tile_batch_v1 JSON.")
            return
        self._log(f"tile-batch-run {path}")
        code, log = run_tile_batch(Path(path))
        self._log(log)
        if code != 0:
            messagebox.showerror("Tile batch", log[:2000])
            return
        try:
            batch = json.loads(Path(path).read_text(encoding="utf-8"))
            bid = batch.get("batch_id")
            folder = repo_root() / "assets/staging/tiles" / str(bid)
            if folder.is_dir():
                self.folder_var.set(str(folder))
                self.state.atlas_folder = str(folder)
        except Exception:  # noqa: BLE001
            pass
        messagebox.showinfo("Tile batch", "Pipeline finished — see log.")

    def on_pack(self) -> None:
        folder = self.folder_var.get().strip()
        if not folder:
            messagebox.showinfo("Pack", "Choose PNG folder.")
            return
        code, log = pack_tile_folder(Path(folder), keyframe_rename=self.keyframe_rename_var.get())
        self._log(log)
        if code != 0:
            messagebox.showerror("Pack", log)
            return
        atlas = find_latest_atlas_in(Path(folder))
        messagebox.showinfo("Pack", f"Atlas OK\n{atlas or 'see log'}")

    def on_lod0(self) -> None:
        code, log = run_lod0_batch(self.lod_batch_var.get(), step=self.lod_phase_var.get())
        self._log(log)
        if code != 0:
            messagebox.showerror("lod0 batch", log[:2000])

    def on_light_blend(self) -> None:
        code, log = open_light_blend()
        self._log(log)

    def on_keyframe_addon(self) -> None:
        code, log = open_keyframe_render_addon()
        self._log(log)
