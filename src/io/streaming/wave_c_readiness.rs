//! Wave C readiness / governance checks (runbook §6 + G3 §4 BQ deltas).

use super::wave_c_prerequisites::{
    gather_wave_c_prerequisites, wave_c_prerequisites_passes, WAVE_C_OPEN_BACKLOG_ITEMS,
};
use crate::gui::editor::world_preview::PreviewPathAuthority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaveCReadinessReport {
    pub prerequisites_ok: bool,
    pub open_backlog_items: u32,
}

#[must_use]
pub fn gather_wave_c_readiness(authority: &PreviewPathAuthority) -> WaveCReadinessReport {
    let prerequisites = gather_wave_c_prerequisites(authority);
    WaveCReadinessReport {
        prerequisites_ok: wave_c_prerequisites_passes(&prerequisites),
        open_backlog_items: WAVE_C_OPEN_BACKLOG_ITEMS.len() as u32,
    }
}

#[must_use]
pub fn wave_c_readiness_passes(report: &WaveCReadinessReport) -> bool {
    report.prerequisites_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_c_readiness_passes_with_default_preview_authority() {
        let report = gather_wave_c_readiness(&PreviewPathAuthority::default());
        assert!(wave_c_readiness_passes(&report));
        assert_eq!(report.open_backlog_items, WAVE_C_OPEN_BACKLOG_ITEMS.len() as u32);
    }
}
