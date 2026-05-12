//! Soft perf / smoke budgets for dev overlays (`atm-scene-1b`).

use bevy::prelude::*;

/// When [`super::diagnostics::AtmosphereDiagnostics`] mean smoke / max toxicity exceed these,
/// the matching `*_over_budget` flags on diagnostics flip on.
#[derive(Resource, Debug, Clone, Copy)]
pub struct AtmospherePerfThresholds {
    pub warn_mean_smoke: f32,
    pub warn_max_toxicity: f32,
}

impl Default for AtmospherePerfThresholds {
    fn default() -> Self {
        Self {
            warn_mean_smoke: 0.28,
            warn_max_toxicity: 0.45,
        }
    }
}
