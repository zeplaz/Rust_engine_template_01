//! World raster preview — split into editor-style submodules (`viewport`, `layers`, raster, chrome).
//! Today: one full CPU RGBA pass per update; roadmap: chunk-dirty atlas, composited layers, GPU path.
//! **Phase D contract:** `preview_render_contract.rs` (`PreviewCameraState`, `PreviewRenderTarget`, `PreviewRenderBudget`) — separate from gameplay camera (`base_visual_dev01_plan_status.md` § `phase-d-preview-render-target`).
//! **Runbook:** `prompts/guides/world_preview_runbook_v1.md` (optimization order, U7 invalidation tie-in).
//! **Wave P:** `composite_preview_contract.rs` + `wave_p_readiness.rs` (consumer-only composite preview entry).

mod color_presets;
mod composite_preview_contract;
mod composite_preview_graph;
mod ecology_preview;
mod gpu_preview;
mod interaction;
pub mod layers;
mod minimap;
mod overlays;
mod preview_render_contract;
mod preview_vt4;
mod registry_inspector;
mod render_raster;
mod texture_cache;
mod tile_sampling;
pub mod tilemap_bridge;
mod ui_sidebar;
mod ui_statusbar;
mod ui_toolbar;
mod viewport;
mod wave_p_readiness;
mod window;
mod cache;

pub use composite_preview_graph::{
    chunk_base_rgba_for_graph, composite_chunk_rgba, materialized_chunk_base_rgba,
    sync_composite_preview_graph_resource, CompositePreviewGraph, CompositePreviewGraphResource,
};
pub use composite_preview_contract::{
    canonical_sources_for_layers, wave_p_consumer_contract_passes, CompositePreviewCanonicalSource,
    CompositePreviewLayerBinding, WAVE_P_CONSUMER_ROOTS, WAVE_P_LAYER_BINDINGS,
    WAVE_P_OPEN_BACKLOG_ITEMS,
};
pub use ecology_preview::{
    blend_fire_overlay, ecology_preview_rgba, ecology_sample_for_world_tile, vegetation_preview_rgba,
    EcologyGpuPassKind, EcologyPreviewSample, EcologyRasterChunkRow,
};
pub use gpu_preview::WorldPreviewGpuRuntime;
pub use preview_render_contract::{
    preview_authoritative_surface, preview_gpu_authoritative_run_if, preview_uses_cpu_raster,
    PreviewAuthoritativeSurface, PreviewCameraState, PreviewPathAuthority, PreviewPresentationDebug,
    PreviewRenderBudget, PreviewRenderMode, PreviewRenderTarget,
};
pub use preview_vt4::capture_world_preview_vt4_probe;
pub use layers::PreviewLayers;
pub use overlays::{
    tag_overlay_rgba, tag_overlay_rgba_pool, TAG_OVERLAY_HIGHLIGHT,
};
pub use tile_sampling::{
    cell_tags_for_world_tile, chunk_cell_key_for_world_tile, chunk_cell_layer_at_world_tile,
    slope_grade_for_world_tile, slope_grade_from_world_elevation_neighbors,
};
pub use texture_cache::{
    init_world_preview_texture, sync_world_preview_texture_size, WorldPreviewTexture,
};
pub use cache::WorldPreviewChunkCaches;
pub use tilemap_bridge::tilemap_overlay_index_for_layers;
pub use wave_p_readiness::{gather_wave_p_readiness, wave_p_readiness_passes, WavePReadinessReport};
pub use color_presets::{preview_biome_rgba_for_tile, terrain_family_preview_rgba};
pub use viewport::EditorViewport;
pub use window::display_world_preview;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

/// Ordering: resize → CPU raster → present swap (`base_visual_dev01_plan_status` § phase-d D-3).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum WorldPreviewRasterOrder {
    SyncTextureSize,
    RasterTiles,
    /// Runs after [`WorldPreviewRasterOrder::RasterTiles`] (CPU wrote `SwapImageBuffers::back`).
    PresentSwap,
}

/// Toggles for the World Preview egui window (independent of whether the World Generator panel is open).
#[derive(Resource)]
pub struct WorldPreviewUiState {
    pub window_open: bool,
}

impl Default for WorldPreviewUiState {
    fn default() -> Self {
        Self { window_open: true }
    }
}

/// Registers world preview resources + raster + chrome systems.
pub struct WorldPreviewPlugin;

impl Plugin for WorldPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::render::Stage5ReadinessProfile::FULL_APP);
        app.init_resource::<CompositePreviewGraphResource>();
        preview_render_contract::init_preview_render_contract_resources(app);
        app.init_resource::<crate::gui::SwapImageBuffers>()
            .init_resource::<WorldPreviewTexture>()
            .init_resource::<gpu_preview::WorldPreviewGpuRuntime>()
            .init_resource::<crate::render::WorldPreviewVt4Probe>()
            .init_resource::<EditorViewport>()
            .init_resource::<WorldPreviewUiState>()
            .init_resource::<WorldPreviewChunkCaches>()
            .configure_sets(
                Update,
                (
                    WorldPreviewRasterOrder::RasterTiles.after(WorldPreviewRasterOrder::SyncTextureSize),
                    WorldPreviewRasterOrder::PresentSwap.after(WorldPreviewRasterOrder::RasterTiles),
                ),
            )
            .add_systems(Startup, init_world_preview_texture)
            .add_systems(
                Startup,
                gpu_preview::prefer_gpu_preview_mode_when_renderer_ready,
            )
            .add_systems(
                Update,
                (
                    preview_render_contract::sync_preview_render_contract_system
                        .before(WorldPreviewRasterOrder::SyncTextureSize),
                    composite_preview_graph::sync_composite_preview_graph_resource
                        .after(preview_render_contract::sync_preview_render_contract_system),
                    preview_render_contract::sync_preview_path_authority
                        .after(composite_preview_graph::sync_composite_preview_graph_resource),
                    sync_world_preview_texture_size.in_set(WorldPreviewRasterOrder::SyncTextureSize),
                    render_raster::update_world_preview_texture
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    texture_cache::present_world_preview_swap_after_raster
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    texture_cache::present_world_preview_gpu_swap
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    gpu_preview::sync_world_preview_offscreen_camera
                        .after(preview_render_contract::sync_preview_render_contract_system),
                    gpu_preview::sync_world_preview_offscreen_camera_transform
                        .after(gpu_preview::sync_world_preview_offscreen_camera),
                    gpu_preview::sync_world_preview_gpu_chunk_quads
                        .after(gpu_preview::sync_world_preview_offscreen_camera_transform),
                    preview_vt4::capture_world_preview_vt4_probe
                        .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
                ),
            )
            .add_systems(EguiPrimaryContextPass, display_world_preview);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{IVec2, UVec2};
    use crate::terrain::material::{MaterialDef, MaterialRegistry};
    use crate::terrain::TerrainFamilyRegistry;
    use std::collections::HashMap;

    fn tiny_grass_registry() -> (TerrainFamilyRegistry, MaterialRegistry) {
        let fam_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/terrain_family_registry.example.json");
        let families =
            TerrainFamilyRegistry::load_from_json(fam_path.to_str().unwrap()).unwrap();
        let grass = families.id("Grassland").unwrap();
        let reg = MaterialRegistry {
            schema_version: 2,
            materials: vec![MaterialDef {
                name: "grass_default".into(),
                family: grass,
                tags: vec![],
                properties: serde_json::json!({}),
                preview_color: [10, 20, 30, 255],
            }],
            name_to_id: HashMap::from([("grass_default".into(), crate::terrain::material::MaterialId(0))]),
        };
        (families, reg)
    }

    #[test]
    fn chunk_cell_layer_prefers_matching_chunk() {
        let size = UVec2::new(2, 2);
        let elev = vec![0.1, 0.2, 0.3, 0.4];
        let slices: Vec<(IVec2, UVec2, &[f32])> = vec![(IVec2::ZERO, size, elev.as_slice())];
        assert_eq!(chunk_cell_layer_at_world_tile(0, 0, &slices), Some(0.1));
        assert_eq!(chunk_cell_layer_at_world_tile(1, 0, &slices), Some(0.2));
        assert_eq!(chunk_cell_layer_at_world_tile(0, 1, &slices), Some(0.3));
    }

    #[test]
    fn chunk_cell_key_matches_flat_index() {
        let geom = vec![(IVec2::ZERO, UVec2::new(2, 2))];
        assert_eq!(
            chunk_cell_key_for_world_tile(1, 0, &geom),
            Some(crate::terrain::ChunkCellKey::new(IVec2::ZERO, 1))
        );
    }

    #[test]
    fn preview_uses_material_def_color() {
        let (families, reg) = tiny_grass_registry();
        let grass = families.id("Grassland").unwrap();
        let chunks: Vec<(IVec2, UVec2, &[crate::terrain::material::MaterialId])> = vec![];
        let c = preview_biome_rgba_for_tile(0, 0, grass, &chunks, &reg, Some(&families));
        assert_eq!(c, [10, 20, 30, 255]);
    }

    #[test]
    fn preview_tag_overlay_highlights_match() {
        use crate::terrain::material::TagId;
        let mut ts = crate::terrain::material::TagSet::default();
        ts.insert(TagId(5));
        assert_eq!(tag_overlay_rgba(TagId(5), &ts), TAG_OVERLAY_HIGHLIGHT);
        assert_eq!(tag_overlay_rgba(TagId(4), &ts), [0, 0, 0, 0]);
    }

    #[test]
    fn preview_tag_pool_highlights_overlap() {
        use crate::terrain::material::TagId;
        let mut pool = crate::terrain::material::TagSet::default();
        pool.insert(TagId(4));
        let mut ts = crate::terrain::material::TagSet::default();
        ts.insert(TagId(5));
        assert_eq!(tag_overlay_rgba_pool(&ts, &pool), [0, 0, 0, 0]);
        ts.insert(TagId(4));
        assert_eq!(tag_overlay_rgba_pool(&ts, &pool), TAG_OVERLAY_HIGHLIGHT);
    }
}
