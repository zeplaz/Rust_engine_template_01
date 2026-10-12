//! Faction tools window — Bevy UI (`F4`).
//!
//! Stub UI for the faction editor; populated against `FactionBlueprint` once the data model lands.
//!
//! Designer:
//! - `prompts/designer_questions/factions/faction_editor/02_ui_egui_panels.md`
//! - `prompts/designer_questions/factions/faction_editor/01_data_model.md`
//! - `prompts/designer_questions/factions/implementation_questions_v1.md`

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::gui::input_bindings::InputBindings;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionToolsPanel {
    Roster,
    Blueprint,
    Diplomacy,
    ImportExport,
}

#[derive(Resource, Debug, Clone)]
pub struct FactionToolsState {
    pub visible: bool,
    pub active: FactionToolsPanel,
}

impl Default for FactionToolsState {
    fn default() -> Self {
        Self {
            visible: false,
            active: FactionToolsPanel::Roster,
        }
    }
}

#[derive(Component)]
pub struct FactionToolsBevyRoot;

#[derive(Component)]
struct FactionToolsBevyBodyText;

#[derive(Default)]
struct FactionToolsStamp {
    key: u64,
}

pub struct FactionToolsUiPlugin;

impl Plugin for FactionToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionToolsState>().add_systems(
            Update,
            (
                toggle_faction_tools,
                sync_faction_tools_bevy.after(toggle_faction_tools),
            )
                .run_if(in_simulation_or_editor),
        );
    }
}

fn toggle_faction_tools(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<FactionToolsState>,
) {
    if keys.just_pressed(bindings.toggle_faction_tools) {
        state.visible = !state.visible;
    }
}

fn faction_tools_body(state: &FactionToolsState) -> String {
    let tab = match state.active {
        FactionToolsPanel::Roster => "Roster",
        FactionToolsPanel::Blueprint => "Blueprint",
        FactionToolsPanel::Diplomacy => "Diplomacy",
        FactionToolsPanel::ImportExport => "Import / Export",
    };
    format!(
        "Faction tools ({tab})\n\
         Stub — FactionBlueprint data model pending.\n\
         Tabs: Roster · Blueprint · Diplomacy · Import/Export"
    )
}

fn sync_faction_tools_bevy(
    mut commands: Commands,
    state: Res<FactionToolsState>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<FactionToolsBevyRoot>>,
    mut body: Query<&mut Text, With<FactionToolsBevyBodyText>>,
    mut stamp: Local<FactionToolsStamp>,
) {
    presence.faction_tools_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.faction_tools_root = false;
        stamp.key = 0;
        return;
    }

    let text = faction_tools_body(state.as_ref());
    let mut key = text.len() as u64;
    for b in text.bytes() {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(b as u64);
    }
    key = key.wrapping_add(match state.active {
        FactionToolsPanel::Roster => 1,
        FactionToolsPanel::Blueprint => 2,
        FactionToolsPanel::Diplomacy => 3,
        FactionToolsPanel::ImportExport => 4,
    });

    if roots.is_empty() {
        let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
        commands
            .spawn((
                FactionToolsBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(64.0),
                    right: Val::Px(16.0),
                    width: Val::Px(320.0),
                    max_height: Val::Px(360.0),
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
                ZIndex(955),
            ))
            .with_children(|p| {
                p.spawn((
                    FactionToolsBevyBodyText,
                    Text::new(text.clone()),
                    tf,
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        stamp.key = key;
        presence.faction_tools_root = true;
        return;
    }

    if stamp.key != key {
        for mut t in &mut body {
            *t = Text::new(text.clone());
        }
        stamp.key = key;
    }
    presence.faction_tools_root = true;
}
