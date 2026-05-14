//! Authoritative **representation domain** contracts (`RepresentationInputs` → `RepresentationResult`).
//!
//! Downstream render, compute, overlay, preview, and particle paths must read [`RepresentationResult`]
//! only — not local zoom / distance / mode branches.

use bevy::prelude::*;

use super::view_representation::{CameraVisualState, VisualBudgetSettings, VisualCadence};
use super::world_representation::{
    LodZoneRegistry, WorldLodBand, WorldRepresentationFrame, WorldResolutionPolicy,
    WorldVisibilityMask,
};
use crate::systems::sim_control::SimStepStamp;

/// World-representation class (not fire-specific). Rename track: `FireVisualLod` → this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum RepresentationBand {
    #[default]
    Full,
    Tactical,
    Strategic,
    OverlayOnly,
    Dormant,
}

impl RepresentationBand {
    #[must_use]
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::Tactical => "TAC",
            Self::Strategic => "STR",
            Self::OverlayOnly => "OVR",
            Self::Dormant => "DRM",
        }
    }
}

#[must_use]
pub const fn representation_band_from_world_lod(band: WorldLodBand) -> RepresentationBand {
    match band {
        WorldLodBand::LocalTactical => RepresentationBand::Full,
        WorldLodBand::Operational => RepresentationBand::Tactical,
        WorldLodBand::Strategic => RepresentationBand::Strategic,
        WorldLodBand::Macro => RepresentationBand::OverlayOnly,
    }
}

/// Influence-domain class (authoring target; not camera-only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LodZoneClass {
    Camera,
    Combat,
    Projectile,
    Mission,
    Sensor,
    Editor,
    JumpPoint,
}

/// Per-domain extract shaping (rename track: `FireVisualExtractPlan` → this type).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldRepresentationExtractPlan {
    pub fire_instances: bool,
    pub fire_instance_cap: usize,
    pub chunk_heat_bin: i32,
    pub overlay_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayPolicy {
    pub fire_heat: bool,
    pub chunk_heat_bin: i32,
    pub overlay_hz_scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuBudgetPolicy {
    pub fire_instance_cap: usize,
    pub particle_rows_cap: usize,
    pub reserved_capacity: u32,
    pub active_capacity: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeBudgetPolicy {
    pub dispatch_hz: f32,
    pub heat_diffusion: bool,
    pub agent_steering: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayChannelMatrix {
    pub fire_heat: bool,
    pub logistics: bool,
    pub ecology: bool,
    pub chunk_heat_bin: i32,
}

impl Default for OverlayChannelMatrix {
    fn default() -> Self {
        Self {
            fire_heat: true,
            logistics: false,
            ecology: false,
            chunk_heat_bin: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewRepresentationPolicy {
    pub authoritative_gpu: bool,
    pub cpu_raster_debug: bool,
    pub partial_hz_scale: f32,
}

impl Default for PreviewRepresentationPolicy {
    fn default() -> Self {
        Self {
            authoritative_gpu: false,
            cpu_raster_debug: true,
            partial_hz_scale: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleRepresentationPolicy {
    pub instanced_draw: bool,
    pub rows_cap: usize,
}

impl Default for ParticleRepresentationPolicy {
    fn default() -> Self {
        Self {
            instanced_draw: true,
            rows_cap: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldViewBounds {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayPolicyInputs {
    pub fire_heat: bool,
    pub logistics: bool,
    pub ecology: bool,
}

impl Default for OverlayPolicyInputs {
    fn default() -> Self {
        Self {
            fire_heat: true,
            logistics: false,
            ecology: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeBudgetInputs {
    pub heat_diffusion: bool,
    pub agent_steering: bool,
}

impl Default for ComputeBudgetInputs {
    fn default() -> Self {
        Self {
            heat_diffusion: true,
            agent_steering: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RepresentationBudgets {
    pub visual: VisualBudgetSettings,
}

impl Default for RepresentationBudgets {
    fn default() -> Self {
        Self {
            visual: VisualBudgetSettings::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RepresentationInputs {
    pub camera: CameraVisualState,
    pub zones: LodZoneRegistry,
    pub budgets: RepresentationBudgets,
    pub cadence: VisualCadence,
    pub stamp: SimStepStamp,
    pub overlay_policy: OverlayPolicyInputs,
    pub compute_budget: ComputeBudgetInputs,
    pub viewport_px: UVec2,
    pub world_view_bounds: WorldViewBounds,
}

/// Rename track: `FireVisualLod` → [`RepresentationBand`].
pub type FireVisualLod = RepresentationBand;

/// Rename track: `FireVisualExtractPlan` → [`WorldRepresentationExtractPlan`].
pub type FireVisualExtractPlan = WorldRepresentationExtractPlan;

/// Single authoritative policy output per frame.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct RepresentationResult {
    pub stamp: SimStepStamp,
    pub active_band: RepresentationBand,
    pub world_lod_band: WorldLodBand,
    pub overlay_matrix: OverlayChannelMatrix,
    pub extract_plan: WorldRepresentationExtractPlan,
    pub overlay_policy: OverlayPolicy,
    pub gpu_budget: GpuBudgetPolicy,
    pub compute_budget: ComputeBudgetPolicy,
    pub preview_policy: PreviewRepresentationPolicy,
    pub particle_policy: ParticleRepresentationPolicy,
    pub visibility: WorldVisibilityMask,
}

impl Default for RepresentationResult {
    fn default() -> Self {
        let band = WorldLodBand::LocalTactical;
        let _active = representation_band_from_world_lod(band);
        let visibility = super::world_representation::visibility_for_band(band);
        let resolution = super::world_representation::resolution_for_band(band);
        Self::from_frame_and_inputs(
            &WorldRepresentationFrame {
                bands: super::world_representation::WorldLodBands { global: band },
                visibility,
                resolution,
                ..Default::default()
            },
            SimStepStamp::default(),
            &RepresentationInputs {
                camera: CameraVisualState::default(),
                zones: LodZoneRegistry::default(),
                stamp: SimStepStamp::default(),
                budgets: RepresentationBudgets::default(),
                cadence: VisualCadence::from(&VisualBudgetSettings::default()),
                overlay_policy: OverlayPolicyInputs::default(),
                compute_budget: ComputeBudgetInputs::default(),
                viewport_px: UVec2::ONE,
                world_view_bounds: WorldViewBounds::default(),
            },
        )
    }
}

impl RepresentationResult {
    #[must_use]
    pub fn from_frame_and_inputs(
        frame: &WorldRepresentationFrame,
        sim_stamp: SimStepStamp,
        inputs: &RepresentationInputs,
    ) -> Self {
        let world_lod_band = frame.global_band();
        let active_band = representation_band_from_world_lod(world_lod_band);
        let visibility = frame.visibility;
        let resolution = frame.resolution;
        let extract_plan = extract_plan_from_resolution(&visibility, &resolution, active_band);
        let overlay_policy = overlay_policy_from(inputs, &resolution, active_band);
        let gpu_budget = gpu_budget_from(&resolution, &extract_plan);
        let compute_budget = compute_budget_from(&resolution, &visibility, &inputs.compute_budget);
        let overlay_matrix = overlay_matrix_from(&overlay_policy, &inputs.overlay_policy);
        let preview_policy = preview_policy_from(active_band, &inputs);
        let particle_policy = particle_policy_from(&gpu_budget, active_band);
        Self {
            stamp: sim_stamp,
            active_band,
            world_lod_band,
            overlay_matrix,
            extract_plan,
            overlay_policy,
            gpu_budget,
            compute_budget,
            preview_policy,
            particle_policy,
            visibility,
        }
    }

    /// Per-chunk compute pathfinding gate (policy-only; no `visibility_for_band` in consumers).
    #[must_use]
    pub fn pathfinding_active_at_compute_band(&self, band: WorldLodBand) -> bool {
        if !self.visibility.pathfinding_field || !self.compute_budget.heat_diffusion {
            return false;
        }
        match band {
            WorldLodBand::LocalTactical | WorldLodBand::Operational => true,
            WorldLodBand::Strategic | WorldLodBand::Macro => false,
        }
    }
}

#[must_use]
pub fn build_representation_inputs(
    camera: &CameraVisualState,
    zones: &LodZoneRegistry,
    budgets: &VisualBudgetSettings,
    cadence: &VisualCadence,
    stamp: SimStepStamp,
) -> RepresentationInputs {
    RepresentationInputs {
        camera: camera.clone(),
        zones: zones.clone(),
        stamp,
        budgets: RepresentationBudgets {
            visual: *budgets,
        },
        cadence: *cadence,
        overlay_policy: OverlayPolicyInputs::default(),
        compute_budget: ComputeBudgetInputs::default(),
        viewport_px: UVec2::ONE,
        world_view_bounds: WorldViewBounds::default(),
    }
}

#[must_use]
fn extract_plan_from_resolution(
    visibility: &WorldVisibilityMask,
    resolution: &WorldResolutionPolicy,
    band: RepresentationBand,
) -> WorldRepresentationExtractPlan {
    WorldRepresentationExtractPlan {
        fire_instances: visibility.fire_instances,
        fire_instance_cap: resolution.fire_instance_cap,
        chunk_heat_bin: resolution.chunk_heat_bin.max(1),
        overlay_only: matches!(band, RepresentationBand::OverlayOnly | RepresentationBand::Dormant),
    }
}

#[must_use]
fn overlay_policy_from(
    inputs: &RepresentationInputs,
    resolution: &WorldResolutionPolicy,
    band: RepresentationBand,
) -> OverlayPolicy {
    let scale = match band {
        RepresentationBand::Full | RepresentationBand::Tactical => 1.0,
        RepresentationBand::Strategic => 0.85,
        RepresentationBand::OverlayOnly => 0.65,
        RepresentationBand::Dormant => 0.35,
    };
    OverlayPolicy {
        fire_heat: true,
        chunk_heat_bin: resolution.chunk_heat_bin.max(1),
        overlay_hz_scale: (inputs.cadence.overlay_hz / inputs.budgets.visual.overlay_hz.max(0.25))
            .clamp(0.25, 1.25)
            * scale,
    }
}

#[must_use]
fn overlay_matrix_from(
    overlay: &OverlayPolicy,
    inputs: &OverlayPolicyInputs,
) -> OverlayChannelMatrix {
    OverlayChannelMatrix {
        fire_heat: overlay.fire_heat && inputs.fire_heat,
        logistics: inputs.logistics,
        ecology: inputs.ecology,
        chunk_heat_bin: overlay.chunk_heat_bin,
    }
}

#[must_use]
fn preview_policy_from(
    band: RepresentationBand,
    inputs: &RepresentationInputs,
) -> PreviewRepresentationPolicy {
    let authoritative_gpu = matches!(band, RepresentationBand::Full | RepresentationBand::Tactical);
    PreviewRepresentationPolicy {
        authoritative_gpu,
        cpu_raster_debug: !authoritative_gpu,
        partial_hz_scale: inputs.cadence.preview_hz / inputs.budgets.visual.preview_hz.max(0.25),
    }
}

#[must_use]
fn particle_policy_from(
    gpu_budget: &GpuBudgetPolicy,
    band: RepresentationBand,
) -> ParticleRepresentationPolicy {
    let instanced_draw = !matches!(band, RepresentationBand::Strategic | RepresentationBand::Dormant)
        && gpu_budget.particle_rows_cap > 0;
    ParticleRepresentationPolicy {
        instanced_draw,
        rows_cap: gpu_budget.particle_rows_cap,
    }
}

#[must_use]
fn gpu_budget_from(
    _resolution: &WorldResolutionPolicy,
    extract_plan: &WorldRepresentationExtractPlan,
) -> GpuBudgetPolicy {
    let cap = if extract_plan.fire_instance_cap == usize::MAX {
        extract_plan
            .fire_instance_cap
            .saturating_sub(0)
            .min(u32::MAX as usize) as u32
    } else {
        extract_plan.fire_instance_cap as u32
    };
    let active = if extract_plan.fire_instances {
        cap
    } else {
        0
    };
    GpuBudgetPolicy {
        fire_instance_cap: extract_plan.fire_instance_cap,
        particle_rows_cap: extract_plan.fire_instance_cap,
        reserved_capacity: cap.max(1),
        active_capacity: active,
    }
}

#[must_use]
fn compute_budget_from(
    resolution: &WorldResolutionPolicy,
    visibility: &WorldVisibilityMask,
    inputs: &ComputeBudgetInputs,
) -> ComputeBudgetPolicy {
    ComputeBudgetPolicy {
        dispatch_hz: resolution.compute_dispatch_hz,
        heat_diffusion: visibility.fire_chunk_heat && inputs.heat_diffusion,
        agent_steering: visibility.agent_steering && inputs.agent_steering,
    }
}

/// Single authoritative policy output per frame.
#[must_use]
pub fn build_representation_result(
    frame: &WorldRepresentationFrame,
    inputs: &RepresentationInputs,
) -> RepresentationResult {
    let mut result = RepresentationResult::from_frame_and_inputs(frame, inputs.stamp, inputs);
    apply_zone_policy(frame, &inputs.zones, &mut result);
    result
}

fn apply_zone_policy(
    frame: &WorldRepresentationFrame,
    zones: &LodZoneRegistry,
    result: &mut RepresentationResult,
) {
    if zones.zones.is_empty() {
        return;
    }
    let focus = frame.focus_chunk;
    let mut overlay_weight = 1.0f32;
    let mut cadence_scale = 1.0f32;
    let mut gpu_boost = 0.0f32;
    let mut compute_boost = 0.0f32;
    let mut band_steps = 0u8;
    for zone in &zones.zones {
        let dx = zone.center.x - focus.x as f32;
        let dy = zone.center.y - focus.y as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > zone.radius {
            continue;
        }
        overlay_weight = overlay_weight.max(zone.priority);
        cadence_scale = cadence_scale.max(0.75 + zone.priority * 0.35);
        match zone.class {
            LodZoneClass::Camera => {
                band_steps = band_steps.max((zone.priority * 1.5).ceil() as u8);
            }
            LodZoneClass::Combat | LodZoneClass::Projectile => {
                band_steps = band_steps.max((zone.priority * 2.0).ceil() as u8);
                gpu_boost = gpu_boost.max(zone.priority);
                result.compute_budget.heat_diffusion = true;
            }
            LodZoneClass::Mission | LodZoneClass::JumpPoint => {
                band_steps = band_steps.max(1);
                gpu_boost = gpu_boost.max(zone.priority * 0.85);
                compute_boost = compute_boost.max(zone.priority * 0.75);
            }
            LodZoneClass::Sensor => {
                compute_boost = compute_boost.max(zone.priority);
                result.compute_budget.agent_steering = true;
            }
            LodZoneClass::Editor => {
                overlay_weight = overlay_weight.max(0.9);
                cadence_scale = cadence_scale.max(1.1);
            }
        }
    }
    for _ in 0..band_steps.min(3) {
        result.world_lod_band = step_world_lod_toward_tactical(result.world_lod_band);
    }
    if band_steps > 0 {
        refresh_representation_result_from_lod_band(result);
    }
    result.overlay_policy.overlay_hz_scale = (result.overlay_policy.overlay_hz_scale
        * overlay_weight.sqrt()
        * cadence_scale)
        .clamp(0.35, 1.25);
    if gpu_boost > 0.0 {
        let active = result.gpu_budget.active_capacity as f32 * (1.0 + gpu_boost * 0.12);
        result.gpu_budget.active_capacity = active.round() as u32;
        result.gpu_budget.particle_rows_cap = result
            .gpu_budget
            .particle_rows_cap
            .saturating_add((gpu_boost * 8.0).round() as usize);
    }
    if compute_boost > 0.0 {
        result.compute_budget.dispatch_hz =
            (result.compute_budget.dispatch_hz * (1.0 + compute_boost * 0.1)).min(120.0);
    }
}

#[inline]
fn step_world_lod_toward_tactical(band: WorldLodBand) -> WorldLodBand {
    match band {
        WorldLodBand::Macro => WorldLodBand::Strategic,
        WorldLodBand::Strategic => WorldLodBand::Operational,
        WorldLodBand::Operational => WorldLodBand::LocalTactical,
        WorldLodBand::LocalTactical => WorldLodBand::LocalTactical,
    }
}

fn refresh_representation_result_from_lod_band(result: &mut RepresentationResult) {
    let band = result.world_lod_band;
    result.active_band = representation_band_from_world_lod(band);
    result.visibility = super::world_representation::visibility_for_band(band);
    let resolution = super::world_representation::resolution_for_band(band);
    result.extract_plan =
        extract_plan_from_resolution(&result.visibility, &resolution, result.active_band);
    result.overlay_policy.chunk_heat_bin = resolution.chunk_heat_bin.max(1);
    result.gpu_budget = gpu_budget_from(&resolution, &result.extract_plan);
    result.compute_budget =
        compute_budget_from(&resolution, &result.visibility, &ComputeBudgetInputs::default());
    result.overlay_matrix = overlay_matrix_from(&result.overlay_policy, &OverlayPolicyInputs::default());
    result.preview_policy = preview_policy_from(result.active_band, &RepresentationInputs {
        camera: CameraVisualState::default(),
        zones: LodZoneRegistry::default(),
        budgets: RepresentationBudgets::default(),
        cadence: VisualCadence::from(&VisualBudgetSettings::default()),
        stamp: result.stamp,
        overlay_policy: OverlayPolicyInputs::default(),
        compute_budget: ComputeBudgetInputs::default(),
        viewport_px: UVec2::ONE,
        world_view_bounds: WorldViewBounds::default(),
    });
    result.particle_policy = particle_policy_from(&result.gpu_budget, result.active_band);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2, Vec3};
    use crate::gui::world_representation::{
        LodZoneSource, OperationalLodZone, WorldLodBands,
    };

    #[test]
    fn world_lod_band_maps_to_representation_band() {
        assert_eq!(
            representation_band_from_world_lod(WorldLodBand::LocalTactical),
            RepresentationBand::Full
        );
        assert_eq!(
            representation_band_from_world_lod(WorldLodBand::Macro),
            RepresentationBand::OverlayOnly
        );
    }

    #[test]
    fn pathfinding_policy_respects_global_and_compute_band() {
        let tactical = RepresentationResult::default();
        assert!(tactical.pathfinding_active_at_compute_band(WorldLodBand::LocalTactical));
        assert!(!tactical.pathfinding_active_at_compute_band(WorldLodBand::Macro));

        let frame = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: super::super::world_representation::visibility_for_band(
                WorldLodBand::Strategic,
            ),
            resolution: super::super::world_representation::resolution_for_band(
                WorldLodBand::Strategic,
            ),
            ..Default::default()
        };
        let inputs = build_representation_inputs(
            &CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            SimStepStamp::new(1, 0),
        );
        let strategic = build_representation_result(&frame, &inputs);
        assert!(!strategic.pathfinding_active_at_compute_band(WorldLodBand::LocalTactical));
    }

    #[test]
    fn strategic_band_zeroes_gpu_instance_active_capacity() {
        let frame = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: super::super::world_representation::visibility_for_band(
                WorldLodBand::Strategic,
            ),
            resolution: super::super::world_representation::resolution_for_band(
                WorldLodBand::Strategic,
            ),
            ..Default::default()
        };
        let inputs = build_representation_inputs(
            &CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            SimStepStamp::new(1, 0),
        );
        let result = build_representation_result(&frame, &inputs);
        assert_eq!(result.active_band, RepresentationBand::Strategic);
        assert_eq!(result.gpu_budget.active_capacity, 0);
        assert!(!result.extract_plan.fire_instances);
    }

    #[test]
    fn combat_zone_steps_band_and_restores_gpu_capacity() {
        let frame = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: super::super::world_representation::visibility_for_band(
                WorldLodBand::Strategic,
            ),
            resolution: super::super::world_representation::resolution_for_band(
                WorldLodBand::Strategic,
            ),
            focus_chunk: IVec2::ZERO,
            ..Default::default()
        };
        let zones = LodZoneRegistry {
            zones: vec![OperationalLodZone {
                zone_id: 7,
                class: LodZoneClass::Combat,
                center: Vec3::ZERO,
                radius: 512.0,
                band: WorldLodBand::Operational,
                priority: 1.0,
                source: LodZoneSource::CombatFront,
            }],
        };
        let inputs = RepresentationInputs {
            camera: CameraVisualState::default(),
            zones,
            budgets: RepresentationBudgets::default(),
            cadence: VisualCadence::from(&VisualBudgetSettings::default()),
            stamp: SimStepStamp::new(2, 0),
            overlay_policy: OverlayPolicyInputs::default(),
            compute_budget: ComputeBudgetInputs::default(),
            viewport_px: UVec2::ONE,
            world_view_bounds: WorldViewBounds::default(),
        };
        let result = build_representation_result(&frame, &inputs);
        assert_eq!(result.world_lod_band, WorldLodBand::LocalTactical);
        assert!(result.extract_plan.fire_instances);
        assert!(result.gpu_budget.active_capacity > 0);
        assert!(result.compute_budget.heat_diffusion);
    }

    #[test]
    fn compute_budget_inputs_gate_heat_diffusion() {
        let frame = WorldRepresentationFrame::default();
        let mut inputs = build_representation_inputs(
            &CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            SimStepStamp::new(1, 0),
        );
        inputs.compute_budget.heat_diffusion = false;
        let result = build_representation_result(&frame, &inputs);
        assert!(!result.compute_budget.heat_diffusion);
    }

    #[test]
    fn strategic_band_shrinks_particle_policy_rows() {
        let mut frame = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: super::super::world_representation::visibility_for_band(
                WorldLodBand::Strategic,
            ),
            resolution: super::super::world_representation::resolution_for_band(
                WorldLodBand::Strategic,
            ),
            ..Default::default()
        };
        let inputs = build_representation_inputs(
            &CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            SimStepStamp::new(1, 0),
        );
        let strategic = build_representation_result(&frame, &inputs);
        frame.bands.global = WorldLodBand::LocalTactical;
        frame.visibility = super::super::world_representation::visibility_for_band(
            WorldLodBand::LocalTactical,
        );
        frame.resolution = super::super::world_representation::resolution_for_band(
            WorldLodBand::LocalTactical,
        );
        let tactical = build_representation_result(&frame, &inputs);
        assert!(tactical.gpu_budget.active_capacity > strategic.gpu_budget.active_capacity);
        assert!(tactical.particle_policy.rows_cap >= strategic.particle_policy.rows_cap);
    }
}
