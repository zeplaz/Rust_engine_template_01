//! Stage 7 shell stress harness — regression metrics for HUD scalability.

use bevy::prelude::*;

use super::hud_async_task_queue::{HudAsyncTask, HudAsyncTaskQueue};
use super::hud_interaction_budget::HudFrameBudget;
use super::retained_widget_cache::RetainedWidgetCache;
use super::shell_framework::ProductShellWidgetId;
use super::shell_widget_timing::ShellWidgetDiagnostics;

#[derive(Clone, Debug, Default)]
pub struct HudShellStressReport {
    pub rebuild_count: u64,
    pub retained_hits: u64,
    pub frame_spikes: u64,
    pub texture_rebinds: u64,
    pub deferred_widgets: u32,
    pub async_dropped: u64,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct HudShellStressHarness {
    pub enabled: bool,
    pub last_report: HudShellStressReport,
}

impl HudShellStressHarness {
    pub fn run_scenario(
        &mut self,
        retained: &mut RetainedWidgetCache,
        timing: &mut ShellWidgetDiagnostics,
        frame_budget: &HudFrameBudget,
        async_queue: &mut HudAsyncTaskQueue,
        texture_rebinds: u64,
    ) -> HudShellStressReport {
        retained.lookups = 0;
        retained.hits = 0;
        retained.misses = 0;
        timing.begin_frame();
        for i in 0..64 {
            let id = ProductShellWidgetId::Transmission;
            let revision = i / 8;
            if retained.should_skip_static(id, revision, revision) {
                continue;
            }
            retained.store_static(
                id,
                revision,
                revision,
                revision,
                vec![format!("stress row {i}")],
            );
        }
        for _ in 0..48 {
            async_queue.enqueue(HudAsyncTask::DispatchLogFormat);
        }
        let report = HudShellStressReport {
            rebuild_count: timing.frame_spike_markers,
            retained_hits: retained.hits,
            frame_spikes: timing.frame_spike_markers,
            texture_rebinds,
            deferred_widgets: frame_budget.deferred_widget_count_frame,
            async_dropped: async_queue.dropped,
        };
        self.last_report = report.clone();
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stress_harness_records_retained_hits() {
        let mut harness = HudShellStressHarness::default();
        let mut retained = RetainedWidgetCache::default();
        let mut timing = ShellWidgetDiagnostics::default();
        let frame_budget = HudFrameBudget::default();
        let mut async_queue = HudAsyncTaskQueue::default();
        let report = harness.run_scenario(&mut retained, &mut timing, &frame_budget, &mut async_queue, 0);
        assert!(report.retained_hits > 0);
        assert!(retained.cache_hit_rate() > 0.0);
    }
}
