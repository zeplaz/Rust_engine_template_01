//! **TRIAGE-FIRE-PLAY-VIS-001** — refresh `debug_runs/design_fire_play_visibility_live.json`.

pub const DESIGN_FIRE_PLAY_VISIBILITY_LIVE_JSON: &str =
    "debug_runs/design_fire_play_visibility_live.json";

#[must_use]
pub fn refresh_design_fire_play_visibility_live_witness() -> bool {
    use crate::systems::fire::{
        triage_fire_play_vis_001_green, triage_fire_play_vis_001_witness_json,
        TriageFirePlayVis001Inputs,
    };
    use crate::engine::launch_args::TestScene;

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
    if !triage_fire_play_vis_001_green(&inputs) {
        return false;
    }
    let body = triage_fire_play_vis_001_witness_json(&inputs);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "TRIAGE-FIRE-PLAY-VIS-001",
        "refresh_design_fire_play_visibility_live_witness",
        DESIGN_FIRE_PLAY_VISIBILITY_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        DESIGN_FIRE_PLAY_VISIBILITY_LIVE_JSON,
        wrapped,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_fire_play_visibility_live_witness_refresh_green() {
        assert!(refresh_design_fire_play_visibility_live_witness());
    }
}
