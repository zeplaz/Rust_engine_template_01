//! Stage 5 **live** execution registry (`STAGE5_TODOS`): file targets, runtime checks, and hooks.
//!
//! **Closure gate (13 rows only):** [`stage5_close_checklist.md`](stage5_close_checklist.md) §A.
//! **Deferred sticky work:** [`stage5_triage_backlog.md`](stage5_triage_backlog.md) — never add VM/fire-streaming/construction rows here.
//!
//! ## Legend (dependency / blast radius)
//! - **ROOT**: must be fixed first; if wrong, downstream reads UNKNOWN masked as OK.
//! - **BLOCKER**: needs root stability; wrong → multi-system confusion.
//! - **DEPENDENT**: only meaningful after blockers.
//! - **CASCADE**: one failure poisons several subsystems.
//!
//! ## Root spine (execute in this order before parallel GPU / fire / LOD)
//! 1. **TODO-01** — runtime readiness fence (`evaluate_app_stage5_readiness`). Unlocks all measurement.
//! 2. **TODO-04** — view authority (ViewManager vs `MapCameraDesired`). Real engine spine; cascades to minimap, fire alignment, LOD, preview.
//! 3. **TODO-06** — `CommittedVisualSnapshotFence` / frame sync. Unlocks GPU + fire truth on a stable frame boundary.
//!
//! Then: TODO-02, TODO-03, TODO-05, GPU layer (07–09), fire (10–11), preview/LOD (12–13) per `STAGE5_TODOS` P0→P4 list.
//!
//! `STAGE5_ROOT_GATE_SEQUENCE` drives which todo [`emit_active_stage5_todo_context`] highlights first.
//!
//! **Predicate `Done`:** Under [`Stage5ReadinessProfile::FULL_APP`], [`hook_post_readiness_evaluate`] calls
//! [`sync_stage5_todo_board_predicates`] → [`reconcile_stage5_todo_board`]: each row is set to [`TodoStatus::Done`]
//! or [`TodoStatus::InProgress`] from [`stage5_readiness_passes`] **and** report slices / witnesses (reopens stale
//! Done). When all rows are Done and readiness passes, hooks log [`STAGE5_BOARD_QUIET`] (throttled), not
//! misleading `all_todos_done` every subsystem tick.
use bevy::prelude::*;

use crate::gui::{
    fire_visual_producer_count, mirror_world_main_camera_from_map_desired, MapCameraDesired,
    MapCameraSystemSet, RepresentationResult, ViewId, ViewManager, WorldRepresentationFrame,
};
use crate::systems::atmosphere::AtmospherePartialWriteMetrics;
use crate::render::extraction::{extract_fire_simulation_snapshot, RenderProjectionGraph};
use crate::render::{
    merge_domain_projection_into_representation, stage5_readiness_passes, AppStage5ReadinessReport,
    CommittedVisualSnapshotFence, GpuIndirectDrawSpine, SharedOverlayFieldBuffers,
    Stage5FireViewChunkWitness, Stage5LodBandLogWitness, Stage5MapCameraBridgeWitness,
    Stage5ReadinessEvalInvocation, Stage5ReadinessProfile, WorldFireParticleDrawDispatch,
    sync_world_fire_indirect_draw,
};

/// Post-mirror pose delta threshold (translation + zoom) for TODO-04 / TODO-05 witnesses.
const MAP_BRIDGE_DRIFT_OK: f32 = 0.1;
/// Consecutive frames under [`MAP_BRIDGE_DRIFT_OK`] with a live WorldMain view (TODO-05 stability).
const MAP_BRIDGE_STABLE_FRAMES: u32 = 6;
/// When all 13 spine rows are [`TodoStatus::Done`] and readiness passes, throttle hook spam.
const STAGE5_ALL_DONE_LOG_INTERVAL_INV: u32 = 120;

/// Per-frame STAGE5_* hook lines (`STAGE5_SPINE_HOOK`, `STAGE5_FIRE_HOOK`, …) are expensive at INFO.
/// Set `STAGE5_VERBOSE=1` to enable them; readiness eval logs stay on `stage5_readiness::live=info`.
#[inline]
fn stage5_per_frame_hooks_verbose() -> bool {
    std::env::var_os("STAGE5_VERBOSE").is_some()
}

/// Live operator status for a tracked Stage 5 closure item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TodoStatus {
    #[default]
    Open,
    InProgress,
    Done,
}

/// One tracked Stage 5 todo (template row in `STAGE5_TODOS`).
///
/// `status` in the static slice is always [`TodoStatus::Open`]; authoritative live
/// status is stored in [`Stage5LiveTodoBoard`].
#[derive(Clone, Copy, Debug)]
pub struct Stage5LiveTodo {
    pub id: &'static str,
    pub status: TodoStatus,
    pub file: &'static str,
    pub system: &'static str,
    pub goal: &'static str,
    pub runtime_check: &'static str,
    pub failure_mode: &'static str,
}

/// Failure-probability order (P0 first). `status` in each row is template only; live status is in [`Stage5LiveTodoBoard`].
pub static STAGE5_TODOS: &[Stage5LiveTodo] = &[
    Stage5LiveTodo {
        id: "TODO-01",
        status: TodoStatus::Open,
        file: "src/render/stage5_readiness.rs",
        system: "Stage5ReadinessProfile, evaluate_app_stage5_readiness",
        goal: "Running-app readiness fence is proven: FULL_APP report fully populated each PostUpdate, not UNKNOWN.",
        runtime_check: "RUST_LOG=stage5_readiness::live=info shows READINESS_EVAL_BEGIN/END/FLAGS, READINESS_VIOLATION_SUMMARY, READINESS_FRAME_FENCE / READINESS_FRAME_FENCE_OK; inv increases.",
        failure_mode: "FULL_APP false-positive green or all gates UNKNOWN in windowed app.",
    },
    Stage5LiveTodo {
        id: "TODO-02",
        status: TodoStatus::Open,
        file: "src/render/stage5_readiness.rs",
        system: "evaluate_app_stage5_readiness (violations vec)",
        goal: "violations.is_empty() reflects same-frame ECS inputs; rows logged under READINESS_VIOLATION_ROW when non-empty.",
        runtime_check: "READINESS_VIOLATION_SUMMARY viol_rows_emitted + viol_rows_truncated == viol_len; viol_digest matches READINESS_EVAL_END.",
        failure_mode: "Pass/fail silently or stale snapshot vs sim.",
    },
    Stage5LiveTodo {
        id: "TODO-03",
        status: TodoStatus::Open,
        file: "src/render/extraction/render_projection_graph.rs",
        system: "run_render_projection_graph, RenderProjectionGraph",
        goal: "All three projection domains (fire, logistics, ecology) reflected on graph resource in runtime order.",
        runtime_check: "READINESS_PROJECTION_GRAPH (evaluate, throttled) + READINESS_PROJECTION_GRAPH_BUILD only when projection counts change (not every tick); STAGE5_READINESS_VERBOSE=1 for full eval trace.",
        failure_mode: "LOD + camera + preview diverge silently.",
    },
    Stage5LiveTodo {
        id: "TODO-04",
        status: TodoStatus::Open,
        file: "src/gui/map_camera.rs, src/gui/view_authority.rs",
        system: "ViewManager, MapCameraDesired, mirror + trace",
        goal: "ViewManager sole authority for WorldMain camera pose after bridge; MapCameraDesired only mirrored.",
        runtime_check: "FULL_APP + RUST_LOG=map_camera_desired::write=debug: MAP_CAMERA_DESIRED_WRITE lines from map_camera_apply_input_to_desired, tile_world_fallback::focus_main_camera_on_world_params, view_representation::apply_minimap_camera_intent when pose changes.",
        failure_mode: "minimap/world drift, dual-write inconsistency.",
    },
    Stage5LiveTodo {
        id: "TODO-05",
        status: TodoStatus::Open,
        file: "src/gui/map_camera.rs",
        system: "map_camera_apply_input_to_desired, mirror_world_main_camera_from_map_desired",
        goal: "No hidden second writer path for world main camera (VM-09B closed).",
        runtime_check: "RUST_LOG=stage5_live_todos=info: STAGE5_MAP_CAMERA_HOOK post_mirror bridge_drift + desired vs WorldMain; no unexplained spikes.",
        failure_mode: "Jitter + desync minimap vs world.",
    },
    Stage5LiveTodo {
        id: "TODO-06",
        status: TodoStatus::Open,
        file: "src/gui/view_representation.rs",
        system: "CommittedVisualSnapshotFence commit path, render-frame sync",
        goal: "CommittedVisualSnapshotFence stamped every frame when sim advances (fence_committed in readiness).",
        runtime_check: "READINESS_EVAL_END fence_fire_tick + STAGE5_SPINE_HOOK fence_fire_tick; READINESS_VIOLATION_SUMMARY when sim advances; no fence violation string.",
        failure_mode: "GPU + CPU desync, stale frames.",
    },
    Stage5LiveTodo {
        id: "TODO-07",
        status: TodoStatus::Open,
        file: "src/render/gpu_indirect_draw.rs",
        system: "sync_world_fire_indirect_draw",
        goal: "instanced_dispatch_ok true in live logs when policy demands instancing.",
        runtime_check: "STAGE5_GPU_HOOK + STAGE5_GPU_INDIRECT_MISMATCH on count drift; READINESS_EVAL_FLAGS inst=true.",
        failure_mode: "Silent CPU fallback draw path.",
    },
    Stage5LiveTodo {
        id: "TODO-08",
        status: TodoStatus::Open,
        file: "src/systems/atmosphere/mod.rs",
        system: "P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE, atmosphere GPU partial path",
        goal: "Compiler-constant gpu_field_authoritative backed by real GPU partial writes each frame when required.",
        runtime_check: "RUST_LOG=stage5_live_todos=info: STAGE5_SPINE_HOOK atm_partial_dispatch / atm_gpu_tex_uploads / atm_full_field_fallback each frame under FULL_APP.",
        failure_mode: "CPU fallback without detection.",
    },
    Stage5LiveTodo {
        id: "TODO-09",
        status: TodoStatus::Open,
        file: "src/render/extraction/fire_visual_extract.rs (sync_shared_overlay_from_simulation), src/render/overlay_field_buffers.rs",
        system: "SharedOverlayFieldBuffers, STAGE5_SPINE_HOOK overlay_rev",
        goal: "Single shared buffer spine for overlays in app.",
        runtime_check: "STAGE5_OVERLAY_SHARED_BUFFERS on heat revision bump; STAGE5_SPINE_HOOK overlay_rev; READINESS_EVAL_FLAGS ovl=true.",
        failure_mode: "Duplicated overlays or stale UI.",
    },
    Stage5LiveTodo {
        id: "TODO-10",
        status: TodoStatus::Open,
        file: "src/render/extraction/fire_visual_extract.rs",
        system: "extract_fire_simulation_snapshot",
        goal: "fire_visual_producer_count == 1 at runtime.",
        runtime_check: "STAGE5_FIRE_HOOK count==1; READINESS_EVAL_FLAGS fire1=true.",
        failure_mode: "Multiple extractors → inconsistent fire.",
    },
    Stage5LiveTodo {
        id: "TODO-11",
        status: TodoStatus::Open,
        file: "src/render/fire_view_extract.rs",
        system: "VisibleFireChunkSet, build_fire_visual_frames_by_view",
        goal: "VisibleFireChunkSet derived from view projection only, matches sim snapshot.",
        runtime_check: "RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet.",
        failure_mode: "Ghost or missing fire chunks.",
    },
    Stage5LiveTodo {
        id: "TODO-12",
        status: TodoStatus::Open,
        file: "src/gui/editor/world_preview/preview_render_contract.rs",
        system: "PreviewCameraState, WorldPreviewGpuRuntime, preview_authoritative_surface",
        goal: "Phase D preview_authoritative_surface is GpuRenderTarget when requested.",
        runtime_check: "READINESS_EVAL_FLAGS prev_rt phd; RUST_LOG=stage5_preview::live=warn sees STAGE5_PREVIEW_CPU_FALLBACK when GPU requested but CpuSwap.",
        failure_mode: "Silent CPU preview fallback.",
    },
    Stage5LiveTodo {
        id: "TODO-13",
        status: TodoStatus::Open,
        file: "src/gui/world_representation.rs",
        system: "compute_world_representation_frame, WorldLodBand transitions",
        goal: "LOD band transitions visible in runtime logs per frame when band changes.",
        runtime_check: "RUST_LOG=world_representation::lod=info on global LOD band change (WorldRepresentation line).",
        failure_mode: "Stuck LOD under zoom.",
    },
];

/// Root gates: **TODO-01 → TODO-04 → TODO-06** before treating GPU / fire / LOD work as trustworthy.
pub const STAGE5_ROOT_GATE_SEQUENCE: &[&str] = &["TODO-01", "TODO-04", "TODO-06"];

/// Live status for each row in [`STAGE5_TODOS`] (same order / length).
#[derive(Resource, Debug)]
pub struct Stage5LiveTodoBoard {
    pub status: Vec<TodoStatus>,
}

/// Throttles `STAGE5_BOARD_QUIET` when the live board is fully green.
#[derive(Resource, Debug, Default)]
struct Stage5TodoBoardQuietLog {
    last_all_done_inv: u32,
}

/// Throttles `STAGE5_ACTIVE_TODO` spam (6 hooks × long INFO lines per frame was ~250ms on Windows).
#[derive(Resource, Debug, Default)]
struct Stage5ActiveTodoLogState {
    last_logged_idx: Option<usize>,
    last_logged_inv: u32,
}

impl Default for Stage5LiveTodoBoard {
    fn default() -> Self {
        Self {
            status: vec![TodoStatus::Open; STAGE5_TODOS.len()],
        }
    }
}

impl Stage5LiveTodoBoard {
    fn merged(&self, idx: usize) -> Stage5LiveTodo {
        let mut row = STAGE5_TODOS[idx];
        row.status = self.status.get(idx).copied().unwrap_or(TodoStatus::Open);
        row
    }

    /// First index that is not [`TodoStatus::Done`], in [`STAGE5_TODOS`] registry order.
    pub fn first_non_done_index(&self) -> Option<usize> {
        self.status
            .iter()
            .enumerate()
            .find(|(_, s)| **s != TodoStatus::Done)
            .map(|(i, _)| i)
    }

    /// Next work item: walk [`STAGE5_ROOT_GATE_SEQUENCE`] first; if all root gates done, use [`Self::first_non_done_index`].
    pub fn first_work_item_index(&self) -> Option<usize> {
        for &id in STAGE5_ROOT_GATE_SEQUENCE {
            let Some(i) = STAGE5_TODOS.iter().position(|t| t.id == id) else {
                continue;
            };
            if self.status.get(i).copied().unwrap_or(TodoStatus::Open) != TodoStatus::Done {
                return Some(i);
            }
        }
        self.first_non_done_index()
    }

    pub fn mark(&mut self, id: &str, next: TodoStatus) {
        for (i, row) in STAGE5_TODOS.iter().enumerate() {
            if row.id == id {
                if let Some(s) = self.status.get_mut(i) {
                    *s = next;
                }
                return;
            }
        }
    }
}

fn full_app_hooks_enabled(profile: Res<Stage5ReadinessProfile>) -> bool {
    *profile == Stage5ReadinessProfile::FULL_APP
}

/// Log the current **next spine work item** (root gate sequence first) whenever a hooked subsystem runs.
pub fn emit_active_stage5_todo_context(world: &mut World, subsystem: &'static str) {
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let passes = world
        .get_resource::<AppStage5ReadinessReport>()
        .is_some_and(stage5_readiness_passes);
    let inv = world
        .get_resource::<Stage5ReadinessEvalInvocation>()
        .map(|x| x.0)
        .unwrap_or(0);
    let Some(active) = world
        .get_resource::<Stage5LiveTodoBoard>()
        .and_then(|board| board.first_work_item_index().map(|i| (i, board.merged(i))))
    else {
        if passes {
            let mut quiet = world.get_resource_or_insert_with(Stage5TodoBoardQuietLog::default);
            if inv.saturating_sub(quiet.last_all_done_inv) < STAGE5_ALL_DONE_LOG_INTERVAL_INV {
                return;
            }
            quiet.last_all_done_inv = inv;
            info!(
                target: "stage5_live_todos",
                "STAGE5_BOARD_QUIET subsystem={subsystem} passes=true done={}/{} inv={inv}",
                STAGE5_TODOS.len(),
                STAGE5_TODOS.len(),
            );
        } else {
            warn!(
                target: "stage5_live_todos",
                "STAGE5_ACTIVE_TODO subsystem={subsystem} board_all_done_but_readiness_not_passing registry_len={}",
                STAGE5_TODOS.len()
            );
        }
        return;
    };
    let (i, row) = active;
    let mut log_state = world.get_resource_or_insert_with(Stage5ActiveTodoLogState::default);
    let should_log_active = stage5_per_frame_hooks_verbose()
        || log_state.last_logged_idx != Some(i)
        || inv.saturating_sub(log_state.last_logged_inv) >= STAGE5_ALL_DONE_LOG_INTERVAL_INV;
    if !should_log_active {
        return;
    }
    log_state.last_logged_idx = Some(i);
    log_state.last_logged_inv = inv;
    info!(
        target: "stage5_live_todos",
        "STAGE5_ACTIVE_TODO subsystem={subsystem} spine_gate_seq={:?} idx={i} id={} status={:?} file={} system={} goal={} runtime_check={} failure_mode={}",
        STAGE5_ROOT_GATE_SEQUENCE,
        row.id,
        row.status,
        row.file,
        row.system,
        row.goal,
        row.runtime_check,
        row.failure_mode
    );
}

/// Per-row closure inputs for [`reconcile_stage5_todo_board`].
#[derive(Clone, Debug)]
struct Stage5TodoPredicateInputs {
    passes: bool,
    inv: u32,
    report: AppStage5ReadinessReport,
    fence_ok: bool,
    vm_wm: bool,
    bridge: Stage5MapCameraBridgeWitness,
    fire_w: Stage5FireViewChunkWitness,
    lod_w: Stage5LodBandLogWitness,
}

#[must_use]
fn stage5_row_predicate_met(id: &str, ctx: &Stage5TodoPredicateInputs) -> bool {
    if !ctx.passes {
        return false;
    }
    match id {
        "TODO-01" => ctx.inv >= 1,
        "TODO-02" => ctx.report.violations.is_empty(),
        "TODO-03" => ctx.report.projection_domains >= 3,
        "TODO-04" => ctx.vm_wm && ctx.bridge.last_post_mirror_drift <= MAP_BRIDGE_DRIFT_OK,
        "TODO-05" => {
            ctx.vm_wm && ctx.bridge.consecutive_frames_bridge_ok >= MAP_BRIDGE_STABLE_FRAMES
        }
        "TODO-06" => ctx.fence_ok,
        "TODO-07" => ctx.report.instanced_dispatch_ok,
        "TODO-08" => ctx.report.gpu_field_authoritative,
        "TODO-09" => ctx.report.overlay_from_shared_buffers_only,
        "TODO-10" => ctx.report.single_fire_extract,
        "TODO-11" => ctx.fire_w.world_main_visible_orphan_chunks == 0,
        "TODO-12" => ctx.report.phase_d_ok && ctx.report.preview_render_target_active,
        "TODO-13" => ctx.lod_w.lod_band_log_emissions >= 1,
        _ => false,
    }
}

/// Set each row to [`TodoStatus::Done`] or [`TodoStatus::InProgress`] from live predicates (reopens stale Done).
fn reconcile_stage5_todo_board(board: &mut Stage5LiveTodoBoard, ctx: &Stage5TodoPredicateInputs) {
    if !ctx.passes {
        reopen_stage5_todos_on_readiness_fail(board);
        return;
    }
    let mut marked_done = 0u32;
    let mut reopened = 0u32;
    for (i, row) in STAGE5_TODOS.iter().enumerate() {
        let met = stage5_row_predicate_met(row.id, ctx);
        let target = if met {
            TodoStatus::Done
        } else {
            TodoStatus::InProgress
        };
        let prev = board.status.get(i).copied().unwrap_or(TodoStatus::Open);
        if prev == target {
            continue;
        }
        if prev == TodoStatus::Done && !met {
            reopened = reopened.saturating_add(1);
        }
        if target == TodoStatus::Done {
            marked_done = marked_done.saturating_add(1);
        }
        if let Some(s) = board.status.get_mut(i) {
            *s = target;
        }
    }
    if marked_done > 0 || reopened > 0 {
        let done_count = board
            .status
            .iter()
            .filter(|s| **s == TodoStatus::Done)
            .count();
        info!(
            target: "stage5_live_todos",
            "STAGE5_TODO_BOARD_RECONCILE marked_done={marked_done} reopened={reopened} done_count={done_count}/{} inv={}",
            STAGE5_TODOS.len(),
            ctx.inv,
        );
    }
}

/// Reopen rows marked [`TodoStatus::Done`] when readiness regresses (stale “all_todos_done” otherwise).
fn reopen_stage5_todos_on_readiness_fail(board: &mut Stage5LiveTodoBoard) {
    let mut reopened = 0u32;
    for s in board.status.iter_mut() {
        if *s == TodoStatus::Done {
            *s = TodoStatus::InProgress;
            reopened = reopened.saturating_add(1);
        }
    }
    if reopened > 0 {
        info!(
            target: "stage5_live_todos",
            "STAGE5_TODO_BOARD_REGRESSION_REOPEN rows_reopened={reopened} registry_len={}",
            STAGE5_TODOS.len(),
        );
    }
}

fn build_stage5_todo_predicate_inputs(
    world: &World,
    report: &AppStage5ReadinessReport,
    passes: bool,
) -> Stage5TodoPredicateInputs {
    Stage5TodoPredicateInputs {
        passes,
        inv: world
            .get_resource::<Stage5ReadinessEvalInvocation>()
            .map(|x| x.0)
            .unwrap_or(0),
        report: report.clone(),
        fence_ok: world.get_resource::<CommittedVisualSnapshotFence>().is_some_and(|f| {
            f.fire.tick > 0 || f.fire.sim_time_micros > 0
        }),
        vm_wm: world
            .get_resource::<ViewManager>()
            .is_some_and(|vm| vm.view(ViewId::WorldMain).is_some()),
        bridge: world
            .get_resource::<Stage5MapCameraBridgeWitness>()
            .cloned()
            .unwrap_or_default(),
        fire_w: world
            .get_resource::<Stage5FireViewChunkWitness>()
            .cloned()
            .unwrap_or_default(),
        lod_w: world
            .get_resource::<Stage5LodBandLogWitness>()
            .cloned()
            .unwrap_or_default(),
    }
}

/// Reconcile live board status from readiness report + witnesses each green evaluation.
fn sync_stage5_todo_board_predicates(world: &mut World, report: &AppStage5ReadinessReport, passes: bool) {
    let ctx = build_stage5_todo_predicate_inputs(world, report, passes);
    let Some(mut board) = world.get_resource_mut::<Stage5LiveTodoBoard>() else {
        return;
    };
    reconcile_stage5_todo_board(&mut board, &ctx);
}

/// Called from [`crate::render::evaluate_app_stage5_readiness`] at end of evaluation.
pub fn hook_post_readiness_evaluate(world: &mut World) {
    emit_active_stage5_todo_context(world, "evaluate_app_stage5_readiness");
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let report = world.resource::<AppStage5ReadinessReport>().clone();
    let passes = stage5_readiness_passes(&report);
    crate::dev::stage5_finish_todos::sync_stage5_finish_todo_board(world, passes);
    let first_v = report
        .violations
        .first()
        .map(String::as_str)
        .unwrap_or("(none)");
    if stage5_per_frame_hooks_verbose() {
        info!(
            target: "stage5_live_todos",
            "STAGE5_READINESS_HOOK passes={passes} violations_first={first_v} producer_count={}",
            fire_visual_producer_count()
        );
    }
    if !passes {
        if let Some(mut board) = world.get_resource_mut::<Stage5LiveTodoBoard>() {
            reopen_stage5_todos_on_readiness_fail(&mut board);
        }
        return;
    }
    sync_stage5_todo_board_predicates(world, &report, passes);
    crate::dev::visual_aidv2_live_todos::hook_post_readiness_visual_aidv2(world);
}

fn hook_first_frame(world: &mut World) {
    emit_active_stage5_todo_context(world, "First(render_frame_start)");
}

fn hook_map_camera_post(world: &mut World) {
    emit_active_stage5_todo_context(world, "map_camera_mirror_chain");
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let desired = world.resource::<MapCameraDesired>();
    let vm = world.resource::<ViewManager>();
    let dt = desired.translation.truncate();
    let d_zoom = desired.scale.x;
    let (wm_tx, wm_zoom) = vm
        .view(ViewId::WorldMain)
        .map(|v| (v.camera.translation, v.camera.zoom))
        .unwrap_or((Vec2::ZERO, -1.0));
    let drift = (dt - wm_tx).length() + (d_zoom - wm_zoom).abs();
    if stage5_per_frame_hooks_verbose() {
        info!(
            target: "stage5_live_todos",
            "STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=({:.2},{:.2}) zoom={:.4} world_main_xy=({:.2},{:.2}) zoom={:.4} bridge_drift={:.4}",
            dt.x,
            dt.y,
            d_zoom,
            wm_tx.x,
            wm_tx.y,
            wm_zoom,
            drift,
        );
    }
    if let Some(mut w) = world.get_resource_mut::<Stage5MapCameraBridgeWitness>() {
        w.last_post_mirror_drift = drift;
        if wm_zoom < 0.0 {
            w.consecutive_frames_bridge_ok = 0;
        } else if drift <= MAP_BRIDGE_DRIFT_OK {
            w.consecutive_frames_bridge_ok = w.consecutive_frames_bridge_ok.saturating_add(1);
        } else {
            w.consecutive_frames_bridge_ok = 0;
        }
    }
}

fn hook_fire_extract_post(world: &mut World) {
    emit_active_stage5_todo_context(world, "fire_extract_post");
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let n = fire_visual_producer_count();
    if stage5_per_frame_hooks_verbose() {
        info!(
            target: "stage5_live_todos",
            "STAGE5_FIRE_HOOK fire_visual_producer_count={n} (expect 1 for FIRE-01)"
        );
    }
}

fn hook_gpu_dispatch_post(
    world: &mut World,
    policy: &RepresentationResult,
    spine: &GpuIndirectDrawSpine,
    draw: &WorldFireParticleDrawDispatch,
) {
    emit_active_stage5_todo_context(world, "gpu_indirect_sync_post");
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    if !policy.particle_policy.instanced_draw {
        return;
    }
    if stage5_per_frame_hooks_verbose() {
        info!(
            target: "stage5_live_todos",
            "STAGE5_GPU_HOOK indirect_instances={} dispatch_count={} draw_instances={} (PHF-01 alignment)",
            spine.world_fire.instance_count,
            spine.dispatch_count,
            draw.instance_count
        );
    }
}

fn system_stage5_live_after_gpu_indirect(world: &mut World) {
    let policy = world.resource::<RepresentationResult>().clone();
    let spine = world.resource::<GpuIndirectDrawSpine>().clone();
    let draw = world.resource::<WorldFireParticleDrawDispatch>().clone();
    hook_gpu_dispatch_post(world, &policy, &spine, &draw);
}

/// Mark every registry row as in-flight so operators see full Stage 5 closure as active.
fn stage5_begin_root_gate_todos(mut board: ResMut<Stage5LiveTodoBoard>) {
    for i in 0..STAGE5_TODOS.len() {
        if matches!(board.status.get(i), Some(TodoStatus::Open)) {
            board.status[i] = TodoStatus::InProgress;
        }
    }
}

/// Register Stage 5 live todo board + logging hooks. Safe to call once from [`crate::engine::EnginePlugin`].
pub fn register_stage5_todo_runtime_hooks(app: &mut App) {
    app.init_resource::<Stage5LiveTodoBoard>()
        .insert_resource(crate::dev::stage5_finish_todos::Stage5FinishTodoBoard::from_template())
        .init_resource::<crate::dev::stage5_finish_todos::Stage5FinishUx06Streak>()
        .init_resource::<Stage5TodoBoardQuietLog>()
        .init_resource::<Stage5ActiveTodoLogState>()
        .init_resource::<Stage5MapCameraBridgeWitness>()
        .add_systems(Startup, stage5_begin_root_gate_todos);
    app.add_systems(
        First,
        hook_first_frame.run_if(full_app_hooks_enabled),
    );
    app.add_systems(
        Update,
        hook_map_camera_post
            .after(mirror_world_main_camera_from_map_desired)
            .in_set(MapCameraSystemSet::ApplyInput)
            .run_if(full_app_hooks_enabled),
    );
    app.add_systems(
        Update,
        stage5_live_spine_probe
            .after(merge_domain_projection_into_representation)
            .run_if(full_app_hooks_enabled),
    );
    app.add_systems(
        Update,
        hook_fire_extract_post
            .after(extract_fire_simulation_snapshot)
            .run_if(full_app_hooks_enabled),
    );
    app.add_systems(
        PostUpdate,
        system_stage5_live_after_gpu_indirect
            .after(sync_world_fire_indirect_draw)
            .run_if(full_app_hooks_enabled),
    );
}

/// Spine consistency log (runs after domain merge).
fn stage5_live_spine_probe(world: &mut World) {
    emit_active_stage5_todo_context(world, "representation_spine_post_merge");
    if *world.resource::<Stage5ReadinessProfile>() != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let policy_ok = world.get_resource::<RepresentationResult>().is_some();
    let graph_ok = world.get_resource::<RenderProjectionGraph>().is_some();
    let fence = world.get_resource::<CommittedVisualSnapshotFence>();
    let fence_ok = fence.is_some_and(|f| f.fire.tick > 0 || f.fire.sim_time_micros > 0);
    let fence_fire_tick = fence.map(|f| f.fire.tick).unwrap_or(0);
    let world_frame_ok = world.get_resource::<WorldRepresentationFrame>().is_some();
    let atm = world.get_resource::<AtmospherePartialWriteMetrics>();
    let (partial_disp, gpu_tex_cnt, full_fb) = atm
        .map(|m| {
            (
                m.partial_compute_dispatch_count,
                m.gpu_texture_upload_count,
                m.full_field_fallback_active,
            )
        })
        .unwrap_or((0, 0, false));
    let overlay = world.get_resource::<SharedOverlayFieldBuffers>();
    let overlay_rev = overlay.map(|o| o.revision).unwrap_or(0);
    let overlay_cells = overlay.map(|o| o.chunk_fire_heat.len()).unwrap_or(0);
    if stage5_per_frame_hooks_verbose() {
        info!(
            target: "stage5_live_todos",
            "STAGE5_SPINE_HOOK policy_present={policy_ok} graph_present={graph_ok} fence_committed={fence_ok} fence_fire_tick={fence_fire_tick} world_frame_present={world_frame_ok} overlay_rev={overlay_rev} overlay_chunk_cells={overlay_cells} atm_partial_dispatch={partial_disp} atm_gpu_tex_uploads={gpu_tex_cnt} atm_full_field_fallback={full_fb}",
        );
    }
}

/// Mark a todo by id after **live** verification (call from tooling or a dedicated system).
pub fn mark_stage5_todo(world: &mut World, id: &'static str, status: TodoStatus) {
    if let Some(mut board) = world.get_resource_mut::<Stage5LiveTodoBoard>() {
        board.mark(id, status);
    }
}

#[cfg(test)]
mod stage5_todo_board_tests {
    use super::*;

    use bevy::math::{Rect, Vec2};

    use crate::gui::{
        ViewCameraState, ViewInstance, ViewInteractionState, ViewProjection, ViewRenderPolicy, ViewRenderTarget,
    };
    use crate::systems::sim_control::SimStepStamp;

    fn all_green_readiness_report() -> AppStage5ReadinessReport {
        AppStage5ReadinessReport {
            vt4_ok: true,
            vt5_ok: true,
            single_fire_extract: true,
            gpu_field_authoritative: true,
            preview_render_target_active: true,
            phase_d_ok: true,
            overlay_from_shared_buffers_only: true,
            particle_lod_scales: true,
            phase_f_lod_proof_ok: true,
            instanced_dispatch_ok: true,
            phase_f_ok: true,
            projection_domains: 3,
            registered_producers: 1,
            duplicate_visual_scan_count: 0,
            violations: Vec::new(),
        }
    }

    fn vm_with_world_main() -> ViewManager {
        let mut vm = ViewManager::default();
        vm.views.insert(
            ViewId::WorldMain,
            ViewInstance {
                id: ViewId::WorldMain,
                camera_entity: Entity::PLACEHOLDER,
                render_target: ViewRenderTarget::None,
                camera: ViewCameraState::default(),
                projection: ViewProjection::default(),
                interaction_state: ViewInteractionState::default(),
                viewport_rect: Rect::from_corners(Vec2::ZERO, Vec2::new(800.0, 600.0)),
                render_policy: ViewRenderPolicy::default(),
            },
        );
        vm
    }

    fn seed_predicate_close_world() -> World {
        let mut world = World::new();
        world.insert_resource(Stage5LiveTodoBoard {
            status: vec![TodoStatus::InProgress; STAGE5_TODOS.len()],
        });
        world.insert_resource(Stage5ReadinessEvalInvocation(3));
        world.insert_resource(CommittedVisualSnapshotFence {
            fire: SimStepStamp::new(1, 1),
        });
        world.insert_resource(vm_with_world_main());
        world.insert_resource(Stage5MapCameraBridgeWitness {
            last_post_mirror_drift: 0.01,
            consecutive_frames_bridge_ok: MAP_BRIDGE_STABLE_FRAMES,
        });
        world.insert_resource(Stage5FireViewChunkWitness {
            world_main_visible_orphan_chunks: 0,
            f7_a_per_view_extract_bounded: true,
        });
        world.insert_resource(Stage5LodBandLogWitness {
            lod_band_log_emissions: 1,
        });
        world
    }

    #[test]
    fn registry_defines_thirteen_stage5_todos() {
        assert_eq!(
            STAGE5_TODOS.len(),
            13,
            "STAGE5_TODOS must list TODO-01 through TODO-13"
        );
    }

    #[test]
    fn predicate_green_with_witnesses_marks_all_thirteen_rows_done() {
        let mut world = seed_predicate_close_world();
        let report = all_green_readiness_report();
        assert!(stage5_readiness_passes(&report));
        sync_stage5_todo_board_predicates(&mut world, &report, true);
        let board = world.resource::<Stage5LiveTodoBoard>();
        assert!(
            board.status.iter().all(|s| *s == TodoStatus::Done),
            "all 13 rows Done only when passes + per-row witnesses/report slices hold"
        );
    }

    #[test]
    fn high_bridge_drift_leaves_camera_todos_open() {
        let mut world = seed_predicate_close_world();
        world.insert_resource(Stage5MapCameraBridgeWitness {
            last_post_mirror_drift: 50.0,
            consecutive_frames_bridge_ok: 0,
        });
        let report = all_green_readiness_report();
        sync_stage5_todo_board_predicates(&mut world, &report, true);
        let board = world.resource::<Stage5LiveTodoBoard>();
        let i4 = STAGE5_TODOS.iter().position(|t| t.id == "TODO-04").unwrap();
        let i5 = STAGE5_TODOS.iter().position(|t| t.id == "TODO-05").unwrap();
        assert_ne!(board.status[i4], TodoStatus::Done);
        assert_ne!(board.status[i5], TodoStatus::Done);
        assert!(board.status.iter().filter(|s| **s == TodoStatus::Done).count() < 13);
    }

    #[test]
    fn fire_orphan_witness_blocks_todo11() {
        let mut world = seed_predicate_close_world();
        world.insert_resource(Stage5FireViewChunkWitness {
            world_main_visible_orphan_chunks: 3,
            f7_a_per_view_extract_bounded: true,
        });
        let report = all_green_readiness_report();
        sync_stage5_todo_board_predicates(&mut world, &report, true);
        let board = world.resource::<Stage5LiveTodoBoard>();
        let i11 = STAGE5_TODOS.iter().position(|t| t.id == "TODO-11").unwrap();
        assert_ne!(board.status[i11], TodoStatus::Done);
    }

    #[test]
    fn zero_lod_log_witness_blocks_todo13() {
        let mut world = seed_predicate_close_world();
        world.insert_resource(Stage5LodBandLogWitness {
            lod_band_log_emissions: 0,
        });
        let report = all_green_readiness_report();
        sync_stage5_todo_board_predicates(&mut world, &report, true);
        let board = world.resource::<Stage5LiveTodoBoard>();
        let i13 = STAGE5_TODOS.iter().position(|t| t.id == "TODO-13").unwrap();
        assert_ne!(board.status[i13], TodoStatus::Done);
    }

    #[test]
    fn readiness_fail_does_not_advance_board() {
        let mut world = World::new();
        world.insert_resource(Stage5LiveTodoBoard {
            status: vec![TodoStatus::InProgress; STAGE5_TODOS.len()],
        });
        let report = all_green_readiness_report();
        sync_stage5_todo_board_predicates(&mut world, &report, false);
        let board = world.resource::<Stage5LiveTodoBoard>();
        assert!(board.status.iter().all(|s| *s == TodoStatus::InProgress));
    }

    #[test]
    fn reconcile_reopens_stale_done_when_witness_regresses() {
        let mut world = seed_predicate_close_world();
        let report = all_green_readiness_report();
        sync_stage5_todo_board_predicates(&mut world, &report, true);
        {
            let board = world.resource::<Stage5LiveTodoBoard>();
            assert!(board.status.iter().all(|s| *s == TodoStatus::Done));
        }
        world.insert_resource(Stage5LodBandLogWitness {
            lod_band_log_emissions: 0,
        });
        sync_stage5_todo_board_predicates(&mut world, &report, true);
        let board = world.resource::<Stage5LiveTodoBoard>();
        let i13 = STAGE5_TODOS.iter().position(|t| t.id == "TODO-13").unwrap();
        assert_eq!(board.status[i13], TodoStatus::InProgress);
        assert!(board.status.iter().any(|s| *s != TodoStatus::Done));
    }
}
