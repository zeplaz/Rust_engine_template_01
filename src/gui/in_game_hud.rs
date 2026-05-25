// Gameplay HUD — **Bevy UI only** (logistics summary + command/alerts placeholders).
// Colors / chrome: [`crate::gui::UiPalette`] — `prompts/guides/ui_design_language_plan_v1.md` §5.
// Dev tools (diagnostics, world gen, etc.) live in **egui** behind shortcuts; see `ui_boundary_guide_v1.md`.
// G3B: prompts/matrix/gap_remediation/runbook/g3b_production_hud_steps_v1.md

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::ui::{ComputedNode, Display, UiGlobalTransform, UiSystems};
use bevy::window::PrimaryWindow;

use crate::engine::BaseState;
use crate::strategic::{
    ActiveMissions, CityPlanningHints, LogisticsAiRuntime, NarrativeObservationBus,
    OperationalTheaterSummary, StrategicOverlayDisplayPolicy, MAX_STRATEGIC_FACTION_SLOTS,
};
use crate::construction::ToolContext;
use crate::gui::hud::{
    update_developmental_cause_strip_system, update_developmental_context_strip_system,
    tool_context_uses_icon_atlas, BuildRailRoot, BuildRailToolIcon, BuildRailToolLabel,
    BuildRailToolSlot, ContextTrayBodyLine, ContextTrayBodyRoot,
    ContextTrayRoot, ContextTrayTab, ContextTrayTabButton, ContextTrayTabLabel, IconId,
    LogisticsVehicleChip, LogisticsVehicleChipIcon, LogisticsVehicleChipLabel,
    LogisticsVehicleChipRow, PetroleumPanelTabIcon, PetroleumPanelTabLabel, PetroleumPanelTabRoot,
    DevelopmentalCauseStripLine,
    DevelopmentalCauseStripRoot, DevelopmentalContextStripLine, MapViewportFrameInset,
    MinimapChromeRoot, MinimapGpuImageNode, OpsStripAlertBadge, OpsStripAlertBadgeText,
    OpsStripAlerts, OpsStripIntel, OpsStripPower, OpsStripTime, OpsStripTrayAffordance,
    OpsStripWeather, OpsStripZone, SimulationShellPhase2Plugin, BUILD_RAIL_W_PX,
    COMMAND_LEFT_STACK_COLUMN_GAP_PX, CONTEXT_RAIL_W_PX, LEFT_CONTEXT_STACK_BODY_W_PX,
    CONTEXT_TRAY_BODY_H_PX, CONTEXT_TRAY_TAB_H_PX, OPS_STRIP_TOP_OFFSET_PX, MAP_FRAME_INSET_PX,
};
use crate::gui::ui_gates::in_simulation_or_editor;

use crate::entities::production::core::{
    resolve_logistics_focus_entity, resource_category_tag, storage_entities_for_focus,
    LogisticsSiteMember, LogisticsSiteRoot, ResourceConsumer, ResourceProducer, ResourceStorage,
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

/// **Simulation** Bevy UI chrome: operations strip + left context column (`base_ui_direction_principls.md`).
#[derive(Component)]
pub struct SimulationCommandShellRoot;

/// Flex “hole” over the live map (world visible through this node). Used to gate map picks vs chrome.
#[derive(Component)]
pub struct SimulationMapViewportFill;

/// Left command stack — **overlays** the map hole (does not shrink [`SimulationMapViewportFill`]).
#[derive(Component)]
struct CommandLeftStackOverlay;

#[derive(Component)]
struct LeftContextRail;

#[derive(Component)]
struct LeftContextStackBody;

#[derive(Component)]
struct LeftContextStackCollapse;

#[derive(Component)]
struct ObjectivesHudLine;

#[derive(Component)]
struct SimulationNarrativeFeedLine;

pub(crate) const LEFT_CONTEXT_RAIL_W_PX: f32 = CONTEXT_RAIL_W_PX;
/// Bottom context tray must not cover the fixed build rail column.
pub(crate) const CONTEXT_TRAY_LEFT_INSET_PX: f32 =
    LEFT_CONTEXT_RAIL_W_PX + COMMAND_LEFT_STACK_COLUMN_GAP_PX + BUILD_RAIL_W_PX;
/// Fixed width for the expanded left command stack (prevents flex from squeezing `sim_map_fill`).
pub(crate) const LEFT_CONTEXT_STACK_W_PX: f32 = LEFT_CONTEXT_STACK_BODY_W_PX;

/// Collapse state for the left command stack (`toggle_command_left_stack` / rail button).
#[derive(Resource, Clone, Copy, Debug)]
pub struct CommandLeftStackState {
    pub collapsed: bool,
}

impl Default for CommandLeftStackState {
    fn default() -> Self {
        // Bevy left stack is narrative-only; construction uses resizable egui `build_toolbox`.
        Self { collapsed: true }
    }
}
const OPS_STRIP_H_PX: f32 = 38.0;
const OPS_STRIP_MONO_PT: f32 = 13.0;
/// L0 developmental context strip — always on in simulation shell (`developmental_ux_runbook_v1.md`).
const DEV_CONTEXT_STRIP_H_PX: f32 = 26.0;
const DEV_CONTEXT_MONO_PT: f32 = 11.5;
/// L2 cause chain hint row (`developmental_ux_runbook_v1.md` § UX-2).
const DEV_CAUSE_STRIP_H_PX: f32 = 22.0;
const HUD_MONO_PT: f32 = 13.5;

/// Fixed chrome above the simulation map fill node (logical px).
pub const SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX: f32 = OPS_STRIP_TOP_OFFSET_PX
    + OPS_STRIP_H_PX
    + DEV_CONTEXT_STRIP_H_PX
    + DEV_CAUSE_STRIP_H_PX;
/// `center_row` padding (must match spawn — used for rescue-floor math).
pub(crate) const CENTER_ROW_EDGE_PAD_PX: f32 = 8.0;
#[inline]
#[must_use]
pub fn simulation_map_fallback_logical_extent(window: Vec2) -> Vec2 {
    Vec2::new(
        window.x.max(crate::gui::hud::VIEWPORT_SIM_MAP_SAFE_MIN_W),
        window.y.max(crate::gui::hud::VIEWPORT_SIM_MAP_SAFE_MIN_H),
    )
}

/// Presentation copy of [`crate::gui::AuthoritativeViewport`] (logical window coords).
///
/// Dimensions always match authoritative; `valid` is true only when the hole latch is settled
/// (pointer hit-test / build overlays). Camera scissor and [`crate::render::ResolvedViewports`]
/// use [`Self::is_adequate_for_camera`] (dimensions only — not gated on `valid`).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationMapViewport {
    pub valid: bool,
    pub min: Vec2,
    pub max: Vec2,
}

/// Per-frame diagnostics for [`sync_simulation_map_viewport_system`] (`SIM_VIEW_SYNC_DEBUG`).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationMapViewportTrace {
    pub measured_valid: bool,
    pub measured_size: Vec2,
    pub committed_from_stable_hold: bool,
    pub committed_size: Vec2,
    pub settle_streak: u8,
    pub layout_settled: bool,
}

/// Stable/pending hole internals for [`crate::render::visual_diagnostics`].
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationMapViewportDebug {
    pub frozen: bool,
    pub pending_min: Vec2,
    pub pending_max: Vec2,
    pub pending_wh: Vec2,
    /// Last raw UI measure before commit (for overlay / authority trace).
    pub measured_valid: bool,
    pub measured_min: Vec2,
    pub measured_max: Vec2,
    /// Floor-stabilized semantic authority (`commit_authority_from_semantic`; sim_map_fill only).
    pub solver_valid: bool,
    pub solver_min: Vec2,
    pub solver_max: Vec2,
    /// Last hole-latch branch ([`crate::gui::authoritative_viewport::advance_simulation_map_hole_latch`]).
    pub last_commit: &'static str,
}

impl SimulationMapViewport {
    /// Logical hole size in window coordinates.
    #[inline]
    #[must_use]
    pub fn logical_size(self) -> Vec2 {
        (self.max - self.min).max(Vec2::ZERO)
    }

    /// Large enough for map camera scissor + orthographic fit (matches [`ViewportRectSanity`]).
    /// Does **not** require [`Self::valid`] — render must not wait on hole latch settle.
    #[inline]
    #[must_use]
    pub fn is_adequate_for_camera(self) -> bool {
        crate::gui::authoritative_viewport::simulation_map_viewport_adequate_dims(self.min, self.max)
    }

    /// `cursor` from [`Window::cursor_position`] (logical px).
    #[inline]
    #[must_use]
    pub fn contains_cursor(self, cursor: Vec2) -> bool {
        self.valid
            && cursor.x >= self.min.x
            && cursor.x <= self.max.x
            && cursor.y >= self.min.y
            && cursor.y <= self.max.y
    }
}

/// When **compact**, the strategic HUD shows a one-line summary; full line includes city-planning hints.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct StrategicHudStripState {
    pub compact: bool,
}

/// PostUpdate ordering: measure Bevy UI map hole before camera scissor + ortho fit.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationViewportSyncSet {
    MeasureUiHole,
    ApplyCameraScissor,
}

pub struct InGameHudPlugin;

impl Plugin for InGameHudPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (
                SimulationViewportSyncSet::MeasureUiHole,
                SimulationViewportSyncSet::ApplyCameraScissor,
            )
                .chain(),
        );
        app.init_resource::<HudLogisticsFocus>()
            .init_resource::<HudAggregateSettings>()
            .init_resource::<StrategicHudStripState>()
            .init_resource::<CommandLeftStackState>()
            .init_resource::<crate::gui::AuthoritativeViewport>()
            .init_resource::<SimulationMapViewport>()
            .init_resource::<crate::gui::viewport_layout_solver::SemanticViewportRect>()
            .init_resource::<SimulationMapViewportTrace>()
            .init_resource::<SimulationMapViewportDebug>()
            .init_resource::<crate::gui::SimulationMapViewportHoleLatch>()
            .init_resource::<crate::gui::hud::ViewportRectSanity>()
            .add_plugins(crate::gui::hud::ViewportIntegrityAssertPlugin)
            .add_plugins(SimulationShellPhase2Plugin);
        register_sim_command_shell_lifecycle(app);
        app.add_systems(
                Update,
                reset_simulation_map_viewport_on_left_stack_toggle
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    strategic_hud_chrome_input,
                    sync_command_left_stack_visibility,
                    command_left_stack_rail_interaction,
                    left_context_stack_collapse_interaction,
                    attach_storage_picking_hooks,
                    cycle_logistics_focus_dev,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                update_developmental_context_strip_system.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                update_developmental_cause_strip_system.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                update_objectives_hud_line.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                update_simulation_narrative_feed_system.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                update_site_logistics_hud.run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                update_strategic_ops_hud.run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_simulation_map_viewport_system,
                    crate::render::view_runtime::commit_simulation_map_hole_to_authority,
                )
                    .chain()
                    .run_if(in_simulation_or_editor)
                    .after(UiSystems::Stack)
                    .in_set(SimulationViewportSyncSet::MeasureUiHole),
            );
    }
}

/// Sim Bevy shell lifecycle — **Simulation only** (never `AppState::WorldGen`; see `world_gen_chrome_contract`).
fn register_sim_command_shell_lifecycle(app: &mut App) {
    app.add_systems(OnEnter(BaseState::Simulation), spawn_simulation_command_shell)
        .add_systems(
            OnExit(BaseState::Simulation),
            despawn_simulation_command_shell,
        );
}

/// Left-stack show/hide changes flex width — reset settle so the map hole can be re-measured.
fn reset_simulation_map_viewport_on_left_stack_toggle(
    state: Res<CommandLeftStackState>,
    mut latch: ResMut<crate::gui::SimulationMapViewportHoleLatch>,
    mut last: Local<Option<bool>>,
) {
    let collapsed = state.collapsed;
    if last.map_or(false, |p| p != collapsed) {
        latch.reset_for_layout_change();
    }
    *last = Some(collapsed);
}

pub fn sync_simulation_map_viewport_system(
    q: Query<(&ComputedNode, &UiGlobalTransform), With<SimulationMapViewportFill>>,
    mut authority: ResMut<crate::gui::AuthoritativeViewport>,
    mut out: ResMut<SimulationMapViewport>,
    mut semantic: ResMut<crate::gui::viewport_layout_solver::SemanticViewportRect>,
    mut trace: ResMut<SimulationMapViewportTrace>,
    mut sim_dbg: ResMut<SimulationMapViewportDebug>,
    mut latch: ResMut<crate::gui::SimulationMapViewportHoleLatch>,
    mut cam_latch: ResMut<crate::gui::MainWorldCameraViewportLatch>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut sanity: ResMut<crate::gui::hud::ViewportRectSanity>,
    mut generation: Local<u64>,
) {
    let Ok(w) = win.single() else {
        out.valid = false;
        latch.hole_ready = false;
        latch.settle_streak = 0;
        return;
    };
    if w.width() < 32.0 || w.height() < 32.0 {
        out.valid = false;
        return;
    }
    let scale = w.scale_factor().max(1e-6);
    let Ok((node, xf)) = q.single() else {
        out.valid = false;
        return;
    };
    let window_logical = Vec2::new(w.width(), w.height());
    if latch.last_window_logical != Vec2::ZERO
        && (window_logical - latch.last_window_logical).length_squared() > 4.0
    {
        latch.reset_for_layout_change();
        cam_latch.using_hole = false;
        cam_latch.valid_streak = 0;
        cam_latch.invalid_streak = 0;
    }
    let mut measured = crate::gui::authoritative_viewport::measure_sim_map_fill_viewport(
        node,
        xf,
        scale,
        window_logical,
        sanity.as_mut(),
    );

    if crate::gui::hud::ui_layout_tree_debug_enabled() {
        let (raw_min, raw_max) =
            crate::gui::authoritative_viewport::measure_sim_map_fill_corners_crosscheck(
                node, xf, scale,
            );
        let (cmin, cmax) = crate::gui::authoritative_viewport::clamp_simulation_map_aabb_to_window(
            raw_min,
            raw_max,
            window_logical,
        );
        let d = (cmax - cmin) - (measured.max - measured.min);
        if d.length() > 1.0 {
            warn!(
                target: "viewport_authority::measure",
                ?d,
                "ComputedNode center vs corner AABB mismatch"
            );
        }
    }

    *generation = generation.wrapping_add(1);
    measured.generation = *generation;

    crate::gui::authoritative_viewport::publish_simulation_map_viewport(
        &mut measured,
        semantic.as_mut(),
        latch.as_mut(),
        out.as_mut(),
        trace.as_mut(),
        sim_dbg.as_mut(),
        window_logical,
        *generation,
    );
    *authority = measured;
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
        "Tools — Options/keys {} · Diagnostics {} · Pressure {} · Faction {} · Logistics list {} · Cycle focus {} · World gen {} · Agent perms {} · Collapse left stack {}.",
        InputBindings::format_key(bindings.toggle_keybindings_options),
        InputBindings::format_key(bindings.toggle_diagnostics),
        InputBindings::format_key(bindings.toggle_pressure_composer),
        InputBindings::format_key(bindings.toggle_faction_tools),
        InputBindings::format_key(bindings.toggle_logistics_targets_panel),
        InputBindings::format_key(bindings.cycle_logistics_focus),
        InputBindings::format_key(bindings.toggle_world_generator),
        InputBindings::format_key(bindings.toggle_agent_permissions),
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

    let construction = format!(
        "Construction — left **Construction** toolbox (Residential / Roads / Rail / Mock shapes / …). Tile info labels {}. Optional: cycle tools with `;`. Shift+click queues blueprint; roads use segment draft until spline path.",
        InputBindings::format_key(bindings.toggle_construction_tile_labels),
    );

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
                ..default()
            },
            FocusPolicy::Pass,
            Pickable::IGNORE,
            ZIndex(750),
            SimulationCommandShellRoot,
            crate::gui::hud::DebugLayoutTag("hud_root"),
            Name::new("hud_root"),
        ))
        .with_children(|shell| {
            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(OPS_STRIP_TOP_OFFSET_PX),
                        width: Val::Percent(100.0),
                        height: Val::Px(OPS_STRIP_H_PX),
                        min_height: Val::Px(OPS_STRIP_H_PX),
                        max_height: Val::Px(OPS_STRIP_H_PX),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(14.0),
                        border: UiRect::bottom(Val::Px(1.0)),
                        border_radius: BorderRadius::ZERO,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_paper_fill()),
                    strip_border,
                    ZIndex(1200),
                    OperationsStripRoot,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(palette.bevy_paper_fill()),
                            BorderColor::all(palette.bevy_border_subtle()),
                            OpsStripZone::Time,
                        ))
                        .with_children(|z| {
                            z.spawn((
                                Text::new("T+00000  RUN    v=1.0x"),
                                TextFont::from_font_size(fs).with_font(font.clone()),
                                TextColor(palette.bevy_fg_data()),
                                OpsStripTime,
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            Node {
                                flex_grow: 1.0,
                                min_width: Val::Px(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.0),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(palette.bevy_border_subtle()),
                            OpsStripZone::Alerts,
                        ))
                        .with_children(|c| {
                            c.spawn((
                                Node {
                                    width: Val::Px(22.0),
                                    height: Val::Px(22.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(palette.bevy_paper_fill()),
                                BorderColor::all(palette.bevy_wire_magenta()),
                                OpsStripAlertBadge,
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("◆0"),
                                    TextFont::from_font_size(9.0).with_font(font.clone()),
                                    TextColor(palette.bevy_fg_data()),
                                    OpsStripAlertBadgeText,
                                ));
                            });
                            c.spawn((
                                Text::new("ALERTS  0"),
                                TextFont::from_font_size(fs).with_font(font.clone()),
                                TextColor(palette.bevy_text_muted()),
                                OpsStripAlerts,
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(palette.bevy_border_subtle()),
                            OpsStripZone::Intel,
                        ))
                        .with_children(|z| {
                            z.spawn((
                                Text::new("INTEL  —"),
                                TextFont::from_font_size(fs).with_font(font.clone()),
                                TextColor(palette.bevy_secondary_text()),
                                OpsStripIntel,
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(palette.bevy_border_subtle()),
                            OpsStripZone::Weather,
                        ))
                        .with_children(|z| {
                            z.spawn((
                                Text::new("WX  —"),
                                TextFont::from_font_size(fs).with_font(font.clone()),
                                TextColor(palette.bevy_secondary_text()),
                                OpsStripWeather,
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(palette.bevy_border_subtle()),
                            OpsStripZone::Power,
                        ))
                        .with_children(|z| {
                            z.spawn((
                                Text::new("PWR  —"),
                                TextFont::from_font_size(fs).with_font(font.clone()),
                                TextColor(palette.bevy_secondary_text()),
                                OpsStripPower,
                            ));
                        });
                    parent
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                margin: UiRect::left(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(palette.bevy_border_subtle()),
                            OpsStripZone::TrayAffordance,
                        ))
                        .with_children(|z| {
                            z.spawn((
                                Text::new("▼ TRAY"),
                                TextFont::from_font_size(11.0).with_font(font.clone()),
                                TextColor(palette.bevy_accent_terminal()),
                                OpsStripTrayAffordance,
                            ));
                        });
                });

            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(OPS_STRIP_TOP_OFFSET_PX + OPS_STRIP_H_PX),
                        width: Val::Percent(100.0),
                        height: Val::Px(DEV_CONTEXT_STRIP_H_PX),
                        min_height: Val::Px(DEV_CONTEXT_STRIP_H_PX),
                        max_height: Val::Px(DEV_CONTEXT_STRIP_H_PX),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        border_radius: BorderRadius::ZERO,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_hud_panel_fill()),
                    strip_border,
                    ZIndex(1150),
                    Pickable::IGNORE,
                    FocusPolicy::Pass,
                ))
                .with_children(|ctx_row| {
                    ctx_row.spawn((
                        Text::new("CONTEXT — …"),
                        TextFont::from_font_size(DEV_CONTEXT_MONO_PT).with_font(font.clone()),
                        TextColor(palette.bevy_secondary_text()),
                        DevelopmentalContextStripLine,
                    ));
                });

            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(OPS_STRIP_TOP_OFFSET_PX + OPS_STRIP_H_PX + DEV_CONTEXT_STRIP_H_PX),
                        width: Val::Percent(100.0),
                        height: Val::Px(DEV_CAUSE_STRIP_H_PX),
                        min_height: Val::Px(DEV_CAUSE_STRIP_H_PX),
                        max_height: Val::Px(DEV_CAUSE_STRIP_H_PX),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(1.0)),
                        border_radius: BorderRadius::ZERO,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_hud_panel_fill()),
                    strip_border,
                    ZIndex(1140),
                    Pickable::IGNORE,
                    FocusPolicy::Pass,
                    Visibility::Visible,
                    DevelopmentalCauseStripRoot,
                ))
                .with_children(|cause_row| {
                    cause_row.spawn((
                        Text::new("CAUSE — …"),
                        TextFont::from_font_size(DEV_CONTEXT_MONO_PT).with_font(font.clone()),
                        TextColor(palette.bevy_text_muted()),
                        DevelopmentalCauseStripLine,
                    ));
                });

            // Full-window map hole; ops/context/cause strips and left stack draw above (higher Z).
            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Pickable::IGNORE,
                    FocusPolicy::Pass,
                    ZIndex(100),
                    crate::gui::hud::DebugLayoutTag("center_row"),
                    Name::new("center_row"),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            min_width: Val::Px(crate::gui::hud::VIEWPORT_SIM_MAP_LAYOUT_MIN_W),
                            min_height: Val::Px(crate::gui::hud::VIEWPORT_SIM_MAP_LAYOUT_MIN_H),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        Pickable::IGNORE,
                        FocusPolicy::Pass,
                        SimulationMapViewportFill,
                        crate::gui::hud::DebugLayoutTag("sim_map_fill"),
                        Name::new("sim_map_fill"),
                    ))
                    .with_children(|inset| {
                        inset.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(MAP_FRAME_INSET_PX),
                                right: Val::Px(MAP_FRAME_INSET_PX),
                                top: Val::Px(MAP_FRAME_INSET_PX),
                                bottom: Val::Px(MAP_FRAME_INSET_PX),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            Pickable::IGNORE,
                            FocusPolicy::Pass,
                            BorderColor::all(palette.bevy_wire_magenta().with_alpha(0.45)),
                            MapViewportFrameInset,
                            Name::new("map_viewport_frame_inset"),
                        ));
                    });
                });

            // Left stack overlays the map — does not participate in viewport measure.
            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(CENTER_ROW_EDGE_PAD_PX),
                        top: Val::Px(SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX),
                        bottom: Val::Px(CENTER_ROW_EDGE_PAD_PX),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        column_gap: Val::Px(COMMAND_LEFT_STACK_COLUMN_GAP_PX),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    FocusPolicy::Pass,
                    ZIndex(900),
                    CommandLeftStackOverlay,
                    crate::gui::hud::DebugLayoutTag("left_stack_overlay"),
                    Name::new("left_stack_overlay"),
                ))
                .with_children(|left_pack| {
                    left_pack
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(LEFT_CONTEXT_RAIL_W_PX),
                                min_height: Val::Px(120.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::ZERO,
                                ..default()
                            },
                            BackgroundColor(palette.bevy_hud_panel_fill()),
                            BorderColor::all(palette.bevy_wire_magenta()),
                            Visibility::Hidden,
                            LeftContextRail,
                        ))
                        .with_children(|rail| {
                            for icon in ["⏱", "⛭", "◎", "☰"] {
                                rail.spawn((
                                    Text::new(icon),
                                    TextFont::from_font_size(14.0).with_font(font.clone()),
                                    TextColor(palette.bevy_text_muted()),
                                ));
                            }
                        });

                    left_pack
                        .spawn((
                            Node {
                                width: Val::Px(BUILD_RAIL_W_PX),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            ZIndex(920),
                            BuildRailRoot,
                            Name::new("build_rail"),
                        ))
                        .with_children(|rail| {
                            for tool in [
                                ToolContext::Roads,
                                ToolContext::Rail,
                                ToolContext::Utilities,
                                ToolContext::Military,
                                ToolContext::Industry,
                                ToolContext::Ecology,
                                ToolContext::Civil,
                            ] {
                                rail.spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(100.0),
                                        min_height: Val::Px(32.0),
                                        padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(palette.bevy_hud_panel_fill()),
                                    BorderColor::all(palette.bevy_border_subtle()),
                                    BuildRailToolSlot(tool),
                                ))
                                .with_children(|b| {
                                    if tool_context_uses_icon_atlas(tool) {
                                        b.spawn((
                                            Node {
                                                width: Val::Px(32.0),
                                                height: Val::Px(32.0),
                                                ..default()
                                            },
                                            BuildRailToolIcon,
                                            bevy::ui::widget::ImageNode::default(),
                                            Visibility::Hidden,
                                        ));
                                    }
                                    b.spawn((
                                        Text::new(tool.label()),
                                        TextFont::from_font_size(10.0).with_font(font.clone()),
                                        TextColor(palette.bevy_text_muted()),
                                        BuildRailToolLabel,
                                    ));
                                });
                            }
                        });

                    left_pack
                        .spawn((
                            Node {
                                width: Val::Px(LEFT_CONTEXT_STACK_W_PX),
                                height: Val::Percent(100.0),
                                max_height: Val::Percent(100.0),
                                flex_shrink: 0.0,
                                padding: UiRect::all(Val::Px(10.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::ZERO,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(palette.bevy_hud_panel_fill()),
                            BorderColor::all(palette.bevy_wire_magenta()),
                            Visibility::Visible,
                            ZIndex(910),
                            LeftContextStackBody,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        align_self: AlignSelf::FlexEnd,
                                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(palette.bevy_hud_panel_fill()),
                                    BorderColor::all(palette.bevy_border_subtle()),
                                    LeftContextStackCollapse,
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new("◀"),
                                        TextFont::from_font_size(12.0).with_font(font.clone()),
                                        TextColor(palette.bevy_text_muted()),
                                    ));
                                });
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
                                Text::new("STORY — …"),
                                tf_hud(HUD_MONO_PT),
                                TextColor(palette.bevy_accent_terminal()),
                                SimulationNarrativeFeedLine,
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

            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Px(260.0),
                        height: Val::Px(220.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                    FocusPolicy::Pass,
                    Visibility::Hidden,
                    BorderColor::all(palette.bevy_wire_magenta().with_alpha(0.75)),
                    BackgroundColor(Color::NONE),
                    ZIndex(850),
                    MinimapChromeRoot,
                    Name::new("minimap_chrome_root"),
                ))
                .with_children(|gpu| {
                    gpu.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        Visibility::Hidden,
                        MinimapGpuImageNode,
                        bevy::ui::widget::ImageNode::from(Handle::<Image>::default()),
                    ));
                });

            shell
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(CONTEXT_TRAY_LEFT_INSET_PX),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        height: Val::Px(CONTEXT_TRAY_TAB_H_PX),
                        min_height: Val::Px(CONTEXT_TRAY_TAB_H_PX),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    Visibility::Hidden,
                    ZIndex(1100),
                    ContextTrayRoot,
                    Name::new("context_tray_root"),
                ))
                .with_children(|tray| {
                    tray.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(CONTEXT_TRAY_TAB_H_PX),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            border: UiRect::top(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(palette.bevy_hud_panel_fill()),
                        BorderColor {
                            top: palette.bevy_wire_magenta(),
                            ..BorderColor::all(Color::NONE)
                        },
                    ))
                    .with_children(|tabs| {
                        for tab in [
                            ContextTrayTab::Alerts,
                            ContextTrayTab::Intel,
                            ContextTrayTab::Logistics,
                            ContextTrayTab::Diagnostics,
                        ] {
                            tabs.spawn((
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(palette.bevy_hud_panel_fill()),
                                BorderColor::all(palette.bevy_wire_magenta().with_alpha(0.55)),
                                ContextTrayTabButton(tab),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new(tab.label()),
                                    TextFont::from_font_size(11.0).with_font(font.clone()),
                                    TextColor(palette.bevy_secondary_text()),
                                    ContextTrayTabLabel,
                                ));
                            });
                        }
                    });
                    tray.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(CONTEXT_TRAY_BODY_H_PX),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(palette.bevy_bg_vellum()),
                        ContextTrayBodyRoot,
                    ))
                    .with_children(|body| {
                        body.spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.0),
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                                margin: UiRect::bottom(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(palette.bevy_hud_panel_fill()),
                            BorderColor::all(palette.bevy_border_subtle()),
                            Visibility::Hidden,
                            PetroleumPanelTabRoot,
                            Name::new("petroleum_panel_tab"),
                        ))
                        .with_children(|tab| {
                            tab.spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                PetroleumPanelTabIcon,
                                bevy::ui::widget::ImageNode::default(),
                            ));
                            tab.spawn((
                                Text::new("Petroleum"),
                                TextFont::from_font_size(11.0).with_font(font.clone()),
                                TextColor(palette.bevy_secondary_text()),
                                PetroleumPanelTabLabel,
                            ));
                        });
                        body.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::FlexStart,
                                column_gap: Val::Px(6.0),
                                margin: UiRect::bottom(Val::Px(6.0)),
                                ..default()
                            },
                            Visibility::Hidden,
                            LogisticsVehicleChipRow,
                            Name::new("logistics_vehicle_chips"),
                        ))
                        .with_children(|row| {
                            for (id, label) in [
                                (IconId::Truck, "Truck"),
                                (IconId::Ural, "Ural"),
                                (IconId::Bus, "Bus"),
                            ] {
                                row.spawn((
                                    Button,
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        row_gap: Val::Px(2.0),
                                        padding: UiRect::axes(Val::Px(4.0), Val::Px(3.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(palette.bevy_hud_panel_fill()),
                                    BorderColor::all(palette.bevy_border_subtle()),
                                    LogisticsVehicleChip(id),
                                ))
                                .with_children(|chip| {
                                    chip.spawn((
                                        Node {
                                            width: Val::Px(24.0),
                                            height: Val::Px(24.0),
                                            flex_shrink: 0.0,
                                            ..default()
                                        },
                                        LogisticsVehicleChipIcon,
                                        bevy::ui::widget::ImageNode::default(),
                                    ));
                                    chip.spawn((
                                        Text::new(label),
                                        TextFont::from_font_size(9.0).with_font(font.clone()),
                                        TextColor(palette.bevy_text_muted()),
                                        LogisticsVehicleChipLabel,
                                    ));
                                });
                            }
                        });
                        body.spawn((
                            Text::new("CONTEXT — select a tab or click ALERTS on ops strip"),
                            TextFont::from_font_size(DEV_CONTEXT_MONO_PT).with_font(font.clone()),
                            TextColor(palette.bevy_secondary_text()),
                            ContextTrayBodyLine,
                        ));
                    });
                });
        });
}

fn truncate_slot(s: &str, max_chars: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max_chars {
        return t.to_string();
    }
    format!("{}…", t.chars().take(max_chars.saturating_sub(1)).collect::<String>())
}

fn update_simulation_narrative_feed_system(
    bus: Option<Res<NarrativeObservationBus>>,
    mut q: Query<&mut Text, With<SimulationNarrativeFeedLine>>,
    mut last_line: Local<String>,
) {
    let Some(bus) = bus.as_ref() else {
        return;
    };
    let tail = bus.format_hud_tail(2);
    let line_display = if tail.is_empty() {
        "STORY — Operational feed idle (routing / theater spikes enqueue lines here).".to_string()
    } else {
        format!("STORY — {tail}")
    };
    if !bus.is_changed() && *last_line == line_display {
        return;
    }
    *last_line = line_display.clone();
    for mut t in &mut q {
        *t = Text::new(line_display.clone());
    }
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
    mut q: ParamSet<(
        Query<(&mut Visibility, &mut Node), With<LeftContextStackBody>>,
        Query<&mut Visibility, With<LeftContextRail>>,
        Query<(&mut Visibility, &mut Node), With<BuildRailRoot>>,
    )>,
) {
    let collapsed = state.collapsed;
    for (mut v, mut node) in q.p0().iter_mut() {
        *v = if collapsed {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        node.display = if collapsed {
            Display::None
        } else {
            Display::Flex
        };
    }
    for mut v in q.p1().iter_mut() {
        *v = if collapsed {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (mut v, mut node) in q.p2().iter_mut() {
        *v = Visibility::Visible;
        node.display = Display::Flex;
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

fn left_context_stack_collapse_interaction(
    q: Query<&Interaction, (Changed<Interaction>, With<LeftContextStackCollapse>)>,
    mut state: ResMut<CommandLeftStackState>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            state.collapsed = true;
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
            let tag = resource_category_tag(ty);
            let cap_hint = if cap > 0.001 {
                format!("/{:.0}", cap)
            } else {
                String::new()
            };
            format!(
                "[{}] {} {:>8.1}{}{} · {}",
                g,
                bar,
                stock,
                cap_hint,
                if flows.is_empty() { String::new() } else { flows },
                tag,
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
    fn simulation_map_viewport_contains_cursor_respects_valid_and_aabb() {
        let vp = SimulationMapViewport {
            valid: true,
            min: Vec2::new(10.0, 20.0),
            max: Vec2::new(100.0, 200.0),
        };
        assert!(vp.contains_cursor(Vec2::new(50.0, 50.0)));
        assert!(!vp.contains_cursor(Vec2::new(5.0, 50.0)));
        let inv = SimulationMapViewport::default();
        assert!(!inv.contains_cursor(Vec2::ZERO));
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
