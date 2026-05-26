//! Bevy UI pause menu — **UI-P5-PAUSE-001** + **UI-P5-DESIGN-001** polish.
//! Plan: `prompts/guides/ui/ui_phase5_pause_menu_plan_v1.md` · design: `src/dev/ui_p5_design_signoff_v1.md`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::engine::states::{BaseState, InGameMenuState, MainMenuState, WorldGenFlowState};
use crate::engine::{AppState, PauseState, WorldGenChromeLatch, WorldGenState};
use crate::gui::editor::world_gen_ui::{CancelActiveWorldGenEvent, WorldGenUiState};
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::pause_menu_confirm::{
    execute_pause_exit_to_main_menu, execute_pause_exit_to_world_gen, PauseMenuConfirm,
    PauseMenuPendingAction, world_gen_work_active,
};
use crate::gui::{CmdUiMonoFont, UiPalette};
use crate::terrain::generation::world_generator_enhanced::{
    WorldGenJobSlot, WorldGenProgress, WorldMarker,
};

#[derive(Component)]
pub struct PauseMenuShellRoot;

#[derive(Component)]
pub struct PauseMenuConfirmShellRoot;

#[derive(Component, Clone, Copy)]
enum PauseMenuButtonAction {
    Resume,
    Save,
    Load,
    WorldGenerator,
    MainMenu,
    Quit,
}

/// **UI-P5-DESIGN-001** — chrome tier for hover / primary / stub / destructive styling.
#[derive(Component, Clone, Copy)]
enum PauseMenuButtonTier {
    Primary,
    Normal,
    Stub,
    Destructive,
}

#[derive(Component)]
struct PauseMenuButtonLabel;

const PAUSE_CARD_MIN_W_PX: f32 = 320.0;
const PAUSE_MENU_BACKDROP_ALPHA: f32 = 0.88;

#[derive(Component, Clone, Copy)]
enum PauseMenuConfirmButtonAction {
    Cancel,
    Confirm,
}

#[derive(SystemParam)]
pub(crate) struct PauseMenuNavParams<'w, 's> {
    pending: ResMut<'w, PauseMenuPendingAction>,
    next_menu: ResMut<'w, NextState<InGameMenuState>>,
    next_base: ResMut<'w, NextState<BaseState>>,
    next_flow: ResMut<'w, NextState<WorldGenFlowState>>,
    next_main_menu: ResMut<'w, NextState<MainMenuState>>,
    next_app: ResMut<'w, NextState<AppState>>,
    next_wg: ResMut<'w, NextState<WorldGenState>>,
    next_pause: ResMut<'w, NextState<PauseState>>,
    commands: Commands<'w, 's>,
    world_roots: Query<'w, 's, Entity, With<WorldMarker>>,
    cancel_world_gen: MessageWriter<'w, CancelActiveWorldGenEvent>,
    app_exit: MessageWriter<'w, AppExit>,
}

#[derive(SystemParam)]
pub(crate) struct PauseMenuWorldGenParams<'w> {
    chrome_latch: ResMut<'w, WorldGenChromeLatch>,
    world_gen_ui: ResMut<'w, WorldGenUiState>,
    world_preview_ui: ResMut<'w, WorldPreviewUiState>,
    job_slot: Res<'w, WorldGenJobSlot>,
    progress: Res<'w, WorldGenProgress>,
}

/// Harness replay: mark pause menu Bevy path for live JSON (shell spawns when menu = Pause).
pub fn witness_pause_menu_bevy_replay(
    witness: &mut crate::gui::hud::simulation_shell_phase2::UiShellMigrationWitness,
) {
    witness.pause_menu_bevy = true;
}

fn despawn_pause_shells(commands: &mut Commands, roots: impl Iterator<Item = Entity>) {
    for e in roots {
        commands.entity(e).despawn();
    }
}

fn pause_backdrop_color(palette: &UiPalette) -> Color {
    palette.bevy_backdrop().with_alpha(PAUSE_MENU_BACKDROP_ALPHA)
}

fn spawn_pause_divider(parent: &mut ChildSpawnerCommands, palette: &UiPalette) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(palette.bevy_wire_magenta().with_alpha(0.55)),
    ));
}

fn spawn_pause_card(
    parent: &mut ChildSpawnerCommands,
    palette: &UiPalette,
    tf: &impl Fn(f32) -> TextFont,
    menu_pt: f32,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(20.0)),
                min_width: Val::Px(PAUSE_CARD_MIN_W_PX),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new("SIMULATION PAUSED"),
                tf(16.0),
                TextColor(palette.bevy_primary_text()),
            ));
            card.spawn((
                Text::new("Menu pause — sim tick (P) is separate."),
                tf(11.0),
                TextColor(palette.bevy_text_muted()),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));
            spawn_pause_divider(card, palette);
            spawn_pause_button(
                card,
                palette,
                tf,
                menu_pt,
                "Resume",
                PauseMenuButtonAction::Resume,
                PauseMenuButtonTier::Primary,
            );
            spawn_pause_button(
                card,
                palette,
                tf,
                menu_pt,
                "Save game (stub)",
                PauseMenuButtonAction::Save,
                PauseMenuButtonTier::Stub,
            );
            spawn_pause_button(
                card,
                palette,
                tf,
                menu_pt,
                "Load game",
                PauseMenuButtonAction::Load,
                PauseMenuButtonTier::Normal,
            );
            spawn_pause_divider(card, palette);
            spawn_pause_button(
                card,
                palette,
                tf,
                menu_pt,
                "World Generator…",
                PauseMenuButtonAction::WorldGenerator,
                PauseMenuButtonTier::Normal,
            );
            spawn_pause_button(
                card,
                palette,
                tf,
                menu_pt,
                "Return to Main Menu",
                PauseMenuButtonAction::MainMenu,
                PauseMenuButtonTier::Destructive,
            );
            spawn_pause_button(
                card,
                palette,
                tf,
                menu_pt,
                "Exit program",
                PauseMenuButtonAction::Quit,
                PauseMenuButtonTier::Destructive,
            );
            card.spawn((
                Text::new("Esc — resume (when tray collapsed)"),
                tf(10.0),
                TextColor(palette.bevy_text_muted()),
                Node {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
            ));
        });
}

fn pause_button_border(_palette: &UiPalette, tier: PauseMenuButtonTier) -> UiRect {
    match tier {
        PauseMenuButtonTier::Primary => UiRect {
            left: Val::Px(2.0),
            top: Val::Px(1.0),
            right: Val::Px(1.0),
            bottom: Val::Px(1.0),
        },
        _ => UiRect::all(Val::Px(1.0)),
    }
}

fn pause_button_idle_colors(
    palette: &UiPalette,
    tier: PauseMenuButtonTier,
) -> (BackgroundColor, BorderColor) {
    match tier {
        PauseMenuButtonTier::Primary => (
            BackgroundColor(palette.bevy_bg_vellum()),
            BorderColor {
                left: palette.bevy_accent_gold(),
                top: palette.bevy_wire_magenta(),
                right: palette.bevy_wire_magenta(),
                bottom: palette.bevy_wire_magenta(),
            },
        ),
        PauseMenuButtonTier::Destructive => (
            BackgroundColor(palette.bevy_button_idle()),
            BorderColor::all(palette.bevy_wire_magenta()),
        ),
        PauseMenuButtonTier::Stub | PauseMenuButtonTier::Normal => (
            BackgroundColor(palette.bevy_button_idle()),
            BorderColor::all(palette.bevy_border_subtle()),
        ),
    }
}

fn spawn_pause_button(
    parent: &mut ChildSpawnerCommands,
    palette: &UiPalette,
    tf: &impl Fn(f32) -> TextFont,
    menu_pt: f32,
    label: &str,
    action: PauseMenuButtonAction,
    tier: PauseMenuButtonTier,
) {
    let (bg, border) = pause_button_idle_colors(palette, tier);
    let text_color = match tier {
        PauseMenuButtonTier::Stub => palette.bevy_text_muted(),
        _ => palette.bevy_primary_text(),
    };
    parent
        .spawn((
            Button,
            Node {
                min_height: Val::Px(36.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: pause_button_border(palette, tier),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            bg,
            border,
            tier,
            action,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                tf(menu_pt),
                TextColor(text_color),
                PauseMenuButtonLabel,
            ));
        });
}

fn pause_menu_button_border_color(
    palette: &UiPalette,
    tier: PauseMenuButtonTier,
    interaction: &Interaction,
) -> BorderColor {
    if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
        return BorderColor::all(palette.bevy_accent_hot());
    }
    match tier {
        PauseMenuButtonTier::Primary => BorderColor {
            left: palette.bevy_accent_gold(),
            top: palette.bevy_wire_magenta(),
            right: palette.bevy_wire_magenta(),
            bottom: palette.bevy_wire_magenta(),
        },
        PauseMenuButtonTier::Destructive => BorderColor::all(palette.bevy_wire_magenta()),
        PauseMenuButtonTier::Stub | PauseMenuButtonTier::Normal => {
            BorderColor::all(palette.bevy_border_subtle())
        }
    }
}

fn sync_pause_menu_button_chrome_system(
    palette: Res<UiPalette>,
    mut buttons: Query<
        (
            &Interaction,
            &PauseMenuButtonTier,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, With<PauseMenuButtonAction>),
    >,
    mut labels: Query<(&ChildOf, &mut TextColor), With<PauseMenuButtonLabel>>,
    parents: Query<(Entity, &PauseMenuButtonTier, &Interaction), With<Button>>,
) {
    for (interaction, tier, mut bg, mut border) in &mut buttons {
        *border = pause_menu_button_border_color(palette.as_ref(), *tier, interaction);
        *bg = match (*tier, interaction) {
            (PauseMenuButtonTier::Primary, Interaction::Hovered | Interaction::Pressed) => {
                BackgroundColor(palette.bevy_bg_vellum())
            }
            (PauseMenuButtonTier::Primary, _) => BackgroundColor(palette.bevy_bg_vellum()),
            (_, Interaction::Hovered | Interaction::Pressed) => {
                BackgroundColor(palette.bevy_hud_panel_fill())
            }
            _ => BackgroundColor(palette.bevy_button_idle()),
        };
    }
    for (parent, mut text_color) in &mut labels {
        let Ok((_, tier, interaction)) = parents.get(parent.parent()) else {
            continue;
        };
        *text_color = match tier {
            PauseMenuButtonTier::Stub => TextColor(palette.bevy_text_muted()),
            _ if *interaction == Interaction::Hovered => TextColor(palette.bevy_accent_hot()),
            _ => TextColor(palette.bevy_primary_text()),
        };
    }
}

fn spawn_pause_menu_shell(
    commands: &mut Commands,
    palette: &UiPalette,
    cmd_mono: &CmdUiMonoFont,
) {
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
                ..default()
            },
            BackgroundColor(pause_backdrop_color(palette)),
            ZIndex(2000),
            FocusPolicy::Block,
            PauseMenuShellRoot,
        ))
        .with_children(|parent| {
            spawn_pause_card(parent, palette, &tf, menu_pt);
        });
}

fn spawn_pause_confirm_shell(
    commands: &mut Commands,
    palette: &UiPalette,
    cmd_mono: &CmdUiMonoFont,
    pending: PauseMenuConfirm,
) {
    let (title, body, confirm_label) = match pending {
        PauseMenuConfirm::ExitToWorldGen => (
            "Leave current world?",
            "Opening World Generator will exit the current simulation world. Unsaved progress may be lost.",
            "Exit to World Generator",
        ),
        PauseMenuConfirm::ExitToMainMenu => (
            "Return to main menu?",
            "Return to the main menu and exit the current simulation world. Unsaved progress may be lost.",
            "Return to Main Menu",
        ),
        PauseMenuConfirm::None => return,
    };
    let menu_pt = 14.0;
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
                ..default()
            },
            BackgroundColor(pause_backdrop_color(palette)),
            ZIndex(2010),
            FocusPolicy::Block,
            PauseMenuConfirmShellRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(18.0)),
                        min_width: Val::Px(360.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::ZERO,
                        ..default()
                    },
                    BackgroundColor(palette.bevy_hud_panel_fill()),
                    BorderColor::all(palette.bevy_wire_magenta()),
                ))
                .with_children(|card| {
                    card.spawn((
                        Text::new(title),
                        tf(17.0),
                        TextColor(palette.bevy_primary_text()),
                    ));
                    card.spawn((
                        Text::new(body),
                        tf(12.0),
                        TextColor(palette.bevy_text_muted()),
                    ));
                    card.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_confirm_button(
                            row,
                            palette,
                            &tf,
                            menu_pt,
                            "Cancel",
                            PauseMenuConfirmButtonAction::Cancel,
                            false,
                        );
                        spawn_confirm_button(
                            row,
                            palette,
                            &tf,
                            menu_pt,
                            confirm_label,
                            PauseMenuConfirmButtonAction::Confirm,
                            true,
                        );
                    });
                });
        });
}

fn spawn_confirm_button(
    parent: &mut ChildSpawnerCommands,
    palette: &UiPalette,
    tf: &impl Fn(f32) -> TextFont,
    menu_pt: f32,
    label: &str,
    action: PauseMenuConfirmButtonAction,
    emphasize: bool,
) {
    let border = if emphasize {
        BorderColor::all(palette.bevy_accent_hot())
    } else {
        BorderColor::all(palette.bevy_wire_magenta())
    };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            BackgroundColor(if emphasize {
                palette.bevy_bg_vellum()
            } else {
                palette.bevy_button_idle()
            }),
            border,
            action,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                tf(menu_pt),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

fn sync_pause_menu_bevy_shell(
    base: Res<State<BaseState>>,
    menu: Res<State<InGameMenuState>>,
    pending: ResMut<PauseMenuPendingAction>,
    mut commands: Commands,
    shell_q: Query<Entity, With<PauseMenuShellRoot>>,
    confirm_q: Query<Entity, With<PauseMenuConfirmShellRoot>>,
    palette: Res<UiPalette>,
    cmd_mono: Res<CmdUiMonoFont>,
    mut witness: Option<ResMut<crate::gui::hud::simulation_shell_phase2::UiShellMigrationWitness>>,
) {
    if *base.get() != BaseState::Simulation || *menu.get() != InGameMenuState::Pause {
        despawn_pause_shells(&mut commands, shell_q.iter().chain(confirm_q.iter()));
        return;
    }

    if pending.is_pending() {
        despawn_pause_shells(&mut commands, shell_q.iter());
        if confirm_q.is_empty() {
            spawn_pause_confirm_shell(
                &mut commands,
                palette.as_ref(),
                cmd_mono.as_ref(),
                pending.confirm,
            );
        }
    } else {
        despawn_pause_shells(&mut commands, confirm_q.iter());
        if shell_q.is_empty() {
            spawn_pause_menu_shell(&mut commands, palette.as_ref(), cmd_mono.as_ref());
        }
    }

    if shell_q.iter().next().is_some() || confirm_q.iter().next().is_some() {
        if let Some(w) = witness.as_mut() {
            w.pause_menu_bevy = true;
        }
    }
}

fn handle_pause_menu_buttons(
    q: Query<
        (&Interaction, &PauseMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut nav: PauseMenuNavParams,
    mut wg: PauseMenuWorldGenParams,
) {
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            PauseMenuButtonAction::Resume => {
                NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
            }
            PauseMenuButtonAction::Save => {
                info!("Pause menu: Save — wire to WorldSaveSpine when ready.");
            }
            PauseMenuButtonAction::Load => {
                NextState::set_if_neq(&mut *nav.next_flow, WorldGenFlowState::LoadingSave);
                NextState::set_if_neq(&mut *nav.next_main_menu, MainMenuState::Load);
                NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
            }
            PauseMenuButtonAction::WorldGenerator => {
                nav.pending.confirm = PauseMenuConfirm::ExitToWorldGen;
            }
            PauseMenuButtonAction::MainMenu => {
                if world_gen_work_active(&wg.job_slot, &wg.progress) {
                    nav.pending.confirm = PauseMenuConfirm::ExitToMainMenu;
                } else {
                    execute_pause_exit_to_main_menu(
                        &mut nav.commands,
                        &nav.world_roots,
                        &mut wg.chrome_latch,
                        &mut nav.next_base,
                        &mut nav.next_flow,
                        &mut nav.next_main_menu,
                        &mut nav.next_app,
                        &mut nav.next_wg,
                        &mut nav.next_pause,
                        &mut nav.cancel_world_gen,
                        &wg.job_slot,
                        &wg.progress,
                    );
                    NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                }
            }
            PauseMenuButtonAction::Quit => {
                nav.app_exit.write(AppExit::Success);
            }
        }
    }
}

fn handle_pause_confirm_buttons(
    q: Query<
        (&Interaction, &PauseMenuConfirmButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut nav: PauseMenuNavParams,
    mut wg: PauseMenuWorldGenParams,
) {
    let confirm_kind = nav.pending.confirm;
    for (interaction, action) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            PauseMenuConfirmButtonAction::Cancel => {
                nav.pending.clear();
            }
            PauseMenuConfirmButtonAction::Confirm => match confirm_kind {
                PauseMenuConfirm::ExitToWorldGen => {
                    execute_pause_exit_to_world_gen(
                        &mut nav.commands,
                        &nav.world_roots,
                        &mut wg.chrome_latch,
                        &mut nav.next_base,
                        &mut nav.next_flow,
                        &mut nav.next_app,
                        &mut nav.next_wg,
                        &mut nav.next_pause,
                        &mut wg.world_gen_ui,
                        &mut wg.world_preview_ui,
                        &mut nav.cancel_world_gen,
                        &wg.job_slot,
                        &wg.progress,
                    );
                    nav.pending.clear();
                    NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                }
                PauseMenuConfirm::ExitToMainMenu => {
                    execute_pause_exit_to_main_menu(
                        &mut nav.commands,
                        &nav.world_roots,
                        &mut wg.chrome_latch,
                        &mut nav.next_base,
                        &mut nav.next_flow,
                        &mut nav.next_main_menu,
                        &mut nav.next_app,
                        &mut nav.next_wg,
                        &mut nav.next_pause,
                        &mut nav.cancel_world_gen,
                        &wg.job_slot,
                        &wg.progress,
                    );
                    nav.pending.clear();
                    NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                }
                PauseMenuConfirm::None => {}
            },
        }
    }
}

pub struct PauseMenuBevyPlugin;

impl Plugin for PauseMenuBevyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_pause_menu_bevy_shell,
                sync_pause_menu_button_chrome_system,
                handle_pause_menu_buttons,
                handle_pause_confirm_buttons,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::hud::simulation_shell_phase2::{
        build_proof_payload, commit_ui_shell_migration_live_proof, ContextTrayState,
        UiShellMigrationWitness,
    };
    use crate::gui::hud::shell_diagnostics::ProductShellDiagnostics;

    #[test]
    fn ui_p5_pause_001_witness_green_when_bevy_flag_set() {
        let mut witness = UiShellMigrationWitness::default();
        witness_pause_menu_bevy_replay(&mut witness);
        assert!(crate::gui::hud::simulation_shell_phase2::ui_p5_pause_001_green(&witness));
        let body = build_proof_payload(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        );
        assert_eq!(body["phase5"]["pause_menu_bevy"], serde_json::json!(true));
        assert_eq!(body["ui_p5_pause_001_green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_p5_001"]["green"], serde_json::json!(true));
        assert!(commit_ui_shell_migration_live_proof(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        ));
    }
}
