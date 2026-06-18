//! Map pick + validation refresh + confirm for build strip tools.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::map_egui_projection::ConstructionMapProjection;
use crate::gui::{InputBindings, MapCameraDesired, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use bevy_egui::EguiContexts;
use super::parametric_commit::parametric_placement_snapshot;
use super::queue_commit_construction_site;
use crate::strategic::{
    evaluate_site_placement_at_world_tile, BuildSiteTile, LayerType,
    StrategicRasterConfig,
};

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::building_definitions::BuildingDefinitionRegistry;
use super::history::{record_demolish_execution, record_zone_spawns, ConstructionHistory};
use super::sessions::ActiveToolSession;
use crate::strategic::FootprintTiles;
use super::build_strip::{BuildStripState, ToolContext};
use super::build_state::{BuildCommandActor, BuildGhostRoot, BuildGhostState, BuildPlacementPreview};
use super::demolish::execute_demolish_at_tile;
use super::pending_construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
use super::terrain_conform::conform_world_y;
use super::zones::spawn_zone_at_tile;
use super::staged_ghost_panel::StagedPlacementMode;
use super::GhostBuildCursor;

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
        _ => strip.active.site_archetype(),
    }
}

/// Left-click on map → [`BuildGhostState::origin`] (skips when egui wants the pointer).
pub fn build_pick_ghost_tile_system(
    buttons: Res<ButtonInput<MouseButton>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    mut ghost: ResMut<BuildGhostState>,
    mut egui_ctx: EguiContexts,
) {
    if strip.active == ToolContext::None {
        ghost.origin = None;
        ghost.drag_active = false;
        return;
    }
    if matches!(
        tool.tool,
        BuildTool::Zone(_)
            | BuildTool::Demolish
            | BuildTool::Road(_)
            | BuildTool::Rail(_)
            | BuildTool::PowerLine(_)
    ) {
        ghost.origin = None;
        ghost.drag_active = false;
        return;
    }
    if let BuildTool::Building(id) = tool.tool {
        ghost.footprint = id.footprint();
    }

    let Ok(window) = win.single() else {
        return;
    };
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        return;
    }

    let Some(cursor_px) = window.cursor_position() else {
        return;
    };

    if map_vp.valid && !map_vp.contains_cursor(cursor_px) {
        return;
    }

    let proj = ConstructionMapProjection::resolve(
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    );
    let Some(world_xy) = proj.cursor_world_xy(cursor_px) else {
        return;
    };

    let x = world_xy.x.floor().max(0.0) as u32;
    let z = world_xy.y.floor().max(0.0) as u32;
    let tile = BuildSiteTile { x, z };
    let _conform_y = conform_world_y(world_xy.x, world_xy.y, &params);

    if buttons.just_pressed(MouseButton::Left) {
        if matches!(
            tool.tool,
            BuildTool::Road(_) | BuildTool::Rail(_) | BuildTool::PowerLine(_)
        ) {
            return;
        }
        ghost.origin = Some(tile);
        ghost.drag_active = true;
        return;
    }

    if buttons.pressed(MouseButton::Left) && ghost.drag_active {
        ghost.origin = Some(tile);
    }

    if buttons.just_released(MouseButton::Left) {
        ghost.drag_active = false;
    }
}

/// Recompute [`BuildPlacementPreview`] when ghost origin or tool changes.
pub fn build_refresh_placement_validation_system(
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    registry: Res<BuildingDefinitionRegistry>,
    occupation: Option<Res<crate::strategic::TileOccupationBook>>,
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
    } else if strip.is_changed() {
        ghost.footprint = strip.active.footprint_for_tool();
    }
    if strip.active == ToolContext::Roads
        || strip.active == ToolContext::Rail
        || matches!(tool.tool, BuildTool::PowerLine(_))
    {
        return;
    }

    let Some(origin) = ghost.origin else {
        preview.report = crate::strategic::SitePlacementValidation::default();
        return;
    };

    preview.report = evaluate_site_placement_at_world_tile(
        origin,
        ghost.footprint,
        config.as_deref(),
        &overlay,
    );

    if let (Some(book), Some(snapshot)) = (
        occupation.as_deref(),
        placement_snapshot_for_building(&tool, &registry, &ghost, origin),
    ) {
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
    tool: Res<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    actor: Res<BuildCommandActor>,
    mut pending: ResMut<PendingConstructionQueue>,
    mut session: ResMut<ActiveToolSession>,
    registry: Res<BuildingDefinitionRegistry>,
    mut history: ResMut<ConstructionHistory>,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut commands: Commands,
    mut occupation: Option<ResMut<crate::strategic::TileOccupationBook>>,
    staging: Res<StagedPlacementMode>,
    sites: Query<(
        Entity,
        &crate::strategic::ConstructionSite,
        &crate::strategic::PlannedSite,
        &crate::strategic::SiteFootprint,
        Option<&crate::economy::activation::BuildingDefinitionRef>,
    )>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if !keys.just_pressed(bindings.confirm_build_placement) {
        return;
    }

    if matches!(
        tool.tool,
        BuildTool::Road(_) | BuildTool::Rail(_) | BuildTool::PowerLine(_)
    ) {
        return;
    }

    // PARAM-002: zone/road/demolish keep Shift+Enter batch; buildings commit single ghost only.
    if !matches!(tool.tool, BuildTool::Building(_)) {
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
                        .and_then(|id| registry.get(id))
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
                        actor.0,
                        entry.archetype,
                        entry.origin,
                        entry.footprint,
                        entry.layer,
                        entry.catalog_id.clone(),
                        placement,
                    );
                    history.queue_site(entry.origin);
                }
                PendingEntryKind::ZonePaint(zone) => {
                    let entity = spawn_zone_at_tile(&mut commands, zone, entry.origin);
                    record_zone_spawns(history.as_mut(), vec![entity]);
                }
                PendingEntryKind::Demolish => {
                    let (_n, events) = execute_demolish_at_tile(
                        &mut commands,
                        entry.origin,
                        &sites,
                        occupation.as_deref_mut(),
                    );
                    record_demolish_execution(history.as_mut(), events);
                }
            }
        }
    }

    if tool.tool == BuildTool::Demolish {
        return;
    }

    let BuildTool::Building(_id) = tool.tool else {
        return;
    };

    if staging.enabled {
        return;
    }

    let Some(origin) = ghost.origin else {
        return;
    };
    if !preview.report.allows_commit {
        return;
    }

    let catalog_id = tool
        .building_intent
        .as_ref()
        .and_then(|i| i.catalog_id.clone());
    let placement = placement_snapshot_for_building(&tool, &registry, &ghost, origin);
    queue_commit_construction_site(
        &mut events,
        actor.0,
        resolve_site_archetype(&tool, &registry, &strip),
        origin,
        ghost.footprint,
        LayerType::Surface,
        catalog_id,
        placement,
    );
    session.record_commit();
    history.queue_site(origin);
    ghost.origin = None;
}

/// Right-click clears the active ghost selection.
pub fn build_cancel_ghost_system(
    buttons: Res<ButtonInput<MouseButton>>,
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if buttons.just_pressed(MouseButton::Right) {
        ghost.origin = None;
        ghost.drag_active = false;
    }
}

/// **PARAM-002** — Enter on building tool commits the active ghost only (no pending batch).
#[must_use]
pub fn enter_commits_single_ghost_witness_green() -> bool {
    enter_commits_building_enter_self_check().is_ok()
}

fn enter_commits_building_enter_self_check() -> Result<(), &'static str> {
    use bevy::ecs::message::MessageReader;
    use bevy::prelude::{App, MinimalPlugins, Update};

    use crate::gui::InputBindings;
    use crate::strategic::CommitConstructionSiteEvent;

    use super::build_strip::{BuildStripState, ToolContext};
    use super::build_tool_authority::{BuildingArchetypeId, BuildTool};
    use super::pending_construction::{PendingConstructionQueue, PendingEntryKind};
    use super::staged_ghost_panel::StagedPlacementMode;

    #[derive(Resource, Default)]
    struct CommitEventCount(u32);

    fn count_commit_events(
        mut reader: MessageReader<CommitConstructionSiteEvent>,
        mut count: ResMut<CommitEventCount>,
    ) {
        for _ in reader.read() {
            count.0 = count.0.saturating_add(1);
        }
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<InputBindings>()
        .init_resource::<BuildStripState>()
        .init_resource::<ActiveBuildTool>()
        .init_resource::<BuildGhostState>()
        .init_resource::<BuildPlacementPreview>()
        .init_resource::<PendingConstructionQueue>()
        .init_resource::<StagedPlacementMode>()
        .init_resource::<ActiveToolSession>()
        .init_resource::<BuildingDefinitionRegistry>()
        .init_resource::<ConstructionHistory>()
        .init_resource::<CommitEventCount>()
        .add_message::<CommitConstructionSiteEvent>()
        .add_systems(
            Update,
            (build_confirm_site_system, count_commit_events).chain(),
        );

    {
        let actor = app.world_mut().spawn_empty().id();
        app.world_mut().insert_resource(BuildCommandActor(actor));
        {
            let mut strip = app.world_mut().resource_mut::<BuildStripState>();
            strip.active = ToolContext::Industry;
        }
        {
            let mut tool = app.world_mut().resource_mut::<ActiveBuildTool>();
            tool.tool = BuildTool::Building(BuildingArchetypeId::Factory);
        }
        let footprint = {
            let mut ghost = app.world_mut().resource_mut::<BuildGhostState>();
            ghost.origin = Some(BuildSiteTile { x: 4, z: 6 });
            ghost.footprint
        };
        {
            let mut preview = app.world_mut().resource_mut::<BuildPlacementPreview>();
            preview.report.allows_commit = true;
            preview.report.valid = true;
        }
        {
            let mut pending = app.world_mut().resource_mut::<PendingConstructionQueue>();
            pending.push(PendingBuildBlueprint {
                kind: PendingEntryKind::BuildSite,
                label: "stale_queue".into(),
                archetype: crate::strategic::SiteArchetype::Factory,
                origin: BuildSiteTile { x: 99, z: 99 },
                footprint,
                layer: LayerType::Surface,
                rotation_quarter_turns: 0,
                mirror_x: false,
                approved: true,
                catalog_id: None,
            });
        }
        let confirm = app.world().resource::<InputBindings>().confirm_build_placement;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(confirm);
    }

    app.update();

    let commits = app.world().resource::<CommitEventCount>().0;
    if commits != 1 {
        return Err("expected_single_commit");
    }
    let ghost = app.world().resource::<BuildGhostState>();
    if ghost.origin.is_some() {
        return Err("ghost_should_clear");
    }
    let pending = app.world().resource::<PendingConstructionQueue>();
    if pending.entries.is_empty() {
        return Err("pending_should_not_drain_on_building_enter");
    }
    Ok(())
}

#[cfg(test)]
mod parametric_input_tests {
    use super::enter_commits_single_ghost_witness_green;

    #[test]
    fn enter_commits_single_ghost_witness() {
        assert!(enter_commits_single_ghost_witness_green());
    }
}

/// Ensures a singleton [`BuildGhostRoot`] + [`GhostBuildCursor`] exists and tracks strip state.
pub fn build_sync_ghost_cursor_entity_system(
    mut cmds: Commands,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    root_q: Query<Entity, With<BuildGhostRoot>>,
    mut cursor_q: Query<(&mut GhostBuildCursor, &mut Transform), With<BuildGhostRoot>>,
) {
    if root_q.is_empty() {
        cmds.spawn((
            Name::new("build_ghost_root"),
            BuildGhostRoot,
            GhostBuildCursor {
                origin: BuildSiteTile { x: 0, z: 0 },
                footprint: ghost.footprint,
            },
            Transform::default(),
        ));
        return;
    }

    let Ok((mut cur, mut xf)) = cursor_q.single_mut() else {
        return;
    };

    cur.footprint = ghost.footprint;
    if strip.active == ToolContext::None {
        return;
    }
    if let Some(o) = ghost.origin {
        cur.origin = o;
        // Tactical map camera + fallback raster use XY; grid row is stored in `BuildSiteTile::z`.
        xf.translation = Vec3::new(o.x as f32 + 0.5, o.z as f32 + 0.5, 1.0);
    }
}
