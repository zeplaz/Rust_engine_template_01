"""Atlas workspace — tile_batch_run + tile_atlas_pack."""

from __future__ import annotations

import json
import tempfile
import threading
import tkinter as tk
from pathlib import Path
from tkinter import filedialog, ttk
from typing import Any, Callable

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

from .aps_collapsible import CollapsibleSection
from .aps_inline_feedback import apply_status_atom, set_inline_status
from .aps_scroll import attach_wheel_area, text_yscroll
from . import aps_theme
from .aps_theme import FONT_UI
from .aps_tk import themed_text
from .aps_tooltips import bind_aps_tooltip
from .aps_workflow_layout import workflow_intro, workflow_primary_row, workflow_status_label
from .atlas_preview_panel import AtlasPreviewPanel
from .job_controller import JobRecord, JobResult, JobState
from .state import ArtDomain, SuiteState

_LANE_LOD_PHASE_TO_STEP = {
    "schema only": "g0g1",
    "geometry": "geometry",
    "promote": "promote",
    "full": "full",
}


class AtlasPanel(ttk.Frame):
    def __init__(
        self,
        master: tk.Misc,
        state: SuiteState,
        *,
        on_log,
        start_job: StartJobFn | None = None,
        atlas_service=None,
    ) -> None:
        super().__init__(master, padding=8)
        self.state = state
        self._atlas = atlas_service
        self._on_log = on_log
        self._start_job = start_job
        self._build()

    def set_domain(self, lane: str) -> None:
        reg = "_landscape_atlas_index" if lane == ArtDomain.LANDSCAPE.value else "_tile_atlas_index"
        lane_word = "landscape" if lane == ArtDomain.LANDSCAPE.value else "buildings"
        self._domain_banner.configure(text=f"Registers to: {lane_word.title()} tile index")
        if lane == ArtDomain.LANDSCAPE.value:
            self.refresh_landscape_register()
        else:
            self._register_status_var.set("")

    def refresh_landscape_register(self) -> dict[str, Any]:
        from rust_engine_mcp.aps_atlas_land_register import check_atlas_land_register

        body = check_atlas_land_register()
        ids = body.get("atlas_ids") or []
        if body.get("register_green"):
            detail = f"{len(ids)} atlas row(s): {', '.join(ids)}" if ids else "registered"
            apply_status_atom(self._register_status_lbl, self._register_status_var, "pass", detail=detail)
        else:
            missing: list[str] = []
            if not body.get("pilot_registered"):
                missing.append("pilot")
            if not body.get("expanded_registered"):
                missing.append("expanded")
            detail = f"missing: {', '.join(missing) or 'check witness'}"
            apply_status_atom(self._register_status_lbl, self._register_status_var, "fail", detail=detail)
        return body

    def _build(self) -> None:
        workflow_intro(
            self,
            "Run tile batch, pack PNGs into an atlas sheet, validate, then register — preview is the main work area.",
        )
        self._domain_banner = ttk.Label(
            self,
            text="Registers to: Buildings tile index",
            font=FONT_UI,
            foreground=aps_theme.COLOR_MUTED,
        )
        self._domain_banner.pack(anchor=tk.W, pady=(0, 4))

        primary = workflow_primary_row(self)
        self.run_batch_btn = ttk.Button(primary, text="Run tile batch", command=self.on_run_batch)
        self.run_batch_btn.pack(side=tk.LEFT, padx=(0, 6))
        bind_aps_tooltip(self.run_batch_btn, "atl_batch_run")
        self.pack_btn = ttk.Button(primary, text="Pack atlas", command=self.on_pack)
        self.pack_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(self.pack_btn, "atl_pack")
        refresh_btn = ttk.Button(primary, text="Refresh preview", command=self._refresh_preview)
        refresh_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(refresh_btn, "atl_preview")
        val_btn = ttk.Button(primary, text="Validate atlas meta", command=self.on_validate_atlas_meta)
        val_btn.pack(side=tk.LEFT, padx=(12, 4))
        bind_aps_tooltip(val_btn, "atl_validate")
        open_folder_btn = ttk.Button(primary, text="Open PNG folder", command=self.on_open_png_folder)
        open_folder_btn.pack(side=tk.LEFT, padx=2)
        bind_aps_tooltip(open_folder_btn, "atl_open_folder")

        self._inline_status_lbl, self._inline_status_var = workflow_status_label(self)
        self._atlas_qc_var = tk.StringVar(value="Run Validate atlas meta for plain-language QC before register.")
        self._atlas_qc_lbl = ttk.Label(
            self,
            textvariable=self._atlas_qc_var,
            wraplength=720,
            justify=tk.LEFT,
            font=FONT_UI,
        )
        self._atlas_qc_lbl.pack(anchor=tk.W, pady=(0, 4))

        self.atlas_preview = AtlasPreviewPanel(self, on_log=self._on_log)
        self.atlas_preview.pack(fill=tk.BOTH, expand=True, pady=(4, 4))
        bind_aps_tooltip(self.atlas_preview._atlas_label, "atl_uv_grid")
        bind_aps_tooltip(self.atlas_preview._cells_inner, "atl_cell_strip")

        setup = CollapsibleSection(self, "Setup paths", expanded=False, padding=4)
        setup.pack(fill=tk.X, pady=4)
        setup_body = setup.body

        reg_row = ttk.Frame(setup_body)
        reg_row.pack(fill=tk.X, pady=(0, 4))
        ttk.Button(reg_row, text="Check landscape register", command=self.refresh_landscape_register).pack(
            side=tk.LEFT
        )
        self._register_status_var = tk.StringVar(value="")
        self._register_status_lbl = ttk.Label(reg_row, textvariable=self._register_status_var, font=("Segoe UI", 9))
        self._register_status_lbl.pack(side=tk.LEFT, padx=(8, 0))

        batch_row = ttk.Frame(setup_body)
        batch_row.pack(fill=tk.X, pady=4)
        batch_lbl = ttk.Label(batch_row, text="Tile job file")
        batch_lbl.pack(side=tk.LEFT)
        bind_aps_tooltip(batch_lbl, "atl_batch_json")
        self.batch_json_var = tk.StringVar()
        self.batch_entry = ttk.Entry(batch_row, textvariable=self.batch_json_var, width=52)
        self.batch_entry.pack(side=tk.LEFT, padx=4, fill=tk.X, expand=True)
        bind_aps_tooltip(self.batch_entry, "atl_batch_json")
        ttk.Button(batch_row, text="Browse…", command=self.on_browse_batch).pack(side=tk.LEFT)
        ttk.Button(batch_row, text="From variant set", command=self.on_batch_from_variant_set).pack(
            side=tk.LEFT, padx=4
        )

        tile_row = ttk.Frame(setup_body)
        tile_row.pack(fill=tk.X, pady=4)
        folder_lbl = ttk.Label(tile_row, text="PNG folder")
        folder_lbl.pack(side=tk.LEFT)
        bind_aps_tooltip(folder_lbl, "atl_folder")
        self.folder_var = tk.StringVar()
        self.folder_entry = ttk.Entry(tile_row, textvariable=self.folder_var, width=52)
        self.folder_entry.pack(side=tk.LEFT, padx=4, fill=tk.X, expand=True)
        bind_aps_tooltip(self.folder_entry, "atl_folder")
        ttk.Button(tile_row, text="Browse…", command=self.on_browse_folder).pack(side=tk.LEFT)
        self.keyframe_rename_var = tk.BooleanVar(value=False)
        self.keyframe_rename_cb = ttk.Checkbutton(
            tile_row, text="Rename keyframe PNGs for packing", variable=self.keyframe_rename_var
        )
        self.keyframe_rename_cb.pack(side=tk.LEFT, padx=(8, 0))
        bind_aps_tooltip(self.keyframe_rename_cb, "atl_keyframe_rename")

        advanced = CollapsibleSection(self, "Advanced (smoke & debug)", expanded=False, padding=4)
        advanced.pack(fill=tk.X, pady=(0, 4))
        adv_body = advanced.body

        lod_row = ttk.Frame(adv_body)
        lod_row.pack(fill=tk.X, pady=4)
        lod_lbl = ttk.Label(lod_row, text="Smoke-test batch")
        lod_lbl.pack(side=tk.LEFT)
        bind_aps_tooltip(lod_lbl, "atl_lod0")
        self.lod_batch_var = tk.StringVar(value="kit_lod0_003")
        self.lod_batch_combo = ttk.Combobox(
            lod_row,
            textvariable=self.lod_batch_var,
            width=16,
            values=[f"kit_lod0_{i:03d}" for i in range(3, 11)],
        )
        self.lod_batch_combo.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(self.lod_batch_combo, "atl_batch")
        self.lod_phase_var = tk.StringVar(value="schema only")
        ttk.Combobox(
            lod_row,
            textvariable=self.lod_phase_var,
            width=12,
            state="readonly",
            values=["schema only", "geometry", "promote", "full"],
        ).pack(side=tk.LEFT, padx=4)
        ttk.Button(lod_row, text="Run lod0 batch", command=self.on_lod0).pack(side=tk.LEFT, padx=4)

        if art_debug_gui_enabled():
            dbg = ttk.Frame(adv_body)
            dbg.pack(fill=tk.X, pady=4)
            ttk.Button(dbg, text="Open light setup (.blend)", command=self.on_light_blend).pack(
                side=tk.LEFT, padx=2
            )
            ttk.Button(dbg, text="Keyframe addon", command=self.on_keyframe_addon).pack(side=tk.LEFT, padx=2)
        else:
            ttk.Label(
                adv_body,
                text="Blender debug buttons are hidden (developer mode only).",
                foreground=aps_theme.COLOR_MUTED,
            ).pack(anchor=tk.W)

        ttk.Label(adv_body, text="Log").pack(anchor=tk.W, pady=(8, 0))
        log_wrap = ttk.Frame(adv_body)
        log_wrap.pack(fill=tk.BOTH, expand=True)
        self.log_text = themed_text(log_wrap, height=6, wrap=tk.WORD, font=("Consolas", 9))
        scroll = ttk.Scrollbar(log_wrap, orient=tk.VERTICAL, command=self.log_text.yview)
        self.log_text.configure(yscrollcommand=scroll.set)
        self.log_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll.pack(side=tk.RIGHT, fill=tk.Y)
        attach_wheel_area(
            self.log_text,
            on_scroll_y=text_yscroll(self.log_text),
            area_id=f"aps-atlas-log-{id(self)}",
        )

        self.folder_var.trace_add("write", lambda *_: self._refresh_preview())

    def _refresh_preview(self) -> None:
        folder = self.folder_var.get().strip()
        self.atlas_preview.load_folder(folder or None)

    def _inline_hint(self, text: str, *, ok: bool | None = None) -> None:
        set_inline_status(self._inline_status_lbl, self._inline_status_var, text, ok=ok)

    def on_validate_atlas_meta(self) -> None:
        from rust_engine_mcp.aps_atlas_qc import format_atlas_qc_display, validate_atlas_folder

        folder = self.folder_var.get().strip()
        if not folder:
            self._inline_hint("Choose PNG folder first.")
            return
        path = Path(folder)
        report, lines = validate_atlas_folder(path)
        meta = None
        meta_path = path / "atlas_meta.json"
        if meta_path.is_file():
            try:
                meta = json.loads(meta_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                meta = None
        text, fg = format_atlas_qc_display(report, lines, meta=meta if isinstance(meta, dict) else None)
        passed = bool(report and report.status == "passed")
        set_inline_status(self._atlas_qc_lbl, self._atlas_qc_var, text, ok=passed if report else None)
        status = report.status if report else "failed"
        self._log(f"atlas meta validate · {status}")
        if status != "passed":
            set_inline_status(self._inline_status_lbl, self._inline_status_var, text, ok=False)
        else:
            set_inline_status(self._inline_status_lbl, self._inline_status_var, text, ok=True)

    def on_open_png_folder(self) -> None:
        folder = self.folder_var.get().strip()
        if not folder:
            self._inline_hint("Choose PNG folder first.")
            return
        import os

        os.startfile(folder)  # noqa: S606 — Windows artist workflow
        self._log(f"opened folder {folder}")

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
            if self._atlas:
                self._atlas.set_tile_batch_path(path)

    def on_browse_folder(self) -> None:
        path = filedialog.askdirectory()
        if path:
            self.folder_var.set(path)
            if self._atlas:
                self._atlas.set_atlas_folder(path)

    def on_batch_from_variant_set(self) -> None:
        data = self.state.variant_set_data
        if not data:
            self._inline_hint("Load or create a variant set first (Variants tab).")
            return
        batch = variant_set.expand_variant_set_to_tile_batch(data)
        tmp = Path(tempfile.gettempdir()) / f"{batch['batch_id']}.json"
        tmp.write_text(json.dumps(batch, indent=2) + "\n", encoding="utf-8")
        self.batch_json_var.set(str(tmp))
        if self._atlas:
            self._atlas.set_tile_batch_path(str(tmp))
        self._log(f"Variant set expanded into a tile job → {tmp}")
        self._inline_hint(f"Prepared a tile job: {tmp.name}", ok=True)

    def on_run_batch(self) -> None:
        path = self.batch_json_var.get().strip()
        if not path:
            self._inline_hint("Choose a tile job JSON file.")
            return
        if not self._start_job:
            self._run_batch_sync(path)
            return

        def worker(cancel: threading.Event) -> JobResult:
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            self._log(f"tile-batch-run {path}")
            code, log = run_tile_batch(Path(path))
            ok = code == 0
            return JobResult(ok, "Tile batch OK" if ok else "Tile batch failed", detail=log)

        def on_done(record: JobRecord) -> None:
            if record.state == JobState.CANCELLED:
                self._inline_hint("Tile batch cancelled.", ok=False)
                return
            if record.result and record.result.ok:
                try:
                    batch = json.loads(Path(path).read_text(encoding="utf-8"))
                    bid = batch.get("batch_id")
                    folder = repo_root() / "assets/staging/tiles" / str(bid)
                    if folder.is_dir():
                        self.folder_var.set(str(folder))
                        if self._atlas:
                            self._atlas.set_atlas_folder(str(folder))
                except Exception:  # noqa: BLE001
                    pass
                self._inline_hint("Tile batch finished — see status log.", ok=True)
            else:
                self._inline_hint("Tile batch failed — see status log.", ok=False)

        self._start_job(
            "Tile batch",
            worker,
            on_done=on_done,
            button=self.run_batch_btn,
            button_label="Run tile batch",
        )

    def _run_batch_sync(self, path: str) -> None:
        self._log(f"tile-batch-run {path}")
        code, log = run_tile_batch(Path(path))
        self._log(log)
        if code != 0:
            self._inline_hint("Tile batch failed.", ok=False)
            return
        try:
            batch = json.loads(Path(path).read_text(encoding="utf-8"))
            bid = batch.get("batch_id")
            folder = repo_root() / "assets/staging/tiles" / str(bid)
            if folder.is_dir():
                self.folder_var.set(str(folder))
                if self._atlas:
                    self._atlas.set_atlas_folder(str(folder))
        except Exception:  # noqa: BLE001
            pass
        self._inline_hint("Tile batch finished.", ok=True)

    def on_pack(self) -> None:
        folder = self.folder_var.get().strip()
        if not folder:
            self._inline_hint("Choose PNG folder.")
            return
        if not self._start_job:
            self._pack_sync(folder)
            return
        folder_path = Path(folder)
        keyframe = self.keyframe_rename_var.get()

        def worker(cancel: threading.Event) -> JobResult:
            if cancel.is_set():
                return JobResult(False, "Cancelled")
            code, log = pack_tile_folder(folder_path, keyframe_rename=keyframe)
            ok = code == 0
            return JobResult(ok, "Pack OK" if ok else "Pack failed", detail=log)

        def on_done(record: JobRecord) -> None:
            if record.state == JobState.CANCELLED:
                self._inline_hint("Pack cancelled.", ok=False)
                return
            if record.result and record.result.ok:
                atlas = find_latest_atlas_in(folder_path)
                self._refresh_preview()
                self._inline_hint(f"Atlas OK — {atlas.name if atlas else 'see log'}", ok=True)
            else:
                self._inline_hint("Pack failed — see status log.", ok=False)

        self._start_job(
            "Pack atlas",
            worker,
            on_done=on_done,
            button=self.pack_btn,
            button_label="Pack atlas",
        )

    def _pack_sync(self, folder: str) -> None:
        code, log = pack_tile_folder(Path(folder), keyframe_rename=self.keyframe_rename_var.get())
        self._log(log)
        if code != 0:
            self._inline_hint("Pack failed.", ok=False)
            return
        atlas = find_latest_atlas_in(Path(folder))
        self._refresh_preview()
        self._inline_hint(f"Atlas OK — {atlas.name if atlas else 'see log'}", ok=True)
    def on_lod0(self) -> None:
        code, log = run_lod0_batch(
            self.lod_batch_var.get(),
            step=_LANE_LOD_PHASE_TO_STEP.get(self.lod_phase_var.get(), self.lod_phase_var.get()),
        )
        self._log(log)
        if code != 0:
            self._inline_hint("lod0 batch failed — see status log.", ok=False)
            return
        self._inline_hint("lod0 batch finished — see status log.", ok=True)

    def on_light_blend(self) -> None:
        code, log = open_light_blend()
        self._log(log)

    def on_keyframe_addon(self) -> None:
        code, log = open_keyframe_render_addon()
        self._log(log)
