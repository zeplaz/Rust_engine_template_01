//! Authoritative world-preview lifecycle — explicit phases instead of ad-hoc system ordering.

use bevy::prelude::*;

use crate::engine::WorldGenFlowState;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use super::{WorldPreviewTexture, WorldPreviewUiState};
use crate::render::ResolvedViewports;
use crate::terrain::generation::world_generator_enhanced::{WorldGenJobSlot, WorldGenParams};
use crate::terrain::material::WorldPreviewState;

/// High-level preview lifecycle (single writer: [`advance_world_preview_lifecycle_system`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewLifecyclePhase {
    #[default]
    Uninitialized,
    GeneratingWorld,
    ReadyToRender,
    Rendering,
    Updating,
}

impl PreviewLifecyclePhase {
    #[must_use]
    pub fn allows_texture_bind(&self) -> bool {
        matches!(
            self,
            Self::ReadyToRender | Self::Rendering | Self::Updating
        )
    }

    #[must_use]
    pub fn allows_egui_present(&self) -> bool {
        matches!(self, Self::Rendering | Self::Updating)
    }

    #[must_use]
    pub fn placeholder_label(self) -> &'static str {
        match self {
            Self::Uninitialized => "Preview initializing",
            Self::GeneratingWorld => "Generating world",
            Self::ReadyToRender => "Preview texture pending",
            Self::Rendering | Self::Updating => "Preview texture pending",
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct WorldPreviewLifecycle {
    pub phase: PreviewLifecyclePhase,
    pub revision: u64,
}

impl WorldPreviewLifecycle {
    fn transition(&mut self, next: PreviewLifecyclePhase) {
        if self.phase == next {
            return;
        }
        self.phase = next;
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn park_uninitialized(&mut self) {
        self.transition(PreviewLifecyclePhase::Uninitialized);
    }
}

/// Frame-local signals from raster / present (writers only; consumed by advance).
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldPreviewLifecycleSignals {
    pub raster_wrote: bool,
    pub present_committed: bool,
}

pub fn note_world_preview_raster_wrote(signals: &mut WorldPreviewLifecycleSignals) {
    signals.raster_wrote = true;
}

pub fn note_world_preview_present_committed(signals: &mut WorldPreviewLifecycleSignals) {
    signals.present_committed = true;
}

#[must_use]
fn world_generation_active(flow: WorldGenFlowState, job_busy: bool) -> bool {
    job_busy || matches!(flow, WorldGenFlowState::NewWorldSetup)
}

#[must_use]
fn render_prerequisites_met(
    flow: WorldGenFlowState,
    params: &WorldGenParams,
    resolved: &ResolvedViewports,
    preview_tex: &WorldPreviewTexture,
    job_busy: bool,
) -> bool {
    if job_busy {
        return false;
    }
    if params.width == 0 || params.height == 0 {
        return false;
    }
    if preview_tex.width == 0 || preview_tex.height == 0 {
        return false;
    }
    if preview_tex.texture == Handle::default() {
        return false;
    }
    if !resolved.world_preview.valid {
        return false;
    }
    match flow {
        WorldGenFlowState::PreviewReady
        | WorldGenFlowState::FullReady
        | WorldGenFlowState::Idle
        | WorldGenFlowState::LoadingSave => true,
        WorldGenFlowState::NewWorldSetup => false,
    }
}

pub fn advance_world_preview_lifecycle_system(
    flow: Res<State<WorldGenFlowState>>,
    job_slot: Res<WorldGenJobSlot>,
    params: Res<WorldGenParams>,
    resolved: Res<ResolvedViewports>,
    preview_tex: Res<WorldPreviewTexture>,
    preview_state: Res<WorldPreviewState>,
    mut lifecycle: ResMut<WorldPreviewLifecycle>,
    mut signals: ResMut<WorldPreviewLifecycleSignals>,
) {
    let flow = *flow.get();
    let gen_active = world_generation_active(flow, job_slot.is_busy());
    let render_ready = render_prerequisites_met(flow, &params, &resolved, &preview_tex, job_slot.is_busy());
    let raster_wrote = signals.raster_wrote;
    let present_committed = signals.present_committed;

    match lifecycle.phase {
        PreviewLifecyclePhase::Uninitialized => {
            if gen_active {
                lifecycle.transition(PreviewLifecyclePhase::GeneratingWorld);
            } else if render_ready {
                lifecycle.transition(PreviewLifecyclePhase::ReadyToRender);
            }
        }
        PreviewLifecyclePhase::GeneratingWorld => {
            if gen_active {
                // hold
            } else if render_ready {
                lifecycle.transition(PreviewLifecyclePhase::ReadyToRender);
            }
        }
        PreviewLifecyclePhase::ReadyToRender => {
            if gen_active {
                lifecycle.transition(PreviewLifecyclePhase::GeneratingWorld);
            } else if raster_wrote {
                lifecycle.transition(PreviewLifecyclePhase::Rendering);
            }
        }
        PreviewLifecyclePhase::Rendering => {
            if gen_active {
                lifecycle.transition(PreviewLifecyclePhase::GeneratingWorld);
            } else if present_committed {
                lifecycle.transition(PreviewLifecyclePhase::Updating);
            }
        }
        PreviewLifecyclePhase::Updating => {
            if gen_active {
                lifecycle.transition(PreviewLifecyclePhase::GeneratingWorld);
            } else if !preview_state.dirty_queue.is_empty() && raster_wrote {
                lifecycle.transition(PreviewLifecyclePhase::Rendering);
            } else if raster_wrote && !present_committed {
                lifecycle.transition(PreviewLifecyclePhase::Rendering);
            }
        }
    }

    *signals = WorldPreviewLifecycleSignals::default();
}

/// When FullReady dismisses preview + generator panels, park lifecycle so GPU bind/raster skip.
pub fn park_preview_lifecycle_when_chrome_dismissed(
    flow: Res<State<WorldGenFlowState>>,
    preview_ui: Res<WorldPreviewUiState>,
    world_gen: Res<WorldGenUiState>,
    mut lifecycle: ResMut<WorldPreviewLifecycle>,
) {
    if preview_ui.window_open || world_gen.visible {
        return;
    }
    if !matches!(*flow.get(), WorldGenFlowState::FullReady) {
        return;
    }
    lifecycle.park_uninitialized();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_prerequisites_require_valid_viewport_and_texture() {
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let mut resolved = ResolvedViewports::default();
        resolved.world_preview.valid = true;
        let preview_tex = WorldPreviewTexture {
            texture: Handle::default(),
            width: 64,
            height: 64,
        };
        assert!(!render_prerequisites_met(
            WorldGenFlowState::Idle,
            &params,
            &resolved,
            &preview_tex,
            false,
        ));
    }

    #[test]
    fn present_allowed_only_after_rendering_phase() {
        assert!(!PreviewLifecyclePhase::ReadyToRender.allows_egui_present());
        assert!(PreviewLifecyclePhase::Rendering.allows_egui_present());
        assert!(PreviewLifecyclePhase::Updating.allows_egui_present());
    }
}
