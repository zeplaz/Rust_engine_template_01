//! Overlay heat persistence / cold-start warmup counters for the fire visual extract (MAP-BLINK-001).

use bevy::prelude::*;

/// PLAY-06e: overlay heat persistence in sim / `--test visual`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct FirePlaybackStabilityStats {
    pub active_fire_chunks: u32,
    pub consecutive_frames_with_heat: u32,
    pub held_empty_snapshot_frames: u32,
    /// MAP-BLINK-001: frames retaining overlay while sim snapshot was empty (PLAY-06c/06d).
    pub held_overlay_persist_frames: u32,
    /// MAP-BLINK-001: cold-start overlay ramp (0..[`OVERLAY_WARMUP_BLEND_FRAMES`]).
    pub overlay_warmup_frames: u32,
    pub stable: bool,
}

impl Default for FirePlaybackStabilityStats {
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

impl FirePlaybackStabilityStats {
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
