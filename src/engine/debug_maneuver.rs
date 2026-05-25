//! Debug maneuver modes for operator / agent runs (see main menu + `--test` CLI).
//!
//! | Mode | Label | Behavior |
//! |------|-------|----------|
//! | **① FrameScreen** | Frame layout test | Small world → sim, UI layout debug, **stay open** |
//! | **② FullCapture** | Full capture proof | Auto world-gen → sim → min frames → proof JSON → graceful exit |
//! | **③ DemoOpen** | Demo world (stay open) | Menu world-gen → sim, **no auto-exit** |
//! | **Menu VFX** | Fire / weather / atmosphere buttons | Same bootstrap as CLI `--test fire|weather|…` via [`DebugQuickWorldGenPending::test_scene`] |
//! | **UnittestWorld** | Saved fixture | Load `assets/fixtures/unittest_world.ron` params, then capture path |

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::states::BaseState;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Minimum sim PostUpdate frames before FULL_APP / shell proof commit (mode ②).
pub const FULL_CAPTURE_MIN_FRAMES_DEFAULT: u32 = 90;

/// Post-proof frames before `AppExit` (Vulkan teardown cushion).
pub const GRACEFUL_EXIT_FRAMES_AFTER_PROOF: u32 = 30;

/// Operator-labeled debug maneuver (distinct from VFX `--test weather|fire|…` scenes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DebugManeuver {
    #[default]
    None,
    /// ① In-game frame / layout screen test — layout debug on, no proof auto-exit.
    FrameScreen,
    /// ② Auto world-gen → sim → capture proofs → graceful exit (`--test visual`).
    FullCapture,
    /// ③ Demo debug world — menu path, gen + enter, stay open.
    DemoOpen,
    /// Deterministic saved params (`assets/fixtures/unittest_world.ron`).
    UnittestWorld,
}

impl DebugManeuver {
    #[must_use]
    pub const fn menu_label(self) -> &'static str {
        match self {
            Self::None => "—",
            Self::FrameScreen => "① Frame layout test",
            Self::FullCapture => "② Full capture (auto exit)",
            Self::DemoOpen => "③ Demo world (stay open)",
            Self::UnittestWorld => "Unittest fixture world",
        }
    }

    #[must_use]
    pub const fn writes_full_capture_proof(self) -> bool {
        matches!(
            self,
            Self::FullCapture | Self::UnittestWorld
        )
    }

    #[must_use]
    pub fn auto_exit_after_proof(self) -> bool {
        matches!(self, Self::FullCapture)
    }
}

/// Saved world-gen params for fast regression boot (no full UI world-gen tuning).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnittestWorldFixture {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
}

impl Default for UnittestWorldFixture {
    fn default() -> Self {
        Self {
            label: "stage5_regression_default".into(),
            width: 320,
            height: 320,
            seed: 42,
        }
    }
}

impl UnittestWorldFixture {
    pub const DEFAULT_PATH: &'static str = "assets/fixtures/unittest_world.ron";

    pub fn apply_to_params(&self, params: &mut WorldGenParams) {
        params.width = self.width.max(64);
        params.height = self.height.max(64);
        params.seed = self.seed;
    }

    #[must_use]
    pub fn load_from_path(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        ron::from_str(&text).ok()
    }

    #[must_use]
    pub fn resolve_path(custom: Option<&str>) -> PathBuf {
        custom
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(Self::DEFAULT_PATH))
    }

    #[must_use]
    pub fn load_resolved(custom: Option<&str>) -> Self {
        let path = Self::resolve_path(custom);
        Self::load_from_path(&path).unwrap_or_else(|| {
            bevy::log::warn!(
                "UnittestWorldFixture missing at {}; using embedded defaults",
                path.display()
            );
            Self::default()
        })
    }
}

/// Forces UI layout tree dumps when maneuver ① is active (without requiring env vars).
#[derive(Resource, Default, Debug)]
pub struct FrameLayoutDebugSession {
    pub active: bool,
}

/// Counts PostUpdate frames in [`BaseState::Simulation`] for capture gating.
#[derive(Resource, Default, Debug)]
pub struct DebugCaptureFrameGate {
    pub sim_frames: u32,
}

pub fn tick_debug_capture_frame_gate(
    base: Res<State<BaseState>>,
    mut gate: ResMut<DebugCaptureFrameGate>,
) {
    if *base.get() == BaseState::Simulation {
        gate.sim_frames = gate.sim_frames.saturating_add(1);
    }
}

pub struct DebugManeuverPlugin;

impl Plugin for DebugManeuverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameLayoutDebugSession>()
            .init_resource::<DebugCaptureFrameGate>()
            .add_systems(PostUpdate, tick_debug_capture_frame_gate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_capture_requires_min_frames_constant() {
        assert!(FULL_CAPTURE_MIN_FRAMES_DEFAULT >= 90);
    }

    #[test]
    fn unittest_fixture_default_applies_seed() {
        let f = UnittestWorldFixture::default();
        let mut p = WorldGenParams::default();
        f.apply_to_params(&mut p);
        assert_eq!(p.width, 320);
        assert_eq!(p.seed, 42);
    }
}
