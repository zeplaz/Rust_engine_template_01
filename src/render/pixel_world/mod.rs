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
//! CameraLocalWeatherSample (RO cloud cover + wind) ──▶ cloud deck sheets
//! WeatherPrecipFrame (RO; streaks scattered over the voxel ground footprint) ──▶ rain / snow streaks
//! PixelWorldCameraFrame + field ──▶ SimulationMapPointerGate::world_pick (PointerGateSet::WorldPick)
//! ```
//!
//! One RTT, one camera active per frame, one pose authority, one pick resource (the pointer gate).
//! Nothing here writes sim state or `RepresentationResult`; every layer reads an existing
//! representation. All geometry lives on [`PIXEL_WORLD_RENDER_LAYER`], seen only by
//! [`PixelWorldCamera`].

use bevy::prelude::*;

mod camera;
mod destruction_mask;
mod field;
mod clouds;
mod ground_material;
mod layers;
mod fire;
mod rain;
mod terrain;
mod water_gpu;

pub use destruction_mask::{
    DestructionMaskBook, DestructionMaskGpu, PresentationUploadRing, MASK_DAMAGED, MASK_DESTROYED,
    MASK_INTACT,
};
pub use ground_material::{PixelWorldGroundMaterial, PixelWorldGroundMaterialHandle};

pub use camera::{
    clamp_pixel_world_zoom, pixel_world_pan_axes, PixelWorldCamera, PixelWorldCameraFrame,
    PixelWorldCameraRig, PIXEL_WORLD_ZOOM_RANGE,
};
pub use field::PixelWorldTerrainField;

/// Render layer of every voxel-view entity (camera, sun, meshes).
pub const PIXEL_WORLD_RENDER_LAYER: usize = 9;

/// Camera culling for the voxel view's trees and fire. **Off by default** while those layers are
/// being verified (operator call, 2026-11-19): every tree chunk is kept and fire is extracted for
/// the whole world at full detail, independent of the camera. Switch on with `PIXEL_WORLD_CULL=1`
/// (or per layer: `PIXEL_WORLD_CULL=trees` / `=fire`) once both are proven, then make it default.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PixelWorldCulling {
    pub trees: bool,
    pub fire: bool,
}

impl PixelWorldCulling {
    #[must_use]
    pub fn from_env() -> Self {
        Self::parse(std::env::var("PIXEL_WORLD_CULL").ok().as_deref())
    }

    #[must_use]
    pub fn parse(v: Option<&str>) -> Self {
        match v.map(str::trim) {
            Some("1" | "true" | "on" | "all") => Self { trees: true, fire: true },
            Some("trees") => Self { trees: true, fire: false },
            Some("fire") => Self { trees: false, fire: true },
            _ => Self::default(),
        }
    }
}

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
        app.insert_resource(PixelWorldCulling::from_env());
        app.add_plugins((
            water_gpu::PixelWorldWaterGpuPlugin,
            ground_material::PixelWorldGroundMaterialPlugin,
        ))
            .init_resource::<PixelWorldTerrainField>()
            .init_resource::<PixelWorldCameraRig>()
            .init_resource::<PixelWorldCameraFrame>()
            .init_resource::<DestructionMaskBook>()
            .init_resource::<DestructionMaskGpu>()
            .init_resource::<PresentationUploadRing>()
            .init_resource::<PixelWorldGroundMaterialHandle>()
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
                    destruction_mask::ensure_destruction_mask_size
                        .after(field::sync_pixel_world_terrain_field)
                        .run_if(crate::gui::in_simulation_or_editor_map),
                    crate::construction::cascade_mask_from_roles_system
                        .after(destruction_mask::ensure_destruction_mask_size)
                        .run_if(crate::gui::in_simulation_or_editor_map),
                    destruction_mask::upload_destruction_mask_gpu
                        .after(crate::construction::cascade_mask_from_roles_system)
                        .run_if(crate::gui::in_simulation_or_editor_map),
                    camera::pixel_world_rotate_input_system
                        .before(crate::gui::MapCameraSystemSet::ApplyInput)
                        .run_if(crate::gui::in_simulation_or_editor_map),
                    camera::pixel_world_mmb_orbit_system
                        .before(crate::gui::MapCameraSystemSet::ApplyInput)
                        .run_if(crate::gui::in_simulation_or_editor_map),
                ),
            )
            .add_systems(PostUpdate, camera::sync_pixel_world_camera.in_set(PixelWorldViewSet::Camera))
            .add_systems(
                PostUpdate,
                (
                    (
                        terrain::sync_pixel_world_terrain,
                        water_gpu::sync_pixel_world_gpu_water,
                    )
                        .chain(),
                    layers::sync_pixel_world_trees,
                    layers::sync_pixel_world_sites,
                    layers::sync_pixel_world_units,
                    layers::sync_pixel_world_ghost,
                    layers::sync_pixel_world_fronts,
                    layers::sync_pixel_world_paths,
                    rain::sync_pixel_world_precip,
                    fire::sync_pixel_world_fire,
                    clouds::sync_pixel_world_clouds,
                )
                    .in_set(PixelWorldViewSet::Layers),
            );
    }
}

#[cfg(test)]
mod culling_tests {
    use super::PixelWorldCulling;

    #[test]
    fn culling_defaults_off_and_parses_per_layer() {
        assert_eq!(PixelWorldCulling::parse(None), PixelWorldCulling { trees: false, fire: false });
        assert_eq!(PixelWorldCulling::parse(Some("1")), PixelWorldCulling { trees: true, fire: true });
        assert_eq!(PixelWorldCulling::parse(Some("trees")), PixelWorldCulling { trees: true, fire: false });
        assert_eq!(PixelWorldCulling::parse(Some("fire")), PixelWorldCulling { trees: false, fire: true });
    }
}
