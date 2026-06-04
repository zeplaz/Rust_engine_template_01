//! High-level UX orchestration states — single source of truth for finish-lane flows.
//!
//! Legacy [`crate::engine::BaseState`], [`crate::engine::WorldGenFlowState`], and
//! [`crate::engine::InGameMenuState`] are driven by [`super::ux_orchestration::bridge_ux_to_legacy`].
//! Subsystems that still mutate legacy states are mirrored back via [`sync_legacy_to_ux`].

use bevy::prelude::*;

/// Application mode (replaces ad-hoc combinations of [`crate::engine::BaseState`] + menus).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Setup,
    WorldGen,
    InGame,
    Paused,
    Shutdown,
}

/// World-generation lifecycle (maps to [`crate::engine::WorldGenFlowState`] + job slot).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum WorldGenState {
    #[default]
    Idle,
    /// Operator editing params / preview panel visible.
    Preview,
    /// Background job running (preview or full).
    Generating,
    /// Preview terrain ready ([`crate::engine::WorldGenFlowState::PreviewReady`]).
    Ready,
    /// Full world ready ([`crate::engine::WorldGenFlowState::FullReady`]).
    FullReady,
    /// Operator skipped preview; skip preview GPU/CPU lifecycle ([`FINISH-UX-07`]).
    Dismissed,
}

/// Pause overlay sub-state while [`AppState::Paused`].
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PauseState {
    #[default]
    Off,
    Menu,
    ConfirmExit,
}

/// Latch: full-world chrome was dismissed (prevents FullReady OnEnter dismiss loops).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WorldGenChromeLatch {
    pub full_ready_dismissed: bool,
}

impl WorldGenChromeLatch {
    pub fn reset_for_new_flow(&mut self) {
        self.full_ready_dismissed = false;
    }

    pub fn mark_full_ready_dismissed(&mut self) {
        self.full_ready_dismissed = true;
    }
}

/// FINISH-UX-06: frame spike guard — throttle heavy work when over budget.
#[derive(Resource, Clone, Debug)]
pub struct UxFrameSpikeGuard {
    pub max_ms: f32,
    pub last_frame_ms: f32,
    pub spike_active: bool,
    pub suppress_preview_this_frame: bool,
    /// Map-fit validation, entity scans, and other optional diagnostics.
    pub suppress_optional_diagnostics: bool,
    /// Consecutive over-budget frames before throttling preview (reduces flicker).
    pub spike_enter_frames: u8,
    pub(crate) spike_over_budget_streak: u8,
}

impl Default for UxFrameSpikeGuard {
    fn default() -> Self {
        Self {
            max_ms: 33.0,
            last_frame_ms: 0.0,
            spike_active: false,
            suppress_preview_this_frame: false,
            suppress_optional_diagnostics: false,
            spike_enter_frames: 2,
            spike_over_budget_streak: 0,
        }
    }
}

#[must_use]
pub fn worldgen_lifecycle_active(worldgen: &WorldGenState) -> bool {
    !matches!(worldgen, WorldGenState::Idle | WorldGenState::Dismissed)
}

#[must_use]
pub fn worldgen_preview_systems_enabled(worldgen: &WorldGenState) -> bool {
    matches!(
        worldgen,
        WorldGenState::Preview
            | WorldGenState::Generating
            | WorldGenState::Ready
            | WorldGenState::FullReady
    )
}
