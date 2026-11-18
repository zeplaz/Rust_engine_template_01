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
    apply_dam_control_intent_to_sites_system, breach_dam_on_strike_impact_system,
    enqueue_dam_flood_strike_once_system, enqueue_dam_flood_water_config_system,
    mirror_stands_into_dam_book_system, mirror_water_into_dam_book_system,
    spawn_dam_flood_cooling_intakes_system, sync_dam_control_to_barrier_intent_system,
    DamFloodCoolerTag,
};

use bevy::prelude::*;

use crate::strategic::ConstructionSite;

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
    /// Peak hydro MW seen at/after S4Operate (DF-6 matrix).
    pub s4_hydro_mw_peak: f32,
    /// DF-7: metres between strike impact and aim (latched on first impact).
    pub s5_impact_err_m: f32,
    /// DF-8/9: sim seconds from breach/strike to G3 discharge rise.
    pub s7_wave_arrival_g3_s: f32,
    /// DF-5/S9: dead-stand burn rate / live-stand burn rate (latched when > 1).
    pub s9_burn_rate_dead_over_live: f32,
    /// Wall-clock sim seconds when S5/S7 breach began (for G3 arrival).
    pub breach_sim_s: f32,
    /// G3 discharge sample at breach start (baseline for arrival detect).
    pub breach_g3_baseline_m3s: f32,
}

impl Default for DamFloodBenchmarkBook {
    fn default() -> Self {
        Self {
            stage: DamFloodStage::S0River,
            stage_entry_tick: [0; 10],
            water_mass_m3: 0.0,
            inundated_tiles: 0,
            dam_integrity_mean: 1.0,
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
            s4_hydro_mw_peak: 0.0,
            s5_impact_err_m: 999.0,
            s7_wave_arrival_g3_s: 0.0,
            s9_burn_rate_dead_over_live: 0.0,
            breach_sim_s: -1.0,
            breach_g3_baseline_m3s: 0.0,
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
                    force_dam_flood_sites_operational_system
                        .after(spawn_dam_flood_cooling_intakes_system),
                    apply_dam_control_intent_to_sites_system
                        .after(force_dam_flood_sites_operational_system),
                    sync_dam_control_to_barrier_intent_system
                        .after(apply_dam_control_intent_to_sites_system),
                    mirror_water_into_dam_book_system,
                    mirror_stands_into_dam_book_system.after(mirror_water_into_dam_book_system),
                    enqueue_dam_flood_strike_once_system,
                    breach_dam_on_strike_impact_system.after(enqueue_dam_flood_strike_once_system),
                ),
            )
            .register_type::<DamFloodStage>()
            .register_type::<DamFloodBenchmarkBook>()
            .register_type::<DamControlIntent>();
    }
}

/// Test/experiment: advance DamFlood dam sites to Operational so barrier install + selection work.
/// Does not invent water truth — only clears the construction material stall for the scenario dam.
pub fn force_dam_flood_sites_operational_system(
    book: Res<DamFloodBenchmarkBook>,
    mut sites: Query<(
        &mut ConstructionSite,
        Option<&mut crate::construction::SiteStageProgress>,
        Option<&mut crate::strategic::SiteResourceManifest>,
    )>,
) {
    if !book.seeded {
        return;
    }
    use crate::strategic::{SiteArchetype, SiteConstructionPhase};
    for (mut site, mut stage, mut manifest) in &mut sites {
        if site.archetype != SiteArchetype::Dam {
            continue;
        }
        if site.phase == SiteConstructionPhase::Operational {
            continue;
        }
        if let Some(ref mut m) = manifest {
            m.delivered_concrete = m.concrete_required;
            m.delivered_steel = m.steel_required;
            m.delivered_fuel = m.fuel_required;
            m.delivered_machinery = m.machinery_required;
        }
        site.phase = SiteConstructionPhase::Operational;
        site.operational_readiness = 1.0;
        if let Some(ref mut st) = stage {
            st.progress = 1.0;
        }
    }
}
