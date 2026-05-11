//! **Pressure composition** — designer/mission “cards” bias a shared climate ([`PressureField`]);
//! they do **not** execute quest logic or force outcomes.
//!
//! Tooling stack: World → factions → agents → missions → pressure fields (see `strategic` module docs).

use bevy::prelude::*;

/// One layer of additive designer input (sliders / mission profile). Clamped when merged into [`PressureField`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PressureProfile {
    pub paranoia: f32,
    pub aggression: f32,
    pub instability: f32,
    /// Extra downward stress on faction cohesion (not a split guarantee).
    pub cohesion_drift: f32,
}

/// Global scalar field — **read** by sim systems and **written** by tooling + active missions.
#[derive(Resource, Clone, Debug)]
pub struct PressureField {
    pub paranoia: f32,
    pub aggression: f32,
    pub instability: f32,
    pub cohesion_drift: f32,
}

impl Default for PressureField {
    fn default() -> Self {
        Self {
            paranoia: 0.0,
            aggression: 0.0,
            instability: 0.0,
            cohesion_drift: 0.0,
        }
    }
}

impl PressureField {
    #[inline]
    fn clamp_channel(x: f32) -> f32 {
        x.clamp(0.0, 1.0)
    }

    /// Relax toward clear air (no active cards); `rate` ∈ (0, 1] fraction removed per step.
    pub fn relax(&mut self, rate: f32) {
        let r = rate.clamp(0.0, 1.0);
        self.paranoia *= 1.0 - r;
        self.aggression *= 1.0 - r;
        self.instability *= 1.0 - r;
        self.cohesion_drift *= 1.0 - r;
    }

    /// Add a profile scaled by `scale` (mission priority, tool strength), then clamp channels.
    pub fn accumulate(&mut self, p: &PressureProfile, scale: f32) {
        let s = scale.max(0.0);
        self.paranoia = Self::clamp_channel(self.paranoia + p.paranoia * s);
        self.aggression = Self::clamp_channel(self.aggression + p.aggression * s);
        self.instability = Self::clamp_channel(self.instability + p.instability * s);
        self.cohesion_drift = Self::clamp_channel(self.cohesion_drift + p.cohesion_drift * s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relax_dampens() {
        let mut f = PressureField {
            paranoia: 0.8,
            aggression: 0.5,
            instability: 0.5,
            cohesion_drift: 0.5,
        };
        f.relax(0.5);
        assert!((f.paranoia - 0.4).abs() < 1e-5);
    }

    #[test]
    fn accumulate_clamps() {
        let mut f = PressureField::default();
        f.accumulate(
            &PressureProfile {
                paranoia: 0.9,
                aggression: 0.9,
                instability: 0.9,
                cohesion_drift: 0.9,
            },
            2.0,
        );
        assert_eq!(f.paranoia, 1.0);
    }
}
