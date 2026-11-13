//! One sim-run dump. Counts only — no `green` field.
//!
//! Score with `python tools/sim_review/score_dump.py`. Do not treat `*_live.json`
//! refresh writers as a run.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::app::AppExit;
use bevy::prelude::*;
use serde_json::{json, Value};

use crate::engine::states::BaseState;
use crate::engine::EngineLaunchArgs;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::WorldFireParticleFrame;
use crate::strategic::{ConstructionSite, SiteConstructionPhase};
use crate::systems::ecology::{
    ChunkCellStand, CELL_BURNING, CELL_BURNT, CELL_GRASS, CELL_OLD, CELL_REGROWTH, CELL_SHRUB,
    CELL_WATER, CELL_YOUNG,
};
use crate::systems::fire::{ChunkFireOverlay, FireEcologyStats};
use crate::systems::sim_control::SimTick;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{ChunkCellMatrix, WaterClass};

pub const SIM_RUN_DUMP_JSON: &str = "debug_runs/sim_run_dump.json";
pub const SIM_RUN_DUMP_SCHEMA: &str = "sim_run_dump_v1";

const DUMP_STRIDE_TICKS: u64 = 30;

#[derive(Resource, Debug)]
struct SimRunDumpState {
    last_tick: u64,
}

impl Default for SimRunDumpState {
    fn default() -> Self {
        Self { last_tick: u64::MAX }
    }
}

/// Dump writes **only** in debug builds when `RUST_ENGINE_SIM_DUMP=1`.
/// Release / production binaries never collect or write a dump.
#[must_use]
pub fn sim_dump_enabled(_launch: Option<&EngineLaunchArgs>) -> bool {
    cfg!(debug_assertions) && env_on("RUST_ENGINE_SIM_DUMP")
}

fn env_on(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[must_use]
pub fn build_sim_run_dump(
    tick: u64,
    seed: u64,
    world_w: u32,
    world_h: u32,
    test_scene: &str,
    sim_entered: bool,
    water: (u32, u32, u32, u32),
    veg: (u32, u32, u32, u32),
    fire_heat_cells: u32,
    fire_instance_rows: u32,
    spark_rows: u32,
    spark_view: &str,
    mean_heat: f32,
    max_heat: f32,
    mean_fuel: f32,
    sites_operational: u32,
    sites_total: u32,
) -> Value {
    let cells = water.0.max(1);
    let river_frac = water.2 as f32 / cells as f32;
    json!({
        "schema": SIM_RUN_DUMP_SCHEMA,
        "proof_grade": "sim_dump",
        "tick": tick,
        "seed": seed,
        "debug": cfg!(debug_assertions),
        "launch": { "test_scene": test_scene },
        "world": { "width": world_w, "height": world_h, "cells": cells },
        "flags": {
            "world_generated": cells > 1,
            "simulation_entered": sim_entered,
        },
        "counts": {
            "terrain_cells": water.0,
            "water_open_cells": water.1,
            "river_cells": water.2,
            "lake_or_ocean_cells": water.3,
            "veg_live_cells": veg.0,
            "veg_burning_cells": veg.1,
            "veg_burnt_cells": veg.2,
            "veg_water_cells": veg.3,
            "fire_heat_cells": fire_heat_cells,
            "fire_instance_rows": fire_instance_rows,
            "spark_rows": spark_rows,
            "sites_operational": sites_operational,
            "sites_total": sites_total,
        },
        "frac": {
            "river": river_frac,
        },
        "max": {
            "fire_heat": max_heat,
            "mean_heat": mean_heat,
            "mean_fuel": mean_fuel,
        },
        "spark_projection_view": spark_view,
    })
}

fn count_water(matrices: &Query<&ChunkCellMatrix>) -> (u32, u32, u32, u32) {
    let mut cells = 0u32;
    let mut open = 0u32;
    let mut river = 0u32;
    let mut standing = 0u32;
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
                WaterClass::Dry | WaterClass::Wetland => {}
            }
        }
    }
    (cells, open, river, standing)
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

fn collect_dump(
    tick: &SimTick,
    params: Option<&WorldGenParams>,
    launch: Option<&EngineLaunchArgs>,
    base: Option<&State<BaseState>>,
    matrices: &Query<&ChunkCellMatrix>,
    stands: &Query<&ChunkCellStand>,
    overlays: &Query<&ChunkFireOverlay>,
    sites: &Query<&ConstructionSite>,
    ecology: Option<&FireEcologyStats>,
    graph: Option<&RenderProjectionGraph>,
    particles: Option<&WorldFireParticleFrame>,
) -> Value {
    let water = count_water(matrices);
    let veg = count_veg(stands);
    let mut fire_heat_cells = 0u32;
    for overlay in overlays {
        for h in &overlay.heat {
            if *h > 0.04 {
                fire_heat_cells = fire_heat_cells.saturating_add(1);
            }
        }
    }
    let mut sites_operational = 0u32;
    let mut sites_total = 0u32;
    for site in sites {
        sites_total = sites_total.saturating_add(1);
        if site.phase == SiteConstructionPhase::Operational {
            sites_operational = sites_operational.saturating_add(1);
        }
    }
    let scene = launch
        .map(|l| format!("{:?}", l.test_scene))
        .unwrap_or_else(|| "None".into());
    let spark_view = particles
        .map(|p| p.spark_witness.projection_view)
        .unwrap_or("");
    build_sim_run_dump(
        tick.0,
        params.map(|p| p.seed).unwrap_or(0),
        params.map(|p| p.width).unwrap_or(0),
        params.map(|p| p.height).unwrap_or(0),
        &scene,
        base.is_some_and(|s| matches!(*s.get(), BaseState::Simulation)),
        water,
        veg,
        fire_heat_cells,
        graph
            .map(|g| g.fire.instance_buffer.len() as u32)
            .unwrap_or(0),
        particles
            .map(|p| p.spark_witness.rows as u32)
            .unwrap_or(0),
        spark_view,
        ecology.map(|e| e.mean_heat).unwrap_or(0.0),
        ecology.map(|e| e.max_heat).unwrap_or(0.0),
        ecology.map(|e| e.mean_fuel).unwrap_or(0.0),
        sites_operational,
        sites_total,
    )
}

fn write_dump_files(body: &Value) -> bool {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let seed = body.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);
    let tick = body.get("tick").and_then(|v| v.as_u64()).unwrap_or(0);
    let run_id = format!("{epoch}-s{seed}-t{tick}");
    let mut stamped = body.clone();
    if let Some(obj) = stamped.as_object_mut() {
        obj.insert("run_id".into(), json!(run_id));
        obj.insert("written_at_epoch_secs".into(), json!(epoch));
    }
    let Ok(text) = serde_json::to_string_pretty(&stamped) else {
        return false;
    };
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let latest = root.join(SIM_RUN_DUMP_JSON);
    if let Some(dir) = latest.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if fs::write(&latest, &text).is_err() {
        return false;
    }
    let archive_dir = root.join("debug_runs").join("sim_runs");
    let _ = fs::create_dir_all(&archive_dir);
    let _ = fs::write(archive_dir.join(format!("{run_id}.json")), text);
    true
}

fn flush_sim_run_dump(
    tick: Res<SimTick>,
    params: Option<Res<WorldGenParams>>,
    launch: Option<Res<EngineLaunchArgs>>,
    base: Option<Res<State<BaseState>>>,
    matrices: Query<&ChunkCellMatrix>,
    stands: Query<&ChunkCellStand>,
    overlays: Query<&ChunkFireOverlay>,
    sites: Query<&ConstructionSite>,
    ecology: Option<Res<FireEcologyStats>>,
    graph: Option<Res<RenderProjectionGraph>>,
    particles: Option<Res<WorldFireParticleFrame>>,
    mut state: ResMut<SimRunDumpState>,
    exit: MessageReader<AppExit>,
) {
    if !sim_dump_enabled(launch.as_deref()) {
        return;
    }
    let exiting = !exit.is_empty();
    let due = exiting
        || (tick.0 >= 8
            && (state.last_tick == u64::MAX || tick.0.saturating_sub(state.last_tick) >= DUMP_STRIDE_TICKS));
    if !due {
        return;
    }
    let body = collect_dump(
        &tick,
        params.as_deref(),
        launch.as_deref(),
        base.as_deref(),
        &matrices,
        &stands,
        &overlays,
        &sites,
        ecology.as_deref(),
        graph.as_deref(),
        particles.as_deref(),
    );
    if write_dump_files(&body) {
        state.last_tick = tick.0;
    }
}

pub struct SimRunDumpPlugin;

impl Plugin for SimRunDumpPlugin {
    fn build(&self, app: &mut App) {
        if !cfg!(debug_assertions) {
            return;
        }
        app.init_resource::<SimRunDumpState>()
            .add_systems(Last, flush_sim_run_dump);
    }
}

#[cfg(test)]
mod tests {
    

}
