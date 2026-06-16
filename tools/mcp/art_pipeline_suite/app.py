"""Art Pipeline Suite — Catalog | Assembly | Variants | Atlas."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

from .assembly_panel import AssemblyPanel
from .atlas_panel import AtlasPanel
from .catalog import CatalogPanel
from .job_controller import DoneCallback, JobController, JobRecord, JobState, JobWorker
from .job_strip import JobStrip
from .scrollable import ScrollableFrame
from .state import SuiteState
from .materials_panel import MaterialsPanel
from .variants_panel import VariantsPanel
from .aps_tooltips import bind_aps_tooltip
from .aps_theme import AUTHORITY_STRIP, COLOR_MUTED, FONT_HINT, DEFAULT_WINDOW_SIZE, MIN_WINDOW_SIZE, init_aps_ttk, wrap_for_widget
from .aps_collapsible import CollapsibleSection
from .aps_scroll import init_aps_scroll
from .pipeline_status_bar import PipelineStatusBar
from .status_log_panel import StatusLogPanel


class ArtPipelineSuiteApp(tk.Tk):
    def __init__(self) -> None:
        super().__init__()
        init_aps_ttk(self)
        init_aps_scroll(self)
        self.title("Rust Engine — Art Pipeline Suite")
        w, h = DEFAULT_WINDOW_SIZE
        self.geometry(f"{w}x{h}")
        self.minsize(*MIN_WINDOW_SIZE)
        self.state = SuiteState()
        self.jobs = JobController(ui_scheduler=lambda fn: self.after(0, fn))
        self.status_summary_var = tk.StringVar(value="Ready")
        self._build_flow_bar()
        self._build_authority_strip()
        self.pipeline_status = PipelineStatusBar(self, self.state)
        self.pipeline_status.pack(fill=tk.X, padx=8)
        self.job_strip = JobStrip(self, self.jobs)
        self._status_log_frame = ttk.Frame(self, padding=(8, 0, 8, 8))
        self._pack_status_log()
        self._build_tabs()
        self.notebook.bind("<<NotebookTabChanged>>", self._on_tab_changed)

    def _add_scrollable_tab(self, panel_cls, text: str, **panel_kw):
        """Notebook page with vertical scroll (horizontal when content overflows width)."""
        tab_root = ttk.Frame(self.notebook)
        scroll = ScrollableFrame(tab_root, enable_horizontal=True)
        scroll.pack(fill=tk.BOTH, expand=True)
        panel = panel_cls(scroll.interior, **panel_kw)
        panel.pack(fill=tk.BOTH, expand=True)
        self.notebook.add(tab_root, text=text)
        panel._aps_tab_root = tab_root
        return panel

    def _build_flow_bar(self) -> None:
        bar = ttk.Frame(self, padding=8)
        bar.pack(fill=tk.X)
        ttk.Label(bar, text="Flow:").pack(side=tk.LEFT)
        btn_asm = ttk.Button(bar, text="Send to Assembly", command=self.on_send_to_assembly)
        btn_asm.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(btn_asm, "flow_send_assembly")
        btn_var = ttk.Button(bar, text="Bake variants", command=self.on_bake_variants)
        btn_var.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(btn_var, "flow_bake_variants")
        btn_pack = ttk.Button(bar, text="Pack atlas", command=self.on_pack_atlas)
        btn_pack.pack(side=tk.LEFT, padx=4)
        bind_aps_tooltip(btn_pack, "flow_pack_atlas")
        ttk.Separator(bar, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=8)
        ttk.Label(
            bar,
            text="All actions call rust_engine_mcp CLI/MCP — agents use the same APIs.",
            foreground=COLOR_MUTED,
        ).pack(side=tk.LEFT)

    def _build_authority_strip(self) -> None:
        frame = ttk.Frame(self, padding=(8, 0, 8, 2))
        frame.pack(fill=tk.X)
        self._authority_var = tk.StringVar(value=AUTHORITY_STRIP)
        self._authority_lbl = ttk.Label(
            frame,
            textvariable=self._authority_var,
            font=FONT_HINT,
            foreground="#0a4a7a",
            wraplength=900,
        )
        self._authority_lbl.pack(anchor=tk.W)

        def _wrap(_event=None) -> None:
            self._authority_lbl.configure(wraplength=wrap_for_widget(frame, minimum=480))

        frame.bind("<Configure>", _wrap)

    def _build_tabs(self) -> None:
        self.notebook = ttk.Notebook(self)
        self.notebook.pack(fill=tk.BOTH, expand=True, padx=8, pady=4)
        job_kw = {"start_job": self._start_job}

        self.catalog = self._add_scrollable_tab(
            CatalogPanel, "Catalog", state=self.state, on_select=self._on_catalog_select
        )
        bind_aps_tooltip(self.catalog, "tab_catalog")
        self.assembly = self._add_scrollable_tab(
            AssemblyPanel,
            "Assembly",
            state=self.state,
            on_log=self._log,
            on_open_in_materials=self._open_material_in_materials_tab,
            **job_kw,
        )
        bind_aps_tooltip(self.assembly, "tab_assembly")
        self.materials = self._add_scrollable_tab(
            MaterialsPanel,
            "Materials",
            state=self.state,
            on_log=self._log,
            on_open_in_assembly=self._open_material_in_assembly,
            **job_kw,
        )
        bind_aps_tooltip(self.materials, "tab_materials")
        self.variants = self._add_scrollable_tab(
            VariantsPanel, "Variants", state=self.state, on_log=self._log, **job_kw
        )
        bind_aps_tooltip(self.variants, "tab_variants")
        self.atlas = self._add_scrollable_tab(
            AtlasPanel, "Atlas", state=self.state, on_log=self._log, **job_kw
        )
        bind_aps_tooltip(self.atlas, "tab_atlas")

    def _pack_status_log(self) -> None:
        self._status_log_frame.pack(fill=tk.BOTH, expand=False)
        self._status_log_section = CollapsibleSection(
            self._status_log_frame,
            "Status log",
            expanded=False,
            padding=2,
        )
        self._status_log_section.pack(fill=tk.BOTH, expand=True)
        ttk.Label(
            self._status_log_section.body,
            textvariable=self.status_summary_var,
            foreground="#333",
        ).pack(anchor=tk.W)
        self.status_log = StatusLogPanel(self._status_log_section.body, height=5)
        self.status_log.pack(fill=tk.BOTH, expand=True, pady=(2, 0))

    def _log(self, line: str) -> None:
        self.state.append_log(line)
        self.status_log.append(line)
        self.status_summary_var.set(line[:80])
        self.pipeline_status.refresh()

    def _poll_jobs(self) -> None:
        job = self.jobs.active_job()
        self.job_strip.sync(job)
        if job and job.state in (JobState.RUNNING, JobState.CANCELLING):
            self.after(100, self._poll_jobs)

    def _start_job(
        self,
        label: str,
        worker: JobWorker,
        *,
        on_done: DoneCallback | None = None,
        button: ttk.Button | None = None,
        button_label: str | None = None,
    ) -> str | None:
        if self.jobs.is_busy():
            self._log("Another job is running — wait or Cancel")
            return None
        orig_text = button.cget("text") if button else None
        if button:
            button.configure(state=tk.DISABLED, text=f"⟳ {button_label or label}…")

        def _done(record: JobRecord) -> None:
            self.job_strip.sync(None)
            if button:
                button.configure(state=tk.NORMAL, text=button_label or orig_text or label)
            if record.result:
                if record.result.detail:
                    for chunk in record.result.detail.splitlines():
                        if chunk.strip():
                            self._log(chunk)
                summary = record.result.summary
                if record.state == JobState.CANCELLED:
                    summary = "Cancelled"
                elif record.state == JobState.FAILED and not record.result.ok:
                    summary = record.result.summary or "Job failed"
                self._log(summary)
            if on_done:
                on_done(record)

        try:
            job_id = self.jobs.run(label, worker, on_done=_done)
        except RuntimeError as exc:
            if button:
                button.configure(state=tk.NORMAL, text=button_label or orig_text or label)
            self._log(str(exc))
            return None
        self.job_strip.sync(self.jobs.active_job())
        self._poll_jobs()
        return job_id

    def _on_tab_changed(self, _event=None) -> None:
        self.pipeline_status.refresh()

    def _on_catalog_select(self, _rec) -> None:
        self._log(f"catalog select: {self.state.selected_module_id}")

    def _open_material_in_assembly(self, profile_id: str) -> None:
        self.notebook.select(self.assembly._aps_tab_root)
        self.assembly.material_browser.highlight_profile(profile_id)
        self.assembly.show_material_assign_callout(profile_id)
        self._log(f"materials → assembly · {profile_id}")

    def _open_material_in_materials_tab(self, profile_id: str) -> None:
        self.notebook.select(self.materials._aps_tab_root)
        self.materials.highlight_profile(profile_id)
        self._log(f"assembly → materials · {profile_id}")

    def on_send_to_assembly(self) -> None:
        self.assembly.sync_from_state()
        self.notebook.select(self.assembly._aps_tab_root)
        self._log(f"assembly ← style {self.state.style_pack_id}")

    def on_bake_variants(self) -> None:
        if not self.state.variant_set_data and not self.state.assembly_id:
            self._log("Bake variants — create assembly snapshot and variant set first (Assembly → Variants)")
            return
        self.notebook.select(self.variants._aps_tab_root)
        if not self.state.variant_set_data:
            self.variants.on_new_from_assembly()
        self.atlas.on_batch_from_variant_set()
        self.notebook.select(self.atlas._aps_tab_root)
        self._log("bake variants → tile_batch prepared on Atlas tab")

    def on_pack_atlas(self) -> None:
        self.atlas.sync_folder_from_state()
        self.notebook.select(self.atlas._aps_tab_root)
        folder = self.state.atlas_folder
        if folder:
            self.atlas.on_pack()
        else:
            self._log("Pack atlas — run tile batch or set PNG folder on Atlas tab")


def run_app() -> None:
    app = ArtPipelineSuiteApp()
    app.mainloop()
