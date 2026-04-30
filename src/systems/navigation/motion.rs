//! Kinematic scale factors for navigation (split from graph/field per `navigation/implementation_questions_v1.md` §11).
use bevy::prelude::*;

/// Scalar applied to displacement from potential-field / steering systems each tick.
#[derive(Component, Debug, Clone, Copy)]
pub struct SpeedModifier {
    pub value: f32,
}

impl Default for SpeedModifier {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

impl SpeedModifier {
    #[inline]
    pub fn calculate_speed_modifier(&self) -> f32 {
        self.value
    }
}
