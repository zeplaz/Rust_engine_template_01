


//use crate::entities::buildings::Building;
//use crate::entities::road_vehicles::RoadVehicle;
use crate::traits::damage::*;
use bevy::prelude::*;

use crate::entities::damages::{DamageState, RoadVehicleDamageInfo, BuildingDamageInfo};

pub struct DamageSystem;




impl Plugin for DamageSystem {
    fn build(&self, app: &mut App) {
        app.add_system(apply_damage.system());
    }
}

fn apply_road_damage(mut query: Query<(&mut RoadVehicleDamageInfo, &mut App)>) {
    for (mut vehicle, mut damage_info) in query.iter_mut() {
        //do damage stuff
    }
}

impl DamageInfoProvider for RoadVehicleDamageInfo {
    type DamageInfo = BuildingDamageInfo;
    
    fn get_damage_info(&self) -> &Self::DamageInfo {
        &self
    }
}                   //{:È复}

impl DamageInfoProvider for BuildingDamageInfo {
    type DamageInfo = BuildingDamageInfo;

    fn get_damage_info(&self) -> &Self::DamageInfo {
        &self
    }
}