use crate::entities::buildings::Building;
use crate::entities::road_vehicles::RoadVehicle;
use crate::traits::damage::*;
use bevy::prelude::*;

use damage_state::{DamageState, RoadVehicleDamageInfo};

pub struct RoadVehicleDamageSystem;

impl Plugin for RoadVehicleDamageSystem {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(apply_damage.system());
    }
}

fn apply_damage(mut query: Query<(&mut RoadVehicle, &mut RoadVehicleDamageInfo)>) {
    for (mut vehicle, mut damage_info) in query.iter_mut() {
        //do damage stuff
    }
}
