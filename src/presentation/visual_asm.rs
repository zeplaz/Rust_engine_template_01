//! Sprite / tile / vehicle / building **action state machine** hooks (Ranking C2).
//!
//! Extends [`crate::construction::procedural::tile_visual_state::VisualState`] with
//! named phases — atlas frame selection stays presentation-side.

use bevy::prelude::*;

/// Visual phase for buildings / trucks / near-band air (not skeletal animation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum VisualActionPhase {
    #[default]
    Idle,
    Work,
    Move,
    Damaged,
    Destroyed,
    Launch,
    Alert,
}

impl VisualActionPhase {
    /// Suggested atlas animation frame base (catalogs may override).
    #[must_use]
    pub fn default_frame(self) -> u8 {
        match self {
            VisualActionPhase::Idle => 0,
            VisualActionPhase::Work => 1,
            VisualActionPhase::Move => 2,
            VisualActionPhase::Damaged => 3,
            VisualActionPhase::Destroyed => 4,
            VisualActionPhase::Launch => 5,
            VisualActionPhase::Alert => 6,
        }
    }
}

/// Presentation-only ASM component for stamped entities (trucks, plants, missiles near-band).
#[derive(Component, Clone, Debug, Default)]
pub struct VisualActionState {
    pub phase: VisualActionPhase,
    /// Local phase clock (presentation dt); not sim tick authority.
    pub phase_age_secs: f32,
    pub facing: u8,
}

impl VisualActionState {
    #[must_use]
    pub fn frame(&self) -> u8 {
        self.phase.default_frame()
    }

    pub fn set_phase(&mut self, phase: VisualActionPhase) {
        if self.phase != phase {
            self.phase = phase;
            self.phase_age_secs = 0.0;
        }
    }
}

/// Advance presentation phase age (call from view/render cadence, not sim writers).
pub fn tick_visual_action_age(time: Res<Time>, mut q: Query<&mut VisualActionState>) {
    let dt = time.delta_secs();
    for mut st in &mut q {
        st.phase_age_secs += dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_change_resets_age() {
        let mut s = VisualActionState {
            phase: VisualActionPhase::Idle,
            phase_age_secs: 3.0,
            facing: 0,
        };
        s.set_phase(VisualActionPhase::Work);
        assert_eq!(s.phase, VisualActionPhase::Work);
        assert_eq!(s.phase_age_secs, 0.0);
        assert_eq!(s.frame(), 1);
    }
}
