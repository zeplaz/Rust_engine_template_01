# RENDER + GUI REFACTOR v1 — from God-Module coupling to bounded pipelines
# Generated 2026-07-04 from full render/ + gui/ architecture scan (117 + 203 `.rs` files).
# Companion: [`plan_cleanup_v1.md`](plan_cleanup_v1.md) D3 · [`dev/diagnostics/`](diagnostics/mod.rs) scaffold ·
#   [`codebase_index_v1.md`](codebase_index_v1.md) RN-*/GU-* · bevy-simulation-grade **07-repo-authority-map**
# Issue codes: RGR-S# (smell/strategy) · RGR-M# (mod.rs / API surface) · RGR-T# (telemetry extraction) ·
#              RGR-V# (viewport/RTT) · RGR-G# (GPU spine) · RGR-H# (harness split)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-RENDER-GUI-REFACTOR-v1
# status:       ACTIVE — Phase 0 closed 2026-07-04 (CHAIN-B) · Phase 1 RGR-M1-001 seeded
# priority:     P2 on master — parallel-safe with MIG mechanical; **after** cleanup D3 baseline OR doc-only D2 registry
# owner:        @sim-steward sequences · @coder render/viewport · @coder gui/hud · @planner for directory splits
# regression:   cargo test -p proc_A_dine01 --lib stage5 construction · validate-report cargo ·
#               refresh stage5_full_app_live.json + view_runtime witness on RGR-V slices
# depends:      PERF-INSTR-VFX-002 baseline before gating plugins (cleanup D3) · VM-A bridge stable during RGR-V
# territory:    src/render/* · src/gui/* (not src/construction/ — consumers only)
# done_bar:     render/mod.rs ≤15 pub use blocks · zero render→dev JSON writers · GPU raster reads ViewProjectionAuthority ·
#               stage5 harness ≤800 LOC (extracted to dev/)

# ═════════════════════════════════════════════════════════════════════
# SCAN SUMMARY (2026-07-04)
# ═════════════════════════════════════════════════════════════════════
#
# | Area | Files | Top smell |
# |------|-------|-----------|
# | render/ | 117 `.rs` | 77 `pub use` blocks in mod.rs; 18+ witness/telemetry re-exports |
# | gui/ | 203 `.rs` | 33 files >400 LOC; 55 gui→render imports, 67 render→gui (bidirectional) |
# | Coupling hub | stage5_full_app_harness.rs (2630 LOC) | Aggregates readiness, minimap, RTT, view authority, live proofs |
# | Viewport | sim_map_rtt.rs + view_runtime/ | RTT path **shipped**; VM-A bridge transitional, not dead code |
# | GPU spine | gpu_buffer_registry + 8 draw modules | Central registry — intentional, not accidental global state |
# | RenderApp hooks | 12 modules | Only render_schedule_perf uses ExtractSchedule; rest is Render/Core2d |
# | dev blur | 18 render files import crate::dev | 4 write debug_runs JSON from render code |
#
# Existing homes (do not duplicate):
#   - `src/dev/runtime_witness/` — envelope, cadence, file I/O
#   - `src/dev/diagnostics/` — event bus + passive witness models (scaffold landed 2026-07-04)
#   - `src/render/view_runtime/` — ViewProjectionAuthority, bridge, commit (VM-* migration)

# ═════════════════════════════════════════════════════════════════════
# SMELL TAXONOMY (validated)
# ═════════════════════════════════════════════════════════════════════
#
# RGR-S1 HIGH COUPLING — render/mod.rs re-exports 77 blocks (plugins + resources + witnesses + gui types).
#   Effect: any crate::render import can reach telemetry, dev re-exports, minimap gui state.
#
# RGR-S2 BIDIRECTIONAL GUI↔RENDER — 55 + 67 cross-imports; render/mod.rs pub use gui minimap types.
#   Effect: circular dependency at type level; refactors require dual-file edits.
#
# RGR-S3 TELEMETRY IN HOT PATHS — visual_readiness_witness, perf_attribution_witness, render_schedule_perf,
#   mig_a_adoption audit writers, stage5_full_app_harness disk flush scheduled unconditionally (cleanup D3).
#   Effect: prod builds schedule witness systems even when latches off; render_schedule_perf wraps RenderExtractApp.
#
# RGR-S4 GOD HARNESS — stage5_full_app_harness.rs (2630 LOC) + simulation_shell_phase2.rs (2888 LOC).
#   Effect: single file owns readiness eval, proof commit, view authority sampling, minimap M2/M3 seeds.
#
# RGR-S5 DUAL VIEW AUTHORITY (transitional, not bug) — ViewProjectionAuthority (render) + ViewManager (gui)
#   rebuilt via view_runtime/bridge.rs each frame. MapCameraDesired commits to authority; bridge rebuilds read model.
#   Effect: complexity during VM migration; **do not delete** until VM-C/D gates green.
#
# RGR-S6 GPU REGISTRY CENTRALIZATION — GPUBufferRegistry / GPUBindGroupRegistry used by 8+ passes.
#   Effect: single allocation hub (by design for spine); migration to BufferVec is **performance refactor**, not hygiene.
#
# RGR-S7 SCHEDULE CONTENTION — frame_perf + stall_watch + render_schedule_perf + full_render_diagnostic
#   bracket overlapping Update/PostUpdate/First/Render sets without unified gating (cleanup D3 item).
#
# NON-SMELLS (scan corrected false positives from generic refactor advice):
#   - MainWorldCameraViewportLatch — **deprecated stub**; RTT path active via SimulationMapFillRect.
#   - relink_core2d_transparent_overlay_order — **not present** in src/render (doc-only reference in vfx queue).
#   - pass.set_camera_viewport in gpu_water_particle_raster — **not found**; raster uses Core2d + RT layers.
#   - "Delete ViewProjectionAuthority" — **would break** construction (15+ files), map_camera, view_runtime.

# ═════════════════════════════════════════════════════════════════════
# TARGET ARCHITECTURE (incremental — not big-bang directory move)
# ═════════════════════════════════════════════════════════════════════
#
# ```text
# Main World                              RenderApp
# ──────────                              ─────────
# gui/ (presentation + input)             render/core/     bind groups, WGSL, registries
#   sim_map_rtt ──RTT handle──►           render/extraction/  ExtractSchedule (sim→render copy)
#   map_camera ──pose──►                  render/queue/       PhaseItem insertion (future)
# view_runtime/authority ◄──bridge──     render/pipelines/   raster/compute passes (future)
# dev/diagnostics/ ◄──Message──           render/probes/      thin timing only (stays in RenderApp)
# dev/runtime_witness/ ──JSON──► debug_runs/
# ```
#
# Principles:
# 1. **Render produces frames; dev observes frames** — no JSON, no envelope writes in render/.
# 2. **Probes stay thin in RenderApp** — timestamp + try_send/Message only; never block on disk I/O.
# 3. **One public API surface** — render/mod.rs exports Plugin group + typed resources consumers need.
# 4. **Complete VM migration before deleting bridge** — ViewProjectionAuthority is the write side; ViewManager is read model.
# 5. **Mechanical directory moves last** — after pub use collapse and telemetry extraction prove compile boundaries.

# ═════════════════════════════════════════════════════════════════════
# CONFLICT MATRIX
# ═════════════════════════════════════════════════════════════════════
# Lane                         | RGR items              | Rule
# -----------------------------|------------------------|------------------------------------------
# PLAN-BEVY-019-MIG-v1 P0      | RGR-G GPU registry     | No BufferVec migration during MIG-P1–P3
# PERF-INSTR-VFX-002           | RGR-T, RGR-S3          | Baseline before gating witness plugins (D3)
# plan_cleanup D3              | RGR-T Phase 0          | Same slices — merge, do not double-pick
# VM-* view_runtime            | RGR-V                  | Bridge stays until VM-C/D witness green
# Stage 5 FULL_APP             | RGR-H                  | Harness split must preserve stage5_full_app_live.json schema
# plan_gpu_terrain             | tile_world_fallback    | Coordinate RGR-V RTT with RN-TWF retirement
# OPS witness spine            | all RGR-T              | Refresh affected paths same PR; no renames without T2 defer lift

# ═════════════════════════════════════════════════════════════════════
# PHASED EXECUTION
# ═════════════════════════════════════════════════════════════════════

## Phase 0 — Stop the bleed (1–2 sessions, zero behavior change)
# Aligns with cleanup D3 + dev/diagnostics scaffold.

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| RGR-T0-001 | RGR-S3 | **@coder_b** (CB-MIG-002) | Wire `DevDiagnosticsPlugin` gated in engine_with_worldgen | validate-report cargo |
| RGR-T0-002 | RGR-S3 | **@coder_b** (CB-MIG-003) | `render_schedule_perf` → `DiagnosticEvent::RenderSchedule` | stage5 lib green |
| RGR-T0-003 | RGR-S3 | **@coder_b** (CB-RGR-001) | Move visual/perf witnesses → dev/diagnostics | **after** CHAIN-A A1-004 |
| RGR-T0-004 | RGR-S3 | **@coder_b** (CB-MIG-001) | Split mig_a adoption vs audit | mig_a_rollup.json still refreshes |
| RGR-M0-001 | RGR-S1 | @coder | Collapse witness pub use blocks in render/mod.rs to `pub use crate::dev::diagnostics::{...}` shim (deprecated) | compiler list of stragglers |
| RGR-D0-001 | RGR-S3 | @coder | Env-latch registry doc (cleanup D2 step 2) listing RGR-affected latches | doc-only or code module |

**Do not:** delete mig_a_adoption.rs or render_schedule_perf.rs files in Phase 0 — **split and redirect**, not delete.

## Phase 1 — mod.rs API collapse (2–3 sessions)

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| RGR-M1-001 | RGR-S1 | @coder | Introduce `render/api.rs` — explicit public surface (plugins + 10 core resources) | mod.rs pub use ≤30 blocks |
| RGR-M1-002 | RGR-S2 | @coder | Remove render→gui re-exports (MinimapShellState etc.); consumers import gui directly | no gui types in render/mod.rs |
| RGR-M1-003 | RGR-S1 | @coder | `#[deprecated]` re-export shims with migration note; fix call sites in construction/gui | validate-report cargo |
| RGR-M1-004 | RGR-S3 | @coder | Gate VisualDiagnosticsPlugin, FullRenderDiagnosticPlugin, StallWatchPlugin with run_if latches | D3 witness: systems skipped when latch off |

Target render/mod.rs end state (~15 exports):
- Plugin group fns or `RenderSpinePlugin` bundle
- `FireVisualFramePlugin`, `ViewportPipelinePlugin`, `ViewRuntimePlugin`, `TerrainInstancedDrawPlugin`
- `ViewProjectionAuthority`, `ResolvedViewports`, `RepresentationResult` (via gui policy — see Phase 2)
- `GPUBufferRegistry` (until RGR-G migration)

## Phase 2 — Viewport / RTT consolidation (highest visual ROI)
# **Not** "delete ViewProjectionAuthority" — **complete** the migration.

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| RGR-V2-001 | RGR-S5 | @coder | GPU fire/water raster: query `ViewProjectionAuthority` SimulationMap surface instead of raw MainWorldCamera | isolation_tests + visual |
| RGR-V2-002 | RGR-S5 | @sim-steward | Audit dual-writer map: enumerate ViewAuthorityWriter per frame in stage5 witness | writer list stable |
| RGR-V2-003 | RGR-V | @coder | Unify minimap RTT (`MinimapRenderTargetRegistry`) doc contract with sim_map_rtt pattern | minimap_compositor_live.json |
| RGR-V2-004 | RGR-V | @coder | Remove deprecated latch witness fields from visual_readiness after 2-week shim | render_hole_steady_flip_count sourced from RTT valid streak |
| RGR-V2-005 | RGR-S4 | @coder | Extract stage5 harness view-authority sampling → dev/diagnostics subscriber | harness −300 LOC |

**Hole glitch note:** RTT + SimulationMapFillRect is already the authoritative path. Remaining flicker is tracked via `MainWorldCameraOrthoTrace` / view_runtime trace — fix in RGR-V2-001/002, not by deleting bridge.

### RTT operator lane — Tracks A / B / C (P0 parallel)

**Hub:** [`plan_tactical_map_rtt_lane_v1.md`](plan_tactical_map_rtt_lane_v1.md) · HANDOFF § PLAN-TACTICAL-MAP-RTT-v1

| Track | Item | Action | IDs |
|-------|------|--------|-----|
| **A** | Item 1 cleanup | Delete latch · rename fill rect → `TacticalMapFillRect` · scrub scissor debug | RTT-A1-001..004 |
| **B** | Item 5 + 2a | `ExtractedCameraMetrics` + View uniform in fire/water WGSL | RTT-B5-001..004 |
| **C** | Void chase | Rebuild release · `--test vfx` · fresh `tactical_map_debug_live.json` | RTT-C-001..005 |

**Loop:** Track C runs after every A/B merge until operator void sign-off.

## Phase 3 — Harness decomposition (2–4 sessions)

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| RGR-H3-001 | RGR-S4 | @coder | Split stage5_full_app_harness: readiness eval / proof commit / view sample | each file ≤800 LOC |
| RGR-H3-002 | RGR-S4 | @designer | simulation_shell_phase2 witness surface → typed collectors in dev/runtime_witness | shell_phase2 −40% witness refs |
| RGR-H3-003 | RGR-S4 | @coder | Move stage6 re-export from render/mod.rs to dev-only path | render/mod.rs no dev re-exports |

## Phase 4 — GPU spine modernization (defer until MIG-V1 + perf baseline)
# **Not** a quick swap — registry owns cross-pass buffer lifetime.

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| RGR-G4-001 | RGR-S6 | @planner | BufferVec migration architecture — which buffers, which passes, rollback | plan slice in planner output |
| RGR-G4-002 | RGR-S6 | @coder | Pilot: tile_debug instances → BufferVec; keep registry for fire spine | GLB/visual smoke |
| RGR-G4-003 | RGR-S6 | @coder | Dedupe gpu_water_particle_raster from fire raster via shared core2d overlay module | one raster template |

## Phase 5 — Directory restructure (mechanical, last)
# Only after Phases 0–3 stable — cleanup plan Phase 4 style.

```text
src/render/
├── mod.rs              # thin — pub use api::*;
├── api.rs              # public contract
├── core/               # gpu_buffer_registry, bind_group, packed_formats
├── extraction/         # (existing)
├── view_runtime/       # (existing)
├── pipelines/          # gpu_*_raster, gpu_*_draw, terrain_instanced
├── probes/             # render_schedule_perf brackets only
└── plugins/            # plugin registration aggregates

src/gui/
├── mod.rs
├── tactical/           # sim_map_rtt, map_camera, in_game_hud, map_view
├── hud/                # (existing — split simulation_shell later)
└── authority/          # view_authority, viewport_*, authoritative_viewport

src/dev/
├── diagnostics/        # (existing — extend, do NOT create parallel dev/telemetry/)
└── runtime_witness/    # (existing)
```

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| RGR-P5-001 | RGR-S1 | @coder | Mechanical move to pipelines/ + probes/ with re-exports | cargo check + stage5 |
| RGR-P5-002 | RGR-S2 | @coder | gui/tactical/ move sim_map_rtt + map_camera | no import cycles |

# ═════════════════════════════════════════════════════════════════════
# FILE HOTSPOT MAP (priority order)
# ═════════════════════════════════════════════════════════════════════
#
# | LOC | File | Phase | Notes |
# |-----|------|-------|-------|
# | 2630 | render/stage5_full_app_harness.rs | H3 | Split first after telemetry extract |
# | 2888 | gui/hud/simulation_shell_phase2.rs | H3 | Witness collector extraction |
# | 1854 | render/tile_world_fallback.rs | V2 | RTT layers + authority; ties to GPU terrain plan |
# | 1632 | gui/in_game_hud.rs | V2 | SimulationMapViewportFill — canonical RTT UI |
# | 1520 | gui/map_camera.rs | V2 | Pose authority; do not "direct Transform hack" |
# | 720 | render/mig_a_adoption.rs | T0 | Split runtime vs audit |
# | 643 | render/gpu_particle_draw.rs | G4 | Registry hub — defer |
# | 490 | render/viewport_pipeline.rs | V2 | ResolvedViewports → authority |
# | 296 | render/view_runtime/bridge.rs | V2 | Keep until VM gates; simplify don't delete |
# | 278 | render/render_schedule_perf.rs | T0 | Probes stay; witness moves |

# ═════════════════════════════════════════════════════════════════════
# ACTIVE PHASE + QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
# current:   RGR-M1-001 ACTIVE (@coder)
# next_pick: RGR-M1-002 after M1-001 lands (render→gui re-export removal)
# queue:     `$ref:tools/orchestrator/queues/render_gui_refactor_queue.json`
# tandem:    operator P0 RTT-C-001..003 parallel OK · CHAIN-A/B lib closed
# blocked:   RGR-G4-* until MIG-V1 · RGR-P5-* until Phase 3 gate · RGR-V2-004 until V2-001 lands

# id                  | issue  | owner     | effort | exit_witness
# RGR-T0-001          | RGR-T0 | coder     | S      | DevDiagnosticsPlugin wired + gated
# RGR-T0-002          | RGR-T0 | coder     | S      | RenderScheduleEvent end-to-end
# RGR-T0-003          | RGR-T0 | coder     | M      | visual/perf witnesses in dev/diagnostics
# RGR-T0-004          | RGR-T0 | coder     | M      | mig_a audit split; runtime plugin unchanged
# RGR-M0-001          | RGR-M0 | coder     | S      | deprecated shim; straggler list in HANDOFF
# RGR-M1-001          | RGR-M1 | coder     | M      | render/api.rs landed
# RGR-V2-001          | RGR-V2 | coder     | M      | fire/water raster uses ViewProjectionAuthority
# RGR-H3-001          | RGR-H3 | coder     | L      | harness ≤800 LOC per file

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES — corrections to generic "delete witnesses" advice
# ═════════════════════════════════════════════════════════════════════
# 1. render_schedule_perf **cannot** move entirely to dev/ — RenderApp timing probes must stay on render thread.
#    Pattern: probes in render/probes/ → Message → dev/diagnostics subscriber → runtime_witness I/O.
# 2. mig_a_adoption **cannot** be deleted — MIG-A10 spine authority, static bulk tags are production runtime.
# 3. ViewProjectionAuthority **cannot** be deleted — 15+ construction/gui files depend on it; complete VM migration first.
# 4. GPUBufferRegistry → BufferVec is a **perf program**, not render-folder cleanup — schedule under RGR-G4 post-MIG.
# 5. Use `dev/diagnostics/` (exists) not new `dev/telemetry/` — avoids parallel witness homes.
# 6. sim_map_rtt is **already** the RTT standard — work is consolidating GPU pass consumers, not replacing FillRect.
