//! Load-aware speed modifiers for heavy vehicles (kinematics split per `navigation/implementation_questions_v1.md` §11).
//! A future ECS system can combine this with `SpeedModifier` after component graphs are clarified (no `dyn` in `Query`).

use crate::entities::vehicles::runtime::{Bus, Truck};
use crate::traits::LoadBasedSpeedModifier;

impl LoadBasedSpeedModifier for Truck {
    fn speed_modifier(&self) -> f32 {
        let cap = self.vehicle.capacity.max(1) as f32;
        let load_ratio = self.current_load / cap;
        // Heavier load → lower modifier floor toward 0.7× baseline curve in original sketch.
        0.7 + (1.0 - load_ratio) * 0.7
    }
}

impl LoadBasedSpeedModifier for Bus {
    fn speed_modifier(&self) -> f32 {
        let cap = self.capacity.max(1) as f32;
        let passenger_ratio = self.passengers as f32 / cap;
        0.8 + (1.0 - passenger_ratio) * 0.8
    }
}
