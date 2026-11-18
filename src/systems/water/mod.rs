//! General surface-water sim — sole depth/flux authority (PLAN-DAM-FLOOD-BENCHMARK-v1 DF-2 / §0).
//!
//! **One fluid.** Depth + flux are the truth. River / lake / ocean are *labels* on that fluid
//! (channel vs standing body), not separate stacked waters. A river entering a lake is the same
//! water changing regime — presentation must never draw a river ribbon on lake tiles.
//!
//! Used by player-built dams, hydro/thermal cooling intakes, floods, and the DamFlood
//! **example** scene. The benchmark configures this API; it does not own a parallel solve.

mod barrier_ops;
mod cooling_intake;
mod flood_consequences;
mod gauge;
mod hydro;
mod probe;
mod salinity;
mod solver_tick;
mod static_depth;
mod surface_water;
mod visual_frame;
mod gpu_field;
mod water_runtime;

pub use barrier_ops::{
    apply_barrier_breach_request_system, apply_barrier_install_request_system,
    BarrierBreachRequest, BarrierBreachSpec, BarrierInstallRequest, BarrierInstallSpec,
};
pub use cooling_intake::{
    evaluate_cooling_from_stage, sample_intake_stage_m, update_cooling_intake_system, CoolingIntake,
    CoolingIntakeSource, CoolingWaterState,
};
pub use flood_consequences::{
    cell_speed_mps, publish_flood_consequences_system, tile_flood_impassable, FloodRouteBlockBook,
    FloodSiteReadinessBook, FLOOD_ROUTE_DEPTH_M, FLOOD_ROUTE_SPEED_MPS,
};
pub use gauge::{
    publish_gauge_readings_system, GaugeBook, GaugeId, GaugeReading, GaugeSite, GaugeBookPlugin,
};
pub use hydro::{update_hydro_generation_system, HydroGenerationState};
pub use visual_frame::{publish_surface_water_visual_frame_system, SurfaceWaterVisualFrame};
pub use gpu_field::{
    field_texel_step, pack_composite_water_field_rgba8, pack_surface_water_field_rgba8,
    pack_water_texel, publish_surface_water_gpu_field_system, SurfaceWaterGpuField,
    SurfaceWaterGpuFieldMeta,
    DEPTH_SCALE_M, FLOW_SPEED_SCALE, GPU_FIELD_MAX_EDGE,
};
pub use solver_tick::{
    project_surface_water_to_substrate_system, publish_gauges_from_solver_system,
    seed_surface_water_from_request_system, step_surface_water_system, SurfaceWaterRoiRequest,
    SurfaceWaterRoiSpec, SurfaceWaterSeedLatch, SurfaceWaterSolverPlugin,
};
pub use surface_water::{
    hydro_power_mw, measure_dam_break_front_distance, ritter_front_speed_mps, BarrierHydraulics,
    ReservoirLevelPool, SurfaceWaterField, SurfaceWaterMode, G_MPS2, SURFACE_WATER_SCHEMA_V1,
};
pub use probe::WaterTileProbe;
pub use salinity::WorldWaterSalinity;
pub use static_depth::WorldStaticWaterDepth;
pub use water_runtime::{
    tile_is_water_prefer_runtime, tile_is_wet, water_look_from_depth_speed, WaterClassRuntime,
    CHANNEL_LOOK_SPEED, DRY_DEPTH_M, DRY_HOLD_S, WET_DEPTH_M,
};

use bevy::prelude::*;

/// Per-unit hydro machine: operator setpoint × availability (damage / trip).
#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub struct DamTurbineUnit {
    pub setpoint_m3s: f32,
    /// 1 = online, 0 = shutdown (damage, trip, or operator isolate). Caps effective Q.
    pub available01: f32,
}

impl Default for DamTurbineUnit {
    fn default() -> Self {
        Self {
            setpoint_m3s: 0.0,
            available01: 1.0,
        }
    }
}

impl DamTurbineUnit {
    #[must_use]
    pub fn effective_m3s(self) -> f32 {
        self.setpoint_m3s.max(0.0) * self.available01.clamp(0.0, 1.0)
    }
}

pub const DAM_TURBINE_UNIT_COUNT: usize = 3;

/// Operator / construction / scenario control of a barrier + outlets + turbine bank.
/// Same resource a player dam or the DamFlood scenario writes — not scene-private.
///
/// Channels (real dam ops, not binary open/close):
/// - `outlet_open01` — low-level / bottom outlet
/// - `mid_outlet_open01` — mid-level / irrigation / compensate path
/// - `gate_open01` — gated spillway bays
/// - `turbines[]` — optional hydro units; damaged units trip → use outlets to augment
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct BarrierControlIntent {
    /// Gated spillway bay opening 0..1.
    pub gate_open01: f32,
    /// Low-level / bottom outlet 0..1.
    pub outlet_open01: f32,
    /// Mid-level outlet 0..1 (augment when turbines trip, irrigation, staged release).
    pub mid_outlet_open01: f32,
    /// Aggregate effective turbine Q (sum of units) — kept for hydro + older readers.
    pub turbine_flow_setpoint_m3s: f32,
    /// Per-unit bank (slots may be unused / setpoint 0).
    pub turbines: [DamTurbineUnit; DAM_TURBINE_UNIT_COUNT],
    /// When true and a crest exists with outlets mostly closed, prefer level-pool routing.
    pub prefer_level_pool: bool,
}

impl Default for BarrierControlIntent {
    fn default() -> Self {
        Self {
            gate_open01: 0.0,
            outlet_open01: 1.0,
            mid_outlet_open01: 0.0,
            turbine_flow_setpoint_m3s: 0.0,
            turbines: [DamTurbineUnit::default(); DAM_TURBINE_UNIT_COUNT],
            prefer_level_pool: false,
        }
    }
}

impl BarrierControlIntent {
    /// Refresh aggregate from the turbine bank (call after editing units).
    pub fn sync_turbine_aggregate(&mut self) {
        self.turbine_flow_setpoint_m3s = self.effective_turbine_m3s();
    }

    #[must_use]
    pub fn effective_turbine_m3s(&self) -> f32 {
        self.turbines.iter().map(|t| t.effective_m3s()).sum()
    }

    #[must_use]
    pub fn commanded_turbine_m3s(&self) -> f32 {
        self.turbines.iter().map(|t| t.setpoint_m3s.max(0.0)).sum()
    }

    /// Flow the operator wanted from turbines but cannot get (damage / trip).
    #[must_use]
    pub fn lost_turbine_m3s(&self) -> f32 {
        (self.commanded_turbine_m3s() - self.effective_turbine_m3s()).max(0.0)
    }

    /// Combined controlled orifice opening used for "mostly shut" checks.
    #[must_use]
    pub fn combined_outlet_open01(&self) -> f32 {
        self.outlet_open01
            .max(self.mid_outlet_open01)
            .clamp(0.0, 1.0)
    }

    /// Seed unit 0 from a legacy single turbine setpoint (scenario RON / old UI).
    pub fn set_legacy_turbine_setpoint(&mut self, q_m3s: f32) {
        self.turbines[0].setpoint_m3s = q_m3s.max(0.0);
        self.turbines[0].available01 = self.turbines[0].available01.clamp(0.0, 1.0);
        if self.turbines[0].available01 <= 0.0 {
            self.turbines[0].available01 = 1.0;
        }
        for t in self.turbines.iter_mut().skip(1) {
            t.setpoint_m3s = 0.0;
        }
        self.sync_turbine_aggregate();
    }
}

/// Coarse dt for slow domains (reservoir fill, ecology clocks, fuel moisture, weather means).
/// Any scene/system may arm this — not DamFlood-only.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct SlowDomainTimeSkip {
    pub enabled: bool,
    pub dt_slow_secs: u32,
}

impl Default for SlowDomainTimeSkip {
    fn default() -> Self {
        Self {
            enabled: false,
            dt_slow_secs: 600,
        }
    }
}

/// Extra ecology/weather-mean seconds accumulated while [`SlowDomainTimeSkip`] is armed (DF-1 D4).
#[derive(Resource, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Resource)]
pub struct SlowDomainElapsedSecs(pub f64);

pub fn apply_slow_domain_time_skip(skip: &mut SlowDomainTimeSkip, enabled: bool, dt_slow_secs: u32) {
    skip.enabled = enabled;
    if dt_slow_secs > 0 {
        skip.dt_slow_secs = dt_slow_secs;
    }
}

/// Advance [`SlowDomainElapsedSecs`] once per sim due while time-skip is on.
pub fn advance_slow_domain_elapsed_system(
    skip: Res<SlowDomainTimeSkip>,
    ctrl: Res<crate::systems::sim_control::SimControlState>,
    clock: Res<crate::systems::sim_control::SimStepClock>,
    mut elapsed: ResMut<SlowDomainElapsedSecs>,
) {
    if !skip.enabled || !ctrl.should_tick() || clock.due == 0 {
        return;
    }
    elapsed.0 += f64::from(skip.dt_slow_secs.max(1)) * f64::from(clock.due);
}

#[must_use]
pub fn slow_domain_fast_quiescent(
    skip: &SlowDomainTimeSkip,
    fire_heat_cells: u32,
    ballistic_in_flight: u32,
    water_solver_active: bool,
) -> bool {
    if !skip.enabled {
        return true;
    }
    fire_heat_cells == 0 && ballistic_in_flight == 0 && !water_solver_active
}

#[cfg(test)]
mod slow_domain_tests {
    use super::*;

    #[test]
    fn time_skip_quiescent_when_disabled() {
        let skip = SlowDomainTimeSkip::default();
        assert!(slow_domain_fast_quiescent(&skip, 9, 2, true));
    }

    #[test]
    fn time_skip_quiescent_requires_quiet_fast_domains() {
        let mut skip = SlowDomainTimeSkip::default();
        apply_slow_domain_time_skip(&mut skip, true, 600);
        assert!(!slow_domain_fast_quiescent(&skip, 1, 0, false));
        assert!(!slow_domain_fast_quiescent(&skip, 0, 1, false));
        assert!(!slow_domain_fast_quiescent(&skip, 0, 0, true));
        assert!(slow_domain_fast_quiescent(&skip, 0, 0, false));
    }
}

pub struct WaterSimPlugin;

impl Plugin for WaterSimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BarrierControlIntent>()
            .init_resource::<SlowDomainTimeSkip>()
            .init_resource::<SlowDomainElapsedSecs>()
            .init_resource::<BarrierInstallRequest>()
            .init_resource::<BarrierBreachRequest>()
            .init_resource::<HydroGenerationState>()
            .init_resource::<SurfaceWaterVisualFrame>()
            .init_resource::<SurfaceWaterGpuField>()
            .init_resource::<SurfaceWaterGpuFieldMeta>()
            .init_resource::<WorldStaticWaterDepth>()
            .init_resource::<FloodRouteBlockBook>()
            .init_resource::<FloodSiteReadinessBook>()
            .add_plugins(GaugeBookPlugin)
            .add_plugins(SurfaceWaterSolverPlugin)
            .add_systems(
                Update,
                (
                    advance_slow_domain_elapsed_system
                        .before(step_surface_water_system),
                    apply_barrier_install_request_system
                        .before(step_surface_water_system),
                    apply_barrier_breach_request_system
                        .before(step_surface_water_system),
                    update_cooling_intake_system.after(step_surface_water_system),
                    update_hydro_generation_system.after(step_surface_water_system),
                    publish_flood_consequences_system.after(step_surface_water_system),
                    publish_surface_water_visual_frame_system.after(step_surface_water_system),
                    publish_surface_water_gpu_field_system
                        .after(publish_surface_water_visual_frame_system),
                ),
            )
            .register_type::<BarrierControlIntent>()
            .register_type::<DamTurbineUnit>()
            .register_type::<SlowDomainTimeSkip>()
            .register_type::<SlowDomainElapsedSecs>()
            .register_type::<HydroGenerationState>()
            .register_type::<CoolingIntake>()
            .register_type::<CoolingIntakeSource>()
            .register_type::<CoolingWaterState>();
    }
}
