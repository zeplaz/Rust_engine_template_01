//! **PixelWorld3D** — the voxel world view (`WorldPresentationKind::PixelWorld3D`, V toggles).
//!
//! Charter: `src/dev/plan_dual_view_pixel_world_v1.md` (consumer under `PIXEL_WORLD_VIEW_SCAFFOLD`).
//!
//! ```text
//! WorldPresentationMode ──▶ which camera renders the map RTT (MainWorldCamera 2D | PixelWorldCamera)
//! ViewProjectionAuthority (WorldMain) ──▶ MapCameraDesiredRes ──▶ PixelWorldCamera (+ Q/E yaw rig)
//! WorldGenDenseTerrainCache + WorldWaterGrid ──▶ PixelWorldTerrainField ──▶ voxel ground / water
//! TileWorldFallbackState.image (the top-down terrain raster) ──▶ ground texture
//! VegetationExtractFrame ──▶ voxel trees     sites ──▶ columns
//! ConstructionVisualRequests ──▶ ghost footprint / massing, draped road + rail ribbons
//! FrontOverlaySources (top-down front builders) ──▶ draped fronts
//! PixelWorldCameraFrame + field ──▶ SimulationMapPointerGate::world_pick (PointerGateSet::WorldPick)
//! ```
//!
//! One RTT, one camera active per frame, one pose authority, one pick resource (the pointer gate).
//! Nothing here writes sim state or `RepresentationResult`; every layer reads an existing
//! representation. All geometry lives on [`PIXEL_WORLD_RENDER_LAYER`], seen only by
//! [`PixelWorldCamera`].

use bevy::prelude::*;

mod camera;
mod field;
mod layers;
mod terrain;

pub use camera::{
    clamp_pixel_world_zoom, pixel_world_pan_axes, PixelWorldCamera, PixelWorldCameraFrame,
    PixelWorldCameraRig, PIXEL_WORLD_ZOOM_RANGE,
};
pub use field::{PixelWorldTerrainField, PIXEL_WORLD_HEIGHT_SCALE};

/// Render layer of every voxel-view entity (camera, sun, meshes).
pub const PIXEL_WORLD_RENDER_LAYER: usize = 9;

/// Tactical XY (x = column, y = row, +Y north) at height `h` → 3D (`+Y` up, north = `-Z`).
#[inline]
#[must_use]
pub fn tile_to_world3(xy: Vec2, h: f32) -> Vec3 {
    Vec3::new(xy.x, h, -xy.y)
}

/// Update / PostUpdate stages of the voxel view.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PixelWorldViewSet {
    /// Field rebuild from the terrain grids (Update).
    Field,
    /// Camera fit + [`PixelWorldCameraFrame`] (PostUpdate, `ApplyCameraProjection`).
    Camera,
    /// Terrain / trees / sites / ghost / fronts meshes (PostUpdate, after the camera fit).
    Layers,
}

pub struct PixelWorldViewPlugin;

impl Plugin for PixelWorldViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PixelWorldTerrainField>()
            .init_resource::<PixelWorldCameraRig>()
            .init_resource::<PixelWorldCameraFrame>()
            .configure_sets(
                PostUpdate,
                (
                    PixelWorldViewSet::Camera
                        .in_set(crate::gui::SimulationViewportSyncSet::ApplyCameraProjection),
                    PixelWorldViewSet::Layers.after(PixelWorldViewSet::Camera),
                ),
            )
            .add_systems(
                Update,
                (
                    camera::spawn_pixel_world_camera.run_if(crate::gui::in_simulation_or_editor_map),
                    field::sync_pixel_world_terrain_field.in_set(PixelWorldViewSet::Field),
                    camera::pixel_world_rotate_input_system
                        .before(crate::gui::MapCameraSystemSet::ApplyInput)
                        .run_if(crate::gui::in_simulation_or_editor_map),
                ),
            )
            .add_systems(PostUpdate, camera::sync_pixel_world_camera.in_set(PixelWorldViewSet::Camera))
            .add_systems(
                PostUpdate,
                (
                    terrain::sync_pixel_world_terrain,
                    layers::sync_pixel_world_trees,
                    layers::sync_pixel_world_sites,
                    layers::sync_pixel_world_ghost,
                    layers::sync_pixel_world_fronts,
                    layers::sync_pixel_world_paths,
                )
                    .in_set(PixelWorldViewSet::Layers),
            );
    }
}
