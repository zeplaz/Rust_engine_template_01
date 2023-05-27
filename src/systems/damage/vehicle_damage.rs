impl DamageInfoProvider for RoadVehicle {
    type DamageInfo = RoadVehicleDamageInfo;

    fn get_damage_info(&self) -> &Self::DamageInfo {
        &self.damage_info
    }
}

// other f

impl TakesDamage for RoadVehicle {
    fn apply_damage(&mut self, amount: f32) {
        // logic to apply damage to the vehicle
    }

    fn repair(&mut self, amount: f32) {
        // logic to repair the vehicle
    }

    fn get_structural_integrity(&self) -> f32 {
        self.damage_info.structural_integrity
    }

    fn damage_state(&self) -> &DamageState {
        &self.damage_info.state
    }

    fn set_damage_state(&mut self, state: DamageState) {
        self.damage_state = state;
    }
}
