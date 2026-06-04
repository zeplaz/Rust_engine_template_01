//! AC-004 — L0 smoke advect + fold helpers (mass-preserving stub).

use super::{AtmosphereClipLevel, AtmosphereClipmapStack};

/// Semi-Lagrangian-style neighbor blend that preserves total mass on L0.
pub fn advect_l0_preserving_mass(level: &mut AtmosphereClipLevel) {
    let n = level.smoke_density.len();
    if n < 2 {
        return;
    }
    let total_before: f32 = level.smoke_density.iter().sum();
    if total_before <= 0.0 {
        return;
    }

    let mut next = level.smoke_density.clone();
    for i in 1..n {
        let blended = (level.smoke_density[i] + level.smoke_density[i - 1]) * 0.5;
        next[i] = blended;
        next[i - 1] = blended;
    }

    let total_after: f32 = next.iter().sum();
    if total_after > 1e-6 {
        let scale = total_before / total_after;
        for v in &mut next {
            *v *= scale;
        }
    }
    level.smoke_density = next;
}

pub fn fold_registry_smoke_into_l0(stack: &mut AtmosphereClipmapStack, smoke_seed: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    if let Some(cell0) = level0.smoke_density.first_mut() {
        *cell0 = cell0.max(smoke_seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn clipmap_advect_preserves_mass_approximately() {
        let mut level = super::super::AtmosphereClipLevel {
            resolution: UVec2::new(4, 1),
            smoke_density: vec![0.1, 0.2, 0.3, 0.4],
        };
        let before: f32 = level.smoke_density.iter().sum();
        advect_l0_preserving_mass(&mut level);
        let after: f32 = level.smoke_density.iter().sum();
        assert!((before - after).abs() < 1e-4);
        assert!(level.smoke_density.iter().any(|v| *v > 0.0));
    }
}
