use bevy::prelude::*;

use crate::construction::{
    BuildingDefinitionRegistry, IndustrialSupplyChainRole, UtilityInfrastructureRole,
};
use crate::economy::IndustrialSupplyChainMembership;
use crate::entities::production::power::capabilities::attach_power_plant_capabilities;
use crate::entities::production::power::components::{
    ElectricalComponent, ElectricalGrid, PowerPlant,
};
use crate::entities::production::power::failure_modes::{
    apply_containment_breach_radiation_hook_system, attach_nuclear_containment_runtime,
    nuclear_decay_heat_cooling_system, nuclear_loop_scram_system, nuclear_meltdown_breach_system,
    project_containment_breach_to_player_event_log, project_meltdown_to_player_event_log,
    project_nuclear_cooling_degraded_to_player_event_log,
    project_nuclear_scram_to_player_event_log, steam_system_placeholder,
    sync_nuclear_offsite_from_utility, variable_renewable_placeholder, ContainmentBreachEvent,
    MeltdownEvent, NuclearCoolingDegradedEvent, NuclearScramEvent,
};
use crate::entities::production::power::grid_topology::{
    emit_grid_overload_signals, purge_removed_power_components_from_grids,
    purge_stale_grid_references, rebuild_electrical_grid_topology,
    recalculate_grid_totals_from_members, GridConnectionRadiusSq, GridOverloadEvent,
};
use crate::entities::production::power::plant_profile::PlantArchetype;
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::types::OperationalStatus;
use crate::strategic::PlannedSite;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::GlobalRenewableWeatherFactors;

pub struct PowerRuntimePlugin;

impl Plugin for PowerRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridConnectionRadiusSq>()
            .init_resource::<PlantDefinitionRegistry>()
            .init_resource::<GlobalRenewableWeatherFactors>()
            .add_message::<GridOverloadEvent>()
            .add_message::<NuclearScramEvent>()
            .add_message::<NuclearCoolingDegradedEvent>()
            .add_message::<MeltdownEvent>()
            .add_message::<ContainmentBreachEvent>()
            .add_systems(
                Update,
                (
                    attach_power_plant_capabilities,
                    attach_nuclear_containment_runtime,
                    rebuild_electrical_grid_topology,
                    purge_removed_power_components_from_grids,
                    purge_stale_grid_references,
                    recalculate_grid_totals_from_members,
                    emit_grid_overload_signals,
                    update_electrical_load_system,
                    update_power_output_system,
                    steam_system_placeholder,
                    sync_nuclear_offsite_from_utility,
                    nuclear_loop_scram_system,
                    nuclear_decay_heat_cooling_system,
                    nuclear_meltdown_breach_system,
                    apply_containment_breach_radiation_hook_system,
                    project_nuclear_scram_to_player_event_log,
                    project_nuclear_cooling_degraded_to_player_event_log,
                    project_meltdown_to_player_event_log,
                    project_containment_breach_to_player_event_log,
                    variable_renewable_placeholder,
                )
                    .chain()
                    .run_if(power_simulation_running),
            );
    }
}

/// Runs when the sim loop should advance (`SimControlState`), matching designer pause/step semantics.
/// When `BaseState` / `SimulationState` are initialized on the app, add them here for stricter editor parity;
/// see `power_legacy_functional_parity_v1.md` §1.
fn power_simulation_running(ctrl: Res<SimControlState>) -> bool {
    ctrl.should_tick()
}

fn update_electrical_load_system(
    registry: Option<Res<BuildingDefinitionRegistry>>,
    grids: Query<&ElectricalGrid>,
    mut loads: Query<(
        Entity,
        &mut ElectricalComponent,
        Option<&PlannedSite>,
        Option<&IndustrialSupplyChainMembership>,
    )>,
) {
    if let Some(registry) = registry.as_deref() {
        for grid in &grids {
            if grid.total_load > grid.total_capacity && grid.total_capacity > 0.0 {
                shed_overloaded_member_loads(registry, grid, &mut loads);
            }
        }
    }
    for (_, mut electrical, _, _) in &mut loads {
        electrical.current_load = electrical
            .current_load
            .clamp(0.0, electrical.max_transfer.max(f32::EPSILON));
    }
}

/// Shed rank. Lower drops first. Unranked roles are omitted.
fn overload_shed_rank(
    supply: Option<IndustrialSupplyChainRole>,
    utility: Option<UtilityInfrastructureRole>,
) -> Option<u8> {
    if let Some(role) = supply {
        return match role {
            IndustrialSupplyChainRole::AluminumFabrication => Some(0),
            IndustrialSupplyChainRole::AluminumSmelter => Some(1),
            IndustrialSupplyChainRole::ConcreteMixer => Some(2),
            IndustrialSupplyChainRole::CementKiln => Some(3),
            IndustrialSupplyChainRole::AggregateMine => Some(4),
            IndustrialSupplyChainRole::BauxiteMine => Some(5),
            IndustrialSupplyChainRole::AluminaRefinery
            | IndustrialSupplyChainRole::IntegratedPlant => None,
        };
    }
    match utility {
        Some(UtilityInfrastructureRole::PowerPlant) => Some(6),
        Some(UtilityInfrastructureRole::Transformer)
        | Some(UtilityInfrastructureRole::Substation)
        | None => None,
    }
}

fn resolve_overload_shed_rank(
    registry: &BuildingDefinitionRegistry,
    planned: Option<&PlannedSite>,
    membership: Option<&IndustrialSupplyChainMembership>,
) -> Option<u8> {
    let catalog_id = planned
        .and_then(|site| site.catalog_id.as_deref())
        .filter(|id| !id.is_empty());
    let def = catalog_id.and_then(|id| registry.get(id));
    let supply = catalog_id.and_then(|id| {
        IndustrialSupplyChainRole::resolve(id, def.and_then(|row| row.supply_chain_role))
    });
    let utility = catalog_id.and_then(|id| {
        UtilityInfrastructureRole::resolve(id, def.and_then(|row| row.utility_role))
    });
    let supply = supply.or_else(|| {
        if utility.is_some() {
            None
        } else {
            membership.map(|row| row.role)
        }
    });
    overload_shed_rank(supply, utility)
}

fn shed_overloaded_member_loads(
    registry: &BuildingDefinitionRegistry,
    grid: &ElectricalGrid,
    loads: &mut Query<(
        Entity,
        &mut ElectricalComponent,
        Option<&PlannedSite>,
        Option<&IndustrialSupplyChainMembership>,
    )>,
) {
    let mut ranked: Vec<(u8, u32, Entity)> = Vec::new();
    for &member in &grid.members {
        let Ok((entity, electrical, planned, membership)) = loads.get(member) else {
            continue;
        };
        let Some(rank) = resolve_overload_shed_rank(registry, planned, membership) else {
            continue;
        };
        if !(electrical.current_load.is_finite() && electrical.current_load > 0.0) {
            continue;
        }
        ranked.push((rank, entity.index().index(), entity));
    }
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));

    let mut remaining = grid.total_load;
    for (_, _, entity) in ranked {
        if !(remaining > grid.total_capacity) {
            break;
        }
        let Ok((_, mut electrical, _, _)) = loads.get_mut(entity) else {
            continue;
        };
        let load = electrical.current_load;
        if !(load.is_finite() && load > 0.0) {
            continue;
        }
        electrical.current_load = 0.0;
        remaining -= load;
    }
}

fn update_power_output_system(
    defs: Res<PlantDefinitionRegistry>,
    mut query: Query<(&mut PowerPlant, Option<&ElectricalComponent>)>,
) {
    for (mut plant, electrical) in &mut query {
        let load_ratio = electrical
            .map(|e| {
                if e.max_transfer <= f32::EPSILON {
                    0.0
                } else {
                    (e.current_load / e.max_transfer).clamp(0.0, 1.0)
                }
            })
            .unwrap_or(0.0);

        let arch = PlantArchetype::for_type(plant.plant_type);
        let type_modifier = arch.efficiency_factor;
        let def_eff = if plant.definition_id.is_empty() {
            type_modifier
        } else if let Some(d) = defs.get(plant.definition_id.as_str()) {
            d.base_efficiency_factor() * d.status_efficiency_mult(plant.status)
        } else {
            type_modifier
        };
        let status_cap = if plant.definition_id.is_empty() {
            if plant.status == OperationalStatus::Operational {
                1.0
            } else {
                0.0
            }
        } else if let Some(d) = defs.get(plant.definition_id.as_str()) {
            d.status_output_fraction(plant.status)
        } else if plant.status == OperationalStatus::Operational {
            1.0
        } else {
            0.0
        };

        let effective =
            (1.0 - load_ratio * 0.5) * def_eff * plant.efficiency.clamp(0.0, 1.0) * status_cap;
        plant.current_output = (plant.max_output * effective).clamp(0.0, plant.max_output);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::construction::{BuildingDefinition, BuildingFamily, FootprintMatrix};
    use crate::infrastructure::{UtilityConnection, UtilityNetworkKind};
    use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype, SiteId};

    fn catalog_def(
        id: &str,
        supply: Option<IndustrialSupplyChainRole>,
        utility: Option<UtilityInfrastructureRole>,
    ) -> BuildingDefinition {
        BuildingDefinition {
            id: id.into(),
            display_name: id.into(),
            footprint: FootprintMatrix {
                width: 1,
                depth: 1,
                cells: vec![1],
            },
            construction_cost: 1,
            construction_time_ticks: 1,
            power_consumption: 1.0,
            power_generation: 0.0,
            workers_required: 0,
            site_archetype: SiteArchetype::Factory,
            family: BuildingFamily::Industry,
            produces: Vec::new(),
            consumes: Vec::new(),
            supply_chain: None,
            supply_chain_role: supply,
            concrete_type: None,
            utility_role: utility,
            plant_definition_id: None,
            transfer_capacity_mva: 0.0,
            is_productive: true,
            procedural_module_id: None,
            procedural_glb_path: None,
            procedural_glb_asset: None,
            grammar_archetype_id: None,
            arch_dna_preset: None,
            site_json_path: None,
            pilot_hover_hint: None,
            district_style: None,
        }
    }

    fn registry() -> BuildingDefinitionRegistry {
        let mut registry = BuildingDefinitionRegistry::default();
        let rows = [
            (
                "t_fab",
                Some(IndustrialSupplyChainRole::AluminumFabrication),
                None,
            ),
            (
                "t_smelter",
                Some(IndustrialSupplyChainRole::AluminumSmelter),
                None,
            ),
            (
                "t_mixer",
                Some(IndustrialSupplyChainRole::ConcreteMixer),
                None,
            ),
            ("t_kiln", Some(IndustrialSupplyChainRole::CementKiln), None),
            (
                "t_agg",
                Some(IndustrialSupplyChainRole::AggregateMine),
                None,
            ),
            (
                "t_bauxite",
                Some(IndustrialSupplyChainRole::BauxiteMine),
                None,
            ),
            ("t_plant", None, Some(UtilityInfrastructureRole::PowerPlant)),
            (
                "t_alumina",
                Some(IndustrialSupplyChainRole::AluminaRefinery),
                None,
            ),
            (
                "t_integrated",
                Some(IndustrialSupplyChainRole::IntegratedPlant),
                None,
            ),
            (
                "t_transformer",
                None,
                Some(UtilityInfrastructureRole::Transformer),
            ),
            (
                "t_substation",
                None,
                Some(UtilityInfrastructureRole::Substation),
            ),
        ];
        for (id, supply, utility) in rows {
            let def = catalog_def(id, supply, utility);
            registry.by_id.insert(id.into(), def);
        }
        registry
    }

    fn planned(site_id: u64, catalog_id: &str) -> PlannedSite {
        PlannedSite {
            site_id: SiteId(site_id),
            origin: BuildSiteTile { x: 0, z: 0 },
            footprint: FootprintTiles { width: 1, depth: 1 },
            archetype: SiteArchetype::Factory,
            layer: LayerType::Surface,
            catalog_id: Some(catalog_id.into()),
            placement: None,
        }
    }

    fn electrical(current: f32, max_transfer: f32) -> ElectricalComponent {
        ElectricalComponent {
            base_load: current,
            current_load: current,
            max_transfer,
            capacity: 1.0,
        }
    }

    fn spawn_load(
        app: &mut App,
        site_id: u64,
        catalog_id: &str,
        current: f32,
        max_transfer: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                electrical(current, max_transfer),
                planned(site_id, catalog_id),
            ))
            .id()
    }

    fn spawn_grid(
        app: &mut App,
        members: &[Entity],
        total_load: f32,
        total_capacity: f32,
    ) -> Entity {
        app.world_mut()
            .spawn(ElectricalGrid {
                members: members.iter().copied().collect::<HashSet<_>>(),
                total_load,
                total_capacity,
                ..Default::default()
            })
            .id()
    }

    fn current_load(app: &App, entity: Entity) -> f32 {
        app.world()
            .get::<ElectricalComponent>(entity)
            .expect("electrical")
            .current_load
    }

    #[test]
    fn overload_sheds_fabrication_before_mine() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(registry())
            .add_systems(Update, update_electrical_load_system);

        let fab_lo = spawn_load(&mut app, 1, "t_fab", 20.0, 100.0);
        let fab_hi = spawn_load(&mut app, 2, "t_fab", 40.0, 5.0);
        let smelter = spawn_load(&mut app, 3, "t_smelter", 20.0, 100.0);
        let mixer = spawn_load(&mut app, 4, "t_mixer", 20.0, 100.0);
        let kiln = spawn_load(&mut app, 5, "t_kiln", 20.0, 100.0);
        let aggregate = spawn_load(&mut app, 6, "t_agg", 20.0, 100.0);
        let bauxite = spawn_load(&mut app, 7, "t_bauxite", 20.0, 100.0);
        let plant = app
            .world_mut()
            .spawn((
                electrical(20.0, 100.0),
                planned(8, "t_plant"),
                UtilityConnection {
                    network_id: 1,
                    kind: UtilityNetworkKind::Power,
                    demand: 1.0,
                    connected: true,
                },
            ))
            .id();
        let alumina = spawn_load(&mut app, 9, "t_alumina", 8.0, 3.0);
        let integrated = spawn_load(&mut app, 10, "t_integrated", 8.0, 8.0);
        let transformer = spawn_load(&mut app, 11, "t_transformer", 8.0, 8.0);
        let substation = spawn_load(&mut app, 12, "t_substation", 8.0, 8.0);
        let order_members = [
            fab_lo,
            fab_hi,
            smelter,
            mixer,
            kiln,
            aggregate,
            bauxite,
            plant,
            alumina,
            integrated,
            transformer,
            substation,
        ];
        let order_load: f32 = order_members
            .iter()
            .map(|entity| current_load(&app, *entity))
            .sum();
        let order_grid = spawn_grid(&mut app, &order_members, order_load, 132.0);

        let tie_early = spawn_load(&mut app, 13, "t_fab", 25.0, 100.0);
        let tie_late = app
            .world_mut()
            .spawn((
                electrical(25.0, 100.0),
                IndustrialSupplyChainMembership {
                    chain_id: "aluminum_primary".into(),
                    role: IndustrialSupplyChainRole::AluminumFabrication,
                },
            ))
            .id();
        let tie_grid = spawn_grid(&mut app, &[tie_early, tie_late], 50.0, 30.0);

        let last_fab = spawn_load(&mut app, 14, "t_fab", 10.0, 100.0);
        let last_smelter = spawn_load(&mut app, 15, "t_smelter", 10.0, 100.0);
        let last_mixer = spawn_load(&mut app, 16, "t_mixer", 10.0, 100.0);
        let last_kiln = spawn_load(&mut app, 17, "t_kiln", 10.0, 100.0);
        let last_aggregate = spawn_load(&mut app, 18, "t_agg", 10.0, 100.0);
        let last_bauxite = spawn_load(&mut app, 19, "t_bauxite", 10.0, 100.0);
        let last_plant = spawn_load(&mut app, 20, "t_plant", 30.0, 100.0);
        let last_alumina = spawn_load(&mut app, 21, "t_alumina", 7.0, 7.0);
        let last_members = [
            last_fab,
            last_smelter,
            last_mixer,
            last_kiln,
            last_aggregate,
            last_bauxite,
            last_plant,
            last_alumina,
        ];
        let last_grid = spawn_grid(&mut app, &last_members, 97.0, 30.0);

        let idle = spawn_load(&mut app, 22, "t_fab", 12.0, 100.0);
        let idle_grid = spawn_grid(&mut app, &[idle], 12.0, 40.0);
        let zero_cap = spawn_load(&mut app, 23, "t_fab", 15.0, 100.0);
        let zero_grid = spawn_grid(&mut app, &[zero_cap], 15.0, 0.0);

        assert!(fab_hi.index() > fab_lo.index());
        assert!(tie_late.index() > tie_early.index());
        assert!(order_load > 132.0);

        app.update();

        assert_eq!(current_load(&app, fab_hi), 0.0);
        assert_eq!(current_load(&app, fab_lo), 0.0);
        assert_eq!(current_load(&app, smelter), 0.0);
        assert_eq!(current_load(&app, mixer), 20.0);
        assert_eq!(current_load(&app, kiln), 20.0);
        assert_eq!(current_load(&app, aggregate), 20.0);
        assert_eq!(current_load(&app, bauxite), 20.0);
        assert_eq!(current_load(&app, plant), 20.0);
        assert_eq!(current_load(&app, alumina), 3.0);
        assert_eq!(current_load(&app, integrated), 8.0);
        assert_eq!(current_load(&app, transformer), 8.0);
        assert_eq!(current_load(&app, substation), 8.0);
        assert!(
            app.world()
                .get::<UtilityConnection>(plant)
                .expect("connection")
                .connected
        );
        assert_eq!(
            app.world()
                .get::<ElectricalGrid>(order_grid)
                .expect("grid")
                .total_load,
            order_load
        );

        assert_eq!(current_load(&app, tie_late), 0.0);
        assert_eq!(current_load(&app, tie_early), 25.0);
        assert_eq!(
            app.world()
                .get::<ElectricalGrid>(tie_grid)
                .expect("tie grid")
                .total_load,
            50.0
        );

        assert_eq!(current_load(&app, last_fab), 0.0);
        assert_eq!(current_load(&app, last_smelter), 0.0);
        assert_eq!(current_load(&app, last_mixer), 0.0);
        assert_eq!(current_load(&app, last_kiln), 0.0);
        assert_eq!(current_load(&app, last_aggregate), 0.0);
        assert_eq!(current_load(&app, last_bauxite), 0.0);
        assert_eq!(current_load(&app, last_plant), 0.0);
        assert_eq!(current_load(&app, last_alumina), 7.0);
        assert_eq!(
            app.world()
                .get::<ElectricalGrid>(last_grid)
                .expect("last grid")
                .total_load,
            97.0
        );

        assert_eq!(current_load(&app, idle), 12.0);
        assert_eq!(
            app.world()
                .get::<ElectricalGrid>(idle_grid)
                .expect("idle grid")
                .total_load,
            12.0
        );
        assert_eq!(current_load(&app, zero_cap), 15.0);
        assert_eq!(
            app.world()
                .get::<ElectricalGrid>(zero_grid)
                .expect("zero grid")
                .total_capacity,
            0.0
        );
    }
}
