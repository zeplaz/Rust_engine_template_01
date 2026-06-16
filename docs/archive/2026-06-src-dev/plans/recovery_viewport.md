# Viewport / Visual Authority Recovery Plan

STATUS (2026-05-22):
**Operational path shipped** — canonical commit: semantic measure → `commit_authority_from_semantic` → `publish_simulation_map_viewport`. Live witnesses: `stage5_full_app_live.json`, `infrastructure_view_isolation_live.json` (isolation green).

**Hardening backlog** (not a broken-app gate): finish VM-B/C contract unification — see [`view_runtime_architecture_v1.md`](view_runtime_architecture_v1.md), [`stage5_5_active_todos.md`](stage5_5_active_todos.md).

### Single-writer map (VM-A5, 2026-05-23)

| Surface | Pose authority | Render contract | Legacy shim (read-only) |
|---------|----------------|-----------------|-------------------------|
| `WorldMain` | `ViewProjectionAuthority::commit_pose` (`MapCameraInput`, `BridgeCompat`) | `ViewportPipeline` | `MapCameraDesired` mirrored for compat |
| `SimulationMap` | Same group as WorldMain (shared camera policy) | `ViewportPipeline` | `MapViewInstances` extents from authority |
| `Minimap` | `MinimapFollow` / `MinimapShell` only | `ViewportPipeline` | Must **not** write `MapCameraDesired` |
| `WorldPreview` | `PreviewPanel` | `ViewportPipeline` | Editor panel state |
| `ViewManager` read model | `sync_view_manager_bridge` | — | **Sole** `ResMut<ViewManager>` — [`infra_proj2_sole_writer_plan_v1.md`](infra_proj2_sole_writer_plan_v1.md) |
| Fire extract caps | `PerViewRepresentationPolicy` | — | `build_fire_visual_frames_by_view` truncates per `ViewId` |
| Chunk residency membership | `ChunkResidencyTable` via `sync_chunk_residency_from_scheduler` | — | Seeds focus window when scheduler pending empty (S6-12) |

Historical note: incremental patches created layered shims; recovery below documents failure patterns to avoid regressions.

**VM-B landed (2026-05-20):**
- `viewport_pipeline` commits via `commit_resolved_viewports_to_authority` (`ViewAuthorityWriter::ViewportPipeline`)
- `apply_map_view_extents_from_authority` replaces `sync_editor_viewport_from_resolved`
- `stage5_full_app_live.json` → `viewport_contracts.view_isolation` includes `view_runtime` + `vm_a_witness` (non-gating)

Subsystem condition:
- fragmented
- overlapping authority
- temporary scaffolding accumulation
- duplicated intent pipelines
- diagnostic code mixed with production logic
- unclear ownership boundaries
- semantic migration incomplete

Primary issue:
LLM-generated incremental patches created layered temporary systems instead of unified architecture.

---

# DETECTED FAILURE PATTERNS

## 1. MULTIPLE VIEWPORT AUTHORITIES

Detected concepts:
- semantic viewport
- measured viewport
- frozen viewport
- rescue floor viewport
- minimap viewport
- camera desired viewport
- stabilized viewport
- fill-derived viewport

Problem:
No single source of truth.

Result:
- sync drift
- stale rects
- temporary override chains
- hidden precedence bugs
- hard-to-debug camera behavior

---

## 2. TEMPORARY MIGRATION LAYERS NEVER REMOVED

Detected:
- deprecated solver wrappers
- compatibility functions
- semantic adapters
- debug shadow pipelines

Problem:
Old systems still partially active.

Result:
- duplicated calculations
- conflicting corrections
- unclear runtime ownership

---

## 3. DEBUG SYSTEMS BECAME CORE LOGIC

Detected:
- trace systems carrying state
- sync signatures participating in orchestration
- visual diagnostics entangled with runtime logic

Problem:
Debug instrumentation evolved into hidden dependencies.

Result:
- visibility problems
- private/public mismatch
- orchestration leakage

---

## 4. CAMERA + VIEWPORT COUPLING IS UNCLEAR

Detected:
- MainWorldCamera
- viewport latch
- desired viewport
- orthographic traces
- minimap viewport sync

Problem:
Camera authority lifecycle undefined.

Result:
- multiple mutation points
- update-order fragility
- viewport jitter potential

---

# REQUIRED ARCHITECTURE

Must transition to:

```text
INPUT INTENT
    ->
SEMANTIC VIEWPORT REQUEST
    ->
VIEWPORT AUTHORITY REGISTRY
    ->
RESOLUTION PIPELINE
    ->
FINAL VIEWPORT COMMIT
    ->
RENDER EXTRACTION

ONLY ONE SYSTEM MAY COMMIT FINAL VIEWPORTS.

REQUIRED NEW CORE TYPES
SINGLE SOURCE OF TRUTH
#[derive(Resource, Debug, Clone)]
pub struct ViewportAuthorityState {
    pub active: ActiveViewport,
    pub pending: Option<ViewportRequest>,
    pub source: ViewportAuthoritySource,
    pub frame_last_updated: u64,
}
VIEWPORT REQUEST
#[derive(Debug, Clone)]
pub struct ViewportRequest {
    pub rect: Rect,
    pub priority: i32,
    pub source: ViewportAuthoritySource,
    pub mode: ViewportMode,
}
AUTHORITY SOURCE
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportAuthoritySource {
    Boot,
    Hud,
    Minimap,
    WorldGen,
    PauseMenu,
    ResizeEvent,
    RecoverySystem,
    DebugOverride,
}
VIEWPORT MODE
#[derive(Debug, Clone)]
pub enum ViewportMode {
    Locked,
    Semantic,
    Frozen,
    Transitional,
    Recovery,
}
REQUIRED NEW PIPELINE
STAGE 1 — INPUT COLLECTION

NO viewport mutation allowed.

Systems only emit:

Event<ViewportRequest>

Examples:

resize
minimap drag
pause menu open
worldgen preview activation
STAGE 2 — AUTHORITY ARBITRATION

Single system:

viewport_authority_resolver

Responsibilities:

choose winning request
reject stale requests
resolve priority conflicts
freeze during transitions
STAGE 3 — VIEWPORT COMMIT

ONLY THIS SYSTEM MUTATES CAMERAS

commit_viewport_to_camera

No other system allowed to:

set viewport
modify camera rect
stabilize transforms
STAGE 4 — DEBUG EXTRACTION

Debug systems become READ-ONLY.

Forbidden:

mutation
hidden state storage
authority override
REQUIRED DIRECTORY RESTRUCTURE

Current likely state:

gui/
    viewport_layout_solver.rs
    sim_view_sync_debug.rs
    visual_diagnostics.rs
    misc temp helpers...

Required:

src/
    viewport/
        authority.rs
        arbitration.rs
        request.rs
        commit.rs
        stabilization.rs
        recovery.rs
        events.rs
        debug.rs

    render/
        camera_commit.rs
        extraction.rs

    gui/
        viewport_emitters/
            minimap.rs
            world_preview.rs
            pause_menu.rs
SYSTEMS TO DELETE OR COLLAPSE

Likely obsolete:

merge_measured_with_solver
solve_sim_viewport_from_map_fill

Likely merge candidates:

stabilization
rescue floor
semantic fill
frozen authority
REQUIRED RULES
RULE 1

Debug systems NEVER mutate runtime state.

RULE 2

Only ONE system commits camera viewport.

RULE 3

Viewport solving cannot occur inside GUI widgets.

RULE 4

Viewport authority must be frame-traceable.

RULE 5

Temporary compatibility adapters must expire.

All temp systems require:

/// TEMP_REMOVE_AFTER:
/// ISSUE:
/// OWNER:
REQUIRED INSTRUMENTATION
FRAME TRACE
#[derive(Resource, Default)]
pub struct ViewportFrameTrace {
    pub history: Vec<ViewportTraceFrame>,
}
TRACE ENTRY
pub struct ViewportTraceFrame {
    pub frame: u64,
    pub source: ViewportAuthoritySource,
    pub rect: Rect,
    pub reason: String,
}
ORCHESTRATOR RESPONSIBILITIES
viewport_watchdog

Detect:

multiple commits/frame
stale authority
jitter loops
recursive stabilization
conflicting semantic requests
REQUIRED CLEANUP PASSES
PASS 1 — PURE CLEANUP

Remove:

unused imports
dead wrappers
stale adapters
hidden temp structs

NO behavior changes.

PASS 2 — OWNERSHIP EXTRACTION

Identify:

who requests viewport
who resolves viewport
who commits viewport

Build graph.

PASS 3 — DEBUG ISOLATION

Move all tracing into:

viewport/debug.rs

Read-only only.

PASS 4 — CAMERA COMMIT CENTRALIZATION

Ban all direct viewport mutation except:

commit_viewport_to_camera
PASS 5 — REMOVE TEMP SCAFFOLDING

Search:

temp
todo
hack
shim
compat
legacy
bridge
scaffold

Generate removal report.

REQUIRED SEARCH TERMS

Run semantic scans for:

viewport
camera
semantic
freeze
stabilize
solver
desired
fill
minimap
sync
authority
commit
rescue
trace
override

Build dependency graph.

RECOMMENDED NEW FILE
src/viewport/mod.rs
pub mod authority;
pub mod arbitration;
pub mod commit;
pub mod debug;
pub mod events;
pub mod recovery;
pub mod request;
pub mod stabilization;
RECOMMENDED CORE SYSTEM
src/viewport/arbitration.rs
use bevy::prelude::*;

use crate::viewport::{
    authority::*,
    events::*,
    request::*,
};

pub fn viewport_authority_resolver(
    mut requests: EventReader<ViewportRequestEvent>,
    mut authority: ResMut<ViewportAuthorityState>,
) {
    let mut best: Option<ViewportRequest> = None;

    for ev in requests.read() {
        let candidate = ev.request.clone();

        let replace = match &best {
            None => true,
            Some(current) => candidate.priority > current.priority,
        };

        if replace {
            best = Some(candidate);
        }
    }

    if let Some(request) = best {
        authority.active.rect = request.rect;
        authority.source = request.source;
        authority.frame_last_updated += 1;
    }
}
HIGH PRIORITY GOAL

Transform viewport system from:

layered hacks + temporary patches

into:

deterministic authority-driven pipeline

Without this:

future GUI work becomes unstable
minimap integration remains fragile
render threading becomes dangerous
camera sync bugs multiply
LLM-generated patches will continue compounding entropy

---

## Migration status (2026-05)

| Item | Status |
|------|--------|
| `merge_measured_with_solver` | **Removed** from public exports |
| `solve_sim_viewport_from_map_fill` | **Removed** |
| `solve_viewport_rescue_floor` | **Removed** |
| Canonical path | `semantic_viewport_from_map_fill` → `commit_authority_from_semantic` → `publish_simulation_map_viewport` |
| `frozen_exceeds_semantic_authority` | Wired in `authoritative_viewport.rs` |
| Empty `ViewportAuthorityDebugPlugin` | **Removed**; use `ViewportIntegrityAssertPlugin` (`VIEWPORT_AUTHORITY_DEBUG=1`) |
| Live drift witness | `debug_runs/viewport_drift.json`; trace via `SIM_VIEW_SYNC_DEBUG=1` |
| `in_game_ui` | Gated behind `legacy_engine` feature (not in default build) |