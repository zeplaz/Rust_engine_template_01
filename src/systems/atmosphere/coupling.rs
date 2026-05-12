//! Fold [`AtmosphereCell`] into gameplay-facing samples (`base_fire2_smoke.md` §8, §16).

use bevy::prelude::*;

use crate::systems::navigation::LogisticsEnvironmentSample;

use super::diagnostics::AtmosphereDiagnostics;
use super::field::{AtmosphereCell, AtmosphereField};
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
    field: Res<AtmosphereField>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    diag.coupling_runs = diag.coupling_runs.wrapping_add(1);
    diag.sample_path_visibility = visibility_between(Vec2::ZERO, Vec2::new(32.0, 0.0), &field);
    let n = field.cells.len().max(1) as f32;
    diag.sample_mean_smoke = field.cells.iter().map(|c| c.smoke_density).sum::<f32>() / n;
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