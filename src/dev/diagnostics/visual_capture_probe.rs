//! **VISUAL-CAPTURE-PROBE-001** — real GPU-pixel capture for tactical-map render regressions.
//!
//! Env-gated (`VFX_CAPTURE=1`) one-shot capture batch fired 240 frames into
//! [`BaseState::Simulation`]. Unlike [`crate::render::probes::vfx_capture_hook`] (text-stub,
//! ignored), this probe spawns real [`Screenshot`] entities against the primary window and the
//! tactical map RTT image, and dumps a metadata JSON describing what was captured. Intended for
//! human/agent visual diagnosis of "tactical map renders nothing" — actual pixels on disk, not
//! witness JSON claims.
//!
//! ## Why 240 frames
//!
//! Bootstrap (camera spawn, RTT bind barrier, first layout measure) takes several frames; 240
//! gives a comfortable steady-state margin before we grab pixels.
//!
//! ## Files written
//!
//! - `debug_runs/captures/window_live.png` — primary window swapchain capture.
//! - `debug_runs/captures/tactical_rtt_live.png` — tactical map RTT image capture (if bound).
//! - `debug_runs/captures/capture_meta_live.json` — frame/camera/window metadata for correlation.

use std::path::Path;

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::PrimaryWindow;

use crate::engine::states::BaseState;
use crate::gui::tactical::map_camera::MainWorldCamera;
use crate::gui::tactical::sim_map_rtt::SimulationMapTexture;

/// Default: fire the capture batch this many frames after entering [`BaseState::Simulation`].
const CAPTURE_AT_FRAME: u32 = 240;

/// Capture frame, overridable via `VFX_CAPTURE_FRAME=<n>` (e.g. 600 to capture after the vfx
/// harness seeds fire at ~frame 330).
fn capture_at_frame() -> u32 {
    std::env::var("VFX_CAPTURE_FRAME")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(CAPTURE_AT_FRAME)
}

const CAPTURE_DIR: &str = "debug_runs/captures";
const WINDOW_CAPTURE_PATH: &str = "debug_runs/captures/window_live.png";
const TACTICAL_RTT_CAPTURE_PATH: &str = "debug_runs/captures/tactical_rtt_live.png";
const CAPTURE_META_PATH: &str = "debug_runs/captures/capture_meta_live.json";

#[must_use]
pub fn visual_capture_probe_enabled() -> bool {
    std::env::var("VFX_CAPTURE").is_ok()
}

/// Frame counter (frames spent in [`BaseState::Simulation`]) + fire-once latch.
#[derive(Resource, Debug, Default)]
pub struct VisualCaptureProbeState {
    pub frames_in_simulation: u32,
    pub fired: bool,
}

/// Count frames spent in [`BaseState::Simulation`] while the probe is armed.
pub fn tick_visual_capture_probe_frames_system(mut state: ResMut<VisualCaptureProbeState>) {
    if state.fired {
        return;
    }
    state.frames_in_simulation = state.frames_in_simulation.saturating_add(1);
}

/// Reset the counter/latch on (re)entry to [`BaseState::Simulation`] — matches sibling witness
/// resets in this module (see [`super::subscribers::reset_witnesses_on_enter_simulation`]).
pub fn reset_visual_capture_probe_on_enter_simulation(mut state: ResMut<VisualCaptureProbeState>) {
    *state = VisualCaptureProbeState::default();
}

/// Fire the one-shot capture batch once [`CAPTURE_AT_FRAME`] is reached.
///
/// Spawns [`Screenshot`] entities (window + tactical RTT, if bound) with `save_to_disk`
/// observers, and writes a metadata JSON snapshot. No panics on absent window/RTT — logs a
/// `warn!` and skips that capture leg gracefully.
pub fn fire_visual_capture_batch_system(
    mut commands: Commands,
    mut state: ResMut<VisualCaptureProbeState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rtt_tex: Option<Res<SimulationMapTexture>>,
    images: Res<Assets<Image>>,
    cameras: Query<(&Camera, &RenderTarget), With<MainWorldCamera>>,
) {
    if state.fired || state.frames_in_simulation < capture_at_frame() {
        return;
    }
    state.fired = true;

    if let Err(e) = std::fs::create_dir_all(CAPTURE_DIR) {
        warn!(
            target: "visual_capture_probe",
            "VISUAL-CAPTURE-PROBE-001: failed to create {CAPTURE_DIR}: {e}"
        );
        return;
    }

    // (a) window screenshot.
    let window_ok = windows.single().is_ok();
    if window_ok {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(Path::new(WINDOW_CAPTURE_PATH)));
        info!(
            target: "visual_capture_probe",
            "VISUAL-CAPTURE-PROBE-001: window capture queued -> {WINDOW_CAPTURE_PATH}"
        );
    } else {
        warn!(
            target: "visual_capture_probe",
            "VISUAL-CAPTURE-PROBE-001: no primary window found, skipping window capture"
        );
    }

    // (b) tactical RTT capture.
    let rtt_handle_debug = match rtt_tex.as_deref() {
        Some(tex) => {
            commands
                .spawn(Screenshot::image(tex.0.clone()))
                .observe(save_to_disk(Path::new(TACTICAL_RTT_CAPTURE_PATH)));
            info!(
                target: "visual_capture_probe",
                "VISUAL-CAPTURE-PROBE-001: tactical RTT capture queued -> {TACTICAL_RTT_CAPTURE_PATH}"
            );
            format!("{:?}", tex.0)
        }
        None => {
            warn!(
                target: "visual_capture_probe",
                "VISUAL-CAPTURE-PROBE-001: SimulationMapTexture resource absent, skipping RTT capture"
            );
            "absent".to_string()
        }
    };

    // (c) metadata JSON. RenderAdapterInfo lives only in the RenderApp sub-app in Bevy 0.19
    // (inserted via the RenderResources bundle in renderer::init) and is not extracted into
    // MainWorld by this crate — skipped cleanly, field omitted rather than guessed.
    let window_resolution = windows
        .single()
        .map(|w| (w.width(), w.height()))
        .unwrap_or((0.0, 0.0));

    let camera_summary: Vec<serde_json::Value> = cameras
        .iter()
        .map(|(cam, target)| {
            serde_json::json!({
                "is_active": cam.is_active,
                "order": cam.order,
                "target_debug": format!("{target:?}"),
            })
        })
        .collect();

    let rtt_image_extent = rtt_tex.as_deref().and_then(|tex| {
        images
            .get(&tex.0)
            .map(|img| (img.width(), img.height()))
    });

    let meta = serde_json::json!({
        "schema": "visual_capture_probe_v1",
        "frame_in_simulation": state.frames_in_simulation,
        "capture_at_frame": capture_at_frame(),
        "window_resolution": { "w": window_resolution.0, "h": window_resolution.1 },
        "rtt_handle_debug": rtt_handle_debug,
        "rtt_image_extent": rtt_image_extent.map(|(w, h)| serde_json::json!({ "w": w, "h": h })),
        "main_world_camera_summary": camera_summary,
        "window_capture_path": WINDOW_CAPTURE_PATH,
        "tactical_rtt_capture_path": TACTICAL_RTT_CAPTURE_PATH,
    });

    match serde_json::to_string_pretty(&meta) {
        Ok(json) => {
            if let Err(e) = std::fs::write(CAPTURE_META_PATH, json) {
                warn!(
                    target: "visual_capture_probe",
                    "VISUAL-CAPTURE-PROBE-001: failed to write {CAPTURE_META_PATH}: {e}"
                );
            } else {
                info!(
                    target: "visual_capture_probe",
                    "VISUAL-CAPTURE-PROBE-001: metadata written -> {CAPTURE_META_PATH}"
                );
            }
        }
        Err(e) => {
            warn!(
                target: "visual_capture_probe",
                "VISUAL-CAPTURE-PROBE-001: failed to serialize capture metadata: {e}"
            );
        }
    }

    info!(
        target: "visual_capture_probe",
        "VISUAL-CAPTURE-PROBE-001: capture batch fired at frame {} (in-simulation)",
        state.frames_in_simulation
    );
}

pub struct VisualCaptureProbePlugin;

impl Plugin for VisualCaptureProbePlugin {
    fn build(&self, app: &mut App) {
        if !visual_capture_probe_enabled() {
            return;
        }
        app.init_resource::<VisualCaptureProbeState>()
            .add_systems(
                OnEnter(BaseState::Simulation),
                reset_visual_capture_probe_on_enter_simulation,
            )
            .add_systems(
                Update,
                (
                    tick_visual_capture_probe_frames_system,
                    fire_visual_capture_batch_system,
                )
                    .chain()
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_state_defaults_unarmed() {
        let state = VisualCaptureProbeState::default();
        assert_eq!(state.frames_in_simulation, 0);
        assert!(!state.fired);
    }

    #[test]
    fn capture_paths_land_under_debug_runs_captures() {
        assert!(WINDOW_CAPTURE_PATH.starts_with(CAPTURE_DIR));
        assert!(TACTICAL_RTT_CAPTURE_PATH.starts_with(CAPTURE_DIR));
        assert!(CAPTURE_META_PATH.starts_with(CAPTURE_DIR));
    }
}
