/*
you call it like this:
fn spawn_entities_system(mut commands: Commands) {
    spawn_entity(&mut commands, EntityType::Building(BuildingType::House));
    spawn_entity(&mut commands, EntityType::RoadVehicle(RoadVehicleType::Bus));
    spawn_entity(&mut commands, EntityType::Tree);
}
*/
use crate::damage::RoadVehicleDamageInfo;
use crate::road_vehicle::RoadVehicleType;
use bevy::prelude::*;

#[derive(Debug)]
pub struct TruckMarker;

fn insert_material_and_light_affected(
    commands: &mut Commands,
    material_handle: Option<Handle<Texture>>,
    materials: &mut Assets<ColorMaterial>,
) -> &mut Commands {
    let material = materials.add(ColorMaterial {
        texture: material_handle.unwrap_or_default(),
        ..Default::default()
    });

    commands
        .insert_bundle(SpriteBundle {
            material,
            ..Default::default()
        })
        .insert(LightAffected)
}

pub struct RoadVehicleAdditionalData {
    mass: f32,
    max_speed: f32,
    damage_info: RoadVehicleDamageInfo,
    truck_capacity: Option<f32>,
    bus_capacity: Option<u32>,
}

pub struct EntityAdditionalData {
    entityinfo: EntityInfo,
    military_civilian: MilitaryCivilian,
    texture_handle: Option<Handle<Texture>>,
    light_datas: Option<Vec<LightData>>,

    road_vehicle_data: Option<RoadVehicleAdditionalData>,
    building_damage_info: Option<BuildingDamageInfo>,
}

fn spawn_entity(
    commands: &mut Commands,
    entity_type: EntityType,
    data: EntityAdditionalData,
    in_materials: Option<ResMut<Assets<ColorMaterial>>>,
) {
    match entity_type {
        EntityType::Building(building_type) => {
            let building = Building {
                entityinfo: EntityInfo {},
                military_civilian: data.military_civilian.unwrap_or(MilitaryCivilian::Civilian),
                damage_info: data.building_damage_info.unwrap_or_default(),
            };

            match building_type {
                BuildingType::House => {
                    commands.spawn().insert(building);
                    if let Some(materials) = in_materials {
                        insert_material_and_light_affected(
                            &mut commands,
                            data.texture_handle,
                            &mut materials,
                        );
                    }
                } // Add other building types here
            }
        }
        EntityType::Tree => {
            // Initialize and spawn a Tree entity
        }
        EntityType::Tile => {
            // Initialize and spawn a Tile entity
        }
        EntityType::RoadVehicle(road_vehicle_type) => {
            let road_vehicle = RoadVehicle {};

            match road_vehicle_type {
                RoadVehicleType::Bus => {
                    commands.spawn().insert(bus);
                    insert_material_and_light_affected(
                        &mut commands,
                        data.texture_handle,
                        &mut materials,
                    );
                }
                RoadVehicleType::Car => {}
                RoadVehicleType::Truck => {} // Add other road vehicle types here
            }
        }
        EntityType::Train => {
            // Initialize and spawn a Train entity
        }
        EntityType::Rail => {
            // Initialize and spawn a Rail entity
        }
        EntityType::Road => {
            // Initialize and spawn a Road entity
        }

        EntityType::Light => {
            if let Some(light_datas) = data.light_datas {
                for light_data in light_datas {
                    commands.spawn().insert(light_data).insert(LightAffected);
                }
            }
        }
    }
}
