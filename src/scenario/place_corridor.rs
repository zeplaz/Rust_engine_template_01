//! `ScenarioStep::PlaceCorridor` (PLAN-ROADS-v1 RD-6): a scripted player draws a path with the
//! corridor tool. Same checks ([`check_corridor_path`]) and the same commit
//! ([`commit_corridor_path`]) as a click-drawn path, so captures test what players get. Each run is
//! logged in [`CorridorPlacementLog`] for the `roads` dump section.

use bevy::prelude::*;

use crate::construction::corridor_tool::{
    check_corridor_path, commit_corridor_path, summarize_path, CorridorToolKind, StretchVerdict,
};
use crate::construction::terrain_conform::{live_height_grid_ready, sample_height_norm};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::WorldGenDenseTerrainCache;
use crate::construction::{ConstructionPlanQueue, CorridorNetwork};
use crate::infrastructure::RoadTier;
use crate::systems::water::WaterTileProbe;

use super::scenario_steps::ScenarioStep;
use super::script_host::EngineScriptHost;

/// One scripted placement and what the tool made of it.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorPlacementRecord {
    pub kind: String,
    pub stretches: Vec<String>,
    pub queued: usize,
    pub length_m: f32,
    /// Wall time of the stretch checks (µs): the live preview runs the same per frame.
    pub check_us: u128,
}

/// Scripted placements this run. Sole writer: [`run_place_corridor_step_system`].
#[derive(Resource, Clone, Debug, Default)]
pub struct CorridorPlacementLog(pub Vec<CorridorPlacementRecord>);

/// `dirt` / `gravel` / `paved` / `concrete` / `motorway` / `rail` / `firebreak`.
#[must_use]
pub fn corridor_kind_from_name(name: &str) -> Option<CorridorToolKind> {
    match name.trim().to_ascii_lowercase().as_str() {
        "rail" => Some(CorridorToolKind::Rail),
        "firebreak" => Some(CorridorToolKind::Firebreak),
        other => RoadTier::from_profile_id(&format!("road_{other}")).map(CorridorToolKind::Road),
    }
}

/// Runs a front `PlaceCorridor` step: check, commit, log, pop.
pub fn run_place_corridor_step_system(
    mut host: ResMut<EngineScriptHost>,
    params: Option<Res<WorldGenParams>>,
    heights: Option<Res<WorldGenDenseTerrainCache>>,
    water: WaterTileProbe,
    roads: Option<Res<CorridorNetwork>>,
    queue: Option<ResMut<ConstructionPlanQueue>>,
    mut log: ResMut<CorridorPlacementLog>,
) {
    let Some(ScenarioStep::PlaceCorridor { kind, points, assist }) = host.pending_steps.front() else {
        return;
    };
    // Wait until terrain, water, the network and the plan queue can answer.
    let (Some(params), Some(roads), Some(mut queue)) = (params, roads, queue) else {
        return;
    };
    if !live_height_grid_ready(heights.as_deref()) || !water.ready() {
        return;
    }
    let (name, pts, assist) = (kind.clone(), points.clone(), *assist);
    host.pending_steps.pop_front();
    host.current_step_index = host.current_step_index.saturating_add(1);
    let Some(kind) = corridor_kind_from_name(&name) else {
        host.execution_log.push(format!("PlaceCorridor: unknown kind {name:?}"));
        return;
    };
    let mut path: Vec<Vec2> = pts.iter().map(|p| Vec2::new(p.tile_x as f32 + 0.5, p.tile_z as f32 + 0.5)).collect();
    let params = params.as_ref();
    let grid = heights.as_deref();
    let ground = |p: Vec2| sample_height_norm(p.x, p.y, params, grid) * params.elevation_range_m;
    let wet = |t: IVec2| water.depth_m(t);
    if assist && path.len() >= 2 {
        let map = UVec2::new(params.width, params.height);
        let mut routed = vec![path[0]];
        for w in path.windows(2) {
            match crate::construction::corridor_tool::assist_route(w[0], w[1], kind, &ground, &wet, roads.as_ref(), map) {
                Some(rest) => routed.extend(rest),
                None => routed.push(w[1]),
            }
        }
        path = routed;
    }
    let started = std::time::Instant::now();
    let checks = check_corridor_path(&path, kind, &ground, &wet, roads.as_ref(), UVec2::new(params.width, params.height));
    let check_us = started.elapsed().as_micros();
    let queued = commit_corridor_path(&checks, kind, &mut queue);
    let summary = summarize_path(&checks, kind);
    let stretches = checks
        .iter()
        .map(|c| match &c.verdict {
            StretchVerdict::Ok => format!("{:.0} m ok", c.length_m),
            StretchVerdict::Works(w) => format!("{:.0} m works: {}", c.length_m, w.join(", ")),
            StretchVerdict::Blocked(why) => format!("{:.0} m blocked: {why}", c.length_m),
        })
        .collect::<Vec<_>>();
    host.execution_log.push(format!("PlaceCorridor {name}: {queued} stretch(es) queued; {}", stretches.join(" | ")));
    let rec = CorridorPlacementRecord { kind: name, stretches, queued, length_m: summary.length_m, check_us };
    log.0.push(rec);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_names_map_to_tiers_and_rail() {
        assert_eq!(corridor_kind_from_name("Gravel"), Some(CorridorToolKind::Road(RoadTier::Gravel)));
        assert_eq!(corridor_kind_from_name("motorway"), Some(CorridorToolKind::Road(RoadTier::Motorway)));
        assert_eq!(corridor_kind_from_name("rail"), Some(CorridorToolKind::Rail));
        assert_eq!(corridor_kind_from_name("street"), None);
    }
}
