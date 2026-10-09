//! CLI / process launch configuration inserted before [`crate::engine::EnginePlugin`] runs.

use bevy::prelude::Resource;

use super::debug_maneuver::{DebugManeuver, FULL_CAPTURE_MIN_FRAMES_DEFAULT};

/// Product spectator / fire proof entry path (VSS-T2-001 witness `entry_path` field).
///
/// `Harness` is **debug fallback only** — not sole ship proof per `TRIP_VSS_TEST_HARNESS.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpectatorEntryPath {
    /// CLI/menu `--test` harness (`apply_test_scene_fire_seeds`) — diagnostic scaffold.
    Harness,
    /// Scenario script / `play_scenario` ignite path.
    Scenario,
    /// Save-game hydrate + sim resume.
    Save,
    /// Operator interactive session (SimEffect / ECS ignition).
    Interactive,
}

impl SpectatorEntryPath {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Harness => "harness",
            Self::Scenario => "scenario",
            Self::Save => "save",
            Self::Interactive => "interactive",
        }
    }
}

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
    /// Uses FullCapture proof path when invoked as `--test visual` (not `--test vfx`).
    Visual,
    /// Operator sandbox — fire + weather + atmosphere; stays open; **no** full-capture auto-proof (`DemoOpen`).
    /// CLI: `--test sandbox` / `--test fireweather`. `--test vfx` is fire-only ([`Self::Vfx`]).
    VfxSandbox,
    /// Generated towns on the play map. Stay-open. Does not seed fire, weather, or atmosphere.
    City,
    /// Fire / fire-VFX only. Stay-open. No weather, atmosphere, or towns.
    /// Fire overlay uses the same heat/fuel/count as [`Self::Fire`].
    Vfx,
    /// Weather + atmosphere (wind / precip). Stay-open. No fire overlay and no towns.
    /// CLI name is `subfx`.
    SubFx,
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
            Self::City => "Generated cities",
            Self::Vfx => "Fire VFX",
            Self::SubFx => "Weather + atmosphere",
        }
    }

    #[must_use]
    pub const fn seeds_fire_overlay(self) -> bool {
        matches!(
            self,
            Self::Fire | Self::Atmosphere | Self::Visual | Self::VfxSandbox | Self::Vfx
        )
    }

    /// Rain / fog / soil weather seed ([`crate::engine::test_harness`] weather arm).
    #[must_use]
    pub const fn seeds_weather(self) -> bool {
        matches!(
            self,
            Self::Weather | Self::Visual | Self::VfxSandbox | Self::SubFx
        )
    }

    /// Global wind + chunk wind field ([`crate::engine::test_harness`] atmosphere arm).
    #[must_use]
    pub const fn seeds_atmosphere(self) -> bool {
        matches!(
            self,
            Self::Atmosphere | Self::Visual | Self::VfxSandbox | Self::SubFx
        )
    }

    /// Empty town book + settlement count so city generation can run.
    #[must_use]
    pub const fn seeds_towns(self) -> bool {
        matches!(self, Self::City)
    }

    /// Fire-only and subordinate-field play must not call [`crate::strategic::settlement::seed_settlement_books_if_empty`].
    #[must_use]
    pub const fn suppresses_town_book_seed(self) -> bool {
        matches!(self, Self::Vfx | Self::SubFx)
    }

    /// Fixed world-gen seed for an isolation play entry. `None` leaves the harness default.
    #[must_use]
    pub const fn isolation_world_seed(self) -> Option<u64> {
        match self {
            Self::City => Some(crate::strategic::settlement::DEFAULT_WORLD_SEED),
            // Same number as [`super::debug_maneuver::UnittestWorldFixture`] default seed.
            Self::Vfx => Some(42),
            // Distinct from the city book seed and the fire fixture seed so this world is not the city map.
            Self::SubFx => Some(1_603),
            _ => None,
        }
    }

    #[must_use]
    pub const fn menu_vfx_bootstrap(self) -> bool {
        !matches!(self, Self::None | Self::City)
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
            // Fire / fire-VFX only. Combined sandbox stays on `sandbox` / `fireweather`.
            "vfx" => {
                args.test_scene = TestScene::Vfx;
                args.maneuver = DebugManeuver::DemoOpen;
                args.visual_auto_exit = false;
            }
            "sandbox" | "fireweather" => {
                args.test_scene = TestScene::VfxSandbox;
                args.maneuver = DebugManeuver::DemoOpen;
                args.visual_auto_exit = false;
            }
            // Settlements and town growth. Empty town book may still be seeded. No VFX menu bootstrap.
            "city" => {
                args.test_scene = TestScene::City;
                args.maneuver = DebugManeuver::DemoOpen;
                args.visual_auto_exit = false;
            }
            // Weather + atmosphere only. CLI name `subfx` (menu label: Weather + atmosphere).
            "subfx" => {
                args.test_scene = TestScene::SubFx;
                args.maneuver = DebugManeuver::DemoOpen;
                args.visual_auto_exit = false;
            }
            other => {
                bevy::log::warn!(
                    "Unknown --test mode {other:?}; isolation commands: \
                     `city`, `vfx`, `subfx`. Combined sandbox: `sandbox`/`fireweather`. \
                     Capture: `visual`. Also `frame`, `demo`, `unittest`, `weather`, `fire`, `atmosphere`."
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

    /// True when CLI/menu `--test` would seed fire via harness (`apply_test_scene_fire_seeds`).
    ///
    /// **Debug fallback only** — forbidden as sole product fire proof (VSS-T2-001).
    /// Product paths: [`SpectatorEntryPath::Scenario`], [`SpectatorEntryPath::Save`],
    /// [`SpectatorEntryPath::Interactive`].
    #[must_use]
    pub fn harness_fire_is_debug_fallback(&self) -> bool {
        self.test_scene.seeds_fire_overlay()
    }

    /// Entry path when the active session is a CLI/menu test harness world.
    #[must_use]
    pub fn test_harness_entry_path(&self) -> Option<SpectatorEntryPath> {
        if self.test_mode() {
            Some(SpectatorEntryPath::Harness)
        } else {
            None
        }
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
                | TestScene::City
                | TestScene::Vfx
                | TestScene::SubFx
        ) || self.full_capture_active();
        let flush_secs = if self.full_capture_active() { 2.0 } else { 5.0 };
        let frame_jsonl_stride = match self.test_scene {
            TestScene::Visual => 30,
            TestScene::VfxSandbox | TestScene::City | TestScene::Vfx | TestScene::SubFx => 10,
            TestScene::Weather | TestScene::Fire | TestScene::Atmosphere => 15,
            TestScene::None => 1,
        };
        let stall_spans = matches!(
            self.test_scene,
            TestScene::VfxSandbox
                | TestScene::City
                | TestScene::Vfx
                | TestScene::SubFx
                | TestScene::Weather
                | TestScene::Fire
                | TestScene::Atmosphere
        ) || self.full_capture_active();
        TestInstrumentationProfile {
            active: true,
            quiet_terminal: true,
            frame_jsonl,
            stall_spans,
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
        let vfx = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None);
        assert_eq!(vfx.test_scene, TestScene::Vfx);
        assert_eq!(vfx.maneuver, DebugManeuver::DemoOpen);
        assert!(!vfx.visual_auto_exit);
        let sandbox = EngineLaunchArgs::from_cli(Some("sandbox".into()), false, None);
        let fireweather = EngineLaunchArgs::from_cli(Some("fireweather".into()), false, None);
        assert_eq!(sandbox.test_scene, TestScene::VfxSandbox);
        assert_eq!(fireweather.test_scene, TestScene::VfxSandbox);
        assert_eq!(sandbox.maneuver, DebugManeuver::DemoOpen);
        assert!(!sandbox.visual_auto_exit);
    }

    #[test]
    fn play_isolation_scenes_seed_only_their_stack() {
        let city = EngineLaunchArgs::from_cli(Some("city".into()), false, None);
        let vfx = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None);
        let subfx = EngineLaunchArgs::from_cli(Some("subfx".into()), false, None);
        let sandbox = EngineLaunchArgs::from_cli(Some("sandbox".into()), false, None);

        assert_eq!(city.test_scene, TestScene::City);
        assert_eq!(city.maneuver, DebugManeuver::DemoOpen);
        assert!(!city.visual_auto_exit);
        assert!(!city.test_scene.seeds_fire_overlay());
        assert!(!city.test_scene.seeds_weather());
        assert!(!city.test_scene.seeds_atmosphere());
        assert!(city.test_scene.seeds_towns());
        assert!(!city.test_scene.suppresses_town_book_seed());
        assert!(!city.test_scene.menu_vfx_bootstrap());

        assert_eq!(vfx.test_scene, TestScene::Vfx);
        assert_eq!(vfx.maneuver, DebugManeuver::DemoOpen);
        assert!(!vfx.visual_auto_exit);
        assert!(vfx.test_scene.seeds_fire_overlay());
        assert!(!vfx.test_scene.seeds_weather());
        assert!(!vfx.test_scene.seeds_atmosphere());
        assert!(!vfx.test_scene.seeds_towns());
        assert!(vfx.test_scene.suppresses_town_book_seed());

        assert_eq!(subfx.test_scene, TestScene::SubFx);
        assert_eq!(subfx.test_scene.menu_label(), "Weather + atmosphere");
        assert_eq!(subfx.maneuver, DebugManeuver::DemoOpen);
        assert!(!subfx.visual_auto_exit);
        assert!(subfx.test_scene.seeds_weather());
        assert!(subfx.test_scene.seeds_atmosphere());
        assert!(!subfx.test_scene.seeds_fire_overlay());
        assert!(!subfx.test_scene.seeds_towns());
        assert!(subfx.test_scene.suppresses_town_book_seed());

        assert_eq!(sandbox.test_scene, TestScene::VfxSandbox);
        assert!(sandbox.test_scene.seeds_fire_overlay());
        assert!(sandbox.test_scene.seeds_weather());
        assert!(sandbox.test_scene.seeds_atmosphere());

        let city_seed = city.test_scene.isolation_world_seed();
        let vfx_seed = vfx.test_scene.isolation_world_seed();
        let subfx_seed = subfx.test_scene.isolation_world_seed();
        assert!(city_seed.is_some() && vfx_seed.is_some() && subfx_seed.is_some());
        assert_ne!(city_seed, vfx_seed);
        assert_ne!(vfx_seed, subfx_seed);
        assert_ne!(city_seed, subfx_seed);
    }

    #[test]
    fn cli_city_stays_open_like_vfx() {
        let city = EngineLaunchArgs::from_cli(Some("city".into()), false, None);
        let vfx = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None);
        assert_eq!(city.test_scene, TestScene::City);
        assert_eq!(city.maneuver, DebugManeuver::DemoOpen);
        assert_eq!(city.maneuver, vfx.maneuver);
        assert!(!city.visual_auto_exit);
        assert_eq!(city.visual_auto_exit, vfx.visual_auto_exit);
        assert!(city.test_mode());
        assert!(!city.test_scene.seeds_fire_overlay());
        assert!(!city.sim_enter_uses_tactical_camera_zoom());
    }

    #[test]
    fn cli_atmosphere_maps() {
        let a = EngineLaunchArgs::from_cli(Some("atmosphere".into()), false, None);
        assert_eq!(a.test_scene, TestScene::Atmosphere);
        assert!(a.test_mode());
    }

    #[test]
    fn harness_fire_seeds_are_debug_fallback_not_product_proof() {
        let fire = EngineLaunchArgs::from_cli(Some("fire".into()), false, None);
        assert!(fire.harness_fire_is_debug_fallback());
        assert_eq!(
            fire.test_harness_entry_path(),
            Some(SpectatorEntryPath::Harness)
        );
        let weather = EngineLaunchArgs::from_cli(Some("weather".into()), false, None);
        assert!(!weather.harness_fire_is_debug_fallback());
        assert_eq!(weather.test_harness_entry_path(), Some(SpectatorEntryPath::Harness));
        let default = EngineLaunchArgs::default();
        assert!(!default.harness_fire_is_debug_fallback());
        assert_eq!(default.test_harness_entry_path(), None);
    }

    #[test]
    fn test_instrumentation_profile_demo_no_stall_spans() {
        let a = EngineLaunchArgs::from_cli(Some("demo".into()), false, None);
        let p = a.test_instrumentation_profile();
        assert!(p.active);
        assert!(p.quiet_terminal);
        assert!(!p.stall_spans, "P3-C: demo uses disk witness only — no implicit STALL spans");
    }

    #[test]
    fn test_instrumentation_profile_visual_full_capture_stall_spans() {
        let a = EngineLaunchArgs::from_cli(Some("visual".into()), false, None);
        assert!(a.full_capture_active());
        assert!(a.test_instrumentation_profile().stall_spans);
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
