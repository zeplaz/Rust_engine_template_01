//! Fold [`AtmosphereCell`] into gameplay-facing samples (`base_fire2_smoke.md` §8, §16).
//!
//! **DEBT-011 / ES-6-3a:** path / mean-smoke probes sample clipmap L0 only (Field deleted).
//! **DEBT-012 keep:** [`merge_atmosphere_into_logistics_sample`] still takes the cell
//! sample DTO (dormant API · class C) — not an L0-direct signature yet.

use bevy::prelude::*;

use crate::substrate::atmosphere::{
    sample_tactical_smoke_from_l0, AtmosphereClipmapStack, CLIPMAP_L0_RES,
};
use crate::systems::navigation::LogisticsEnvironmentSample;

use super::diagnostics::AtmosphereDiagnostics;
use super::field::AtmosphereCell;
use super::pipeline::AtmospherePipelineSet;
use super::visibility::visibility_between;

/// Merge atmosphere hazards into a logistics environment row (same tick semantics as pathfinding builders).
#[inline]
pub fn merge_atmosphere_into_logistics_sample(sample: &mut LogisticsEnvironmentSample, cell: &AtmosphereCell) {
    sample.smoke_density = sample.smoke_density.max(cell.smoke_density);
    sample.toxicity = sample.toxicity.max(cell.toxicity);
}

/// Path / smoke probes folded into [`AtmosphereDiagnostics`] each frame.
fn atmosphere_coupling_refresh(
    stack: Res<AtmosphereClipmapStack>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    diag.coupling_runs = diag.coupling_runs.wrapping_add(1);
    diag.sample_path_visibility =
        visibility_between(Vec2::ZERO, Vec2::new(32.0, 0.0), stack.as_ref());

    let size = stack
        .levels
        .first()
        .map(|l| l.resolution)
        .unwrap_or(CLIPMAP_L0_RES);
    let n = (size.x * size.y).max(1) as f32;
    let mut sum = 0.0f32;
    for y in 0..size.y {
        for x in 0..size.x {
            sum += sample_tactical_smoke_from_l0(stack.as_ref(), x, y).unwrap_or(0.0);
        }
    }
    diag.sample_mean_smoke = sum / n;
}

pub fn coupling_systems(app: &mut App) {
    app.add_systems(
        Update,
        atmosphere_coupling_refresh.in_set(AtmospherePipelineSet::Coupling),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::ChunkEcology;
    use crate::systems::navigation::LogisticsEnvironmentSample;

    #[test]
    fn merge_takes_max_smoke_and_toxic() {
        let mut s = LogisticsEnvironmentSample::from_chunk_ecology_vegetation(
            &ChunkEcology::default(),
            &crate::systems::ecology::VegetationField::default(),
            0.1,
        );
        let cell = AtmosphereCell {
            smoke_density: 0.9,
            toxicity: 0.4,
            ..Default::default()
        };
        merge_atmosphere_into_logistics_sample(&mut s, &cell);
        assert!((s.smoke_density - 0.9).abs() < 1e-5);
        assert!((s.toxicity - 0.4).abs() < 1e-5);
    }
}
