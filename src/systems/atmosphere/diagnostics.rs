//! Frame counters / cheap aggregates for egui (`base_fire2_smoke.md` §13).
//!
//! **DEBT-011 / ES-6-3a:** mean smoke / visibility / max toxicity sample clipmap L0 only.

use bevy::prelude::*;

use crate::substrate::atmosphere::{
    sample_tactical_toxicity_from_l0, AtmosphereClipmapStack, CLIPMAP_L0_RES,
};

use super::incremental_schedule::AtmospherePartialWriteMetrics;
use super::perf_overlay::AtmospherePerfThresholds;
use super::pipeline::AtmospherePipelineSet;
use super::visibility::sample_tactical_visibility_from_l0;

#[derive(Resource, Debug, Default, Clone)]
pub struct AtmosphereDiagnostics {
    pub field_fill_runs: u64,
    pub advect_runs: u64,
    pub emitter_sync_runs: u64,
    /// Retired ES-0 — stub controller removed from schedule; stays 0.
    pub particle_controller_runs: u64,
    pub coupling_runs: u64,
    pub visual_extract_runs: u64,
    /// Retired ES-0 — render-prep placeholder removed; stays 0.
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
    pub partial_field_writes: u64,
    pub full_field_reconciles: u64,
    /// MIG-A6 — true when last fill used [`Query::contiguous_iter`].
    pub last_fill_contiguous: bool,
    pub stale_partial_region_skips: u64,
    pub partial_write_metrics: AtmospherePartialWriteMetrics,
    /// W1 — distinct L0 advect writers observed (`AtmosphereClipmapStack::advect_writer_mask`).
    /// Dump counter only; the reviewer script scores it (expected ≤ 1).
    pub l0_advect_writers: u32,
    /// W1 — mirror of `AtmosphereClipmapStack::revision` at the last diagnostics sample.
    pub l0_revision: u64,
    /// W1 — L0 smoke max / mass at the last diagnostics sample (dump counters).
    pub l0_smoke_max: f32,
    pub l0_smoke_mass: f32,
    /// W4 — cumulative smoke mass removed by L0 rain washout this session.
    pub rain_washout_mass: f32,
    /// W4 — contiguous firestorm plume count (from `WeatherSourceTerms` build).
    pub firestorm_plumes: u32,
    /// W4 — peak plume updraft (m/s).
    pub plume_updraft_peak_mps: f32,
    /// W4 — moisture injected by last weather solve (or last terms build estimate).
    pub pyro_moisture_injected: f32,
    /// W4 — cloud lift applied by last weather solve that read source terms.
    pub pyro_cloud_delta: f32,
    /// W4 — L0 tiles with smoke+cloud both above threshold.
    pub smoke_cloud_overlap_cells: u32,
    /// W6 — distinct GPU weather/fire **field** upload writers (expects `1` = `WeatherFireFieldUniforms`).
    pub gpu_field_upload_paths: u32,
    /// W6 — [`super::frame::AtmosphereFrame::revision`] last read by presentation (GPU bridge / precip).
    pub presentation_frame_revision: u64,
    /// W7 — reviewer counter: extra L0 advect writers beyond the sole WindAdvect authority.
    /// `0` when `l0_advect_writers ≤ 1`. Not a production `*_green` predicate.
    pub second_writer_violations: u32,
}

fn atmosphere_diagnostics_sample(
    stack: Res<AtmosphereClipmapStack>,
    thresholds: Res<AtmospherePerfThresholds>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    let size = stack
        .levels
        .first()
        .map(|l| l.resolution)
        .unwrap_or(CLIPMAP_L0_RES);
    let n = (size.x * size.y).max(1) as f32;
    let mut sum_s = 0f32;
    let mut sum_v = 0f32;
    let mut max_t = 0f32;
    for y in 0..size.y {
        for x in 0..size.x {
            sum_s += crate::substrate::atmosphere::sample_tactical_smoke_from_l0(
                stack.as_ref(),
                x,
                y,
            )
            .unwrap_or(0.0);
            sum_v += sample_tactical_visibility_from_l0(stack.as_ref(), x, y);
            max_t = max_t.max(
                sample_tactical_toxicity_from_l0(stack.as_ref(), x, y).unwrap_or(0.0),
            );
        }
    }
    diag.last_mean_smoke = sum_s / n;
    diag.last_mean_visibility = sum_v / n;
    diag.last_max_toxicity = max_t;
    diag.l0_advect_writers = stack.l0_advect_writers();
    diag.l0_revision = stack.revision;
    diag.l0_smoke_max = crate::substrate::atmosphere::clipmap_l0_smoke_max(stack.as_ref());
    diag.l0_smoke_mass = stack.l0_smoke_mass();
    // W7 Accept: dump second_writer_violations == 0 (reviewer/script, not production gate).
    diag.second_writer_violations = stack.l0_advect_writers().saturating_sub(1);

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
