//! Application shell — **Bevy UI only**: main menu + load stub. No egui chrome here.
//!
//! See `prompts/guides/ui_boundary_guide_v1.md` + shell refactor direction.

use crate::engine::states::{BaseState, MainMenuState, WorldGenFlowState};
use crate::engine::AppState;
use crate::engine::ux_orchestration::{ux_begin_world_gen_from_menu, UxBridgeSet};
use crate::gui::AppStartState;
use crate::gui::CmdUiMonoFont;
use crate::gui::ui_windows::UiState;
use crate::gui::UiPalette;
use crate::engine::debug_maneuver::{DebugManeuver, FrameLayoutDebugSession};
use crate::engine::test_harness::{
    arm_debug_quick_world_gen, DebugQuickWorldGenPending, TestWorldHarness,
};
use crate::engine::TestScene;
use crate::gui::editor::world_gen_ui::CancelActiveWorldGenEvent;
use crate::gui::pause_menu_confirm::world_gen_work_active;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, WorldGenJobSlot, WorldGenParams, WorldGenProgress,
    WorldMarker,
};
use bevy::app::AppExit;
use bevy::ecs::system::SystemParam;
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
    /// ① Frame / layout screen test (192², layout debug, stay open).
    DebugFrameLayout,
    /// ③ Demo debug world (192², auto gen, stay open).
    DebugDemoOpen,
    /// Fire + weather + atmosphere sandbox (320², stay open).
    DebugVfxSandbox,
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

#[derive(SystemParam)]
struct MenuDebugBootstrap<'w> {
    next_app: ResMut<'w, NextState<crate::engine::AppState>>,
    next_wg: ResMut<'w, NextState<crate::engine::WorldGenState>>,
    next_base: ResMut<'w, NextState<BaseState>>,
    next_world_flow: ResMut<'w, NextState<WorldGenFlowState>>,
    chrome_latch: ResMut<'w, crate::engine::WorldGenChromeLatch>,
    world_gen_ui: ResMut<'w, crate::gui::editor::world_gen_ui::WorldGenUiState>,
    world_preview_ui: ResMut<'w, crate::gui::editor::world_preview::WorldPreviewUiState>,
    params: ResMut<'w, WorldGenParams>,
    debug_quick: ResMut<'w, DebugQuickWorldGenPending>,
    harness: ResMut<'w, TestWorldHarness>,
    layout_session: ResMut<'w, FrameLayoutDebugSession>,
    job: Res<'w, WorldGenJobSlot>,
    progress: Res<'w, WorldGenProgress>,
    cancel_world_gen: MessageWriter<'w, CancelActiveWorldGenEvent>,
}

impl Plugin for AppShellPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadStubPath>()
            .add_systems(
                Update,
                (
                    sync_menu_shell,
                    handle_main_menu_buttons,
                    handle_main_menu_debug_buttons,
                    handle_load_menu_buttons,
                )
                    .before(UxBridgeSet),
            );
    }
}

fn sync_menu_shell(
    app_start: Res<State<AppStartState>>,
    base: Res<State<BaseState>>,
    ux_app: Res<State<AppState>>,
    menu: Res<State<MainMenuState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_state: ResMut<UiState>,
    main_q: Query<Entity, With<MainMenuShellRoot>>,
    load_q: Query<Entity, With<LoadMenuShellRoot>>,
    load_path: Res<LoadStubPath>,
    palette: Res<UiPalette>,
    cmd_mono: Res<CmdUiMonoFont>,
) {
    // UX AppState wins over transient legacy BaseState flicker during world-gen bootstrap.
    if *app_start.get() != AppStartState::Menu
        || *base.get() != BaseState::MainMenu
        || *ux_app.get() != AppState::Setup
    {
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
                spawn_main_menu(
                    &mut commands,
                    &asset_server,
                    &mut ui_state,
                    palette.as_ref(),
                    cmd_mono.as_ref(),
                );
            }
        }
        MainMenuState::Load => {
            if main_q.iter().next().is_some() {
                for e in main_q.iter() {
                    commands.entity(e).despawn();
                }
            }
            if load_q.is_empty() {
                spawn_load_menu(
                    &mut commands,
                    &load_path,
                    palette.as_ref(),
                    cmd_mono.as_ref(),
                );
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

fn spawn_main_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_state: &mut UiState,
    palette: &UiPalette,
    cmd_mono: &CmdUiMonoFont,
) {
    if ui_state.font_handle.is_none() {
        ui_state.font_handle = Some(asset_server.load("fonts/FiraMono-Medium.ttf"));
    }

    let sharp = BorderRadius::ZERO;
    let menu_pt = 16.0;
    let tf = |pt: f32| TextFont::from_font_size(pt).with_font(cmd_mono.0.clone());

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
            BackgroundColor(palette.bevy_backdrop()),
            MainMenuShellRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Proc Alpha Dine"),
                tf(20.0),
                TextColor(palette.bevy_primary_text()),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));
            for (label, action) in [
                ("New World", MainMenuButtonAction::NewWorld),
                (
                    "Debug ① — Frame layout test",
                    MainMenuButtonAction::DebugFrameLayout,
                ),
                (
                    "Debug ③ — Demo world (stay open)",
                    MainMenuButtonAction::DebugDemoOpen,
                ),
                (
                    "Debug — Fire + weather + atmosphere (stay open)",
                    MainMenuButtonAction::DebugVfxSandbox,
                ),
                ("Load World", MainMenuButtonAction::OpenLoad),
                ("New map in editor", MainMenuButtonAction::NewMapEditor),
                ("Quit", MainMenuButtonAction::Quit),
            ] {
                parent
                    .spawn((
                        Button,
                        Node {
                            min_width: Val::Px(340.0),
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: sharp,
                            ..default()
                        },
                        BackgroundColor(palette.bevy_button_idle()),
                        BorderColor::all(palette.bevy_wire_magenta()),
                    ))
                    .insert(action)
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            tf(menu_pt),
                            TextColor(palette.bevy_primary_text()),
                        ));
                    });
            }
        });
}

fn spawn_load_menu(
    commands: &mut Commands,
    load_path: &LoadStubPath,
    palette: &UiPalette,
    cmd_mono: &CmdUiMonoFont,
) {
    let sharp = BorderRadius::ZERO;
    let path_display = load_path.0.clone();
    let menu_pt = 15.0;
    let tf = |pt: f32| TextFont::from_font_size(pt).with_font(cmd_mono.0.clone());

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
            BackgroundColor(palette.bevy_backdrop()),
            LoadMenuShellRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Load World (stub)"),
                tf(18.0),
                TextColor(palette.bevy_primary_text()),
            ));
            parent.spawn((
                Text::new(
                    "No file picker yet — path is developer-configurable via LoadStubPath resource.",
                ),
                tf(13.0),
                TextColor(palette.bevy_text_muted()),
                Node {
                    max_width: Val::Px(520.0),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new(format!("Path: {path_display}")),
                tf(13.0),
                TextColor(palette.bevy_secondary_text()),
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
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: sharp,
                            ..default()
                        },
                        BackgroundColor(palette.bevy_button_idle()),
                        BorderColor::all(palette.bevy_wire_magenta()),
                    ))
                    .insert(action)
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            tf(menu_pt),
                            TextColor(palette.bevy_primary_text()),
                        ));
                    });
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn begin_menu_debug_world_gen(
    commands: &mut Commands,
    world_roots: &Query<Entity, With<WorldMarker>>,
    bootstrap: &mut MenuDebugBootstrap,
    maneuver: DebugManeuver,
    test_scene: TestScene,
    world_size: u32,
) {
    // Arm first so `sync_legacy_to_ux` skips stale FullReady mirroring on the click frame.
    arm_debug_quick_world_gen(
        &mut bootstrap.debug_quick,
        &mut bootstrap.harness,
        maneuver,
        test_scene,
    );
    if world_gen_work_active(&bootstrap.job, &bootstrap.progress) {
        bootstrap.cancel_world_gen.write(CancelActiveWorldGenEvent);
    }
    despawn_generated_world_entities(commands, world_roots);
    bootstrap.harness.logistics_visual_seeded = false;
    NextState::set_if_neq(
        &mut *bootstrap.next_world_flow,
        WorldGenFlowState::Idle,
    );
    let mut p = WorldGenParams::default();
    p.width = world_size;
    p.height = world_size;
    *bootstrap.params = p;
    let size_label = format!("{}×{}", world_size, world_size);
    bootstrap.world_gen_ui.visible = true;
    bootstrap.layout_session.active = maneuver == DebugManeuver::FrameScreen;
    ux_begin_world_gen_from_menu(
        &mut bootstrap.next_app,
        &mut bootstrap.next_wg,
        &mut bootstrap.next_base,
        &mut bootstrap.next_world_flow,
        &mut bootstrap.chrome_latch,
        &mut bootstrap.world_gen_ui,
        &mut bootstrap.world_preview_ui,
    );
    info!(
        target: "app_shell",
        maneuver = maneuver.menu_label(),
        test_scene = test_scene.menu_label(),
        size = size_label,
        "Debug from menu: preview → full → enter sim (stay open). \
         Full capture auto-exit: `cargo run -p proc_A_dine01 -- --test visual`"
    );
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
    mut next_app: ResMut<NextState<crate::engine::AppState>>,
    mut next_wg: ResMut<NextState<crate::engine::WorldGenState>>,
    mut chrome_latch: ResMut<crate::engine::WorldGenChromeLatch>,
    mut world_gen_ui: ResMut<crate::gui::editor::world_gen_ui::WorldGenUiState>,
    mut world_preview_ui: ResMut<crate::gui::editor::world_preview::WorldPreviewUiState>,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            MainMenuButtonAction::NewWorld => {
                despawn_generated_world_entities(&mut commands, &world_roots);
                ux_begin_world_gen_from_menu(
                    &mut next_app,
                    &mut next_wg,
                    &mut next_base,
                    &mut next_world_flow,
                    &mut chrome_latch,
                    &mut world_gen_ui,
                    &mut world_preview_ui,
                );
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
            MainMenuButtonAction::DebugFrameLayout
            | MainMenuButtonAction::DebugDemoOpen
            | MainMenuButtonAction::DebugVfxSandbox => {}
        }
    }
}

fn handle_main_menu_debug_buttons(
    q: Query<
        (&Interaction, &MainMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    mut bootstrap: MenuDebugBootstrap,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            MainMenuButtonAction::DebugFrameLayout => {
                begin_menu_debug_world_gen(
                    &mut commands,
                    &world_roots,
                    &mut bootstrap,
                    DebugManeuver::FrameScreen,
                    TestScene::None,
                    192,
                );
            }
            MainMenuButtonAction::DebugDemoOpen => {
                begin_menu_debug_world_gen(
                    &mut commands,
                    &world_roots,
                    &mut bootstrap,
                    DebugManeuver::DemoOpen,
                    TestScene::None,
                    192,
                );
            }
            MainMenuButtonAction::DebugVfxSandbox => {
                begin_menu_debug_world_gen(
                    &mut commands,
                    &world_roots,
                    &mut bootstrap,
                    DebugManeuver::DemoOpen,
                    TestScene::VfxSandbox,
                    320,
                );
            }
            _ => {}
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
