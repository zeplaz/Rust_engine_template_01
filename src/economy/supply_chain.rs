//! Supply-chain activation — attach per-step runtime components from catalog defs.

use bevy::prelude::*;

use crate::construction::{
    BuildingDefinition, BuildingDefinitionRegistry, IndustrialSupplyChainRole,
    UtilityInfrastructureRole,
};
use crate::entities::production::aluminum::{
    AluminaRefineryRuntime, AluminumFabricationPlantRuntime, AluminumSmelterRuntime,
    BauxiteMineRuntime, FabricationLineType,
};
use crate::entities::production::concrete::{
    AggregateMineRuntime, CementKilnRuntime, ConcreteMixerRuntime,
};
use crate::entities::production::power::{
    ElectricalComponent, ElectricalGrid, PowerPlant, SubstationComponent, ThermalComponent,
    TransformerComponent,
};
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::production::power::power_states::PowerDistributionType;
use crate::entities::structure::components::Building;
use crate::entities::types::OperationalStatus;
use crate::entities::types::s_flagz::{BuildingType, ConcreteType, FactoryType};

/// Links operational site to its chain for future resource-flow graph (I2).
#[derive(Component, Clone, Debug)]
pub struct IndustrialSupplyChainMembership {
    pub chain_id: String,
    pub role: IndustrialSupplyChainRole,
}

#[must_use]
pub fn electrical_from_power_units(power_consumption: f32) -> ElectricalComponent {
    let load = (power_consumption / 100.0).max(0.01);
    ElectricalComponent {
        base_load: load,
        current_load: load,
        max_transfer: load * 1.25,
        capacity: load * 0.5,
    }
}

fn building_type_for_def(def: &BuildingDefinition) -> BuildingType {
    if let Some(ct) = def.concrete_type {
        return BuildingType::FactoryType(FactoryType::ConcreteType(ct));
    }
    let id = def.id.to_lowercase();
    if id.contains("concrete") {
        BuildingType::FactoryType(FactoryType::ConcreteType(ConcreteType::Portland))
    } else if id.contains("aluminum") || id.contains("smelter") || id.contains("bauxite") {
        BuildingType::FactoryType(FactoryType::MetalProcessing)
    } else {
        BuildingType::Generic
    }
}

fn insert_utility_runtime(commands: &mut Commands, entity: Entity, def: &BuildingDefinition) {
    let utility = UtilityInfrastructureRole::resolve(def.id.as_str(), def.utility_role)
        .expect("utility_role required");
    let building = Building {
        building_type: BuildingType::Generic,
    };
    let mva = def.transfer_capacity_mva.max(1.0);
    let electrical = ElectricalComponent {
        base_load: (def.power_consumption / 100.0).max(0.01),
        current_load: (def.power_consumption / 100.0).max(0.01),
        max_transfer: mva,
        capacity: mva * 0.85,
    };

    match utility {
        UtilityInfrastructureRole::Transformer => {
            commands.entity(entity).insert((
                TransformerComponent {
                    input_voltage: 138_000.0,
                    output_voltage: 13_800.0,
                },
                ThermalComponent {
                    current_temperature: 25.0,
                    max_temperature: 120.0,
                },
                ElectricalGrid::default(),
                building,
                electrical,
            ));
        }
        UtilityInfrastructureRole::Substation => {
            let mut input = std::collections::HashMap::new();
            let mut output = std::collections::HashMap::new();
            input.insert(PowerDistributionType::ThreePhaseHeavyIndustrial, 138_000.0);
            output.insert(PowerDistributionType::ThreePhaseMediumIndustrial, 13_800.0);
            output.insert(PowerDistributionType::ThreePhaseHeavyIndustrial, 69_000.0);
            commands.entity(entity).insert((
                SubstationComponent {
                    input_voltage: input,
                    output_voltages: output,
                },
                ThermalComponent {
                    current_temperature: 25.0,
                    max_temperature: 150.0,
                },
                ElectricalGrid::default(),
                building,
                electrical,
            ));
        }
        UtilityInfrastructureRole::PowerPlant => {
            let plant_defs = PlantDefinitionRegistry::from_embedded_json();
            let def_id = def
                .plant_definition_id
                .as_deref()
                .unwrap_or("coal_ultra_supercritical_650mw_v1");
            let row = plant_defs.get(def_id);
            let plant_type = row
                .map(|p| p.plant_type)
                .unwrap_or(crate::entities::production::power::power_states::PowerPlantType::Coal);
            let nameplate = row
                .map(|p| p.output_model.nameplate_mw)
                .unwrap_or(def.power_generation.max(100.0));
            commands.entity(entity).insert((
                PowerPlant {
                    definition_id: def_id.to_string(),
                    plant_type,
                    max_output: nameplate,
                    current_output: 0.0,
                    status: OperationalStatus::Standby,
                    efficiency: row
                        .and_then(|p| {
                            p.operational
                                .status_modifiers
                                .get(&OperationalStatus::Operational)
                        })
                        .map(|m| m.efficiency_multiplier)
                        .unwrap_or(1.0),
                },
                building,
                electrical,
            ));
        }
    }
}

/// Attach runtime components for this catalog row's supply-chain role.
pub fn insert_supply_chain_runtime(commands: &mut Commands, entity: Entity, def: &BuildingDefinition) {
    if UtilityInfrastructureRole::resolve(def.id.as_str(), def.utility_role).is_some() {
        insert_utility_runtime(commands, entity, def);
        return;
    }

    let power = def.power_consumption;
    let role = match IndustrialSupplyChainRole::resolve(def.id.as_str(), def.supply_chain_role) {
        Some(r) => r,
        None => {
            if power > 0.0 {
                commands.entity(entity).insert((
                    Building {
                        building_type: building_type_for_def(def),
                    },
                    electrical_from_power_units(power),
                ));
            }
            return;
        }
    };

    if let Some(chain) = def.supply_chain.as_ref() {
        commands.entity(entity).insert(IndustrialSupplyChainMembership {
            chain_id: chain.clone(),
            role,
        });
    }

    let building = Building {
        building_type: building_type_for_def(def),
    };
    let electrical = electrical_from_power_units(power);

    match role {
        IndustrialSupplyChainRole::AggregateMine => {
            commands.entity(entity).insert((
                AggregateMineRuntime {
                    deposit_quality: 0.75,
                    extraction_rate: 1.0,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::CementKiln => {
            commands.entity(entity).insert((
                CementKilnRuntime {
                    temperature: 900.0,
                    capacity: 100.0,
                    efficiency: 0.85,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::ConcreteMixer => {
            commands.entity(entity).insert((
                ConcreteMixerRuntime {
                    capacity: 80.0,
                    mixing_efficiency: 0.9,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::IntegratedPlant => {
            commands.entity(entity).insert((
                CementKilnRuntime {
                    temperature: 900.0,
                    capacity: 100.0,
                    efficiency: 0.85,
                },
                ConcreteMixerRuntime {
                    capacity: 80.0,
                    mixing_efficiency: 0.9,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::BauxiteMine => {
            commands.entity(entity).insert((
                BauxiteMineRuntime {
                    ore_richness: 0.7,
                    mine_depth: 10.0,
                    max_depth: 200.0,
                    depletion_rate: 0.02,
                    environmental_impact: 0.15,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::AluminaRefinery => {
            commands.entity(entity).insert((
                AluminaRefineryRuntime {
                    digestion_temperature: 140.0,
                    pressure: 5.0,
                    red_mud_storage: 0.0,
                    max_red_mud_storage: 1000.0,
                    caustic_soda_efficiency: 0.82,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::AluminumSmelter => {
            commands.entity(entity).insert((
                AluminumSmelterRuntime {
                    pot_count: 24,
                    current_efficiency: 0.88,
                    anode_degradation: 0.1,
                    cryolite_level: 1.0,
                    fluoride_emissions: 0.0,
                },
                building,
                electrical,
            ));
        }
        IndustrialSupplyChainRole::AluminumFabrication => {
            commands.entity(entity).insert((
                AluminumFabricationPlantRuntime {
                    production_line_type: FabricationLineType::Extrusion,
                    alloy_mixing_capacity: 50.0,
                    product_quality: 0.92,
                    scrap_rate: 0.05,
                },
                building,
                electrical,
            ));
        }
    }
}

/// Activate by catalog id using the building registry (authoritative power + role metadata).
pub fn insert_supply_chain_runtime_for_catalog(
    commands: &mut Commands,
    entity: Entity,
    catalog_id: &str,
    registry: &BuildingDefinitionRegistry,
) {
    if let Some(def) = registry.get(catalog_id) {
        insert_supply_chain_runtime(commands, entity, def);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
    use crate::construction::IndustrialSupplyChainRole;

    #[test]
    fn supply_chain_catalog_covers_concrete_and_aluminum_steps() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        for id in [
            "concrete_aggregate_mine",
            "concrete_cement_kiln",
            "concrete_mixer_plant",
            "aluminum_bauxite_mine",
            "aluminum_alumina_refinery",
            "aluminum_smelter1",
            "aluminum_fabrication_plant",
        ] {
            let d = reg.get(id).expect(id);
            assert!(
                IndustrialSupplyChainRole::resolve(d.id.as_str(), d.supply_chain_role).is_some(),
                "{id} should resolve a role"
            );
            assert!(d.power_consumption > 0.0, "{id} needs power_consumption");
        }
    }

    #[test]
    fn geopolymer_mixer_has_geopolymer_concrete_type() {
        let reg = load_building_definitions_from_dir(default_buildings_dir());
        let d = reg.get("concrete_mixer_geopolymer").expect("def");
        assert_eq!(d.concrete_type, Some(ConcreteType::Geopolymer));
        assert_eq!(
            IndustrialSupplyChainRole::resolve(d.id.as_str(), d.supply_chain_role),
            Some(IndustrialSupplyChainRole::ConcreteMixer)
        );
    }
}
