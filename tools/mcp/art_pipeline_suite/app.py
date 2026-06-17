"""Art Pipeline Suite — Buildings: Catalog|Assembly|… · Landscape: Presets|Grammar|States|Atlas."""



from __future__ import annotations



import tkinter as tk

from tkinter import ttk



from .assembly_panel import AssemblyPanel

from .atlas_panel import AtlasPanel

from .catalog import CatalogPanel

from .job_controller import DoneCallback, JobController, JobRecord, JobState, JobWorker

from .job_strip import JobStrip

from .landscape_grammar_panel import LandscapeGrammarPanel

from .landscape_presets_panel import LandscapePresetsPanel

from .landscape_states_panel import LandscapeStatesPanel

from .scrollable import ScrollableFrame

from .state import SuiteState, ArtDomain

from .materials_panel import MaterialsPanel

from .variants_panel import VariantsPanel

from .aps_tooltips import bind_aps_tooltip, hide_all_tooltips

from .aps_inline_feedback import flow_prerequisite_message

from .aps_theme import (
    COLOR_INPUT_BG,
    COLOR_LANE_BUILDING,
    COLOR_LANE_LANDSCAPE,
    COLOR_MUTED,
    COLOR_PANEL_BG,
    FONT_HINT,
    FONT_UI_BOLD,
    PAD_MD,
    DEFAULT_WINDOW_SIZE,
    MIN_WINDOW_SIZE,
    init_aps_ttk,
    wrap_for_widget,
)

from .domain_router import (

    authority_for,

    clear_cross_lane_selection,

    flow_verbs_for,

    load_active_lane,

    save_active_lane,

)

from .aps_collapsible import CollapsibleSection

from .aps_scroll import init_aps_scroll

from .pipeline_status_bar import PipelineStatusBar

from .status_log_panel import StatusLogPanel



# DES-APS-E1-IA-OPTION-D-001 — dual notebook page sets (not label-rename on one 5-tab row).

OPTION_D_DUAL_NOTEBOOK = True





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

        self.state.art_domain = load_active_lane()

        self.jobs = JobController(ui_scheduler=lambda fn: self.after(0, fn))

        self.status_summary_var = tk.StringVar(value="Ready")

        self._lane_var = tk.StringVar(value=self.state.art_domain)

        self._flow_buttons: dict[str, ttk.Button] = {}

        self._build_lane_bar()

        self._build_flow_bars()

        self._build_authority_strip()

        self.pipeline_status = PipelineStatusBar(self, self.state)

        self.pipeline_status.pack(fill=tk.X, padx=8)

        self.job_strip = JobStrip(self, self.jobs)

        self._status_log_frame = ttk.Frame(self, padding=(8, 0, 8, 8))

        self._pack_status_log()

        self._notebook_container = ttk.Frame(self)

        self._notebook_container.pack(fill=tk.BOTH, expand=True, padx=8, pady=4)

        self._notebook_buildings = ttk.Notebook(self._notebook_container)

        self._notebook_landscape = ttk.Notebook(self._notebook_container)

        self.notebook = self._notebook_buildings

        self._build_buildings_tabs()

        self._build_landscape_tabs()

        self._apply_lane(self.state.art_domain, log=False)
        self._notebook_buildings.bind("<<NotebookTabChanged>>", self._on_tab_changed)
        self._notebook_landscape.bind("<<NotebookTabChanged>>", self._on_tab_changed)
        self.bind("<Control-Key-1>", lambda _e: self._apply_lane(ArtDomain.BUILDINGS.value))
        self.bind("<Control-Key-2>", lambda _e: self._apply_lane(ArtDomain.LANDSCAPE.value))

    def _build_lane_bar(self) -> None:
        wrap = ttk.Frame(self, padding=(PAD_MD, 6, PAD_MD, 0))
        wrap.pack(fill=tk.X)
        bar = ttk.Frame(wrap)
        bar.pack(fill=tk.X)
        ttk.Label(bar, text="Lane:", font=FONT_UI_BOLD).pack(side=tk.LEFT, padx=(0, 6))
        self._lane_buildings_btn = ttk.Radiobutton(
            bar,
            text="Buildings",
            style="Aps.Lane.TRadiobutton",
            value=ArtDomain.BUILDINGS.value,
            variable=self._lane_var,
            command=self._on_lane_selected,
        )
        self._lane_buildings_btn.pack(side=tk.LEFT, padx=4)
        self._lane_landscape_btn = ttk.Radiobutton(
            bar,
            text="Landscape",
            style="Aps.Lane.TRadiobutton",
            value=ArtDomain.LANDSCAPE.value,
            variable=self._lane_var,
            command=self._on_lane_selected,
        )
        self._lane_landscape_btn.pack(side=tk.LEFT, padx=4)
        self._lane_underline = tk.Frame(wrap, height=3, bg=COLOR_LANE_BUILDING)
        self._lane_underline.pack(fill=tk.X, pady=(2, 0))
        chip_frame = tk.Frame(bar, relief=tk.RIDGE, borderwidth=1, padx=8, pady=2)
        chip_frame.pack(side=tk.LEFT, padx=(12, 0))
        self._lane_chip = ttk.Label(chip_frame, text="", font=FONT_HINT)
        self._lane_chip.pack()



    def _on_lane_selected(self) -> None:

        self._apply_lane(self._lane_var.get())



    def _apply_lane(self, lane: str, *, log: bool = True) -> None:

        lane = ArtDomain.LANDSCAPE.value if lane == ArtDomain.LANDSCAPE.value else ArtDomain.BUILDINGS.value

        clear_cross_lane_selection(self.state, lane)

        self.state.art_domain = lane

        self._lane_var.set(lane)

        if lane == ArtDomain.LANDSCAPE.value:

            self._notebook_buildings.pack_forget()

            self._notebook_landscape.pack(fill=tk.BOTH, expand=True)

            self.notebook = self._notebook_landscape

            self._flow_buildings.pack_forget()

            self._flow_landscape.pack(fill=tk.X)

        else:

            self._notebook_landscape.pack_forget()

            self._notebook_buildings.pack(fill=tk.BOTH, expand=True)

            self.notebook = self._notebook_buildings

            self._flow_landscape.pack_forget()

            self._flow_buildings.pack(fill=tk.X)

        self._authority_var.set(authority_for(lane))
        chip = "Buildings lane" if lane == ArtDomain.BUILDINGS.value else "Landscape lane"
        fg = COLOR_LANE_BUILDING if lane == ArtDomain.BUILDINGS.value else COLOR_LANE_LANDSCAPE
        self._lane_chip.configure(text=chip, foreground=fg)
        self._lane_underline.configure(bg=fg)
        if lane == ArtDomain.BUILDINGS.value:
            self._lane_buildings_btn.configure(text="▣ Buildings")
            self._lane_landscape_btn.configure(text="Landscape")
        else:
            self._lane_buildings_btn.configure(text="Buildings")
            self._lane_landscape_btn.configure(text="▣ Landscape")
        if hasattr(self, "_authority_border"):
            self._authority_border.configure(bg=fg)
        self._authority_lbl.configure(foreground=fg)

        self.pipeline_status.set_domain(lane)

        if lane == ArtDomain.LANDSCAPE.value:

            self.landscape_presets.refresh_list()

            self.landscape_grammar.refresh_from_state()

            self.landscape_states.refresh_from_state()

            self.landscape_atlas.set_domain(lane)

        else:

            self.atlas.set_domain(lane)

        save_active_lane(lane)

        self.pipeline_status.refresh()

        if log:

            self._log(f"lane → {lane} · tab set swapped")



    def _add_scrollable_tab(self, notebook: ttk.Notebook, panel_cls, text: str, **panel_kw):

        tab_root = ttk.Frame(notebook)

        scroll = ScrollableFrame(tab_root, enable_horizontal=True)

        scroll.pack(fill=tk.BOTH, expand=True)

        panel = panel_cls(scroll.interior, **panel_kw)

        panel.pack(fill=tk.BOTH, expand=True)

        notebook.add(tab_root, text=text)

        panel._aps_tab_root = tab_root

        return panel



    def _build_flow_bars(self) -> None:

        self._flow_buildings = ttk.Frame(self, padding=8)

        self._flow_landscape = ttk.Frame(self, padding=8)

        self._flow_buildings.pack(fill=tk.X)

        for frame, lane, handlers in (

            (self._flow_buildings, ArtDomain.BUILDINGS.value, self._buildings_flow_handlers()),

            (self._flow_landscape, ArtDomain.LANDSCAPE.value, self._landscape_flow_handlers()),

        ):

            ttk.Label(frame, text="Flow:").pack(side=tk.LEFT)

            for key, label in flow_verbs_for(lane):

                handler = handlers[key]

                btn = ttk.Button(frame, text=label, command=handler)

                btn.pack(side=tk.LEFT, padx=4)

                self._flow_buttons[key] = btn

            ttk.Separator(frame, orient=tk.VERTICAL).pack(side=tk.LEFT, fill=tk.Y, padx=8)

            ttk.Label(

                frame,

                text="All actions call rust_engine_mcp CLI/MCP — agents use the same APIs.",

                foreground=COLOR_MUTED,

            ).pack(side=tk.LEFT)

        self._flow_hint_var = tk.StringVar(value="")

        self._flow_hint_lbl = ttk.Label(

            self._flow_buildings, textvariable=self._flow_hint_var, foreground="#8b0000", font=FONT_HINT

        )

        self._flow_hint_lbl.pack(side=tk.LEFT, padx=(12, 0))

        self._flow_hint_lbl_land = ttk.Label(

            self._flow_landscape, textvariable=self._flow_hint_var, foreground="#8b0000", font=FONT_HINT

        )

        self._flow_hint_lbl_land.pack(side=tk.LEFT, padx=(12, 0))

        self._flow_landscape.pack_forget()



    def _buildings_flow_handlers(self) -> dict[str, object]:

        return {

            "send_to_assembly": self.on_send_to_assembly,

            "bake_variants": self.on_bake_variants,

            "pack_atlas": self.on_pack_atlas,

        }



    def _landscape_flow_handlers(self) -> dict[str, object]:

        return {

            "generate_grammar": self.on_generate_grammar,

            "bake_states": self.on_bake_states,

            "pack_lg5_atlas": self.on_pack_lg5_atlas,

        }



    def _show_flow_prerequisite(self, action: str) -> bool:

        msg = flow_prerequisite_message(action, self.state)

        if msg:

            self._flow_hint_var.set(msg)

            self._log(msg)

            return False

        self._flow_hint_var.set("")

        return True



    def _build_authority_strip(self) -> None:
        outer = ttk.Frame(self, padding=(PAD_MD, 0, PAD_MD, 2))
        outer.pack(fill=tk.X)
        row = ttk.Frame(outer)
        row.pack(fill=tk.X)
        self._authority_border = tk.Frame(row, width=4, bg=COLOR_LANE_BUILDING)
        self._authority_border.pack(side=tk.LEFT, fill=tk.Y)
        self._authority_var = tk.StringVar(value=authority_for(self.state.art_domain))
        self._authority_lbl = ttk.Label(
            row,
            textvariable=self._authority_var,
            font=FONT_HINT,
            foreground=COLOR_LANE_BUILDING,
            wraplength=900,
        )
        self._authority_lbl.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(6, 0))

        def _wrap(_event=None) -> None:
            self._authority_lbl.configure(wraplength=wrap_for_widget(outer, minimum=480))

        outer.bind("<Configure>", _wrap)



    def _build_buildings_tabs(self) -> None:

        job_kw = {"start_job": self._start_job}

        nb = self._notebook_buildings

        self.catalog = self._add_scrollable_tab(

            nb, CatalogPanel, "Catalog", state=self.state, on_select=self._on_catalog_select

        )

        bind_aps_tooltip(self.catalog, "tab_catalog")

        self.assembly = self._add_scrollable_tab(

            nb,

            AssemblyPanel,

            "Assembly",

            state=self.state,

            on_log=self._log,

            on_open_in_materials=self._open_material_in_materials_tab,

            **job_kw,

        )

        bind_aps_tooltip(self.assembly, "tab_assembly")

        self.materials = self._add_scrollable_tab(

            nb,

            MaterialsPanel,

            "Materials",

            state=self.state,

            on_log=self._log,

            on_open_in_assembly=self._open_material_in_assembly,

            **job_kw,

        )

        bind_aps_tooltip(self.materials, "tab_materials")

        self.variants = self._add_scrollable_tab(

            nb, VariantsPanel, "Variants", state=self.state, on_log=self._log, **job_kw

        )

        bind_aps_tooltip(self.variants, "tab_variants")

        self.atlas = self._add_scrollable_tab(nb, AtlasPanel, "Atlas", state=self.state, on_log=self._log, **job_kw)

        bind_aps_tooltip(self.atlas, "tab_atlas")

        self.atlas.set_domain(ArtDomain.BUILDINGS.value)



    def _build_landscape_tabs(self) -> None:

        job_kw = {"start_job": self._start_job}

        nb = self._notebook_landscape

        self.landscape_presets = self._add_scrollable_tab(

            nb,

            LandscapePresetsPanel,

            "Presets",

            state=self.state,

            on_select=self._on_landscape_preset_select,

            on_log=self._log,

        )

        self.landscape_grammar = self._add_scrollable_tab(

            nb,

            LandscapeGrammarPanel,

            "Grammar",

            state=self.state,

            on_log=self._log,

        )

        self.landscape_states = self._add_scrollable_tab(

            nb,

            LandscapeStatesPanel,

            "States",

            state=self.state,

            on_log=self._log,

        )

        self.landscape_atlas = self._add_scrollable_tab(

            nb, AtlasPanel, "Atlas", state=self.state, on_log=self._log, **job_kw

        )

        self.landscape_atlas.set_domain(ArtDomain.LANDSCAPE.value)



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

        hide_all_tooltips()

        self.pipeline_status.refresh()



    def _on_catalog_select(self, _rec) -> None:

        self._log(f"catalog select: {self.state.selected_module_id}")



    def _on_landscape_preset_select(self, preset_id: str) -> None:

        self.landscape_grammar.refresh_from_state()

        self.landscape_states.refresh_from_state()

        self.pipeline_status.refresh()

        self._log(f"landscape preset select: {preset_id}")



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

        if not self._show_flow_prerequisite("send_to_assembly"):

            return

        self.assembly.sync_from_state()

        self.notebook.select(self.assembly._aps_tab_root)

        self._log(f"assembly ← style {self.state.style_pack_id}")



    def on_bake_variants(self) -> None:

        if not self._show_flow_prerequisite("bake_variants"):

            return

        self.notebook.select(self.variants._aps_tab_root)

        if not self.state.variant_set_data:

            self.variants.on_new_from_assembly()

        self.atlas.on_batch_from_variant_set()

        self.notebook.select(self.atlas._aps_tab_root)

        self._log("bake variants → tile_batch prepared on Atlas tab")



    def on_pack_atlas(self) -> None:

        if not self._show_flow_prerequisite("pack_atlas"):

            return

        self.atlas.sync_folder_from_state()

        self.notebook.select(self.atlas._aps_tab_root)

        folder = self.state.atlas_folder

        if folder:

            self.atlas.on_pack()

        else:

            self._log("Pack atlas — run tile batch or set PNG folder on Atlas tab")



    def on_generate_grammar(self) -> None:

        if not self._show_flow_prerequisite("generate_grammar"):

            return

        self.landscape_grammar.refresh_from_state()

        self.landscape_grammar.mark_saved()

        self.notebook.select(self.landscape_grammar._aps_tab_root)

        self.landscape_states.refresh_from_state()

        self.pipeline_status.refresh()



    def on_bake_states(self) -> None:

        if not self._show_flow_prerequisite("bake_states"):

            return

        from rust_engine_mcp.paths import repo_root

        expanded_batch = (
            repo_root() / "assets/staging/specs/tile_batch_landscape_expanded_v1.json"
        )
        if expanded_batch.is_file():
            self.state.tile_batch_path = str(expanded_batch)
            self.landscape_atlas.batch_json_var.set(str(expanded_batch))

        self.landscape_states.mark_states_ready()

        self.notebook.select(self.landscape_atlas._aps_tab_root)

        self.notebook.select(self.landscape_atlas._aps_tab_root)

        self.pipeline_status.refresh()

        self._log("bake states → tile_batch scaffold on Landscape Atlas tab")



    def on_pack_lg5_atlas(self) -> None:

        if not self._show_flow_prerequisite("pack_lg5_atlas"):

            return

        self.landscape_atlas.sync_folder_from_state()

        self.notebook.select(self.landscape_atlas._aps_tab_root)

        folder = self.state.atlas_folder

        if folder:
            self.landscape_atlas.on_pack()
            self.state.landscape_stamp_registered = True
            self.pipeline_status.refresh()
            self._log("Pack LG-5 atlas — stamp registered (scaffold)")

        else:

            self._log("Pack LG-5 atlas — run bake states or set PNG folder on Landscape Atlas tab")





def run_app() -> None:

    app = ArtPipelineSuiteApp()

    app.mainloop()

