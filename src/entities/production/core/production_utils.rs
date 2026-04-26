use bevy::prelude::*;

pub mod prelude {
    pub use crate::entities::prelude::*;
}
pub use crate::entities::prelude::*;


fn categorize_resources() -> HashMap<ResourceCategory, Vec<ResourceType>> {
    let mut resource_categories: HashMap<ResourceCategory, Vec<ResourceType>> = HashMap::new();

    resource_categories.insert(ResourceCategory::RawMaterial, vec![
        ResourceType::Wood,
        ResourceType::Coal,
        ResourceType::Oil,
        ResourceType::RareEarth,
        ResourceType::Metal,
    ]);

    resource_categories.insert(ResourceCategory::ProcessedMaterial, vec![
        ResourceType::Steel,
        ResourceType::Concrete(ConcreteType::SomeConcreteType),  // replace with actual ConcreteType
        ResourceType::Fertilizer,
        ResourceType::Chemicals,
        ResourceType::Electronics,
       
    ]);

    resource_categories.insert(ResourceCategory::Energy, vec![
        ResourceType::Energy,
        ResourceType::Fuel,
    ]);

    resource_categories.insert(ResourceCategory::Military, vec![
        ResourceType::Ammunition,
        ResourceType::WarSupply,
    ]);

    resource_categories.insert(ResourceCategory::Human, vec![
        ResourceType::Knowledge,
        ResourceType::Labour
    ]);

    resource_categories.insert(ResourceCategory::Essentials, vec![
        ResourceType::Wood,
        ResourceType::Food,
        ResourceType::Water,
        ResourceType::Paper,
    ]);

    resource_categories
}