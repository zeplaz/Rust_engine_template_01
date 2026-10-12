//! Pick logistics HUD focus from a list (hotkey, **Options** keybindings, this Bevy panel).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

// Declared here (not in `gui/mod.rs`) so the logistics lane owns registration; resolves to
// `src/gui/logistics_orders_panel.rs`.
#[path = "logistics_orders_panel.rs"]
pub mod logistics_orders_panel;
#[path = "logistics_orders_overlay.rs"]
pub mod logistics_orders_overlay;
pub use logistics_orders_panel::{LogisticsOrdersPanelPlugin, LogisticsOrdersPanelState};
pub use logistics_orders_overlay::{build_orders_overlay, OrdersOverlayFrame, RouteTone};

use crate::entities::production::core::LogisticsSiteRoot;
use crate::gui::input_bindings::InputBindings;
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;

#[derive(Resource, Debug, Clone)]
pub struct LogisticsTargetsPanelState {
    pub visible: bool,
}

impl Default for LogisticsTargetsPanelState {
    fn default() -> Self {
        Self { visible: false }
    }
}

#[derive(Component)]
pub struct LogisticsTargetsBevyRoot;

#[derive(Component)]
struct LogisticsTargetsPickButton(Entity);

#[derive(Default)]
struct TargetsStamp {
    key: u64,
}

pub struct LogisticsTargetsPanelPlugin;

impl Plugin for LogisticsTargetsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LogisticsOrdersPanelPlugin);
        app.init_resource::<LogisticsTargetsPanelState>()
            .add_systems(
                Update,
                (
                    toggle_logistics_targets_panel,
                    sync_logistics_targets_bevy_panel,
                    logistics_targets_pick_clicks,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn toggle_logistics_targets_panel(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<LogisticsTargetsPanelState>,
) {
    // Plain F6 — Shift+F6 is owned by the orders panel.
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if !shift && keys.just_pressed(bindings.toggle_logistics_targets_panel) {
        state.visible = !state.visible;
    }
}

fn sync_logistics_targets_bevy_panel(
    mut commands: Commands,
    state: Res<LogisticsTargetsPanelState>,
    focus: Res<HudLogisticsFocus>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<LogisticsTargetsBevyRoot>>,
    sites: Query<(Entity, Option<&Name>), With<LogisticsSiteRoot>>,
    mut stamp: Local<TargetsStamp>,
) {
    presence.logistics_targets_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.logistics_targets_root = false;
        stamp.key = 0;
        return;
    }

    let mut site_rows: Vec<(Entity, String)> = sites
        .iter()
        .map(|(e, name)| {
            let label = name
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|| format!("site {e:?}"));
            (e, label)
        })
        .collect();
    site_rows.sort_by(|a, b| a.1.cmp(&b.1));

    let mut key = site_rows.len() as u64 + state.visible as u64;
    if let Some(sel) = focus.tracked_entity {
        key = key.wrapping_add(sel.to_bits());
    }
    for (e, _) in site_rows.iter().take(64) {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(e.to_bits());
    }

    if !roots.is_empty() && stamp.key == key {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }

    let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
    commands
        .spawn((
            LogisticsTargetsBevyRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0),
                right: Val::Px(360.0),
                width: Val::Px(280.0),
                max_height: Val::Px(420.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
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
        .with_children(|root| {
            root.spawn((
                Text::new("Logistics targets"),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            if site_rows.is_empty() {
                root.spawn((
                    Text::new("(no LogisticsSiteRoot entities)"),
                    tf.clone(),
                    TextColor(palette.bevy_text_muted()),
                ));
            }
            for (entity, label) in site_rows {
                let selected = focus.tracked_entity == Some(entity);
                root.spawn((
                    LogisticsTargetsPickButton(entity),
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(if selected {
                        palette.bevy_button_idle()
                    } else {
                        palette.bevy_hud_panel_fill()
                    }),
                    BorderColor::all(palette.bevy_wire_magenta()),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        tf.clone(),
                        TextColor(palette.bevy_primary_text()),
                    ));
                });
            }
        });
    stamp.key = key;
    presence.logistics_targets_root = true;
}

fn logistics_targets_pick_clicks(
    mut focus: ResMut<HudLogisticsFocus>,
    interactions: Query<(&Interaction, &LogisticsTargetsPickButton), Changed<Interaction>>,
) {
    for (interaction, pick) in &interactions {
        if *interaction == Interaction::Pressed {
            focus.tracked_entity = Some(pick.0);
        }
    }
}
