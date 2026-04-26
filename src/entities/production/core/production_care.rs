// src/production/core/production_core.rs
use super::building_core::Building;
use super::resources::{ResourceStorage, ResourceType};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource, Debug, Clone)]
pub struct ProductionSettings {
    pub transport_range: f32,
    pub base_production_rate: f32,
    pub storage_deterioration_rate: f32,
}

impl Default for ProductionSettings {
    fn default() -> Self {
        Self {
            transport_range: 1000.0,
            base_production_rate: 5.0,
            storage_deterioration_rate: 0.001,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ResourceProducer {
    pub resource_type: ResourceType,
    pub production_rate: f32,
    pub max_production_rate: f32,
    pub energy_consumption: f32,
    pub efficiency: f32,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ResourceConsumer {
    pub resource_types: Vec<ResourceType>,
    pub consumption_rates: HashMap<ResourceType, f32>,
    pub required_amounts: HashMap<ResourceType, f32>,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ProductionChain {
    pub input_buildings: Vec<Entity>,
    pub output_buildings: Vec<Entity>,
    pub resource_flow: HashMap<(Entity, Entity), Vec<ResourceType>>,
    pub automated_transport: bool,
}

pub fn update_resource_storage(
    mut storage_query: Query<&mut ResourceStorage>,
    time: Res<Time>,
    settings: Res<ProductionSettings>,
) {
    for mut storage in storage_query.iter_mut() {
        // Apply deterioration to perishable resources
        for (resource_type, amount) in storage.resources.iter_mut() {
            if resource_type.is_perishable() {
                *amount -= *amount * settings.storage_deterioration_rate * time.delta_seconds();
                if *amount < 0.001 {
                    *amount = 0.0;
                }
            }
        }
    }
}

pub fn connect_production_chains(
    mut commands: Commands,
    buildings: Query<(Entity, &Transform, &Building)>,
    producers: Query<(Entity, &ResourceProducer)>,
    consumers: Query<(Entity, &ResourceConsumer)>,
    settings: Res<ProductionSettings>,
) {
    // Find potential connections between producers and consumers
    for (producer_entity, producer, producer_transform) in producers.iter().map(|(e, p)| {
        let transform = buildings
            .get(e)
            .map(|(_, t, _)| t)
            .unwrap_or(&Transform::default());
        (e, p, transform)
    }) {
        for (consumer_entity, consumer, consumer_transform) in consumers.iter().map(|(e, c)| {
            let transform = buildings
                .get(e)
                .map(|(_, t, _)| t)
                .unwrap_or(&Transform::default());
            (e, c, transform)
        }) {
            // Skip self-connections
            if producer_entity == consumer_entity {
                continue;
            }

            // Check if consumer needs what producer makes
            if consumer.resource_types.contains(&producer.resource_type) {
                // Check if within transport range
                let distance = producer_transform
                    .translation
                    .distance(consumer_transform.translation);
                if distance <= settings.transport_range {
                    // Create or update production chain
                    // This is simplified - in a real implementation you'd check for existing chains
                    commands.spawn(ProductionChain {
                        input_buildings: vec![producer_entity],
                        output_buildings: vec![consumer_entity],
                        resource_flow: HashMap::from([(
                            (producer_entity, consumer_entity),
                            vec![producer.resource_type],
                        )]),
                        automated_transport: true,
                    });
                }
            }
        }
    }
}

pub fn update_building_status(
    mut buildings: Query<&mut Building>,
    // Other queries needed for status updates
) {
    // Implementation details
}
