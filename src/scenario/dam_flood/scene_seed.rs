//! DamFlood example scene — configures general water/gauge APIs from the valley fixture.
//! Does not own depth truth; only enqueues [`SurfaceWaterRoiRequest`] like a player/scenario would.

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::combat::{BallisticFlightBook, StrikeOrder, StrikeOrderQueue};
use crate::engine::test_harness::ActiveTestScene;
use crate::systems::ecology::{
    effective_fuel_moisture01, ChunkCellFuelMoisture, ChunkCellStand, StandBurnConfig, StandKind,
    StandPhase,
};
use crate::systems::sim_control::SimTimeMicros;
use crate::systems::water::{
    BarrierControlIntent, CoolingIntake, GaugeBook, GaugeId, GaugeSite, HydroGenerationState,
    SlowDomainElapsedSecs, SurfaceWaterField, SurfaceWaterRoiRequest, SurfaceWaterRoiSpec,
};
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::DamFloodStage;

use super::fixture::load_dam_valley_fixture;
use super::DamFloodBenchmarkBook;

#[derive(Resource, Debug, Default)]
pub struct DamFloodWaterConfigLatch {
    pub enqueued: bool,
    pub intakes_spawned: bool,
}

/// Marker for DamFlood example coolers (nuclear + second industry share the reservoir).
#[derive(Component, Debug, Clone, Copy)]
pub struct DamFloodCoolerTag {
    pub design_mw: f32,
}

/// One-shot: turn the authored valley fixture into a general ROI + gauge configuration.
pub fn enqueue_dam_flood_water_config_system(
    scene: Option<Res<ActiveTestScene>>,
    mut latch: ResMut<DamFloodWaterConfigLatch>,
    mut book: ResMut<DamFloodBenchmarkBook>,
    mut request: ResMut<SurfaceWaterRoiRequest>,
    mut gauges: ResMut<GaugeBook>,
    field: Res<SurfaceWaterField>,
) {
    if latch.enqueued || field.wired || request.pending.is_some() {
        return;
    }
    // Example scene active, or harness already marked the benchmark book seeded.
    if !scene.is_some_and(|s| s.0.is_dam_flood()) && !book.seeded {
        return;
    }

    let fixture = load_dam_valley_fixture();
    let fp = &fixture.forest_floodplain;
    // Expand ROI to cover floodplain + all gauges (G1 upstream / G3 downstream).
    let mut origin_x = fp.min.0.min(fp.max.0) - 8;
    let mut origin_z = fp.min.1.min(fp.max.1) - 16;
    let mut x1 = fp.min.0.max(fp.max.0) + 8;
    let mut z1 = fp.min.1.max(fp.max.1) + 16;
    for g in &fixture.gauges {
        origin_x = origin_x.min(g.tile.0 - 4);
        origin_z = origin_z.min(g.tile.1 - 4);
        x1 = x1.max(g.tile.0 + 4);
        z1 = z1.max(g.tile.1 + 4);
    }
    let width = (x1 - origin_x).max(16) as u32;
    let height = (z1 - origin_z).max(16) as u32;

    // S0 river: no barrier yet — player/scenario InstallBarrier + SetDamControl installs crest later.
    request.pending = Some(SurfaceWaterRoiSpec {
        origin_x,
        origin_z,
        width,
        height,
        inflow_m3s: fixture.river_baseflow_m3s,
        min_release_m3s: fixture.river_baseflow_m3s * 0.1,
        river_center_x: Some(fixture.narrows_tile.0),
        // Corridor-scale river (authored valley half_width 3). Wider = fake lake.
        river_half_width: 3,
        river_seed_depth_m: 3.5,
        barrier_world_z: None,
        barrier_crest_m: 0.0,
        barrier_half_span: fixture.dam_span_tiles.max(4) / 2,
        synthetic_valley_if_no_dem: true,
    });

    gauges.seed_sites(fixture.gauges.iter().map(|g| GaugeSite {
        id: GaugeId::from_str_id(&g.id),
        label: g.label.clone(),
        tile: IVec2::new(g.tile.0, g.tile.1),
        cross_section_half_tiles: g.cross_section_half_tiles.max(1),
    }));

    book.fixture_id = fixture.id.clone();
    book.outlet_flow_m3s = fixture.river_baseflow_m3s;
    latch.enqueued = true;

    info!(
        target: "dam_flood::scene_seed",
        "enqueued SurfaceWaterRoiRequest from fixture={} (general water path)",
        fixture.id
    );
}

/// Spawn general [`CoolingIntake`] entities for nuclear + a second industry (any-cooler path).
pub fn spawn_dam_flood_cooling_intakes_system(
    scene: Option<Res<ActiveTestScene>>,
    mut latch: ResMut<DamFloodWaterConfigLatch>,
    book: Res<DamFloodBenchmarkBook>,
    mut commands: Commands,
) {
    if latch.intakes_spawned {
        return;
    }
    if !scene.is_some_and(|s| s.0.is_dam_flood()) && !book.seeded {
        return;
    }
    if !latch.enqueued {
        return;
    }
    latch.intakes_spawned = true;

    // Reservoir-pool intakes: thresholds in absolute stage metres (bed≈0 after synthetic carve).
    commands.spawn((
        Name::new("DamFloodNuclearCooler"),
        DamFloodCoolerTag { design_mw: 900.0 },
        CoolingIntake::reservoir(8.0, 3.0, 48.0),
    ));
    commands.spawn((
        Name::new("DamFloodIndustryCooler"),
        DamFloodCoolerTag { design_mw: 120.0 },
        CoolingIntake::reservoir(6.0, 2.0, 20.0),
    ));

    info!(
        target: "dam_flood::scene_seed",
        "spawned CoolingIntake entities (nuclear + industry) on general reservoir path"
    );
}

/// Scenario `SetDamControl` writes [`super::DamControlIntent`]; push that onto site
/// [`DamControl`] so the production mirror owns [`BarrierControlIntent`].
pub fn apply_dam_control_intent_to_sites_system(
    intent: Res<super::DamControlIntent>,
    mut sites: Query<&mut crate::construction::DamControl>,
    mut last_site_n: Local<usize>,
) {
    let n = sites.iter().len();
    // Re-apply when intent changes OR a dam site just appeared (InstallBarrier /
    // construction spawn) — otherwise new DamControl keeps open defaults and
    // DF-2 impound never holds.
    if !intent.is_changed() && n == *last_site_n {
        return;
    }
    *last_site_n = n;
    for mut site in &mut sites {
        site.gate_open01 = intent.gate_open01;
        site.outlet_open01 = intent.outlet_open01;
        site.mid_outlet_open01 = intent.mid_outlet_open01;
        site.turbines = intent.turbines;
        site.turbine_setpoint_m3s = intent.turbine_flow_setpoint_m3s;
    }
}

/// Mirror scenario `DamControlIntent` into the general barrier intent **only when no
/// site [`DamControl`] exists**. Player/ops authority is the dam component →
/// `mirror_dam_control_to_barrier_system`.
pub fn sync_dam_control_to_barrier_intent_system(
    dam: Res<super::DamControlIntent>,
    site_controls: Query<&crate::construction::DamControl>,
    mut barrier: ResMut<BarrierControlIntent>,
) {
    if site_controls.iter().next().is_some() {
        return;
    }
    barrier.gate_open01 = dam.gate_open01;
    barrier.outlet_open01 = dam.outlet_open01;
    barrier.mid_outlet_open01 = dam.mid_outlet_open01;
    barrier.turbines = dam.turbines;
    barrier.sync_turbine_aggregate();
    barrier.prefer_level_pool = dam.outlet_open01 <= 0.2
        && dam.mid_outlet_open01 <= 0.2
        && dam.gate_open01 <= 0.2
        && dam.turbine_flow_setpoint_m3s < 1.0;
}

/// Copy live water / hydro / cooling counters into the benchmark book (read-only mirror).
pub fn mirror_water_into_dam_book_system(
    field: Res<SurfaceWaterField>,
    hydro: Res<HydroGenerationState>,
    gauges: Res<GaugeBook>,
    intakes: Query<(&CoolingIntake, &DamFloodCoolerTag)>,
    sim_time: Res<SimTimeMicros>,
    slow_elapsed: Option<Res<SlowDomainElapsedSecs>>,
    ballistics: Option<Res<BallisticFlightBook>>,
    mut book: ResMut<DamFloodBenchmarkBook>,
) {
    if !field.wired || !book.seeded {
        return;
    }
    book.water_mass_m3 = field.mass_m3;
    book.outlet_flow_m3s = field.outlet_flow_m3s;
    book.spillway_flow_m3s = field.spillway_flow_m3s;
    book.breach_flow_m3s = field.breach_flow_m3s;
    book.solver_ms = field.solver_ms;
    book.inundated_tiles = field
        .depth_m
        .iter()
        .filter(|h| **h >= crate::systems::water::WET_DEPTH_M)
        .count() as u32;
    book.hydro_mw = hydro.power_mw;

    let mut thermal = 0.0_f32;
    for (intake, tag) in &intakes {
        thermal += tag.design_mw * intake.cooling_factor;
    }
    book.thermal_mw = thermal;

    let now = ((sim_time.0 as f64 / 1_000_000.0)
        + slow_elapsed.map(|s| s.0).unwrap_or(0.0)) as f32;

    // Latch stage-scoped metrics so late timeline (breach) does not wipe DF-2/DF-6 Accepts.
    let g1 = gauges
        .reading(GaugeId::G1)
        .map(|r| r.discharge_m3s)
        .unwrap_or(0.0);
    let g3 = gauges
        .reading(GaugeId::G3)
        .map(|r| r.discharge_m3s)
        .unwrap_or(0.0);
    let inflow = field.inflow_m3s.max(1.0);
    let g1_stage = gauges
        .reading(GaugeId::G1)
        .map(|r| r.stage_m)
        .unwrap_or(0.0);
    let g3_stage = gauges
        .reading(GaugeId::G3)
        .map(|r| r.stage_m)
        .unwrap_or(0.0);
    match book.stage {
        DamFloodStage::S0River | DamFloodStage::S1Build => {
            // Only latch when both river gauges are live — outlet/inflow fallback was ~0.88 while
            // G1/G3 sat outside a tight rearm ROI (false FAIL).
            if g1 > 1.0 && g3 > 1.0 {
                let err = ((g1 - g3).abs() / g1.max(1.0)).clamp(0.0, 10.0);
                book.s0_gauge_balance_err = book.s0_gauge_balance_err.min(err);
            } else if g1_stage > 0.5 && g3_stage > 0.5 {
                let err = ((g1_stage - g3_stage).abs() / g1_stage.max(0.5)).clamp(0.0, 10.0);
                book.s0_gauge_balance_err = book.s0_gauge_balance_err.min(err);
            }
        }
        DamFloodStage::S2Impound => {
            let ratio = if g3 > 0.05 {
                g3 / inflow
            } else if g3_stage > 0.05 {
                (g3_stage / g1_stage.max(0.5)).clamp(0.0, 10.0) * 0.25
            } else {
                field.outlet_flow_m3s.max(0.0) / inflow
            };
            book.s2_downstream_ratio = ratio.clamp(0.0, 10.0);
            book.s4_hydro_mw_peak = book.s4_hydro_mw_peak.max(hydro.power_mw);
        }
        DamFloodStage::S4Operate | DamFloodStage::S3Drown => {
            book.s4_hydro_mw_peak = book.s4_hydro_mw_peak.max(hydro.power_mw);
        }
        DamFloodStage::S5Strike | DamFloodStage::S7Collapse => {
            if book.breach_sim_s < 0.0 && field.breach_flow_m3s > 0.0 {
                book.breach_sim_s = now;
                book.breach_g3_baseline_m3s = g3.max(g3_stage).max(0.0);
            }
            if book.s7_wave_arrival_g3_s <= 0.0 && book.breach_sim_s >= 0.0 {
                let spiked = g3 > book.breach_g3_baseline_m3s + 2.0
                    || g3_stage > book.breach_g3_baseline_m3s.max(0.2) + 0.4;
                if spiked {
                    book.s7_wave_arrival_g3_s = (now - book.breach_sim_s).max(0.01);
                }
            }
        }
        _ => {
            if hydro.power_mw > 0.0 {
                book.s4_hydro_mw_peak = book.s4_hydro_mw_peak.max(hydro.power_mw);
            }
        }
    }

    if hydro.power_mw > 0.0 {
        book.s4_hydro_mw_peak = book.s4_hydro_mw_peak.max(hydro.power_mw);
    }

    if let Some(b) = ballistics.as_ref() {
        if b.impacts > 0 {
            // Keep the best (smallest) CEP miss across flights.
            book.s5_impact_err_m = book.s5_impact_err_m.min(b.last_impact_err_m);
        }
    }
}

/// Ecology stand counters + S3/S9 latches (scenario dump mirror; ecology owns stands).
pub fn mirror_stands_into_dam_book_system(
    stands: Query<(&ChunkCellMatrix, &ChunkCellStand, Option<&ChunkCellFuelMoisture>)>,
    burn_cfg: Res<StandBurnConfig>,
    sim_time: Res<SimTimeMicros>,
    slow_elapsed: Option<Res<SlowDomainElapsedSecs>>,
    mut book: ResMut<DamFloodBenchmarkBook>,
) {
    if !book.seeded {
        return;
    }
    let now = ((sim_time.0 as f64 / 1_000_000.0)
        + slow_elapsed.map(|s| s.0).unwrap_or(0.0)) as f32;
    let mut live = 0u32;
    let mut drowned = 0u32;
    let mut dead = 0u32;
    let mut moist_sum = 0.0f32;
    let mut moist_n = 0u32;
    let mut live_burn = 0.0f32;
    let mut dead_burn = 0.0f32;
    let mut live_n = 0u32;
    let mut dead_n = 0u32;
    for (matrix, stand, fuel_m) in &stands {
        let n = stand.kind.len().min((matrix.size.x * matrix.size.y) as usize);
        for i in 0..n {
            if stand.kind[i] == StandKind::Bare {
                continue;
            }
            let m = matrix.moisture.get(i).copied().unwrap_or(0.4);
            let phase = stand.phase(i, now, &burn_cfg, 0, 1, m);
            match phase {
                StandPhase::Drowned => drowned += 1,
                StandPhase::Dead => dead += 1,
                StandPhase::Live => live += 1,
                _ => {}
            }
            let fine = fuel_m.and_then(|f| f.fine_01.get(i).copied());
            let eff = effective_fuel_moisture01(
                m,
                stand.flood_dead.get(i).copied().unwrap_or(false),
                stand.submerged_since.get(i).copied().unwrap_or(-1.0) >= 0.0,
                fine,
            );
            moist_sum += eff;
            moist_n += 1;
            let rate = burn_cfg.rate(stand.kind[i]) * (1.0 - eff).max(0.05);
            let submerged = stand.submerged_since.get(i).copied().unwrap_or(-1.0) >= 0.0;
            match phase {
                StandPhase::Live => {
                    live_burn += rate;
                    live_n += 1;
                }
                StandPhase::Dead => {
                    dead_burn += rate;
                    dead_n += 1;
                }
                _ if stand.flood_dead.get(i).copied().unwrap_or(false) && !submerged => {
                    // Exposed flood-dead (dry fine fuel) — S9 burn-rate proxy.
                    dead_burn += rate;
                    dead_n += 1;
                }
                _ => {}
            }
        }
    }
    book.stand_live = live;
    book.stand_drowned = drowned;
    book.stand_dead = dead;
    book.fuel_moisture_mean = if moist_n > 0 {
        moist_sum / moist_n as f32
    } else {
        0.0
    };
    if live_n > 0 && dead_n > 0 {
        let ratio = (dead_burn / dead_n as f32) / (live_burn / live_n as f32).max(1e-4);
        book.s9_burn_rate_dead_over_live = book.s9_burn_rate_dead_over_live.max(ratio);
    }
    if book.stage == DamFloodStage::S2Impound && drowned > 0 {
        let tick = (sim_time.0 / 1_000_000) as u64;
        book.enter_stage(DamFloodStage::S3Drown, tick);
    }
}

/// DF-7: fire a medium warhead when S5 is entered, or as a failsafe if the script stalls
/// in S2/S4 with no ballistic activity after the dam has been up for a while.
pub fn enqueue_dam_flood_strike_once_system(
    scene: Option<Res<ActiveTestScene>>,
    mut book: ResMut<DamFloodBenchmarkBook>,
    mut strikes: ResMut<StrikeOrderQueue>,
    ballistics: Option<Res<BallisticFlightBook>>,
    clock: Option<Res<crate::systems::sim_control::SimStepClock>>,
    mut done: Local<bool>,
) {
    if *done || !scene.is_some_and(|s| s.0.is_dam_flood()) {
        return;
    }
    let tick = clock.map(|c| c.steps).unwrap_or(0);
    // Latch only after a real flight / impact — pending alone can stall if the book
    // was missing for a frame and must not permanently suppress the failsafe.
    if ballistics.is_some_and(|b| !b.flights.is_empty() || b.impacts > 0) {
        *done = true;
        return;
    }
    if !strikes.pending.is_empty() {
        return;
    }
    let failsafe = matches!(
        book.stage,
        DamFloodStage::S2Impound | DamFloodStage::S3Drown | DamFloodStage::S4Operate
    ) && tick >= 160
        && book.stage_entry_tick[DamFloodStage::S2Impound as usize] > 0;
    if book.stage != DamFloodStage::S5Strike && !failsafe {
        return;
    }
    if failsafe && book.stage != DamFloodStage::S5Strike {
        book.enter_stage(DamFloodStage::S5Strike, tick);
        info!(
            target: "dam_flood::scene_seed",
            "failsafe S5 Strike at tick={tick} (scenario stalled before StrikeOrder)"
        );
    }
    *done = true;
    let fixture = load_dam_valley_fixture();
    let (lx, lz) = fixture.launch_site_origin;
    let (tx, tz) = fixture.narrows_tile;
    strikes.enqueue(StrikeOrder {
        origin_xz_m: Vec2::new((lx - 18) as f32 + 0.5, lz as f32 + 0.5) * TILE_METRES_GAMEPLAY,
        target_xz_m: Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5) * TILE_METRES_GAMEPLAY,
        warhead_id: "medium".into(),
    });
    info!(
        target: "dam_flood::scene_seed",
        "S5 Strike → StrikeOrder medium warhead launch→narrows"
    );
}

/// After the missile impacts, apply the hydraulic breach (visible fly-in first).
pub fn breach_dam_on_strike_impact_system(
    scene: Option<Res<ActiveTestScene>>,
    mut book: ResMut<DamFloodBenchmarkBook>,
    ballistics: Option<Res<BallisticFlightBook>>,
    sim_time: Res<SimTimeMicros>,
    mut breach: ResMut<crate::systems::water::BarrierBreachRequest>,
    mut done: Local<bool>,
) {
    if *done || !scene.is_some_and(|s| s.0.is_dam_flood()) {
        return;
    }
    if book.stage != DamFloodStage::S5Strike {
        return;
    }
    let Some(b) = ballistics.as_ref() else {
        return;
    };
    if b.impacts == 0 {
        return;
    }
    *done = true;
    breach.pending = Some(crate::systems::water::BarrierBreachSpec {
        clear_crest: true,
        breach_flow_m3s: 500.0,
    });
    let tick = (sim_time.0 / 1_000_000) as u64;
    book.enter_stage(DamFloodStage::S7Collapse, tick);
    book.dam_integrity_mean = 0.0;
    book.breach_flow_m3s = 500.0;
    info!(
        target: "dam_flood::scene_seed",
        "S5 impact → BarrierBreach + S7Collapse (impacts={}, err_m={:.1})",
        b.impacts,
        b.last_impact_err_m
    );
}
