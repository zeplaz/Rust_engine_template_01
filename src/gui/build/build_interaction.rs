//! Map pick + validation refresh + confirm for build strip tools.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::map_camera::{primary_cursor_world_xy, MainWorldCamera};
use crate::gui::SimulationMapViewport;
use bevy_egui::EguiContexts;
use crate::gui::input_bindings::InputBindings;
use super::queue_commit_construction_site;
use crate::strategic::{
    evaluate_site_placement_at_world_tile, BuildSiteTile, LayerType, StrategicRasterConfig,
};

use super::build_strip::{BuildStripState, ToolContext};
use super::build_state::{BuildCommandActor, BuildGhostRoot, BuildGhostState, BuildPlacementPreview};
use super::GhostBuildCursor;

/// Left-click on map → [`BuildGhostState::origin`] (skips when egui wants the pointer).
pub fn build_pick_ghost_tile_system(
    buttons: Res<ButtonInput<MouseButton>>,
    win: Query<&Window, With<PrimaryWindow>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
    map_vp: Res<SimulationMapViewport>,
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    mut egui_ctx: EguiContexts,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if strip.active == ToolContext::None {
        ghost.origin = None;
        return;
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

    let Ok((cam, cam_xf)) = cam_q.single() else {
        return;
    };

    let Some(world_xy) = primary_cursor_world_xy(cam, cam_xf, window, cursor_px) else {
        return;
    };

    let x = world_xy.x.floor().max(0.0) as u32;
    let z = world_xy.y.floor().max(0.0) as u32;
    ghost.origin = Some(BuildSiteTile { x, z });
}

/// Recompute [`BuildPlacementPreview`] when ghost origin or tool changes.
pub fn build_refresh_placement_validation_system(
    strip: Res<BuildStripState>,
    mut ghost: ResMut<BuildGhostState>,
    config: Option<Res<StrategicRasterConfig>>,
    overlay: Query<&crate::strategic::ChunkStrategicOverlay>,
    mut preview: ResMut<BuildPlacementPreview>,
) {
    if strip.is_changed() {
        ghost.footprint = strip.active.footprint_for_tool();
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
}

/// Bound key → [`queue_commit_construction_site`] when placement allows.
pub fn build_confirm_site_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    actor: Res<BuildCommandActor>,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    let Some(origin) = ghost.origin else {
        return;
    };
    if !preview.report.allows_commit {
        return;
    }

    if keys.just_pressed(bindings.confirm_build_placement) {
        queue_commit_construction_site(
            &mut events,
            actor.0,
            strip.active.site_archetype(),
            origin,
            ghost.footprint,
            LayerType::Surface,
        );
    }
}

/// Ensures a singleton [`BuildGhostRoot`] + [`GhostBuildCursor`] exists and tracks strip state.
pub fn build_sync_ghost_cursor_entity_system(
    mut cmds: Commands,
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    root_q: Query<Entity, With<BuildGhostRoot>>,
    mut cursor_q: Query<&mut GhostBuildCursor, With<BuildGhostRoot>>,
) {
    if root_q.is_empty() {
        cmds.spawn((
            Name::new("build_ghost_root"),
            BuildGhostRoot,
            GhostBuildCursor {
                origin: BuildSiteTile { x: 0, z: 0 },
                footprint: ghost.footprint,
            },
        ));
        return;
    }

    let Ok(mut cur) = cursor_q.single_mut() else {
        return;
    };

    cur.footprint = ghost.footprint;
    if strip.active == ToolContext::None {
        return;
    }
    if let Some(o) = ghost.origin {
        cur.origin = o;
    }
}
