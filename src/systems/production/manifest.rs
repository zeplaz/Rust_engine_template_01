use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct ProductionDomainEntry {
    pub id: String,
    pub runtime_plugin: String,
    pub tools_plugin: Option<String>,
    pub serializable_types: Vec<String>,
    pub ecs_runtime_types: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ProductionManifest {
    pub domains: Vec<ProductionDomainEntry>,
}

impl ProductionManifest {
    pub fn find_domain(&self, id: &str) -> Option<&ProductionDomainEntry> {
        self.domains.iter().find(|d| d.id == id)
    }
}

pub fn default_production_manifest() -> ProductionManifest {
    ProductionManifest {
        domains: vec![
            ProductionDomainEntry {
                id: "concrete".to_string(),
                runtime_plugin: "ConcreteRuntimePlugin".to_string(),
                tools_plugin: Some("ProductionToolsUiPlugin".to_string()),
                serializable_types: vec!["ConcreteProductionConfig".to_string(), "ConcreteType".to_string()],
                ecs_runtime_types: vec![
                    "CementKilnRuntime".to_string(),
                    "AggregateMineRuntime".to_string(),
                    "ConcreteMixerRuntime".to_string(),
                ],
                notes: vec![
                    "Legacy sys.rs hard-disabled from module graph".to_string(),
                    "Use config resource for deterministic tuning".to_string(),
                    "Serialization: ConcreteSerializationPlugin (systems/production/serialization.rs)".to_string(),
                ],
            },
            ProductionDomainEntry {
                id: "aluminum".to_string(),
                runtime_plugin: "AluminumRuntimePlugin".to_string(),
                tools_plugin: Some("ProductionToolsUiPlugin".to_string()),
                serializable_types: vec![
                    "AluminumProductionConfig".to_string(),
                    "FabricationLineType".to_string(),
                ],
                ecs_runtime_types: vec![
                    "BauxiteMineRuntime".to_string(),
                    "AluminaRefineryRuntime".to_string(),
                    "AluminumSmelterRuntime".to_string(),
                    "AluminumFabricationPlantRuntime".to_string(),
                ],
                notes: vec![
                    "Legacy production_sys.rs hard-disabled from module graph".to_string(),
                    "Prepared for additional manufacturing domains".to_string(),
                    "Serialization: AluminumSerializationPlugin".to_string(),
                ],
            },
            ProductionDomainEntry {
                id: "power".to_string(),
                runtime_plugin: "PowerRuntimePlugin".to_string(),
                tools_plugin: Some("ProductionToolsUiPlugin".to_string()),
                serializable_types: vec!["PowerPlantType".to_string(), "PowerDistributionType".to_string()],
                ecs_runtime_types: vec!["PowerPlant".to_string(), "ElectricalComponent".to_string()],
                notes: vec![
                    "Power domain: entities/production/power/ — PlantArchetype, grid_topology, capability markers, PowerRuntimePlugin".to_string(),
                    "Runtime load/output systems now isolated".to_string(),
                    "Serialization: PowerSerializationPlugin".to_string(),
                ],
            },
            ProductionDomainEntry {
                id: "manufacturing_core".to_string(),
                runtime_plugin: "ProductionRuntimePlugin".to_string(),
                tools_plugin: Some("ProductionToolsUiPlugin".to_string()),
                serializable_types: vec!["ManufacturingBlueprint".to_string(), "ManufacturingDomain".to_string()],
                ecs_runtime_types: vec!["ManufacturingNode".to_string()],
                notes: vec![
                    "Domain-agnostic super-structure for future expansion".to_string(),
                ],
            },
        ],
    }
}
