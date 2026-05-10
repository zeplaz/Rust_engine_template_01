//! Live **simulation hooks** for [`strategic_fields_and_ai_orchestrator_v1.md`](../../prompts/guides/strategic_fields_and_ai_orchestrator_v1.md)
//! child runbooks — extends [`super::plugin::StrategicFieldsPlugin`] field buffers with coupling + AI-facing aggregates.

use bevy::prelude::*;

use super::logistics_net::logistics_net_inject_into_overlays;
use super::runbook_rounds::city_planning::site_score;
use super::runbook_rounds::corridor::CorridorType;
use super::ChunkStrategicOverlay;

/// **infrastructure_corridor_runbook** — corridor spine attached to world entities; `wear` accumulates until maintenance systems exist.
#[derive(Component, Clone, Debug)]
pub struct InfrastructureCorridor {
    _corridor_type: CorridorType,
    pub wear: f32,
}

impl InfrastructureCorridor {
    pub fn new(corridor_type: CorridorType) -> Self {
        Self {
            _corridor_type: corridor_type,
            wear: 0.0,
        }
    }

    #[inline]
    pub fn corridor_type(&self) -> CorridorType {
        self._corridor_type
    }
}

/// **settlement_growth_runbook** — coarse population site; `anchor_chunk` selects which [`ChunkStrategicOverlay`] is evaluated for city planning.
#[derive(Component, Clone, Debug)]
pub struct SettlementSite {
    pub population: u32,
    pub anchor_chunk: IVec2,
    pub water_access: f32,
    /// Fractional population growth (deterministic accrual).
    pub growth_accumulator: f32,
}

impl SettlementSite {
    pub fn new(population: u32, anchor_chunk: IVec2, water_access: f32) -> Self {
        Self {
            population,
            anchor_chunk,
            water_access,
            growth_accumulator: 0.0,
        }
    }
}

/// **logistics_ai_runbook** — running telemetry logistics AI / UI can read (congestion proxy from throughput field).
#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsAiRuntime {
    pub congestion_proxy: f32,
}

/// **ai_operational_warfare_runbook** — rolled-up threat / sustain metrics for operational AI.
#[derive(Resource, Clone, Debug, Default)]
pub struct OperationalTheaterSummary {
    pub mean_threat_slot0: f32,
    pub mean_logistics_strength: f32,
}

/// **ai_city_planning_runbook** — last evaluated best site score across anchored settlements.
#[derive(Resource, Clone, Debug, Default)]
pub struct CityPlanningHints {
    pub last_best_site_score: f32,
}

pub struct StrategicSimulationPlugin;

impl Plugin for StrategicSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LogisticsAiRuntime>()
            .init_resource::<OperationalTheaterSummary>()
            .init_resource::<CityPlanningHints>()
            .add_systems(
                Update,
                strategic_fields_coupling_tick.after(logistics_net_inject_into_overlays),
            )
            .add_systems(
                Update,
                settlement_and_corridor_tick.after(strategic_fields_coupling_tick),
            );
    }
}

/// **strategic_overlay** + **logistics_ai** + **operational** + **city_planning** — one pass for overlay coupling and aggregates.
fn strategic_fields_coupling_tick(
    mut overlays: Query<&mut ChunkStrategicOverlay>,
    mut logistics_ai: ResMut<LogisticsAiRuntime>,
    mut theater: ResMut<OperationalTheaterSummary>,
    mut city: ResMut<CityPlanningHints>,
    settlements: Query<&SettlementSite>,
) {
    let mut cong_acc = 0.0f32;
    let mut cong_n = 0.0f32;
    let mut tsum = 0.0f32;
    let mut lsum = 0.0f32;
    let mut ncells = 0.0f32;

    for mut o in &mut overlays {
        for i in 0..o.len_cells() {
            let t = o.logistics_throughput.get(i).copied().unwrap_or(0.0);
            o.mobility_cost[i] = 1.0 - t.clamp(0.0, 1.0);
            cong_acc += 1.0 - t.min(1.0);
            cong_n += 1.0;
            tsum += o.threat[i][0];
            lsum += o.logistics_strength[i][0];
            ncells += 1.0;
        }
    }

    logistics_ai.congestion_proxy = if cong_n > 0.0 { cong_acc / cong_n } else { 0.0 };
    if ncells > 0.0 {
        theater.mean_threat_slot0 = tsum / ncells;
        theater.mean_logistics_strength = lsum / ncells;
    } else {
        theater.mean_threat_slot0 = 0.0;
        theater.mean_logistics_strength = 0.0;
    }

    let mut best = f32::NEG_INFINITY;
    for s in &settlements {
        for o in overlays.iter() {
            if s.anchor_chunk != o.chunk_coord {
                continue;
            }
            let n = o.len_cells().max(1) as f32;
            let mut ls = 0.0f32;
            let mut th = 0.0f32;
            let mut fr = 0.0f32;
            for i in 0..o.len_cells() {
                ls += o.logistics_strength[i][0];
                th += o.threat[i][0];
                fr += o.fire_risk[i];
            }
            let sc = site_score(ls / n, fr / n, th / n);
            best = best.max(sc);
        }
    }
    city.last_best_site_score = if best.is_finite() { best } else { 0.0 };
}

/// **settlement_growth** + **infrastructure_corridor** maintenance stub.
fn settlement_and_corridor_tick(
    time: Res<Time>,
    mut settlements: Query<&mut SettlementSite>,
    mut corridors: Query<&mut InfrastructureCorridor>,
) {
    let dt = time.delta_secs().clamp(0.0, 0.25);
    for mut s in &mut settlements {
        let rate = 2.0 * s.water_access.clamp(0.0, 2.0);
        s.growth_accumulator += rate * dt;
        while s.growth_accumulator >= 1.0 {
            s.population = s.population.saturating_add(1);
            s.growth_accumulator -= 1.0;
        }
    }
    let wear_dt = dt * 0.01;
    for mut c in &mut corridors {
        c.wear = (c.wear + wear_dt).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::logistics_net::logistics_net_inject_into_overlays;
    use crate::strategic::{
        LogisticsEdge, LogisticsGraph, LogisticsNode, LogisticsNodeId, StrategicFieldsAndAiPlugin,
    };
    use crate::systems::terrain::MaterialUnificationPlugin;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use crate::terrain::ChunkCellKey;
    use bevy::asset::AssetPlugin;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    #[test]
    fn strategic_bundle_runs_coupling_and_settlement_growth() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .init_resource::<LogisticsGraph>()
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(StrategicFieldsAndAiPlugin)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.5)));

        app.world_mut().insert_resource(LogisticsGraph {
            nodes: vec![
                LogisticsNode {
                    id: LogisticsNodeId(0),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
                },
                LogisticsNode {
                    id: LogisticsNodeId(1),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
                },
            ],
            edges: vec![LogisticsEdge {
                from: LogisticsNodeId(0),
                to: LogisticsNodeId(1),
                capacity: 1.0,
                disruption: 0.0,
                traversal_cost: 1.0,
            }],
        });

        app.world_mut().spawn((
            Chunk { coord: IVec2::ZERO },
            ChunkCellMatrix::new(UVec2::new(2, 1)),
        ));
        app.world_mut().spawn(SettlementSite::new(100, IVec2::ZERO, 1.0));
        app.world_mut().spawn(InfrastructureCorridor::new(CorridorType::Rail));

        for _ in 0..10 {
            app.update();
        }

        let pop = {
            let mut q = app.world_mut().query::<&SettlementSite>();
            q.iter(app.world()).next().expect("settlement").population
        };
        assert!(pop > 100, "population should grow");

        let wear = {
            let mut q = app.world_mut().query::<&InfrastructureCorridor>();
            q.iter(app.world()).next().expect("corridor").wear
        };
        assert!(wear > 0.0);

        let la = app.world().resource::<LogisticsAiRuntime>();
        assert!(la.congestion_proxy >= 0.0);

        let od = app.world().resource::<OperationalTheaterSummary>();
        assert!(od.mean_logistics_strength >= 0.0);

        // Ensure coupling runs after inject: mobility reflects throughput
        let (_, ov) = {
            let mut q = app.world_mut().query::<(&Chunk, &ChunkStrategicOverlay)>();
            q.iter(app.world()).next().expect("chunk overlay")
        };
        assert!(ov.mobility_cost[0] < 1.0 || ov.logistics_throughput[0] > 0.0);
    }

    #[test]
    fn simulation_plugin_orders_after_logistics_inject() {
        use crate::strategic::StrategicSimulationPlugin;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StrategicSimulationPlugin)
            .add_systems(Update, logistics_net_inject_into_overlays);
        app.world_mut().init_resource::<LogisticsGraph>();
        // would panic if schedule invalid
        app.update();
    }
}
