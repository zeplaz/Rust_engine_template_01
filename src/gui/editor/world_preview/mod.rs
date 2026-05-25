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
mod preview_lifecycle;
mod preview_readiness;
mod preview_render_state;
mod preview_render_contract;
mod preview_vt4;
mod registry_interchange;
mod render_target_barrier;
mod registry_inspector;
mod render_raster;
mod texture_cache;
mod tile_sampling;
pub mod tilemap_bridge;
mod ui_sidebar;
mod ui_statusbar;
mod ui_toolbar;
mod viewport;
mod viewport_authority;
mod viewport_suggestion;
mod wave_p_live_proof;
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
pub use gpu_preview::{WorldPreviewGpuCamera, WorldPreviewGpuRuntime};
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
pub use preview_render_state::PreviewRenderState;
pub use preview_render_contract::{
    init_preview_render_contract_resources, preview_authoritative_surface,
    preview_gpu_authoritative_run_if, preview_uses_cpu_raster, PreviewAuthoritativeSurface,
    PreviewCameraState, PreviewPathAuthority, PreviewPresentationDebug, PreviewRenderBudget,
    PreviewRenderMode, PreviewRenderTarget,
};
pub use preview_vt4::capture_world_preview_vt4_probe;
pub use render_target_barrier::{
    committed_render_target_handle, sync_world_preview_render_viewport_contract,
    try_commit_world_preview_render_target, PendingRenderTargetBind, WorldPreviewGpuResizeQueue,
    WorldPreviewRenderTargetBindBarrier, WorldPreviewRenderTargetRegistry,
    WorldPreviewRenderViewportContract, WorldPreviewViewportEvent,
};
pub use registry_interchange::{
    material_registry_interchange_path, open_registry_interchange_in_desktop_shell,
    tag_registry_interchange_path,
};
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
pub use color_presets::{
    height_to_color, moisture_to_color, preview_biome_rgba_for_tile, temperature_to_color,
    terrain_family_preview_rgba,
};
pub use viewport::EditorViewport;
pub use viewport_authority::WorldPreviewViewportAuthority;
pub use viewport_suggestion::write_world_preview_viewport_request;
pub(crate) use window::display_world_preview;

use bevy::prelude::*;
use bevy_egui::{egui, EguiPrimaryContextPass};

use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::engine::{AppState, WorldGenChromeLatch};
use crate::gui::ViewRepresentationSystemSet;
use crate::render::{
    attrib_preview_cpu_raster_after, attrib_preview_cpu_raster_before,
    attrib_preview_gpu_present_after, attrib_preview_gpu_present_before,
};

/// Deferred GPU resize → commit → camera bind (`render_target_barrier` lifecycle).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum WorldPreviewLifecycleSet {
    ApplyGpuResize,
    CommitRenderTarget,
    BindGpuCamera,
    CameraTransform,
    ChunkQuads,
}

/// Ordering: resize → CPU raster → present swap (`base_visual_dev01_plan_status` § phase-d D-3).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum WorldPreviewRasterOrder {
    SyncTextureSize,
    RasterTiles,
    /// Runs after [`WorldPreviewRasterOrder::RasterTiles`] (CPU wrote `SwapImageBuffers::back`).
    PresentSwap,
}

/// D-01: single egui workspace hosts map chrome; generator is a slide sheet (not a second float).
pub const WORLD_PREVIEW_UNIFIED_WORKSPACE: bool = true;

/// Toggles for the World Preview egui workspace window.
#[derive(Resource)]
pub struct WorldPreviewUiState {
    pub window_open: bool,
    pub last_window_rect: Option<egui::Rect>,
    pub last_viewport_rect: Option<egui::Rect>,
}

impl Default for WorldPreviewUiState {
    fn default() -> Self {
        Self {
            window_open: false,
            last_window_rect: None,
            last_viewport_rect: None,
        }
    }
}

/// True when procedural world-gen UI should stay available (preview + generator panels).
#[must_use]
pub fn world_gen_flow_expects_chrome(flow: WorldGenFlowState) -> bool {
    matches!(
        flow,
        WorldGenFlowState::NewWorldSetup
            | WorldGenFlowState::PreviewReady
            | WorldGenFlowState::FullReady
    )
}

/// True while the operator is in the world-generator UX lane (not in-game / paused gameplay).
#[must_use]
pub fn world_gen_ux_app_active(app: Res<State<AppState>>) -> bool {
    matches!(*app.get(), AppState::WorldGen)
}

/// Editor / world-gen flows only — never while in active gameplay simulation.
#[must_use]
pub fn world_gen_editor_chrome_allowed(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
) -> bool {
    world_gen_ux_app_active(app) && !matches!(*base.get(), BaseState::Simulation)
}

/// True when D-01 unified workspace shell is active (map + generator slide sheet, one float).
#[must_use]
pub fn world_preview_unified_workspace(preview_ui: &WorldPreviewUiState) -> bool {
    WORLD_PREVIEW_UNIFIED_WORKSPACE && preview_ui.window_open
}

/// Open the unified world-gen workspace (F8 / NewWorldSetup latch).
pub fn open_world_gen_workspace(
    world_gen_ui: &mut crate::gui::editor::world_gen_ui::WorldGenUiState,
    preview_ui: &mut WorldPreviewUiState,
) {
    world_gen_ui.visible = true;
    preview_ui.window_open = true;
    if WORLD_PREVIEW_UNIFIED_WORKSPACE {
        world_gen_ui.generator_sheet_open = true;
    }
}

/// Open generator + preview once when entering a new world-build flow (not every frame).
pub fn open_world_gen_chrome_on_new_world_setup(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
    latch: Res<WorldGenChromeLatch>,
    mut world_gen_ui: ResMut<crate::gui::editor::world_gen_ui::WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
) {
    if !world_gen_editor_chrome_allowed(base, app) || latch.full_ready_dismissed {
        return;
    }
    let opened = !world_gen_ui.visible || !preview_ui.window_open;
    open_world_gen_workspace(&mut world_gen_ui, &mut preview_ui);
    if opened {
        crate::engine::worldgen_chrome_debug::log_chrome_open(
            "on_enter_new_world_setup",
            world_gen_ui.visible,
            preview_ui.window_open,
        );
    }
}

/// Close generator + preview chrome and park lifecycle (load-in / FullReady dismiss).
pub fn dismiss_world_gen_preview_chrome(
    world_gen_ui: &mut crate::gui::editor::world_gen_ui::WorldGenUiState,
    preview_ui: &mut WorldPreviewUiState,
    lifecycle: &mut WorldPreviewLifecycle,
    latch: &mut crate::engine::WorldGenChromeLatch,
    reason: &'static str,
) {
    latch.mark_full_ready_dismissed();
    world_gen_ui.visible = false;
    world_gen_ui.generator_sheet_open = false;
    preview_ui.window_open = false;
    lifecycle.park_uninitialized();
    crate::engine::worldgen_chrome_debug::log_chrome_dismiss(
        reason,
        latch.full_ready_dismissed,
        world_gen_ui.visible,
        preview_ui.window_open,
    );
}

/// World-gen / preview egui may draw only in [`AppState::WorldGen`] with an open panel/window.
#[must_use]
pub fn world_gen_chrome_may_render(
    app: Res<State<AppState>>,
    preview_ui: Res<WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) -> bool {
    if WORLD_PREVIEW_UNIFIED_WORKSPACE {
        world_gen_ux_app_active(app) && preview_ui.window_open
    } else {
        world_gen_ux_app_active(app) && (preview_ui.window_open || world_gen.visible)
    }
}

/// World preview GPU/CPU work runs only while the operator can see preview or generator chrome.
#[must_use]
pub fn world_preview_chrome_active(
    app: Res<State<AppState>>,
    preview_ui: Res<WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) -> bool {
    world_gen_chrome_may_render(app, preview_ui, world_gen)
}

/// FINISH-UX-06/07: preview raster/lifecycle only when UX world-gen is active, chrome visible, no spike guard.
#[must_use]
pub fn world_preview_pipeline_enabled(
    app: Res<State<AppState>>,
    worldgen: Res<State<crate::engine::WorldGenState>>,
    guard: Res<crate::engine::UxFrameSpikeGuard>,
    preview_ui: Res<WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) -> bool {
    crate::engine::worldgen_preview_systems_enabled(worldgen.get())
        && !guard.suppress_preview_this_frame
        && world_gen_chrome_may_render(app, preview_ui, world_gen)
}

/// Registers world preview resources + raster + chrome systems.
pub struct WorldPreviewPlugin;

fn seed_full_app_gpu_preview_authority_for_readiness(
    profile: Res<crate::render::Stage5ReadinessProfile>,
    mut gpu: ResMut<WorldPreviewGpuRuntime>,
) {
    if *profile == crate::render::Stage5ReadinessProfile::FULL_APP {
        gpu.offscreen_renderer_ready = true;
    }
}

impl Plugin for WorldPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::render::Stage5ReadinessProfile::FULL_APP);
        app.init_resource::<CompositePreviewGraphResource>();
        preview_render_contract::init_preview_render_contract_resources(app);
        app.init_resource::<crate::gui::SwapImageBuffers>()
            .init_resource::<WorldPreviewTexture>()
            .init_resource::<gpu_preview::WorldPreviewGpuRuntime>()
            .init_resource::<crate::render::WorldPreviewVt4Probe>()
            .init_resource::<WorldPreviewUiState>()
            .add_message::<WorldPreviewViewportEvent>()
            .init_resource::<WorldPreviewViewportAuthority>()
            .init_resource::<WorldPreviewGpuResizeQueue>()
            .init_resource::<WorldPreviewRenderTargetRegistry>()
            .init_resource::<WorldPreviewRenderViewportContract>()
            .init_resource::<WorldPreviewRenderTargetBindBarrier>()
            .init_resource::<WorldPreviewChunkCaches>()
            .init_resource::<WorldPreviewLifecycle>()
            .init_resource::<WorldPreviewLifecycleSignals>()
            .init_resource::<WorldPreviewReady>()
            .init_resource::<WorldPreviewReadinessDiagnostics>()
            .init_resource::<PreviewRenderState>()
            .init_resource::<wave_p_live_proof::WavePLiveProofState>()
            .init_resource::<crate::gui::hud::ViewportRectSanity>()
            .configure_sets(
                Update,
                (
                    WorldPreviewRasterOrder::RasterTiles.after(WorldPreviewRasterOrder::SyncTextureSize),
                    WorldPreviewRasterOrder::PresentSwap.after(WorldPreviewRasterOrder::RasterTiles),
                    WorldPreviewLifecycleSet::ApplyGpuResize
                        .in_set(ViewRepresentationSystemSet::RenderTargets),
                    WorldPreviewLifecycleSet::CommitRenderTarget
                        .after(WorldPreviewLifecycleSet::ApplyGpuResize),
                    WorldPreviewLifecycleSet::BindGpuCamera
                        .after(WorldPreviewLifecycleSet::CommitRenderTarget),
                    WorldPreviewLifecycleSet::CameraTransform
                        .after(WorldPreviewLifecycleSet::BindGpuCamera),
                    WorldPreviewLifecycleSet::ChunkQuads
                        .after(WorldPreviewLifecycleSet::CameraTransform),
                ),
            )
            .add_systems(Startup, init_world_preview_texture)
            .add_systems(
                Startup,
                gpu_preview::prefer_gpu_preview_mode_when_renderer_ready,
            )
            .add_systems(
                Startup,
                seed_full_app_gpu_preview_authority_for_readiness,
            )
            .add_systems(
                Update,
                (
                    viewport_authority::sync_world_preview_viewport_authority
                        .in_set(ViewRepresentationSystemSet::ResolveViewport)
                        .after(crate::render::ViewportPipelineSet::Resolve),
                    viewport_authority::queue_world_preview_gpu_resize_request
                        .in_set(ViewRepresentationSystemSet::RenderTargets),
                    preview_render_contract::sync_preview_render_contract_after_egui
                        .in_set(ViewRepresentationSystemSet::RenderTargets),
                    preview_render_contract::sync_preview_render_contract_system
                        .before(WorldPreviewRasterOrder::SyncTextureSize),
                    composite_preview_graph::sync_composite_preview_graph_resource
                        .after(preview_render_contract::sync_preview_render_contract_system),
                    preview_render_contract::sync_preview_path_authority
                        .after(composite_preview_graph::sync_composite_preview_graph_resource),
                    prime_world_preview_editor_camera
                        .in_set(ViewRepresentationSystemSet::ResolveViewport)
                        .after(crate::render::ViewportPipelineSet::Resolve),
                    sync_world_preview_texture_size.in_set(WorldPreviewRasterOrder::SyncTextureSize),
                    sync_world_preview_ready
                        .after(prime_world_preview_editor_camera)
                        .after(sync_world_preview_texture_size),
                ),
            )
            .add_systems(
                Update,
                (
                    attrib_preview_cpu_raster_before
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .after(sync_world_preview_ready)
                        .before(render_raster::update_world_preview_texture)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    render_raster::update_world_preview_texture
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    attrib_preview_cpu_raster_after
                        .in_set(WorldPreviewRasterOrder::RasterTiles)
                        .after(render_raster::update_world_preview_texture)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    texture_cache::present_world_preview_swap_after_raster
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_uses_cpu_raster),
                    attrib_preview_gpu_present_before
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .before(texture_cache::present_world_preview_gpu_swap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    texture_cache::present_world_preview_gpu_swap
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    attrib_preview_gpu_present_after
                        .in_set(WorldPreviewRasterOrder::PresentSwap)
                        .after(texture_cache::present_world_preview_gpu_swap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    preview_render_contract::sync_preview_render_target_from_presentation
                        .after(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(preview_render_contract::preview_gpu_authoritative_run_if),
                    advance_world_preview_lifecycle_system
                        .after(WorldPreviewRasterOrder::PresentSwap)
                        .run_if(world_preview_pipeline_enabled),
                    preview_lifecycle::park_preview_lifecycle_when_chrome_dismissed
                        .after(advance_world_preview_lifecycle_system),
                    texture_cache::apply_world_preview_gpu_resize_request
                        .in_set(WorldPreviewLifecycleSet::ApplyGpuResize)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::enforce_gpu_preview_pooled_swap
                        .before(WorldPreviewLifecycleSet::ApplyGpuResize)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::seed_world_preview_render_target_registry
                        .after(gpu_preview::enforce_gpu_preview_pooled_swap)
                        .before(WorldPreviewLifecycleSet::ApplyGpuResize)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::commit_world_preview_render_target
                        .in_set(WorldPreviewLifecycleSet::CommitRenderTarget)
                        .run_if(world_preview_pipeline_enabled),
                ),
            )
            .add_systems(
                Update,
                (
                    gpu_preview::sync_world_preview_offscreen_camera
                        .in_set(WorldPreviewLifecycleSet::BindGpuCamera)
                        .run_if(world_preview_pipeline_enabled),
                    render_target_barrier::sync_world_preview_render_viewport_contract
                        .in_set(ViewRepresentationSystemSet::RenderTargets)
                        .after(WorldPreviewLifecycleSet::BindGpuCamera)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::sync_world_preview_offscreen_camera_transform
                        .in_set(WorldPreviewLifecycleSet::CameraTransform)
                        .after(crate::gui::ViewRepresentationSystemSet::CameraSync)
                        .run_if(world_preview_pipeline_enabled),
                    gpu_preview::sync_world_preview_gpu_chunk_quads
                        .in_set(WorldPreviewLifecycleSet::ChunkQuads)
                        .run_if(world_preview_pipeline_enabled),
                    preview_vt4::capture_world_preview_vt4_probe
                        .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
                    viewport_authority::debug_trace_world_preview_viewport_authority
                        .in_set(ViewRepresentationSystemSet::PostFX),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                display_world_preview
                    .after(crate::gui::sync_shell_layout_drag_gate)
                    .in_set(ViewRepresentationSystemSet::UiCollect)
                    .run_if(world_preview_chrome_active),
            )
            .add_systems(
                Update,
                wave_p_live_proof::write_wave_p_live_proof_system
                    .run_if(in_state(crate::engine::states::BaseState::Simulation)),
            );
    }
}

pub use wave_p_live_proof::{WAVE_P_LIVE_JSON, WavePLiveProofState};

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
    fn unified_workspace_open_syncs_flags() {
        let mut world_gen = crate::gui::editor::world_gen_ui::WorldGenUiState::default();
        let mut preview = WorldPreviewUiState::default();
        open_world_gen_workspace(&mut world_gen, &mut preview);
        assert!(world_gen.visible);
        assert!(preview.window_open);
        assert!(world_gen.generator_sheet_open);
    }

    #[test]
    fn unified_workspace_chrome_may_render_uses_preview_only() {
        let preview = WorldPreviewUiState {
            window_open: true,
            ..Default::default()
        };
        assert!(world_preview_unified_workspace(&preview));
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
