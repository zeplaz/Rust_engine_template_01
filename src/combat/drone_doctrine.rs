//! D6 — faction drone doctrine (promotes strategic stub into mission mix).

use bevy::prelude::*;

use crate::strategic::{doctrine_strike_weight, DroneDoctrine};

use super::drone_missions::DroneRole;

/// Per-faction doctrine selection (enqueue/UI later; default ReconHeavy).
#[derive(Resource, Clone, Debug)]
pub struct FactionDroneDoctrine {
    /// Index by faction slot (0..MAX).
    pub by_slot: [DroneDoctrine; 8],
}

impl Default for FactionDroneDoctrine {
    fn default() -> Self {
        Self {
            by_slot: [DroneDoctrine::ReconHeavy; 8],
        }
    }
}

impl FactionDroneDoctrine {
    #[must_use]
    pub fn get(&self, slot: u8) -> DroneDoctrine {
        self.by_slot
            .get(slot as usize)
            .copied()
            .unwrap_or(DroneDoctrine::ReconHeavy)
    }

    pub fn set(&mut self, slot: u8, d: DroneDoctrine) {
        if let Some(s) = self.by_slot.get_mut(slot as usize) {
            *s = d;
        }
    }
}

/// Preferred primary role for new AOs / sortie bias under doctrine.
#[must_use]
pub fn doctrine_preferred_role(d: DroneDoctrine) -> DroneRole {
    match d {
        DroneDoctrine::ReconHeavy => DroneRole::ReconLoiter,
        DroneDoctrine::SaturationStrike => DroneRole::SwarmMember,
        DroneDoctrine::EwSuppression => DroneRole::ReconLoiter,
        DroneDoctrine::LogisticsInterdiction => DroneRole::FpvStrike,
    }
}

/// Scale cruise/FPV effect strength by doctrine strike weight.
#[must_use]
pub fn doctrine_effect_scale(d: DroneDoctrine) -> f32 {
    doctrine_strike_weight(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_prefers_swarm_and_higher_strike() {
        assert_eq!(
            doctrine_preferred_role(DroneDoctrine::SaturationStrike),
            DroneRole::SwarmMember
        );
        assert!(
            doctrine_effect_scale(DroneDoctrine::SaturationStrike)
                > doctrine_effect_scale(DroneDoctrine::ReconHeavy)
        );
    }
}
