//! Batched HUD layout persistence — debounce writes during drag.

use std::collections::HashMap;

use bevy::prelude::*;

use super::layout_store::{HudLayoutStore, HudWidgetFrame};
use super::shell_framework::ProductShellWidgetId;

const IDLE_CHECKPOINT_SECS: f32 = 2.0;

#[derive(Resource, Clone, Debug, Default)]
pub struct PendingHudLayoutCommit {
    pub drag_active: bool,
    pub pending: HashMap<ProductShellWidgetId, HudWidgetFrame>,
    pub deferred_captures: u64,
    pub flushed_captures: u64,
    pub drag_mutation_attempts_frame: u32,
    pub drag_mutation_by_widget: HashMap<ProductShellWidgetId, u32>,
    idle_secs: f32,
}

impl PendingHudLayoutCommit {
    #[must_use]
    pub fn can_emit_layout_capture(&self) -> bool {
        !self.drag_active
    }

    pub fn set_drag_active(&mut self, active: bool) {
        if active {
            self.drag_active = true;
            self.idle_secs = 0.0;
        } else if self.drag_active {
            self.drag_active = false;
        }
    }

pub fn queue_capture() {}

    pub fn flush(&mut self, layout: &mut HudLayoutStore) -> u32 {
        if self.drag_active || self.pending.is_empty() {
            return 0;
        }
        let mut applied = 0u32;
        for (id, frame) in self.pending.drain() {
            layout.set_frame(id, frame);
            layout.bump_layout_captures_applied(id);
            applied = applied.saturating_add(1);
        }
        self.flushed_captures = self.flushed_captures.wrapping_add(applied as u64);
        applied
    }

    #[must_use]
    pub fn top_drag_mutation_offender(&self) -> Option<ProductShellWidgetId> {
        self.drag_mutation_by_widget
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, _)| *id)
    }

    pub fn begin_frame(&mut self) {
        self.drag_mutation_attempts_frame = 0;
        self.drag_mutation_by_widget.clear();
    }
}

pub fn flush_pending_hud_layout_commits(
    time: Res<Time>,
    mut pending: ResMut<PendingHudLayoutCommit>,
    mut layout: ResMut<HudLayoutStore>,
) {
    if pending.drag_active {
        pending.idle_secs = 0.0;
        return;
    }
    pending.idle_secs += time.delta_secs();
    if pending.pending.is_empty() {
        return;
    }
    if pending.idle_secs >= IDLE_CHECKPOINT_SECS {
        pending.flush(&mut layout);
        pending.idle_secs = 0.0;
    }
}

pub fn finalize_pending_hud_layout_commits(
    mut pending: ResMut<PendingHudLayoutCommit>,
    mut layout: ResMut<HudLayoutStore>,
) {
    if pending.drag_active {
        return;
    }
    pending.flush(&mut layout);
}

pub fn flush_pending_hud_layout_on_pointer_release(
    mut pending: ResMut<PendingHudLayoutCommit>,
    mut layout: ResMut<HudLayoutStore>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if mouse.just_released(MouseButton::Left) {
        pending.flush(&mut layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_freeze_blocks_layout_capture_emit() {
        let mut pending = PendingHudLayoutCommit::default();
        assert!(pending.can_emit_layout_capture());
        pending.drag_active = true;
        assert!(!pending.can_emit_layout_capture());
    }
}
