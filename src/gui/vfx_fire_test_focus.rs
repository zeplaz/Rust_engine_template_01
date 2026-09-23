//! One-shot camera focus onto harness-seeded fire for CLI `--test vfx|visual|fire|atmosphere`.
//!
//! Not a draw path — gizmos/egui overlays were removed (wrong projection / shaking frames).
//! Display truth for sparks is Core2d raster + heat tint, not this module.

use bevy::prelude::*;

use crate::engine::ActiveTestScene;
use crate::gui::tactical::map_camera::{
    in_simulation_or_editor_map, map_camera_desired_fit_tile_aabb, MainWorldCamera,
    MapCameraDesired, MapCameraDesiredRes, MapCameraSystemSet,
};
use crate::gui::view_authority::commit_map_camera_pose_to_view_authority;
use crate::gui::{SimulationMapTexture, SimulationMapViewport};
use crate::render::{
    FIRE_SPARK_FULL_SCATTER_PX_PER_TILE, FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE,
};
use crate::render::view_runtime::{ViewProjectionAuthority, ViewRuntimeTrace};
use crate::systems::fire::ChunkSurfaceFire;
use crate::terrain::generation::{
    chunk_world_origin, world_generator_enhanced::WorldGenParams, Chunk, ChunkCellMatrix,
};

/// World tile AABB around harness-seeded fire (camera focus only — not drawn).
#[derive(Resource, Clone, Debug, Default)]
pub struct VfxFireTestRegion {
    pub active: bool,
    pub min_world: Vec2,
    pub max_world: Vec2,
    pub seeded_chunk_count: u32,
    /// One-shot pan/zoom onto the burn cluster.
    pub needs_camera_focus: bool,
    /// Set after [`focus_vfx_fire_test_camera_on_region`] commits authority (witness / retry guard).
    pub focus_applied: bool,
}

impl VfxFireTestRegion {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.active && self.max_world.x > self.min_world.x && self.max_world.y > self.min_world.y
    }

    /// First publish only — never re-focus (avoids camera shake vs wheel / smooth).
    pub fn set_bounds(&mut self, min_world: Vec2, max_world: Vec2, seeded_chunk_count: u32) {
        let first = !self.is_valid();
        self.active = true;
        self.min_world = min_world;
        self.max_world = max_world;
        self.seeded_chunk_count = seeded_chunk_count;
        if first {
            self.needs_camera_focus = true;
        }
    }

    /// Keep AABB frozen after first publish (harness may call; no-op once active).
    pub fn update_bounds_keep_focus(
        &mut self,
        _min_world: Vec2,
        _max_world: Vec2,
        _seeded_chunk_count: u32,
    ) {
    }
}

const REGION_PAD_TILES: f32 = 4.0;

#[must_use]
pub fn seeded_fire_region_from_chunks(
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    params: &WorldGenParams,
    min_heat: f32,
) -> Option<(Vec2, Vec2, u32)> {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut count = 0u32;
    for (_, chunk, matrix, fire) in fire_q.iter() {
        if fire.heat < min_heat {
            continue;
        }
        count += 1;
        let origin = chunk_world_origin(chunk.coord, matrix.size);
        let extent = Vec2::new(matrix.size.x.max(1) as f32, matrix.size.y.max(1) as f32);
        min = min.min(origin);
        max = max.max(origin + extent);
    }
    if count == 0 {
        return None;
    }
    let world_max = Vec2::new(params.width.max(1) as f32, params.height.max(1) as f32);
    min = (min - Vec2::splat(REGION_PAD_TILES)).max(Vec2::ZERO);
    max = (max + Vec2::splat(REGION_PAD_TILES)).min(world_max);
    Some((min, max, count))
}

pub fn publish_vfx_fire_test_region(
    params: &WorldGenParams,
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    region: &mut VfxFireTestRegion,
) {
    if region.is_valid() {
        return;
    }
    let Some((min, max, count)) = seeded_fire_region_from_chunks(fire_q, params, 0.02) else {
        return;
    };
    region.set_bounds(min, max, count);
}

fn sync_vfx_fire_test_region_armed(
    scene: Option<Res<ActiveTestScene>>,
    mut region: ResMut<VfxFireTestRegion>,
) {
    let armed = scene.is_some_and(|s| s.0.seeds_fire_overlay());
    if !armed {
        *region = VfxFireTestRegion::default();
        return;
    }
    // Retry until focus runs — OnEnter world-fit can win the first frame before seeds publish.
    if region.is_valid() && !region.focus_applied {
        region.needs_camera_focus = true;
    }
}

fn sync_vfx_fire_test_region_from_fire_system(
    scene: Option<Res<ActiveTestScene>>,
    params: Res<WorldGenParams>,
    fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    mut region: ResMut<VfxFireTestRegion>,
) {
    if !scene.is_some_and(|s| s.0.seeds_fire_overlay()) {
        return;
    }
    if !region.is_valid() {
        publish_vfx_fire_test_region(params.as_ref(), &fire_q, region.as_mut());
    }
}

fn focus_vfx_fire_test_camera_on_region(
    scene: Option<Res<ActiveTestScene>>,
    mut region: ResMut<VfxFireTestRegion>,
    params: Res<WorldGenParams>,
    map_vp: Res<SimulationMapViewport>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut authority: ResMut<ViewProjectionAuthority>,
    mut trace: ResMut<ViewRuntimeTrace>,
    mut desired_res: ResMut<MapCameraDesiredRes>,
    mut cam: Query<(&mut Transform, &mut MapCameraDesired), With<MainWorldCamera>>,
) {
    if !region.needs_camera_focus || !region.is_valid() {
        return;
    }
    if !scene.is_some_and(|s| s.0.seeds_fire_overlay()) {
        return;
    }
    let world_w = params.width.max(1) as f32;
    let world_h = params.height.max(1) as f32;
    let window_px = windows
        .single()
        .ok()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let tex_extent = crate::gui::simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    let mut desired = map_camera_desired_fit_tile_aabb(
        region.min_world,
        region.max_world,
        map_vp.as_ref(),
        window_px,
        tex_extent,
        world_w,
        world_h,
        1.12,
    );
    let z = desired
        .scale
        .x
        .abs()
        .max(FIRE_SPARK_FULL_SCATTER_PX_PER_TILE)
        .max(FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE * 3.0);
    desired.scale = Vec3::splat(z);
    commit_map_camera_pose_to_view_authority(authority.as_mut(), trace.as_mut(), &desired);
    desired_res.0 = desired.clone();
    for (mut t, mut d) in cam.iter_mut() {
        t.translation.x = desired.translation.x;
        t.translation.y = desired.translation.y;
        t.translation.z = desired.translation.z;
        t.scale = Vec3::ONE;
        *d = desired.clone();
    }
    region.needs_camera_focus = false;
    region.focus_applied = true;
}

/// PostUpdate snap so smooth lerp cannot leave VfxSandbox at whole-map zoom after focus.
fn snap_vfx_fire_test_camera_pose(
    scene: Option<Res<ActiveTestScene>>,
    region: Res<VfxFireTestRegion>,
    desired_res: Res<MapCameraDesiredRes>,
    mut cam: Query<(&mut Transform, &mut MapCameraDesired), With<MainWorldCamera>>,
) {
    if !scene.is_some_and(|s| s.0.seeds_fire_overlay()) {
        return;
    }
    if !region.focus_applied || region.needs_camera_focus {
        return;
    }
    let desired = &desired_res.0;
    for (mut t, mut d) in cam.iter_mut() {
        t.translation.x = desired.translation.x;
        t.translation.y = desired.translation.y;
        t.translation.z = desired.translation.z;
        t.scale = Vec3::ONE;
        *d = desired.clone();
    }
}

pub struct VfxFireTestFocusPlugin;

impl Plugin for VfxFireTestFocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VfxFireTestRegion>()
            .add_systems(
                Update,
                (
                    sync_vfx_fire_test_region_armed,
                    sync_vfx_fire_test_region_from_fire_system,
                    focus_vfx_fire_test_camera_on_region
                        .after(MapCameraSystemSet::DeriveDesired)
                        .before(MapCameraSystemSet::Smooth),
                )
                    .chain()
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                PostUpdate,
                snap_vfx_fire_test_camera_pose
                    .after(crate::gui::sync_main_world_camera_viewport_and_projection)
                    .run_if(in_simulation_or_editor_map),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_valid_when_bounds_positive() {
        let mut r = VfxFireTestRegion::default();
        assert!(!r.is_valid());
        r.set_bounds(Vec2::new(400.0, 400.0), Vec2::new(500.0, 500.0), 3);
        assert!(r.is_valid());
        assert!(r.needs_camera_focus);
        r.needs_camera_focus = false;
        r.set_bounds(Vec2::new(410.0, 410.0), Vec2::new(510.0, 510.0), 3);
        assert!(!r.needs_camera_focus);
        assert!(!r.focus_applied);
    }
}
