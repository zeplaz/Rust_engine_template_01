use crate::traits::damage::{DamageInfoProvider, *};
use bevy::prelude::*;

use crate::entities::damages as entity_damages;

use entity_damages::{BuildingDamageInfo, RoadVehicleDamageInfo};

pub struct DamageSystem;

impl Plugin for DamageSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_road_damage);
    }
}

fn apply_road_damage(mut query: Query<&mut RoadVehicleDamageInfo>) {
    for mut damage_info in query.iter_mut() {
        // TODO: apply damage accumulation logic here
        let _ = &mut *damage_info;
    }
}

impl DamageInfoProvider for RoadVehicleDamageInfo {
    type DamageInfo = RoadVehicleDamageInfo;

    fn get_damage_info(&self) -> &Self::DamageInfo {
        self
    }
}

impl DamageInfoProvider for BuildingDamageInfo {
    type DamageInfo = BuildingDamageInfo;

    fn get_damage_info(&self) -> &Self::DamageInfo {
        self
    }
}
