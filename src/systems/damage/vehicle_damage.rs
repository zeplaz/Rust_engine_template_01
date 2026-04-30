use crate::entities::damages as entity_damages;
use crate::entities::vehicles::runtime::RoadVehicle;
use crate::traits::damage::{DamageInfoProvider, TakesDamage};

use entity_damages::{DamageState, RoadVehicleDamageInfo};

impl DamageInfoProvider for RoadVehicle {
    type DamageInfo = RoadVehicleDamageInfo;

    fn get_damage_info(&self) -> &Self::DamageInfo {
        &self.damage_info
    }
}

impl TakesDamage for RoadVehicle {
    fn apply_damage(&mut self, amount: f32) {
        self.damage_info.structural_integrity =
            (self.damage_info.structural_integrity - amount * 0.1).max(0.0);
    }

    fn repair(&mut self, amount: f32) {
        self.damage_info.structural_integrity =
            (self.damage_info.structural_integrity + amount).min(1.0);
    }

    fn get_structural_integrity(&self) -> f32 {
        self.damage_info.structural_integrity
    }

    fn damage_state(&self) -> &DamageState {
        &self.damage_info.state
    }
}
