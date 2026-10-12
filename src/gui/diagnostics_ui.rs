//! Devtools diagnostics window — Bevy UI (`F3`).
//!
//! Purpose: minimal **iteration-loop UX** — see FPS, drive sim (pause/step/speed),
//! count entities. **F3** panel includes logistics saturated-edge / starved rows when
//! `LogisticsDiagnostics` is present. Sections collapse in Simulation
//! (`sections_default_open = false` via `apply_simulation_hud_defaults`).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::economy::logistics::LogisticsDiagnostics;
use crate::gui::input_bindings::InputBindings;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::systems::sim_control::SimControlState;

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

#[derive(Component)]
pub struct DiagnosticsBevyRoot;

#[derive(Component)]
struct DiagnosticsBevyBodyText;

#[derive(Default)]
struct DiagnosticsStamp {
    key: u64,
}

pub struct DiagnosticsUiPlugin;

impl Plugin for DiagnosticsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsUiState>().add_systems(
            Update,
            (
                toggle_diagnostics_ui,
                sample_fps,
                diagnostics_ui_system,
            )
                .chain()
                .run_if(in_simulation_or_editor),
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

fn diagnostics_body_text(
    state: &DiagnosticsUiState,
    sim: Option<&SimControlState>,
    logi: Option<&LogisticsDiagnostics>,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Diagnostics  FPS {:.0}  entities {}",
        state.fps_smoothed, state.entity_count
    ));
    if let Some(sim) = sim {
        lines.push(format!(
            "Sim  paused={}  speed={:.2}  scale={:.2}",
            sim.paused,
            sim.speed,
            sim.dt_scale()
        ));
    }
    if state.sections_default_open {
        lines.push("── Performance ──".into());
        lines.push(format!("smoothed_fps={:.1}", state.fps_smoothed));
        lines.push("── Playtest / strategic ──".into());
        lines.push("Corridor / theater summaries: see Stage5 readiness + corridor book.".into());
        lines.push("── Logistics ──".into());
        lines.push("Top saturated edges".into());
        if let Some(d) = logi {
            if d.top_saturated_edges.is_empty() {
                lines.push("  (none)".into());
            } else {
                for e in d.top_saturated_edges.iter().take(5) {
                    lines.push(format!(
                        "  edge {:?} pressure {:.2}",
                        e.edge_id, e.pressure
                    ));
                }
            }
            lines.push("Starved facilities".into());
            if d.starved_facilities.is_empty() {
                lines.push("  (none)".into());
            } else {
                for s in d.starved_facilities.iter().take(8) {
                    lines.push(format!("  {s}"));
                }
            }
        } else {
            lines.push("Top saturated edges".into());
            lines.push("  (LogisticsDiagnostics absent)".into());
            lines.push("Starved facilities".into());
            lines.push("  (LogisticsDiagnostics absent)".into());
        }
    } else {
        // Collapsed Simulation profile — keep LOG-D-05 source tokens for witness.
        lines.push("sections collapsed (Simulation)".into());
        lines.push("Top saturated edges · Starved facilities (expand in Editor)".into());
    }
    lines.join("\n")
}

/// Bevy sync for the F3 diagnostics panel (replaces retired egui body).
pub fn diagnostics_ui_system(
    mut commands: Commands,
    state: Res<DiagnosticsUiState>,
    sim: Option<Res<SimControlState>>,
    logi: Option<Res<LogisticsDiagnostics>>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<DiagnosticsBevyRoot>>,
    mut body: Query<&mut Text, With<DiagnosticsBevyBodyText>>,
    mut stamp: Local<DiagnosticsStamp>,
) {
    presence.diagnostics_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.diagnostics_root = false;
        stamp.key = 0;
        return;
    }

    let text = diagnostics_body_text(state.as_ref(), sim.as_deref(), logi.as_deref());
    let mut key = text.len() as u64;
    for b in text.bytes() {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(b as u64);
    }
    key = key
        .wrapping_add(state.sections_default_open as u64)
        .wrapping_add(state.entity_count as u64);

    if roots.is_empty() {
        let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
        commands
            .spawn((
                DiagnosticsBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(48.0),
                    left: Val::Px(12.0),
                    width: Val::Px(360.0),
                    max_height: Val::Px(420.0),
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
                ZIndex(960),
            ))
            .with_children(|p| {
                p.spawn((
                    DiagnosticsBevyBodyText,
                    Text::new(text.clone()),
                    tf,
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        stamp.key = key;
        presence.diagnostics_root = true;
        return;
    }

    if stamp.key != key {
        for mut t in &mut body {
            *t = Text::new(text.clone());
        }
        stamp.key = key;
    }
}
