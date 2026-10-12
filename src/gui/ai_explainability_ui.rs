//! L7 **AI explainability** window — Bevy UI; macro resolution + micro pipeline.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::gui::input_bindings::InputBindings;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::strategic::{
    format_hybrid_telemetry_explain, DecisionExplainabilitySnapshot, HybridSimLastResolved,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct ExplainabilityUiState {
    pub visible: bool,
}

#[derive(Component)]
pub struct AiExplainabilityBevyRoot;

#[derive(Component)]
struct AiExplainabilityBevyBodyText;

#[derive(Default)]
struct AiExplainStamp {
    key: u64,
}

pub struct AiExplainabilityPlugin;

impl Plugin for AiExplainabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplainabilityUiState>().add_systems(
            Update,
            (
                toggle_explainability_ui,
                sync_ai_explainability_bevy.after(toggle_explainability_ui),
            )
                .run_if(in_simulation_or_editor),
        );
    }
}

fn toggle_explainability_ui(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<ExplainabilityUiState>,
) {
    if keys.just_pressed(bindings.toggle_ai_explainability) {
        state.visible = !state.visible;
    }
}

fn explainability_body(
    hybrid: Option<&HybridSimLastResolved>,
    decision: Option<&DecisionExplainabilitySnapshot>,
) -> String {
    let mut lines = vec!["AI explainability".to_string()];
    match hybrid.and_then(|h| h.telemetry.as_ref()) {
        Some(t) => lines.extend(format_hybrid_telemetry_explain(t)),
        None => lines.push("Hybrid telemetry: (absent)".into()),
    }
    if let Some(d) = decision {
        lines.push(format!(
            "Decision tick {}  composed {:.4}",
            d.sim_tick, d.composed
        ));
        if d.pipeline_contributors.is_empty() {
            lines.push("Pipeline contributors: (none)".into());
        } else {
            lines.push("Pipeline contributors:".into());
            for r in d.pipeline_contributors.iter().take(8) {
                lines.push(format!("  · {r}"));
            }
        }
    } else {
        lines.push("DecisionExplainabilitySnapshot: (absent)".into());
    }
    lines.join("\n")
}

fn sync_ai_explainability_bevy(
    mut commands: Commands,
    state: Res<ExplainabilityUiState>,
    hybrid: Option<Res<HybridSimLastResolved>>,
    decision: Option<Res<DecisionExplainabilitySnapshot>>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<AiExplainabilityBevyRoot>>,
    mut body: Query<&mut Text, With<AiExplainabilityBevyBodyText>>,
    mut stamp: Local<AiExplainStamp>,
) {
    presence.ai_explainability_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.ai_explainability_root = false;
        stamp.key = 0;
        return;
    }

    let text = explainability_body(hybrid.as_deref(), decision.as_deref());
    let mut key = text.len() as u64;
    for b in text.bytes() {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(b as u64);
    }

    if roots.is_empty() {
        let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
        commands
            .spawn((
                AiExplainabilityBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(72.0),
                    left: Val::Px(380.0),
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
                ZIndex(956),
            ))
            .with_children(|p| {
                p.spawn((
                    AiExplainabilityBevyBodyText,
                    Text::new(text.clone()),
                    tf,
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        stamp.key = key;
        presence.ai_explainability_root = true;
        return;
    }

    if stamp.key != key {
        for mut t in &mut body {
            *t = Text::new(text.clone());
        }
        stamp.key = key;
    }
    presence.ai_explainability_root = true;
}
