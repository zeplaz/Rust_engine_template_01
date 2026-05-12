//! Frame counters / cheap aggregates for egui (`base_fire2_smoke.md` §13).

use bevy::prelude::*;

use super::field::AtmosphereField;
use super::perf_overlay::AtmospherePerfThresholds;
use super::pipeline::AtmospherePipelineSet;

#[derive(Resource, Debug, Default, Clone)]
pub struct AtmosphereDiagnostics {
    pub field_fill_runs: u64,
    pub advect_runs: u64,
    pub emitter_sync_runs: u64,
    pub particle_controller_runs: u64,
    pub coupling_runs: u64,
    pub visual_extract_runs: u64,
    pub render_prep_runs: u64,
    pub last_emitter_extract_count: usize,
    pub last_smoke_extract_count: usize,
    pub last_mean_smoke: f32,
    pub last_mean_visibility: f32,
    pub last_max_toxicity: f32,
    pub mean_smoke_over_budget: bool,
    pub max_toxicity_over_budget: bool,
    /// Sample from [`super::visibility::visibility_between`] along a fixed probe segment (HUD / debug).
    pub sample_path_visibility: f32,
    pub sample_mean_smoke: f32,
}

fn atmosphere_diagnostics_sample(
    field: Res<AtmosphereField>,
    thresholds: Res<AtmospherePerfThresholds>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    let n = field.cells.len().max(1) as f32;
    let mut sum_s = 0f32;
    let mut sum_v = 0f32;
    let mut max_t = 0f32;
    for c in &field.cells {
        sum_s += c.smoke_density;
        sum_v += c.visibility;
        max_t = max_t.max(c.toxicity);
    }
    diag.last_mean_smoke = sum_s / n;
    diag.last_mean_visibility = sum_v / n;
    diag.last_max_toxicity = max_t;

    diag.mean_smoke_over_budget = diag.last_mean_smoke > thresholds.warn_mean_smoke;
    diag.max_toxicity_over_budget = diag.last_max_toxicity > thresholds.warn_max_toxicity;
}

pub fn atmosphere_diagnostics_systems(app: &mut App) {
    app.init_resource::<AtmosphereDiagnostics>()
        .init_resource::<AtmospherePerfThresholds>()
        .add_systems(
            Update,
            atmosphere_diagnostics_sample.in_set(AtmospherePipelineSet::Diagnostics),
        );
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::systems::atmosphere::AtmospherePlugin;
    use crate::systems::sim_control::SimControlPlugin;

    #[test]
    fn diagnostics_counters_advance_with_ticks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        app.add_plugins(AtmospherePlugin);

        app.update();
        let a = app.world().resource::<super::AtmosphereDiagnostics>().field_fill_runs;
        app.update();
        let b = app.world().resource::<super::AtmosphereDiagnostics>().field_fill_runs;
        assert!(b >= a + 1, "expected field_fill_runs to advance each sim tick");
    }
}
