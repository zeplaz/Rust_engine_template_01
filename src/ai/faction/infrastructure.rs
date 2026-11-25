//! RD-8 AI builds roads and rail (PLAN-ROADS-v1): an AI faction's **infrastructure brain**. Every
//! couple of sim minutes it looks at its own network use and takes at most one action, through
//! the same funnels a player uses (the line is billed to the faction, built by its trucks):
//!
//! 1. **Clear rubble** off a road or railway near its sites that rubble or a crater blocks
//!    (cut roads reroute by themselves — the router treats them as walls — this reopens them).
//! 2. **Upgrade** a busy road near its sites (volume / capacity ≥ `upgrade_vc`; track → dirt →
//!    gravel → paved; worn tracks included).
//! 3. **Pave** a long truck leg that runs mostly off-road: a gravel road along the route assist.
//! 4. **Rail link** a heavy long haul with no yards: a freight yard near each end and a railway
//!    between them (route assist); hauls then go by rail (RD-7 intermodal).
//!
//! Pure planner [`plan_infrastructure`] over an [`InfraView`]; [`ai_infrastructure_system`] builds
//! the view and carries the action out. Tuning: `road_tiers.ron` `ai`.

use std::collections::{BTreeMap, HashSet};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::FactionAiController;
use crate::construction::corridor_network::{CorridorKind, CorridorNetwork};
use crate::construction::corridor_tool::{assist_route, check_corridor_path, commit_corridor_path_for, CorridorToolKind};
use crate::construction::{ConstructionPlanQueue, ConstructionIntent, ConstructionType};
use crate::economy::logistics::orders::snapshot::{LogisticsOrdersSnapshot, SiteRole};
use crate::economy::logistics::orders::{LogisticsOrdersSnapshotRes, OwnerId, SiteLegRouteBook};
use crate::infrastructure::road_tier::AiRoadTuning;
use crate::infrastructure::RoadTier;
use crate::strategic::{BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype, SiteId};
use crate::systems::sim_control::SimStepClock;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// One infrastructure decision.
#[derive(Clone, Debug, PartialEq)]
pub enum InfraAction {
    ClearRubble { tiles: Vec<IVec2> },
    Upgrade { a: BuildSiteTile, b: BuildSiteTile, to: RoadTier },
    PaveLeg { from: Vec2, to: Vec2 },
    RailLink { from: Vec2, to: Vec2 },
}

/// What the faction sees of its network (tile units).
#[derive(Clone, Debug, Default)]
pub struct InfraView {
    /// Where its sites stand.
    pub sites: Vec<Vec2>,
    /// Road stretches near its sites: ends, tier, volume / capacity.
    pub roads: Vec<(BuildSiteTile, BuildSiteTile, RoadTier, f32)>,
    /// Blocked stretches near its sites and the rubble / crater tiles on them.
    pub blocked: Vec<Vec<IVec2>>,
    /// Its truck legs: ends, metres on roads, metres off-road.
    pub legs: Vec<(Vec2, Vec2, f32, f32)>,
    /// Its hauls: ends, units per hour.
    pub hauls: Vec<(Vec2, Vec2, f32)>,
    /// Its freight yards.
    pub yards: Vec<Vec2>,
}

/// The faction's next infrastructure action, most urgent first.
#[must_use]
pub fn plan_infrastructure(v: &InfraView, t: &AiRoadTuning, rail_reach: f32, rail_min_tiles: f32) -> Option<InfraAction> {
    if let Some(tiles) = v.blocked.iter().find(|t| !t.is_empty()) {
        return Some(InfraAction::ClearRubble { tiles: tiles.clone() });
    }
    if let Some((a, b, tier, _)) = v
        .roads
        .iter()
        .filter(|(_, _, tier, vc)| *vc >= t.upgrade_vc && matches!(tier, RoadTier::Track | RoadTier::Dirt | RoadTier::Gravel))
        .max_by(|x, y| x.3.total_cmp(&y.3))
    {
        return Some(InfraAction::Upgrade { a: *a, b: *b, to: crate::construction::next_road_tier(*tier) });
    }
    if let Some((from, to, _, _)) = v
        .legs
        .iter()
        .filter(|(_, _, road, off)| *off >= t.pave_min_offroad_m && *road / (*road + *off).max(1.0) <= t.pave_max_road_share)
        .max_by(|x, y| x.3.total_cmp(&y.3))
    {
        return Some(InfraAction::PaveLeg { from: *from, to: *to });
    }
    let near_yard = |p: Vec2| v.yards.iter().any(|y| y.distance(p) <= rail_reach * 3.0);
    if let Some((from, to, _)) = v
        .hauls
        .iter()
        .filter(|(a, b, uh)| *uh >= t.rail_min_units_h && a.distance(*b) >= rail_min_tiles && !(near_yard(*a) && near_yard(*b)))
        .max_by(|x, y| x.2.total_cmp(&y.2))
    {
        return Some(InfraAction::RailLink { from: *from, to: *to });
    }
    None
}

/// Everything the brain reads (one domain: the faction's network use; read only).
#[derive(SystemParam)]
pub struct InfraInputs<'w> {
    snapshot: Option<Res<'w, LogisticsOrdersSnapshotRes>>,
    legs: Option<Res<'w, SiteLegRouteBook>>,
    network: Option<Res<'w, CorridorNetwork>>,
    flow: Option<Res<'w, crate::economy::logistics::road_flow::RoadFlowBook>>,
    directory: Option<Res<'w, crate::systems::transport::TransportEdgeDirectory>>,
    debris: Option<Res<'w, crate::economy::logistics::debris_block::DebrisRouteBlockBook>>,
    delta: Option<Res<'w, crate::terrain::ground_deform::GroundDeltaBook>>,
}

/// Ground the corridor checks and route assist read.
#[derive(SystemParam)]
pub struct InfraTerrain<'w> {
    params: Option<Res<'w, crate::terrain::generation::world_generator_enhanced::WorldGenParams>>,
    dense: Option<Res<'w, crate::terrain::generation::WorldGenDenseTerrainCache>>,
    water: crate::systems::water::WaterTileProbe<'w>,
}

/// Where the brain's actions go (request queues of their owners).
#[derive(SystemParam)]
pub struct InfraSinks<'w> {
    queue: Option<ResMut<'w, ConstructionPlanQueue>>,
    commits: Option<ResMut<'w, Messages<CommitConstructionSiteEvent>>>,
    deform: Option<ResMut<'w, crate::terrain::ground_deform::GroundDeformQueue>>,
}

fn centre(t: BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

impl InfraInputs<'_> {
    fn view(&self, slot: u8, t: &AiRoadTuning) -> Option<InfraView> {
        let snap: &LogisticsOrdersSnapshot = &self.snapshot.as_deref()?.0;
        let owner = OwnerId(slot);
        let pos = |id: crate::economy::logistics::orders::model::SiteId| snap.site(id).map(|s| Vec2::from(s.world_xz) / TILE_METRES_GAMEPLAY);
        let sites: Vec<Vec2> = snap.sites.iter().filter(|s| s.owner == owner).map(|s| Vec2::from(s.world_xz) / TILE_METRES_GAMEPLAY).collect();
        if sites.is_empty() {
            return None;
        }
        let near = |p: Vec2| sites.iter().any(|s| s.distance(p) <= t.near_tiles);
        let dir = self.directory.as_deref();
        let mut roads = Vec::new();
        let mut blocked = Vec::new();
        let rubble: HashSet<IVec2> = self
            .debris
            .as_deref()
            .map(|b| {
                b.blocked
                    .iter()
                    .filter_map(|k| {
                        let (x, z) = k.strip_prefix('t')?.split_once('_')?;
                        Some(IVec2::new(x.parse().ok()?, z.parse().ok()?))
                    })
                    .collect()
            })
            .unwrap_or_default();
        if let Some(net) = self.network.as_deref() {
            for seg in &net.segments {
                for (a, b) in seg.spans() {
                    if !near((centre(a) + centre(b)) * 0.5) {
                        continue;
                    }
                    let on: Vec<IVec2> = crate::construction::build_site_tiles_between(a, b)
                        .into_iter()
                        .map(|t| IVec2::new(t.x as i32, t.z as i32))
                        .filter(|t| rubble.contains(t))
                        .collect();
                    if !on.is_empty() {
                        blocked.push(on);
                    }
                    if let CorridorKind::Road { tier, .. } = seg.kind {
                        let vc = dir
                            .and_then(|d| crate::construction::find_edge_id_for_segment(d, a, b))
                            .and_then(|id| self.flow.as_deref().and_then(|f| f.by_edge.get(&id)))
                            .map_or(0.0, |f| f.vc());
                        roads.push((a, b, tier, vc));
                    }
                }
            }
        }
        let offroad = crate::infrastructure::road_tier_table().offroad_speed_mps();
        let legs = self
            .legs
            .as_deref()
            .map(|book| {
                book.legs
                    .iter()
                    .filter(|((a, _), _)| snap.site(*a).is_some_and(|s| s.owner == owner))
                    .filter_map(|((a, b), leg)| {
                        let (road, off) = leg.route.road_offroad_m(offroad);
                        Some((pos(*a)?, pos(*b)?, road, off))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let hauls = snap
            .orders
            .iter()
            .filter(|o| o.owner == owner && !o.status.is_terminal())
            .filter_map(|o| Some((pos(o.source)?, pos(o.sink)?, o.throughput_per_min * 60.0)))
            .collect();
        let yards = snap
            .sites
            .iter()
            .filter(|s| s.owner == owner && s.role == SiteRole::Yard)
            .map(|s| Vec2::from(s.world_xz) / TILE_METRES_GAMEPLAY)
            .collect();
        Some(InfraView { sites, roads, blocked, legs, hauls, yards })
    }
}

/// Each enabled AI faction takes at most one infrastructure action per `ai.interval_s`.
pub fn ai_infrastructure_system(
    clock: Option<Res<SimStepClock>>,
    controllers: Query<(Entity, &FactionAiController)>,
    inputs: InfraInputs,
    terrain: InfraTerrain,
    mut sinks: InfraSinks,
    mut last: Local<BTreeMap<u8, u64>>,
) {
    let Some(clock) = clock else {
        return;
    };
    let table = crate::infrastructure::road_tier_table();
    let t = &table.ai;
    for (entity, c) in &controllers {
        if !c.enabled {
            continue;
        }
        // Stagger factions so they do not all plan in the same step.
        let due = last.get(&c.slot).map_or(clock.steps >= u64::from(c.slot) * 7, |s| clock.steps >= s + t.interval_s);
        if !due {
            continue;
        }
        last.insert(c.slot, clock.steps);
        let Some(view) = inputs.view(c.slot, t) else {
            continue;
        };
        let Some(action) = plan_infrastructure(&view, t, table.rail.yard_reach_tiles, table.rail.intermodal_min_tiles) else {
            continue;
        };
        bevy::log::info!(target: "ai::infrastructure", slot = c.slot, ?action, "AI infrastructure");
        carry_out(c.slot, entity, &action, &inputs, &terrain, &mut sinks);
    }
}

fn carry_out(slot: u8, owner: Entity, action: &InfraAction, inputs: &InfraInputs, terrain: &InfraTerrain<'_>, sinks: &mut InfraSinks) {
    match action {
        InfraAction::ClearRubble { tiles } => {
            let (Some(delta), Some(deform)) = (inputs.delta.as_deref(), sinks.deform.as_deref_mut()) else {
                return;
            };
            // Engineers level the heap / fill the crater back to the ground it was.
            let cells: Vec<(IVec2, f32)> = tiles.iter().map(|t| (*t, -delta.delta_at(t.x, t.y))).filter(|(_, d)| d.abs() > 0.05).collect();
            if !cells.is_empty() {
                deform.push(crate::terrain::ground_deform::GroundDeformOp::Grade { cells });
            }
        }
        InfraAction::Upgrade { a, b, to } => {
            let Some(queue) = sinks.queue.as_deref_mut() else { return };
            queue.enqueue(ConstructionIntent {
                entity_type: ConstructionType::RoadSegment { head: *a, tail: *b, tier: *to },
                world_position: (centre(*a) + centre(*b)) * 0.5,
                rotation: 0.0,
            });
            if let Some(plan) = queue.plans.last_mut() {
                plan.charged = true;
                plan.owner_slot = slot;
            }
        }
        InfraAction::PaveLeg { from, to } => {
            build_line(slot, *from, *to, CorridorToolKind::Road(RoadTier::Gravel), inputs, terrain, sinks);
        }
        InfraAction::RailLink { from, to } => {
            // A yard beside each end, then the railway between the yards.
            let (Some(params), Some(commits)) = (terrain.params.as_deref(), sinks.commits.as_deref_mut()) else {
                return;
            };
            let yard_at = |p: Vec2, away: Vec2| {
                let o = (p + (away - p).normalize_or_zero() * 4.0).floor().as_ivec2();
                IVec2::new(o.x.clamp(1, params.width as i32 - 10), o.y.clamp(1, params.height as i32 - 5))
            };
            let (ya, yb) = (yard_at(*from, *to), yard_at(*to, *from));
            for y in [ya, yb] {
                commits.write(CommitConstructionSiteEvent {
                    site_id: SiteId::UNASSIGNED,
                    owner,
                    archetype: SiteArchetype::RailDepot,
                    origin: BuildSiteTile { x: y.x as u32, z: y.y as u32 },
                    footprint: FootprintTiles { width: 8, depth: 3 },
                    layer: LayerType::Surface,
                    catalog_id: Some("freight_yard_v1".into()),
                    placement: None,
                });
            }
            // Track along the yards' long side, so the yard reaches it.
            let track = |y: IVec2| y.as_vec2() + Vec2::new(4.0, 4.5);
            build_line(slot, track(ya), track(yb), CorridorToolKind::Rail, inputs, terrain, sinks);
        }
    }
}

/// Check and commit a line from `from` to `to` along the route assist, billed to `slot`.
fn build_line(slot: u8, from: Vec2, to: Vec2, kind: CorridorToolKind, inputs: &InfraInputs, terrain: &InfraTerrain<'_>, sinks: &mut InfraSinks) {
    let (Some(params), Some(net), Some(queue)) = (terrain.params.as_deref(), inputs.network.as_deref(), sinks.queue.as_deref_mut()) else {
        return;
    };
    let grid = terrain.dense.as_deref();
    let ground = |p: Vec2| crate::construction::terrain_conform::sample_height_norm(p.x, p.y, params, grid) * params.elevation_range_m;
    let wet = |t: IVec2| terrain.water.depth_m(t);
    let map = UVec2::new(params.width, params.height);
    let mut path = vec![from];
    match assist_route(from, to, kind, &ground, &wet, net, map) {
        Some(rest) => path.extend(rest),
        None => path.push(to),
    }
    let checks = check_corridor_path(&path, kind, &ground, &wet, net, map);
    commit_corridor_path_for(&checks, kind, queue, slot);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> AiRoadTuning {
        AiRoadTuning::default()
    }

    fn tile(x: u32, z: u32) -> BuildSiteTile {
        BuildSiteTile { x, z }
    }

    #[test]
    fn rubble_first_then_the_busiest_unpaved_road() {
        let mut v = InfraView { sites: vec![Vec2::ZERO], ..Default::default() };
        v.roads = vec![(tile(0, 0), tile(8, 0), RoadTier::Dirt, 0.9), (tile(8, 0), tile(16, 0), RoadTier::Gravel, 0.7), (tile(0, 4), tile(8, 4), RoadTier::Paved, 2.0)];
        assert_eq!(plan_infrastructure(&v, &t(), 10.0, 60.0), Some(InfraAction::Upgrade { a: tile(0, 0), b: tile(8, 0), to: RoadTier::Gravel }), "paved roads are not upgraded here");
        v.blocked = vec![vec![IVec2::new(3, 0)]];
        assert_eq!(plan_infrastructure(&v, &t(), 10.0, 60.0), Some(InfraAction::ClearRubble { tiles: vec![IVec2::new(3, 0)] }));
    }

    #[test]
    fn a_long_mostly_off_road_leg_gets_a_road_and_a_heavy_long_haul_gets_rail() {
        let mut v = InfraView { sites: vec![Vec2::ZERO], ..Default::default() };
        v.legs = vec![(Vec2::ZERO, Vec2::new(80.0, 0.0), 100.0, 1200.0), (Vec2::ZERO, Vec2::new(20.0, 0.0), 300.0, 30.0)];
        assert_eq!(plan_infrastructure(&v, &t(), 10.0, 60.0), Some(InfraAction::PaveLeg { from: Vec2::ZERO, to: Vec2::new(80.0, 0.0) }));
        v.legs.clear();
        v.hauls = vec![(Vec2::ZERO, Vec2::new(200.0, 0.0), 500.0), (Vec2::ZERO, Vec2::new(30.0, 0.0), 900.0)];
        assert_eq!(plan_infrastructure(&v, &t(), 10.0, 60.0), Some(InfraAction::RailLink { from: Vec2::ZERO, to: Vec2::new(200.0, 0.0) }), "the short haul stays on the road");
        v.yards = vec![Vec2::new(2.0, 0.0), Vec2::new(198.0, 0.0)];
        assert_eq!(plan_infrastructure(&v, &t(), 10.0, 60.0), None, "yards already serve it");
    }
}
