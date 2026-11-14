//! D5 — drone research / recipe gates (data only; UI later).
//!
//! Multipliers feed [`super::drone_missions::pulse_drone_sorties`] link/sensor resist and
//! unlock fiber / autonomy modalities. Production (D4b) reads recipe flags.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Research progress for UAS weather resist + recipe unlocks (sole writer: future tech UI / cheats).
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DroneResearchState {
    /// 0..1 added into link_quality resist.
    pub wx_resist_link: f32,
    /// 0..1 added into sensor_range_mul resist.
    pub wx_resist_sensor: f32,
    pub fiber_ops: bool,
    pub autonomy_l2: bool,
    pub autonomy_l3: bool,
    pub uas_assembly_yield: f32,
    pub seeker_ir_unlock: bool,
    pub cruise_line_unlock: bool,
}

impl Default for DroneResearchState {
    fn default() -> Self {
        Self {
            wx_resist_link: 0.0,
            wx_resist_sensor: 0.0,
            fiber_ops: false,
            autonomy_l2: false,
            autonomy_l3: false,
            uas_assembly_yield: 1.0,
            seeker_ir_unlock: false,
            cruise_line_unlock: false,
        }
    }
}

impl DroneResearchState {
    #[must_use]
    pub fn link_resist(&self, base: f32) -> f32 {
        (base + self.wx_resist_link).clamp(0.0, 0.85)
    }

    #[must_use]
    pub fn sensor_resist(&self, base: f32) -> f32 {
        (base + self.wx_resist_sensor).clamp(0.0, 0.85)
    }

    #[must_use]
    pub fn max_autonomy_level(&self) -> u8 {
        if self.autonomy_l3 {
            3
        } else if self.autonomy_l2 {
            2
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resist_clamps_and_autonomy_gates() {
        let mut r = DroneResearchState::default();
        assert_eq!(r.max_autonomy_level(), 1);
        r.autonomy_l2 = true;
        assert_eq!(r.max_autonomy_level(), 2);
        assert!((r.link_resist(0.9) - 0.85).abs() < 1e-3);
    }
}
