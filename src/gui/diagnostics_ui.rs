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

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::style::{error_text, UiPalette};
use crate::gui::gameplay_capture::GameplayRecorder;
use crate::gui::input_bindings::InputBindings;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::engine::test_harness::ActiveTestScene;
use crate::render::WeatherFireFieldDebugOverlay;
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::systems::transport::TransportEdgeDirectory;
use crate::systems::weather::{WeatherPrecipVisualSample, WeatherVisualSettings};
use crate::strategic::{
    align_corridor_book_with_transport_directory, CorridorConstructionBook,
    CorridorConstructionPhase, CorridorConstructionStatus, LogisticsAiRuntime,
    OperationalTheaterSummary,
};

/// UI visibility + cheap rolling FPS estimate.
#[derive(Resource, Debug, Clone)]
pub struct DiagnosticsUiState {
    pub visible: bool,
    /// Exponential-moving-average FPS; populated each frame from `Time::delta_secs()`.
    pub fps_smoothed: f32,
}

impl Default for DiagnosticsUiState {
    fn default() -> Self {
        Self {
            visible: false,
            fps_smoothed: 0.0,
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

fn sample_fps(time: Res<Time>, mut state: ResMut<DiagnosticsUiState>) {
    let dt = time.delta_secs();
    if dt > f32::EPSILON {
        let inst = 1.0 / dt;
        state.fps_smoothed = if state.fps_smoothed <= 0.0 {
            inst
        } else {
            state.fps_smoothed * 0.9 + inst * 0.1
        };
    }
}

/// Renders the panel; consumers add tabs by extending this system or chaining own systems
/// in `EguiPrimaryContextPass` after this one.
pub fn diagnostics_ui_system(
    mut contexts: EguiContexts,
    state: Res<DiagnosticsUiState>,
    bindings: Res<InputBindings>,
    mut ctrl: ResMut<SimControlState>,
    tick: Res<SimTick>,
    entities: Query<Entity>,
    mut wx: ResMut<WeatherVisualSettings>,
    wx_sample: Res<WeatherPrecipVisualSample>,
    mut gpu_field_debug: ResMut<WeatherFireFieldDebugOverlay>,
    test_scene: Option<Res<ActiveTestScene>>,
    cap: Res<GameplayRecorder>,
    directory: Res<TransportEdgeDirectory>,
    mut construction_book: ResMut<CorridorConstructionBook>,
    theater: Option<Res<OperationalTheaterSummary>>,
    logistics_ai: Option<Res<LogisticsAiRuntime>>,
    palette: Res<UiPalette>,
) -> Result {
    if !state.visible {
        return Ok(());
    }

    let entity_count = entities.iter().count();
    let ctx = contexts.ctx_mut()?;

    egui::Window::new(format!(
        "Diagnostics ({})",
        InputBindings::format_key(bindings.toggle_diagnostics)
    ))
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(format!("FPS (EMA): {:.1}", state.fps_smoothed));
            ui.label(format!("Sim tick:  {}", tick.0));
            ui.label(format!("Entities:  {entity_count}"));
            if let Some(ts) = test_scene.as_ref() {
                ui.label(format!("CLI test scene: {:?}", ts.0));
            }

            ui.separator();
            ui.heading("Gameplay capture");
            ui.small(format!(
                "{} start · {} stop (Options to rebind)",
                InputBindings::format_key(bindings.start_gameplay_recording),
                InputBindings::format_key(bindings.stop_gameplay_recording)
            ));
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
                ui.small(s);
            } else {
                ui.small(format!(
                    "PNG folder + clip.gif under {}",
                    GameplayRecorder::default_captures_root().display()
                ));
            }

            ui.separator();
            ui.heading("Sim control");
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

            ui.separator();
            ui.heading("GPU weather / fire field (compute)");
            ui.checkbox(&mut gpu_field_debug.show, "Debug sprite (128² Rgba32Float field, bottom-left)");
            ui.small("CPU uploads mean rain/snow/fog + mean chunk surface fire heat; WGSL relaxes a ping-pong texture. Visual-only.");

            ui.separator();
            ui.heading("Weather FX (preview)");
            ui.checkbox(&mut wx.enabled, "Enable weather VFX");
            ui.add_enabled_ui(wx.enabled, |ui| {
                ui.checkbox(&mut wx.overlay, "Screen overlay (rain/fog tint)");
                ui.checkbox(&mut wx.particles, "Precip particles (streaks / flakes)");
                ui.add(
                    egui::Slider::new(&mut wx.max_precip_particles, 32usize..=512usize)
                        .text("Particle pool"),
                );
            });
            if wx_sample.chunk_count == 0 {
                ui.small("No ChunkWeather yet — open map with materialized chunks or run a scene that spawns chunks.");
            } else {
                ui.small(format!(
                    "Mean precip sample ({} chunks): rain {:.2}, snow {:.2}, fog {:.2}",
                    wx_sample.chunk_count, wx_sample.rain, wx_sample.snow, wx_sample.fog
                ));
            }

            ui.separator();
            egui::CollapsingHeader::new("Playtest — strategic / doctrine")
                .default_open(false)
                .show(ui, |ui| {
                    ui.small("Bake/load transport (editor G4) auto-aligns the construction book: new edges → Completed; stale rows dropped; existing phases kept.");
                    ui.label(format!(
                        "Transport edges: {} · book rows: {}",
                        directory.by_edge.len(),
                        construction_book.by_edge.len()
                    ));
                    if let (Some(th), Some(la)) = (theater.as_deref(), logistics_ai.as_deref()) {
                        ui.label(format!(
                            "Theater μ threat[0]: {:.2} · μ logistics[0]: {:.2} · active faction slots: {}",
                            th.mean_threat_by_slot[0],
                            th.mean_logistics_strength_by_slot[0],
                            th.active_faction_slots
                        ));
                        ui.label(format!(
                            "Logistics AI: congest {:.2} · edge dmg {:.2} · stockpile fill {:.2} · industry proxy {:.2} · manifest domains {:.2}",
                            la.congestion_proxy,
                            la.mean_edge_damage,
                            la.stockpile_fill_ratio,
                            la.industrial_output_proxy,
                            la.production_domain_proxy
                        ));
                    } else {
                        ui.small("Theater / logistics AI resources not loaded (StrategicSimulationPlugin missing in this app).");
                    }

                    egui::CollapsingHeader::new("Doctrine checklist (traceability)")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.small("Maps modern systems warfare targets → sim layers. Full: prompts/guides/doctrine_simulation_alignment_runbook_v1.md");
                            ui.small("• Intel / recon fields ↔ drone & sensor coverage (recon_confidence + weather visibility).");
                            ui.small("• EW ↔ routing_congestion / ew_denial overlay scalars (transport-derived + toggles).");
                            ui.small("• Logistics attacks ↔ throughput collapse on LogisticsGraph + congestion proxy.");
                            ui.small("• Infrastructure strikes ↔ disruption on edges + infra graph integrity (resilience runbook).");
                        });

                    egui::CollapsingHeader::new("Research program (design authority)")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.small("Capability = institutions + industrial maturity + doctrine pressure — not an isolated tech-tree button.");
                            ui.small("See: prompts/guides/research_capability_ecosystem_runbook_v1.md · orchestrator: infrastructure_and_research_orchestrator_v1.md");
                        });

                    ui.horizontal(|ui| {
                        if ui.button("Re-align book ↔ directory").on_hover_text("Drop orphan book rows; add Completed for new edge ids; keep existing phases.").clicked() {
                            align_corridor_book_with_transport_directory(&directory, construction_book.as_mut());
                        }
                        if ui.button("All edges → Completed").clicked() {
                            for eid in directory.by_edge.keys() {
                                construction_book.by_edge.insert(*eid, CorridorConstructionStatus::default());
                            }
                        }
                    });

                    let mut keys: Vec<_> = directory.by_edge.keys().copied().collect();
                    keys.sort_by_key(|k| k.0);
                    keys.truncate(24);
                    if keys.is_empty() {
                        ui.small("No transport edges — bake roads in map editor or load dev_transport_network.ron (or .json fixture).");
                    } else {
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                            for eid in keys {
                                let st = construction_book
                                    .by_edge
                                    .entry(eid)
                                    .or_insert(CorridorConstructionStatus::default());
                                ui.group(|ui| {
                                    ui.label(format!("Edge {}", eid.0));
                                    ui.horizontal(|ui| {
                                        ui.radio_value(&mut st.phase, CorridorConstructionPhase::Planned, "Planned");
                                        ui.radio_value(&mut st.phase, CorridorConstructionPhase::InProgress, "In progress");
                                        ui.radio_value(&mut st.phase, CorridorConstructionPhase::Completed, "Completed");
                                    });
                                    if st.phase == CorridorConstructionPhase::InProgress {
                                        ui.add(egui::Slider::new(&mut st.progress, 0.0..=1.0).text("Traffic progress"));
                                    }
                                });
                            }
                        });
                    }
                });

            // TODO: tabs — chunk streamer, production manifest summary, faction roster.
        });

    Ok(())
}
