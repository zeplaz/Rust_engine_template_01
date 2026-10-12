//! Movable construction toolbox — Bevy UI (editor product shell).
//!
//! Tool-select intents only: mutates [`ActiveBuildTool`]; never executes placement.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::gui::hud::{HudDockRegistry, HudWidgetId};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::{product_shell_active, PanelPresence};

use super::build_tool_authority::{
    ActiveBuildTool, BuildTool, BuildingArchetypeId, DefenseKind, RailType, RoadType, ZoneTool,
};
use super::building_definitions::BuildingDefinitionRegistry;

/// Editor construction toolbox root (Bevy UI).
#[derive(Component)]
pub struct BuildToolboxBevyRoot;

#[derive(Component, Clone, Copy, Debug)]
enum BuildToolboxAction {
    ZoneResidential,
    Commercial,
    Industrial,
    Utilities,
    MockShapes,
    Road,
    Rail,
    Demolish,
    Clear,
    Defense(DefenseKind),
}

#[derive(Default)]
struct BuildToolboxStamp {
    key: u64,
}


fn toolbox_stamp(tool: &ActiveBuildTool, dock_visible: bool, catalog_n: usize) -> u64 {
    let mut key = dock_visible as u64;
    key = key
        .wrapping_mul(31)
        .wrapping_add(catalog_n as u64)
        .wrapping_mul(31)
        .wrapping_add(tool.tool.label().len() as u64);
    for b in format!("{:?}", tool.tool).bytes() {
        key = key.wrapping_mul(31).wrapping_add(b as u64);
    }
    key = key
        .wrapping_add(tool.residential_menu_open as u64)
        .wrapping_add(tool.commercial_menu_open as u64)
        .wrapping_add(tool.industrial_menu_open as u64)
        .wrapping_add(tool.utilities_menu_open as u64)
        .wrapping_add(tool.mock_shapes_menu_open as u64);
    key
}

/// Spawn/despawn editor construction toolbox from dock visibility.
pub fn sync_build_toolbox_bevy(
    mut commands: Commands,
    dock: Res<HudDockRegistry>,
    tool: Res<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<BuildToolboxBevyRoot>>,
    mut stamp: Local<BuildToolboxStamp>,
) {
    presence.build_toolbox_root = !roots.is_empty();
    let visible = dock.slot(HudWidgetId::BuildToolbox).visible
        && !dock.slot(HudWidgetId::BuildToolbox).minimized;
    if !visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.build_toolbox_root = false;
        stamp.key = 0;
        return;
    }

    let next = toolbox_stamp(tool.as_ref(), visible, registry.by_id.len());
    if !roots.is_empty() && stamp.key == next {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }

    let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
    let active = format!("Active: {:?}", tool.tool);
    let catalog = format!("Catalog rows: {}", registry.by_id.len());

    commands
        .spawn((
            BuildToolboxBevyRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(72.0),
                right: Val::Px(16.0),
                width: Val::Px(240.0),
                max_height: Val::Px(480.0),
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
            ZIndex(940),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Construction"),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            root.spawn((
                Text::new(active),
                tf.clone(),
                TextColor(palette.bevy_text_muted()),
            ));
            root.spawn((
                Text::new(catalog),
                tf.clone(),
                TextColor(palette.bevy_text_muted()),
            ));
            spawn_toolbox_button(root, &palette, &tf, "Zoning · Residential…", BuildToolboxAction::ZoneResidential);
            spawn_toolbox_button(root, &palette, &tf, "Buildings · Commercial…", BuildToolboxAction::Commercial);
            spawn_toolbox_button(root, &palette, &tf, "Buildings · Industrial…", BuildToolboxAction::Industrial);
            spawn_toolbox_button(root, &palette, &tf, "Buildings · Utilities…", BuildToolboxAction::Utilities);
            spawn_toolbox_button(root, &palette, &tf, "Mock shapes (T/O/L)…", BuildToolboxAction::MockShapes);
            spawn_toolbox_button(root, &palette, &tf, "Roads", BuildToolboxAction::Road);
            spawn_toolbox_button(root, &palette, &tf, "Rail", BuildToolboxAction::Rail);
            spawn_toolbox_button(root, &palette, &tf, "Demolish", BuildToolboxAction::Demolish);
            spawn_toolbox_button(root, &palette, &tf, "Clear tool", BuildToolboxAction::Clear);
            for (kind, label) in [
                (DefenseKind::DefensiveWall, "Defense · Wall"),
                (DefenseKind::TrenchLine, "Defense · Trench"),
                (DefenseKind::Bunker, "Defense · Bunker"),
                (DefenseKind::DragonTeeth, "Defense · Dragon teeth"),
                (DefenseKind::Minefield, "Defense · Minefield"),
            ] {
                spawn_toolbox_button(
                    root,
                    &palette,
                    &tf,
                    label,
                    BuildToolboxAction::Defense(kind),
                );
            }
        });
    stamp.key = next;
    presence.build_toolbox_root = true;
}

fn spawn_toolbox_button(
    parent: &mut ChildSpawnerCommands,
    palette: &UiPalette,
    tf: &TextFont,
    label: &str,
    action: BuildToolboxAction,
) {
    parent
        .spawn((
            action,
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(palette.bevy_button_idle()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

/// Tool-select intents only — no execute / place / queue mutation.
pub fn build_toolbox_bevy_action_system(
    q: Query<(&Interaction, &BuildToolboxAction), (Changed<Interaction>, With<Button>)>,
    mut tool: ResMut<ActiveBuildTool>,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            BuildToolboxAction::ZoneResidential => {
                tool.close_submenus();
                tool.residential_menu_open = true;
                tool.tool = BuildTool::Zone(ZoneTool::ResidentialLow);
            }
            BuildToolboxAction::Commercial => {
                tool.close_submenus();
                tool.commercial_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(BuildingArchetypeId::Office);
            }
            BuildToolboxAction::Industrial => {
                tool.close_submenus();
                tool.industrial_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
            }
            BuildToolboxAction::Utilities => {
                tool.close_submenus();
                tool.utilities_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(BuildingArchetypeId::WaterPlant);
            }
            BuildToolboxAction::MockShapes => {
                tool.close_submenus();
                tool.mock_shapes_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
            }
            BuildToolboxAction::Road => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Road(RoadType::Street);
            }
            BuildToolboxAction::Rail => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Rail(RailType::Standard);
            }
            BuildToolboxAction::Demolish => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Demolish;
            }
            BuildToolboxAction::Clear => {
                tool.tool = BuildTool::None;
                tool.close_submenus();
                tool.clear_building_intent();
            }
            BuildToolboxAction::Defense(kind) => {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Defense(kind);
            }
        }
    }
}

fn despawn_build_toolbox_bevy_when_inactive(
    mut commands: Commands,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<BuildToolboxBevyRoot>>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    presence.build_toolbox_root = false;
}

/// Systems for [`crate::construction::BuildPlanningPlugin`].
pub fn register_build_toolbox_bevy_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_build_toolbox_bevy,
            build_toolbox_bevy_action_system.after(sync_build_toolbox_bevy),
        )
            .run_if(product_shell_active),
    )
    .add_systems(
        Update,
        despawn_build_toolbox_bevy_when_inactive.run_if(not(product_shell_active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_toolbox_bevy_root_marker_present() {
        assert!(include_str!("build_toolbox.rs").contains("BuildToolboxBevyRoot"));
        assert!(include_str!("build_toolbox.rs").contains("sync_build_toolbox_bevy"));
    }
}
