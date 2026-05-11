//! **Pressure composition tooling** — egui composer + Bevy UI mini-strip (not quest scripting).
//!
//! | Designer surface | Implementation |
//! |---|---|
//! | World inspector | egui: [`WorldFields`] + [`PressureField`] sliders |
//! | Faction pressure editor | egui: per-entity [`Faction`] scalars + regional list |
//! | Agent trait inspector | egui: [`Agent`] + [`ScriptInfluence`] |
//! | Mission composer | egui: [`ActiveMissions`] + pressure profile |
//! | Simulation graph view | egui: faction cohesion / heuristic columns |
//! | Event log | egui: [`StrategicEmergenceLog`] |
//! | Bevy HUD strip | bottom-right [`Text`] mirror |

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::input_bindings::InputBindings;
use crate::gui::style::UiPalette;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::strategic::{
    mission_success_readout_note, ActiveMissions, Agent, AgentFactionLink, DecisionPipelineSink,
    Faction, FractureOverlaySettings, FractureProbabilityOverlay, GpuBridgeState, HybridAgentTraits,
    Mission as MissionT, MissionId, PressureField, PressureProfile, RegionalStatsOverlay, ScriptInfluence,
    StrategicEmergenceLog, WorldFields,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressureComposerTab {
    World,
    Faction,
    Agent,
    Mission,
    Graph,
    Log,
}

#[derive(Resource, Debug, Clone)]
pub struct PressureComposerState {
    pub visible: bool,
    pub tab: PressureComposerTab,
    /// Toggle Bevy UI mirror strip (see [`PressureBevyOverlayRoot`]).
    pub show_bevy_strip: bool,
    pub draft_duration: u64,
    pub draft_pressure: PressureProfile,
    pub draft_priority: f32,
    pub draft_bias: HybridAgentTraits,
    pub draft_success_label: String,
    pub draft_participants: Vec<Entity>,
    pub entity_picker_cursor: usize,
}

impl Default for PressureComposerState {
    fn default() -> Self {
        Self {
            visible: false,
            tab: PressureComposerTab::World,
            show_bevy_strip: true,
            draft_duration: 2_000,
            draft_pressure: PressureProfile {
                paranoia: 0.15,
                aggression: 0.1,
                instability: 0.12,
                cohesion_drift: 0.08,
            },
            draft_priority: 0.65,
            draft_bias: HybridAgentTraits {
                paranoia: 0.05,
                aggression: 0.04,
                cruelty: 0.03,
                ..Default::default()
            },
            draft_success_label: String::from("Region cohesion < 0.3"),
            draft_participants: Vec::new(),
            entity_picker_cursor: 0,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct PressureComposerScratch {
    pub next_mission_id: u64,
}

#[derive(Component)]
pub struct PressureBevyOverlayRoot;

#[derive(Component)]
struct PressureBevyOverlayText;

pub struct PressureComposerPlugin;

impl Plugin for PressureComposerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PressureComposerState>()
            .init_resource::<PressureComposerScratch>()
            .add_systems(
                Update,
                toggle_pressure_composer.run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                pressure_composer_egui_system.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                sync_pressure_bevy_overlay_visibility.run_if(in_simulation_or_editor),
            );
    }
}

pub struct StrategicToolingPlugin;

impl Plugin for StrategicToolingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PressureComposerPlugin, PressureOverlayBevyPlugin));
    }
}

pub struct PressureOverlayBevyPlugin;

impl Plugin for PressureOverlayBevyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::engine::BaseState::Simulation),
            spawn_pressure_bevy_overlay,
        )
        .add_systems(
            OnExit(crate::engine::BaseState::Simulation),
            despawn_pressure_bevy_overlay,
        )
        .add_systems(
            Update,
            sync_pressure_bevy_overlay_text.run_if(in_simulation_or_editor),
        );
    }
}

fn toggle_pressure_composer(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<PressureComposerState>,
) {
    if keys.just_pressed(bindings.toggle_pressure_composer) {
        state.visible = !state.visible;
    }
}

fn sync_pressure_bevy_overlay_visibility(
    state: Res<PressureComposerState>,
    mut q: Query<&mut Visibility, With<PressureBevyOverlayRoot>>,
) {
    let vis = if state.show_bevy_strip {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut v in &mut q {
        *v = vis;
    }
}

fn spawn_pressure_bevy_overlay(mut commands: Commands, existing: Query<Entity, With<PressureBevyOverlayRoot>>) {
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            PressureBevyOverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                max_width: Val::Px(380.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
            Visibility::Visible,
            ZIndex(900),
        ))
        .with_children(|p| {
            p.spawn((
                PressureBevyOverlayText,
                Text::new("Pressure — …"),
                TextColor(Color::WHITE),
            ));
        });
}

fn despawn_pressure_bevy_overlay(mut commands: Commands, q: Query<Entity, With<PressureBevyOverlayRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn sync_pressure_bevy_overlay_text(
    world_f: Res<WorldFields>,
    pressure: Res<PressureField>,
    fracture: Res<FractureProbabilityOverlay>,
    gpu: Res<GpuBridgeState>,
    log: Res<StrategicEmergenceLog>,
    sink: Res<DecisionPipelineSink>,
    mut text_q: Query<&mut Text, With<PressureBevyOverlayText>>,
) {
    let tail = log.tail_joined(3);
    let lane = format!("{:?}", gpu.lane);
    let block = format!(
        "PressureField  p {:.2} a {:.2} i {:.2} cd {:.2}\nWorld  instab {:.2} econ {:.2}\nFracture lens  mean {:.2} max {:.2}\nGPU bridge  {}\nPipeline  mean {:.4} (n {})\n── log ──\n{}",
        pressure.paranoia,
        pressure.aggression,
        pressure.instability,
        pressure.cohesion_drift,
        world_f.instability_index,
        world_f.economic_pressure,
        fracture.mean_heuristic,
        fracture.max_heuristic,
        lane,
        sink.last_mean_composed_score,
        sink.last_agent_samples,
        if tail.is_empty() { "—".into() } else { tail }
    );
    for mut t in &mut text_q {
        *t = Text::new(block.clone());
    }
}

fn pressure_composer_egui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<PressureComposerState>,
    mut scratch: ResMut<PressureComposerScratch>,
    mut world_f: ResMut<WorldFields>,
    mut pressure_field: ResMut<PressureField>,
    mut missions: ResMut<ActiveMissions>,
    palette: Res<UiPalette>,
    bindings: Res<InputBindings>,
    mut faction_q: ParamSet<(Query<(Entity, &mut Faction)>, Query<&Faction>)>,
    agents: Query<(Entity, &Agent, Option<&ScriptInfluence>)>,
    links: Query<&AgentFactionLink>,
    regional: Res<RegionalStatsOverlay>,
    log: Res<StrategicEmergenceLog>,
    sink: Res<DecisionPipelineSink>,
    fracture: Res<FractureProbabilityOverlay>,
    mut fracture_settings: ResMut<FractureOverlaySettings>,
) -> Result {
    if !state.visible {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;

    crate::gui::std_floating(egui::Window::new(format!(
        "Pressure composer — climate, not script ({})",
        InputBindings::format_key(bindings.toggle_pressure_composer)
    )))
    .default_size(egui::vec2(520.0, 620.0))
    .show(ctx, |ui| {
        ui.label(
            egui::RichText::new(
                "Authoring: pressure fields + mission packages. No quest chains or forced outcomes.",
            )
            .weak(),
        );
        ui.checkbox(&mut state.show_bevy_strip, "Show Bevy pressure strip (mirror)");
        ui.collapsing("Fracture overlay (dev)", |ui| {
            ui.checkbox(
                &mut fracture_settings.spawn_sub_faction_stub_entities,
                "Spawn SubFactionStub on fracture event (optional dev marker)",
            );
            ui.label(format!(
                "Fracture probability lens: mean {:.3} max {:.3} (informational)",
                fracture.mean_heuristic, fracture.max_heuristic
            ));
        });
        ui.separator();
        ui.horizontal(|ui| {
            use PressureComposerTab::*;
            ui.selectable_value(&mut state.tab, World, "World");
            ui.selectable_value(&mut state.tab, Faction, "Faction");
            ui.selectable_value(&mut state.tab, Agent, "Agent");
            ui.selectable_value(&mut state.tab, Mission, "Mission");
            ui.selectable_value(&mut state.tab, Graph, "Graph");
            ui.selectable_value(&mut state.tab, Log, "Event log");
        });
        ui.separator();

        match state.tab {
            PressureComposerTab::World => {
                ui.heading("World inspector");
                ui.add(egui::Slider::new(&mut world_f.economic_pressure, 0.0..=1.0).text("economic_pressure"));
                ui.add(egui::Slider::new(&mut world_f.instability_index, 0.0..=1.0).text("instability_index"));
                ui.add(egui::Slider::new(&mut world_f.war_tension, 0.0..=1.0).text("war_tension"));
                ui.add(egui::Slider::new(&mut world_f.resource_scarcity, 0.0..=1.0).text("resource_scarcity"));
                ui.add(egui::Slider::new(&mut world_f.public_sentiment, 0.0..=1.0).text("public_sentiment"));
                ui.heading("Global PressureField");
                ui.add(egui::Slider::new(&mut pressure_field.paranoia, 0.0..=1.0).text("paranoia"));
                ui.add(egui::Slider::new(&mut pressure_field.aggression, 0.0..=1.0).text("aggression"));
                ui.add(egui::Slider::new(&mut pressure_field.instability, 0.0..=1.0).text("instability"));
                ui.add(egui::Slider::new(&mut pressure_field.cohesion_drift, 0.0..=1.0).text("cohesion_drift"));
                ui.small("Regional heat (sparse overlay):");
                egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                    let mut ids: Vec<u32> = regional.by_region_id.keys().copied().collect();
                    ids.sort_unstable();
                    for id in ids.iter().take(48) {
                        let st = regional.by_region_id.get(id).copied().unwrap_or_default();
                        ui.label(format!(
                            "region {id}: stability {:.2} corrupt {:.2} militar {:.2}",
                            st.stability, st.corruption, st.militarization
                        ));
                    }
                    if ids.is_empty() {
                        crate::gui::style::muted_text(ui, &palette, "No RegionalStatsOverlay entries yet.");
                    }
                });
            }
            PressureComposerTab::Faction => {
                ui.heading("Faction pressure editor");
                crate::gui::style::warning_text(
                    ui,
                    &palette,
                    "Direct numeric edits — scenario / debug only (inputs, not outcomes).",
                );
                for (_, mut f) in faction_q.p0().iter_mut() {
                    ui.group(|ui| {
                        ui.add(egui::Slider::new(&mut f.cohesion, 0.0..=1.0).text("cohesion"));
                        ui.add(egui::Slider::new(&mut f.control_strength, 0.0..=1.0).text("control_strength"));
                        ui.add(egui::Slider::new(&mut f.resources, 0.0..=500.0).text("resources"));
                    });
                }
                if faction_q.p0().is_empty() {
                    crate::gui::style::muted_text(ui, &palette, "No `Faction` entities in scene.");
                }
            }
            PressureComposerTab::Agent => {
                ui.heading("Agent trait inspector");
                egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                    for (e, agent, script) in agents.iter() {
                        let fac = links
                            .iter()
                            .find(|l| l.agent == e)
                            .map(|l| format!("{:?}", l.faction))
                            .unwrap_or_else(|| "—".into());
                        ui.group(|ui| {
                            ui.label(format!("Entity {:?} | faction link {}", e, fac));
                            ui.label(format!(
                                "mode {:?} | ambition {:.2} paranoia {:.2} empathy {:.2}",
                                agent.mode,
                                agent.traits.ambition,
                                agent.traits.paranoia,
                                agent.traits.empathy
                            ));
                            ui.label(format!(
                                "emotion fear {:.2} anger {:.2} conf {:.2}",
                                agent.emotional_state.fear,
                                agent.emotional_state.anger,
                                agent.emotional_state.confidence
                            ));
                            if let Some(s) = script {
                                ui.label(format!(
                                    "ScriptInfluence pri {:.2} | card instability {:.2}",
                                    s.priority, s.pressure_profile.instability
                                ));
                            } else {
                                ui.small("no ScriptInfluence component");
                            }
                        });
                    }
                    if agents.is_empty() {
                        crate::gui::style::muted_text(ui, &palette, "No `Agent` entities in scene.");
                    }
                });
            }
            PressureComposerTab::Mission => {
                ui.heading("Mission composer (pressure package)");
                ui.add(
                    egui::Slider::new(&mut state.draft_duration, 0..=20_000)
                        .text("duration_ticks (0 = until removed)"),
                );
                if state.draft_duration == 0 {
                    ui.small("0 = manual remove only.");
                }
                ui.add(egui::Slider::new(&mut state.draft_priority, 0.0..=1.0).text("influence priority"));
                ui.label("Global pressure profile:");
                ui.add(egui::Slider::new(&mut state.draft_pressure.paranoia, 0.0..=1.0).text("paranoia"));
                ui.add(egui::Slider::new(&mut state.draft_pressure.aggression, 0.0..=1.0).text("aggression"));
                ui.add(egui::Slider::new(&mut state.draft_pressure.instability, 0.0..=1.0).text("instability"));
                ui.add(egui::Slider::new(&mut state.draft_pressure.cohesion_drift, 0.0..=1.0).text("cohesion_drift"));
                ui.label("Trait bias (ScriptInfluence.bias_vector):");
                ui.add(egui::Slider::new(&mut state.draft_bias.ambition, -0.2..=0.2).text("ambition Δ"));
                ui.add(egui::Slider::new(&mut state.draft_bias.paranoia, -0.2..=0.2).text("paranoia Δ"));
                ui.add(egui::Slider::new(&mut state.draft_bias.cruelty, -0.2..=0.2).text("cruelty Δ"));
                ui.add(egui::Slider::new(&mut state.draft_bias.empathy, -0.2..=0.2).text("empathy Δ"));
                ui.horizontal(|ui| {
                    ui.label("Success readout label:");
                    ui.text_edit_singleline(&mut state.draft_success_label);
                });

                ui.label("Participants:");
                let pick_list: Vec<Entity> = faction_q
                    .p0()
                    .iter()
                    .map(|(e, _)| e)
                    .chain(agents.iter().map(|(e, _, _)| e))
                    .collect();
                if !pick_list.is_empty() {
                    state.entity_picker_cursor = state
                        .entity_picker_cursor
                        .min(pick_list.len().saturating_sub(1));
                    let cur = pick_list[state.entity_picker_cursor];
                    ui.label(format!("cursor {} → {:?}", state.entity_picker_cursor, cur));
                    ui.horizontal(|ui| {
                        if ui.button("←").clicked() && state.entity_picker_cursor > 0 {
                            state.entity_picker_cursor -= 1;
                        }
                        if ui.button("→").clicked() && state.entity_picker_cursor + 1 < pick_list.len() {
                            state.entity_picker_cursor += 1;
                        }
                        if ui.button("add").clicked() && !state.draft_participants.contains(&cur) {
                            state.draft_participants.push(cur);
                        }
                        if ui.button("clear").clicked() {
                            state.draft_participants.clear();
                        }
                    });
                }
                ui.label(format!("picked: {:?}", state.draft_participants));

                if ui.button("Push mission").clicked() {
                    scratch.next_mission_id = scratch.next_mission_id.wrapping_add(1);
                    let dur = if state.draft_duration == 0 {
                        None
                    } else {
                        Some(state.draft_duration)
                    };
                    let lab = if state.draft_success_label.trim().is_empty() {
                        None
                    } else {
                        Some(state.draft_success_label.clone())
                    };
                    let inf = ScriptInfluence {
                        priority: state.draft_priority,
                        bias_vector: state.draft_bias,
                        ..Default::default()
                    };
                    let mut m = MissionT::new(MissionId(scratch.next_mission_id), state.draft_participants.clone(), inf);
                    m.global_pressure = state.draft_pressure;
                    m.duration_ticks = dur;
                    m.success_readout_label = lab;
                    missions.missions.push(m);
                }

                ui.separator();
                ui.heading("Active missions");
                let instab = world_f.instability_index;
                let mut remove_idx: Option<usize> = None;
                for (i, m) in missions.missions.iter().enumerate() {
                    ui.group(|ui| {
                        ui.label(format!(
                            "{:?} | elapsed {:?} / {:?} | n_part {}",
                            m.id, m.ticks_elapsed, m.duration_ticks, m.participants.len()
                        ));
                        let note =
                            mission_success_readout_note(m, &faction_q.p1(), &m.participants, instab);
                        ui.small(note);
                        if ui.button("Remove").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                }
                if let Some(i) = remove_idx {
                    missions.missions.remove(i);
                }
            }
            PressureComposerTab::Graph => {
                ui.heading("Simulation graph (tabular v1)");
                let instab = world_f.instability_index;
                ui.label(format!(
                    "Fracture lens (informational): mean {:.3} max {:.3}",
                    fracture.mean_heuristic, fracture.max_heuristic
                ));
                egui::ScrollArea::vertical().max_height(480.0).show(ui, |ui| {
                    ui.label(format!(
                        "pipeline mean {:.4} | samples {}",
                        sink.last_mean_composed_score, sink.last_agent_samples
                    ));
                    for (e, f) in faction_q.p0().iter() {
                        let h = ((1.0 - f.cohesion) * instab).clamp(0.0, 1.0);
                        ui.label(format!(
                            "{:?} | coh {:.2} ctrl {:.2} | heuristic {:.2}",
                            e, f.cohesion, f.control_strength, h
                        ));
                    }
                });
            }
            PressureComposerTab::Log => {
                ui.heading("Emergence event log");
                egui::ScrollArea::vertical().max_height(520.0).show(ui, |ui| {
                    ui.monospace(log.tail_joined(120));
                });
            }
        }
    });

    Ok(())
}
