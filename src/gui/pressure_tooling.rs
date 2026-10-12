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
use bevy::ui::FocusPolicy;

use crate::gui::input_bindings::InputBindings;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::strategic::{
    ActiveMissions, DecisionPipelineSink, FractureProbabilityOverlay, GpuBridgeState,
    HybridAgentTraits, MissionPressure, PressureField, PressureProfile, StrategicEmergenceLog,
    WorldFields,
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
    pub draft_mission_pressure: MissionPressure,
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
            // Production: off — strip painted "Pres/Worl" into sim chrome (VISUAL-JANK-CHROME-001).
            // Arm via F2 composer checkbox or set true in Editor tooling.
            show_bevy_strip: false,
            draft_duration: 2_000,
            draft_pressure: PressureProfile {
                paranoia: 0.15,
                aggression: 0.1,
                instability: 0.12,
                cohesion_drift: 0.08,
            },
            draft_mission_pressure: MissionPressure::default(),
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
                (
                    toggle_pressure_composer,
                    sync_pressure_composer_bevy.after(toggle_pressure_composer),
                    sync_pressure_bevy_overlay_visibility,
                )
                    .run_if(in_simulation_or_editor),
            );
    }
}

#[derive(Component)]
pub struct PressureComposerBevyRoot;

#[derive(Component)]
struct PressureComposerBevyBodyText;

#[derive(Default)]
struct PressureComposerStamp {
    key: u64,
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

fn spawn_pressure_bevy_overlay(
    mut commands: Commands,
    existing: Query<Entity, With<PressureBevyOverlayRoot>>,
    palette: Res<UiPalette>,
    mono: Res<super::CmdUiMonoFont>,
) {
    if !existing.is_empty() {
        return;
    }
    let tf = TextFont::from_font_size(12.5).with_font(mono.0.clone());
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
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            Visibility::Hidden,
            ZIndex(900),
        ))
        .with_children(|p| {
            p.spawn((
                PressureBevyOverlayText,
                Text::new("Pressure — …"),
                tf,
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

fn despawn_pressure_bevy_overlay(mut commands: Commands, q: Query<Entity, With<PressureBevyOverlayRoot>>) {
    for e in &q {
        commands.entity(e).try_despawn();
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

fn pressure_composer_body(
    state: &PressureComposerState,
    world_f: &WorldFields,
    pressure: &PressureField,
    missions: Option<&ActiveMissions>,
    log: &StrategicEmergenceLog,
) -> String {
    let tab = match state.tab {
        PressureComposerTab::World => "World",
        PressureComposerTab::Faction => "Faction",
        PressureComposerTab::Agent => "Agent",
        PressureComposerTab::Mission => "Mission",
        PressureComposerTab::Graph => "Graph",
        PressureComposerTab::Log => "Log",
    };
    let mut lines = vec![format!("Pressure composer ({tab})")];
    lines.push(format!(
        "Field  p {:.2} a {:.2} i {:.2} cd {:.2}",
        pressure.paranoia, pressure.aggression, pressure.instability, pressure.cohesion_drift
    ));
    lines.push(format!(
        "World  instab {:.2} econ {:.2}",
        world_f.instability_index, world_f.economic_pressure
    ));
    if let Some(m) = missions {
        lines.push(format!("Active missions: {}", m.missions.len()));
    }
    let tail = log.tail_joined(4);
    lines.push("── log ──".into());
    lines.push(if tail.is_empty() {
        "—".into()
    } else {
        tail
    });
    lines.push(format!(
        "Bevy strip: {}",
        if state.show_bevy_strip { "ON" } else { "OFF" }
    ));
    lines.join("\n")
}

fn sync_pressure_composer_bevy(
    mut commands: Commands,
    state: Res<PressureComposerState>,
    world_f: Res<WorldFields>,
    pressure: Res<PressureField>,
    missions: Option<Res<ActiveMissions>>,
    log: Res<StrategicEmergenceLog>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<PressureComposerBevyRoot>>,
    mut body: Query<&mut Text, With<PressureComposerBevyBodyText>>,
    mut stamp: Local<PressureComposerStamp>,
) {
    presence.pressure_composer_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.pressure_composer_root = false;
        stamp.key = 0;
        return;
    }

    let text = pressure_composer_body(
        state.as_ref(),
        world_f.as_ref(),
        pressure.as_ref(),
        missions.as_deref(),
        log.as_ref(),
    );
    let mut key = text.len() as u64;
    for b in text.bytes() {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(b as u64);
    }

    if roots.is_empty() {
        let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
        commands
            .spawn((
                PressureComposerBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(56.0),
                    left: Val::Px(16.0),
                    width: Val::Px(340.0),
                    max_height: Val::Px(440.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::ZERO,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(palette.bevy_hud_panel_fill()),
                BorderColor::all(palette.bevy_wire_magenta()),
                FocusPolicy::Block,
                ZIndex(954),
            ))
            .with_children(|p| {
                p.spawn((
                    PressureComposerBevyBodyText,
                    Text::new(text.clone()),
                    tf,
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        stamp.key = key;
        presence.pressure_composer_root = true;
        return;
    }

    if stamp.key != key {
        for mut t in &mut body {
            *t = Text::new(text.clone());
        }
        stamp.key = key;
    }
    presence.pressure_composer_root = true;
}
