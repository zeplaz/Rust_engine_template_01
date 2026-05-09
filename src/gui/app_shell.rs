//! Application shell — **Bevy UI only**: main menu + load stub. No egui chrome here.
//!
//! See `prompts/guides/ui_boundary_guide_v1.md` + shell refactor direction.

use crate::engine::states::{BaseState, MainMenuState, WorldGenFlowState};
use crate::gui::AppStartState;
use crate::gui::ui_windows::UiState;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, WorldMarker,
};
use bevy::app::AppExit;
use bevy::prelude::*;

/// Root entity for the Bevy main-menu layout (despawned when leaving front-end menu).
#[derive(Component)]
pub struct MainMenuShellRoot;

/// Root entity for the load-stub screen (Bevy UI).
#[derive(Component)]
pub struct LoadMenuShellRoot;

#[derive(Component, Clone, Copy)]
enum MainMenuButtonAction {
    NewWorld,
    DebugEnter,
    OpenLoad,
    NewMapEditor,
    Quit,
}

#[derive(Component, Clone, Copy)]
enum LoadMenuButtonAction {
    Cancel,
    EditorStub,
    SimStub,
}

/// Stub path label for load flows (no text-field widget yet; edit here or wire saves later).
#[derive(Resource, Debug, Clone)]
pub struct LoadStubPath(pub String);

impl Default for LoadStubPath {
    fn default() -> Self {
        Self("saves/slot_0.ron".to_string())
    }
}

pub struct AppShellPlugin;

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadStubPath>()
            .add_systems(Update, (sync_menu_shell, handle_main_menu_buttons, handle_load_menu_buttons));
    }
}

fn sync_menu_shell(
    app_start: Res<State<AppStartState>>,
    base: Res<State<BaseState>>,
    menu: Res<State<MainMenuState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_state: ResMut<UiState>,
    main_q: Query<Entity, With<MainMenuShellRoot>>,
    load_q: Query<Entity, With<LoadMenuShellRoot>>,
    load_path: Res<LoadStubPath>,
) {
    if *app_start.get() != AppStartState::Menu || *base.get() != BaseState::MainMenu {
        for e in main_q.iter() {
            commands.entity(e).despawn();
        }
        for e in load_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    match *menu.get() {
        MainMenuState::MainMenu => {
            if load_q.iter().next().is_some() {
                for e in load_q.iter() {
                    commands.entity(e).despawn();
                }
            }
            if main_q.is_empty() {
                spawn_main_menu(&mut commands, &asset_server, &mut ui_state);
            }
        }
        MainMenuState::Load => {
            if main_q.iter().next().is_some() {
                for e in main_q.iter() {
                    commands.entity(e).despawn();
                }
            }
            if load_q.is_empty() {
                spawn_load_menu(&mut commands, &load_path);
            }
        }
        MainMenuState::Settings | MainMenuState::Editor => {
            for e in main_q.iter() {
                commands.entity(e).despawn();
            }
            for e in load_q.iter() {
                commands.entity(e).despawn();
            }
        }
    }
}

const MENU_BTN_BG: Color = Color::srgb(0.16, 0.2, 0.28);

fn spawn_main_menu(commands: &mut Commands, asset_server: &AssetServer, ui_state: &mut UiState) {
    if ui_state.font_handle.is_none() {
        ui_state.font_handle = Some(asset_server.load("fonts/FiraMono-Medium.ttf"));
    }

    let border_radius = BorderRadius::all(Val::Px(6.0));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.07, 0.1)),
            MainMenuShellRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Proc Alpha Dine"),
                TextColor(Color::srgb(0.92, 0.93, 0.96)),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));
            for (label, action) in [
                ("New World", MainMenuButtonAction::NewWorld),
                ("Enter simulation (debug)", MainMenuButtonAction::DebugEnter),
                ("Load World", MainMenuButtonAction::OpenLoad),
                ("New map in editor", MainMenuButtonAction::NewMapEditor),
                ("Quit", MainMenuButtonAction::Quit),
            ] {
                parent
                    .spawn((
                        Button,
                        Node {
                            min_width: Val::Px(280.0),
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius,
                            ..default()
                        },
                        BackgroundColor(MENU_BTN_BG),
                    ))
                    .insert(action)
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            TextColor(Color::srgb(0.9, 0.92, 0.98)),
                        ));
                    });
            }
        });
}

fn spawn_load_menu(commands: &mut Commands, load_path: &LoadStubPath) {
    let border_radius = BorderRadius::all(Val::Px(6.0));
    let path_display = load_path.0.clone();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.07, 0.1)),
            LoadMenuShellRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Load World (stub)"),
                TextColor(Color::srgb(0.92, 0.93, 0.96)),
            ));
            parent.spawn((
                Text::new(
                    "No file picker yet — path is developer-configurable via LoadStubPath resource.",
                ),
                TextColor(Color::srgb(0.65, 0.72, 0.82)),
                Node {
                    max_width: Val::Px(520.0),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(format!("Path: {path_display}")),
                TextColor(Color::srgb(0.78, 0.85, 0.95)),
            ));
            for (label, action) in [
                ("Cancel", LoadMenuButtonAction::Cancel),
                (
                    "Open in editor (stub)",
                    LoadMenuButtonAction::EditorStub,
                ),
                ("Load into game (stub)", LoadMenuButtonAction::SimStub),
            ] {
                parent
                    .spawn((
                        Button,
                        Node {
                            min_width: Val::Px(260.0),
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius,
                            ..default()
                        },
                        BackgroundColor(MENU_BTN_BG),
                    ))
                    .insert(action)
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            TextColor(Color::srgb(0.9, 0.92, 0.98)),
                        ));
                    });
            }
        });
}

fn handle_main_menu_buttons(
    q: Query<
        (&Interaction, &MainMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut next_world_flow: ResMut<NextState<WorldGenFlowState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            MainMenuButtonAction::NewWorld => {
                despawn_generated_world_entities(&mut commands, &world_roots);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::NewWorldSetup);
            }
            MainMenuButtonAction::DebugEnter => {
                NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
            }
            MainMenuButtonAction::OpenLoad => {
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::LoadingSave);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::Load);
                info!(
                    "Load World: procedural generation is disabled in this flow; deserialize when saves are wired."
                );
            }
            MainMenuButtonAction::NewMapEditor => {
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_base, BaseState::Editor);
            }
            MainMenuButtonAction::Quit => {
                app_exit.write(AppExit::Success);
            }
        }
    }
}

fn handle_load_menu_buttons(
    q: Query<
        (&Interaction, &LoadMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    path: Res<LoadStubPath>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut next_world_flow: ResMut<NextState<WorldGenFlowState>>,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            LoadMenuButtonAction::Cancel => {
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
            }
            LoadMenuButtonAction::EditorStub => {
                info!(
                    "Open saved map in editor (stub): no file read yet — would load {:?}",
                    path.0.as_str()
                );
                despawn_generated_world_entities(&mut commands, &world_roots);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_base, BaseState::Editor);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
            }
            LoadMenuButtonAction::SimStub => {
                info!(
                    "Load stub: no file read yet — would load {:?}; entering simulation.",
                    path.0.as_str()
                );
                despawn_generated_world_entities(&mut commands, &world_roots);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
            }
        }
    }
}
