// Gameplay HUD — **Bevy UI only** (logistics summary + command/alerts placeholders).
// Colors / chrome: [`crate::gui::UiPalette`] — `prompts/guides/ui_design_language_plan_v1.md` §5.
// Dev tools (diagnostics, world gen, etc.) live in **egui** behind shortcuts; see `ui_boundary_guide_v1.md`.
// G3B: prompts/matrix/gap_remediation/runbook/g3b_production_hud_steps_v1.md

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::engine::BaseState;
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::strategic::{
    ActiveMissions, CityPlanningHints, FractureProbabilityOverlay, LogisticsAiRuntime,
    OperationalTheaterSummary, StrategicOverlayDisplayPolicy, MAX_STRATEGIC_FACTION_SLOTS,
};
use crate::gui::build::{BuildOverlayVisibility, BuildStripState};
use crate::gui::ui_gates::in_simulation_or_editor;

use crate::entities::production::core::{
    resolve_logistics_focus_entity, storage_entities_for_focus, LogisticsSiteMember,
    LogisticsSiteRoot, ResourceConsumer, ResourceProducer, ResourceStorage,
    ResourceStorageCapacity, ResourceType,
};

use super::input_bindings::InputBindings;
use super::logistics_focus::{HudAggregateSettings, HudLogisticsFocus};
use super::{CmdUiMonoFont, UiPalette};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
struct LogisticsPickHookAttached;

#[derive(Component)]
struct StrategicOpsHudLine;

/// Top bar — `base_ui_direction_principls.md` “operational command table”.
#[derive(Component)]
pub struct OperationsStripRoot;

#[derive(Component)]
struct OpsStripTime;

#[derive(Component)]
struct OpsStripAlerts;

#[derive(Component)]
struct OpsStripIntelRoutes;

#[derive(Component)]
struct OpsStripBuild;

#[derive(Component)]
struct OpsStripIntelEw;

/// **Simulation** Bevy UI chrome: operations strip + left context column (`base_ui_direction_principls.md`).
#[derive(Component)]
pub struct SimulationCommandShellRoot;

#[derive(Component)]
struct LeftContextRail;

#[derive(Component)]
struct LeftContextStackBody;

#[derive(Component)]
struct ObjectivesHudLine;

const LEFT_CONTEXT_RAIL_W_PX: f32 = 36.0;

/// Collapse state for the left command stack (`toggle_command_left_stack` / rail button).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct CommandLeftStackState {
    pub collapsed: bool,
}
const OPS_STRIP_H_PX: f32 = 38.0;
const OPS_STRIP_MONO_PT: f32 = 13.0;
const HUD_MONO_PT: f32 = 13.5;

/// When **compact**, the strategic HUD shows a one-line summary; full line includes city-planning hints.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct StrategicHudStripState {
    pub compact: bool,
}

pub struct InGameHudPlugin;

impl Plugin for InGameHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudLogisticsFocus>()
            .init_resource::<HudAggregateSettings>()
            .init_resource::<StrategicHudStripState>()
            .init_resource::<CommandLeftStackState>()
            .add_systems(OnEnter(BaseState::Simulation), spawn_simulation_command_shell)
            .add_systems(
                OnExit(BaseState::Simulation),
                despawn_simulation_command_shell,
            )
            .add_systems(
                Update,
                (
                    strategic_hud_chrome_input,
                    sync_command_left_stack_visibility,
                    command_left_stack_rail_interaction,
                    attach_storage_picking_hooks,
                    cycle_logistics_focus_dev,
                    update_operations_strip,
                    update_objectives_hud_line,
                    update_site_logistics_hud,
                    update_strategic_ops_hud,
                )
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn despawn_simulation_command_shell(
    mut commands: Commands,
    roots: Query<Entity, With<SimulationCommandShellRoot>>,
) {
    for e in &roots {
        commands.entity(e).try_despawn();
    }
}

fn spawn_simulation_command_shell(
    mut commands: Commands,
    bindings: Res<InputBindings>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    existing: Query<Entity, With<SimulationCommandShellRoot>>,
) {
    if !existing.is_empty() {
        return;
    }

    let font = mono.0.clone();
    let fs = OPS_STRIP_MONO_PT;
    let tf_hud = |size: f32| TextFont::from_font_size(size).with_font(font.clone());

    let tools = format!(
        "Tools — Options/keys {} · Diagnostics {} · Pressure {} · Faction {} · Logistics list {} · Cycle focus {} · World gen {} · Agent perms {} · Scenario (editor) {} · Collapse left stack {}.",
        InputBindings::format_key(bindings.toggle_keybindings_options),
        InputBindings::format_key(bindings.toggle_diagnostics),
        InputBindings::format_key(bindings.toggle_pressure_composer),
        InputBindings::format_key(bindings.toggle_faction_tools),
        InputBindings::format_key(bindings.toggle_logistics_targets_panel),
        InputBindings::format_key(bindings.cycle_logistics_focus),
        InputBindings::format_key(bindings.toggle_world_generator),
        InputBindings::format_key(bindings.toggle_agent_permissions),
        InputBindings::format_key(bindings.toggle_scenario_script_panel),
        InputBindings::format_key(bindings.toggle_command_left_stack),
    );

    let hint = format!(
        "Logistics — select storage ({}) · pressure ({}) · Strategic compact ({}) · overlays ({}/{}) · build cycle ({}) · left context toggle ({})",
        InputBindings::format_key(bindings.cycle_logistics_focus),
        InputBindings::format_key(bindings.toggle_pressure_composer),
        InputBindings::format_key(bindings.toggle_strategic_hud_strip_compact),
        InputBindings::format_key(bindings.toggle_strategic_overlay_routing_congestion),
        InputBindings::format_key(bindings.toggle_strategic_overlay_ew_denial),
        InputBindings::format_key(bindings.cycle_build_planning_tool),
        InputBindings::format_key(bindings.toggle_command_left_stack),
    );

    let construction = "Construction — map buildings/rails in the Editor (terrain/road tools + bake). In-sim **planning strip**: cycle build mode with `;` (Options → Key bindings); ops strip shows BUILD line; ghost placement / commit wiring continues in P2-F.";

    let strip_border = BorderColor {
        left: Color::NONE,
        top: Color::NONE,
        right: Color::NONE,
        bottom: palette.bevy_wire_magenta(),
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                ..default()
            },
            FocusPolicy::Pass,
            Pickable::IGNORE,
            ZIndex(750),
            SimulationCommandShellRoot,
        ))
        .with_children(|shell| {
            shell
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(OPS_STRIP_H_PX),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(14.0),
                        border: UiRect::bottom(Val::Px(1.0)),
                        border_radius: BorderRadius::ZERO,
                        ..default()
                    },
                    BackgroundColor(palette.bevy_hud_panel_fill()),
                    strip_border,
                    ZIndex(1200),
                    OperationsStripRoot,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("> TIME  …"),
                        TextFont::from_font_size(fs).with_font(font.clone()),
                        TextColor(palette.bevy_primary_text()),
                        OpsStripTime,
                    ));
                    parent
                        .spawn((
                            Node {
                                flex_grow: 1.0,
                                min_width: Val::Px(120.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_children(|c| {
                            c.spawn((
                                Text::new("ALERTS  —"),
                                TextFont::from_font_size(fs).with_font(font.clone()),
                                TextColor(palette.bevy_text_muted()),
                                OpsStripAlerts,
                            ));
                        });
                    parent.spawn((
                        Text::new("ROUTES —"),
                        TextFont::from_font_size(fs).with_font(font.clone()),
                        TextColor(palette.bevy_secondary_text()),
                        OpsStripIntelRoutes,
                    ));
                    parent.spawn((
                        Text::new("BUILD —"),
                        TextFont::from_font_size(fs).with_font(font.clone()),
                        TextColor(palette.bevy_secondary_text()),
                        OpsStripBuild,
                    ));
                    parent.spawn((
                        Text::new("EW/DENY —"),
                        TextFont::from_font_size(fs).with_font(font.clone()),
                        TextColor(palette.bevy_secondary_text()),
                        OpsStripIntelEw,
                    ));
                });

            shell
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        min_height: Val::Px(0.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::new(
                            Val::Px(8.0),
                            Val::Px(6.0),
                            Val::Px(8.0),
                            Val::Px(8.0),
                        ),
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                    FocusPolicy::Pass,
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::FlexStart,
                            column_gap: Val::Px(6.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                        FocusPolicy::Pass,
                    ))
                    .with_children(|left_pack| {
                        left_pack
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(LEFT_CONTEXT_RAIL_W_PX),
                                    min_height: Val::Px(120.0),
                                    padding: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::ZERO,
                                    ..default()
                                },
                                BackgroundColor(palette.bevy_hud_panel_fill()),
                                BorderColor::all(palette.bevy_wire_magenta()),
                                Visibility::Hidden,
                                LeftContextRail,
                                ZIndex(850),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("«\nH\nU\nD"),
                                    tf_hud(11.0),
                                    TextColor(palette.bevy_accent_terminal()),
                                ));
                            });

                        left_pack
                            .spawn((
                                Node {
                                    padding: UiRect::all(Val::Px(10.0)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(8.0),
                                    max_width: Val::Px(520.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::ZERO,
                                    ..default()
                                },
                                BackgroundColor(palette.bevy_hud_panel_fill()),
                                BorderColor::all(palette.bevy_wire_magenta()),
                                Visibility::Visible,
                                ZIndex(800),
                                LeftContextStackBody,
                                HudRoot,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new("> Alerts & objectives — …"),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_accent_terminal()),
                                    ObjectivesHudLine,
                                ));
                                parent.spawn((
                                    Text::new("Strategic — theater / logistics AI (initializing…)"),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_secondary_text()),
                                    StrategicOpsHudLine,
                                ));
                                parent.spawn((
                                    Text::new(tools),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_text_muted()),
                                ));
                                parent.spawn((
                                    Text::new(hint),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_secondary_text()),
                                ));
                                parent.spawn((
                                    Text::new("Site logistics — …"),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_primary_text()),
                                    ResourceDisplay,
                                ));
                                parent.spawn((
                                    Text::new(construction),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_text_muted()),
                                ));
                                parent.spawn((
                                    Text::new("Threat & drone-adjacent readouts: see Diagnostics theater block and the strategic summary line below (not a separate F-key panel yet)."),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_text_muted()),
                                ));
                                parent.spawn((
                                    Text::new("World gen / Editor: F8 opens generator; map editor palettes are in-editor egui."),
                                    tf_hud(HUD_MONO_PT),
                                    TextColor(palette.bevy_text_muted()),
                                ));
                            });
                    });

                    row.spawn((
                        Node {
                            flex_grow: 1.0,
                            min_width: Val::Px(0.0),
                            min_height: Val::Px(1.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                        FocusPolicy::Pass,
                    ));
                });
        });
}

fn update_operations_strip(
    tick: Res<SimTick>,
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    policy: Res<StrategicOverlayDisplayPolicy>,
    theater: Res<OperationalTheaterSummary>,
    logistics: Res<LogisticsAiRuntime>,
    missions: Res<ActiveMissions>,
    fracture: Res<FractureProbabilityOverlay>,
    build_strip: Res<BuildStripState>,
    build_overlays: Res<BuildOverlayVisibility>,
    mut q_time: Query<&mut Text, With<OpsStripTime>>,
    mut q_alerts: Query<&mut Text, With<OpsStripAlerts>>,
    mut q_routes: Query<&mut Text, With<OpsStripIntelRoutes>>,
    mut q_build: Query<&mut Text, With<OpsStripBuild>>,
    mut q_ew: Query<&mut Text, With<OpsStripIntelEw>>,
) {
    let run = if ctrl.paused { "PAUSE" } else { "RUN" };
    let time_line = format!(
        "> TIME  wall {:>6.1}s  │  SIM n={}  {:<5}  v={:.1}x",
        time.elapsed_secs(),
        tick.0,
        run,
        ctrl.speed
    );
    for mut t in &mut q_time {
        *t = Text::new(time_line.clone());
    }

    let n_m = missions.missions.len();
    let m0 = missions.missions.first();
    let mission_hint = m0
        .and_then(|m| {
            m.success_readout_label
                .as_deref()
                .or(m.objectives.first().map(|o| o.label.as_str()))
        })
        .unwrap_or("—");
    let alerts_line = format!(
        "ALERTS  msn {} | T0 {:.2} altμ {:.2} fac {} | {}",
        n_m,
        theater.mean_threat_by_slot[0],
        alt_mean_threat(&theater),
        theater.active_faction_slots,
        truncate_slot(mission_hint, 28),
    );
    for mut t in &mut q_alerts {
        *t = Text::new(alerts_line.clone());
    }

    let routes = format!(
        "ROUTES  layer {}  proxy {:.2}  edgeμ {:.2}  stock {:.2}",
        if policy.apply_routing_congestion { "on" } else { "off" },
        logistics.congestion_proxy,
        logistics.mean_edge_damage,
        logistics.stockpile_fill_ratio,
    );
    for mut t in &mut q_routes {
        *t = Text::new(routes.clone());
    }

    let build = format!(
        "BUILD  mode {}  (terrain {}  net {}  cost {})",
        build_strip.active.label(),
        if build_overlays.terrain { "on" } else { "off" },
        if build_overlays.network { "on" } else { "off" },
        if build_overlays.cost { "on" } else { "off" },
    );
    for mut t in &mut q_build {
        *t = Text::new(build.clone());
    }

    let ew = format!(
        "EW/DENY  layer {}  fract m {:.2}  ind prox {:.2}",
        if policy.apply_ew_denial { "on" } else { "off" },
        fracture.mean_heuristic,
        logistics.industrial_output_proxy,
    );
    for mut t in &mut q_ew {
        *t = Text::new(ew.clone());
    }
}

#[inline]
fn alt_mean_threat(theater: &OperationalTheaterSummary) -> f32 {
    let mut t_alt = 0.0f32;
    let mut n_alt = 0.0f32;
    for i in 1..MAX_STRATEGIC_FACTION_SLOTS {
        let a = theater.mean_threat_by_slot[i];
        if a > 1e-4 || theater.mean_logistics_strength_by_slot[i] > 1e-4 {
            t_alt += a;
            n_alt += 1.0;
        }
    }
    if n_alt > 0.0 { t_alt / n_alt } else { 0.0 }
}

fn truncate_slot(s: &str, max_chars: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max_chars {
        return t.to_string();
    }
    format!("{}…", t.chars().take(max_chars.saturating_sub(1)).collect::<String>())
}

fn update_objectives_hud_line(
    missions: Res<ActiveMissions>,
    theater: Res<OperationalTheaterSummary>,
    mut q: Query<&mut Text, With<ObjectivesHudLine>>,
) {
    let n = missions.missions.len();
    let focus = missions
        .missions
        .first()
        .and_then(|m| {
            m.success_readout_label.clone().or_else(|| {
                m.objectives
                    .first()
                    .map(|o| o.label.clone())
                    .filter(|s| !s.is_empty())
            })
        })
        .unwrap_or_else(|| "—".into());
    let line = format!(
        "> Objectives — msn {} | {} | T0 {:.2} (fac {})",
        n,
        truncate_slot(&focus, 36),
        theater.mean_threat_by_slot[0],
        theater.active_faction_slots,
    );
    for mut t in &mut q {
        *t = Text::new(line.clone());
    }
}

fn sync_command_left_stack_visibility(
    state: Res<CommandLeftStackState>,
    mut body: Query<&mut Visibility, With<LeftContextStackBody>>,
    mut rail: Query<&mut Visibility, With<LeftContextRail>>,
) {
    for mut v in &mut body {
        *v = if state.collapsed {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    for mut v in &mut rail {
        *v = if state.collapsed {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn command_left_stack_rail_interaction(
    q: Query<&Interaction, (Changed<Interaction>, With<LeftContextRail>)>,
    mut state: ResMut<CommandLeftStackState>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            state.collapsed = false;
        }
    }
}

fn strategic_hud_chrome_input(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut policy: ResMut<StrategicOverlayDisplayPolicy>,
    mut strip: ResMut<StrategicHudStripState>,
    mut left_stack: ResMut<CommandLeftStackState>,
) {
    if keys.just_pressed(bindings.toggle_command_left_stack) {
        left_stack.collapsed = !left_stack.collapsed;
    }
    if keys.just_pressed(bindings.toggle_strategic_hud_strip_compact) {
        strip.compact = !strip.compact;
    }
    if keys.just_pressed(bindings.toggle_strategic_overlay_routing_congestion) {
        policy.apply_routing_congestion = !policy.apply_routing_congestion;
    }
    if keys.just_pressed(bindings.toggle_strategic_overlay_ew_denial) {
        policy.apply_ew_denial = !policy.apply_ew_denial;
    }
}

fn attach_storage_picking_hooks(
    mut commands: Commands,
    q: Query<Entity, (With<ResourceStorage>, Without<LogisticsPickHookAttached>)>,
) {
    for e in q.iter() {
        commands
            .entity(e)
            .insert((Pickable::default(), LogisticsPickHookAttached))
            .observe(on_logistics_storage_clicked);
    }
}

fn on_logistics_storage_clicked(
    mut click: On<Pointer<Click>>,
    mut focus: ResMut<HudLogisticsFocus>,
    roots: Query<(), With<LogisticsSiteRoot>>,
    members: Query<&LogisticsSiteMember>,
) {
    if click.event().event.button != PointerButton::Primary {
        return;
    }
    let entity = click.event().entity;
    let is_hub = roots.get(entity).is_ok();
    let member_of = members.get(entity).ok();
    let resolved = resolve_logistics_focus_entity(entity, member_of, is_hub);
    focus.tracked_entity = Some(resolved);
    click.propagate(false);
}

fn cycle_logistics_focus_dev(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut focus: ResMut<HudLogisticsFocus>,
    roots: Query<(), With<LogisticsSiteRoot>>,
    members: Query<&LogisticsSiteMember>,
    with_storage: Query<Entity, With<ResourceStorage>>,
) {
    if !keys.just_pressed(bindings.cycle_logistics_focus) {
        return;
    }
    let list: Vec<Entity> = with_storage.iter().collect();
    if list.is_empty() {
        focus.tracked_entity = None;
        return;
    }
    let next_raw = match focus.tracked_entity {
        Some(cur) => match list.iter().position(|e| *e == cur) {
            Some(i) => list[(i + 1) % list.len()],
            None => list[0],
        },
        None => list[0],
    };
    let is_hub = roots.get(next_raw).is_ok();
    let m = members.get(next_raw).ok();
    focus.tracked_entity = Some(resolve_logistics_focus_entity(next_raw, m, is_hub));
}

fn merge_amounts_and_caps(
    entities: &[Entity],
    storage_q: &Query<&ResourceStorage>,
    cap_q: &Query<&ResourceStorageCapacity>,
) -> (HashMap<ResourceType, f32>, HashMap<ResourceType, f32>) {
    let mut amounts: HashMap<ResourceType, f32> = HashMap::new();
    let mut caps: HashMap<ResourceType, f32> = HashMap::new();
    for &e in entities {
        if let Ok(s) = storage_q.get(e) {
            for (&ty, &amt) in &s.amounts {
                *amounts.entry(ty).or_insert(0.0) += amt;
            }
        }
        if let Ok(c) = cap_q.get(e) {
            for (&ty, &mx) in &c.max_amounts {
                if mx > 0.0 {
                    *caps.entry(ty).or_insert(0.0) += mx;
                }
            }
        }
    }
    (amounts, caps)
}

fn update_strategic_ops_hud(
    theater: Res<OperationalTheaterSummary>,
    logistics: Res<LogisticsAiRuntime>,
    city: Res<CityPlanningHints>,
    policy: Res<StrategicOverlayDisplayPolicy>,
    strip: Res<StrategicHudStripState>,
    mut text_q: Query<&mut Text, With<StrategicOpsHudLine>>,
) {
    let mut t_alt = 0.0f32;
    let mut n_alt = 0.0f32;
    for i in 1..MAX_STRATEGIC_FACTION_SLOTS {
        let a = theater.mean_threat_by_slot[i];
        if a > 1e-4 || theater.mean_logistics_strength_by_slot[i] > 1e-4 {
            t_alt += a;
            n_alt += 1.0;
        }
    }
    let alt_t = if n_alt > 0.0 { t_alt / n_alt } else { 0.0 };
    let layers = format!(
        "layers C:{} E:{}",
        if policy.apply_routing_congestion { "on" } else { "off" },
        if policy.apply_ew_denial { "on" } else { "off" },
    );
    let line = if strip.compact {
        format!(
            "Strategic — compact | T {:.2} L {:.2} | congest {:.2} dmg {:.2} | {}",
            theater.mean_threat_by_slot[0],
            theater.mean_logistics_strength_by_slot[0],
            logistics.congestion_proxy,
            logistics.mean_edge_damage,
            layers,
        )
    } else {
        format!(
            "Strategic — threat {:.2} / logi {:.2} (alt μT {:.2}) | congest {:.2} edge dmg {:.2} stock {:.2} industry {:.2} manifest {:.2} | site {:.2} util {:.2} rebuild {:.2} | {}",
            theater.mean_threat_by_slot[0],
            theater.mean_logistics_strength_by_slot[0],
            alt_t,
            logistics.congestion_proxy,
            logistics.mean_edge_damage,
            logistics.stockpile_fill_ratio,
            logistics.industrial_output_proxy,
            logistics.production_domain_proxy,
            city.last_best_site_score,
            city.utility_redundancy_hint,
            city.adaptive_rebuild_pressure,
            layers,
        )
    };
    for mut text in text_q.iter_mut() {
        *text = Text::new(line.clone());
    }
}

fn update_site_logistics_hud(
    time: Res<Time>,
    mut settings: ResMut<HudAggregateSettings>,
    bindings: Res<InputBindings>,
    focus: Res<HudLogisticsFocus>,
    roots: Query<(), With<LogisticsSiteRoot>>,
    storage_entity_q: Query<Entity, With<ResourceStorage>>,
    storage_q: Query<&ResourceStorage>,
    cap_q: Query<&ResourceStorageCapacity>,
    member_q: Query<(Entity, &LogisticsSiteMember)>,
    producer_q: Query<&ResourceProducer>,
    consumer_q: Query<&ResourceConsumer>,
    mut text_q: Query<&mut Text, With<ResourceDisplay>>,
) {
    settings.accumulator += time.delta_secs();
    if settings.accumulator < settings.summary_interval_secs {
        return;
    }
    settings.accumulator = 0.0;

    let summary = match focus.tracked_entity {
        None => format!(
            "Site logistics — no focus\n({} cycle · {} list · primary-click storage)",
            InputBindings::format_key(bindings.cycle_logistics_focus),
            InputBindings::format_key(bindings.toggle_logistics_targets_panel),
        ),
        Some(hub_or_single) => {
            let is_hub = roots.get(hub_or_single).is_ok();
            let involved = storage_entities_for_focus(
                hub_or_single,
                is_hub,
                &storage_entity_q,
                &member_q,
            );
            if involved.is_empty() {
                format!(
                    "Site logistics — {:?}\n(no ResourceStorage on this focus)",
                    hub_or_single
                )
            } else {
                let (amounts, caps) = merge_amounts_and_caps(&involved, &storage_q, &cap_q);
                let flow_src = if is_hub {
                    hub_or_single
                } else {
                    involved[0]
                };
                let producer = producer_q.get(flow_src).ok();
                let consumer = consumer_q.get(flow_src).ok();
                format_merged_site_panel(
                    hub_or_single,
                    is_hub,
                    &involved,
                    &amounts,
                    &caps,
                    producer,
                    consumer,
                )
            }
        }
    };

    for mut text in text_q.iter_mut() {
        *text = Text::new(summary.clone());
    }
}

pub fn resource_glyph(ty: ResourceType) -> char {
    match ty {
        ResourceType::Wood => 'W',
        ResourceType::Coal => 'K',
        ResourceType::Oil => 'O',
        ResourceType::RareEarth => 'R',
        ResourceType::Metal => 'M',
        ResourceType::Steel => 'S',
        ResourceType::Concrete => 'C',
        ResourceType::Fertilizer => 'F',
        ResourceType::Chemicals => 'H',
        ResourceType::Electronics => 'E',
        ResourceType::Energy => 'N',
        ResourceType::Fuel => 'u',
        ResourceType::Ammunition => 'A',
        ResourceType::WarSupply => 'G',
        ResourceType::Knowledge => 'Q',
        ResourceType::Labour => 'L',
        ResourceType::Food => 'f',
        ResourceType::Water => 'w',
        ResourceType::Paper => 'P',
        ResourceType::Electricity => 'X',
    }
}

fn ascii_bar(stock: f32, denom: f32, width: usize) -> String {
    if denom <= 0.0 || width == 0 {
        return ".".repeat(width);
    }
    let filled = ((stock / denom).clamp(0.0, 1.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "|".repeat(filled), ".".repeat(width - filled))
}

fn flow_suffix(
    ty: ResourceType,
    producer: Option<&ResourceProducer>,
    consumer: Option<&ResourceConsumer>,
) -> String {
    let mut prod = 0.0f32;
    let mut cons = 0.0f32;
    if let Some(p) = producer {
        if p.resource_type == ty {
            prod = p.production_rate * p.efficiency.clamp(0.0, 1.0);
        }
    }
    if let Some(c) = consumer {
        if let Some(r) = c.consumption_rates.get(&ty) {
            cons = *r;
        }
    }
    if prod > 0.001 || cons > 0.001 {
        format!(" +{:.1}/s −{:.1}/s", prod, cons)
    } else {
        String::new()
    }
}

fn format_merged_site_panel(
    focus: Entity,
    is_hub: bool,
    involved: &[Entity],
    amounts: &HashMap<ResourceType, f32>,
    caps: &HashMap<ResourceType, f32>,
    producer: Option<&ResourceProducer>,
    consumer: Option<&ResourceConsumer>,
) -> String {
    let row_max = amounts
        .values()
        .cloned()
        .fold(0.01f32, |a, b| a.max(b));

    let mut pairs: Vec<(ResourceType, f32)> = amounts
        .iter()
        .filter(|(_, v)| **v > 0.001)
        .map(|(k, v)| (*k, *v))
        .collect();
    pairs.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));

    let kind = if is_hub { "hub roll-up" } else { "storage" };
    let header = format!(
        "Site {:?} ({kind}) — {} storages\n",
        focus,
        involved.len()
    );
    if pairs.is_empty() {
        return format!("{header}(empty inventory)");
    }

    let lines: Vec<String> = pairs
        .into_iter()
        .map(|(ty, stock)| {
            let g = resource_glyph(ty);
            let cap = caps.get(&ty).copied().unwrap_or(0.0);
            let denom = if cap > 0.001 {
                cap
            } else {
                row_max
            };
            let bar = ascii_bar(stock, denom, 8);
            let flows = flow_suffix(ty, producer, consumer);
            let cap_hint = if cap > 0.001 {
                format!("/{:.0}", cap)
            } else {
                String::new()
            };
            format!(
                "[{}] {} {:>8.1}{}{}",
                g,
                bar,
                stock,
                cap_hint,
                if flows.is_empty() { String::new() } else { flows }
            )
        })
        .collect();

    format!("{}{}", header, lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_respects_capacity_denominator() {
        let b = ascii_bar(50.0, 100.0, 8);
        assert_eq!(b, "||||....");
    }

    #[test]
    fn flow_shows_producer_and_consumer() {
        use std::collections::HashMap;
        let p = ResourceProducer {
            resource_type: ResourceType::Wood,
            production_rate: 10.0,
            max_production_rate: 10.0,
            energy_consumption: 0.0,
            efficiency: 1.0,
        };
        let mut rates = HashMap::new();
        rates.insert(ResourceType::Wood, 3.0);
        let c = ResourceConsumer {
            resource_types: vec![ResourceType::Wood],
            consumption_rates: rates,
            required_amounts: HashMap::new(),
        };
        let s = flow_suffix(ResourceType::Wood, Some(&p), Some(&c));
        assert!(s.contains("10.0"));
        assert!(s.contains("3.0"));
    }
}
