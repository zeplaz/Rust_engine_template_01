//! CLI / process launch configuration inserted before [`crate::engine::EnginePlugin`] runs.

use bevy::prelude::Resource;

use super::debug_maneuver::{DebugManeuver, FULL_CAPTURE_MIN_FRAMES_DEFAULT};

/// `--test weather|fire|atmosphere|visual`: generated world + sim debug defaults for VFX / systems checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TestScene {
    #[default]
    None,
    Weather,
    Fire,
    /// Strong wind + hot chunks for atmosphere field / advection smoke tests.
    Atmosphere,
    /// **Recommended** smoke / fire / GPU field / precip demo: combines wind, fire, weather, and fuel cues.
    Visual,
    /// Menu stay-open: fire + weather + atmosphere with debug overlay visible (operator sandbox).
    VfxSandbox,
}

impl TestScene {
    #[must_use]
    pub const fn menu_label(self) -> &'static str {
        match self {
            Self::None => "—",
            Self::Weather => "Weather sim",
            Self::Fire => "Fire sim",
            Self::Atmosphere => "Atmosphere / wind",
            Self::Visual => "Full capture visual",
            Self::VfxSandbox => "Fire + weather + atmosphere",
        }
    }

    #[must_use]
    pub const fn seeds_fire_overlay(self) -> bool {
        matches!(
            self,
            Self::Fire | Self::Atmosphere | Self::Visual | Self::VfxSandbox
        )
    }

    #[must_use]
    pub const fn menu_vfx_bootstrap(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Effective launch flags (from `main`, environment wrappers, etc.).
#[derive(Clone, Debug, Resource)]
pub struct EngineLaunchArgs {
    pub test_scene: TestScene,
    pub maneuver: DebugManeuver,
    /// When false, full-capture still runs harness/proof but does not `AppExit` after proof.
    pub visual_auto_exit: bool,
    /// Mode ②: minimum sim frames before proof JSON commit.
    pub min_capture_frames: u32,
    /// Optional override for [`super::debug_maneuver::UnittestWorldFixture`].
    pub unittest_fixture_path: Option<String>,
}

impl Default for EngineLaunchArgs {
    fn default() -> Self {
        Self {
            test_scene: TestScene::None,
            maneuver: DebugManeuver::None,
            visual_auto_exit: true,
            min_capture_frames: FULL_CAPTURE_MIN_FRAMES_DEFAULT,
            unittest_fixture_path: None,
        }
    }
}

impl EngineLaunchArgs {
    #[must_use]
    pub fn from_cli(
        test_raw: Option<String>,
        stay_open: bool,
        unittest_fixture: Option<String>,
    ) -> Self {
        let mut args = Self::default();
        args.unittest_fixture_path = unittest_fixture;

        let Some(raw) = test_raw else {
            return args;
        };

        match raw.to_lowercase().as_str() {
            "weather" => {
                args.test_scene = TestScene::Weather;
            }
            "fire" => {
                args.test_scene = TestScene::Fire;
            }
            "atmosphere" => {
                args.test_scene = TestScene::Atmosphere;
            }
            // ② Full capture — auto world-gen, min frames, proof, graceful exit (unless --stay-open).
            "visual" | "capture" | "full" => {
                args.test_scene = TestScene::Visual;
                args.maneuver = DebugManeuver::FullCapture;
                args.visual_auto_exit = !stay_open;
            }
            // ① Frame / layout screen test — small world, layout debug, stay open.
            "frame" | "layout" | "framescreen" => {
                args.test_scene = TestScene::Visual;
                args.maneuver = DebugManeuver::FrameScreen;
                args.visual_auto_exit = false;
            }
            // Saved unittest fixture → same capture path as visual, fixed params.
            "unittest" | "unit" => {
                args.test_scene = TestScene::Visual;
                args.maneuver = DebugManeuver::UnittestWorld;
                args.visual_auto_exit = !stay_open;
            }
            // ③ Demo from CLI (mirrors menu stay-open path).
            "demo" | "open" => {
                args.test_scene = TestScene::Visual;
                args.maneuver = DebugManeuver::DemoOpen;
                args.visual_auto_exit = false;
            }
            "vfx" | "sandbox" | "fireweather" => {
                args.test_scene = TestScene::VfxSandbox;
                args.maneuver = DebugManeuver::DemoOpen;
                args.visual_auto_exit = false;
            }
            other => {
                bevy::log::warn!(
                    "Unknown --test mode {other:?}; use \
                     `frame`, `visual`/`capture`, `demo`, `vfx`, `unittest`, or `weather`/`fire`/`atmosphere`."
                );
            }
        }

        args
    }

    /// Legacy helper — prefer [`Self::from_cli`].
    #[must_use]
    pub fn from_test_cli_flag(raw: Option<String>, stay_open: bool) -> Self {
        Self::from_cli(raw, stay_open, None)
    }

    #[must_use]
    pub fn test_mode(&self) -> bool {
        self.test_scene != TestScene::None
    }

    #[must_use]
    pub fn full_capture_active(&self) -> bool {
        self.maneuver.writes_full_capture_proof() && self.test_scene == TestScene::Visual
    }

    /// P2-VFX-VISUAL-001 — tactical zoom + fire visibility loosened for visual proof / VfxSandbox.
    #[must_use]
    pub fn visual_tactical_vfx_proof(&self) -> bool {
        self.full_capture_active()
            || matches!(self.test_scene, TestScene::Visual | TestScene::VfxSandbox)
    }

    /// Tactical zoom on sim enter — proof capture / VfxSandbox only; interactive `--test demo` stays world-fit.
    #[must_use]
    pub fn sim_enter_uses_tactical_camera_zoom(&self) -> bool {
        match self.test_scene {
            TestScene::VfxSandbox => true,
            TestScene::Visual => self.full_capture_active(),
            _ => false,
        }
    }

    /// Disk analytics + stall spans + ECS inventory for `--test` harness runs.
    #[must_use]
    pub fn test_instrumentation_profile(&self) -> TestInstrumentationProfile {
        if !self.test_mode() {
            return TestInstrumentationProfile::default();
        }
        let frame_jsonl = matches!(
            self.test_scene,
            TestScene::Weather
                | TestScene::Fire
                | TestScene::Atmosphere
                | TestScene::Visual
                | TestScene::VfxSandbox
        ) || self.full_capture_active();
        let flush_secs = if self.full_capture_active() { 2.0 } else { 5.0 };
        let frame_jsonl_stride = match self.test_scene {
            TestScene::Visual => 30,
            TestScene::VfxSandbox => 10,
            TestScene::Weather | TestScene::Fire | TestScene::Atmosphere => 15,
            TestScene::None => 1,
        };
        TestInstrumentationProfile {
            active: true,
            quiet_terminal: true,
            frame_jsonl,
            stall_spans: true,
            flush_secs,
            frame_jsonl_stride,
        }
    }
}

/// Auto instrumentation profile published when [`EngineLaunchArgs::test_mode`] is true.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TestInstrumentationProfile {
    pub active: bool,
    pub quiet_terminal: bool,
    pub frame_jsonl: bool,
    pub stall_spans: bool,
    pub flush_secs: f32,
    pub frame_jsonl_stride: u32,
}

impl Default for TestInstrumentationProfile {
    fn default() -> Self {
        Self {
            active: false,
            quiet_terminal: false,
            frame_jsonl: false,
            stall_spans: false,
            flush_secs: 5.0,
            frame_jsonl_stride: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::debug_maneuver::DebugManeuver;

    #[test]
    fn demo_open_uses_world_fit_camera_not_tactical_proof() {
        let a = EngineLaunchArgs::from_cli(Some("demo".into()), false, None);
        assert_eq!(a.maneuver, DebugManeuver::DemoOpen);
        assert_eq!(a.test_scene, TestScene::Visual);
        assert!(!a.full_capture_active());
        assert!(!a.sim_enter_uses_tactical_camera_zoom());
        let capture = EngineLaunchArgs::from_cli(Some("visual".into()), false, None);
        assert!(capture.sim_enter_uses_tactical_camera_zoom());
    }

    #[test]
    fn cli_visual_maps_full_capture() {
        let a = EngineLaunchArgs::from_cli(Some("visual".into()), false, None);
        assert_eq!(a.test_scene, TestScene::Visual);
        assert_eq!(a.maneuver, DebugManeuver::FullCapture);
        assert!(a.visual_auto_exit);
        let b = EngineLaunchArgs::from_cli(Some("capture".into()), true, None);
        assert_eq!(b.maneuver, DebugManeuver::FullCapture);
        assert!(b.full_capture_active());
        assert!(!b.visual_auto_exit);
    }

    #[test]
    fn cli_frame_stays_open() {
        let a = EngineLaunchArgs::from_cli(Some("frame".into()), false, None);
        assert_eq!(a.maneuver, DebugManeuver::FrameScreen);
        assert!(!a.visual_auto_exit);
    }

    #[test]
    fn cli_unittest_fixture_path() {
        let a = EngineLaunchArgs::from_cli(
            Some("unittest".into()),
            false,
            Some("assets/fixtures/custom.ron".into()),
        );
        assert_eq!(a.maneuver, DebugManeuver::UnittestWorld);
        assert_eq!(
            a.unittest_fixture_path.as_deref(),
            Some("assets/fixtures/custom.ron")
        );
    }

    #[test]
    fn test_scene_menu_labels() {
        assert_eq!(TestScene::VfxSandbox.menu_label(), "Fire + weather + atmosphere");
        assert!(TestScene::VfxSandbox.seeds_fire_overlay());
        assert!(!TestScene::Weather.seeds_fire_overlay());
    }

    #[test]
    fn cli_vfx_sandbox_stays_open() {
        let a = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None);
        assert_eq!(a.test_scene, TestScene::VfxSandbox);
        assert_eq!(a.maneuver, DebugManeuver::DemoOpen);
        assert!(!a.visual_auto_exit);
    }

    #[test]
    fn cli_atmosphere_maps() {
        let a = EngineLaunchArgs::from_cli(Some("atmosphere".into()), false, None);
        assert_eq!(a.test_scene, TestScene::Atmosphere);
        assert!(a.test_mode());
    }

    #[test]
    fn test_instrumentation_profile_vfx() {
        let a = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None);
        let p = a.test_instrumentation_profile();
        assert!(p.active);
        assert!(p.quiet_terminal);
        assert!(p.frame_jsonl);
        assert!(p.stall_spans);
    }

    #[test]
    fn test_instrumentation_profile_visual_flush() {
        let a = EngineLaunchArgs::from_cli(Some("visual".into()), false, None);
        assert!((a.test_instrumentation_profile().flush_secs - 2.0).abs() < f32::EPSILON);
    }
}
