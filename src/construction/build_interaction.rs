//! Map pick + validation refresh + confirm for build strip tools.
//!
//! Buildings use a two-click FSM ([`BuildPlacementMode`]): Preview (`Place`) →
//! Adjust (lock) → second LMB commits via the same [`CommitConstructionSiteEvent`]
//! funnel as Enter.

use std::sync::atomic::{AtomicU32, Ordering};

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::parametric_commit::parametric_placement_snapshot;
use super::placement_scaling::clamp_scale_factor;
use super::queue_commit_construction_site;
use crate::construction::map_egui_projection::ConstructionMapProjection;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::{InputBindings, SimulationMapViewport};
use crate::strategic::{BuildSiteTile, LayerType, StrategicRasterConfig};

use super::build_state::{
    BuildGhostRoot, BuildGhostState, BuildPlacementMode, BuildPlacementPreview,
};
use super::build_validation::{evaluate_ghost_site_placement, ghost_valid_flag};
use super::map_pick_params::ConstructionMapPick;
use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::{ActiveBuildTool, BuildTool, DefenseKind};
use super::building_definitions::BuildingDefinitionRegistry;
use super::defense_concrete_stock::{
    apply_defensive_wall_concrete_gate, consume_concrete_on_wall_commit, total_available_concrete,
    INSUFFICIENT_CONCRETE_ERROR,
};
use super::defense_staging_stock::{
    apply_deployable_staging_gate, enqueue_deployable_staging_debit_on_commit,
    INSUFFICIENT_STAGED_ERROR, STOCK_IN_TRANSIT_ERROR,
};
use super::demolish::execute_demolish_at_tile;
use super::history::{record_demolish_execution, record_zone_spawns, ConstructionHistory};
use super::pending_construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
use super::place_feedback::{PlaceFeedbackPulse, TwoClickCommitParams};
use super::sessions::ActiveToolSession;
use super::terrain_conform::conform_world_y;
use super::zones::spawn_zone_at_tile;
use super::{GhostBuildCursor, GhostValid};
use crate::economy::logistics::{InTransitLedger, PendingSiteStagingDebits, SiteStagingStock};
use crate::economy::resource_flow::ResourceFlowNode;
use crate::strategic::FootprintTiles;

/// INPUT-02 — screen distance before a press counts as drag-to-build (not a click).
pub const BUILD_POINTER_DRAG_MIN_DISTANCE_PX: f32 = 6.0;
/// INPUT-02 — hold longer than this with any nontrivial move also counts as drag.
pub const BUILD_POINTER_DRAG_MAX_CLICK_SECS: f32 = 0.28;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildPointerGesture {
    Click,
    Drag,
}

/// Classify a pointer release after LMB down (gea4v1 §9.5.4.3).
#[must_use]
pub fn classify_build_pointer_gesture(distance_px: f32, hold_secs: f32) -> BuildPointerGesture {
    if distance_px >= BUILD_POINTER_DRAG_MIN_DISTANCE_PX {
        return BuildPointerGesture::Drag;
    }
    if hold_secs > BUILD_POINTER_DRAG_MAX_CLICK_SECS && distance_px > 1.0 {
        return BuildPointerGesture::Drag;
    }
    BuildPointerGesture::Click
}

/// Honest witness counters (lib + live) — TRIAGE-BUILD-CLICK-PLACE-001.
static TWO_CLICK_MODE_LOCKS: AtomicU32 = AtomicU32::new(0);
static TWO_CLICK_LMB_COMMITS: AtomicU32 = AtomicU32::new(0);

#[inline]
fn note_mode_lock() {
    TWO_CLICK_MODE_LOCKS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
fn note_lmb_commit() {
    TWO_CLICK_LMB_COMMITS.fetch_add(1, Ordering::Relaxed);
}

/// Map pick allowed: play-area ∧ ¬egui_blocks when gate present (P0b chrome click-through).
#[must_use]
pub fn construction_map_pick_allowed(
    gate: Option<&SimulationMapPointerGate>,
    egui_wants_pointer: bool,
    map_vp: &SimulationMapViewport,
    cursor_px: Vec2,
) -> bool {
    if let Some(g) = gate {
        return g.in_play_area && !g.egui_blocks;
    }
    !egui_wants_pointer && map_vp.contains_cursor(cursor_px)
}

/// When true, tactical map wheel zoom must yield (Ctrl rotate / Shift scale in Adjust).
#[must_use]
pub fn build_adjust_blocks_map_wheel(ghost: &BuildGhostState, keys: &ButtonInput<KeyCode>) -> bool {
    if ghost.placement_mode != BuildPlacementMode::Adjust {
        return false;
    }
    keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight)
}

pub(crate) fn placement_snapshot_for_building(
    tool: &ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
    ghost: &BuildGhostState,
    origin: BuildSiteTile,
) -> Option<crate::strategic::CommittedPlacementSnapshot> {
    let intent = tool.building_intent.as_ref()?;
    let catalog_id = intent.catalog_id.as_deref()?;
    let def = registry.get(catalog_id)?;
    Some(parametric_placement_snapshot(
        &def.footprint,
        def.family,
        origin,
        ghost.rotation_quarter_turns,
        ghost.mirror_x,
        Some(ghost.scale_factor),
    ))
}

fn resolve_site_archetype(
    tool: &ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
    strip: &BuildStripState,
) -> crate::strategic::SiteArchetype {
    if let Some(intent) = &tool.building_intent {
        if let Some(id) = &intent.catalog_id {
            if let Some(def) = registry.get(id) {
                return def.site_archetype;
            }
        }
    }
    match tool.tool {
        BuildTool::Building(id) => id.site_archetype(),
        BuildTool::Defense(kind) => kind.site_archetype(),
        _ => strip.active.site_archetype(),
    }
}

/// Left-click on map → Preview follow / Adjust lock (buildings) or legacy origin set.
///
/// Buildings: Preview (`Place`) follows cursor; first LMB → [`BuildPlacementMode::Adjust`].
/// Second LMB commits in [`build_place_on_second_lmb_system`] (same funnel as Enter).
pub fn build_pick_ghost_tile_system(
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    pick: ConstructionMapPick,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
    mut ghost: ResMut<BuildGhostState>,
    mut press: Local<Option<(Vec2, f32)>>,
) {
    ghost.locked_this_frame = false;

    if strip.active == ToolContext::None {
        ghost.unlock_to_preview();
        return;
    }
    if matches!(
        tool.tool,
        BuildTool::Zone(_)
            | BuildTool::Demolish
            | BuildTool::Road(_)
            | BuildTool::Rail(_)
            | BuildTool::PowerLine(_)
            | BuildTool::Firebreak
    ) {
        ghost.unlock_to_preview();
        return;
    }
    if let BuildTool::Building(id) = tool.tool {
        ghost.footprint = id.footprint();
    } else if let BuildTool::Defense(kind) = tool.tool {
        ghost.footprint = kind.footprint();
    }

    // Industry/Civil rail arms Building(Factory) before a catalog pick — do not place a
    // ghost until building_intent is set (picker/submenu selection).
    if matches!(tool.tool, BuildTool::Building(_)) && tool.building_intent.is_none() {
        ghost.unlock_to_preview();
        return;
    }
    // Military picker open with no defense row armed — no ghost.
    if matches!(tool.tool, BuildTool::None) {
        ghost.unlock_to_preview();
        return;
    }

    let Ok(window) = pick.win.single() else {
        return;
    };
    let Some(cursor_px) = window.cursor_position() else {
        return;
    };
    // Same predicate as before when the gate exists. `egui_wants` is unused on that path.
    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }
    if !construction_map_pick_allowed(
        pointer_gate.as_deref(),
        false,
        pick.map_vp.as_ref(),
        cursor_px,
    ) {
        return;
    }

    // Voxel view: the gate's ray hit is the pick (never the top-down ortho).
    let world_xy = match pointer_gate
        .as_deref()
        .and_then(SimulationMapPointerGate::pixel_world_pick)
    {
        Some(hit) => hit,
        None => {
            let proj = ConstructionMapProjection::resolve(
                pick.authority.as_deref(),
                pick.desired.as_ref(),
                pick.map_vp.as_ref(),
                pick.params.as_ref(),
            );
            proj.cursor_world_xy_with_ortho(cursor_px, pick.ortho.as_ref())
                .or_else(|| proj.cursor_world_xy(cursor_px))
        }
    };
    let Some(world_xy) = world_xy else {
        return;
    };

    let x = world_xy.x.floor().max(0.0) as u32;
    let z = world_xy.y.floor().max(0.0) as u32;
    let tile = BuildSiteTile { x, z };
    let _conform_y = conform_world_y(
        world_xy.x,
        world_xy.y,
        pick.params.as_ref(),
        pick.height_grid(),
    );

    let building_two_click = tool.tool.uses_two_click_place();

    if building_two_click {
        match ghost.placement_mode {
            BuildPlacementMode::Place => {
                // Preview: ghost follows cursor.
                ghost.origin = Some(tile);
                if buttons.just_pressed(MouseButton::Left) {
                    ghost.placement_mode = BuildPlacementMode::Adjust;
                    ghost.origin = Some(tile);
                    ghost.drag_active = false;
                    ghost.locked_this_frame = true;
                    ghost.last_click_screen = Some(cursor_px);
                    ghost.last_action_tile = Some(tile);
                    note_mode_lock();
                }
            }
            BuildPlacementMode::Adjust => {
                // Locked: origin stays fixed; second LMB handled after validation.
                if buttons.just_released(MouseButton::Left) {
                    ghost.drag_active = false;
                }
            }
        }
        return;
    }

    // Non-building: legacy click / drag origin (zone/road handled above via early return).
    if buttons.just_released(MouseButton::Left) {
        if let Some((start, t0)) = *press {
            let hold = time.elapsed_secs() - t0;
            let dist = cursor_px.distance(start);
            if classify_build_pointer_gesture(dist, hold) == BuildPointerGesture::Click {
                ghost.drag_active = false;
            }
        }
        *press = None;
    }

    if buttons.just_pressed(MouseButton::Left) {
        ghost.origin = Some(tile);
        ghost.drag_active = false;
        *press = Some((cursor_px, time.elapsed_secs()));
        return;
    }

    if buttons.pressed(MouseButton::Left) {
        if let Some((start, t0)) = *press {
            let hold = time.elapsed_secs() - t0;
            let dist = cursor_px.distance(start);
            if classify_build_pointer_gesture(dist, hold) == BuildPointerGesture::Drag {
                ghost.drag_active = true;
                ghost.origin = Some(tile);
            }
        }
    }
}

/// RULE-NARROW-SYSTEMS-v1: placement refresh is staged (≤8 params each).
/// Site → occupancy → water reject → material gates.
fn placement_path_tools_skip(strip: &BuildStripState, tool: &ActiveBuildTool) -> bool {
    strip.active == ToolContext::Roads
        || strip.active == ToolContext::Rail
        || matches!(tool.tool, BuildTool::PowerLine(_))
}

/// Recompute site validity on [`BuildPlacementPreview`] (construction domain only).
pub fn build_refresh_placement_validation_system(
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    config: Option<Res<StrategicRasterConfig>>,
    overlay: Query<&crate::strategic::ChunkStrategicOverlay>,
    mut preview: ResMut<BuildPlacementPreview>,
) {
    if let Some(intent) = tool.building_intent.as_ref() {
        ghost.footprint = FootprintTiles {
            width: intent.footprint.width.max(1),
            depth: intent.footprint.depth.max(1),
        };
    } else if let BuildTool::Building(id) = tool.tool {
        ghost.footprint = id.footprint();
    } else if let BuildTool::Defense(kind) = tool.tool {
        ghost.footprint = kind.footprint();
    } else if strip.is_changed() {
        ghost.footprint = strip.active.footprint_for_tool();
    }
    if placement_path_tools_skip(&strip, &tool) {
        return;
    }

    let Some(origin) = ghost.origin else {
        preview.report = crate::strategic::SitePlacementValidation::default();
        return;
    };

    preview.report =
        evaluate_ghost_site_placement(origin, ghost.footprint, config.as_deref(), &overlay);

    let _stamped = super::preview_commit_guard::stamp_empty_footprint(
        &mut preview.report,
        ghost.footprint.width,
        ghost.footprint.depth,
    );
}

/// Occupation overlap reject (strategic occupation surface).
pub fn build_apply_occupancy_reject_system(
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    registry: Res<BuildingDefinitionRegistry>,
    occupation: Option<Res<crate::strategic::TileOccupationBook>>,
    pending: Option<Res<PendingConstructionQueue>>,
    mut preview: ResMut<BuildPlacementPreview>,
) {
    if placement_path_tools_skip(&strip, &tool) {
        return;
    }
    let Some(origin) = ghost.origin else {
        return;
    };
    let Some(book) = occupation.as_deref() else {
        return;
    };
    if let Some(snapshot) = placement_snapshot_for_building(&tool, &registry, &ghost, origin) {
        if book.would_overlap(&snapshot.weights) {
            preview.report.valid = false;
            preview.report.allows_commit = false;
            if !preview
                .report
                .errors
                .iter()
                .any(|e| e == "weighted_overlap")
            {
                preview.report.errors.push("weighted_overlap".to_string());
            }
        }
    } else if pending.is_some() {
        let tiles = super::preview_commit_guard::oriented_rect_tiles(
            origin.x as i32,
            origin.z as i32,
            ghost.footprint.width,
            ghost.footprint.depth,
            ghost.rotation_quarter_turns,
        );
        super::preview_commit_guard::apply_rect_occupancy_reject(
            &mut preview.report,
            book,
            &tiles,
        );
    }
}

/// Water footprint reject — reads water domain via [`WaterTileProbe`] only (KEEP bundle).
pub fn build_apply_water_reject_system(
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    water_probe: crate::systems::water::WaterTileProbe,
    mut preview: ResMut<BuildPlacementPreview>,
) {
    if placement_path_tools_skip(&strip, &tool) || ghost.origin.is_none() {
        return;
    }
    if !water_probe.available() {
        return;
    }
    let Some(origin) = ghost.origin else {
        return;
    };
    let tiles = super::preview_commit_guard::oriented_rect_tiles(
        origin.x as i32,
        origin.z as i32,
        ghost.footprint.width,
        ghost.footprint.depth,
        ghost.rotation_quarter_turns,
    );
    let catalog_id = tool
        .building_intent
        .as_ref()
        .and_then(|intent| intent.catalog_id.as_deref());
    super::preview_commit_guard::apply_water_footprint_reject(
        &mut preview.report,
        catalog_id,
        &tiles,
        |x, z| water_probe.is_water(x, z),
    );
    super::preview_commit_guard::apply_dam_banks_reject(
        &mut preview.report,
        catalog_id,
        &tiles,
        |x, z| water_probe.is_water(x, z),
    );
}

/// Concrete + deployable staging gates (logistics surfaces only).
pub fn build_apply_material_gates_system(
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    concrete_nodes: Query<&ResourceFlowNode>,
    staging: Option<Res<SiteStagingStock>>,
    pending_staging: Option<Res<PendingSiteStagingDebits>>,
    ledger: Option<Res<InTransitLedger>>,
    mut preview: ResMut<BuildPlacementPreview>,
) {
    if placement_path_tools_skip(&strip, &tool) || ghost.origin.is_none() {
        return;
    }
    let available = total_available_concrete(concrete_nodes.iter());
    apply_defensive_wall_concrete_gate(tool.tool, ghost.footprint, available, &mut preview.report);

    if let (Some(staging), Some(pending), Some(ledger)) = (
        staging.as_deref(),
        pending_staging.as_deref(),
        ledger.as_deref(),
    ) {
        apply_deployable_staging_gate(
            tool.tool,
            ghost.footprint,
            staging,
            pending,
            ledger,
            &mut preview.report,
        );
    } else if matches!(
        tool.tool,
        BuildTool::Defense(DefenseKind::DragonTeeth | DefenseKind::Minefield)
    ) {
        preview.report.allows_commit = false;
        if !preview
            .report
            .errors
            .iter()
            .any(|e| e == INSUFFICIENT_STAGED_ERROR)
        {
            preview
                .report
                .errors
                .push(INSUFFICIENT_STAGED_ERROR.to_string());
        }
    }
}

/// Shift+left-click queues a valid ghost as a pending blueprint (no immediate commit).
pub fn build_queue_blueprint_on_shift_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    mut pending: ResMut<PendingConstructionQueue>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if !super::build_tool_authority::shift_lmb_queues_building_blueprint(tool.tool) {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if !(keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)) {
        return;
    }
    let Some(origin) = ghost.origin else {
        return;
    };
    if !preview.report.allows_commit {
        return;
    }
    let archetype = resolve_site_archetype(&tool, &registry, &strip);
    let catalog_id = tool
        .building_intent
        .as_ref()
        .and_then(|i| i.catalog_id.clone());
    pending.push(PendingBuildBlueprint {
        kind: PendingEntryKind::BuildSite,
        label: format!("{},{}", origin.x, origin.z),
        archetype,
        origin,
        footprint: ghost.footprint,
        layer: LayerType::Surface,
        rotation_quarter_turns: ghost.rotation_quarter_turns,
        mirror_x: ghost.mirror_x,
        approved: false,
        catalog_id,
    });
}

/// Alt+drag paints valid tiles into the pending blueprint queue.
pub fn build_drag_paint_queue_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    mut pending: ResMut<PendingConstructionQueue>,
    mut last_tile: Local<Option<BuildSiteTile>>,
) {
    if strip.active == ToolContext::None {
        *last_tile = None;
        return;
    }
    if !super::build_tool_authority::shift_lmb_applies_to_active_tool(tool.tool) {
        *last_tile = None;
        return;
    }
    if !(keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)) {
        *last_tile = None;
        return;
    }
    if !buttons.pressed(MouseButton::Left) || !ghost.drag_active {
        return;
    }
    let Some(origin) = ghost.origin else {
        return;
    };
    if !preview.report.allows_commit {
        return;
    }
    if last_tile.is_some_and(|tile| tile == origin) {
        return;
    }
    *last_tile = Some(origin);
    let archetype = resolve_site_archetype(&tool, &registry, &strip);
    let catalog_id = tool
        .building_intent
        .as_ref()
        .and_then(|i| i.catalog_id.clone());
    pending.push(PendingBuildBlueprint {
        kind: PendingEntryKind::BuildSite,
        label: format!("paint:{},{}", origin.x, origin.z),
        archetype,
        origin,
        footprint: ghost.footprint,
        layer: LayerType::Surface,
        rotation_quarter_turns: ghost.rotation_quarter_turns,
        mirror_x: ghost.mirror_x,
        approved: false,
        catalog_id,
    });
}

pub fn build_rotate_mirror_ghost_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if keys.just_pressed(bindings.rotate_build_ghost) {
        ghost.rotation_quarter_turns = (ghost.rotation_quarter_turns + 1) % 4;
    }
    if keys.just_pressed(bindings.mirror_build_ghost) {
        ghost.mirror_x = !ghost.mirror_x;
    }
}

/// Ctrl+scroll rotates · Shift+scroll scales while locked (Adjust).
pub fn build_adjust_modifiers_system(
    keys: Res<ButtonInput<KeyCode>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    scroll: Res<AccumulatedMouseScroll>,
    mut ghost: ResMut<BuildGhostState>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if !tool.tool.uses_two_click_place() {
        return;
    }
    if ghost.placement_mode != BuildPlacementMode::Adjust {
        return;
    }
    let dy = scroll.delta.y;
    if dy.abs() < f32::EPSILON {
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ctrl {
        // Discrete quarter-turns on wheel notches.
        if dy > 0.0 {
            ghost.rotation_quarter_turns = (ghost.rotation_quarter_turns + 1) % 4;
        } else {
            ghost.rotation_quarter_turns = (ghost.rotation_quarter_turns + 3) % 4;
        }
    } else if shift {
        let step = 0.05 * dy.signum();
        ghost.scale_factor = clamp_scale_factor(ghost.scale_factor + step);
    }
}

/// Commit active building ghost through the single [`CommitConstructionSiteEvent`] funnel.
/// Returns true when a commit message was queued.
fn try_commit_active_building_ghost(
    tool: &ActiveBuildTool,
    strip: &BuildStripState,
    ghost: &mut BuildGhostState,
    preview: &BuildPlacementPreview,
    registry: &BuildingDefinitionRegistry,
    session: &mut ActiveToolSession,
    history: &mut ConstructionHistory,
    events: &mut MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    actor: Entity,
    staging_enabled: bool,
    from_lmb: bool,
    pulse: &mut PlaceFeedbackPulse,
    concrete_nodes: &mut Query<&mut ResourceFlowNode>,
    staging_stock: Option<&SiteStagingStock>,
    pending_staging: Option<&mut PendingSiteStagingDebits>,
) -> bool {
    if staging_enabled {
        return false;
    }
    if !tool.tool.uses_two_click_place() {
        return false;
    }
    let Some(origin) = ghost.origin else {
        return false;
    };
    if !preview.report.allows_commit {
        if preview.report.errors.iter().any(|e| {
            e == INSUFFICIENT_CONCRETE_ERROR
                || e == INSUFFICIENT_STAGED_ERROR
                || e == STOCK_IN_TRANSIT_ERROR
        }) {
            if preview
                .report
                .errors
                .iter()
                .any(|e| e == INSUFFICIENT_CONCRETE_ERROR)
            {
                super::defense_concrete_stock::note_blocked_no_stock();
            } else {
                super::defense_staging_stock::note_blocked_no_stock();
            }
        }
    }
    let occupied = super::preview_commit_guard::report_is_footprint_occupied(&preview.report);
    if !super::preview_commit_guard::may_queue_commit(
        ghost.footprint.width,
        ghost.footprint.depth,
        preview.report.allows_commit,
        occupied,
    ) {
        return false;
    }

    // Debit concrete on in-situ defense (wall/trench/bunker) before sole commit funnel.
    if consume_concrete_on_wall_commit(tool.tool, ghost.footprint, concrete_nodes).is_err() {
        return false;
    }

    // Deployable: enqueue staging debit (FreightDispatch applies try_debit) — never UI ResMut.
    if let (Some(stock), Some(pending)) = (staging_stock, pending_staging) {
        if enqueue_deployable_staging_debit_on_commit(tool.tool, ghost.footprint, stock, pending)
            .is_err()
        {
            return false;
        }
    } else if matches!(
        tool.tool,
        BuildTool::Defense(DefenseKind::DragonTeeth | DefenseKind::Minefield)
    ) {
        super::defense_staging_stock::note_blocked_no_stock();
        return false;
    }

    let catalog_id = tool
        .building_intent
        .as_ref()
        .and_then(|i| i.catalog_id.clone());
    let placement = placement_snapshot_for_building(tool, registry, ghost, origin);
    let pulse_tiles: Vec<(IVec2, f32)> = if let Some(ref snap) = placement {
        snap.weights
            .iter()
            .copied()
            .filter(|(_, w)| *w > 0.001)
            .collect()
    } else {
        use super::building_catalog::FootprintMatrix;
        let ox = origin.x as i32;
        let oz = origin.z as i32;
        let matrix = tool
            .building_intent
            .as_ref()
            .map(|i| i.footprint.clone())
            .unwrap_or_else(|| {
                FootprintMatrix::from_size(ghost.footprint.width, ghost.footprint.depth, true)
            });
        matrix
            .occupied_local_offsets()
            .into_iter()
            .map(|(dx, dz)| (IVec2::new(ox + dx as i32, oz + dz as i32), 1.0))
            .collect()
    };
    let mut footprint = ghost.footprint;
    if let Some((w, d)) = super::preview_commit_guard::oriented_footprint(
        footprint.width,
        footprint.depth,
        ghost.rotation_quarter_turns,
    ) {
        footprint.width = w;
        footprint.depth = d;
    }
    queue_commit_construction_site(
        events,
        actor,
        resolve_site_archetype(tool, registry, strip),
        origin,
        footprint,
        LayerType::Surface,
        catalog_id,
        placement,
    );
    session.record_commit();
    history.queue_site(origin);
    if from_lmb {
        note_lmb_commit();
    }
    // Disposable settle — after successful queue, before FSM unlock (charter DES-P3-PLACE-ANIM).
    pulse.arm(pulse_tiles);
    ghost.unlock_to_preview();
    true
}

/// Second LMB in Adjust → same commit path as Enter (TRIAGE-BUILD-CLICK-PLACE-001).
pub fn build_place_on_second_lmb_system(
    buttons: Res<ButtonInput<MouseButton>>,
    win: Query<&Window, With<PrimaryWindow>>,
    map_vp: Res<SimulationMapViewport>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
    mut commit: TwoClickCommitParams,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut concrete_nodes: Query<&mut ResourceFlowNode>,
    staging_stock: Option<Res<SiteStagingStock>>,
    mut pending_staging: Option<ResMut<PendingSiteStagingDebits>>,
) {
    if commit.strip.active == ToolContext::None {
        return;
    }
    if !commit.tool.tool.uses_two_click_place() {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    // INT-005: the click that locks Preview→Adjust (earlier in the chain, same frame) must not
    // also commit; only a later click on the locked ghost places it (same predicate as
    // `stage_active_ghost_on_lmb_system`).
    if commit.ghost.placement_mode != BuildPlacementMode::Adjust || commit.ghost.locked_this_frame {
        return;
    }

    let Ok(window) = win.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let cursor_px = Vec2::new(cursor.x, cursor.y);
    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }
    if !construction_map_pick_allowed(
        pointer_gate.as_deref(),
        false,
        map_vp.as_ref(),
        cursor_px,
    ) {
        return;
    }

    let _ = try_commit_active_building_ghost(
        commit.tool.as_ref(),
        commit.strip.as_ref(),
        commit.ghost.as_mut(),
        commit.preview.as_ref(),
        commit.registry.as_ref(),
        commit.session.as_mut(),
        commit.history.as_mut(),
        &mut events,
        commit.actor.0,
        commit.staging.enabled,
        true,
        commit.pulse.as_mut(),
        &mut concrete_nodes,
        staging_stock.as_deref(),
        pending_staging.as_deref_mut(),
    );
}

pub fn build_clear_pending_queue_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    strip: Res<BuildStripState>,
    mut pending: ResMut<PendingConstructionQueue>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if keys.just_pressed(bindings.clear_pending_blueprints) {
        pending.clear_unapproved();
    }
}

/// Bound key → approve pending blueprints + commit valid ghost.
pub fn build_confirm_site_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut commit: TwoClickCommitParams,
    mut pending: ResMut<PendingConstructionQueue>,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut commands: Commands,
    mut occupation: Option<ResMut<crate::strategic::TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<crate::strategic::VoxelOccupationBook>>,
    voxel_cfg: Option<Res<crate::strategic::VoxelOccupationConfig>>,
    mut destruction_mask: Option<ResMut<crate::render::pixel_world::DestructionMaskBook>>,
    mut debris_frame: Option<ResMut<crate::render::WorldFireParticleFrame>>,
    staging_stock: Option<Res<SiteStagingStock>>,
    mut pending_staging: Option<ResMut<PendingSiteStagingDebits>>,
    mut queries: ParamSet<(
        Query<&mut ResourceFlowNode>,
        Query<(
            Entity,
            &crate::strategic::ConstructionSite,
            &crate::strategic::PlannedSite,
            &crate::strategic::SiteFootprint,
            Option<&crate::economy::activation::BuildingDefinitionRef>,
        )>,
    )>,
) {
    if commit.strip.active == ToolContext::None {
        return;
    }
    if !keys.just_pressed(bindings.confirm_build_placement) {
        return;
    }

    if matches!(
        commit.tool.tool,
        BuildTool::Road(_) | BuildTool::Rail(_) | BuildTool::PowerLine(_) | BuildTool::Firebreak
    ) {
        return;
    }

    // Buildings: Enter commits the active ghost only (PARAM-002). Pending blueprints stay
    // queued for explicit Build/tray approve — do not batch-drain on Enter.
    let building_tool = commit.tool.tool.uses_two_click_place();
    if !building_tool {
        let batch_approve = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        if batch_approve {
            pending.approve_all();
        }

        for entry in pending.drain_approved() {
            match entry.kind {
                PendingEntryKind::BuildSite => {
                    let placement = entry
                        .catalog_id
                        .as_deref()
                        .and_then(|id| commit.registry.get(id))
                        .map(|def| {
                            parametric_placement_snapshot(
                                &def.footprint,
                                def.family,
                                entry.origin,
                                entry.rotation_quarter_turns,
                                entry.mirror_x,
                                None,
                            )
                        });
                    queue_commit_construction_site(
                        &mut events,
                        commit.actor.0,
                        entry.archetype,
                        entry.origin,
                        entry.footprint,
                        entry.layer,
                        entry.catalog_id.clone(),
                        placement,
                    );
                    commit.history.queue_site(entry.origin);
                }
                PendingEntryKind::ZonePaint(zone) => {
                    let entity = spawn_zone_at_tile(&mut commands, zone, entry.origin);
                    record_zone_spawns(commit.history.as_mut(), vec![entity]);
                }
                PendingEntryKind::Demolish => {
                    let commit_auth = voxel_cfg
                        .as_ref()
                        .is_some_and(|c| c.commit_authority_enabled);
                    let (_n, events) = execute_demolish_at_tile(
                        &mut commands,
                        entry.origin,
                        &queries.p1(),
                        occupation.as_deref_mut(),
                        voxel_occupation.as_deref_mut(),
                        commit_auth,
                        destruction_mask.as_deref_mut(),
                        debris_frame.as_deref_mut(),
                    );
                    record_demolish_execution(commit.history.as_mut(), events);
                }
            }
        }
    }

    if commit.tool.tool == BuildTool::Demolish {
        return;
    }

    let _ = try_commit_active_building_ghost(
        commit.tool.as_ref(),
        commit.strip.as_ref(),
        commit.ghost.as_mut(),
        commit.preview.as_ref(),
        commit.registry.as_ref(),
        commit.session.as_mut(),
        commit.history.as_mut(),
        &mut events,
        commit.actor.0,
        commit.staging.enabled,
        false,
        commit.pulse.as_mut(),
        &mut queries.p0(),
        staging_stock.as_deref(),
        pending_staging.as_deref_mut(),
    );
}

/// Right-click: Adjust → Preview unlock; else clear ghosts + drop tool (GUI-ESC-001 RMB).
pub fn build_cancel_ghost_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut strip: ResMut<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    mut tool: ResMut<ActiveBuildTool>,
    mut path: ResMut<super::roads::ActiveRoadPlacement>,
    mut power: ResMut<super::power_lines::ActivePowerLinePlacement>,
    mut rail: ResMut<super::rail::ActiveRailPlacement>,
    mut zone: ResMut<super::zones::ActiveZonePaint>,
    mut latch: ResMut<super::build_mode::BuildEscCancelLatch>,
) {
    if strip.active == ToolContext::None && tool.tool == BuildTool::None {
        return;
    }
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    // Design: RMB cancels lock → Preview while building/defense tool stays armed.
    if tool.tool.uses_two_click_place() && ghost.placement_mode == BuildPlacementMode::Adjust {
        ghost.unlock_to_preview();
        return;
    }
    ghost.unlock_to_preview();
    path.control_points.clear();
    path.generated_segments.clear();
    power.clear_path();
    rail.control_points.clear();
    rail.generated_segments.clear();
    zone.clear();
    tool.tool = BuildTool::None;
    tool.close_submenus();
    tool.clear_building_intent();
    strip.active = ToolContext::None;
    zone.zone = None;
    latch.awaiting_second_esc_to_drop_tool = false;
}



#[must_use]
pub fn two_click_mode_lock_count() -> u32 {
    TWO_CLICK_MODE_LOCKS.load(Ordering::Relaxed)
}

#[must_use]
pub fn two_click_lmb_commit_count() -> u32 {
    TWO_CLICK_LMB_COMMITS.load(Ordering::Relaxed)
}









#[cfg(test)]
mod input_gesture_tests {
    use super::*;

    #[test]
    fn sub_threshold_motion_is_click_not_drag() {
        assert_eq!(
            classify_build_pointer_gesture(3.0, 0.05),
            BuildPointerGesture::Click
        );
    }

    #[test]
    fn distance_threshold_promotes_drag() {
        assert_eq!(
            classify_build_pointer_gesture(BUILD_POINTER_DRAG_MIN_DISTANCE_PX, 0.02),
            BuildPointerGesture::Drag
        );
    }

    #[test]
    fn slow_micro_move_stays_click() {
        assert_eq!(
            classify_build_pointer_gesture(2.0, BUILD_POINTER_DRAG_MAX_CLICK_SECS),
            BuildPointerGesture::Click
        );
    }

    #[test]
    fn slow_large_move_becomes_drag() {
        assert_eq!(
            classify_build_pointer_gesture(4.0, BUILD_POINTER_DRAG_MAX_CLICK_SECS + 0.05),
            BuildPointerGesture::Drag
        );
    }
}

#[cfg(test)]
mod parametric_input_tests {












}

/// Ensures a singleton [`BuildGhostRoot`] + [`GhostBuildCursor`] + [`GhostValid`] exists.
pub fn build_sync_ghost_cursor_entity_system(
    mut cmds: Commands,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    root_q: Query<Entity, With<BuildGhostRoot>>,
    mut cursor_q: Query<
        (&mut GhostBuildCursor, &mut GhostValid, &mut Transform),
        With<BuildGhostRoot>,
    >,
) {
    if root_q.is_empty() {
        cmds.spawn((
            Name::new("build_ghost_root"),
            BuildGhostRoot,
            GhostBuildCursor {
                origin: BuildSiteTile { x: 0, z: 0 },
                footprint: ghost.footprint,
            },
            GhostValid(ghost_valid_flag(&preview.report)),
            Transform::default(),
        ));
        return;
    }

    let Ok((mut cur, mut valid, mut xf)) = cursor_q.single_mut() else {
        return;
    };

    cur.footprint = ghost.footprint;
    *valid = GhostValid(ghost_valid_flag(&preview.report));
    if strip.active == ToolContext::None {
        return;
    }
    if let Some(o) = ghost.origin {
        cur.origin = o;
        // Tactical map camera + fallback raster use XY; grid row is stored in `BuildSiteTile::z`.
        xf.translation = Vec3::new(o.x as f32 + 0.5, o.z as f32 + 0.5, 1.0);
    }
}
