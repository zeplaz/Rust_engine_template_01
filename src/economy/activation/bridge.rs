//! Construction → operational industrial facility bridge (Priority 1).

use bevy::prelude::*;

use crate::construction::BuildingDefinitionRegistry;

use crate::economy::supply_chain::insert_supply_chain_runtime_for_catalog;
use crate::entities::production::core::manufacturing_node_for_catalog;
use crate::infrastructure::{
    initial_utility_power_connected, sync_utility_connection_power_system, UtilityConnection,
    UtilityNetworkKind,
};
use crate::strategic::{ConstructionSite, SiteConstructionPhase};

/// Catalog row chosen at placement — authoritative link to `assets/configs/buildings/*.json`.
#[derive(Component, Clone, Debug, Default)]
pub struct BuildingDefinitionRef {
    pub catalog_id: String,
}

/// Idempotent marker — facility bundle attached once.
#[derive(Component, Clone, Copy, Debug)]
pub struct IndustrialFacilityActivated;

pub struct IndustrialActivationPlugin;

impl Plugin for IndustrialActivationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crate::economy::resource_flow::ResourceFlowPlugin,
            crate::economy::spatial_district::SpatialDistrictPlugin,
            crate::economy::logistics::LogisticsThroughputPlugin,
            super::grid_overload_ux::GridOverloadUxPlugin,
            super::power_island_ux::PowerIslandUxPlugin,
        ));
        app.init_resource::<crate::strategic::StrategicCommandQueue>()
            .add_plugins(crate::strategic::Stage7BehavioralPlugin)
            .init_resource::<super::concrete_chain_e2e::ConcreteChainE2eStats>()
            .init_resource::<super::concrete_chain_e2e::Stage7PlayChainSeedState>()
            .init_resource::<super::concrete_chain_e2e::IndE02DefaultPlaySeedState>()
            .init_resource::<super::concrete_chain_e2e::IndE03GridOverloadSeedState>()
            .init_resource::<super::concrete_chain_e2e::RowhouseVictorianDemoSeedState>();
        app.add_systems(
            OnEnter(crate::engine::states::BaseState::Simulation),
            (
                super::concrete_chain_e2e::reset_stage7_play_chain_seed_on_enter_simulation,
                super::concrete_chain_e2e::reset_ind_e02_default_play_seed_on_enter_simulation,
                super::concrete_chain_e2e::reset_ind_e03_grid_overload_seed_on_enter_simulation,
                super::concrete_chain_e2e::reset_rowhouse_victorian_demo_seed_on_enter_simulation,
            ),
        );
        app.add_systems(
            Update,
            (
                super::concrete_chain_e2e::seed_ind_e02_default_play_once
                    .before(crate::strategic::commit_construction_site_system),
                super::concrete_chain_e2e::seed_rowhouse_victorian_production_demo_once
                    .after(crate::strategic::commit_construction_site_system)
                    .after(super::concrete_chain_e2e::ConcreteChainE2eSet::FastForwardPortland),
                super::concrete_chain_e2e::seed_stage7_play_concrete_chain_once,
                super::concrete_chain_e2e::fast_forward_portland_chain_sites_to_operational
                    .in_set(super::concrete_chain_e2e::ConcreteChainE2eSet::FastForwardPortland)
                    .after(crate::strategic::commit_construction_site_system),
                crate::economy::site_placement::ensure_site_world_transform_system,
                activate_industrial_facilities_system
                    .after(crate::strategic::site_provisioning_system),
                crate::economy::logistics::register_facility_portals_system,
                crate::economy::concrete_batch::register_concrete_batch_on_activation_system,
                sync_utility_connection_power_system,
            ),
        );
        app.add_systems(
            Update,
            crate::economy::concrete_batch::tick_concrete_batch_cure_system
                .run_if(crate::economy::resource_flow::economy_sim_running),
        );
        app.add_systems(
            Update,
            super::concrete_chain_e2e::seed_ind_e03_grid_overload_witness_once
                .after(crate::economy::resource_flow::collect_grid_overload_witness_system),
        );
    }
}



/// When a site reaches **Operational**, attach production/runtime components from its catalog def.
pub fn activate_industrial_facilities_system(
    mut commands: Commands,
    registry: Res<BuildingDefinitionRegistry>,
    q: Query<
        (Entity, &ConstructionSite, &BuildingDefinitionRef),
        (
            With<ConstructionSite>,
            With<BuildingDefinitionRef>,
            Without<IndustrialFacilityActivated>,
        ),
    >,
) {
    for (entity, site, def_ref) in &q {
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }
        let network_id = entity.to_bits();
        let connected = initial_utility_power_connected(registry.as_ref(), def_ref.catalog_id.as_str());
        let utility = UtilityConnection {
            network_id,
            kind: UtilityNetworkKind::Power,
            demand: 1.0,
            connected,
        };
        if def_ref.catalog_id.is_empty() || def_ref.catalog_id.starts_with("builtin:") {
            commands
                .entity(entity)
                .insert((IndustrialFacilityActivated, utility));
            continue;
        }
        insert_supply_chain_runtime_for_catalog(
            &mut commands,
            entity,
            def_ref.catalog_id.as_str(),
            registry.as_ref(),
        );
        // MFG-03 — attach ManufacturingNode so ManufacturingCorePlugin has live entities.
        let mut entity_cmds = commands.entity(entity);
        entity_cmds.insert((IndustrialFacilityActivated, utility));
        if let Some(mfg) = manufacturing_node_for_catalog(def_ref.catalog_id.as_str()) {
            entity_cmds.insert(mfg);
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct OverloadCount(u32);
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
    use crate::economy::resource_flow::{register_resource_flow_nodes_system, ResourceFlowNode};
    use crate::entities::production::aluminum::{
        AluminaRefineryRuntime, AluminumSmelterRuntime, BauxiteMineRuntime,
    };
    use crate::entities::production::concrete::{
        AggregateMineRuntime, CementKilnRuntime, ConcreteMixerRuntime,
    };
    use crate::entities::production::power::{
        ElectricalComponent, ElectricalGrid, GridConnectionRadiusSq, GridOverloadEvent, PowerPlant,
        PowerRuntimePlugin, SubstationComponent, TransformerComponent,
    };
    use crate::entities::production::power::grid_topology::{
        emit_grid_overload_signals, rebuild_electrical_grid_topology,
        recalculate_grid_totals_from_members,
    };
    use crate::entities::structure::components::Building;
    use crate::strategic::{ConstructionSite, SiteArchetype, SiteConstructionPhase};
    use crate::systems::sim_control::SimControlState;

    fn run_activation(app: &mut App, catalog_id: &str) -> Entity {
        let e = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    site_id: 1,
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase: SiteConstructionPhase::Operational,
                    operational_readiness: 1.0,
                },
                BuildingDefinitionRef {
                    catalog_id: catalog_id.into(),
                },
            ))
            .id();
        app.update();
        e
    }

    fn app_with_registry() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
        app.add_systems(
            Update,
            (
                activate_industrial_facilities_system,
                register_resource_flow_nodes_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn operational_smelter_gets_manufacturing_node() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "aluminum_smelter1");
        let node = app
            .world()
            .get::<crate::entities::production::core::ManufacturingNode>(e)
            .expect("ManufacturingNode on activate (MFG-03)");
        assert_eq!(node.blueprint_id, "mfg_aluminum_cast_v1");
    }

    #[test]
    fn operational_concrete_plant_gets_manufacturing_node() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "concrete_basic_production_plant");
        let node = app
            .world()
            .get::<crate::entities::production::core::ManufacturingNode>(e)
            .expect("ManufacturingNode on activate (MFG-03)");
        assert_eq!(node.blueprint_id, "mfg_concrete_batch_v1");
    }

    #[test]
    fn operational_integrated_concrete_plant_gets_kiln_and_mixer() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "concrete_basic_production_plant");
        assert!(app.world().get::<CementKilnRuntime>(e).is_some());
        assert!(app.world().get::<ConcreteMixerRuntime>(e).is_some());
        assert!(app.world().get::<ResourceFlowNode>(e).is_some());
    }

    #[test]
    fn operational_aggregate_mine_gets_mine_runtime_only() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "concrete_aggregate_mine");
        assert!(app.world().get::<AggregateMineRuntime>(e).is_some());
        assert!(app.world().get::<CementKilnRuntime>(e).is_none());
        assert!(app.world().get::<ResourceFlowNode>(e).is_some());
    }

    #[test]
    fn operational_aluminum_chain_steps_get_distinct_runtimes() {
        let mut app = app_with_registry();
        let mine = run_activation(&mut app, "aluminum_bauxite_mine");
        let refinery = run_activation(&mut app, "aluminum_alumina_refinery");
        let smelter = run_activation(&mut app, "aluminum_smelter1");
        assert!(app.world().get::<BauxiteMineRuntime>(mine).is_some());
        assert!(app.world().get::<AluminaRefineryRuntime>(refinery).is_some());
        assert!(app.world().get::<AluminumSmelterRuntime>(smelter).is_some());
        let smelter_load = app.world().get::<ElectricalComponent>(smelter).unwrap().base_load;
        let mine_load = app.world().get::<ElectricalComponent>(mine).unwrap().base_load;
        assert!(smelter_load > mine_load * 5.0);
    }

    #[test]
    fn operational_smelter_registers_flow_node_alumina_and_aluminum() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "aluminum_smelter1");
        assert!(app.world().get::<IndustrialFacilityActivated>(e).is_some());
        let node = app
            .world()
            .get::<ResourceFlowNode>(e)
            .expect("ResourceFlowNode registered on activate");
        assert!(
            node.consumption.iter().any(|r| r.tag == "Alumina"),
            "smelter must consume Alumina"
        );
        assert!(
            node.production.iter().any(|r| r.tag == "Aluminum"),
            "smelter must produce Aluminum"
        );
    }

    #[test]
    fn operational_transformer_gets_transformer_component() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "grid_distribution_transformer");
        assert!(app.world().get::<TransformerComponent>(e).is_some());
    }

    #[test]
    fn operational_substation_gets_substation_component() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "grid_substation");
        assert!(app.world().get::<SubstationComponent>(e).is_some());
    }

    #[test]
    fn operational_coal_plant_gets_power_plant_component() {
        let mut app = app_with_registry();
        let e = run_activation(&mut app, "utilities_coal_plant");
        let plant = app.world().get::<PowerPlant>(e).expect("power plant");
        assert_eq!(plant.definition_id, "coal_ultra_supercritical_650mw_v1");
        assert!(plant.max_output > 100.0);
    }

    #[test]
    fn transformer_bus_emits_overload_when_smelters_cluster() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, PowerRuntimePlugin));
        app.insert_resource(SimControlState::default());
        app.insert_resource(GridConnectionRadiusSq(32.0 * 32.0));

        let host = app
            .world_mut()
            .spawn((
                Transform::from_translation(Vec3::ZERO),
                GlobalTransform::default(),
                TransformerComponent {
                    input_voltage: 138_000.0,
                    output_voltage: 13_800.0,
                },
                ElectricalGrid::default(),
                ElectricalComponent {
                    base_load: 0.1,
                    current_load: 0.1,
                    max_transfer: 2.0,
                    capacity: 2.0,
                },
                Building {
                    building_type: crate::entities::types::s_flagz::BuildingType::Generic,
                },
            ))
            .id();

        for i in 0..4 {
            app.world_mut().spawn((
                Transform::from_translation(Vec3::new(i as f32 * 8.0, 0.0, 0.0)),
                GlobalTransform::default(),
                Building {
                    building_type: crate::entities::types::s_flagz::BuildingType::Generic,
                },
                ElectricalComponent {
                    base_load: 2.0,
                    current_load: 2.0,
                    max_transfer: 2.0,
                    capacity: 0.0,
                },
                crate::infrastructure::UtilityConnection::power(i as u64 + 1, 1.0, true),
            ));
        }

        fn count_overloads(
            mut reader: MessageReader<GridOverloadEvent>,
            mut count: ResMut<OverloadCount>,
        ) {
            for _ in reader.read() {
                count.0 += 1;
            }
        }

        app.init_resource::<OverloadCount>();
        app.add_systems(
            Update,
            (
                rebuild_electrical_grid_topology,
                recalculate_grid_totals_from_members,
                emit_grid_overload_signals,
                count_overloads,
            )
                .chain(),
        );

        app.update();
        assert!(
            app.world().resource::<OverloadCount>().0 > 0,
            "expected GridOverloadEvent when smelter cluster exceeds transformer capacity"
        );
        let grid = app.world().get::<ElectricalGrid>(host).expect("host grid");
        assert!(grid.total_load > grid.total_capacity);
    }
}
