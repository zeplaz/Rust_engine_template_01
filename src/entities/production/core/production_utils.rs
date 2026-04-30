// Staging helper for resource taxonomy; not yet wired into runtime systems.
use std::collections::HashMap;

use crate::entities::types::p_enumz::ResourceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceCategory {
    RawMaterial,
    ProcessedMaterial,
    Energy,
    Military,
    Human,
    Essentials,
}

fn categorize_resources() -> HashMap<ResourceCategory, Vec<ResourceType>> {
    let mut resource_categories: HashMap<ResourceCategory, Vec<ResourceType>> = HashMap::default();

    resource_categories.insert(
        ResourceCategory::RawMaterial,
        vec![
            ResourceType::Wood,
            ResourceType::Coal,
            ResourceType::Oil,
            ResourceType::RareEarth,
            ResourceType::Metal,
        ],
    );

    resource_categories.insert(
        ResourceCategory::ProcessedMaterial,
        vec![
            ResourceType::Steel,
            ResourceType::Concrete,
            ResourceType::Fertilizer,
            ResourceType::Chemicals,
            ResourceType::Electronics,
        ],
    );

    resource_categories.insert(
        ResourceCategory::Energy,
        vec![ResourceType::Energy, ResourceType::Fuel],
    );

    resource_categories.insert(
        ResourceCategory::Military,
        vec![ResourceType::Ammunition, ResourceType::WarSupply],
    );

    resource_categories.insert(
        ResourceCategory::Human,
        vec![ResourceType::Knowledge, ResourceType::Labour],
    );

    resource_categories.insert(
        ResourceCategory::Essentials,
        vec![
            ResourceType::Wood,
            ResourceType::Food,
            ResourceType::Water,
            ResourceType::Paper,
        ],
    );

    resource_categories
}
