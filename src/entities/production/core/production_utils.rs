//! Resource taxonomy for HUD / logistics copy (`ResourceCategory`).
//! [`resource_category_of`] powers the site logistics panel resource rows.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::entities::types::p_enumz::ResourceType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceCategory {
    RawMaterial,
    ProcessedMaterial,
    Energy,
    Military,
    Human,
    Essentials,
}

impl ResourceCategory {
    #[inline]
    pub const fn as_tag(self) -> &'static str {
        match self {
            ResourceCategory::RawMaterial => "raw",
            ResourceCategory::ProcessedMaterial => "proc",
            ResourceCategory::Energy => "nrg",
            ResourceCategory::Military => "mil",
            ResourceCategory::Human => "hum",
            ResourceCategory::Essentials => "ess",
        }
    }
}

/// Static buckets for tooling / UI (not yet authoritative for routing).
pub fn categorize_resources() -> HashMap<ResourceCategory, Vec<ResourceType>> {
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
        vec![ResourceType::Energy, ResourceType::Fuel, ResourceType::Electricity],
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
        vec![ResourceType::Food, ResourceType::Water, ResourceType::Paper],
    );

    resource_categories
}

fn type_to_category_map() -> &'static HashMap<ResourceType, ResourceCategory> {
    static MAP: OnceLock<HashMap<ResourceType, ResourceCategory>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for (cat, types) in categorize_resources() {
            for ty in types {
                m.entry(ty).or_insert(cat);
            }
        }
        m
    })
}

#[inline]
pub fn resource_category_of(ty: ResourceType) -> ResourceCategory {
    *type_to_category_map()
        .get(&ty)
        .unwrap_or(&ResourceCategory::ProcessedMaterial)
}

#[inline]
pub fn resource_category_tag(ty: ResourceType) -> &'static str {
    resource_category_of(ty).as_tag()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wood_is_raw_not_essentials() {
        assert_eq!(resource_category_of(ResourceType::Wood), ResourceCategory::RawMaterial);
    }

    #[test]
    fn food_is_essentials() {
        assert_eq!(
            resource_category_of(ResourceType::Food),
            ResourceCategory::Essentials
        );
    }
}
