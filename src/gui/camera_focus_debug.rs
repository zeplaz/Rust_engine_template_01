//! Map camera focus → chunk grid + LOD for debug overlays and unified tracing.
//!
//! Uses the same ~64 world-unit chunk spacing assumed by fire visual tests / preview sampling.
//!
//! **Production gate (VISUAL-JANK-LOD-001):** yellow focus / green terrain chunk squares must
//! never paint the Simulation playfield unless an explicit overlay flag is armed
//! (`CAMERA_FOCUS_DEBUG=1` **and** [`CameraFocusDebug::enabled`]). Single authority:
//! [`CameraFocusDebug::lod_paint_active`].

use bevy::math::Isometry2d;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::engine::states::BaseState;
use crate::gui::tactical::map_camera::{in_simulation_or_editor_map, MapCameraDesiredRes};
use crate::gui::view_authority::tactical_camera_world_pose;
use crate::gui::world_representation::WorldRepresentationFrame;
use crate::gui::{ViewAuthoritySystemSet, ViewManager};
use crate::render::{tactical_fire_visual, FireVisualFramesByView};
use crate::render::extraction::sim_visual_extract::{FireVisualFrame, FIRE_VISUAL_ACTIVE_HEAT_EPS};
use crate::terrain::generation::{chunk_world_center, Chunk, ChunkCellMatrix};

/// Approximate world extent (XY) covered by one chunk index step for debug tiling.
/// Default slab size when chunk matrix is unavailable (see test harness `SLAB = 32`).
pub const DEBUG_CHUNK_SPACING_WORLD: f32 = 32.0;

/// Explicit opt-in for LOD chunk debug paint (`CAMERA_FOCUS_DEBUG=1|true|on`).
#[must_use]
pub fn camera_focus_debug_env_armed() -> bool {
    std::env::var("CAMERA_FOCUS_DEBUG")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "on" | "TRUE" | "ON"))
}

/// Dev overlay: camera world XY, derived chunk, LOD band, optional nearest zone id.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraFocusDebug {
    /// Runtime toggle — **inert in Simulation** unless [`camera_focus_debug_env_armed`].
    pub enabled: bool,
    pub world_pos: Vec2,
    pub focus_chunk: IVec2,
    pub region_id: Option<u32>,
    pub lod_band: crate::gui::world_representation::WorldLodBand,
    pub overlay_radius_chunks: i32,
    /// When true, LOD debug tiles scale in world space by `1 / MapCameraDesired.scale.x` so
    /// apparent on-screen size stays closer to constant as the tactical camera zoom changes.
    pub screen_stabilize_lod_overlay: bool,
}

impl Default for CameraFocusDebug {
    fn default() -> Self {
        // Production default OFF. Env arm alone does not paint — still need `enabled`.
        Self {
            enabled: false,
            world_pos: Vec2::ZERO,
            focus_chunk: IVec2::ZERO,
            region_id: None,
            lod_band: crate::gui::world_representation::WorldLodBand::Strategic,
            overlay_radius_chunks: 6,
            screen_stabilize_lod_overlay: false,
        }
    }
}

impl CameraFocusDebug {
    /// Single authority: yellow/green LOD chunk squares may paint.
    #[inline]
    #[must_use]
    pub fn lod_paint_active(self) -> bool {
        self.enabled && camera_focus_debug_env_armed()
    }
}

/// Schedule gate for gizmo LOD draw (pairs with GPU path check in `build_tile_debug_instances`).
pub fn camera_focus_lod_paint_active(debug: Res<CameraFocusDebug>) -> bool {
    debug.lod_paint_active()
}

/// Simulation session: force LOD overlay off unless env armed (no sticky mid-session enable).
pub fn enforce_camera_focus_lod_off_in_simulation(
    base: Res<State<BaseState>>,
    mut debug: ResMut<CameraFocusDebug>,
) {
    if *base.get() != BaseState::Simulation {
        return;
    }
    if !camera_focus_debug_env_armed() {
        debug.enabled = false;
    }
}

pub struct CameraFocusDebugPlugin;

impl Plugin for CameraFocusDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFocusDebug>()
            .add_systems(
                Update,
                enforce_camera_focus_lod_off_in_simulation
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                Update,
                (
                    update_camera_focus_debug
                        .after(ViewAuthoritySystemSet::SyncViewManager)
                        .run_if(camera_focus_lod_paint_active),
                    trace_camera_focus_line
                        .after(update_camera_focus_debug)
                        .run_if(camera_focus_lod_paint_active),
                    draw_sim_focus_debug_overlay
                        .after(trace_camera_focus_line)
                        .run_if(camera_focus_lod_paint_active)
                        .run_if(crate::gui::tile_debug_types::tile_debug_use_gizmos_instead),
                )
                    .chain()
                    .run_if(in_simulation_or_editor_map),
            );
    }
}

/// Chunks that have fire above the active threshold in either GPU instance rows or `chunk_heat`
/// (same union as tile-debug overlay — avoids logs showing `fire_active=0` when `chunk_heat` still has heat).
#[must_use]
pub fn fire_chunk_coords_above_visual_eps(fire: &FireVisualFrame) -> HashSet<IVec2> {
    fire.chunk_coords_with_active_heat()
}

pub fn update_camera_focus_debug(
    desired: Res<MapCameraDesiredRes>,
    view_manager: Res<ViewManager>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    lod_frame: Res<WorldRepresentationFrame>,
    zones: Res<crate::gui::LodZoneRegistry>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    mut debug: ResMut<CameraFocusDebug>,
) {
    // Defense in depth — schedule already `run_if(camera_focus_lod_paint_active)`.
    if !debug.lod_paint_active() {
        return;
    }
    let (world_pos, _) = tactical_camera_world_pose(authority.as_deref(), &view_manager, &desired);
    debug.world_pos = world_pos;
    let default_size = chunks
        .iter()
        .next()
        .map(|(_, m)| m.size)
        .unwrap_or(bevy::math::UVec2::splat(32));
    let spacing = default_size.x.max(default_size.y).max(1) as f32;
    debug.focus_chunk = IVec2::new(
        (world_pos.x / spacing).floor() as i32,
        (world_pos.y / spacing).floor() as i32,
    );
    debug.lod_band = lod_frame.global_band();

    let mut best: Option<(u32, f32)> = None;
    for z in &zones.zones {
        let dx = z.center.x - world_pos.x;
        let dy = z.center.y - world_pos.y;
        let d2 = dx * dx + dy * dy;
        if z.radius <= 0.0 {
            continue;
        }
        if d2 > z.radius * z.radius {
            continue;
        }
        let prev = best.map(|(_, d)| d).unwrap_or(f32::MAX);
        if d2 < prev {
            best = Some((z.zone_id, d2));
        }
    }
    debug.region_id = best.map(|(id, _)| id);
}

pub fn trace_camera_focus_line(
    focus: Res<CameraFocusDebug>,
    fire_by_view: Res<FireVisualFramesByView>,
    mut last_tile: Local<Option<IVec2>>,
    mut last_fire: Local<usize>,
    mut tick: Local<u32>,
) {
    if !focus.lod_paint_active() {
        return;
    }
    let fire = tactical_fire_visual(fire_by_view.as_ref());
    let fire_active = fire_chunk_coords_above_visual_eps(fire).len();
    *tick = tick.wrapping_add(1);
    let tile = focus.focus_chunk;
    let tile_changed = *last_tile != Some(tile);
    let fire_changed = fire_active != *last_fire;
    if tile_changed || fire_changed || *tick % 90 == 0 {
        info!(
            "FOCUS: tile={:?} region={:?} lod={:?} fire_active={} world_xy={:?}",
            tile,
            focus.region_id,
            focus.lod_band,
            fire_active,
            focus.world_pos
        );
        *last_tile = Some(tile);
        *last_fire = fire_active;
    }
}

pub fn draw_sim_focus_debug_overlay(
    debug: Res<CameraFocusDebug>,
    view_manager: Res<ViewManager>,
    desired: Res<MapCameraDesiredRes>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    mut gizmos: Gizmos,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    fire_by_view: Res<FireVisualFramesByView>,
) {
    if !debug.lod_paint_active() {
        return;
    }
    let fire = tactical_fire_visual(fire_by_view.as_ref());
    let mut chunk_set = HashSet::<IVec2>::default();
    let mut chunk_sizes: HashMap<IVec2, bevy::math::UVec2> = HashMap::new();
    for (c, m) in &chunks {
        chunk_set.insert(c.coord);
        chunk_sizes.insert(c.coord, m.size);
    }

    let default_size = chunk_sizes
        .values()
        .next()
        .copied()
        .unwrap_or(bevy::math::UVec2::splat(32));
    let grid_step = Vec2::new(default_size.x as f32, default_size.y as f32);

    let r = debug.overlay_radius_chunks.clamp(1, 12);
    let center = debug.focus_chunk;
    let (_, cam_scale) = tactical_camera_world_pose(authority.as_deref(), &view_manager, &desired);
    let cam_scale = cam_scale.abs().max(0.001);
    let lod_size_mul = match debug.lod_band {
        crate::gui::world_representation::WorldLodBand::LocalTactical => 0.85,
        crate::gui::world_representation::WorldLodBand::Operational => 0.95,
        crate::gui::world_representation::WorldLodBand::Strategic => 1.0,
        crate::gui::world_representation::WorldLodBand::Macro => 1.05,
    };

    for dy in -r..=r {
        for dx in -r..=r {
            let tile = center + IVec2::new(dx, dy);
            let size = chunk_sizes.get(&tile).copied().unwrap_or(default_size);
            let pos = chunk_world_center(tile, size);
            let is_focus = tile == center;
            let mut extent = Vec2::new(size.x as f32, size.y as f32) * lod_size_mul;
            if is_focus {
                extent *= 1.08;
            }
            if debug.screen_stabilize_lod_overlay {
                extent /= cam_scale;
                extent = extent.clamp(Vec2::splat(6.0), grid_step * 1.25);
            }
            let color = if is_focus {
                Color::srgb(0.95, 0.85, 0.15)
            } else if chunk_set.contains(&tile) {
                Color::srgb(0.2, 0.75, 0.25)
            } else {
                Color::srgb(0.12, 0.12, 0.14)
            };
            gizmos.rect_2d(Isometry2d::from_translation(pos), extent, color);
        }
    }

    for row in &fire.instances {
        if row.heat() < FIRE_VISUAL_ACTIVE_HEAT_EPS {
            continue;
        }
        let pos = Vec2::new(row.world_xyz_radius.x, row.world_xyz_radius.y);
        let heat = row.heat().clamp(0.0, 1.0);
        let mut marker = Vec2::splat(3.0 + heat * 5.0);
        if debug.screen_stabilize_lod_overlay {
            marker /= cam_scale;
            marker = marker.clamp(Vec2::splat(2.0), Vec2::splat(12.0));
        }
        gizmos.rect_2d(
            Isometry2d::from_translation(pos),
            marker,
            Color::srgb(1.0, 0.15, 0.12),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lod_paint_inactive_without_env_even_if_enabled() {
        // Env not set in unit test → enabled alone must not paint.
        let mut d = CameraFocusDebug::default();
        d.enabled = true;
        assert!(!d.lod_paint_active());
        assert!(!camera_focus_debug_env_armed());
    }

    #[test]
    fn lod_paint_inactive_when_disabled() {
        let d = CameraFocusDebug::default();
        assert!(!d.enabled);
        assert!(!d.lod_paint_active());
    }
}
