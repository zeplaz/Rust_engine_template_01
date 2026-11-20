//! CQ-68 per-domain dump reporters. Each writes one `SimRunDump` section.
//! The JSON writer in `sim_run_dump` only reads the section map + run state.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde_json::{json, Map, Value};

use crate::ai::faction::{
    AiOrder, FactionAiDecisionLog, FactionAiRoster, FactionHq, FactionStartConfig,
};
use crate::combat::{DeploymentOutcomes, Formation, TheaterBook};
use crate::economy::logistics::orders::{
    LogisticsOrderSimRes, LogisticsOrdersSnapshotRes, OrderStatus,
};
use crate::economy::resource_flow::{
    FacilityFlowState, ResourceFlowNode, ResourceFlowSimStats, OUTPUT_BUFFER_CAP,
};
use crate::engine::states::BaseState;
use crate::engine::EngineLaunchArgs;
use crate::gui::ViewId;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::extraction::sim_visual_extract::FIRE_VISUAL_ACTIVE_HEAT_EPS;
use crate::render::{FrameHitchLog, WorldFireParticleFrame};
use crate::strategic::{
    ChunkStrategicOverlay, ConstructionSite, FrontlineState, SiteConstructionPhase,
};
use crate::systems::ecology::{
    ChunkCellStand, CELL_BURNING, CELL_BURNT, CELL_GRASS, CELL_OLD, CELL_REGROWTH, CELL_SHRUB,
    CELL_WATER, CELL_YOUNG,
};
use crate::systems::fire::{ChunkFireOverlay, FireEcologyStats, FireEntityExposureStats};
use crate::systems::sim_control::SimTick;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, WaterClass};

use super::sim_run_dump::{
    push_history, SimDumpArm, SimRunDump, SimRunDumpState, SIM_RUN_DUMP_SCHEMA, TERRITORY_CONTROL_MIN,
};

fn count_water(matrices: &Query<&ChunkCellMatrix>) -> (u32, u32, u32, u32, u32) {
    let mut cells = 0u32;
    let mut open = 0u32;
    let mut river = 0u32;
    let mut standing = 0u32;
    let mut wetland = 0u32;
    for m in matrices {
        for w in &m.water {
            cells = cells.saturating_add(1);
            match *w {
                WaterClass::River => {
                    open = open.saturating_add(1);
                    river = river.saturating_add(1);
                }
                WaterClass::Lake | WaterClass::Ocean => {
                    open = open.saturating_add(1);
                    standing = standing.saturating_add(1);
                }
                WaterClass::Wetland => wetland = wetland.saturating_add(1),
                WaterClass::Dry => {}
            }
        }
    }
    (cells, open, river, standing, wetland)
}

fn count_veg(stands: &Query<&ChunkCellStand>) -> (u32, u32, u32, u32) {
    let mut live = 0u32;
    let mut burning = 0u32;
    let mut burnt = 0u32;
    let mut water = 0u32;
    for stand in stands {
        for c in stand.render.iter() {
            match *c {
                CELL_GRASS | CELL_SHRUB | CELL_YOUNG | CELL_OLD | CELL_REGROWTH => {
                    live = live.saturating_add(1);
                }
                CELL_BURNING => burning = burning.saturating_add(1),
                CELL_BURNT => burnt = burnt.saturating_add(1),
                CELL_WATER => water = water.saturating_add(1),
                _ => {}
            }
        }
    }
    (live, burning, burnt, water)
}

pub fn dump_construction_section(
    arm: Res<SimDumpArm>,
    sites: Query<&ConstructionSite>,
    hqs: Query<&FactionHq>,
    start: Option<Res<FactionStartConfig>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let mut by_phase: BTreeMap<String, u32> = BTreeMap::new();
    let mut operational = 0u32;
    let mut total = 0u32;
    for site in &sites {
        total += 1;
        *by_phase.entry(format!("{:?}", site.phase)).or_default() += 1;
        if site.phase == SiteConstructionPhase::Operational {
            operational += 1;
        }
    }
    let hq_slots: Vec<u8> = hqs.iter().map(|h| h.slot).collect();
    let configured = start.as_ref().map_or(0, |c| c.factions.len() as u32);
    dump.put_value(
        "construction",
        json!({
            "sites_total": total,
            "sites_operational": operational,
            "sites_by_phase": by_phase,
            "hq_present": hq_slots.len(),
            "hq_configured": configured,
            "hq_slots": hq_slots,
        }),
    );
}

/// Vegetation draw inputs: extract row density + stand codes (counts only, no verdicts).
pub fn dump_vegetation_section(
    arm: Res<SimDumpArm>,
    veg: Option<Res<crate::render::extraction::VegetationExtractFrame>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let Some(veg) = veg else {
        dump.put_value("vegetation", json!({ "extract_present": false }));
        return;
    };
    let n = veg.rows.len();
    let dens: Vec<f32> = veg.rows.iter().map(|r| r.modifiers.mean_density).collect();
    let mean = if n == 0 { 0.0 } else { dens.iter().sum::<f32>() / n as f32 };
    let min = dens.iter().copied().fold(f32::INFINITY, f32::min);
    let max = dens.iter().copied().fold(0.0_f32, f32::max);
    let low = dens.iter().filter(|d| **d < 0.02).count();
    let with_cells = veg.rows.iter().filter(|r| r.cells.is_some()).count();
    let mut codes: BTreeMap<u8, u64> = BTreeMap::new();
    for r in &veg.rows {
        if let Some(c) = r.cells.as_deref() {
            for code in c {
                *codes.entry(*code).or_default() += 1;
            }
        }
    }
    dump.put_value(
        "vegetation",
        json!({
            "extract_present": true,
            "rows": n,
            "rows_with_cells": with_cells,
            "density_mean": mean,
            "density_min": if min.is_finite() { min } else { 0.0 },
            "density_max": max,
            "rows_density_lt_002": low,
            "cell_codes": codes.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<Map<String, Value>>(),
        }),
    );
}

/// Ecology canopy chain per chunk (counts only): succession stage, canopy, target, cap.
pub fn dump_canopy_section(
    arm: Res<SimDumpArm>,
    chunks: Query<(
        &crate::systems::ecology::VegetationField,
        &crate::systems::ecology::CanopyDrivers,
        Option<&crate::systems::ecology::SuccessionState>,
    )>,
    matrices: Query<(&Chunk, &ChunkCellMatrix)>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    // Family + water class histogram inside the dam fixture floodplain box (120..=200)^2.
    let fams = crate::terrain::family::default_terrain_families();
    let mut fp_family: BTreeMap<String, u32> = BTreeMap::new();
    let mut fp_water: BTreeMap<String, u32> = BTreeMap::new();
    for (chunk, m) in &matrices {
        for y in 0..m.size.y {
            for x in 0..m.size.x {
                let wx = chunk.coord.x * m.size.x as i32 + x as i32;
                let wy = chunk.coord.y * m.size.y as i32 + y as i32;
                if !(120..=200).contains(&wx) || !(120..=200).contains(&wy) {
                    continue;
                }
                let i = m.idx(x, y);
                let name = m.family.get(i).and_then(|f| fams.def(*f)).map_or("?".to_string(), |d| d.name.clone());
                *fp_family.entry(name).or_default() += 1;
                *fp_water.entry(format!("{:?}", m.water.get(i).copied().unwrap_or_default())).or_default() += 1;
            }
        }
    }
    let mut cap_max = 0.0f32;
    let mut stages: BTreeMap<String, u32> = BTreeMap::new();
    let (mut n, mut canopy, mut cmax, mut target, mut cap, mut growth) = (0u32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let mut bands = [0u32; 4];
    for (veg, drv, succ) in &chunks {
        n += 1;
        canopy += veg.canopy_density;
        cmax = cmax.max(veg.canopy_density);
        target += drv.population_density.unwrap_or(-1.0);
        cap += drv.suitability_cap;
        cap_max = cap_max.max(drv.suitability_cap);
        growth += drv.growth_scale;
        let b = match veg.canopy_density {
            c if c < 0.2 => 0,
            c if c < 0.45 => 1,
            c if c < 0.75 => 2,
            _ => 3,
        };
        bands[b] += 1;
        if let Some(s) = succ {
            *stages.entry(format!("{:?}", s.stage)).or_default() += 1;
        }
    }
    let nf = n.max(1) as f32;
    dump.put_value(
        "canopy",
        json!({
            "chunks": n,
            "canopy_mean": canopy / nf,
            "canopy_max": cmax,
            "population_target_mean": target / nf,
            "suitability_cap_mean": cap / nf,
            "suitability_cap_max": cap_max,
            "floodplain_family": fp_family,
            "floodplain_water": fp_water,
            "growth_scale_mean": growth / nf,
            "canopy_bands_grass_shrub_young_old": bands,
            "succession_stages": stages,
        }),
    );
}

pub fn dump_economy_section(
    arm: Res<SimDumpArm>,
    plants: Query<(
        &ResourceFlowNode,
        &FacilityFlowState,
        Option<&crate::economy::facility_inputs::FacilityInputs>,
    )>,
    flow_stats: Option<Res<ResourceFlowSimStats>>,
    logistics: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let (mut operational, mut idle, mut starved) = (0u32, 0u32, 0u32);
    let (mut buf_units, mut buf_cap) = (0.0f64, 0.0f64);
    let mut rows: Vec<Value> = Vec::new();
    for (node, state, inputs) in &plants {
        let supply = inputs.map_or(1.0, |i| i.factor());
        rows.push(json!({
            "catalog_id": node.catalog_id,
            "produces": node.production.iter().map(|r| r.tag.clone()).collect::<Vec<_>>(),
            "starved": state.starved,
            "output_full": state.output_full,
            "output_scale": state.output_scale,
            "workforce": inputs.map(|i| i.workforce),
            "power": inputs.map(|i| i.power),
            "power_state": inputs.map(|i| format!("{:?}", i.power_state)),
        }));
        if state.starved {
            starved += 1;
        } else if state.output_full || supply <= 0.0 {
            idle += 1;
        } else {
            operational += 1;
        }
        for rate in &node.production {
            let have = match rate.resource {
                Some(rt) => node.inventory.get(&rt).copied().unwrap_or(0.0),
                None => node.buffer_by_tag.get(&rate.tag).copied().unwrap_or(0.0),
            };
            buf_units += f64::from(have);
            buf_cap += f64::from(OUTPUT_BUFFER_CAP);
        }
    }
    let produced: BTreeMap<String, f64> = flow_stats
        .as_ref()
        .map(|s| {
            s.produced_by_tag
                .iter()
                .map(|(k, v)| (k.clone(), f64::from(*v)))
                .collect()
        })
        .unwrap_or_default();
    let produced_total: f64 = produced.values().sum();
    let depot_stock_units: u64 = logistics.as_ref().map_or(0, |l| {
        l.0.sites
            .iter()
            .flat_map(|s| s.stock.iter())
            .map(|r| u64::from(r.on_hand))
            .sum()
    });
    dump.put_value(
        "economy",
        json!({
            "plants_total": operational + idle + starved,
            "plants_operational": operational,
            "plants_idle": idle,
            "plants_starved": starved,
            "produced_total": produced_total,
            "produced_by_tag": produced,
            "storage_units": buf_units,
            "storage_capacity": buf_cap,
            "storage_fill": if buf_cap > 0.0 { buf_units / buf_cap } else { 0.0 },
            "depot_stock_units": depot_stock_units,
            "plants": rows,
        }),
    );
}

pub fn dump_logistics_section(
    arm: Res<SimDumpArm>,
    logistics: Option<Res<LogisticsOrdersSnapshotRes>>,
    logistics_sim: Option<Res<LogisticsOrderSimRes>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let Some(snap) = logistics.as_ref().map(|l| &l.0) else {
        dump.put_value("logistics", json!({ "available": false }));
        return;
    };
    let standing = snap.standing_lines().count();
    let one_off = snap.one_off_runs().count();
    let one_off_completed = snap
        .one_off_runs()
        .filter(|o| matches!(o.status, OrderStatus::Completed))
        .count();
    let trips_done: u64 = snap.orders.iter().map(|o| u64::from(o.trips_done)).sum();
    let delivered_units: u64 = snap.orders.iter().map(|o| u64::from(o.delivered)).sum();
    let draw_total = snap.demands.iter().filter(|d| d.draw_rate_per_min > 0).count();
    let unserved = snap
        .demands
        .iter()
        .filter(|d| d.draw_rate_per_min > 0 && d.served_by == 0)
        .count();
    let starving = snap
        .demands
        .iter()
        .filter(|d| d.starved_ticks > 0 || d.on_hand == 0 && d.draw_rate_per_min > 0)
        .count();
    let vehicles_busy = snap.fleets.iter().map(|f| u64::from(f.busy)).sum::<u64>();
    let vehicles_total = snap.fleets.iter().map(|f| u64::from(f.total)).sum::<u64>();
    let (fire_delayed, vehicles_lost, cargo_lost) = logistics_sim.as_ref().map_or((0, 0, 0), |s| {
        (
            s.sim.fire_delayed_trips,
            s.sim.vehicles_lost_to_fire,
            s.sim.cargo_lost_to_fire,
        )
    });
    dump.put_value(
        "logistics",
        json!({
            "available": true,
            "sites": snap.sites.len(),
            "orders_total": snap.orders.len(),
            "standing_lines": standing,
            "one_off_orders": one_off,
            "one_off_completed": one_off_completed,
            "deliveries_completed": trips_done,
            "delivered_units": delivered_units,
            "draw_demands": draw_total,
            "unserved_draw_demands": unserved,
            "draw_demand_rows": snap
                .demands
                .iter()
                .filter(|d| d.draw_rate_per_min > 0)
                .map(|d| format!(
                    "owner {} {} site {} {} draw {}/min on_hand {} served_by {}",
                    d.owner.0, d.consumer.label(), d.site.0, d.cargo, d.draw_rate_per_min, d.on_hand, d.served_by
                ))
                .collect::<Vec<_>>(),
            "starving_demands": starving,
            "vehicles_total": vehicles_total,
            "vehicles_busy": vehicles_busy,
            "rejected_requests": snap.rejected.len(),
            "fire_delayed_trips": fire_delayed,
            "vehicles_lost_to_fire": vehicles_lost,
            "cargo_lost_to_fire": cargo_lost,
        }),
    );
}

pub fn dump_ai_section(
    arm: Res<SimDumpArm>,
    ai_log: Option<Res<FactionAiDecisionLog>>,
    roster: Option<Res<FactionAiRoster>>,
    mut state: ResMut<SimRunDumpState>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    if let Some(log) = ai_log.as_ref() {
        for rec in &log.records {
            let t = state.ai.entry(rec.slot).or_default();
            if t.last_seen_tick.is_some_and(|seen| rec.sim_tick <= seen) {
                continue;
            }
            t.last_seen_tick = Some(rec.sim_tick);
            t.decisions += 1;
            t.last_goal = rec.goal.label().to_string();
            for (order, _) in &rec.orders {
                match order {
                    AiOrder::StandingLine { .. }
                    | AiOrder::SupplyRun { .. }
                    | AiOrder::SupplyLine { .. } => {
                        t.orders_logistics += 1;
                    }
                    AiOrder::Deploy { .. } | AiOrder::Reinforce { .. } | AiOrder::Attack { .. } => {
                        t.orders_deployment += 1;
                    }
                    AiOrder::Build { .. } | AiOrder::Develop { .. } | AiOrder::FoundDepot { .. } => {
                        t.orders_build += 1;
                    }
                }
            }
        }
    }
    let roster_slots: Vec<u8> = roster
        .as_ref()
        .map(|r| r.slots.iter().map(|(s, _)| *s).collect())
        .unwrap_or_default();
    let mut factions = Map::new();
    let mut slots: Vec<u8> = roster_slots.clone();
    for s in state.ai.keys() {
        if !slots.contains(s) {
            slots.push(*s);
        }
    }
    slots.sort_unstable();
    for slot in &slots {
        let t = state.ai.get(slot).cloned().unwrap_or_default();
        factions.insert(
            slot.to_string(),
            json!({
                "decisions": t.decisions,
                "orders_logistics": t.orders_logistics,
                "orders_deployment": t.orders_deployment,
                "orders_build": t.orders_build,
                "orders_total": t.orders_logistics + t.orders_deployment + t.orders_build,
                "last_goal": t.last_goal,
            }),
        );
    }
    dump.put_value(
        "ai",
        json!({
            "roster_slots": roster_slots,
            "total_decisions": ai_log.as_ref().map_or(0, |l| l.total_decisions),
            "goal_switches": ai_log.as_ref().map_or(0, |l| l.total_goal_switches),
            "factions": Value::Object(factions),
        }),
    );
}

pub fn dump_combat_section(
    arm: Res<SimDumpArm>,
    formations: Query<&Formation>,
    strategic: Query<&ChunkStrategicOverlay>,
    frontline: Option<Res<FrontlineState>>,
    theaters: Option<Res<TheaterBook>>,
    outcomes: Option<Res<DeploymentOutcomes>>,
    fire_entities: Option<Res<FireEntityExposureStats>>,
    mut state: ResMut<SimRunDumpState>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let engaged = formations.iter().filter(|f| f.engaged).count();
    state.track("engaged_formations", engaged as f64);
    let mut held: BTreeMap<u8, u32> = BTreeMap::new();
    for ov in &strategic {
        for control in &ov.faction_control {
            let (slot, best) = control
                .iter()
                .enumerate()
                .fold((0usize, 0.0f32), |a, (i, v)| if *v > a.1 { (i, *v) } else { a });
            if best > TERRITORY_CONTROL_MIN {
                *held.entry(slot as u8).or_default() += 1;
            }
        }
    }
    if state.territory_first.is_empty() && !held.is_empty() {
        state.territory_first = held.clone();
    }
    let mut changed = 0i64;
    let mut slots: Vec<u8> = held.keys().chain(state.territory_first.keys()).copied().collect();
    slots.sort_unstable();
    slots.dedup();
    for s in &slots {
        let now = i64::from(held.get(s).copied().unwrap_or(0));
        let first = i64::from(state.territory_first.get(s).copied().unwrap_or(0));
        changed += (now - first).abs();
    }
    let (ok, rejected) = outcomes.as_ref().map_or((0, 0), |o| {
        let ok = o.recent.iter().filter(|r| r.result.is_ok()).count();
        (ok, o.recent.len() - ok)
    });
    let fire_exposed = fire_entities.as_ref().map_or(0u32, |e| e.formations_exposed);
    let fire_heat_mean = fire_entities.as_ref().map_or(0.0f32, |e| {
        if e.formations_exposed == 0 {
            0.0
        } else {
            e.formation_heat_sum / e.formations_exposed as f32
        }
    });
    state.track("fire_exposed_formations", f64::from(fire_exposed));
    dump.put_value(
        "combat",
        json!({
            "front_segments": frontline.as_ref().map_or(0, |f| f.segments.len()),
            "front_cells": frontline.as_ref().map_or(0, |f| f.segments.iter().map(|s| s.cells.len()).sum::<usize>()),
            "contested_cells": frontline.as_ref().map_or(0, |f| f.segments.iter().map(|s| s.contested_cells as usize).sum::<usize>()),
            "theaters": theaters.as_ref().map_or(0, |t| t.theaters.len()),
            "formations": formations.iter().count(),
            "engaged_formations": engaged,
            "engaged_formations_series": state.series_json("engaged_formations"),
            "fire_exposed_formations": fire_exposed,
            "fire_exposed_formations_series": state.series_json("fire_exposed_formations"),
            "fire_exposed_mean_heat": fire_heat_mean,
            "deployment_orders_resolved_ok": ok,
            "deployment_orders_resolved_rejected": rejected,
            "territory_cells_by_slot": held,
            "territory_cells_first_by_slot": state.territory_first,
            "territory_change_cells": changed,
        }),
    );
}

fn fire_wind_spread_ratio(
    overlays: &Query<(&Chunk, &ChunkFireOverlay, Option<&ChunkWeather>)>,
) -> (f64, f64, f64) {
    let mut wind = Vec2::ZERO;
    let mut wn = 0u32;
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut hot = 0u32;
    for (chunk, ovl, wx) in overlays {
        if let Some(wx) = wx {
            wind += wx.wind_mps;
            wn += 1;
        }
        let sx = (ovl.heat.len() as f32).sqrt() as i32;
        if sx <= 0 {
            continue;
        }
        for (i, h) in ovl.heat.iter().enumerate() {
            if *h < 0.08 {
                continue;
            }
            hot = hot.saturating_add(1);
            let lx = (i as i32) % sx;
            let ly = (i as i32) / sx;
            let x = chunk.coord.x.wrapping_mul(sx).wrapping_add(lx);
            let y = chunk.coord.y.wrapping_mul(sx).wrapping_add(ly);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }
    if hot == 0 || wn == 0 {
        return (0.0, 0.0, 0.0);
    }
    let wind = wind / wn as f32;
    let speed = wind.length();
    let (dx, dy) = ((max_x - min_x).max(0) as f64, (max_y - min_y).max(0) as f64);
    let along = if speed < 0.5 {
        1.0
    } else if wind.x.abs() >= wind.y.abs() {
        (dx + 1.0) / (dy + 1.0)
    } else {
        (dy + 1.0) / (dx + 1.0)
    };
    (along, speed as f64, f64::from(hot))
}

pub fn dump_fire_section(
    arm: Res<SimDumpArm>,
    stands: Query<&ChunkCellStand>,
    overlays: Query<(&Chunk, &ChunkFireOverlay, Option<&ChunkWeather>)>,
    ecology: Option<Res<FireEcologyStats>>,
    fire_entities: Option<Res<FireEntityExposureStats>>,
    mut state: ResMut<SimRunDumpState>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let veg = count_veg(&stands);
    let regrowth = stands
        .iter()
        .map(|s| s.render.iter().filter(|c| **c == CELL_REGROWTH).count() as u32)
        .sum::<u32>();
    let mut heat_cells = 0u32;
    for (_chunk, overlay, _wx) in &overlays {
        for h in &overlay.heat {
            if *h > 0.04 {
                heat_cells = heat_cells.saturating_add(1);
            }
        }
    }
    let ever = u64::from(veg.1) + u64::from(veg.2);
    state.ignited_cells_cum += ever.saturating_sub(state.prev_ever_burning);
    state.prev_ever_burning = ever;
    state.track("burning_cells", f64::from(veg.1));
    state.track("burnt_cells", f64::from(veg.2));
    state.track("regrowth_cells", f64::from(regrowth));
    state.track("heat_cells", f64::from(heat_cells));
    state.track("veg_live_cells", f64::from(veg.0));
    let land = veg.0.saturating_add(veg.1).saturating_add(veg.2).max(1);
    state.track("burnt_fraction", f64::from(veg.2) / f64::from(land));
    let (lb_ratio, mean_wind, hot_n) = fire_wind_spread_ratio(&overlays);
    state.track("fire_length_breadth", lb_ratio);
    state.track("fire_mean_wind_mps", mean_wind);
    let ember_emitted = ecology.as_ref().map_or(0u64, |e| e.ember_spots_emitted);
    let ember_applied = ecology.as_ref().map_or(0u64, |e| e.ember_spots_applied);
    state.track("ember_spots_emitted", ember_emitted as f64);
    state.track("ember_spots_applied", ember_applied as f64);
    let collapsed = fire_entities
        .as_ref()
        .map_or(0u64, |e| e.sites_collapsed_to_fire);
    let threatened = fire_entities.as_ref().map_or(0u32, |e| e.sites_threatened);
    let disrupted = fire_entities.as_ref().map_or(0u32, |e| e.sites_disrupted);
    let deteriorating = fire_entities
        .as_ref()
        .map_or(0u32, |e| e.sites_deteriorating);
    state.track("sites_collapsed_to_fire", collapsed as f64);
    state.track("sites_threatened", f64::from(threatened));
    state.track("sites_disrupted", f64::from(disrupted));
    state.track("sites_deteriorating", f64::from(deteriorating));
    dump.put_value(
        "fire",
        json!({
            "ignited_cells_cum": state.ignited_cells_cum,
            "burning_cells": state.series_json("burning_cells"),
            "burnt_cells": state.series_json("burnt_cells"),
            "regrowth_cells": state.series_json("regrowth_cells"),
            "heat_cells": state.series_json("heat_cells"),
            "veg_live_cells": state.series_json("veg_live_cells"),
            "burnt_fraction": state.series_json("burnt_fraction"),
            "length_breadth": state.series_json("fire_length_breadth"),
            "mean_wind_mps": state.series_json("fire_mean_wind_mps"),
            "ember_spots_emitted": state.series_json("ember_spots_emitted"),
            "ember_spots_applied": state.series_json("ember_spots_applied"),
            "sites_collapsed_to_fire": state.series_json("sites_collapsed_to_fire"),
            "sites_threatened": state.series_json("sites_threatened"),
            "sites_disrupted": state.series_json("sites_disrupted"),
            "sites_deteriorating": state.series_json("sites_deteriorating"),
            "hot_cells_bbox": hot_n,
            "max_heat": ecology.as_ref().map_or(0.0, |e| e.max_heat),
        }),
    );
}

pub fn dump_fire_chain_section(
    arm: Res<SimDumpArm>,
    overlays: Query<(&Chunk, &ChunkFireOverlay, Option<&ChunkWeather>)>,
    graph: Option<Res<RenderProjectionGraph>>,
    index: Option<Res<crate::render::ChunkFireEntityIndex>>,
    residency: Option<Res<crate::io::streaming::ChunkResidencyTable>>,
    fire_chunks: Query<
        Entity,
        (
            With<Chunk>,
            With<ChunkCellMatrix>,
            With<crate::systems::fire::ChunkSurfaceFire>,
        ),
    >,
    snapshot: Option<Res<crate::render::FireSimulationSnapshot>>,
    extract: Option<Res<crate::render::FireExtractDiagnostics>>,
    active: Option<Res<crate::render::ActiveFireChunkSet>>,
    visible: Option<Res<crate::render::VisibleFireChunkSet>>,
    by_view: Option<Res<crate::render::FireVisualFramesByView>>,
    fence: Option<Res<crate::render::CommittedVisualSnapshotFence>>,
    policy: Option<Res<crate::gui::RepresentationResult>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let mut emitting_cells = 0u32;
    for (_chunk, o, _wx) in &overlays {
        for (i, h) in o.heat.iter().enumerate() {
            let fuel = o.fuel.get(i).copied().unwrap_or(0.0);
            let hold = o.burn_hold.get(i).copied().unwrap_or(0);
            if *h >= FIRE_VISUAL_ACTIVE_HEAT_EPS && (fuel > 0.0 || hold > 0) {
                emitting_cells += 1;
            }
        }
    }
    let view_rows =
        |id: ViewId| by_view.as_ref().and_then(|b| b.by_id.get(&id)).map_or(0, |fr| fr.instances.len());
    let visible_n =
        |id: ViewId| visible.as_ref().and_then(|v| v.per_view.get(&id)).map_or(-1, |s| s.len() as i64);
    let r = extract.as_ref().map(|d| d.last.clone()).unwrap_or_default();
    let index_resolving = index.as_ref().map_or(0, |ix| {
        ix.by_coord
            .values()
            .filter(|e| fire_chunks.contains(**e))
            .count()
    });
    let residency_bounds = residency.as_ref().and_then(|t| {
        let mut it = t.entries.keys();
        let first = *it.next()?;
        Some(
            t.entries
                .keys()
                .fold((first, first), |(lo, hi), c| (lo.min(*c), hi.max(*c))),
        )
    });
    dump.put_value(
        "fire_chain",
        json!({
            "index_len": index.as_ref().map_or(0, |ix| ix.by_coord.len()),
            "index_resolving": index_resolving,
            "fire_chunk_entities": fire_chunks.iter().count(),
            "residency_bounds": residency_bounds.map(|(lo, hi)| [lo.x, lo.y, hi.x, hi.y]),
            "emitting_cells": emitting_cells,
            "snapshot_instances": snapshot.as_ref().map_or(0, |s| s.instances.len()),
            "snapshot_chunk_heat": snapshot.as_ref().map_or(0, |s| s.chunk_heat.len()),
            "snapshot_tick": snapshot.as_ref().map_or(0, |s| s.stamp.tick),
            "fence_tick": fence.as_ref().map_or(0, |s| s.fire.tick),
            "graph_snapshot_tick": graph.as_ref().map_or(0, |g| g.fire.snapshot_stamp),
            "active_chunks": active.as_ref().map_or(0, |a| a.chunks.len()),
            "visible_world_main": visible_n(ViewId::WorldMain),
            "visible_sim_map": visible_n(ViewId::SimulationMap),
            "view_rows_world_main": view_rows(ViewId::WorldMain),
            "view_rows_sim_map": view_rows(ViewId::SimulationMap),
            "graph_rows": graph.as_ref().map_or(0, |g| g.fire.instance_buffer.len()),
            "policy_fire_instances": policy.as_ref().is_some_and(|p| p.extract_plan.fire_instances),
            "policy_fire_cap": policy.as_ref().map_or(0, |p| p.extract_plan.fire_instance_cap.min(1 << 30)),
            "extract": {
                "ran_full_scan": r.ran_full_scan,
                "full_reconcile": r.full_reconcile,
                "cadence_skipped": r.cadence_skipped,
                "fingerprint_skipped": r.fingerprint_skipped,
                "scan_set_len": r.scan_set_len,
                "residency_len": r.residency_len,
                "chunks_iterated": r.chunks_iterated,
                "chunks_profiled": r.chunks_profiled,
                "instances_written": r.instances_written,
                "extract_ms": r.extract_ms,
            },
        }),
    );
}

/// World envelope + history (terrain counts, launch meta). Runs after domain sections.
pub fn dump_world_envelope_section(
    arm: Res<SimDumpArm>,
    tick: Res<SimTick>,
    params: Option<Res<WorldGenParams>>,
    launch: Option<Res<EngineLaunchArgs>>,
    base: Option<Res<State<BaseState>>>,
    matrices: Query<&ChunkCellMatrix>,
    stands: Query<&ChunkCellStand>,
    overlays: Query<(&Chunk, &ChunkFireOverlay, Option<&ChunkWeather>)>,
    ecology: Option<Res<FireEcologyStats>>,
    graph: Option<Res<RenderProjectionGraph>>,
    particles: Option<Res<WorldFireParticleFrame>>,
    mut state: ResMut<SimRunDumpState>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let water = count_water(&matrices);
    let veg = count_veg(&stands);
    let regrowth = stands
        .iter()
        .map(|s| s.render.iter().filter(|c| **c == CELL_REGROWTH).count() as u32)
        .sum::<u32>();
    let mut fire_heat_cells = 0u32;
    for (_chunk, overlay, _wx) in &overlays {
        for h in &overlay.heat {
            if *h > 0.04 {
                fire_heat_cells = fire_heat_cells.saturating_add(1);
            }
        }
    }
    let cells = water.0.max(1);
    let scene = launch
        .as_ref()
        .map(|l| format!("{:?}", l.test_scene))
        .unwrap_or_else(|| "None".into());
    let presentation_kind = dump
        .get("presentation")
        .and_then(|p| p.get("kind"))
        .and_then(|k| k.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let sites_operational = dump
        .get("construction")
        .and_then(|c| c.get("sites_operational"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let deliveries = dump
        .get("logistics")
        .and_then(|l| l.get("deliveries_completed"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let produced = dump
        .get("economy")
        .and_then(|e| e.get("produced_total"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let engaged = dump
        .get("combat")
        .and_then(|c| c.get("engaged_formations"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let frame_ms_last = state.frame_ms.last().copied().unwrap_or(0.0);
    push_history(
        &mut state,
        json!({
            "tick": tick.0,
            "burning": veg.1,
            "burnt": veg.2,
            "regrowth": regrowth,
            "heat_cells": fire_heat_cells,
            "fire_inst": graph.as_ref().map_or(0, |g| g.fire.instance_buffer.len()),
            "sites_operational": sites_operational,
            "deliveries": deliveries,
            "produced": produced,
            "engaged": engaged,
            "frame_ms": frame_ms_last,
        }),
    );
    let spark_view = particles
        .as_ref()
        .map_or("", |p| p.spark_witness.projection_view);
    dump.put_value(
        "envelope",
        json!({
            "tick": tick.0,
            "seed": params.as_ref().map_or(0, |p| p.seed),
            "launch": {
                "test_scene": scene,
                "presentation": presentation_kind,
            },
            "world": {
                "width": params.as_ref().map_or(0, |p| p.width),
                "height": params.as_ref().map_or(0, |p| p.height),
                "cells": cells,
            },
            "flags": {
                "world_generated": cells > 1,
                "simulation_entered": base.as_ref().is_some_and(|s| matches!(*s.get(), BaseState::Simulation)),
            },
            "counts": {
                "terrain_cells": water.0,
                "water_open_cells": water.1,
                "river_cells": water.2,
                "lake_or_ocean_cells": water.3,
                "wetland_cells": water.4,
                "veg_live_cells": veg.0,
                "veg_burning_cells": veg.1,
                "veg_burnt_cells": veg.2,
                "veg_water_cells": veg.3,
                "fire_heat_cells": fire_heat_cells,
                "fire_instance_rows": graph.as_ref().map_or(0, |g| g.fire.instance_buffer.len() as u32),
                "spark_rows": particles.as_ref().map_or(0, |p| p.spark_witness.rows as u32),
            },
            "frac": { "river": water.2 as f32 / cells as f32 },
            "max": {
                "fire_heat": ecology.as_ref().map_or(0.0, |e| e.max_heat),
                "mean_heat": ecology.as_ref().map_or(0.0, |e| e.mean_heat),
                "mean_fuel": ecology.as_ref().map_or(0.0, |e| e.mean_fuel),
            },
            "spark_projection_view": spark_view,
            "schema": SIM_RUN_DUMP_SCHEMA,
            "proof_grade": "sim_dump",
            "debug": cfg!(debug_assertions),
            "build": if cfg!(debug_assertions) { "debug" } else { "release" },
        }),
    );
}

pub fn dump_series_sections(
    arm: Res<SimDumpArm>,
    weather: Option<Res<super::sim_run_dump_weather::WeatherDumpSeries>>,
    fidelity: Option<Res<super::sim_run_dump_fidelity::FidelityDumpSeries>>,
    presentation: Option<Res<super::sim_run_dump_massing::PresentationDumpSeries>>,
    atmosphere: Option<Res<crate::systems::atmosphere::AtmosphereDiagnostics>>,
    atmosphere_frame: Option<Res<crate::systems::atmosphere::AtmosphereFrame>>,
    voxel_occupation: Option<Res<crate::strategic::VoxelOccupationBook>>,
    voxel_occupation_cfg: Option<Res<crate::strategic::VoxelOccupationConfig>>,
    precip: Option<Res<crate::render::WeatherPrecipFrame>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let mut weather_v = super::sim_run_dump_weather::weather_json(weather.as_deref());
    if let Some(inst) = precip.as_ref().map(|p| p.instance_count) {
        if let Some(obj) = weather_v.as_object_mut() {
            obj.insert("precip_instances".into(), json!(inst));
        }
    }
    dump.put_value("weather", weather_v);
    dump.put_value(
        "fidelity",
        super::sim_run_dump_fidelity::fidelity_json(fidelity.as_deref()),
    );
    dump.put_value(
        "presentation",
        super::sim_run_dump_massing::presentation_json(presentation.as_deref()),
    );
    dump.put_value(
        "atmosphere",
        super::sim_run_dump_atmosphere::atmosphere_json(
            atmosphere.as_deref(),
            atmosphere_frame.as_deref(),
        ),
    );
    dump.put_value(
        "occupation",
        super::sim_run_dump_occupation::occupation_json(
            voxel_occupation.as_deref(),
            voxel_occupation_cfg.as_deref(),
        ),
    );
}

/// Assemble final JSON from sections only (CQ-68).
pub fn flush_sim_run_dump(
    arm: Res<SimDumpArm>,
    sections: Res<SimRunDump>,
    mut state: ResMut<SimRunDumpState>,
    hitch: Option<Res<FrameHitchLog>>,
    launch: Option<Res<EngineLaunchArgs>>,
) {
    if !arm.due {
        return;
    }
    if !super::sim_run_dump::sim_dump_enabled(launch.as_deref()) {
        return;
    }
    let mut body = sections
        .get("envelope")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let obj = body.as_object_mut().expect("envelope object");
    for key in [
        "construction",
        "dam_site",
        "dam_flood_history",
        "voxel_camera",
        "vegetation",
        "canopy",
        "economy",
        "logistics",
        "ai",
        "combat",
        "fire",
        "fire_chain",
        "weather",
        "fidelity",
        "atmosphere",
        "presentation",
        "occupation",
    ] {
        if let Some(v) = sections.get(key).cloned() {
            obj.insert(key.into(), v);
        }
    }
    if let Some(q) = sections.get("quadrants").cloned() {
        obj.insert("quadrants".into(), q);
    }
    if let Some(d) = sections.get("dam_flood").cloned() {
        if !d.is_null() {
            obj.insert("dam_flood".into(), d);
        }
    }
    obj.insert("perf".into(), state.perf_json(hitch.as_deref()));
    obj.insert("history".into(), Value::Array(state.history.clone()));
    if super::sim_run_dump::write_dump_files_pub(&body) {
        state.last_tick = arm.tick;
    }
}

/// Voxel camera / pick rectangle (counts only): the map fill rect the click maths uses, and where
/// the dam centre projects with that maths — compare against the screenshot to measure an offset.
pub fn dump_voxel_camera_section(
    arm: Res<SimDumpArm>,
    cam: Option<Res<crate::render::pixel_world::PixelWorldCameraFrame>>,
    fill: Option<Res<crate::gui::tactical::sim_map_rtt::TacticalMapFillRect>>,
    field: Option<Res<crate::render::pixel_world::PixelWorldTerrainField>>,
    window: Query<&bevy::window::Window, With<bevy::window::PrimaryWindow>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let win = window.single().ok().map(|w| json!({
        "logical": [w.width(), w.height()],
        "physical": [w.physical_width(), w.physical_height()],
        "scale_factor": w.scale_factor(),
    }));
    let proj = |tile: Vec2| -> Option<[f32; 2]> {
        let (cam, field) = (cam.as_deref()?, field.as_deref()?);
        let h = field.surface_at(tile)?;
        let p = crate::render::pixel_world::tile_to_world3(tile, h);
        cam.world_to_screen(p).map(|v| [v.x, v.y])
    };
    dump.put_value(
        "voxel_camera",
        json!({
            "active": cam.as_deref().is_some_and(|c| c.active),
            "screen_min": cam.as_deref().map(|c| [c.screen_min.x, c.screen_min.y]),
            "screen_max": cam.as_deref().map(|c| [c.screen_max.x, c.screen_max.y]),
            "aspect": cam.as_deref().map(|c| c.aspect),
            "fill_valid": fill.as_deref().map(|f| f.valid),
            "fill_min": fill.as_deref().map(|f| [f.min.x, f.min.y]),
            "fill_max": fill.as_deref().map(|f| [f.max.x, f.max.y]),
            "window": win,
            "dam_centre_screen": proj(Vec2::new(160.0, 160.0)),
            "valley_south_screen": proj(Vec2::new(160.0, 120.0)),
        }),
    );
}
