//! Passive render-schedule witness **models** (RenderApp timing probes removed — Great Unhook P0).

use bevy::prelude::*;

/// Wall times between render schedule boundaries (ms).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderScheduleSpans {
    pub extract_schedule_ms: f32,
    pub extract_commands_ms: f32,
    pub prepare_assets_ms: f32,
    pub prepare_meshes_ms: f32,
    pub manage_views_ms: f32,
    pub queue_ms: f32,
    pub phase_sort_ms: f32,
    pub prepare_ms: f32,
    pub render_and_present_ms: f32,
    pub cleanup_ms: f32,
    pub post_cleanup_ms: f32,
    pub total_render_app_ms: f32,
}

impl RenderScheduleSpans {
    #[must_use]
    pub fn sum_phases_ms(&self) -> f32 {
        self.extract_schedule_ms
            + self.extract_commands_ms
            + self.prepare_assets_ms
            + self.prepare_meshes_ms
            + self.manage_views_ms
            + self.queue_ms
            + self.phase_sort_ms
            + self.prepare_ms
            + self.render_and_present_ms
            + self.cleanup_ms
            + self.post_cleanup_ms
    }
}

/// Latest render-thread spans consumed on the main world (typically previous frame).
#[derive(Resource, Clone, Debug, Default)]
pub struct RenderScheduleWitness {
    pub spans: RenderScheduleSpans,
    pub main_thread_handoff_total_ms: f32,
    pub frames_received: u64,
}

/// Main-thread extract handoff timing — legacy field retained for sim_spectrum JSON shape.
#[derive(Resource, Clone, Debug, Default)]
pub struct RenderScheduleHandoffMs(pub f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_schedule_spans_sum_matches_fields() {
        let s = RenderScheduleSpans {
            extract_schedule_ms: 1.0,
            extract_commands_ms: 2.0,
            prepare_assets_ms: 3.0,
            prepare_meshes_ms: 4.0,
            manage_views_ms: 5.0,
            queue_ms: 6.0,
            phase_sort_ms: 7.0,
            prepare_ms: 8.0,
            render_and_present_ms: 9.0,
            cleanup_ms: 1.0,
            post_cleanup_ms: 1.0,
            total_render_app_ms: 47.0,
        };
        assert!((s.sum_phases_ms() - 47.0).abs() < 1e-3);
    }
}
