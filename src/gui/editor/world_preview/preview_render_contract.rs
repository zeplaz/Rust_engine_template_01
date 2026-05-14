//! **Phase D — preview render architecture** (`base_visual_dev01_plan_status.md` § `phase-d-preview-render-target`).
//!
//! **D-1** — Preview transform + mode are **owned here**, separate from gameplay [`crate::gui::map_camera::MapCameraDesired`].
//! **D-2** — [`PreviewRenderTarget`] names the GPU image + extent the preview pipeline targets.
//! **D-5** — [`PreviewRenderBudget`] (`max_hz`, `force_redraw`); `max_hz` follows [`crate::gui::VisualCadence`]
//! when present, else [`crate::gui::VisualBudgetSettings`], else defaults.

use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;

use super::texture_cache::WorldPreviewTexture;
use super::viewport::EditorViewport;

/// How the world preview is produced (`GpuRenderTarget` reserved for Bevy camera → `RenderTarget` → egui).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PreviewRenderMode {
    /// Today: CPU full-world raster into `WorldPreviewTexture` (`render_raster.rs`).
    #[default]
    CpuRaster,
    /// Bevy sub-camera renders into `PreviewRenderTarget.image` (no egui-owned raster).
    GpuRenderTarget,
}

/// Which surface egui presents when both CPU and GPU paths exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PreviewAuthoritativeSurface {
    #[default]
    CpuSwap,
    GpuRenderTarget,
}

/// Runtime authority for preview pixels (Phase D strict DONE is GPU target + egui display-only).
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PreviewPathAuthority {
    pub authoritative_surface: PreviewAuthoritativeSurface,
    pub cpu_raster_fallback_active: bool,
    pub gpu_render_target_requested: bool,
    /// Monotonic present ticks when GPU target is authoritative (egui display-only).
    pub gpu_present_count: u32,
}

pub fn sync_preview_path_authority(
    cam: Res<PreviewCameraState>,
    gpu_rt: Res<crate::gui::editor::world_preview::WorldPreviewGpuRuntime>,
    mut authority: ResMut<PreviewPathAuthority>,
    mut debug: ResMut<PreviewPresentationDebug>,
) {
    authority.gpu_render_target_requested = cam.mode == PreviewRenderMode::GpuRenderTarget;
    authority.authoritative_surface = preview_authoritative_surface(&gpu_rt, &cam);
    authority.cpu_raster_fallback_active = matches!(
        authority.authoritative_surface,
        PreviewAuthoritativeSurface::CpuSwap
    );
    debug.authoritative_surface = authority.authoritative_surface;
}

/// Bevy schedule `run_if` — GPU preview owns the render target + swap presentation path.
#[inline]
pub fn preview_gpu_authoritative_run_if(
    cam: Res<PreviewCameraState>,
    gpu_rt: Res<super::WorldPreviewGpuRuntime>,
) -> bool {
    preview_authoritative_surface(&gpu_rt, &cam) == PreviewAuthoritativeSurface::GpuRenderTarget
}

#[must_use]
pub fn preview_authoritative_surface(
    gpu_rt: &crate::gui::editor::world_preview::WorldPreviewGpuRuntime,
    cam: &PreviewCameraState,
) -> PreviewAuthoritativeSurface {
    if gpu_rt.offscreen_renderer_ready && cam.mode == PreviewRenderMode::GpuRenderTarget {
        PreviewAuthoritativeSurface::GpuRenderTarget
    } else {
        PreviewAuthoritativeSurface::CpuSwap
    }
}

/// **Preview-only** camera state — never read gameplay [`crate::gui::MainWorldCamera`].
#[derive(Resource, Debug, Clone)]
pub struct PreviewCameraState {
    /// World-space preview center (tile-ish coordinates; matches [`EditorViewport::camera_center`] semantics).
    pub center: Vec2,
    pub zoom: f32,
    pub mode: PreviewRenderMode,
}

impl Default for PreviewCameraState {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            mode: PreviewRenderMode::CpuRaster,
        }
    }
}

/// GPU image the preview pipeline writes or samples (D-2).
#[derive(Resource, Debug, Clone)]
pub struct PreviewRenderTarget {
    pub image: Handle<Image>,
    pub size: UVec2,
}

impl Default for PreviewRenderTarget {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            size: UVec2::ZERO,
        }
    }
}

/// Cadence contract for preview redraw (D-5); aligns with global visual budgets when available.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PreviewRenderBudget {
    pub max_hz: f32,
    pub force_redraw: bool,
}

impl Default for PreviewRenderBudget {
    fn default() -> Self {
        Self {
            max_hz: 12.0,
            force_redraw: false,
        }
    }
}

/// Debug counters for D-3 swap / cadence HUD (`swap_count` increments on CPU double-buffer present).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PreviewPresentationDebug {
    pub swap_count: u32,
    /// Monotonic tick from the sync system (not Bevy frame index).
    pub last_contract_sync_tick: u64,
    pub last_front_asset_id_bits: u64,
    pub last_back_asset_id_bits: u64,
    pub authoritative_surface: PreviewAuthoritativeSurface,
}

pub(crate) fn sync_preview_render_contract_resources(
    preview_cam: &mut PreviewCameraState,
    target: &mut PreviewRenderTarget,
    debug: &mut PreviewPresentationDebug,
    viewport: &EditorViewport,
    tex: &WorldPreviewTexture,
    swap: &crate::gui::SwapImageBuffers,
    budget: &mut PreviewRenderBudget,
    visual_cadence: Option<&crate::gui::VisualCadence>,
    visual_budgets: Option<&crate::gui::VisualBudgetSettings>,
    sync_tick: u64,
) -> bool {
    budget.max_hz = visual_cadence
        .map(|c| c.preview_hz)
        .or_else(|| visual_budgets.map(|b| b.preview_hz))
        .filter(|&h| h.is_finite() && h > 0.25)
        .unwrap_or(12.0);

    let changed = match preview_cam.mode {
        PreviewRenderMode::CpuRaster => {
            let prev_center = preview_cam.center;
            let prev_zoom = preview_cam.zoom;
            let prev_size = target.size;
            let prev_image = target.image.clone();

            preview_cam.center = viewport.camera_center;
            preview_cam.zoom = viewport.zoom;
            target.image = tex.texture.clone();
            target.size = UVec2::new(tex.width, tex.height);

            prev_center != preview_cam.center
                || prev_zoom != preview_cam.zoom
                || prev_size != target.size
                || prev_image != target.image
        }
        PreviewRenderMode::GpuRenderTarget => {
            let prev_center = preview_cam.center;
            let prev_zoom = preview_cam.zoom;
            let prev_size = target.size;
            let prev_image = target.image.clone();

            preview_cam.center = viewport.camera_center;
            preview_cam.zoom = viewport.zoom;
            target.image = swap.front.clone();
            target.size = UVec2::new(tex.width, tex.height);

            prev_center != preview_cam.center
                || prev_zoom != preview_cam.zoom
                || prev_size != target.size
                || prev_image != target.image
        }
    };
    if changed {
        debug.last_contract_sync_tick = sync_tick;
    }
    changed
}

/// **D-4** — CPU double-buffer path runs only when GPU preview is not authoritative.
#[inline]
pub fn preview_uses_cpu_raster(
    cam: Res<PreviewCameraState>,
    gpu_rt: Res<super::WorldPreviewGpuRuntime>,
) -> bool {
    cam.mode == PreviewRenderMode::CpuRaster || !gpu_rt.offscreen_renderer_ready
}

/// True when egui should sample [`PreviewRenderTarget`] instead of the CPU swap front.
#[inline]
#[allow(dead_code)]
pub fn preview_gpu_authoritative(
    cam: Res<PreviewCameraState>,
    gpu_rt: Res<super::WorldPreviewGpuRuntime>,
) -> bool {
    preview_authoritative_surface(&gpu_rt, &cam) == PreviewAuthoritativeSurface::GpuRenderTarget
}

/// GPU preview binds [`crate::gui::SwapImageBuffers`] front/back; no separate render image.
pub(crate) fn ensure_gpu_preview_target_image(
    mode: PreviewRenderMode,
    target: &mut PreviewRenderTarget,
    tex: &WorldPreviewTexture,
    swap: &crate::gui::SwapImageBuffers,
) {
    if mode != PreviewRenderMode::GpuRenderTarget {
        return;
    }
    if tex.width == 0 || tex.height == 0 {
        return;
    }
    target.image = swap.front.clone();
    target.size = UVec2::new(tex.width, tex.height);
}

pub(crate) fn preview_image_asset_id_bits(handle: &Handle<Image>) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    handle.id().hash(&mut h);
    h.finish()
}

pub(crate) fn init_preview_render_contract_resources(app: &mut App) {
    app.init_resource::<PreviewCameraState>()
        .init_resource::<PreviewRenderTarget>()
        .init_resource::<PreviewRenderBudget>()
        .init_resource::<PreviewPresentationDebug>()
        .init_resource::<PreviewPathAuthority>();
}

pub(crate) fn sync_preview_render_contract_system(
    mut preview_cam: ResMut<PreviewCameraState>,
    mut target: ResMut<PreviewRenderTarget>,
    mut debug: ResMut<PreviewPresentationDebug>,
    mut budget: ResMut<PreviewRenderBudget>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    viewport: Res<EditorViewport>,
    tex: Res<WorldPreviewTexture>,
    gpu_rt: Res<crate::gui::editor::world_preview::WorldPreviewGpuRuntime>,
    visual_cadence: Option<Res<crate::gui::VisualCadence>>,
    visual_budgets: Option<Res<crate::gui::VisualBudgetSettings>>,
    mut sync_tick: Local<u64>,
) {
    *sync_tick = sync_tick.wrapping_add(1);
    let vc = visual_cadence.as_deref();
    let vb = visual_budgets.as_deref();
    let changed = sync_preview_render_contract_resources(
        &mut preview_cam,
        &mut target,
        &mut debug,
        &viewport,
        &tex,
        &swap,
        &mut budget,
        vc,
        vb,
        *sync_tick,
    );
    ensure_gpu_preview_target_image(preview_cam.mode, &mut target, &tex, &swap);
    if changed && preview_cam.mode == PreviewRenderMode::GpuRenderTarget {
        swap.dirty = true;
    }
    debug.last_front_asset_id_bits = preview_image_asset_id_bits(&target.image);
    debug.authoritative_surface = preview_authoritative_surface(&gpu_rt, &preview_cam);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::editor::world_preview::WorldPreviewGpuRuntime;

    #[test]
    fn authoritative_surface_prefers_gpu_when_renderer_ready() {
        let gpu_rt = crate::gui::editor::world_preview::WorldPreviewGpuRuntime {
            offscreen_renderer_ready: true,
        };
        let cam = PreviewCameraState {
            mode: PreviewRenderMode::GpuRenderTarget,
            ..Default::default()
        };
        assert_eq!(
            preview_authoritative_surface(&gpu_rt, &cam),
            PreviewAuthoritativeSurface::GpuRenderTarget
        );
        let cam_cpu = PreviewCameraState::default();
        assert_eq!(
            preview_authoritative_surface(&gpu_rt, &cam_cpu),
            PreviewAuthoritativeSurface::CpuSwap
        );
    }

    #[test]
    fn preview_defaults_cpu_raster() {
        let s = PreviewCameraState::default();
        assert_eq!(s.mode, PreviewRenderMode::CpuRaster);
    }

    #[test]
    fn gpu_authoritative_run_if_matches_surface() {
        let cam = PreviewCameraState {
            mode: PreviewRenderMode::GpuRenderTarget,
            ..Default::default()
        };
        let gpu_rt = WorldPreviewGpuRuntime {
            offscreen_renderer_ready: true,
        };
        assert_eq!(
            preview_authoritative_surface(&gpu_rt, &cam),
            PreviewAuthoritativeSurface::GpuRenderTarget
        );
    }

    #[test]
    fn cpu_raster_gated_when_gpu_target_authoritative() {
        let cam = PreviewCameraState {
            mode: PreviewRenderMode::GpuRenderTarget,
            ..Default::default()
        };
        let gpu_rt = WorldPreviewGpuRuntime {
            offscreen_renderer_ready: true,
        };
        assert_eq!(
            preview_authoritative_surface(&gpu_rt, &cam),
            PreviewAuthoritativeSurface::GpuRenderTarget
        );
        assert!(
            !(cam.mode == PreviewRenderMode::CpuRaster || !gpu_rt.offscreen_renderer_ready)
        );
    }

    #[test]
    fn cpu_raster_disabled_when_gpu_target_authoritative() {
        let cam = PreviewCameraState {
            mode: PreviewRenderMode::GpuRenderTarget,
            ..Default::default()
        };
        let gpu_rt = WorldPreviewGpuRuntime {
            offscreen_renderer_ready: true,
        };
        assert_eq!(
            preview_authoritative_surface(&gpu_rt, &cam),
            PreviewAuthoritativeSurface::GpuRenderTarget
        );
        assert!(
            cam.mode != PreviewRenderMode::CpuRaster && gpu_rt.offscreen_renderer_ready
        );
    }

    #[test]
    fn cpu_raster_sync_copies_viewport_and_texture() {
        let mut cam = PreviewCameraState::default();
        let mut tgt = PreviewRenderTarget::default();
        let mut dbg = PreviewPresentationDebug::default();
        let mut budget = PreviewRenderBudget::default();
        let mut vp = EditorViewport::default();
        vp.camera_center = Vec2::new(10.0, 20.0);
        vp.zoom = 2.5;
        let mut tex = WorldPreviewTexture::default();
        tex.width = 64;
        tex.height = 48;
        tex.texture = Handle::default();

        let swap = crate::gui::SwapImageBuffers::default();

        sync_preview_render_contract_resources(
            &mut cam, &mut tgt, &mut dbg, &vp, &tex, &swap, &mut budget, None, None, 1,
        );
        assert_eq!(cam.center, vp.camera_center);
        assert_eq!(cam.zoom, vp.zoom);
        assert_eq!(tgt.size, UVec2::new(64, 48));
        assert_eq!(tgt.image, tex.texture);
        assert_eq!(dbg.last_contract_sync_tick, 1);
        assert_eq!(budget.max_hz, 12.0);
    }

    #[test]
    fn preview_budget_prefers_visual_cadence_over_visual_budget_settings() {
        let mut cam = PreviewCameraState::default();
        let mut tgt = PreviewRenderTarget::default();
        let mut dbg = PreviewPresentationDebug::default();
        let mut budget = PreviewRenderBudget::default();
        let vp = EditorViewport::default();
        let tex = WorldPreviewTexture::default();
        let cadence = crate::gui::VisualCadence {
            minimap_hz: 10.0,
            preview_hz: 7.5,
            overlay_hz: 15.0,
            atmosphere_hz: 30.0,
        };
        let mut budgets = crate::gui::VisualBudgetSettings::default();
        budgets.preview_hz = 24.0;

        let swap = crate::gui::SwapImageBuffers::default();

        sync_preview_render_contract_resources(
            &mut cam,
            &mut tgt,
            &mut dbg,
            &vp,
            &tex,
            &swap,
            &mut budget,
            Some(&cadence),
            Some(&budgets),
            1,
        );
        assert!((budget.max_hz - 7.5).abs() < 1e-5);
    }

    #[test]
    fn gpu_mode_syncs_viewport_and_binds_swap_front_not_cpu_texture() {
        let mut cam = PreviewCameraState {
            center: Vec2::ONE,
            zoom: 3.0,
            mode: PreviewRenderMode::GpuRenderTarget,
        };
        let mut tgt = PreviewRenderTarget::default();
        let mut dbg = PreviewPresentationDebug::default();
        let mut budget = PreviewRenderBudget::default();
        let mut vp = EditorViewport::default();
        vp.camera_center = Vec2::new(99.0, 88.0);
        vp.zoom = 1.25;
        let mut tex = WorldPreviewTexture::default();
        tex.texture = Handle::default();
        tex.width = 32;
        tex.height = 32;

        let mut images = Assets::<Image>::default();
        let front = images.add(super::super::texture_cache::rgba_preview_image(32, 32));
        let back = images.add(super::super::texture_cache::rgba_preview_image(32, 32));
        let swap = crate::gui::SwapImageBuffers {
            front: front.clone(),
            back,
            dirty: false,
        };

        sync_preview_render_contract_resources(
            &mut cam, &mut tgt, &mut dbg, &vp, &tex, &swap, &mut budget, None, None, 1,
        );
        assert_eq!(cam.center, vp.camera_center);
        assert_eq!(cam.zoom, vp.zoom);

        ensure_gpu_preview_target_image(
            PreviewRenderMode::GpuRenderTarget,
            &mut tgt,
            &tex,
            &swap,
        );
        assert_eq!(tgt.image, front);
        assert_ne!(tgt.image, tex.texture);
        assert_eq!(tgt.size, UVec2::new(32, 32));
    }
}
