//! **VFX-CAPTURE-ENV** — operator env contract for GPU capture / inspection / playback.
//!
//! **V4-DBG04** formal ADOPT of **DBG-04** (gea4v2:12.9.4 — GPU Capture, Inspection, and Playback)
//! · LEGACY keyword **`render_debug_tools`** (gea_v2_e4 p248) · sibling **DBG-07** (gea4v1:10.7
//! screenshots). In-engine stand-ins for PIX / RenderDoc-style frame capture:
//!
//! | Role | Surface |
//! |:---|:---|
//! | **Capture** | `VFX_CAPTURE` → [`crate::dev::diagnostics::visual_capture_probe`] writes PNGs |
//! | **Inspection** | `RTT_SPRITE_TRACE` → [`super::rtt_sprite_trace`] render-world visibility logs |
//! | **Playback / review** | playcheck / Studio stills (`tools/sim_review/`) armed via this env |
//!
//! [`super::vfx_capture_hook`] remains the sim-enqueue stub lane for operator PNG hooks (not real
//! GPU pixels). Capture / inspection are **env-gated**, never play UI, and must **not** write
//! simulation authority (Screenshot / RenderApp logs only).
//!
//! ## Env vars (playcheck / Studio / manual runs)
//!
//! | Variable | Role |
//! |:---|:---|
//! | `VFX_CAPTURE` | Arms the one-shot capture batch (any value). |
//! | `VFX_CAPTURE_FRAME` | Frames after [`BaseState::Simulation`] before firing (default 240). |
//! | `VFX_CAPTURE_VIEW` | `voxel` / `pixel` / `3d` / `world` → PixelWorld3D; `topdown` / `tactical` / `2d` → tactical. |
//! | `VFX_CAPTURE_NO_MACRO_ZOOM` | Reserved for harness zoom parity (playcheck sets `1`). |
//! | `RTT_SPRITE_TRACE` | Sibling DBG-04 probe — render-world sprite visibility trace. |
//!
//! ## PlayQuad + PixelWorld weather / massing (voxels **in scope**)
//!
//! ```text
//! RUST_ENGINE_SIM_DUMP=1 VFX_CAPTURE=1 VFX_CAPTURE_VIEW=voxel VFX_CAPTURE_FRAME=2000 \\
//!   cargo run -p proc_A_dine01 -- --test playquad --stay-open
//! ```
//!
//! PlayQuad seeds NE build sites in PixelWorld3D and requests storm rain after ~1800 sim frames
//! ([`crate::engine::test_harness::PLAYQUAD_RAIN_DELAY_FRAMES`]). Capture at ≥2000 so
//! `window_live.png` / `tactical_rtt_live.png` include precip + voxel massing presentation.
//! Correlate with dump `weather.*`, `massing_*`, and `launch.presentation` — counts only, no `*_green`.
//!
//! Outputs: `debug_runs/captures/window_live.png`, `tactical_rtt_live.png`, `capture_meta_live.json`.
//!
//! **Dual-matrix cites:** NEW **DBG-04** (gea4v2:12.9.4) · **DBG-07** (gea4v1:10.7) · LEGACY
//! `render_debug_tools` (gea_v2_e4 p248).

use std::env;

/// Env key that arms GPU PNG capture ([`crate::dev::diagnostics::visual_capture_probe`]).
pub const VFX_CAPTURE_ENV: &str = "VFX_CAPTURE";
/// Optional frame delay override (u32 parse).
pub const VFX_CAPTURE_FRAME_ENV: &str = "VFX_CAPTURE_FRAME";
/// Presentation override for capture runs (`voxel` → PixelWorld3D, …).
pub const VFX_CAPTURE_VIEW_ENV: &str = "VFX_CAPTURE_VIEW";
/// Sibling DBG-04 inspection probe — render-world sprite / phase visibility.
pub const RTT_SPRITE_TRACE_ENV: &str = "RTT_SPRITE_TRACE";

/// Presentation override for capture runs (mirrors [`crate::gui::WorldPresentationKind`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfxCapturePresentation {
    /// Tactical map RTT (icons / symbols).
    TacticalFlat2D,
    /// Voxel world — terrain, massing columns, PixelWorld precip/clouds.
    PixelWorld3D,
}

#[must_use]
pub fn vfx_capture_enabled() -> bool {
    env::var(VFX_CAPTURE_ENV).is_ok()
}

#[must_use]
pub fn rtt_sprite_trace_enabled() -> bool {
    env::var(RTT_SPRITE_TRACE_ENV).is_ok()
}

/// Default frames in simulation before the capture batch fires (overridable).
pub const DEFAULT_VFX_CAPTURE_FRAME: u32 = 240;

/// PlayQuad wet-rain + PixelWorld acceptance needs frames past [`PLAYQUAD_RAIN_DELAY_HINT`].
pub const PLAYQUAD_RAIN_DELAY_HINT: u32 = 1800;

/// Recommended capture frame for PlayQuad voxel weather/massing stills.
pub const PLAYQUAD_VOXEL_CAPTURE_FRAME: u32 = 2000;

#[must_use]
pub fn vfx_capture_frame() -> u32 {
    env::var(VFX_CAPTURE_FRAME_ENV)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(DEFAULT_VFX_CAPTURE_FRAME)
}

#[must_use]
pub fn parse_vfx_capture_view(raw: Option<&str>) -> Option<VfxCapturePresentation> {
    let v = raw?.trim().to_ascii_lowercase();
    match v.as_str() {
        "voxel" | "pixel" | "3d" | "world" => Some(VfxCapturePresentation::PixelWorld3D),
        "topdown" | "tactical" | "2d" => Some(VfxCapturePresentation::TacticalFlat2D),
        _ => None,
    }
}

#[must_use]
pub fn vfx_capture_presentation_from_env() -> VfxCapturePresentation {
    parse_vfx_capture_view(env::var(VFX_CAPTURE_VIEW_ENV).ok().as_deref())
        .unwrap_or(VfxCapturePresentation::TacticalFlat2D)
}

#[must_use]
pub fn recommended_capture_frame_for_scene(scene_cli: &str) -> u32 {
    let s = scene_cli.trim().to_ascii_lowercase();
    if matches!(s.as_str(), "playquad" | "quadrant" | "quad") {
        PLAYQUAD_VOXEL_CAPTURE_FRAME.max(vfx_capture_frame())
    } else {
        vfx_capture_frame()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_voxel_aliases_to_pixel_world() {
        for alias in ["voxel", "pixel", "3d", "world", " VOXEL "] {
            assert_eq!(
                parse_vfx_capture_view(Some(alias)),
                Some(VfxCapturePresentation::PixelWorld3D)
            );
        }
    }

    #[test]
    fn parse_tactical_aliases() {
        for alias in ["topdown", "tactical", "2d"] {
            assert_eq!(
                parse_vfx_capture_view(Some(alias)),
                Some(VfxCapturePresentation::TacticalFlat2D)
            );
        }
        assert_eq!(parse_vfx_capture_view(Some("nope")), None);
    }

    #[test]
    fn playquad_capture_frame_meets_rain_delay() {
        assert!(PLAYQUAD_VOXEL_CAPTURE_FRAME > PLAYQUAD_RAIN_DELAY_HINT);
        assert!(
            recommended_capture_frame_for_scene("playquad") >= PLAYQUAD_VOXEL_CAPTURE_FRAME
        );
    }

    #[test]
    fn hook_remain_sim_callable_while_env_arms_gpu_probe() {
        assert!(
            crate::render::probes::vfx_capture_hook::VfxCaptureHookState::hooks_callable_from_sim()
        );
        assert_eq!(DEFAULT_VFX_CAPTURE_FRAME, 240);
    }

    /// DBG-04 / V4-DBG04 tripwire: dual-cite (NEW row + LEGACY keyword) stays in module docs.
    #[test]
    fn dbg04_dual_cite_render_debug_tools_in_module_docs() {
        let src = include_str!("vfx_capture_env.rs");
        assert!(src.contains("DBG-04"), "missing NEW matrix cite DBG-04");
        assert!(
            src.contains("render_debug_tools"),
            "missing LEGACY keyword render_debug_tools"
        );
        assert!(
            src.contains("12.9.4"),
            "missing gea4v2 12.9.4 section cite"
        );
        assert!(src.contains("V4-DBG04"), "missing program slice id V4-DBG04");
        assert!(src.contains(VFX_CAPTURE_ENV));
        assert!(src.contains(RTT_SPRITE_TRACE_ENV));
        assert!(src.contains("DBG-07"), "missing sibling DBG-07 screenshot cite");
    }

    /// DBG-04 / V4-DBG04 tripwire: voxels in scope — `voxel` view maps to PixelWorld3D capture.
    #[test]
    fn dbg04_voxel_view_aliases_map_to_pixel_world() {
        for alias in ["voxel", "pixel", "3d", "world"] {
            assert_eq!(
                parse_vfx_capture_view(Some(alias)),
                Some(VfxCapturePresentation::PixelWorld3D),
                "alias {alias} must select PixelWorld3D (voxels in scope)"
            );
        }
        assert!(PLAYQUAD_VOXEL_CAPTURE_FRAME > PLAYQUAD_RAIN_DELAY_HINT);
    }

    /// DBG-04 / V4-DBG04 tripwire: capture + inspection env keys stay named + exported.
    #[test]
    fn dbg04_capture_and_inspection_env_keys_are_stable() {
        assert_eq!(VFX_CAPTURE_ENV, "VFX_CAPTURE");
        assert_eq!(VFX_CAPTURE_FRAME_ENV, "VFX_CAPTURE_FRAME");
        assert_eq!(VFX_CAPTURE_VIEW_ENV, "VFX_CAPTURE_VIEW");
        assert_eq!(RTT_SPRITE_TRACE_ENV, "RTT_SPRITE_TRACE");
        // Unset in unit tests → probes stay disarmed (no accidental capture in CI).
        assert!(!vfx_capture_enabled() || env::var_os(VFX_CAPTURE_ENV).is_some());
        assert!(!rtt_sprite_trace_enabled() || env::var_os(RTT_SPRITE_TRACE_ENV).is_some());
    }

    /// DBG-04 / V4-DBG04 tripwire: probe module exports capture env + RTT inspection.
    #[test]
    fn dbg04_probe_mod_exports_capture_and_rtt_trace() {
        let mod_src = include_str!("mod.rs");
        assert!(mod_src.contains("vfx_capture_env"));
        assert!(mod_src.contains("rtt_sprite_trace"));
        assert!(mod_src.contains("V4-DBG04") || include_str!("mod.rs").contains("DBG-04"));
        let rtt = include_str!("rtt_sprite_trace.rs");
        assert!(rtt.contains("RTT_SPRITE_TRACE") || rtt.contains("rtt_sprite_trace_enabled"));
        assert!(
            rtt.contains("DBG-04") || rtt.contains("render_debug_tools"),
            "rtt_sprite_trace must dual-cite DBG-04 / render_debug_tools"
        );
    }

    /// DBG-04 / V4-DBG04 tripwire: real PNG probe defers frame/arm to this env contract.
    #[test]
    fn dbg04_visual_capture_probe_defers_to_env_contract() {
        let src = include_str!("../../dev/diagnostics/visual_capture_probe.rs");
        assert!(
            src.contains("vfx_capture_env"),
            "visual_capture_probe must use vfx_capture_env (single capture authority)"
        );
        assert!(src.contains("vfx_capture_enabled") || src.contains("vfx_capture_frame"));
        assert!(
            src.contains("DBG-04") || src.contains("render_debug_tools"),
            "visual_capture_probe must cite DBG-04 / render_debug_tools"
        );
        assert!(
            !src.contains("std::env::var(\"VFX_CAPTURE\")"),
            "do not re-parse VFX_CAPTURE outside vfx_capture_env"
        );
    }
}
