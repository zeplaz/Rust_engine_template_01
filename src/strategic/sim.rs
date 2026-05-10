//! Live **simulation hooks** for [`strategic_fields_and_ai_orchestrator_v1.md`](../../prompts/guides/strategic_fields_and_ai_orchestrator_v1.md)
//! child runbooks — extends [`super::plugin::StrategicFieldsPlugin`] field buffers with coupling + AI-facing aggregates.

use bevy::prelude::*;

use super::logistics_net::logistics_net_inject_into_overlays;
use super::runbook_rounds::city_planning::utility_redundancy_weight;
use super::runbook_rounds::city_planning::{site_score, SettlementArchetype};
use super::runbook_rounds::corridor::CorridorType;
use super::runbook_rounds::settlement::{ecology_hazard_pressure, migration_pull};
use super::transport_bridge::transport_mean_damage;
use super::{ChunkStrategicOverlay, MAX_STRATEGIC_FACTION_SLOTS};
use crate::entities::production::core::{ResourceProducer, ResourceStorage, ResourceStorageCapacity};
use crate::systems::transport::TransportFieldStore;

/// Links [`InfrastructureCorridor`] wear to a live transport edge.
#[derive(Component, Clone, Debug)]
pub struct StrategicTransportCorridor {
    pub edge_id: crate::systems::transport::TransportEdgeId,
}

/// **infrastructure_corridor_runbook** — corridor spine attached to world entities; `wear` reflects transport fields when linked.
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

/// **settlement_growth_runbook** — coarse population site; socio signals feed migration pull.
#[derive(Component, Clone, Debug)]
pub struct SettlementSite {
    pub population: u32,
    pub anchor_chunk: IVec2,
    pub water_access: f32,
    /// Fractional population growth (deterministic accrual).
    pub growth_accumulator: f32,
    pub jobs_opportunity: f32,
    pub housing_supply: f32,
    pub public_safety: f32,
    pub informal_pressure: f32,
    pub adaptation_reserve: f32,
}

impl SettlementSite {
    pub fn new(population: u32, anchor_chunk: IVec2, water_access: f32) -> Self {
        Self {
            population,
            anchor_chunk,
            water_access,
            growth_accumulator: 0.0,
            jobs_opportunity: 0.5,
            housing_supply: 0.6,
            public_safety: 0.7,
            informal_pressure: 0.0,
            adaptation_reserve: 0.25,
        }
    }
}

/// **logistics_ai_runbook** — telemetry for AI / HUD (graph inject + production + transport health).
#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsAiRuntime {
    pub congestion_proxy: f32,
    pub mean_edge_damage: f32,
    pub stockpile_fill_ratio: f32,
    pub industrial_output_proxy: f32,
}

/// **ai_operational_warfare_runbook** — rolled-up threat / sustain metrics per faction slot.
#[derive(Resource, Clone, Debug)]
pub struct OperationalTheaterSummary {
    pub mean_threat_by_slot: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub mean_logistics_strength_by_slot: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub active_faction_slots: usize,
}

impl Default for OperationalTheaterSummary {
    fn default() -> Self {
        Self {
            mean_threat_by_slot: [0.0; MAX_STRATEGIC_FACTION_SLOTS],
            mean_logistics_strength_by_slot: [0.0; MAX_STRATEGIC_FACTION_SLOTS],
            active_faction_slots: 0,
        }
    }
}

/// **ai_city_planning_runbook** — planning hints for UI / downstream builders.
#[derive(Resource, Clone, Debug)]
pub struct CityPlanningHints {
    pub last_best_site_score: f32,
    pub utility_redundancy_hint: f32,
    pub adaptive_rebuild_pressure: f32,
    pub primary_archetype: SettlementArchetype,
}

impl Default for CityPlanningHints {
    fn default() -> Self {
        Self {
            last_best_site_score: 0.0,
            utility_redundancy_hint: 1.0,
            adaptive_rebuild_pressure: 0.0,
            primary_archetype: SettlementArchetype::AgriculturalRegion,
        }
    }
}

pub struct StrategicSimulationPlugin;

impl Plugin for StrategicSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransportFieldStore>();
        app.init_resource::<LogisticsAiRuntime>()
            .init_resource::<OperationalTheaterSummary>()
            .init_resource::<CityPlanningHints>()
            .add_systems(
                Update,
                refresh_settlement_socio_signals.before(strategic_fields_coupling_tick),
            )
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

fn refresh_settlement_socio_signals(
    mut settlements: Query<&mut SettlementSite>,
    overlays: Query<&ChunkStrategicOverlay>,
    producers: Query<&ResourceProducer>,
) {
    let job_proxy = (producers.iter().count() as f32 / 12.0).min(1.0);
    for mut s in &mut settlements {
        let mut mean_threat = 0.0f32;
        let mut n = 0.0f32;
        let mut fire_avg = 0.0f32;
        for o in &overlays {
            if o.chunk_coord != s.anchor_chunk {
                continue;
            }
            for i in 0..o.len_cells() {
                mean_threat += o.threat[i][0];
                fire_avg += o.fire_risk[i];
                n += 1.0;
            }
        }
        if n > 0.0 {
            mean_threat /= n;
            fire_avg /= n;
        }
        s.public_safety = (1.0 - mean_threat).clamp(0.0, 1.0);
        s.jobs_opportunity = job_proxy;
        let pop_f = s.population as f32;
        s.housing_supply = (1.0 / (1.0 + pop_f * 0.0003)).clamp(0.1, 1.0);
        let pull = migration_pull(s.jobs_opportunity, s.public_safety, s.housing_supply);
        s.informal_pressure = ((1.0 - s.housing_supply) * pull).clamp(0.0, 1.0);
        let eco = ecology_hazard_pressure(fire_avg, 0.0);
        s.adaptation_reserve = (s.adaptation_reserve + 0.015 * (1.0 - eco)).min(1.0);
    }
}

/// **strategic_overlay** + **logistics_ai** + **operational** + **city_planning** — one pass for overlay coupling and aggregates.
#[allow(clippy::too_many_arguments)]
fn strategic_fields_coupling_tick(
    mut overlays: Query<&mut ChunkStrategicOverlay>,
    transport_fields: Res<TransportFieldStore>,
    mut logistics_ai: ResMut<LogisticsAiRuntime>,
    mut theater: ResMut<OperationalTheaterSummary>,
    mut city: ResMut<CityPlanningHints>,
    settlements: Query<&SettlementSite>,
    storages: Query<&ResourceStorage>,
    caps: Query<&ResourceStorageCapacity>,
    producers: Query<&ResourceProducer>,
) {
    let mut cong_acc = 0.0f32;
    let mut cong_n = 0.0f32;
    let mut ncells = 0.0f32;

    let mut threat_acc = [0.0f32; MAX_STRATEGIC_FACTION_SLOTS];
    let mut logi_acc = [0.0f32; MAX_STRATEGIC_FACTION_SLOTS];

    for mut o in &mut overlays {
        for i in 0..o.len_cells() {
            let t = o.logistics_throughput.get(i).copied().unwrap_or(0.0);
            let rc = o.routing_congestion.get(i).copied().unwrap_or(0.0);
            let ew = o.ew_denial.get(i).copied().unwrap_or(0.0);
            o.mobility_cost[i] = (1.0 - t.clamp(0.0, 1.0) + 0.28 * rc).clamp(0.0, 1.0);
            if i < o.civilian_stability.len() {
                let th0 = o.threat[i][0].clamp(0.0, 1.0);
                o.civilian_stability[i] =
                    (0.65 * (1.0 - th0) + 0.35 * (1.0 - ew)).clamp(0.0, 1.0);
            }
            if i < o.recon_confidence.len() {
                for slot in 0..MAX_STRATEGIC_FACTION_SLOTS {
                    let base = o.recon_confidence[i][slot];
                    o.recon_confidence[i][slot] = (base * (1.0 - ew * 0.65)).clamp(0.0, 1.0);
                }
            }
            cong_acc += 1.0 - t.min(1.0);
            cong_n += 1.0;
            ncells += 1.0;
            for s in 0..MAX_STRATEGIC_FACTION_SLOTS {
                threat_acc[s] += o.threat[i][s];
                logi_acc[s] += o.logistics_strength[i][s];
            }
        }
    }

    logistics_ai.congestion_proxy = if cong_n > 0.0 { cong_acc / cong_n } else { 0.0 };
    logistics_ai.mean_edge_damage = transport_mean_damage(&transport_fields);

    let mut sum_amt = 0.0f32;
    let mut sum_cap = 0.0f32;
    for s in &storages {
        for &v in s.amounts.values() {
            sum_amt += v;
        }
    }
    for c in &caps {
        for &m in c.max_amounts.values() {
            sum_cap += m;
        }
    }
    logistics_ai.stockpile_fill_ratio = if sum_cap > 0.0 {
        (sum_amt / sum_cap).min(1.0)
    } else {
        0.0
    };
    let prod_sum: f32 = producers
        .iter()
        .map(|p| p.production_rate * p.efficiency.max(0.01))
        .sum();
    logistics_ai.industrial_output_proxy = (prod_sum / 80.0).min(1.0);

    if ncells > 0.0 {
        let mut active = 0usize;
        for s in 0..MAX_STRATEGIC_FACTION_SLOTS {
            theater.mean_threat_by_slot[s] = threat_acc[s] / ncells;
            theater.mean_logistics_strength_by_slot[s] = logi_acc[s] / ncells;
            if theater.mean_threat_by_slot[s] > 0.02 || theater.mean_logistics_strength_by_slot[s] > 0.02 {
                active = active.max(s + 1);
            }
        }
        theater.active_faction_slots = active;
    } else {
        theater.mean_threat_by_slot = [0.0; MAX_STRATEGIC_FACTION_SLOTS];
        theater.mean_logistics_strength_by_slot = [0.0; MAX_STRATEGIC_FACTION_SLOTS];
        theater.active_faction_slots = 0;
    }

    let dmg = logistics_ai.mean_edge_damage;
    let mean_th_s0 = theater.mean_threat_by_slot[0];

    let mut best = f32::NEG_INFINITY;
    let mut arche_maj = SettlementArchetype::AgriculturalRegion;
    for s in &settlements {
        let tier = super::runbook_rounds::settlement::tier_from_population(s.population);
        arche_maj = match tier {
            super::runbook_rounds::settlement::SettlementTier::Metropolis
            | super::runbook_rounds::settlement::SettlementTier::City => SettlementArchetype::IndustrialHub,
            super::runbook_rounds::settlement::SettlementTier::Town => SettlementArchetype::LogisticsJunction,
            _ => SettlementArchetype::AgriculturalRegion,
        };
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
    city.primary_archetype = arche_maj;
    city.utility_redundancy_hint = (utility_redundancy_weight(arche_maj)
        * (1.0 - logistics_ai.congestion_proxy * 0.35 - dmg * 0.25))
        .clamp(0.15, 2.5);
    city.adaptive_rebuild_pressure = (dmg * 0.55 + mean_th_s0 * 0.45).clamp(0.0, 1.0);
}

/// **settlement_growth** + **infrastructure_corridor** maintenance.
#[allow(clippy::type_complexity)]
fn settlement_and_corridor_tick(
    time: Res<Time>,
    mut settlements: Query<&mut SettlementSite>,
    mut standalone_corridors: Query<&mut InfrastructureCorridor, Without<StrategicTransportCorridor>>,
    mut linked_corridors: Query<
        (&mut InfrastructureCorridor, &StrategicTransportCorridor),
    >,
    fields: Res<TransportFieldStore>,
) {
    let dt = time.delta_secs().clamp(0.0, 0.25);
    let wear_dt = dt * 0.01;
    for mut s in &mut settlements {
        let pull = migration_pull(
            s.jobs_opportunity,
            s.public_safety,
            s.housing_supply,
        );
        let rate = 2.0
            * s.water_access.clamp(0.0, 2.0)
            * (0.45 + 0.55 * pull)
            * (1.0 - s.informal_pressure * 0.15);
        s.growth_accumulator += rate * dt;
        while s.growth_accumulator >= 1.0 {
            s.population = s.population.saturating_add(1);
            s.growth_accumulator -= 1.0;
        }
    }
    for mut c in &mut standalone_corridors {
        c.wear = (c.wear + wear_dt).min(1.0);
    }
    for (mut c, link) in &mut linked_corridors {
        if let Some(st) = fields.by_edge.get(&link.edge_id) {
            c.wear = (st.damage * 0.55 + st.congestion * 0.35 + st.danger * 0.1).clamp(0.0, 1.0);
        }
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
        app.world_mut()
            .spawn(InfrastructureCorridor::new(crate::strategic::CorridorType::Rail));

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
        assert!(la.mean_edge_damage >= 0.0);

        let od = app.world().resource::<OperationalTheaterSummary>();
        assert!(od.mean_logistics_strength_by_slot[0] >= 0.0);

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
