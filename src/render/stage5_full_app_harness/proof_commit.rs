//! RGR-H3-001 split — FULL_APP live proof finalize system (FINISH-UX-06 streak gate + commit).
//! Carved verbatim from `stage5_full_app_harness.rs` (pre-split monolith).

use bevy::prelude::*;

use crate::dev::{Stage5FinishUx06Streak, FINISH_UX06_STREAK_DONE};
use crate::engine::DebugCaptureFrameGate;
use crate::render::stage5_readiness::{stage5_readiness_passes, AppStage5ReadinessReport, Stage5ReadinessProfile};

use super::proof_payload::build_stage5_full_app_live_proof_payload;
use super::proof_reads::{write_minimap_compositor_live_proof_from_reads, Stage5FullAppLiveProofReads};
use super::witness_gates::{visual_tactical_vfx_witness_required, TacticalVfxWitnessGates};

/// Max PostUpdate frames after full-render diagnostic capture before `--test visual` fails.
pub const VISUAL_PROBE_UX06_MAX_FRAMES_AFTER_CAPTURE: u32 = 2400;

/// Max frames to wait for tactical fire/water particle rows before committing proof anyway.
pub const VISUAL_PROBE_TACTICAL_VFX_MAX_FRAMES: u32 = 900;

#[derive(Default)]
pub(crate) struct VisualProbeFinalizeState {
    proof_committed: bool,
    frames_since_capture: u32,
    last_logged_streak: u32,
}

fn finish_ux06_streak_done(streak: Option<&Stage5FinishUx06Streak>) -> bool {
    streak
        .map(|s| s.consecutive_good >= FINISH_UX06_STREAK_DONE)
        .unwrap_or(false)
}

/// `--test visual` proof should not commit before the instanced fire spine is wired.
/// Non-zero instance rows are optional (strategic zoom / cold ecology may be empty while
/// `instanced_dispatch_ok` still holds).
fn visual_probe_fire_witness_ready(
    reads: &Stage5FullAppLiveProofReads,
    report: &AppStage5ReadinessReport,
    require_graph_fire_buffer: bool,
) -> bool {
    let buffer_rows = reads
        .projection
        .as_ref()
        .map(|g| g.fire.instance_buffer.len())
        .unwrap_or(0);
    if require_graph_fire_buffer {
        return buffer_rows > 0;
    }
    if report.instanced_dispatch_ok && buffer_rows > 0 {
        return true;
    }
    let particle_rows = reads
        .particles
        .as_ref()
        .map(|f| f.instances.len())
        .unwrap_or(0);
    let draw_rows = reads
        .draw
        .as_ref()
        .map(|d| d.instance_count)
        .unwrap_or(0);
    buffer_rows > 0 || particle_rows > 0 || draw_rows > 0
}

/// After `--test visual` captures diagnostics: hold until **FINISH-UX-06** reaches
/// [`FINISH_UX06_STREAK_DONE`] consecutive clean readiness evals, then write proof JSON.
/// After proof commit, [`crate::render::gpu_surface_teardown::tick_visual_test_graceful_exit`] requests
/// `AppExit` so Vulkan surfaces tear down cleanly (manual window close can panic in wgpu).
pub(crate) fn finalize_visual_full_app_live_probe(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    profile: Res<Stage5ReadinessProfile>,
    report: Res<AppStage5ReadinessReport>,
    gate: Res<crate::render::FullRenderDiagnosticGate>,
    summary: Res<crate::render::FullRenderDiagnosticSummary>,
    proof_reads: Stage5FullAppLiveProofReads,
    streak: Option<Res<Stage5FinishUx06Streak>>,
    visual_exit: ResMut<crate::render::gpu_surface_teardown::VisualTestGracefulExit>,
    capture_gate: Option<Res<DebugCaptureFrameGate>>,
    mut state: Local<VisualProbeFinalizeState>,
) {
    if state.proof_committed || *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    if !gate.captured {
        return;
    }

    state.frames_since_capture = state.frames_since_capture.saturating_add(1);

    let min_frames = launch.min_capture_frames;
    let sim_frames = capture_gate.as_ref().map(|g| g.sim_frames).unwrap_or(0);
    if sim_frames < min_frames {
        if state.frames_since_capture == 1 || state.frames_since_capture % 30 == 0 {
            info!(
                target: "stage5_full_app_harness",
                sim_frames,
                min_frames,
                "visual probe waiting for minimum sim capture frames"
            );
        }
        return;
    }

    if crate::render::full_render_diagnostic_has_critical_anomaly(&summary) {
        warn!(
            ?summary,
            "FULL_APP visual probe blocked by render anomaly flags"
        );
        return;
    }
    if !stage5_readiness_passes(&report) {
        warn!(
            violations = ?report.violations,
            "FULL_APP visual probe blocked by readiness violations"
        );
        return;
    }

    let streak_n = streak.as_ref().map(|s| s.consecutive_good).unwrap_or(0);
    let ux06_done = finish_ux06_streak_done(streak.as_deref());

    if !ux06_done {
        if state.frames_since_capture >= VISUAL_PROBE_UX06_MAX_FRAMES_AFTER_CAPTURE {
            let blocker = streak
                .as_ref()
                .and_then(|s| s.last_blocker.as_deref())
                .unwrap_or("unknown");
            warn!(
                target: "stage5_full_app_harness",
                streak = streak_n,
                target_streak = FINISH_UX06_STREAK_DONE,
                frames_after_capture = state.frames_since_capture,
                last_blocker = ?blocker,
                "FULL_APP visual probe timed out waiting for FINISH-UX-06 streak"
            );
            if launch.visual_auto_exit && !visual_exit.armed {
                crate::render::gpu_surface_teardown::arm_visual_test_graceful_exit(visual_exit);
            }
            return;
        }
        if streak_n >= state.last_logged_streak.saturating_add(15)
            || (streak_n == 0 && state.last_logged_streak != 0)
        {
            info!(
                target: "stage5_full_app_harness",
                streak = streak_n,
                target_streak = FINISH_UX06_STREAK_DONE,
                frames_after_capture = state.frames_since_capture,
                last_blocker = streak.as_ref().and_then(|s| s.last_blocker.as_deref()),
                "visual probe waiting for FINISH-UX-06 streak"
            );
            state.last_logged_streak = streak_n;
        }
        return;
    }

    let require_f2_buffer = visual_tactical_vfx_witness_required(launch);
    if !visual_probe_fire_witness_ready(&proof_reads, &report, require_f2_buffer) {
        if streak_n >= state.last_logged_streak.saturating_add(15) {
            let buffer_rows = proof_reads
                .projection
                .as_ref()
                .map(|g| g.fire.instance_buffer.len())
                .unwrap_or(0);
            let particle_rows = proof_reads
                .particles
                .as_ref()
                .map(|f| f.instances.len())
                .unwrap_or(0);
            info!(
                target: "stage5_full_app_harness",
                fire_instance_buffer_rows = buffer_rows,
                particle_rows,
                "visual probe waiting for fire/particle witness before proof commit"
            );
            state.last_logged_streak = streak_n;
        }
        return;
    }

    if visual_tactical_vfx_witness_required(launch) {
        let gates = TacticalVfxWitnessGates::evaluate(
            proof_reads.particles.as_deref(),
            proof_reads.water_catalog.as_deref(),
            proof_reads.water_particles.as_deref(),
            proof_reads.projection.as_deref(),
        );
        if !gates.all_green_for_visual_proof(true) {
            if state.frames_since_capture < VISUAL_PROBE_TACTICAL_VFX_MAX_FRAMES {
                if state.frames_since_capture == 1 || state.frames_since_capture % 30 == 0 {
                    info!(
                        target: "stage5_full_app_harness",
                        ?gates,
                        fire_rows = proof_reads
                            .particles
                            .as_ref()
                            .map(|p| p.spark_witness.rows)
                            .unwrap_or(0),
                        water_streaks = proof_reads
                            .water_particles
                            .as_ref()
                            .map(|p| p.witness.river_streaks)
                            .unwrap_or(0),
                        "visual probe waiting for tactical VFX witness (zoom + particle rows)"
                    );
                }
                return;
            }
            warn!(
                target: "stage5_full_app_harness",
                ?gates,
                frames = state.frames_since_capture,
                "tactical VFX witness not green — committing proof after timeout"
            );
        }
    }

    const PROOF_PATH: &str = "debug_runs/stage5_full_app_live.json";
    let body = build_stage5_full_app_live_proof_payload(&report, &gate, &summary, &proof_reads);
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "FULL_APP",
        "stage5_full_app_harness",
        PROOF_PATH,
        body,
    );
    if !crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, payload) {
        warn!("stage5 live proof write failed: {PROOF_PATH}");
        return;
    }
    info!(
        target: "stage5_full_app_harness",
        path = PROOF_PATH,
        streak = streak_n,
        "wrote stage5 FULL_APP live proof (FINISH-UX-06 streak complete)"
    );

    if let (Some(board), Some(witness), Some(hud)) = (
        proof_reads.va2_board.as_ref(),
        proof_reads.va2_witness.as_ref(),
        proof_reads.va2_hud.as_ref(),
    ) {
        if crate::dev::write_visual_aidv2_live_proof(board, witness, hud) {
            let done = board
                .status
                .iter()
                .filter(|s| **s == crate::dev::stage5_live_todos::TodoStatus::Done)
                .count();
            info!(
                target: "visual_aidv2_live_todos",
                path = crate::dev::VISUAL_AID_V2_LIVE_JSON,
                done,
                total = crate::dev::VISUAL_AID_V2_TODOS.len(),
                "wrote visual aid v2 live proof"
            );
        }
    }

    state.proof_committed = true;

    if crate::render::minimap_gpu_compositor_env_enabled() {
        write_minimap_compositor_live_proof_from_reads(&proof_reads);
    }

    if launch.visual_auto_exit {
        if !visual_exit.armed {
            crate::render::gpu_surface_teardown::arm_visual_test_graceful_exit(visual_exit);
        }
    } else {
        info!(
            target: "stage5_full_app_harness",
            path = PROOF_PATH,
            "--test visual --stay-open: proof written; close the window when finished"
        );
    }
}
