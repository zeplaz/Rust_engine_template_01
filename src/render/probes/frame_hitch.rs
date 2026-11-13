//! Frame hitch / hang detector. Ships in release; per frame it costs one `Instant::now`, a few
//! compares and draining a fixed-size scope table — no allocation unless it logs.
//!
//! A frame is a **hitch** when its wall interval (end of `Last` to end of `Last`) exceeds
//! [`FrameHitchLog::threshold_ms`] (50 ms release, 500 ms debug, `HITCH_MS=<ms>` overrides), and a
//! **stall** when it exceeds [`STALL_MS`]. Hitches log at most one `WARN hitch` line per
//! [`LOG_COOLDOWN_SECS`] (later ones are counted into the next line); stalls always log.
//!
//! Each line carries frame #, dt, sim tick, main-schedule CPU vs time outside it (render sync,
//! present / swapchain wait, OS), and the costliest [`PerfScope`](crate::render::PerfScope) labels
//! of that frame. Grep `hitch` in a log to find them; the sim dump keeps the worst ones.

use std::time::Instant;

use bevy::prelude::*;

use crate::dev::perf_scope_frame_log::take_frame_scopes;
use crate::systems::sim_control::SimTick;

use super::frame_perf::FrameWallClock;

pub const HITCH_MS_RELEASE: f32 = 50.0;
/// Debug frames run ~10x slower than release; scale the bar so a hitch still means "much worse
/// than usual" rather than "every frame".
pub const HITCH_MS_DEBUG: f32 = 500.0;
pub const STALL_MS: f32 = 1000.0;
const LOG_COOLDOWN_SECS: f32 = 5.0;
pub const HITCH_TOP_SCOPES: usize = 4;

/// One flagged frame. `scopes` are the costliest `PerfScope` labels (empty label = unused slot).
#[derive(Clone, Copy, Debug, Default)]
pub struct HitchRecord {
    pub frame: u64,
    pub dt_ms: f32,
    /// First → Last of the main schedule this frame.
    pub main_ms: f32,
    /// `dt − main`: render-app sync, present / swapchain wait, OS / window events.
    pub outside_main_ms: f32,
    pub sim_tick: u64,
    pub scopes: [(&'static str, f32); HITCH_TOP_SCOPES],
}

#[derive(Resource, Debug)]
pub struct FrameHitchLog {
    pub threshold_ms: f32,
    pub frame: u64,
    /// Wall interval of the frame that just ended (ms).
    pub last_frame_ms: f32,
    pub hitches: u64,
    pub stalls: u64,
    /// Set for the frame that just ended when it was a hitch; cleared otherwise.
    pub this_frame: Option<HitchRecord>,
    last_end: Option<Instant>,
    next_log_at: Option<Instant>,
    unreported: u32,
    unreported_worst_ms: f32,
}

impl Default for FrameHitchLog {
    fn default() -> Self {
        let default_ms = if cfg!(debug_assertions) {
            HITCH_MS_DEBUG
        } else {
            HITCH_MS_RELEASE
        };
        let threshold_ms = std::env::var("HITCH_MS")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| *v > 0.0)
            .unwrap_or(default_ms);
        Self {
            threshold_ms,
            frame: 0,
            last_frame_ms: 0.0,
            hitches: 0,
            stalls: 0,
            this_frame: None,
            last_end: None,
            next_log_at: None,
            unreported: 0,
            unreported_worst_ms: 0.0,
        }
    }
}

/// Runs at the end of `Last`, after the frame-perf summary.
pub fn detect_frame_hitches(
    mut log: ResMut<FrameHitchLog>,
    wall: Res<FrameWallClock>,
    tick: Option<Res<SimTick>>,
) {
    let now = Instant::now();
    let scopes = take_frame_scopes();
    log.frame = log.frame.wrapping_add(1);
    log.this_frame = None;
    let Some(prev) = log.last_end.replace(now) else {
        return;
    };
    let dt_ms = now.duration_since(prev).as_secs_f32() * 1000.0;
    log.last_frame_ms = dt_ms;
    if dt_ms < log.threshold_ms {
        return;
    }
    let main_ms = wall.stamped_wall_ms().min(dt_ms);
    let record = HitchRecord {
        frame: log.frame,
        dt_ms,
        main_ms,
        outside_main_ms: dt_ms - main_ms,
        sim_tick: tick.map_or(0, |t| t.0),
        scopes: scopes.top::<HITCH_TOP_SCOPES>(),
    };
    log.this_frame = Some(record);
    log.hitches += 1;
    let stall = dt_ms >= STALL_MS;
    if stall {
        log.stalls += 1;
    }
    if !stall && log.next_log_at.is_some_and(|t| now < t) {
        log.unreported += 1;
        log.unreported_worst_ms = log.unreported_worst_ms.max(dt_ms);
        return;
    }
    log.next_log_at = Some(now + std::time::Duration::from_secs_f32(LOG_COOLDOWN_SECS));
    let [s0, s1, s2, s3] = record.scopes;
    let kind = if stall { "STALL" } else { "HITCH" };
    warn!(
        target: "hitch",
        "{kind} frame={} dt={:.1}ms (>{:.0}) main={:.1}ms outside_main={:.1}ms tick={} top=[{} {:.1} | {} {:.1} | {} {:.1} | {} {:.1}] +{} unreported (worst {:.1}ms) total={}",
        record.frame,
        dt_ms,
        log.threshold_ms,
        main_ms,
        record.outside_main_ms,
        record.sim_tick,
        s0.0, s0.1, s1.0, s1.1, s2.0, s2.1, s3.0, s3.1,
        log.unreported,
        log.unreported_worst_ms,
        log.hitches,
    );
    log.unreported = 0;
    log.unreported_worst_ms = 0.0;
}
