//! Stage 5 **closure witnesses** — tiny ECS resources so [`crate::dev::stage5_live_todos`] can mark each
//! TODO-01…TODO-13 [`TodoStatus::Done`] only when that row’s predicate is satisfied (not blanket on `passes`).

use bevy::prelude::*;

/// Latest post-mirror drift between [`crate::gui::MapCameraDesired`] and [`crate::gui::ViewId::WorldMain`].
/// Updated from [`crate::dev::stage5_live_todos::register_stage5_todo_runtime_hooks`] map-camera hook.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Stage5MapCameraBridgeWitness {
    pub last_post_mirror_drift: f32,
    pub consecutive_frames_bridge_ok: u32,
}

impl Default for Stage5MapCameraBridgeWitness {
    fn default() -> Self {
        Self {
            last_post_mirror_drift: f32::MAX,
            consecutive_frames_bridge_ok: 0,
        }
    }
}

/// [`crate::render::fire_view_extract::build_fire_visual_frames_by_view`] publishes orphan chunk count for WorldMain.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Stage5FireViewChunkWitness {
    pub world_main_visible_orphan_chunks: u32,
    /// F7-A-001: every [`crate::render::fire_view_extract::FireVisualFramesByView`] row ⊆ that view's visible set.
    pub f7_a_per_view_extract_bounded: bool,
}

/// PLAY-06e: overlay heat persistence in sim / `--test visual` (not a Stage 5 gate).
#[derive(Resource, Debug, Clone, Copy)]
pub struct FirePlaybackStabilityWitness {
    pub active_fire_chunks: u32,
    pub consecutive_frames_with_heat: u32,
    pub held_empty_snapshot_frames: u32,
    /// MAP-BLINK-001: frames retaining overlay while sim snapshot was empty (PLAY-06c/06d).
    pub held_overlay_persist_frames: u32,
    /// MAP-BLINK-001: cold-start overlay ramp (0..[`OVERLAY_WARMUP_BLEND_FRAMES`]).
    pub overlay_warmup_frames: u32,
    pub stable: bool,
}

impl Default for FirePlaybackStabilityWitness {
    fn default() -> Self {
        Self {
            active_fire_chunks: 0,
            consecutive_frames_with_heat: 0,
            held_empty_snapshot_frames: 0,
            held_overlay_persist_frames: 0,
            overlay_warmup_frames: 0,
            stable: false,
        }
    }
}

impl FirePlaybackStabilityWitness {
    pub const STABLE_FRAME_THRESHOLD: u32 = 10;
    /// MAP-BLINK-001: frames to blend overlay heat in after cold start.
    pub const OVERLAY_WARMUP_BLEND_FRAMES: u32 = 8;

    pub fn note_overlay_frame(&mut self, active_chunks: usize) {
        self.active_fire_chunks = active_chunks as u32;
        if active_chunks > 0 {
            self.consecutive_frames_with_heat = self
                .consecutive_frames_with_heat
                .saturating_add(1);
            self.stable = self.consecutive_frames_with_heat >= Self::STABLE_FRAME_THRESHOLD;
        } else {
            self.consecutive_frames_with_heat = 0;
        }
    }

    pub fn note_held_overlay_frame(&mut self) {
        self.held_empty_snapshot_frames = self.held_empty_snapshot_frames.saturating_add(1);
        self.held_overlay_persist_frames = self.held_overlay_persist_frames.saturating_add(1);
        self.consecutive_frames_with_heat = self
            .consecutive_frames_with_heat
            .saturating_add(1);
        self.stable = self.consecutive_frames_with_heat >= Self::STABLE_FRAME_THRESHOLD;
    }

    pub fn note_overlay_warmup_frame(&mut self) {
        if self.overlay_warmup_frames < Self::OVERLAY_WARMUP_BLEND_FRAMES {
            self.overlay_warmup_frames = self.overlay_warmup_frames.saturating_add(1);
        }
    }
}

/// Increments each time [`crate::gui::world_representation::compute_world_representation_frame`] emits the LOD band log.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Stage5LodBandLogWitness {
    pub lod_band_log_emissions: u64,
}
