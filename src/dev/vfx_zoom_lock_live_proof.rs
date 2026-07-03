//! P0-VFX-ZOOM-LOCK-001 — VfxSandbox scroll zoom must not hard-lock to tactical band.

pub const VFX_ZOOM_LOCK_LIVE_JSON: &str = "debug_runs/vfx_zoom_lock_live.json";

#[must_use]
pub fn vfx_zoom_lock_001_witness_json() -> serde_json::Value {
    use crate::engine::{EngineLaunchArgs, TestScene};
    use crate::render::stage5_full_app_harness::{
        tactical_vfx_hard_lock_enabled, vfx_sandbox_scroll_zoom_free,
        visual_tactical_vfx_camera_lock_enabled,
    };

    let vfx_launch = EngineLaunchArgs {
        test_scene: TestScene::VfxSandbox,
        ..EngineLaunchArgs::default()
    };

    let scroll_free = vfx_sandbox_scroll_zoom_free(Some(&vfx_launch));
    let hard_lock = visual_tactical_vfx_camera_lock_enabled();
    serde_json::json!({
        "gate": "P0-VFX-ZOOM-LOCK-001",
        "green": scroll_free && !hard_lock,
        "vfx_sandbox_scroll_free": scroll_free,
        "visual_test_hard_lock": hard_lock,
        "tactical_vfx_hard_lock_env": tactical_vfx_hard_lock_enabled(),
    })
}

#[must_use]
pub fn refresh_vfx_zoom_lock_live_witness() -> bool {
    let body = vfx_zoom_lock_001_witness_json();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "P0-VFX-ZOOM-LOCK-001",
        "refresh_vfx_zoom_lock_live_witness",
        VFX_ZOOM_LOCK_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VFX_ZOOM_LOCK_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vfx_zoom_lock_live_witness_refresh_green() {
        assert!(refresh_vfx_zoom_lock_live_witness());
    }
}
