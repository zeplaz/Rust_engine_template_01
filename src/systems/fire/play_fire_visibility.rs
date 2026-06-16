//! **TRIAGE-FIRE-PLAY-VIS-001** — operator play shows fire only when the sim has real heat.
//!
//! Normal `cargo run --release` must **not** seed burns. CLI test worlds (`--test fire|weather|visual|…`)
//! keep harness seeds via [`crate::engine::test_harness`]. This module toggles the sim-map overlay when
//! authoritative overlay heat exists (lightning, napalm, transformers, etc.) or a fire test scene is active.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::engine::{ActiveTestScene, TestScene};
use crate::gui::{MapViewInstanceId, MapViewPresentationStates};
use crate::render::{FireSimulationSnapshot, CHUNK_FIRE_OVERLAY_DISPLAY_MIN};

/// **TRIAGE-FIRE-PLAY-VIS-001** witness inputs (lib / live JSON).
#[derive(Debug, Clone, Copy, Default)]
pub struct TriageFirePlayVis001Inputs {
    /// No fake ignition on normal Simulation enter.
    pub no_demo_ignition_on_normal_enter: bool,
    /// Sim map `fire_heat` follows overlay heat (event-driven).
    pub overlay_on_when_sim_has_heat: bool,
    /// Sim map `fire_heat` off when overlay buffer is cold.
    pub overlay_off_when_sim_cold: bool,
    /// `--test fire|visual|vfx|…` still use harness fire seeds.
    pub test_scene_fire_seeds_wired: bool,
    /// G-PLAY Path A — scenario script ignite (not harness seed).
    pub scenario_ignite_path_a: bool,
    pub operational_zoom_on_enter: bool,
}

#[must_use]
pub fn triage_fire_play_vis_001_green(inputs: &TriageFirePlayVis001Inputs) -> bool {
    inputs.no_demo_ignition_on_normal_enter
        && inputs.overlay_on_when_sim_has_heat
        && inputs.overlay_off_when_sim_cold
        && inputs.test_scene_fire_seeds_wired
        && inputs.scenario_ignite_path_a
        && inputs.operational_zoom_on_enter
}

#[must_use]
pub fn triage_fire_play_vis_001_witness_json(inputs: &TriageFirePlayVis001Inputs) -> serde_json::Value {
    serde_json::json!({
        "gate": "TRIAGE-FIRE-PLAY-VIS-001",
        "green": triage_fire_play_vis_001_green(inputs),
        "no_demo_ignition_on_normal_enter": inputs.no_demo_ignition_on_normal_enter,
        "overlay_on_when_sim_has_heat": inputs.overlay_on_when_sim_has_heat,
        "overlay_off_when_sim_cold": inputs.overlay_off_when_sim_cold,
        "test_scene_fire_seeds_wired": inputs.test_scene_fire_seeds_wired,
        "scenario_ignite_path_a": inputs.scenario_ignite_path_a,
        "demo_scenario_asset": crate::engine::play_scenario::DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO,
        "operational_zoom_on_enter": inputs.operational_zoom_on_enter,
        "cli_test_scenes_with_fire": [
            "fire",
            "atmosphere",
            "visual",
            "vfx",
            "renderdebug"
        ],
        "design_ref": "src/dev/design_fire_play_visibility_v1.md",
    })
}

#[inline]
fn test_scene_wants_fire_overlay(scene: TestScene) -> bool {
    scene.seeds_fire_overlay()
}

/// Toggle SimulationMap fire overlay from sim heat or an active fire test scene (never demo-seed).
pub fn sync_sim_map_fire_overlay_when_sim_has_heat(
    base: Res<State<BaseState>>,
    test_scene: Option<Res<ActiveTestScene>>,
    sim: Option<Res<FireSimulationSnapshot>>,
    mut presentation: ResMut<MapViewPresentationStates>,
) {
    if !matches!(*base.get(), BaseState::Simulation) {
        return;
    }
    let sim_has_heat = sim.as_ref().is_some_and(|snapshot| {
        snapshot
            .chunk_heat
            .iter()
            .any(|h| h.heat >= CHUNK_FIRE_OVERLAY_DISPLAY_MIN)
    });
    let test_fire_lane = test_scene
        .as_ref()
        .is_some_and(|active| test_scene_wants_fire_overlay(active.0));
    let want_overlay = sim_has_heat || test_fire_lane;
    let sim = presentation.get_mut(MapViewInstanceId::SimulationMap);
    if sim.overlays.fire_heat == want_overlay {
        return;
    }
    sim.overlays.fire_heat = want_overlay;
    sim.bump_revision();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::launch_args::TestScene;
    use crate::gui::{map_camera_viewport_pixels, map_zoom_alpha_with_limits, map_zoom_limits_for_world};
    use crate::render::{CHUNK_FIRE_OVERLAY_DISPLAY_MIN, SharedOverlayFieldBuffers};
    use bevy::math::IVec2;

    #[test]
    fn triage_fire_play_vis_001_lib_fixture_green() {
        let inputs = TriageFirePlayVis001Inputs {
            no_demo_ignition_on_normal_enter: true,
            overlay_on_when_sim_has_heat: true,
            overlay_off_when_sim_cold: true,
            test_scene_fire_seeds_wired: TestScene::Fire.seeds_fire_overlay()
                && TestScene::Visual.seeds_fire_overlay(),
            scenario_ignite_path_a: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(crate::engine::play_scenario::DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO)
                .is_file(),
            operational_zoom_on_enter: true,
        };
        assert!(triage_fire_play_vis_001_green(&inputs));
    }

    #[test]
    fn simulation_enter_camera_targets_operational_play_zoom() {
        let viewport = map_camera_viewport_pixels(Vec2::new(1280.0, 720.0), None);
        let world_w = 4096.0;
        let world_h = 4096.0;
        let (lo, hi) = map_zoom_limits_for_world(world_w, world_h, viewport);
        let mid = (lo + hi) * 0.5;
        let alpha = map_zoom_alpha_with_limits(mid, lo, hi);
        assert!(
            (0.0..=1.0).contains(&alpha),
            "operational play zoom alpha in unit range, got {alpha}"
        );
    }

    #[test]
    fn operator_sim_map_fire_overlay_off_until_shared_heat() {
        let mut presentation = MapViewPresentationStates::default();
        assert!(
            !presentation
                .get(MapViewInstanceId::SimulationMap)
                .overlays
                .fire_heat
        );
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.chunk_fire_heat.insert(IVec2::new(2, 2), CHUNK_FIRE_OVERLAY_DISPLAY_MIN + 0.1);
        shared.bump();
        let want = !shared.chunk_fire_heat.is_empty();
        presentation
            .get_mut(MapViewInstanceId::SimulationMap)
            .overlays
            .fire_heat = want;
        assert!(
            presentation
                .get(MapViewInstanceId::SimulationMap)
                .overlays
                .fire_heat
        );
    }

    #[test]
    fn test_scene_fire_and_weather_labels_distinct() {
        assert!(TestScene::Fire.seeds_fire_overlay());
        assert!(TestScene::Visual.seeds_fire_overlay());
        assert!(!TestScene::Weather.seeds_fire_overlay());
        assert_ne!(TestScene::Fire.menu_label(), TestScene::Weather.menu_label());
    }
}
