//! General surface-water sim — sole depth/flux authority (PLAN-DAM-FLOOD-BENCHMARK-v1 DF-2 / §0).
//!
//! **One fluid.** Depth + flux are the truth. River / lake / ocean are *labels* on that fluid
//! (channel vs standing body), not separate stacked waters. A river entering a lake is the same
//! water changing regime — presentation must never draw a river ribbon on lake tiles.
//!
//! Used by player-built dams, hydro/thermal cooling intakes, floods, and the DamFlood
//! **example** scene. The benchmark configures this API; it does not own a parallel solve.
//!
//! WM-1: **one map-sized field** (seeded once the world exists), solved over active tiles only
//! (`tiles`, `step_kernel`); the solver runs on the sim clock.

mod barrier_ops;
mod bed_follow;
pub mod sediment;
mod cooling_intake;
mod dam_faces;
mod field_read;
mod flood_consequences;
mod gauge;
mod hydro;
mod probe;
mod rain;
mod salinity;
mod solver_tick;
mod sources;
mod static_depth;
mod step_kernel;
mod surface_water;
mod tiles;
mod visual_frame;
mod field_feed;
mod gpu_field;
mod water_runtime;
mod wet_mask;

pub use barrier_ops::{
    apply_barrier_breach_request_system, apply_barrier_column_request_system,
    apply_barrier_install_request_system, apply_water_obstacle_request_system,
    BarrierBreachRequest, BarrierBreachSpec, BarrierColumnRequest, BarrierInstallRequest,
    BarrierInstallSpec, WaterObstacleRequest,
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
    field_texel_step, field_texture_dims, pack_surface_water_field_rgba8,
    pack_water_texel, publish_surface_water_gpu_field_system, SurfaceWaterGpuField,
    SurfaceWaterGpuFieldMeta,
    DEPTH_SCALE_M, FLOW_SPEED_SCALE, GPU_FIELD_MAX_EDGE,
};
pub use solver_tick::{
    publish_gauges_from_solver_system, seed_surface_water_system, step_surface_water_system, SurfaceWaterBaseflow,
    SurfaceWaterSeedLatch, SurfaceWaterSolverPlugin, MAX_WATER_STEPS_PER_FRAME,
};
pub use dam_faces::{DamFlows, DamGeometry, DamOps, DamSite, FaceFlows, CHANNEL_BAND_M};
pub use surface_water::{
    hydro_power_mw, measure_dam_break_front_distance, ritter_front_speed_mps, BarrierHydraulics,
    PoolWindow, ReservoirLevelPool, SurfaceWaterField, SurfaceWaterMode, FAR_FIELD_2D_DT_CAP_S,
    FAR_FIELD_MIN_DT_S, G_MPS2, SURFACE_WATER_SCHEMA_V1,
};
pub use field_feed::{publish_water_dirty_feed_system, WaterFieldDirtyFeed};
pub use wet_mask::{update_water_wet_mask_system, WaterFieldWetMask, WET_MASK_DEPTH_M};
pub use tiles::{WaterTileGrid, WATER_SLEEP_STEPS, WATER_TILE_CELLS};
pub use probe::WaterTileProbe;
pub use salinity::WorldWaterSalinity;
pub use static_depth::WorldStaticWaterDepth;
pub use rain::{sample_weather_rain_system, SoilClass, DRIFT_REPORT_M, EVAP_MM_H, RAIN_MIN_MM_H, RAIN_WAKE_MM_H, SLOW_PASS_S};
pub use sources::{RiverSource, RIVER_MAX_SPEED_MPS};
pub use water_runtime::{water_look_from_depth_speed, CHANNEL_LOOK_SPEED, WET_DEPTH_M};

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
}

impl Default for BarrierControlIntent {
    fn default() -> Self {
        Self {
            gate_open01: 0.0,
            outlet_open01: 1.0,
            mid_outlet_open01: 0.0,
            turbine_flow_setpoint_m3s: 0.0,
            turbines: [DamTurbineUnit::default(); DAM_TURBINE_UNIT_COUNT],
        }
    }
}

/// Control of each dam by its owner entity (WM-3). Construction publishes (one entry per dam
/// site); water is the only reader, applying each entry to the dam of that owner in the field.
/// A dam with no entry (bare scenario rows, a dam not yet mirrored) follows the global
/// [`BarrierControlIntent`].
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct BarrierControlBook {
    by_dam: std::collections::HashMap<Entity, DamOps>,
}

impl BarrierControlBook {
    /// Publish `ops` for `dam`; returns true when the entry changed.
    pub fn set(&mut self, dam: Entity, ops: DamOps) -> bool {
        self.by_dam.insert(dam, ops) != Some(ops)
    }

    #[must_use]
    pub fn get(&self, dam: Entity) -> Option<DamOps> {
        self.by_dam.get(&dam).copied()
    }

    /// Drop the entries of dams that no longer exist; true when any was removed.
    pub fn retain(&mut self, mut keep: impl FnMut(Entity) -> bool) -> bool {
        let n = self.by_dam.len();
        self.by_dam.retain(|e, _| keep(*e));
        self.by_dam.len() != n
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.by_dam.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_dam.is_empty()
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
            .init_resource::<BarrierControlBook>()
            .init_resource::<SlowDomainTimeSkip>()
            .init_resource::<SlowDomainElapsedSecs>()
            .init_resource::<BarrierInstallRequest>()
            .init_resource::<BarrierBreachRequest>()
            .init_resource::<BarrierColumnRequest>()
            .init_resource::<WaterObstacleRequest>()
            .init_resource::<HydroGenerationState>()
            .init_resource::<SurfaceWaterVisualFrame>()
            .init_resource::<SurfaceWaterGpuField>()
            .init_resource::<SurfaceWaterGpuFieldMeta>()
            .init_resource::<WaterFieldDirtyFeed>()
            .init_resource::<WaterFieldWetMask>()
            .init_resource::<WorldStaticWaterDepth>()
            .init_resource::<FloodRouteBlockBook>()
            .init_resource::<FloodSiteReadinessBook>()
            .add_plugins(GaugeBookPlugin)
            .add_plugins(SurfaceWaterSolverPlugin)
            .init_resource::<crate::terrain::ground_deform::GroundDeltaJournal>()
            .init_resource::<crate::terrain::ground_deform::GroundDeformQueue>()
            // DW-5: debris in fast water moves downstream (queued to terrain, drained next frame).
            .add_systems(
                Update,
                sediment::wash_debris_downstream_system.after(step_surface_water_system),
            )
            .add_systems(
                Update,
                bed_follow::follow_ground_deform_bed_system
                    .after(crate::terrain::ground_deform::GroundDeformSet::Drain)
                    // Before the seed: a field seeded this frame samples post-drain chunk elevation.
                    .before(seed_surface_water_system)
                    .before(step_surface_water_system),
            )
            .add_systems(
                Update,
                (
                    advance_slow_domain_elapsed_system
                        .before(step_surface_water_system),
                    apply_barrier_install_request_system
                        .before(step_surface_water_system),
                    apply_barrier_breach_request_system
                        .before(step_surface_water_system),
                    apply_barrier_column_request_system
                        .after(apply_barrier_install_request_system)
                        .before(step_surface_water_system),
                    apply_water_obstacle_request_system.before(step_surface_water_system),
                    update_cooling_intake_system.after(step_surface_water_system),
                    update_hydro_generation_system.after(step_surface_water_system),
                    publish_flood_consequences_system.after(step_surface_water_system),
                    publish_surface_water_visual_frame_system.after(step_surface_water_system),
                    // WM-5 / WM-6: the one consumer of the moved-chunk queue; every render-side
                    // consumer reads the same feed after this.
                    publish_water_dirty_feed_system.after(step_surface_water_system),
                    publish_surface_water_gpu_field_system
                        .after(publish_surface_water_visual_frame_system)
                        .after(publish_water_dirty_feed_system),
                    update_water_wet_mask_system.after(publish_water_dirty_feed_system),
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
