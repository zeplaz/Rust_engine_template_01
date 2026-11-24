//! DamFlood **example scenario** harness (PLAN-DAM-FLOOD-BENCHMARK-v1).
//!
//! Lives under [`crate::scenario`] — not a sim domain. Configures general water / gauge /
//! barrier / time-skip so a player-style dam → flood chain can be scored.
//! Does **not** own water truth. See [`crate::dev::plan_scenario_vs_systems_and_research_v1`].

mod author_valley;
mod fixture;
mod placement;
mod scene_seed;

pub use author_valley::{
    author_dam_flood_valley_catalog_look_system, author_dam_flood_valley_rearm_water_system,
    author_dam_flood_valley_system, nearest_dry_tile, nearest_dry_tile_depth,
    DamFloodValleyAuthorLatch,
};
pub use fixture::{load_dam_valley_fixture, DamValleyFixture, DAM_VALLEY_FIXTURE_REL};
pub use placement::dam_channel_span_footprint;
pub use scene_seed::{
    apply_dam_control_intent_to_sites_system, enqueue_dam_flood_water_config_system,
    mirror_dam_integrity_into_book_system, mirror_stands_into_dam_book_system, mirror_water_into_dam_book_system,
    spawn_dam_flood_cooling_intakes_system, sync_dam_control_to_barrier_intent_system,
    DamFloodCoolerTag,
};

use bevy::prelude::*;


/// Benchmark narrative stage (counts-only dump). Sole writer: scenario / harness ops.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Default)]
pub enum DamFloodStage {
    #[default]
    S0River,
    S1Build,
    S2Impound,
    S3Drown,
    S4Operate,
    S5Strike,
    S6Leak,
    S7Collapse,
    S8Recede,
    S9DryBurn,
}

impl DamFloodStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::S0River => "S0River",
            Self::S1Build => "S1Build",
            Self::S2Impound => "S2Impound",
            Self::S3Drown => "S3Drown",
            Self::S4Operate => "S4Operate",
            Self::S5Strike => "S5Strike",
            Self::S6Leak => "S6Leak",
            Self::S7Collapse => "S7Collapse",
            Self::S8Recede => "S8Recede",
            Self::S9DryBurn => "S9DryBurn",
        }
    }
}

/// Benchmark dump counters + stage clock (mirrors general water; not a second water writer).
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct DamFloodBenchmarkBook {
    pub stage: DamFloodStage,
    pub stage_entry_tick: [u64; 10],
    pub water_mass_m3: f64,
    pub inundated_tiles: u32,
    pub dam_integrity_mean: f32,
    pub outlet_flow_m3s: f32,
    pub spillway_flow_m3s: f32,
    pub breach_flow_m3s: f32,
    pub stand_live: u32,
    pub stand_drowned: u32,
    pub stand_dead: u32,
    pub fuel_moisture_mean: f32,
    pub hydro_mw: f32,
    pub thermal_mw: f32,
    pub solver_ms: f32,
    pub fixture_id: String,
    pub seeded: bool,
    /// Latched while stage == S0 (DF-2 matrix).
    pub s0_gauge_balance_err: f32,
    /// Latched while stage == S2Impound (DF-2 matrix).
    pub s2_downstream_ratio: f32,
    /// Peak drowned stand count (S3).
    pub s3_drowned_peak: u32,
    /// Peak hydro MW seen at/after S4Operate (DF-6 matrix).
    pub s4_hydro_mw_peak: f32,
    /// DF-7: metres between strike impact and aim (latched on first impact).
    pub s5_impact_err_m: f32,
    /// DF-8/9: water-solver seconds from the breach to the breach wave's arrival at G3 (the first
    /// time G3's real cross-section flux stays above its operate-stage level by a wave's margin).
    pub s7_wave_arrival_g3_s: f32,
    /// DF-5/S9: dead-stand burn rate / live-stand burn rate (latched when > 1).
    pub s9_burn_rate_dead_over_live: f32,
    /// Water-solver clock (`SurfaceWaterField::elapsed_s`) when the dam broke (-1 = intact). The
    /// solver clock, not sim time + time skip: skipped hours are not water that travelled.
    pub breach_sim_s: f32,
    /// G3 flux (m3/s) before the breach: its level during the operate stage with the gates open.
    pub breach_g3_baseline_m3s: f32,
    /// Running level of G3's flux while the dam operates (-1 = not sampled yet).
    pub g3_flux_operate_m3s: f32,
    /// Consecutive frames G3's flux stood above the wave margin.
    pub g3_wave_hits: u8,
}

impl Default for DamFloodBenchmarkBook {
    fn default() -> Self {
        Self {
            stage: DamFloodStage::S0River,
            stage_entry_tick: [0; 10],
            water_mass_m3: 0.0,
            inundated_tiles: 0,
            dam_integrity_mean: 0.0,
            outlet_flow_m3s: 0.0,
            spillway_flow_m3s: 0.0,
            breach_flow_m3s: 0.0,
            stand_live: 0,
            stand_drowned: 0,
            stand_dead: 0,
            fuel_moisture_mean: 0.0,
            hydro_mw: 0.0,
            thermal_mw: 0.0,
            solver_ms: 0.0,
            fixture_id: String::new(),
            seeded: false,
            s0_gauge_balance_err: 1.0,
            s2_downstream_ratio: 1.0,
            s3_drowned_peak: 0,
            s4_hydro_mw_peak: 0.0,
            s5_impact_err_m: 999.0,
            s7_wave_arrival_g3_s: 0.0,
            s9_burn_rate_dead_over_live: 0.0,
            breach_sim_s: -1.0,
            breach_g3_baseline_m3s: 0.0,
            g3_flux_operate_m3s: -1.0,
            g3_wave_hits: 0,
        }
    }
}

impl DamFloodBenchmarkBook {
    pub fn enter_stage(&mut self, stage: DamFloodStage, tick: u64) {
        self.stage = stage;
        let i = stage as usize;
        if i < self.stage_entry_tick.len() && self.stage_entry_tick[i] == 0 {
            self.stage_entry_tick[i] = tick;
        }
    }
}

/// Scenario play-faction owner for DamFlood sites (formations use faction 0).
/// Empty entity until harness seeds; CommitSite / dam commit reuse this for control tests.
#[derive(Resource, Debug, Clone, Copy)]
pub struct DamFloodPlayerOwner(pub Option<Entity>);

impl Default for DamFloodPlayerOwner {
    fn default() -> Self {
        Self(None)
    }
}

/// Scenario / ops-card dam setpoints — synced into general [`crate::systems::water::BarrierControlIntent`].
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct DamControlIntent {
    /// Gated spillway bays.
    pub gate_open01: f32,
    /// Low-level / bottom outlet.
    pub outlet_open01: f32,
    /// Mid-level outlet (augment when turbines trip).
    pub mid_outlet_open01: f32,
    /// Aggregate effective turbine Q (mirrors bank).
    pub turbine_flow_setpoint_m3s: f32,
    pub turbines: [crate::systems::water::DamTurbineUnit;
        crate::systems::water::DAM_TURBINE_UNIT_COUNT],
    /// Which turbine slot the ops card adjusts (0..N-1).
    pub focus_turbine: u8,
}

impl Default for DamControlIntent {
    fn default() -> Self {
        Self {
            gate_open01: 0.0,
            outlet_open01: 1.0,
            mid_outlet_open01: 0.0,
            turbine_flow_setpoint_m3s: 0.0,
            turbines: [crate::systems::water::DamTurbineUnit::default();
                crate::systems::water::DAM_TURBINE_UNIT_COUNT],
            focus_turbine: 0,
        }
    }
}

impl DamControlIntent {
    pub fn sync_turbine_aggregate(&mut self) {
        self.turbine_flow_setpoint_m3s = self
            .turbines
            .iter()
            .map(|t| t.effective_m3s())
            .sum();
    }

    pub fn set_legacy_turbine_setpoint(&mut self, q_m3s: f32) {
        self.turbines[0].setpoint_m3s = q_m3s.max(0.0);
        if self.turbines[0].available01 <= 0.0 {
            self.turbines[0].available01 = 1.0;
        }
        for t in self.turbines.iter_mut().skip(1) {
            t.setpoint_m3s = 0.0;
        }
        self.sync_turbine_aggregate();
    }

    #[must_use]
    pub fn lost_turbine_m3s(&self) -> f32 {
        let commanded: f32 = self.turbines.iter().map(|t| t.setpoint_m3s.max(0.0)).sum();
        let effective: f32 = self.turbines.iter().map(|t| t.effective_m3s()).sum();
        (commanded - effective).max(0.0)
    }
}

/// Registers scenario harness resources + systems. Water truth stays in [`crate::systems::water`].
pub struct DamFloodPlugin;

impl Plugin for DamFloodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DamFloodBenchmarkBook>()
            .init_resource::<DamControlIntent>()
            .add_systems(PreStartup, insert_dam_flood_faction_start_system)
            .init_resource::<DamFloodPlayerOwner>()
            .init_resource::<scene_seed::DamFloodWaterConfigLatch>()
            .init_resource::<DamFloodValleyAuthorLatch>()
            .add_systems(
                Update,
                (
                    author_dam_flood_valley_system,
                    author_dam_flood_valley_catalog_look_system
                        .after(author_dam_flood_valley_system),
                    author_dam_flood_valley_rearm_water_system
                        .after(author_dam_flood_valley_system),
                    enqueue_dam_flood_water_config_system
                        .after(author_dam_flood_valley_rearm_water_system),
                    spawn_dam_flood_cooling_intakes_system
                        .after(enqueue_dam_flood_water_config_system),
                    // CLOSE-S2: the dam is built by construction + logistics (no forced Operational).
                    apply_dam_control_intent_to_sites_system
                        .after(spawn_dam_flood_cooling_intakes_system),
                    sync_dam_control_to_barrier_intent_system
                        .after(apply_dam_control_intent_to_sites_system),
                    mirror_water_into_dam_book_system,
                    mirror_stands_into_dam_book_system.after(mirror_water_into_dam_book_system),
                    // CLOSE-S1: strike failsafe + breach-on-impact removed — the script's
                    // StrikeOrder flies a real warhead; blast → DamBody → leak/breach owns damage.
                    mirror_dam_integrity_into_book_system,
                ),
            )
            .register_type::<DamFloodStage>()
            .register_type::<DamFloodBenchmarkBook>()
            .register_type::<DamControlIntent>();
    }
}

/// DamFlood scene start: the fixture names its own faction start config (HQ near the narrows,
/// dam-sized stockpile). Inserted in PreStartup so the faction plugin's default doesn't apply.
fn insert_dam_flood_faction_start_system(
    mut commands: Commands,
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
) {
    if !launch.is_some_and(|l| l.test_scene.is_dam_flood()) {
        return;
    }
    let fixture = load_dam_valley_fixture();
    if let Some(cfg) = fixture
        .faction_start
        .as_deref()
        .and_then(crate::ai::faction::FactionStartConfig::load_path)
    {
        commands.insert_resource(cfg);
    }
}
