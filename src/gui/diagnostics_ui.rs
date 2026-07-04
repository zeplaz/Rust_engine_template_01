//! Devtools diagnostics window — egui (`F3`).
//!
//! Purpose: minimal **iteration-loop UX** — see FPS, drive sim (pause/step/speed),
//! count entities. **F3** panel includes a **Playtest — strategic / doctrine** section
//! (`CorridorConstructionBook`, theater summaries, doctrine & research reminders).
//!
//! Designer:
//! - `prompts/designer_questions/tools_ui/spec/04_metrics_diagnostics.md`
//! - `prompts/designer_questions/tools_ui/implementation_questions_v1.md` §5–10
//!
//! Pattern mirrors `crate::gui::agent_permissions_ui::permissions_ui_system`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::style::{
    error_text, muted_label, primary_label, section_heading, CmdHeadingStyle, UiPalette,
    widget_scroll_vertical_capped, widget_scroll_vertical_fill,
};
use crate::gui::editor::world_preview::{PreviewPathAuthority, PreviewPresentationDebug};
use crate::gui::gameplay_capture::GameplayRecorder;
use crate::gui::representation_policy::RepresentationResult;
use crate::gui::input_bindings::InputBindings;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::engine::test_harness::ActiveTestScene;
use crate::render::{
    fire_streaming_b_green, ActiveFireChunkSet, AppStage5ReadinessReport, FireChunkRuntime,
    FireStreamingLiveProofState, FireStreamingWitness, WeatherFireFieldDebugOverlay,
    FIRE_SIM_CHUNK_ACTIVE_EPS, FIRE_STREAMING_SLEEP_RADIUS,
};
use crate::systems::atmosphere::AtmosphereDiagnostics;
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::systems::transport::TransportEdgeDirectory;
use crate::systems::weather::{WeatherPrecipVisualSample, WeatherVisualSettings};
use crate::strategic::{
    align_corridor_book_with_transport_directory, CorridorConstructionBook,
    CorridorConstructionPhase, CorridorConstructionRow, LogisticsAiRuntime,
    OperationalTheaterSummary,
};

/// UI visibility + cheap rolling FPS estimate.
#[derive(Resource, Debug, Clone)]
pub struct DiagnosticsUiState {
    pub visible: bool,
    /// Exponential-moving-average FPS; populated each frame from `Time::delta_secs()`.
    pub fps_smoothed: f32,
    /// Last entity count (updated with FPS sampler).
    pub entity_count: usize,
    /// PLAY-01c: collapsing headers default open when true (editor profile).
    pub sections_default_open: bool,
}

impl Default for DiagnosticsUiState {
    fn default() -> Self {
        Self {
            visible: false,
            fps_smoothed: 0.0,
            entity_count: 0,
            // PLAY-01c: collapsed in simulation; editor may set true on enter.
            sections_default_open: false,
        }
    }
}

pub struct DiagnosticsUiPlugin;

impl Plugin for DiagnosticsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsUiState>()
            .add_systems(
                Update,
                (toggle_diagnostics_ui, sample_fps).run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                diagnostics_ui_system.run_if(in_simulation_or_editor),
            );
    }
}

fn toggle_diagnostics_ui(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<DiagnosticsUiState>,
) {
    if keys.just_pressed(bindings.toggle_diagnostics) {
        state.visible = !state.visible;
    }
}

fn sample_fps(
    time: Res<Time>,
    entities: Query<Entity>,
    mut state: ResMut<DiagnosticsUiState>,
    mut entity_count_cache: Local<(f32, usize)>,
    spike_guard: Option<Res<crate::engine::UxFrameSpikeGuard>>,
) {
    let dt = time.delta_secs();
    if dt > f32::EPSILON {
        let inst = 1.0 / dt;
        state.fps_smoothed = if state.fps_smoothed <= 0.0 {
            inst
        } else {
            state.fps_smoothed * 0.9 + inst * 0.1
        };
    }
    // Full-world runs can have millions of entities; counting every frame dominated Update (~220ms).
    entity_count_cache.0 += dt;
    if state.visible
        && !spike_guard.is_some_and(|g| g.suppress_optional_diagnostics)
        && entity_count_cache.0 >= 0.5
    {
        entity_count_cache.0 = 0.0;
        entity_count_cache.1 = entities.iter().len();
    }
    state.entity_count = entity_count_cache.1;
}

/// Bundles optional ecology program diagnostics (VEG-DIAG-COMPOSITE-001).
#[derive(SystemParam)]
pub struct EcologyDiagnosticsPanels<'w, 's> {
    programs: Query<'w, 's, &'static crate::systems::ecology::LandscapeProgramOnChunk>,
    disturbances: Query<'w, 's, &'static crate::systems::ecology::DisturbanceHistory>,
}

/// **VEG-DIAG-EXTRACT-PANEL-001** — read-only vegetation extract sample rows.
#[derive(SystemParam)]
pub struct VegExtractDiagnosticsPanel<'w> {
    frame: Option<Res<'w, crate::render::extraction::VegetationExtractFrame>>,
}

/// Bundles optional spine diagnostics to stay within Bevy system-param limits.
#[derive(SystemParam)]
pub struct DiagnosticsSpinePanels<'w> {
    atmosphere: Option<Res<'w, AtmosphereDiagnostics>>,
    stage5: Option<Res<'w, AppStage5ReadinessReport>>,
    policy: Option<Res<'w, RepresentationResult>>,
    preview_authority: Option<Res<'w, PreviewPathAuthority>>,
    preview_debug: Option<Res<'w, PreviewPresentationDebug>>,
    logistics_diag: Option<Res<'w, crate::economy::logistics::LogisticsDiagnostics>>,
    logistics_rt: Option<Res<'w, crate::economy::logistics::LogisticsThroughputRuntimeWitness>>,
    fire_witness: Option<Res<'w, FireStreamingWitness>>,
    fire_ecology: Option<Res<'w, crate::systems::fire::witness_collectors::FireEcologyWitness>>,
    fire_active: Option<Res<'w, ActiveFireChunkSet>>,
    fire_runtime: Option<Res<'w, FireChunkRuntime>>,
    fire_proof: Option<Res<'w, FireStreamingLiveProofState>>,
    test_scene: Option<Res<'w, ActiveTestScene>>,
    theater: Option<Res<'w, OperationalTheaterSummary>>,
    logistics_ai: Option<Res<'w, LogisticsAiRuntime>>,
}

/// Renders the panel; consumers add tabs by extending this system or chaining own systems
/// in `EguiPrimaryContextPass` after this one.
pub fn diagnostics_ui_system(
    mut contexts: EguiContexts,
    state: Res<DiagnosticsUiState>,
    bindings: Res<InputBindings>,
    mut ctrl: ResMut<SimControlState>,
    tick: Res<SimTick>,
    mut wx: ResMut<WeatherVisualSettings>,
    wx_sample: Res<WeatherPrecipVisualSample>,
    mut gpu_field_debug: ResMut<WeatherFireFieldDebugOverlay>,
    cap: Res<GameplayRecorder>,
    directory: Res<TransportEdgeDirectory>,
    mut construction_book: ResMut<CorridorConstructionBook>,
    palette: Res<UiPalette>,
    spine: DiagnosticsSpinePanels,
    ecology: EcologyDiagnosticsPanels,
    veg_extract: VegExtractDiagnosticsPanel,
) -> Result {
    if !state.visible {
        return Ok(());
    }

    let entity_count = state.entity_count;
    let ctx = contexts.ctx_mut()?;

    crate::gui::std_floating(egui::Window::new(format!(
        "Diagnostics ({})",
        InputBindings::format_key(bindings.toggle_diagnostics)
    )))
    .default_size(egui::vec2(420.0, 520.0))
        .collapsible(true)
        .show(ctx, |ui| {
            widget_scroll_vertical_fill("diagnostics_body_scroll", ui.available_height()).show(ui, |ui| {
            primary_label(ui, &palette, format!("FPS (EMA): {:.1}", state.fps_smoothed));
            primary_label(ui, &palette, format!("Sim tick:  {}", tick.0));
            primary_label(ui, &palette, format!("Entities:  {entity_count}"));
            if let Some(ts) = spine.test_scene.as_ref() {
                primary_label(ui, &palette, format!("CLI test scene: {:?}", ts.0));
            }

            if let (Some(ld), Some(rt)) = (
                spine.logistics_diag.as_deref(),
                spine.logistics_rt.as_deref(),
            ) {
                ui.separator();
                section_heading(ui, &palette, CmdHeadingStyle::Gt, "Logistics throughput (LOG-D-05)");
                primary_label(
                    ui,
                    &palette,
                    format!(
                        "Routes open: {} · blocked: {} · proofs: {} · saturation max: {:.2}",
                        ld.routes_open,
                        ld.routes_blocked,
                        ld.proofs.len(),
                        rt.edge_saturation_max
                    ),
                );
            }

            ui.separator();
            section_heading(ui, &palette, CmdHeadingStyle::Gt, "Gameplay capture");
            muted_label(
                ui,
                &palette,
                format!(
                    "{} start · {} stop (Options to rebind)",
                    InputBindings::format_key(bindings.start_gameplay_recording),
                    InputBindings::format_key(bindings.stop_gameplay_recording)
                ),
            );
            if cap.active {
                if let Some(dir) = cap.session_dir.as_ref() {
                    error_text(
                        ui,
                        &palette,
                        format!("● REC {} frames → {}", cap.frames_recorded, dir.display()),
                    );
                } else {
                    error_text(ui, &palette, "● REC");
                }
            } else if let Some(s) = cap.last_summary() {
                muted_label(ui, &palette, s);
            } else {
                muted_label(
                    ui,
                    &palette,
                    format!(
                        "PNG folder + clip.gif under {}",
                        GameplayRecorder::default_captures_root().display()
                    ),
                );
            }

            ui.separator();
            section_heading(ui, &palette, CmdHeadingStyle::Gt, "Sim control");
            ui.horizontal(|ui| {
                if ui.button(if ctrl.paused { "Play" } else { "Pause" }).clicked() {
                    ctrl.paused = !ctrl.paused;
                }
                if ui.button("Step").clicked() {
                    ctrl.steps_remaining = ctrl.steps_remaining.saturating_add(1);
                    ctrl.paused = true;
                }
            });
            ui.add(egui::Slider::new(&mut ctrl.speed, 0.0..=8.0).text("speed"));

            if let Some(d) = spine.atmosphere.as_ref() {
                ui.separator();
                egui::CollapsingHeader::new("Atmosphere + visual extract (CPU)")
                    .default_open(state.sections_default_open)
                    .show(ui, |ui| {
                        section_heading(ui, &palette, CmdHeadingStyle::Gt, "Atmosphere (CPU field)");
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "fill #{} · advect #{} · emitters #{} · particles #{} · coupling #{} · visual #{} · render_prep #{}",
                                d.field_fill_runs,
                                d.advect_runs,
                                d.emitter_sync_runs,
                                d.particle_controller_runs,
                                d.coupling_runs,
                                d.visual_extract_runs,
                                d.render_prep_runs
                            ),
                        );
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "mean smoke {:.3} · mean vis {:.3} · max toxic {:.3} · path vis {:.3} · path smoke {:.3} · extract emitters {} · smoke cells {}",
                                d.last_mean_smoke,
                                d.last_mean_visibility,
                                d.last_max_toxicity,
                                d.sample_path_visibility,
                                d.sample_mean_smoke,
                                d.last_emitter_extract_count,
                                d.last_smoke_extract_count
                            ),
                        );
                        let partial = &d.partial_write_metrics;
                        let reconcile_age = partial
                            .last_partial_stamp
                            .tick
                            .saturating_sub(partial.last_full_reconcile_stamp.tick);
                        let full_mb = partial.full_field_texture_bytes as f64 / (1024.0 * 1024.0);
                        let gpu_mb = partial.gpu_texture_upload_bytes as f64 / (1024.0 * 1024.0);
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "ATM: dirty={} partial_uploads={} gpu_mb={:.3} full_mb={:.3} reconcile_age={} partial_disp={} full_fallback={}",
                                partial.dirty_region_count,
                                partial.gpu_texture_upload_count,
                                gpu_mb,
                                full_mb,
                                reconcile_age,
                                partial.partial_compute_dispatch_count,
                                partial.full_field_fallback_active
                            ),
                        );
                        let drift = [
                            d.field_fill_runs,
                            d.advect_runs,
                            d.visual_extract_runs,
                            d.last_emitter_extract_count as u64,
                            d.last_smoke_extract_count as u64,
                        ]
                        .iter()
                        .fold(0x9E37_79B9_7F4A_7C15u64, |a, &b| {
                            a.wrapping_mul(31).wrapping_add(b)
                        })
                        ^ (d.last_mean_smoke.to_bits() as u64)
                        ^ (d.last_mean_visibility.to_bits() as u64).rotate_left(17);
                        muted_label(
                            ui,
                            &palette,
                            format!("visual extract drift {:016x} (cheap fingerprint)", drift),
                        );
                        if d.mean_smoke_over_budget || d.max_toxicity_over_budget {
                            error_text(
                                ui,
                                &palette,
                                format!(
                                    "Atmosphere perf: smoke_budget={} toxic_budget={}",
                                    d.mean_smoke_over_budget, d.max_toxicity_over_budget
                                ),
                            );
                        }
                    });
            }

            if let Some(eco) = spine.fire_ecology.as_deref() {
                ui.separator();
                egui::CollapsingHeader::new("Fire ecology preview (DES-ECOLOGY-PREVIEW-V2)")
                    .default_open(false)
                    .show(ui, |ui| {
                        section_heading(ui, &palette, CmdHeadingStyle::Gt, "Sim heat / fuel");
                        primary_label(
                            ui,
                            &palette,
                            format!(
                                "fuel={} band={} old_growth={:.2} heat={:.3} max={:.3}",
                                eco.mean_fuel,
                                eco.fuel_band_label(),
                                eco.mean_old_growth,
                                eco.mean_heat,
                                eco.max_heat,
                            ),
                        );
                        primary_label(
                            ui,
                            &palette,
                            format!(
                                "ignition_gate={} heat_stable={} frames={}",
                                if eco.ignition_gate_open() { "open" } else { "closed" },
                                eco.heat_mostly_stable(),
                                eco.frames_sampled,
                            ),
                        );
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "spread: depleted={} neighbor={}",
                                eco.fuel_depleted_cells, eco.neighbor_spread_cells,
                            ),
                        );
                    });
            }

            if ecology.programs.iter().next().is_some() || ecology.disturbances.iter().next().is_some() {
                ui.separator();
                egui::CollapsingHeader::new("Landscape grammar (composite)")
                    .default_open(state.sections_default_open)
                    .show(ui, |ui| {
                        section_heading(ui, &palette, CmdHeadingStyle::Gt, "Topology programs");
                        let mut preset_counts: std::collections::BTreeMap<String, u32> =
                            std::collections::BTreeMap::new();
                        let mut kind_union: std::collections::BTreeSet<String> =
                            std::collections::BTreeSet::new();
                        let mut depth_max = 0usize;
                        for program in ecology.programs.iter() {
                            *preset_counts
                                .entry(program.preset_id.clone())
                                .or_insert(0) += 1;
                            depth_max = depth_max.max(program.evaluation.nested_depth_max);
                            for kind in &program.evaluation.topology_kinds {
                                kind_union.insert(kind.clone());
                            }
                        }
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "chunks={} presets={} kinds={} nested_depth_max={}",
                                ecology.programs.iter().len(),
                                preset_counts.len(),
                                kind_union.len(),
                                depth_max,
                            ),
                        );
                        for (preset, count) in preset_counts.iter().take(8) {
                            muted_label(ui, &palette, format!("• {preset} ×{count}"));
                        }
                        let mut kinds: Vec<_> = kind_union.into_iter().collect();
                        kinds.sort();
                        if !kinds.is_empty() {
                            section_heading(ui, &palette, CmdHeadingStyle::Gt, "Topology kinds");
                            muted_label(ui, &palette, kinds.join(" · "));
                        }
                        let fire_events: u32 = ecology
                            .disturbances
                            .iter()
                            .flat_map(|h| h.events.iter())
                            .filter(|e| {
                                matches!(
                                    e.kind,
                                    crate::systems::ecology::DisturbanceKind::Fire
                                )
                            })
                            .count() as u32;
                        let build_events: u32 = ecology
                            .disturbances
                            .iter()
                            .flat_map(|h| h.events.iter())
                            .filter(|e| {
                                matches!(
                                    e.kind,
                                    crate::systems::ecology::DisturbanceKind::ConstructionClear
                                )
                            })
                            .count() as u32;
                        section_heading(ui, &palette, CmdHeadingStyle::Gt, "Disturbance timeline");
                        muted_label(
                            ui,
                            &palette,
                            format!("fire={fire_events} construction_clear={build_events}"),
                        );
                    });
            }

            if let Some(frame) = veg_extract.frame.as_ref() {
                ui.separator();
                egui::CollapsingHeader::new("Vegetation extract (VEG-DIAG-EXTRACT-001)")
                    .default_open(false)
                    .show(ui, |ui| {
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "revision={} rows={} tick={}",
                                frame.revision,
                                frame.rows.len(),
                                frame.stamp.tick
                            ),
                        );
                        for row in frame.rows.iter().take(8) {
                            muted_label(
                                ui,
                                &palette,
                                format!(
                                    "({},{}) vk={} g={} burn={} f={:02}",
                                    row.coord.x,
                                    row.coord.y,
                                    row.variant_key,
                                    row.extract_glyph,
                                    row.burn_active,
                                    row.frame_index,
                                ),
                            );
                        }
                        if frame.rows.len() > 8 {
                            muted_label(
                                ui,
                                &palette,
                                format!("… {} more rows", frame.rows.len() - 8),
                            );
                        }
                    });
            }

            if let Some(report) = spine.stage5.as_ref() {
                ui.separator();
                egui::CollapsingHeader::new("Stage 5 readiness")
                    .default_open(report.violations.is_empty())
                    .show(ui, |ui| {
                        let stamp_tick = spine
                            .policy
                            .as_deref()
                            .map(|policy| policy.stamp.tick)
                            .unwrap_or(0);
                        muted_label(
                            ui,
                            &palette,
                            format!(
                                "stamp={} VT-4={} VT-5={} phase_d={} phase_f={} fire_extract={} gpu_field={} preview_gpu={} overlay_shared={} particle_lod={} phase_f_lod={} domains={} producers={} dup_extract={}",
                                stamp_tick,
                                report.vt4_ok,
                                report.vt5_ok,
                                report.phase_d_ok,
                                report.phase_f_ok,
                                report.single_fire_extract,
                                report.gpu_field_authoritative,
                                report.preview_render_target_active,
                                report.overlay_from_shared_buffers_only,
                                report.particle_lod_scales,
                                report.phase_f_lod_proof_ok,
                                report.projection_domains,
                                report.registered_producers,
                                report.duplicate_visual_scan_count,
                            ),
                        );
                        if let Some(authority) = spine.preview_authority.as_deref() {
                            muted_label(
                                ui,
                                &palette,
                                format!(
                                    "preview authority={:?} gpu_requested={} cpu_fallback={} gpu_present={}",
                                    authority.authoritative_surface,
                                    authority.gpu_render_target_requested,
                                    authority.cpu_raster_fallback_active,
                                    authority.gpu_present_count,
                                ),
                            );
                        }
                        if let Some(debug) = spine.preview_debug.as_deref() {
                            muted_label(
                                ui,
                                &palette,
                                format!(
                                    "preview debug surface={:?} front_asset_bits={}",
                                    debug.authoritative_surface,
                                    debug.last_front_asset_id_bits,
                                ),
                            );
                        }
                        for violation in &report.violations {
                            error_text(ui, &palette, violation);
                        }
                    });
            }

            if spine.fire_witness.is_some() {
                ui.separator();
                let witness = spine.fire_witness.as_deref();
                let active = spine.fire_active.as_deref();
                let runtime = spine.fire_runtime.as_deref();
                let green = match (witness, active) {
                    (Some(w), Some(a)) => fire_streaming_b_green(w, a),
                    _ => false,
                };
                egui::CollapsingHeader::new("Fire Phase 7 — chunk streaming (F7-B)")
                    .default_open(state.sections_default_open)
                    .show(ui, |ui| {
                        muted_label(
                            ui,
                            &palette,
                            format!("F7B gate=FIRE7-F7-B-001 green={green}"),
                        );
                        if let Some(w) = witness {
                            muted_label(
                                ui,
                                &palette,
                                format!(
                                    "F7B focus_chunk=({}, {}) sleep_r={}",
                                    w.focus_chunk.x,
                                    w.focus_chunk.y,
                                    FIRE_STREAMING_SLEEP_RADIUS,
                                ),
                            );
                            let active_count = active.map(|a| a.chunks.len()).unwrap_or(0);
                            muted_label(
                                ui,
                                &palette,
                                format!(
                                    "F7B sleep={} wake={} active={}",
                                    w.sleep_transitions, w.wake_transitions, active_count,
                                ),
                            );
                        }
                        let runtime_writer = spine
                            .fire_proof
                            .as_deref()
                            .map(FireStreamingLiveProofState::written)
                            .unwrap_or(false);
                        muted_label(
                            ui,
                            &palette,
                            format!("F7B runtime_writer={runtime_writer}"),
                        );
                        if let Some(rt) = runtime {
                            let vis = rt.chunks.values().filter(|c| c.visual_active).count();
                            let sim = rt
                                .chunks
                                .values()
                                .filter(|c| {
                                    c.active || c.max_heat > FIRE_SIM_CHUNK_ACTIVE_EPS
                                })
                                .count();
                            let tot = rt.chunks.len();
                            muted_label(
                                ui,
                                &palette,
                                format!(
                                    "F7B visual_active={vis} sim_active={sim} total_chunks={tot}"
                                ),
                            );
                        }
                        ui.separator();
                        section_heading(
                            ui,
                            &palette,
                            CmdHeadingStyle::Gt,
                            "Map gizmo legend (focus / tile debug)",
                        );
                        muted_label(
                            ui,
                            &palette,
                            "Focus chunk — gold #F2D926 — camera-derived focus tile",
                        );
                        muted_label(
                            ui,
                            &palette,
                            "Fire active — red #FF261E — chunk in fire-active union",
                        );
                        muted_label(
                            ui,
                            &palette,
                            "Terrain resident — green #33BF40 — chunk entity present",
                        );
                        muted_label(
                            ui,
                            &palette,
                            "Empty — dark gray #1E1E24 — no terrain / no fire",
                        );
                    });
            }

            ui.separator();
            section_heading(ui, &palette, CmdHeadingStyle::Gt, "GPU weather / fire field (compute)");
            ui.checkbox(&mut gpu_field_debug.show, "Debug sprite (128² Rgba32Float field, bottom-left)");
            muted_label(
                ui,
                &palette,
                "CPU uploads mean rain/snow/fog + **fire from visual extract** (emitters); smoke extract biases heat; ecology means in extra. WGSL relaxes ping-pong. Visual-only.",
            );

            ui.separator();
            section_heading(ui, &palette, CmdHeadingStyle::Gt, "Weather FX (preview)");
            ui.checkbox(&mut wx.enabled, "Enable weather VFX");
            ui.add_enabled_ui(wx.enabled, |ui| {
                ui.checkbox(&mut wx.overlay, "Screen overlay (rain/fog tint)");
                ui.checkbox(&mut wx.particles, "Precip particles (streaks / flakes)");
                ui.checkbox(
                    &mut wx.background_aesthetic,
                    "Background precip (zoomed-out digital AE)",
                );
                ui.add(
                    egui::Slider::new(&mut wx.max_precip_particles, 32usize..=512usize)
                        .text("Particle pool"),
                );
            });
            if wx_sample.chunk_count == 0 {
                muted_label(
                    ui,
                    &palette,
                    "No ChunkWeather yet — open map with materialized chunks or run a scene that spawns chunks.",
                );
            } else {
                muted_label(
                    ui,
                    &palette,
                    format!(
                        "Mean precip sample ({} chunks): rain {:.2}, snow {:.2}, fog {:.2}",
                        wx_sample.chunk_count, wx_sample.rain, wx_sample.snow, wx_sample.fog
                    ),
                );
            }

            ui.separator();
            egui::CollapsingHeader::new("Playtest — strategic / doctrine")
                .default_open(false)
                .show(ui, |ui| {
                    muted_label(
                        ui,
                        &palette,
                        "Bake/load transport (editor G4) auto-aligns the construction book: new edges → Completed; stale rows dropped; existing phases kept.",
                    );
                    primary_label(
                        ui,
                        &palette,
                        format!(
                            "Transport edges: {} · book rows: {}",
                            directory.by_edge.len(),
                            construction_book.rows.len()
                        ),
                    );
                    if let (Some(th), Some(la)) =
                        (spine.theater.as_deref(), spine.logistics_ai.as_deref())
                    {
                        primary_label(
                            ui,
                            &palette,
                            format!(
                                "Theater μ threat[0]: {:.2} · μ logistics[0]: {:.2} · active faction slots: {}",
                                th.mean_threat_by_slot[0],
                                th.mean_logistics_strength_by_slot[0],
                                th.active_faction_slots
                            ),
                        );
                        primary_label(
                            ui,
                            &palette,
                            format!(
                                "Logistics AI: congest {:.2} · edge dmg {:.2} · stockpile fill {:.2} · industry proxy {:.2} · manifest domains {:.2}",
                                la.congestion_proxy,
                                la.mean_edge_damage,
                                la.stockpile_fill_ratio,
                                la.industrial_output_proxy,
                                la.production_domain_proxy
                            ),
                        );
                    } else {
                        muted_label(
                            ui,
                            &palette,
                            "Theater / logistics AI resources not loaded (StrategicSimulationPlugin missing in this app).",
                        );
                    }

                    egui::CollapsingHeader::new("Doctrine checklist (traceability)")
                        .default_open(false)
                        .show(ui, |ui| {
                            muted_label(
                                ui,
                                &palette,
                                "Maps modern systems warfare targets → sim layers. Full: prompts/guides/doctrine_simulation_alignment_runbook_v1.md",
                            );
                            muted_label(
                                ui,
                                &palette,
                                "• Intel / recon fields ↔ drone & sensor coverage (recon_confidence + weather visibility).",
                            );
                            muted_label(
                                ui,
                                &palette,
                                "• EW ↔ routing_congestion / ew_denial overlay scalars (transport-derived + toggles).",
                            );
                            muted_label(
                                ui,
                                &palette,
                                "• Logistics attacks ↔ throughput collapse on LogisticsGraph + congestion proxy.",
                            );
                            muted_label(
                                ui,
                                &palette,
                                "• Infrastructure strikes ↔ disruption on edges + infra graph integrity (resilience runbook).",
                            );
                        });

                    egui::CollapsingHeader::new("Research program (design authority)")
                        .default_open(false)
                        .show(ui, |ui| {
                            muted_label(
                                ui,
                                &palette,
                                "Capability = institutions + industrial maturity + doctrine pressure — not an isolated tech-tree button.",
                            );
                            muted_label(
                                ui,
                                &palette,
                                "See: prompts/guides/research_capability_ecosystem_runbook_v1.md · orchestrator: infrastructure_and_research_orchestrator_v1.md",
                            );
                        });

                    ui.horizontal(|ui| {
                        if ui.button("Re-align book ↔ directory").on_hover_text("Drop orphan book rows; add Completed for new edge ids; keep existing phases.").clicked() {
                            align_corridor_book_with_transport_directory(&directory, construction_book.as_mut());
                        }
                        if ui.button("All edges → Completed").clicked() {
                            for eid in directory.by_edge.keys() {
                                construction_book
                                    .rows
                                    .insert(*eid, CorridorConstructionRow::completed(*eid));
                            }
                        }
                    });

                    let mut keys: Vec<_> = directory.by_edge.keys().copied().collect();
                    keys.sort_by_key(|k| k.0);
                    keys.truncate(24);
                    if keys.is_empty() {
                        muted_label(
                            ui,
                            &palette,
                            "No transport edges — bake roads in map editor or load dev_transport_network.ron (or .json fixture).",
                        );
                    } else {
                        widget_scroll_vertical_capped("diagnostics_transport_edges_scroll", 220.0)
                            .show(ui, |ui| {
                            for eid in keys {
                                let row = construction_book
                                    .rows
                                    .entry(eid)
                                    .or_insert_with(|| CorridorConstructionRow::completed(eid));
                                ui.group(|ui| {
                                    primary_label(ui, &palette, format!("Edge {}", eid.0));
                                    ui.horizontal(|ui| {
                                        ui.radio_value(&mut row.phase, CorridorConstructionPhase::Planned, "Planned");
                                        ui.radio_value(&mut row.phase, CorridorConstructionPhase::InProgress, "In progress");
                                        ui.radio_value(&mut row.phase, CorridorConstructionPhase::Completed, "Completed");
                                    });
                                    if row.phase == CorridorConstructionPhase::InProgress {
                                        ui.add(egui::Slider::new(&mut row.progress, 0.0..=1.0).text("Traffic progress"));
                                    }
                                });
                            }
                        });
                    }
                });

            // TODO: tabs — chunk streamer, production manifest summary, faction roster.
            });
        });

    Ok(())
}
