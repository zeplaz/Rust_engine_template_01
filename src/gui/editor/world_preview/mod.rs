//! World raster preview — split into editor-style submodules (`viewport`, `layers`, raster, chrome).
//! Today: one full CPU RGBA pass per update; roadmap: chunk-dirty atlas, composited layers, GPU path.
//! **Phase D contract:** `preview_render_contract.rs` (`PreviewCameraState`, `PreviewRenderTarget`, `PreviewRenderBudget`) — separate from gameplay camera (`base_visual_dev01_plan_status.md` § `phase-d-preview-render-target`).
//! **Runbook:** `prompts/guides/world_preview_runbook_v1.md` (optimization order, U7 invalidation tie-in).
//! **Wave P:** `composite_preview_contract.rs` + `wave_p_readiness.rs` (consumer-only composite preview entry).
//! **CLN-MOD-003:** chrome + plugin live in siblings; render_contract / raster already extracted.

mod cache;
mod chrome;
mod color_presets;
mod composite_preview_contract;
mod composite_preview_graph;
mod ecology_preview;
mod gpu_preview;
mod interaction;
pub mod layers;
mod minimap;
mod overlays;
mod plugin;
mod preview_lifecycle;
mod preview_readiness;
mod preview_render_contract;
mod preview_render_state;
mod preview_vt4;
mod registry_interchange;
mod registry_inspector;
mod render_raster;
mod render_target_barrier;
mod texture_cache;
mod tile_sampling;
pub mod tilemap_bridge;
mod ui_sidebar;
mod ui_statusbar;
mod ui_toolbar;
mod ui_wp_coder_a_witness;
mod viewport;
mod viewport_authority;
mod viewport_suggestion;
mod wave_p_readiness;
mod wave_p_witness;
mod window;

pub use chrome::{
    d02_layout_witness, d02_layout_witness_hd_baseline_sheet_closed, d02_map_area_fraction,
    d02_sidebar_max_width_px, d04_layout_witness, d04_sheet_width_px, d07_inset_side_px,
    d07_layout_witness, dismiss_world_gen_preview_chrome, open_world_gen_chrome_on_new_world_setup,
    open_world_gen_workspace, world_gen_chrome_may_render, world_gen_editor_chrome_allowed,
    world_gen_flow_expects_chrome, world_gen_ux_app_active, world_preview_chrome_active,
    world_preview_pipeline_enabled, world_preview_unified_workspace, WorldPreviewUiState,
    D02_HD_BASELINE_H, D02_HD_BASELINE_W, D02_MAP_MIN_AREA_FRAC, D02_SIDEBAR_MIN_W,
    D02_STATUS_H_PX, D02_TOOLBAR_H_PX, D04_MAP_DIM_ALPHA, D04_SHEET_WIDTH_FRAC,
    D04_SHEET_WIDTH_MAX, D04_SHEET_WIDTH_MIN, D07_INSET_SIDE_DEFAULT, D07_INSET_SIDE_MAX,
    D07_INSET_SIDE_MIN, WORLD_PREVIEW_UNIFIED_WORKSPACE,
};
pub use cache::WorldPreviewChunkCaches;
pub use color_presets::{
    height_to_color, moisture_to_color, preview_biome_rgba_for_tile, temperature_to_color,
    terrain_family_preview_rgba,
};
pub use composite_preview_contract::{
    canonical_sources_for_layers, wave_p_consumer_contract_passes, CompositePreviewCanonicalSource,
    CompositePreviewLayerBinding, WAVE_P_CONSUMER_ROOTS, WAVE_P_LAYER_BINDINGS,
    WAVE_P_OPEN_BACKLOG_ITEMS,
};
pub use composite_preview_graph::{
    chunk_base_rgba_for_graph, composite_chunk_rgba, materialized_chunk_base_rgba,
    sync_composite_preview_graph_resource, CompositePreviewGraph, CompositePreviewGraphResource,
};
pub use ecology_preview::{
    blend_fire_overlay, count_chunks_with_topology_tint_bias, count_distinct_topology_visible_rgba,
    ecology_preview_rgba, ecology_sample_for_world_tile, preview_samples_from_topology_kinds,
    topology_kind_tint_modulator, topology_tint_bias_for_kinds, vegetation_preview_rgba,
    EcologyGpuPassKind, EcologyPreviewSample, EcologyRasterChunkRow,
};
pub use gpu_preview::{WorldPreviewGpuCamera, WorldPreviewGpuRuntime};
pub use layers::PreviewLayers;
pub use overlays::{tag_overlay_rgba, tag_overlay_rgba_pool, TAG_OVERLAY_HIGHLIGHT};
pub use plugin::WorldPreviewPlugin;
pub use preview_lifecycle::{
    advance_world_preview_lifecycle_system, note_world_preview_present_committed,
    note_world_preview_raster_wrote, PreviewLifecyclePhase, WorldPreviewLifecycle,
    WorldPreviewLifecycleSignals,
};
pub use preview_readiness::{
    classify_world_preview_readiness, compute_world_preview_ready, preview_camera_initialized,
    preview_projection_computed, preview_render_target_exists, prime_world_preview_editor_camera,
    sync_world_preview_ready, world_generation_complete, PreviewState, WorldPreviewReady,
    WorldPreviewReadinessDiagnostics,
};
pub use preview_render_contract::{
    init_preview_render_contract_resources, preview_authoritative_surface,
    preview_gpu_authoritative_run_if, preview_uses_cpu_raster, PreviewAuthoritativeSurface,
    PreviewCameraState, PreviewPathAuthority, PreviewPresentationDebug, PreviewRenderBudget,
    PreviewRenderMode, PreviewRenderTarget,
};
pub use preview_render_state::PreviewRenderState;
pub use preview_vt4::capture_world_preview_vt4_probe;
pub use registry_interchange::{
    material_registry_interchange_path, open_registry_interchange_in_desktop_shell,
    tag_registry_interchange_path,
};
pub use render_target_barrier::{
    committed_render_target_handle, sync_world_preview_render_viewport_contract,
    try_commit_world_preview_render_target, PendingRenderTargetBind, WorldPreviewGpuResizeQueue,
    WorldPreviewRenderTargetBindBarrier, WorldPreviewRenderTargetRegistry,
    WorldPreviewRenderViewportContract, WorldPreviewViewportEvent,
};
pub use texture_cache::{
    init_world_preview_texture, sync_world_preview_texture_size, WorldPreviewTexture,
};
pub use tile_sampling::{
    cell_tags_for_world_tile, chunk_cell_key_for_world_tile, chunk_cell_layer_at_world_tile,
    slope_grade_for_world_tile, slope_grade_from_world_elevation_neighbors,
};
pub use tilemap_bridge::tilemap_overlay_index_for_layers;
pub use ui_wp_coder_a_witness::refresh_coder_a_ui_wp_wave_p_witness;
pub use viewport::EditorViewport;
pub use viewport_authority::WorldPreviewViewportAuthority;
pub use viewport_suggestion::write_world_preview_viewport_request;
pub use wave_p_readiness::{gather_wave_p_readiness, wave_p_readiness_passes, WavePReadinessReport};
pub use wave_p_witness::{
    build_d02_layout_witness, build_d04_layout_witness, build_d07_layout_witness,
    build_ui_phase4_wp_tail_witness, build_wave_p_witness_payload, WavePLiveProofState,
    WAVE_P_LIVE_JSON,
};
pub(crate) use window::display_world_preview;

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
