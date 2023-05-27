impl LoadBasedSpeedModifier for Truck {
    fn speed_modifier(&self) -> f32 {
        let load_ratio = self.current_load / self.capacity;
        // Assuming that a full truck moves at 50% of its max_speed
        0.7 + (1.0 - load_ratio) * 0.7
    }
}

impl LoadBasedSpeedModifier for Bus {
    fn speed_modifier(&self) -> f32 {
        let passenger_ratio = self.passengers as f32 / self.capacity as f32;
        0.8 + (1.0 - passenger_ratio) * 0.8
    }
}

/*

fn motion_calculations_system(
    mut query: Query<(&RoadVehicle, &dyn LoadBasedSpeedModifier, &mut Transform)>,
    time: Res<Time>,
) {
    for (road_vehicle, speed_modifier, mut transform) in query.iter_mut() {
        // Calculate the speed modifier based on the load or passengers
        let modifier = speed_modifier.speed_modifier();

        // Apply motion calculations based on the road_vehicle's properties,
        // such as velocity, max_speed, damage_info, and the modifier
        let modified_speed = road_vehicle.max_speed * modifier;
        road_vehicle.velocity = road_vehicle.velocity.normalize() * modified_speed;

        // Update the position based on the velocity and the time delta
        transform.translation += road_vehicle.velocity * time.delta_seconds();
    }
}
*/
