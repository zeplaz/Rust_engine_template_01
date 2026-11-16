//! # RND-19 — Double/triple buffering, v-sync (V4-RND19)
//!
//! Dual-cite: NEW **RND-19** (gea4v2 **11.3.4.1** *Double and Triple Buffering*
//! p50-51; gea4v1 **8.5.2.5** *Screen Tearing and V-Sync* p518-519) · LEGACY
//! book pages **gea_v2_e4** p50-51 · **gea_v1_e4** p518-519 (no dedicated
//! `vsync` / `double_buffer` keyword row in the legacy matrix — cite the PDF
//! anchors). Related ops latch: `PERF_NO_VSYNC` / `PERF_VSYNC` (K12 env-latch).
//!
//! ## Why present shows up as "stalls"
//!
//! Under vsync, `get_current_texture` blocks until a swapchain slot is free.
//! **Double** buffering (latency **2**) holds one front + one back — the CPU/GPU
//! wait on the vblank when both are occupied. **Triple** buffering (latency **3**)
//! adds a second back buffer so the GPU can keep rendering while the display
//! holds the front; stalls drop, input-to-photon latency rises. Uncapped present
//! (`AutoNoVsync`) prefers Immediate/Mailbox and is for perf probes only.
//!
//! ## Locked present / buffer contract (sole authority)
//!
//! | Latch | Present mode | `desired_maximum_frame_latency` | Policy |
//! |:---|:---|:---|:---|
//! | Default debug (vsync on) | [`PresentMode::AutoVsync`] | **3** (triple) | Tear-free, lower stall |
//! | `PERF_NO_VSYNC=1` / `PERF_VSYNC=0` / release | [`PresentMode::AutoNoVsync`] | **1** | Uncapped probe |
//!
//! Sole writer of primary-window present config: `EnginePlugin` in
//! `engine/engine_with_worldgen.rs` via [`resolve_present_surface_config`].
//! Preview/assembly heads stay on fixed `AutoNoVsync` (not play path).
//!
//! Debt not taken: runtime UI toggle · per-view swapchains · forcing `Fifo` /
//! `Mailbox` over Auto\* · dual-writing `Window.present_mode` from systems.
//!
//! Tripwires: `rnd19_*` (no GPU).

use std::num::NonZeroU32;

use bevy::window::PresentMode;

/// Env latch that forces uncapped present (ops / `--test` / release).
pub const PERF_NO_VSYNC_ENV: &str = "PERF_NO_VSYNC";
/// Alternate latch: `PERF_VSYNC=0|false` disables vsync (same as no-vsync).
pub const PERF_VSYNC_ENV: &str = "PERF_VSYNC";

/// GEA double-buffer queue depth (front + one back). Documented; not the debug default.
pub const PRESENT_DOUBLE_BUFFER_LATENCY: u32 = 2;
/// GEA triple-buffer queue depth — debug vsync default (fewer present stalls).
pub const PRESENT_TRIPLE_BUFFER_LATENCY: u32 = 3;
/// Uncapped / probe path — minimize queued frames for honest GPU timing.
pub const PRESENT_UNCAPPED_BUFFER_LATENCY: u32 = 1;

/// Named buffer policy for tripwires and docs (maps 1:1 to latency constants).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresentBufferPolicy {
    /// Latency 2 — classic double buffer (more vsync stall risk).
    Double,
    /// Latency 3 — triple buffer under vsync (locked debug default).
    Triple,
    /// Latency 1 — uncapped / perf probe.
    Uncapped,
}

impl PresentBufferPolicy {
    #[inline]
    #[must_use]
    pub const fn frame_latency(self) -> u32 {
        match self {
            Self::Double => PRESENT_DOUBLE_BUFFER_LATENCY,
            Self::Triple => PRESENT_TRIPLE_BUFFER_LATENCY,
            Self::Uncapped => PRESENT_UNCAPPED_BUFFER_LATENCY,
        }
    }
}

/// Committed primary-window present / swapchain hints (immutable at plugin build).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentSurfaceConfig {
    pub present_mode: PresentMode,
    pub buffer_policy: PresentBufferPolicy,
}

impl PresentSurfaceConfig {
    /// wgpu / Bevy `Window.desired_maximum_frame_latency` for this policy.
    #[inline]
    #[must_use]
    pub fn desired_maximum_frame_latency(self) -> NonZeroU32 {
        NonZeroU32::new(self.buffer_policy.frame_latency()).expect("buffer latency ≥ 1")
    }
}

/// `true` when env / release latch requests uncapped present (no vsync wait).
#[inline]
#[must_use]
pub fn env_requests_no_vsync(
    perf_no_vsync: Option<&str>,
    perf_vsync: Option<&str>,
    release_build: bool,
) -> bool {
    if release_build {
        return true;
    }
    if perf_no_vsync.is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")) {
        return true;
    }
    perf_vsync.is_some_and(|v| v == "0" || v.eq_ignore_ascii_case("false"))
}

/// Resolve the locked present mode + buffer-count contract (RND-19 sole resolver).
#[inline]
#[must_use]
pub fn resolve_present_surface_config(
    perf_no_vsync: Option<&str>,
    perf_vsync: Option<&str>,
    release_build: bool,
) -> PresentSurfaceConfig {
    if env_requests_no_vsync(perf_no_vsync, perf_vsync, release_build) {
        PresentSurfaceConfig {
            present_mode: PresentMode::AutoNoVsync,
            buffer_policy: PresentBufferPolicy::Uncapped,
        }
    } else {
        PresentSurfaceConfig {
            present_mode: PresentMode::AutoVsync,
            buffer_policy: PresentBufferPolicy::Triple,
        }
    }
}

/// Read process env + `cfg!(not(debug_assertions))` into the locked config.
#[inline]
#[must_use]
pub fn resolve_present_surface_config_from_env() -> PresentSurfaceConfig {
    resolve_present_surface_config(
        std::env::var(PERF_NO_VSYNC_ENV).ok().as_deref(),
        std::env::var(PERF_VSYNC_ENV).ok().as_deref(),
        cfg!(not(debug_assertions)),
    )
}

#[cfg(test)]
mod rnd19_present_tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_src(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel)
    }

    fn read(rel: &str) -> String {
        std::fs::read_to_string(repo_src(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    }

    /// RND-19 tripwire: buffer-count constants match GEA double/triple (+ uncapped probe).
    #[test]
    fn rnd19_buffer_count_contract_double_triple_uncapped() {
        assert_eq!(PRESENT_DOUBLE_BUFFER_LATENCY, 2);
        assert_eq!(PRESENT_TRIPLE_BUFFER_LATENCY, 3);
        assert_eq!(PRESENT_UNCAPPED_BUFFER_LATENCY, 1);
        assert_eq!(PresentBufferPolicy::Double.frame_latency(), 2);
        assert_eq!(PresentBufferPolicy::Triple.frame_latency(), 3);
        assert_eq!(PresentBufferPolicy::Uncapped.frame_latency(), 1);
        assert!(PRESENT_TRIPLE_BUFFER_LATENCY > PRESENT_DOUBLE_BUFFER_LATENCY);
        assert!(PRESENT_DOUBLE_BUFFER_LATENCY > PRESENT_UNCAPPED_BUFFER_LATENCY);
    }

    /// RND-19 tripwire: debug default is AutoVsync + triple latency (fewer present stalls).
    #[test]
    fn rnd19_debug_default_is_autovsync_triple_buffer() {
        let cfg = resolve_present_surface_config(None, None, false);
        assert_eq!(cfg.present_mode, PresentMode::AutoVsync);
        assert_eq!(cfg.buffer_policy, PresentBufferPolicy::Triple);
        assert_eq!(cfg.desired_maximum_frame_latency().get(), 3);
    }

    /// RND-19 tripwire: PERF_NO_VSYNC / PERF_VSYNC=0 / release → AutoNoVsync + uncapped latency.
    #[test]
    fn rnd19_perf_and_release_force_uncapped_no_vsync() {
        for (no_vsync, vsync, release) in [
            (Some("1"), None, false),
            (Some("true"), None, false),
            (None, Some("0"), false),
            (None, Some("false"), false),
            (None, None, true),
        ] {
            let cfg = resolve_present_surface_config(no_vsync, vsync, release);
            assert_eq!(
                cfg.present_mode,
                PresentMode::AutoNoVsync,
                "no_vsync={no_vsync:?} vsync={vsync:?} release={release}"
            );
            assert_eq!(cfg.buffer_policy, PresentBufferPolicy::Uncapped);
            assert_eq!(cfg.desired_maximum_frame_latency().get(), 1);
        }
        // Explicit vsync keep: PERF_VSYNC=1 must not disable when PERF_NO_VSYNC unset.
        let keep = resolve_present_surface_config(None, Some("1"), false);
        assert_eq!(keep.present_mode, PresentMode::AutoVsync);
        assert_eq!(keep.buffer_policy, PresentBufferPolicy::Triple);
    }

    /// RND-19 tripwire: engine plugin is the sole primary-window present writer.
    #[test]
    fn rnd19_engine_plugin_applies_present_surface_config() {
        let engine = read("engine/engine_with_worldgen.rs");
        assert!(
            engine.contains("resolve_present_surface_config_from_env")
                || engine.contains("resolve_present_surface_config"),
            "EnginePlugin must call RND-19 present resolver"
        );
        assert!(
            engine.contains("desired_maximum_frame_latency"),
            "EnginePlugin must set Window.desired_maximum_frame_latency (buffer count)"
        );
        assert!(
            engine.contains("present_mode"),
            "EnginePlugin must set Window.present_mode"
        );
        // No second production present_mode writer outside preview (preview is AutoNoVsync-only).
        let present_src = read("render/view_runtime/present.rs");
        assert!(
            present_src.contains("Sole writer") || present_src.contains("sole writer"),
            "present.rs must name sole writer"
        );
    }

    /// RND-19 tripwire: voxels-in-scope note — PixelWorld shares the primary window present path.
    #[test]
    fn rnd19_voxel_pixel_world_shares_primary_present_path() {
        // PixelWorld RTT is not a second swapchain — it composites into the same presented window.
        let cam = read("render/pixel_world/camera.rs");
        assert!(
            !cam.contains("PresentMode::"),
            "PixelWorld must not dual-write PresentMode (RND-19 sole present authority)"
        );
    }
}
