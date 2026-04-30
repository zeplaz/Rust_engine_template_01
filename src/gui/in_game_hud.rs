// In-game HUD using native Bevy UI — Node component pattern.
// egui is NOT used here; this is player-facing runtime UI.
//
// See: prompts/guides/ui_boundary_guide_v1.md

use bevy::prelude::*;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ResourceDisplay;

pub struct InGameHudPlugin;

impl Plugin for InGameHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_resource_display);
    }
}

fn spawn_hud(mut commands: Commands) {
    // Root HUD container: anchored top-left, transparent.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        HudRoot,
    ))
    .with_children(|parent| {
        // Resource counter placeholder — will be driven by ProductionComponent queries.
        parent.spawn((
            Node { ..default() },
            Text::new("Resources: --"),
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ResourceDisplay,
        ));
    });
}

fn update_resource_display(
    // TODO: query ProductionComponent / WorldData resources and format text.
    _query: Query<&mut Text, With<ResourceDisplay>>,
) {
    // Placeholder: will be wired to production resource totals.
}
