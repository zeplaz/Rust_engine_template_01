//! Authoritative world-preview readiness — explicit lifecycle state + one boolean gate for GPU/CPU.

use bevy::prelude::*;

use super::preview_render_contract::{PreviewAuthoritativeSurface, PreviewPathAuthority};
use super::texture_cache::WorldPreviewTexture;
use super::viewport::EditorViewport;
use crate::gui::map_view::MapViewInstances;
use super::WorldPreviewRenderTargetRegistry;
use crate::engine::WorldGenFlowState;
use crate::gui::SwapImageBuffers;
use crate::render::ResolvedViewports;
use crate::terrain::generation::world_generator_enhanced::{WorldGenJobSlot, WorldGenParams};
use crate::terrain::material::{invalidate_world, InvalidationReason, WorldPreviewState};

/// Lifecycle for editor world preview (replaces implicit “four independent bools = one ready”).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PreviewState {
    #[default]
    Uninitialized,
    Loading,
    Ready,
    /// World generation satisfied but one of camera / texture / projection is not yet authoritative.
    Degraded,
    /// Reserved for explicit failure paths (e.g. invalid dimensions after gen).
    Failed,
}

/// First blocking gate in pipeline order: world → camera → texture → projection.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPreviewReadinessDiagnostics {
    pub state: PreviewState,
    pub world_ready: bool,
    pub camera_ready: bool,
    pub texture_ready: bool,
    pub projection_ready: bool,
    /// First unsatisfied gate, or `None` when [`PreviewState::Ready`].
    pub missing: Option<&'static str>,
}

impl Default for WorldPreviewReadinessDiagnostics {
    fn default() -> Self {
        Self {
            state: PreviewState::Uninitialized,
            world_ready: false,
            camera_ready: false,
            texture_ready: false,
            projection_ready: false,
            missing: Some("uninitialized"),
        }
    }
}

#[must_use]
pub fn classify_world_preview_readiness(
    world_ready: bool,
    camera_ready: bool,
    texture_ready: bool,
    projection_ready: bool,
) -> WorldPreviewReadinessDiagnostics {
    let missing = if !world_ready {
        Some("world_generation")
    } else if !camera_ready {
        Some("preview_camera")
    } else if !texture_ready {
        Some("preview_texture")
    } else if !projection_ready {
        Some("viewport_projection")
    } else {
        None
    };

    let state = if world_ready && camera_ready && texture_ready && projection_ready {
        PreviewState::Ready
    } else if !world_ready {
        PreviewState::Loading
    } else {
        PreviewState::Degraded
    };

    WorldPreviewReadinessDiagnostics {
        state,
        world_ready,
        camera_ready,
        texture_ready,
        projection_ready,
        missing,
    }
}

/// When true, preview raster / GPU present may run and egui may bind the texture.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldPreviewReady(pub bool);

#[must_use]
pub fn world_generation_complete(flow: WorldGenFlowState, job_busy: bool) -> bool {
    if job_busy {
        return false;
    }
    match flow {
        WorldGenFlowState::PreviewReady
        | WorldGenFlowState::FullReady
        | WorldGenFlowState::Idle
        | WorldGenFlowState::LoadingSave => true,
        WorldGenFlowState::NewWorldSetup => false,
    }
}

#[must_use]
pub fn preview_camera_initialized(viewport: &EditorViewport, params: &WorldGenParams) -> bool {
    params.width > 0
        && params.height > 0
        && viewport.camera_initialized
}

#[must_use]
pub fn preview_render_target_exists(
    path: &PreviewPathAuthority,
    preview_tex: &WorldPreviewTexture,
    registry: &WorldPreviewRenderTargetRegistry,
    images: &Assets<Image>,
    swap: &SwapImageBuffers,
) -> bool {
    if preview_tex.width == 0 || preview_tex.height == 0 {
        return false;
    }
    let handle_present = |h: &Handle<Image>| *h != Handle::default() && images.get(h).is_some();
    match path.authoritative_surface {
        PreviewAuthoritativeSurface::GpuRenderTarget => {
            // After GPU resize, `registry.committed_image` can still point at removed assets until
            // `try_commit_world_preview_render_target` runs; `swap.front` / `swap.back` are already valid.
            // Prefer committed when live, then swap, then `preview_tex.texture`.
            if handle_present(&registry.committed_image) {
                return true;
            }
            if handle_present(&swap.front) {
                return true;
            }
            if handle_present(&swap.back) {
                return true;
            }
            handle_present(&preview_tex.texture)
        }
        PreviewAuthoritativeSurface::CpuSwap => handle_present(&preview_tex.texture),
    }
}

#[must_use]
pub fn preview_projection_computed(resolved: &ResolvedViewports) -> bool {
    resolved.world_preview.valid
}

#[must_use]
pub fn compute_world_preview_ready(
    flow: WorldGenFlowState,
    job_busy: bool,
    viewport: &EditorViewport,
    params: &WorldGenParams,
    path: &PreviewPathAuthority,
    preview_tex: &WorldPreviewTexture,
    registry: &WorldPreviewRenderTargetRegistry,
    images: &Assets<Image>,
    swap: &SwapImageBuffers,
    resolved: &ResolvedViewports,
) -> bool {
    let world_ready = world_generation_complete(flow, job_busy);
    let camera_ready = preview_camera_initialized(viewport, params);
    let texture_ready =
        preview_render_target_exists(path, preview_tex, registry, images, swap);
    let projection_ready = preview_projection_computed(resolved);
    let d = classify_world_preview_readiness(world_ready, camera_ready, texture_ready, projection_ready);
    d.state == PreviewState::Ready
}

/// Prime editor viewport camera after world generation, before render activation.
pub fn prime_world_preview_editor_camera(
    flow: Res<State<WorldGenFlowState>>,
    job_slot: Res<WorldGenJobSlot>,
    params: Res<WorldGenParams>,
    mut views: ResMut<MapViewInstances>,
) {
    if !world_generation_complete(*flow.get(), job_slot.is_busy()) {
        return;
    }
    if views.world_preview.camera_initialized {
        return;
    }
    let tex_w = params.width as f32;
    let tex_h = params.height as f32;
    if tex_w <= 0.0 || tex_h <= 0.0 {
        return;
    }
    views.world_preview.reset_camera_for_map(tex_w, tex_h);
}

pub fn sync_world_preview_ready(
    flow: Res<State<WorldGenFlowState>>,
    job_slot: Res<WorldGenJobSlot>,
    params: Res<WorldGenParams>,
    views: Res<MapViewInstances>,
    path: Res<PreviewPathAuthority>,
    preview_tex: Res<WorldPreviewTexture>,
    registry: Res<WorldPreviewRenderTargetRegistry>,
    images: Res<Assets<Image>>,
    swap: Res<SwapImageBuffers>,
    resolved: Res<ResolvedViewports>,
    chunks: Query<&crate::terrain::generation::Chunk>,
    mut ready: ResMut<WorldPreviewReady>,
    mut diagnostics: ResMut<WorldPreviewReadinessDiagnostics>,
    mut preview_state: ResMut<WorldPreviewState>,
    mut last_logged: Local<Option<(PreviewState, bool, bool, bool, bool)>>,
) {
    let world_ready = world_generation_complete(*flow.get(), job_slot.is_busy());
    let camera_ready = preview_camera_initialized(&views.world_preview, &params);
    let texture_ready =
        preview_render_target_exists(&path, &preview_tex, &registry, &images, &swap);
    let projection_ready = preview_projection_computed(&resolved);

    let diag = classify_world_preview_readiness(
        world_ready,
        camera_ready,
        texture_ready,
        projection_ready,
    );
    *diagnostics = diag;

    let next_ready = diag.state == PreviewState::Ready;
    let was_ready = ready.0;
    ready.0 = next_ready;

    let contract_valid = resolved.world_preview.valid;
    debug_assert_eq!(
        projection_ready, contract_valid,
        "preview_projection_computed must stay aligned with ResolvedViewports.world_preview.valid"
    );

    let wp = &resolved.world_preview;
    let log_key = (diag.state, world_ready, camera_ready, texture_ready, projection_ready);
    if *last_logged != Some(log_key) {
        info!(
            state = ?diag.state,
            world_ready,
            camera_ready,
            texture_ready,
            projection_ready,
            missing = ?diag.missing,
            contract_valid,
            wp_half_x = wp.half_extents.x,
            wp_half_y = wp.half_extents.y,
            wp_logical_w = wp.logical_size.x,
            wp_logical_h = wp.logical_size.y,
            viewport_rev = resolved.revision,
            "PREVIEW STATE: world={world_ready} cam={camera_ready} tex={texture_ready} proj={projection_ready}"
        );
        *last_logged = Some(log_key);
    }

    if next_ready && !was_ready {
        let coords = chunks.iter().map(|chunk| chunk.coord);
        invalidate_world(InvalidationReason::Tuning, &mut preview_state, coords);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::math::UVec2;
    use bevy::render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    };

    fn test_image() -> Image {
        let size = Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        };
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            ..default()
        };
        image.asset_usage = RenderAssetUsages::MAIN_WORLD;
        image.data = Some(vec![0; 64]);
        image
    }

    #[test]
    fn classify_ready_only_when_all_gates_green() {
        let d = classify_world_preview_readiness(true, true, true, true);
        assert_eq!(d.state, PreviewState::Ready);
        assert!(d.missing.is_none());
    }

    #[test]
    fn classify_loading_when_world_blocked() {
        let d = classify_world_preview_readiness(false, true, true, true);
        assert_eq!(d.state, PreviewState::Loading);
        assert_eq!(d.missing, Some("world_generation"));
    }

    #[test]
    fn classify_degraded_reports_first_missing_gate() {
        let d = classify_world_preview_readiness(true, false, false, false);
        assert_eq!(d.state, PreviewState::Degraded);
        assert_eq!(d.missing, Some("preview_camera"));
    }

    #[test]
    fn ready_requires_all_gates() {
        let mut viewport = EditorViewport::default();
        viewport.camera_initialized = true;
        let params = WorldGenParams {
            width: 4,
            height: 4,
            ..Default::default()
        };
        let mut images = Assets::<Image>::default();
        let handle = images.add(test_image());
        let preview_tex = WorldPreviewTexture {
            texture: handle,
            width: 4,
            height: 4,
        };
        let path = PreviewPathAuthority::default();
        let registry = WorldPreviewRenderTargetRegistry::default();
        let swap = SwapImageBuffers {
            front: preview_tex.texture.clone(),
            back: Handle::default(),
            dirty: false,
        };
        let mut resolved = ResolvedViewports::default();
        resolved.world_preview.valid = true;
        assert!(compute_world_preview_ready(
            WorldGenFlowState::PreviewReady,
            false,
            &viewport,
            &params,
            &path,
            &preview_tex,
            &registry,
            &images,
            &swap,
            &resolved,
        ));
        resolved.world_preview.valid = false;
        assert!(!compute_world_preview_ready(
            WorldGenFlowState::PreviewReady,
            false,
            &viewport,
            &params,
            &path,
            &preview_tex,
            &registry,
            &images,
            &swap,
            &resolved,
        ));
    }

    #[test]
    fn gpu_texture_ready_accepts_swap_when_committed_asset_missing() {
        let mut viewport = EditorViewport::default();
        viewport.camera_initialized = true;
        let params = WorldGenParams {
            width: 4,
            height: 4,
            ..Default::default()
        };
        let mut images = Assets::<Image>::default();
        let stale = images.add(test_image());
        let live = images.add(test_image());
        let _ = images.remove(stale.id());

        let preview_tex = WorldPreviewTexture {
            texture: stale.clone(),
            width: 4,
            height: 4,
        };
        let mut path = PreviewPathAuthority::default();
        path.authoritative_surface = PreviewAuthoritativeSurface::GpuRenderTarget;

        let registry = WorldPreviewRenderTargetRegistry {
            revision: 1,
            committed_image: stale,
            committed_size: UVec2::splat(4),
        };
        let swap = SwapImageBuffers {
            front: live,
            back: Handle::default(),
            dirty: false,
        };
        let mut resolved = ResolvedViewports::default();
        resolved.world_preview.valid = true;

        assert!(preview_render_target_exists(
            &path, &preview_tex, &registry, &images, &swap,
        ));
        assert!(compute_world_preview_ready(
            WorldGenFlowState::PreviewReady,
            false,
            &viewport,
            &params,
            &path,
            &preview_tex,
            &registry,
            &images,
            &swap,
            &resolved,
        ));
    }
}
