//! **View Representation Layer** — camera intent drives LOD, overlay weighting, atmosphere accent,
//! and FX class selection (not independent `if zoom` / `if tactical` branches).
//!
//! Design: `prompts/guides/base_visual_dev01_plan_status.md` (invariants + three themes).
//! Today: [`CameraVisualState`] is synced from [`super::map_camera::MapCameraSettings`] /
//! [`super::map_camera::MapCameraDesired`] until [`CameraIntent`] gains follow-target wiring.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::compute::ComputeDispatchGraph;
use crate::render::{
    record_visual_agreement_frame, GpuRepresentationMetrics, VisualAgreementFrame,
    WorldFireParticleFrame,
};
use crate::render::extraction::RenderProjectionGraph;

use super::representation_policy::RepresentationResult;

use super::map_camera::{
    in_simulation_or_editor_map, map_zoom_alpha, MapCameraDesired, MapCameraMode, MapCameraSettings,
    MapCameraSystemSet, MAP_ZOOM_CLAMP,
};

pub fn apply_camera_visual_from_map_snapshot(
    settings: &MapCameraSettings,
    desired_scale_x: f32,
    visual: &mut CameraVisualState,
    fx: &mut FxVisibilitySettings,
) {
    visual.intent = match settings.mode {
        MapCameraMode::Strategic => CameraIntent::Strategic,
        MapCameraMode::Tactical => CameraIntent::Tactical,
        MapCameraMode::Cinematic => CameraIntent::Cinematic,
    };
    visual.zoom_alpha = map_zoom_alpha(desired_scale_x);

    visual.strategic_weight = match settings.mode {
        MapCameraMode::Strategic => 1.0,
        MapCameraMode::Tactical => 0.35,
        MapCameraMode::Cinematic => 0.2,
    };
    visual.cinematic_weight = match settings.mode {
        MapCameraMode::Cinematic => 1.0,
        MapCameraMode::Tactical => 0.25,
        MapCameraMode::Strategic => 0.15,
    };

    fx.atmosphere_weight =
        (visual.strategic_weight * 0.55 + visual.cinematic_weight * 1.15).clamp(0.15, 1.25);
    fx.overlay_blend = (0.75 + visual.zoom_alpha * 0.2).clamp(0.0, 1.0);
}

// --- Camera intent pipeline (Theme 1st) --------------------------------------

/// High-level view mode; extends [`MapCameraMode`] with follow / free pan (`MapCameraMode` bridge
/// maps the first three variants only until gameplay wires [`CameraIntent::FollowEntity`].
#[derive(Debug, Clone)]
pub enum CameraIntent {
    Strategic,
    Tactical,
    Cinematic,
    FreePan,
    FollowEntity(Entity),
}

impl Default for CameraIntent {
    fn default() -> Self {
        Self::Strategic
    }
}

/// Derived weights for downstream systems (atmosphere, LOD, overlays, particles).
#[derive(Resource, Debug, Clone)]
pub struct CameraVisualState {
    pub intent: CameraIntent,
    /// Normalized zoom in `[0, 1]` from [`MAP_ZOOM_CLAMP`] using [`MapCameraDesired::scale`].
    pub zoom_alpha: f32,
    pub strategic_weight: f32,
    pub cinematic_weight: f32,
}

impl Default for CameraVisualState {
    fn default() -> Self {
        Self {
            intent: CameraIntent::default(),
            zoom_alpha: 0.5,
            strategic_weight: 1.0,
            cinematic_weight: 0.0,
        }
    }
}

/// Who owns the main map view (HUD can show `CAM: …` from this).
#[derive(Debug, Clone)]
pub enum CameraOwner {
    Player,
    FollowTarget(Entity),
    CinematicTrack,
    Replay,
}

impl Default for CameraOwner {
    fn default() -> Self {
        Self::Player
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveCameraOwner {
    pub owner: CameraOwner,
}

/// World-anchored tactical fire / smoke garnish (sim-tied locations, readability-first).
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct WorldFireFx;

/// Camera-reactive macro atmosphere garnish (non-authoritative presentation).
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct AtmosphereFx;

/// Policy knobs derived from [`CameraVisualState`] (populate in a future resolver system).
#[derive(Resource, Debug, Clone, Copy)]
pub struct FxVisibilitySettings {
    pub world_fx_distance: f32,
    pub atmosphere_weight: f32,
    pub overlay_blend: f32,
}

impl Default for FxVisibilitySettings {
    fn default() -> Self {
        Self {
            world_fx_distance: 2_000.0,
            atmosphere_weight: 0.7,
            overlay_blend: 0.85,
        }
    }
}

// --- LOD + overlay matrix (Theme 3rd) ----------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FireVisualLod {
    Full,
    Clustered,
    OverlayOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct FireLodSelection {
    pub entity: Entity,
    pub lod: FireVisualLod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayChannel {
    FireHeat,
    Smoke,
    Moisture,
    Ecology,
    Mobility,
    Wind,
}

/// Single producer target for minimap / preview / debug / GPU weather (`HashMap` grows per channel).
#[derive(Resource, Debug, Clone, Default)]
pub struct OverlayFieldFrame {
    pub stamp: crate::systems::sim_control::SimStepStamp,
    pub fields: HashMap<OverlayChannel, Handle<Image>>,
    /// CPU fire overlay: revision of [`crate::render::SharedOverlayFieldBuffers`] (single producer; map not duplicated here).
    pub fire_heat_overlay_revision: u64,
}

// --- Multirate budgets --------------------------------------------------------

#[derive(Resource, Debug, Clone, Copy)]
pub struct VisualBudgetSettings {
    pub preview_hz: f32,
    pub minimap_hz: f32,
    pub atmosphere_hz: f32,
    pub overlay_hz: f32,
}

impl Default for VisualBudgetSettings {
    fn default() -> Self {
        Self {
            preview_hz: 12.0,
            minimap_hz: 10.0,
            atmosphere_hz: 30.0,
            overlay_hz: 15.0,
        }
    }
}

impl From<&VisualBudgetSettings> for VisualCadence {
    fn from(b: &VisualBudgetSettings) -> Self {
        Self {
            minimap_hz: b.minimap_hz,
            preview_hz: b.preview_hz,
            overlay_hz: b.overlay_hz,
            atmosphere_hz: b.atmosphere_hz,
        }
    }
}

/// Central **Hz policy** for multirate visual systems (`meta-schedule-contracts`).
/// [`VisualBudgetSettings`] remains the authoring surface in tooling; this resource is what
/// `run_if(on_visual_cadence_*)` and preview/minimap throttles should read.
#[derive(Resource, Debug, Clone, Copy)]
pub struct VisualCadence {
    pub minimap_hz: f32,
    pub preview_hz: f32,
    pub overlay_hz: f32,
    pub atmosphere_hz: f32,
}

impl Default for VisualCadence {
    fn default() -> Self {
        Self::from(&VisualBudgetSettings::default())
    }
}

/// `run_if` helpers pick one lane; each keeps its own [`Local`] accumulator.
#[inline]
fn visual_cadence_interval_tick(
    time: &Time,
    hz: f32,
    acc: &mut f32,
) -> bool {
    let interval = preview_partial_min_interval_from_hz(hz);
    *acc += time.delta_secs();
    if *acc >= interval {
        *acc -= interval;
        true
    } else {
        false
    }
}

#[must_use]
pub fn on_visual_cadence_minimap(
    time: Res<Time>,
    cadence: Res<VisualCadence>,
    mut acc: Local<f32>,
) -> bool {
    visual_cadence_interval_tick(&time, cadence.minimap_hz, &mut acc)
}

#[must_use]
pub fn on_visual_cadence_preview(
    time: Res<Time>,
    cadence: Res<VisualCadence>,
    mut acc: Local<f32>,
) -> bool {
    visual_cadence_interval_tick(&time, cadence.preview_hz, &mut acc)
}

#[must_use]
pub fn on_visual_cadence_overlay(
    time: Res<Time>,
    cadence: Res<VisualCadence>,
    mut acc: Local<f32>,
) -> bool {
    visual_cadence_interval_tick(&time, cadence.overlay_hz, &mut acc)
}

#[must_use]
pub fn on_visual_cadence_atmosphere(
    time: Res<Time>,
    cadence: Res<VisualCadence>,
    mut acc: Local<f32>,
) -> bool {
    visual_cadence_interval_tick(&time, cadence.atmosphere_hz, &mut acc)
}

fn sync_visual_cadence_from_visual_budget_settings(
    budgets: Res<VisualBudgetSettings>,
    mut cadence: ResMut<VisualCadence>,
) {
    if !budgets.is_changed() {
        return;
    }
    *cadence = VisualCadence::from(&*budgets);
}

/// Minimum seconds between partial passes for a given effective preview rate (Hz).
#[inline]
pub fn preview_partial_min_interval_from_hz(hz: f32) -> f32 {
    let hz = if hz.is_finite() && hz > 0.25 { hz } else { 12.0 };
    1.0 / hz
}

/// Minimum seconds between partial / overlay-only world preview CPU passes (`VisualBudgetSettings::preview_hz`).
#[inline]
pub fn preview_partial_min_interval_secs(budgets: Option<&VisualBudgetSettings>) -> f32 {
    let hz = budgets
        .map(|b| b.preview_hz)
        .filter(|&h| h.is_finite() && h > 0.25)
        .unwrap_or(12.0);
    preview_partial_min_interval_from_hz(hz)
}

#[must_use]
pub fn camera_owner_label(owner: &CameraOwner) -> &'static str {
    match owner {
        CameraOwner::Player => "PLAYER",
        CameraOwner::FollowTarget(_) => "FOLLOW",
        CameraOwner::CinematicTrack => "CINE",
        CameraOwner::Replay => "REPLAY",
    }
}

// --- Preview / CPU path double-buffer (generic utility; Theme 2nd) ------------

/// Front is what egui reads; writers mutate back then swap when `dirty` and upload complete.
#[derive(Resource, Debug, Clone)]
pub struct SwapImageBuffers {
    pub front: Handle<Image>,
    pub back: Handle<Image>,
    pub dirty: bool,
}

impl Default for SwapImageBuffers {
    fn default() -> Self {
        Self {
            front: Handle::default(),
            back: Handle::default(),
            dirty: false,
        }
    }
}

fn sync_overlay_field_frame_from_shared_fire(
    fire: Option<Res<crate::render::sim_visual_extract::FireVisualFrame>>,
    shared: Option<Res<crate::render::SharedOverlayFieldBuffers>>,
    mut overlay: ResMut<OverlayFieldFrame>,
) {
    if let Some(frame) = fire {
        overlay.stamp = frame.stamp;
    }
    if let Some(s) = shared {
        overlay.fire_heat_overlay_revision = s.revision;
    }
}

fn sync_representation_metrics_for_hud(
    policy: Res<RepresentationResult>,
    projection: Res<RenderProjectionGraph>,
    particles: Res<WorldFireParticleFrame>,
    compute: Res<ComputeDispatchGraph>,
    mut metrics: ResMut<GpuRepresentationMetrics>,
) {
    metrics.active_band = policy.active_band;
    metrics.instance_rows = projection.fire.instance_buffer.len() as u32;
    metrics.particle_rows = particles.instances.len() as u32;
    metrics.reserved_capacity = policy.gpu_budget.reserved_capacity;
    metrics.active_capacity = policy.gpu_budget.active_capacity;
    let mut dispatches = 0u32;
    if compute.fire_influence.dispatch_active {
        dispatches += 1;
    }
    if compute.heat_diffusion.last_generation > 0 {
        dispatches += 1;
    }
    if !particles.instances.is_empty() {
        dispatches += 1;
    }
    metrics.record_dispatch_count(dispatches);
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewRepresentationSystemSet {
    SyncOverlayField,
}

// --- Plugin -----------------------------------------------------------------

pub struct ViewRepresentationPlugin;

impl Plugin for ViewRepresentationPlugin {
    fn build(&self, app: &mut App) {
        super::world_representation::register_world_representation_frame(app);
        app.add_plugins(crate::render::Stage5ReadinessPlugin);
        app.add_plugins(crate::render::VtCiMatrixPlugin);
        app.add_plugins(crate::render::PhaseFLodProofPlugin);
        app.add_plugins(crate::render::DomainProjectionFramePlugin);
        app.add_plugins(crate::render::GpuIndirectDrawSpinePlugin);
        app.add_plugins(crate::io::streaming::StreamingSpinePlugin);
        app.init_resource::<VisualAgreementFrame>()
            .init_resource::<crate::render::OverlayAgreementDebug>()
            .init_resource::<GpuRepresentationMetrics>()
            .init_resource::<crate::render::LogisticsVisualSnapshot>()
            .init_resource::<crate::render::EcologyVisualSnapshot>()
            .init_resource::<CameraVisualState>()
            .init_resource::<FxVisibilitySettings>()
            .init_resource::<VisualBudgetSettings>()
            .init_resource::<VisualCadence>()
            .init_resource::<ActiveCameraOwner>()
            .init_resource::<OverlayFieldFrame>()
            .add_systems(PreUpdate, sync_visual_cadence_from_visual_budget_settings)
            .configure_sets(
                Update,
                ViewRepresentationSystemSet::SyncOverlayField
                    .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
            )
            .add_systems(
                Update,
                sync_overlay_field_frame_from_shared_fire
                    .in_set(ViewRepresentationSystemSet::SyncOverlayField),
            )
            .add_systems(
                Update,
                (
                    crate::render::publish_logistics_visual_snapshot,
                    crate::render::publish_ecology_visual_snapshot,
                )
                    .chain()
                    .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles)
                    .in_set(ViewRepresentationSystemSet::SyncOverlayField),
            )
            .add_systems(
                Update,
                sync_representation_metrics_for_hud
                    .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu)
                    .after(crate::compute::ComputeDispatchSystemSet::Dispatch),
            )
            .add_systems(
                Update,
                crate::render::sync_particle_draw_dispatch_from_policy
                    .after(crate::render::merge_domain_projection_into_representation)
                    .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu),
            )
            .add_systems(
                Update,
                record_visual_agreement_frame
                    .after(ViewRepresentationSystemSet::SyncOverlayField)
                    .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu)
                    .after(crate::gui::editor::world_preview::capture_world_preview_vt4_probe),
            )
            .add_systems(
                Update,
                sync_camera_visual_state_from_map_camera
                    .after(MapCameraSystemSet::ApplyInput)
                    .before(MapCameraSystemSet::Smooth)
                    .run_if(in_simulation_or_editor_map),
            );
    }
}

fn sync_camera_visual_state_from_map_camera(
    settings: Res<MapCameraSettings>,
    desired: Res<MapCameraDesired>,
    mut visual: ResMut<CameraVisualState>,
    mut fx: ResMut<FxVisibilitySettings>,
) {
    apply_camera_visual_from_map_snapshot(&settings, desired.scale.x, &mut visual, &mut fx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_partial_min_interval_respects_budget_hz() {
        let mut b = VisualBudgetSettings::default();
        b.preview_hz = 24.0;
        assert!((preview_partial_min_interval_secs(Some(&b)) - 1.0 / 24.0).abs() < 1e-4);
        assert!((preview_partial_min_interval_secs(None) - 1.0 / 12.0).abs() < 1e-4);
    }

    #[test]
    fn visual_cadence_defaults_match_visual_budget_defaults() {
        let b = VisualBudgetSettings::default();
        let c = VisualCadence::default();
        assert!((c.preview_hz - b.preview_hz).abs() < 1e-5);
        assert!((c.minimap_hz - b.minimap_hz).abs() < 1e-5);
        assert!((c.overlay_hz - b.overlay_hz).abs() < 1e-5);
        assert!((c.atmosphere_hz - b.atmosphere_hz).abs() < 1e-5);
    }

    /// **VT-2** (`visual-test-matrix-upgrade`): normalized zoom must track clamped scale monotonically.
    #[test]
    fn vt2_zoom_alpha_monotonic_over_scale_sweep() {
        let (lo, hi) = MAP_ZOOM_CLAMP;
        let mut prev = crate::gui::map_camera::map_zoom_alpha(lo - 50.0);
        for i in 0..=1000 {
            let t = i as f32 / 1000.0;
            let s = lo - 5.0 + (hi - lo + 10.0) * t;
            let a = crate::gui::map_camera::map_zoom_alpha(s);
            assert!(
                a + 1e-5 >= prev,
                "zoom_alpha decreased at scale={s}: {a} < {prev}"
            );
            prev = a;
        }
        assert!((crate::gui::map_camera::map_zoom_alpha(lo) - 0.0).abs() < 1e-5);
        assert!((crate::gui::map_camera::map_zoom_alpha(hi) - 1.0).abs() < 1e-5);
    }

    /// **VT-2** — full [`super::apply_camera_visual_from_map_snapshot`] path: `zoom_alpha` + `overlay_blend` monotonic in scale.
    #[test]
    fn vt2_pipeline_zoom_and_overlay_blend_monotonic_over_scale_sweep() {
        use crate::gui::map_camera::MapCameraSettings;
        let settings = MapCameraSettings::default();
        let (lo, hi) = MAP_ZOOM_CLAMP;
        let mut visual = CameraVisualState::default();
        let mut fx = FxVisibilitySettings::default();
        let mut prev_za = 0.0_f32;
        let mut prev_ob = 0.0_f32;
        for i in 0..=500 {
            let t = i as f32 / 500.0;
            let s = lo - 2.0 + (hi - lo + 4.0) * t;
            super::apply_camera_visual_from_map_snapshot(&settings, s, &mut visual, &mut fx);
            assert!(
                visual.zoom_alpha + 1e-5 >= prev_za,
                "zoom_alpha regressed at scale={s}"
            );
            assert!(
                fx.overlay_blend + 1e-5 >= prev_ob,
                "overlay_blend regressed at scale={s}"
            );
            prev_za = visual.zoom_alpha;
            prev_ob = fx.overlay_blend;
        }
    }

    #[test]
    fn overlay_field_frame_tracks_shared_fire_overlay_revision() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<OverlayFieldFrame>();
        app.init_resource::<crate::render::SharedOverlayFieldBuffers>();
        app.add_systems(Update, sync_overlay_field_frame_from_shared_fire);

        let r0 = app.world().resource::<crate::render::SharedOverlayFieldBuffers>().revision;
        app.update();
        assert_eq!(
            app.world().resource::<OverlayFieldFrame>().fire_heat_overlay_revision,
            r0
        );

        app.world_mut()
            .resource_mut::<crate::render::SharedOverlayFieldBuffers>()
            .bump();
        let r1 = app.world().resource::<crate::render::SharedOverlayFieldBuffers>().revision;
        app.update();
        assert_eq!(
            app.world().resource::<OverlayFieldFrame>().fire_heat_overlay_revision,
            r1
        );
        assert_ne!(r0, r1);
    }
}
