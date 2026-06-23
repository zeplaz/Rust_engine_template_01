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
    COLOR_TEXT_BODY,
    FONT_HINT,
    FONT_UI_BOLD,
    GAP_MD,
    DEFAULT_WINDOW_SIZE,
    MIN_WINDOW_SIZE,
    init_aps_ttk,
    wrap_for_widget,
)

from .domain_router import (

    authority_for,

    clear_cross_lane_selection,

    load_active_lane,

    pipeline_steps_for,

    save_active_lane,

)

from .aps_collapsible import CollapsibleSection
from .aps_headless import apply_headless_root, headless_tests_enabled

from .aps_scroll import init_aps_scroll

from .pipeline_status_bar import PipelineStatusBar

from .status_log_panel import StatusLogPanel



# DES-APS-E1-IA-OPTION-D-001 — dual notebook page sets (not label-rename on one 5-tab row).

OPTION_D_DUAL_NOTEBOOK = True





class ArtPipelineSuiteApp(tk.Tk):

    def __init__(self) -> None:

        super().__init__()

        if headless_tests_enabled():
            apply_headless_root(self)

        init_aps_ttk(self)

        init_aps_scroll(self)

        self.title("Rust Engine — Art Pipeline Suite")

        if headless_tests_enabled():
            self.geometry("1x1+-20000+-20000")
        else:
            w, h = DEFAULT_WINDOW_SIZE
            self.geometry(f"{w}x{h}")

        self.minsize(*MIN_WINDOW_SIZE)

        self.state = SuiteState()

        self.state.art_domain = load_active_lane()

        self.jobs = JobController(ui_scheduler=lambda fn: self.after(0, fn))

        self.status_summary_var = tk.StringVar(value="Ready")

        self._lane_var = tk.StringVar(value=self.state.art_domain)

        # Tracks the lane currently applied to the chrome/notebooks so a redundant
        # click on the already-active lane is a cheap no-op (no disk reads, no
        # widget rebuilds). Stays None until the first _apply_lane lands.
        self._applied_lane: str | None = None

        self._build_lane_bar()

        # P7 Slice B — flow-verb handlers are kept (the spine's advance action runs
        # them) but the always-on lane flow-verb ROW is dropped: the pipeline spine
        # is now the single nav surface.
        self._flow_handlers = {
            **self._buildings_flow_handlers(),
            **self._landscape_flow_handlers(),
        }

        self._chrome_row2 = ttk.Frame(self, padding=(GAP_MD, 0, GAP_MD, 2))

        self._build_authority_strip(self._chrome_row2)

        self.pipeline_status = PipelineStatusBar(
            self._chrome_row2,
            self.state,
            on_step_click=self._on_pipeline_step,
            on_advance=self._on_spine_advance,
            flow_ready=self._flow_verb_ready,
            flow_blocked_reason=self._flow_verb_blocked_reason,
        )

        self.pipeline_status.pack(side=tk.RIGHT, fill=tk.X, expand=True)

        self._chrome_row2.pack(fill=tk.X)

        self.job_strip = JobStrip(self, self.jobs)

        # P7 Slice C — ≤2 always-on chrome rows ABOVE the work area (lane bar +
        # the pipeline-spine row). The collapsible status log moves BELOW the
        # notebook so it no longer eats height above the work surface.
        self._status_log_frame = ttk.Frame(self, padding=(8, 0, 8, 8))

        self._notebook_container = ttk.Frame(self)

        self._notebook_container.pack(side=tk.TOP, fill=tk.BOTH, expand=True, padx=8, pady=4)

        self._pack_status_log()

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
        self._onboarding_panel = None
        self._maybe_onboarding()

    def _maybe_onboarding(self) -> None:
        from rust_engine_mcp.aps_uiux_onboard import (
            load_onboarding_seen,
            mark_onboarding_seen,
            onboarding_greeting_lines,
        )

        if load_onboarding_seen():
            return
        mark_onboarding_seen()
        # Plain-language greeting also lands in the log so it survives dismissal.
        self._log(" · ".join(onboarding_greeting_lines()))
        self._show_onboarding_panel()

    def _show_onboarding_panel(self) -> None:
        """P5.6 — dismissible first-run "How this works" card over the work area."""
        from .aps_onboarding_panel import OnboardingPanel

        if getattr(self, "_onboarding_panel", None) is not None:
            return
        panel = OnboardingPanel(self._notebook_container, on_dismiss=self._dismiss_onboarding)
        panel.place(relx=0.5, rely=0.0, anchor=tk.N, relwidth=0.9)
        self._onboarding_panel = panel

    def _dismiss_onboarding(self) -> None:
        self._onboarding_panel = None
        self._log("Onboarding dismissed — reopen the steps any time from the Next step line up top.")

    def _build_lane_bar(self) -> None:
        wrap = ttk.Frame(self, padding=(GAP_MD, 6, GAP_MD, 0))
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
        self._lane_underline = tk.Frame(wrap, height=2, bg=COLOR_LANE_BUILDING)
        self._lane_underline.pack(fill=tk.X, pady=(2, 0))
        # P7 Slice B — the lane bar no longer carries flow-verb buttons; the
        # pipeline spine (row 2) is the single advance surface.



    def _on_lane_selected(self) -> None:

        self._apply_lane(self._lane_var.get())



    def _apply_lane(self, lane: str, *, log: bool = True) -> None:

        lane = ArtDomain.LANDSCAPE.value if lane == ArtDomain.LANDSCAPE.value else ArtDomain.BUILDINGS.value

        # Re-selecting the already-applied lane is a no-op: the heavy work (disk
        # reads, pill rebuilds, lane persistence) only matters on an actual swap.
        # The lane radio var is re-pinned so the radiobutton stays consistent.
        if lane == self._applied_lane:

            self._lane_var.set(lane)

            return

        clear_cross_lane_selection(self.state, lane)

        self.state.art_domain = lane

        self._lane_var.set(lane)

        # --- Instant visual swap (synchronous): the two notebooks are persistent
        # (pack_forget/pack, never rebuilt) so their content survives the swap. ---
        if lane == ArtDomain.LANDSCAPE.value:

            self._notebook_buildings.pack_forget()

            self._notebook_landscape.pack(fill=tk.BOTH, expand=True)

            self.notebook = self._notebook_landscape

        else:

            self._notebook_landscape.pack_forget()

            self._notebook_buildings.pack(fill=tk.BOTH, expand=True)

            self.notebook = self._notebook_buildings

        self._authority_var.set(authority_for(lane))
        fg = COLOR_LANE_BUILDING if lane == ArtDomain.BUILDINGS.value else COLOR_LANE_LANDSCAPE
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

        # set_domain rebuilds the correct pills so the spine shows the right steps
        # immediately; the lighter atlas set_domain + spine refresh stay inline too.
        self.pipeline_status.set_domain(lane)

        if lane == ArtDomain.LANDSCAPE.value:

            self.landscape_atlas.set_domain(lane)

        else:

            self.atlas.set_domain(lane)

        self.pipeline_status.refresh()

        self._applied_lane = lane

        # --- Defer the heavy, jank-causing work off the click ---
        # The landscape panel refreshes (disk reads + widget rebuilds) and the
        # lane-persistence disk write run at idle so the swap feels instant. They
        # only matter on the landscape lane; buildings panels need no re-read here.
        if lane == ArtDomain.LANDSCAPE.value:

            self.after_idle(self._refresh_landscape_panels)

        self.after_idle(lambda lane=lane: save_active_lane(lane))

        if log:

            self._log(f"lane → {lane} · tab set swapped")



    def _refresh_landscape_panels(self) -> None:
        """Deferred (after_idle) refresh of the landscape panels — kept off the
        lane-switch click so the swap feels instant. Guarded so a lane flip-back
        before the idle callback runs does not refresh the wrong lane."""

        if self._applied_lane != ArtDomain.LANDSCAPE.value:

            return

        self.landscape_presets.refresh_list()

        self.landscape_grammar.refresh_from_state()

        self.landscape_states.refresh_from_state()



    def _add_scrollable_tab(self, notebook: ttk.Notebook, panel_cls, text: str, **panel_kw):

        tab_root = ttk.Frame(notebook)

        scroll = ScrollableFrame(tab_root, enable_horizontal=False)

        scroll.pack(fill=tk.BOTH, expand=True)

        panel = panel_cls(scroll.interior, **panel_kw)

        panel.pack(fill=tk.BOTH, expand=True)

        notebook.add(tab_root, text=text)

        panel._aps_tab_root = tab_root

        return panel



    def _on_spine_advance(self, verb: str) -> None:
        """The spine's single advance action — runs the lane/step flow verb."""
        handler = self._flow_handlers.get(verb)
        if handler is not None:
            handler()

    def _flow_verb_blocked_reason(self, verb: str) -> str | None:
        """Readiness reason for the spine advance button (Phase 4.5 S2).

        Returned inline by the spine when the verb is not yet runnable — no modal,
        no whisper at the far end of the bar.
        """
        return flow_prerequisite_message(verb, self.state)

    def _flow_verb_ready(self, verb: str) -> bool:
        return self._flow_verb_blocked_reason(verb) is None

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

        # P7 Slice B — the spine disables a not-ready advance button and shows the
        # reason inline; this guard remains as a safety net (e.g. keyboard / future
        # callers) and logs the reason rather than surfacing a far-end red string.

        msg = flow_prerequisite_message(action, self.state)

        if msg:

            self._log(msg)

            return False

        return True



    def _build_authority_strip(self, parent: tk.Misc) -> None:
        row = ttk.Frame(parent)
        row.pack(side=tk.LEFT, fill=tk.X, expand=True)
        self._authority_border = tk.Frame(row, width=4, bg=COLOR_LANE_BUILDING)
        self._authority_border.pack(side=tk.LEFT, fill=tk.Y)
        self._authority_var = tk.StringVar(value=authority_for(self.state.art_domain))
        self._authority_lbl = ttk.Label(
            row,
            textvariable=self._authority_var,
            font=FONT_HINT,
            foreground=COLOR_LANE_BUILDING,
            wraplength=420,
        )
        self._authority_lbl.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(6, 0))

        def _wrap(_event=None) -> None:
            self._authority_lbl.configure(wraplength=wrap_for_widget(parent, minimum=240))

        parent.bind("<Configure>", _wrap)



    def _build_buildings_tabs(self) -> None:

        job_kw = {"start_job": self._start_job}

        nb = self._notebook_buildings

        self.catalog = self._add_scrollable_tab(

            nb, CatalogPanel, "Catalog", state=self.state, on_select=self._on_catalog_select

        )

        bind_aps_tooltip(self.catalog, "tab_catalog")

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

        self.variants = self._add_scrollable_tab(

            nb,
            VariantsPanel,
            "Variants",
            state=self.state,
            on_log=self._log,
            on_go_assembly=self._focus_assembly_tab,
            **job_kw,

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

        # Bottom band, below the work area (Slice C — not an above-the-fold chrome row).
        self._status_log_frame.pack(side=tk.BOTTOM, fill=tk.X, expand=False)

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

            foreground=COLOR_TEXT_BODY,

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

        idx = self.notebook.index(self.notebook.select())

        steps = pipeline_steps_for(self.state.art_domain)

        if 0 <= idx < len(steps):

            self.pipeline_status.set_current(steps[idx][0])

        self.pipeline_status.refresh()



    def _on_pipeline_step(self, key: str) -> None:

        steps = pipeline_steps_for(self.state.art_domain)

        for i, (step_key, _label) in enumerate(steps):

            if step_key == key:

                self.notebook.select(i)

                self.pipeline_status.set_current(key)

                return



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

    def _focus_assembly_tab(self) -> None:
        self.notebook.select(self.assembly._aps_tab_root)
        if hasattr(self.assembly, "refresh_generation_trace"):
            self.assembly.refresh_generation_trace()
        self._log("variants → assembly · edit generation")



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

        # P7 Slice B / S4 — narrate the multi-step work instead of running it silent.

        self.notebook.select(self.variants._aps_tab_root)

        if not self.state.variant_set_data:

            self._log("Bake variants — step 1: created a variant set from this assembly.")

            self.variants.on_new_from_assembly()

        else:

            self._log("Bake variants — step 1: using your existing variant set.")

        self._log("Bake variants — step 2: preview selected variant")
        if self.state.assembly_snapshot_data:
            self.variants.preview_selected_variant(force=True)
        else:
            self._log("Bake variants — step 2: warn — no assembly snapshot; preview skipped")

        self._log("Bake variants — step 3: expanding the variant set into a tile job.")

        self.atlas.on_batch_from_variant_set()

        self.notebook.select(self.atlas._aps_tab_root)

        self._log("Bake variants — step 4: tile job prepared on the Atlas step. Pack it next.")



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

        self._log("Bake states — tile job prepared on the Landscape Atlas step.")



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
            self._log("Pack landscape atlas — tiles registered for the map.")

        else:

            self._log("Pack landscape atlas — bake states or set a PNG folder on the Landscape Atlas step first.")





def run_app() -> None:

    app = ArtPipelineSuiteApp()

    app.mainloop()

