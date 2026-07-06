# TACTICAL MAP RTT LANE v1 — parallel tracks A / B / C (operator P0)
# Generated 2026-07-04 · integrates RTT/VFX operator lane in HANDOFF § PLAN-BEVY-019-MIG-v1.
# Companions: [`plan_render_gui_refactor_v1.md`](plan_render_gui_refactor_v1.md) (Track A) ·
#   [`plan_gpu_particle_backend_split_v1.md`](plan_gpu_particle_backend_split_v1.md) (Track B) ·
#   [`visual_run_blockers.md`](visual_run_blockers.md) VR-10+ · witness `debug_runs/tactical_map_debug_live.json`

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-TACTICAL-MAP-RTT-v1
# status:       ACTIVE — Tracks A/B **coder slices SHIPPED 2026-07-04**; Track C operator witness OPEN
# priority:     P0 on master — **same lane as RTT/VFX operator verify** (HANDOFF ACTIVE)
# owner:        @coder_a CHAIN-A (RTT B5→A1→C) · @coder_b CHAIN-B (migration+cleanup+BQ+city) · @sim-steward (C witness) · @operator (sign-off)
# regression:   `cargo check -p proc_A_dine01` · `cargo run --release -- --test vfx` ·
#               refresh `debug_runs/tactical_map_debug_live.json` every Track C slice
# done_bar:     ☑ Track A lib (RTT-A1-001..004) · ☑ Track B lib (RTT-B5-001..004) ·
#               ☐ Track C operator (`tactical_map_debug_live.json` frame 120+ · `--test vfx` display)

# ═════════════════════════════════════════════════════════════════════
# PARALLEL TRACKS (pick one per session; coordinate file ownership)
# ═════════════════════════════════════════════════════════════════════
#
# | Track | ID prefix | Scope | Exit witness |
# |-------|-----------|-------|--------------|
# | **A** | RTT-A1-* | Viewport hygiene — delete latch, rename fill rect, scrub scissor debug | compile + stage5 lib |
# | **B** | RTT-B5-* | GPU VFX — ExtractedCameraMetrics + View uniform in fire/water WGSL | stage5 + `--test vfx` |
# | **C** | RTT-C-* | Tactical map void — rebuild + fresh tactical_map_debug_live.json | `tactical_map_debug_live.json` |

# ═════════════════════════════════════════════════════════════════════
# TRACK A — Item 1: viewport cleanup (delete latch · rename fill rect · scrub scissor debug)
# ═════════════════════════════════════════════════════════════════════
# Parent: plan_render_gui_refactor_v1.md § RGR-V2 · plan_cleanup Phase 0 hygiene
#
# Problem: RTT path is authoritative but legacy hole/latch/scissor debug slots still confuse
# witnesses and operator triage (`render_hole_steady_flip_count`, `using_hole`, scissor compare UI).
#
# | ID | Action | Files | Exit |
# |----|--------|-------|------|
# | RTT-A1-001 | **Delete** `MainWorldCameraViewportLatch` + empty reset fn | `gui/map_camera.rs`, `visual_readiness_witness.rs`, `dev/diagnostics/models.rs` | no `ViewportLatch` symbol in src/ |
# | RTT-A1-002 | **Rename** `SimulationMapFillRect` → `TacticalMapFillRect`; keep type alias `SimulationMapViewport` one release | `gui/sim_map_rtt.rs`, `gui/mod.rs`, `in_game_hud.rs`, `extracted_camera_metrics.rs`, `tactical_map_debug.rs`, construction pick chain | mechanical rename; stage5 + construction lib green |
# | RTT-A1-003 | **Scrub** scissor/hole debug strings and dead compare branches | `gui/hud/sim_view_sync_debug.rs`, `render/debug_viewport_overlay.rs`, `gui/hud/viewport_authority_debug.rs`, `gui/authoritative_viewport.rs` | grep `scissor` / `using_hole` / `hole latch` → docs-only or removed |
# | RTT-A1-004 | Repoint `render_hole_steady_flip_count` witness to RTT fill validity streak (or remove field) | `visual_readiness_witness.rs`, `stage5_full_app_harness.rs` | stage5 JSON field documented or dropped |
#
# Do NOT: delete `ViewProjectionAuthority` · delete `sim_map_rtt` · revert to window scissor path

# ═════════════════════════════════════════════════════════════════════
# TRACK B — Item 5 + 2a: ExtractedCameraMetrics + View uniform (fire/water WGSL)
# ═════════════════════════════════════════════════════════════════════
# Parent: plan_gpu_particle_backend_split_v1.md phases **2a** (generic view types) + **5** (WGSL ownership)
#
# Problem: `gpu_fire_particle_raster.rs` / `gpu_water_particle_raster.rs` still query `MainWorldCamera`
# for `view_proj` while emit path uses `ExtractedCameraMetrics` — drift causes particles off-map / invisible.
#
# | ID | Action | Files | Exit |
# |----|--------|-------|------|
# | RTT-B5-001 | Define shared `ViewUniform` / `ParticleViewGlobals` ShaderType from `ExtractedCameraMetrics` | `render/gpu_instanced_quad.rs` or `extracted_camera_metrics.rs` | one struct, documented fields |
# | RTT-B5-002 | Fire raster: build `view_proj` from extracted metrics only (drop MainWorldCamera query) | `gpu_fire_particle_raster.rs`, `fire_particle_draw.wgsl` | WGSL `View` block comments + field ownership |
# | RTT-B5-003 | Water raster: same pattern | `gpu_water_particle_raster.rs`, water draw WGSL | parity with fire |
# | RTT-B5-004 | Order: sync after `ExtractedCameraMetricsSet::Sync` on both Update + PostUpdate | plugins unchanged; add compile test | `--test vfx` sparks/precip visible on tactical map |
#
# Stable IDs unchanged (plan_gpu_particle_backend_split § Stable IDs — do not rename buffers).

# ═════════════════════════════════════════════════════════════════════
# TRACK C — Tactical map void chase (rebuild + fresh witness)
# ═════════════════════════════════════════════════════════════════════
#
# Problem: operator still reports tactical map void / missing VFX despite VR-10–VR-13 fixes on disk.
# `tactical_map_debug_live.json` is the authoritative triage artifact (always-on in `--test` harness).
#
# Current disk (2026-07-04): sim_image coverage_ratio **1.0** at frame 300 but minimap_image **0.0**
# — void may be UI compositing / wrong texture bound to ImageNode, not CPU fallback.
#
# | ID | Action | Owner | Exit |
# |----|--------|-------|------|
# | RTT-C-001 | **Rebuild** release after each A/B slice: `cargo build --release -p proc_A_dine01` | @coder | fresh binary |
# | RTT-C-002 | Run `cargo run --release -- --test vfx` (or `--test visual --stay-open`) ≥120 sim frames | @operator / @coder | repro notes |
# | RTT-C-003 | Refresh `debug_runs/tactical_map_debug_live.json` via `TacticalMapDebugPlugin` | @sim-steward | frame 120+ block present |
# | RTT-C-004 | Triage hints from witness: RenderLayers, RTT upload, ImageNode handle, fallback vs compositor | @coder | diagnosis_hints updated in JSON |
# | RTT-C-005 | If sim_image green but UI void → trace `SimulationMapViewportFill` + `SimulationMapTexture` bind | `in_game_hud.rs`, `sim_map_rtt.rs` | ImageNode shows non-zero center pixel |
#
# Loop: **C after every A/B merge** until operator sign-off. Do not close RTT lane on lib green alone.

# ═════════════════════════════════════════════════════════════════════
# QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
# RTT-A1-001 | RTT-A1 | coder | S | latch deleted; compile green
# RTT-A1-002 | RTT-A1 | coder | M | TacticalMapFillRect rename + alias shim
# RTT-A1-003 | RTT-A1 | coder | S | scissor debug scrubbed
# RTT-B5-001 | RTT-B5 | coder | S | ViewUniform struct
# RTT-B5-002 | RTT-B5 | coder | M | fire WGSL + raster
# RTT-B5-003 | RTT-B5 | coder | M | water WGSL + raster
# RTT-C-001..005 | RTT-C | coder_a/steward/operator | S–M | tactical_map_debug_live.json refreshed
# CB-MIG-001     | CHAIN-B | coder_b | M | mig_a audit split
# CB-MIG-002     | CHAIN-B | coder_b | S | DevDiagnosticsPlugin gated
# CB-MIG-003     | CHAIN-B | coder_b | S | render_schedule DiagnosticEvent
# CB-CLN-001     | CHAIN-B | coder_b | S | CLN-P0-R1/S8/P10
# CB-BQ-001      | CHAIN-B | coder_b | S | BQ-F2 style filter
# CB-BQ-002      | CHAIN-B | coder_b | S | BQ-F3 slot violation
# CB-CITY-001    | CHAIN-B | coder_b | M | CITY-G0-S1C split
# CB-RGR-001     | CHAIN-B | coder_b | M | RGR-T0-003 after A1-004
# CB-CITY-002    | CHAIN-B | coder_b | M | CITY-G2-C6 visual

# ═════════════════════════════════════════════════════════════════════
# CONFLICT MATRIX
# ═════════════════════════════════════════════════════════════════════
# Track A rename     | construction pick chain, map_egui_projection | single PR or staged alias
# Track B WGSL       | plan_gpu_particle stable buffer IDs          | no buffer renames
# Track C void       | minimap_compositor, tile_world_fallback      | read witness before editing raster
# plan_render_gui    | RGR-V2-004 overlaps RTT-A1-004               | same owner, one session

# ═════════════════════════════════════════════════════════════════════
# DUAL CODER CHAINS (2026-07-04 — parallel on master)
# ═════════════════════════════════════════════════════════════════════
#
# **CHAIN-A (@coder_a)** — RTT operator lane · execute **in order** (one PR per ID or batched per track):
#
#   RTT-B5-001 → RTT-B5-002 → RTT-B5-003 → RTT-B5-004
#        ↓
#   RTT-A1-001 → RTT-A1-002 → RTT-A1-003 → RTT-A1-004
#        ↓
#   RTT-C-004 → RTT-C-005  (+ steward RTT-C-001..003 after each merge)
#
# **CHAIN-B (@coder_b)** — POST-MIG migration tail + cleanup + product fixes · **no CHAIN-A files**:
#
#   CB-MIG-001  RGR-T0-004  Split mig_a adoption runtime vs audit JSON
#   CB-MIG-002  RGR-T0-001  Wire gated DevDiagnosticsPlugin (engine_with_worldgen)
#   CB-MIG-003  RGR-T0-002  render_schedule_perf → DiagnosticEvent (probes stay in render)
#   CB-CLN-001  CLN-P0-*    Cleanup Phase 0 hygiene (R1, S8, P10 — see plan_cleanup)
#   CB-BQ-001   BQ-F2-001   Style-aware prefer_stylepack_tier (construction)
#   CB-BQ-002   BQ-F3-001   MissingSlotViolation — kill silent hide_slot
#   CB-CITY-001 CITY-G0-S1C  building_grammar 3-way split (no behavior change)
#   CB-RGR-001  RGR-T0-003  Move perf/visual witness systems → dev/diagnostics (**after** CHAIN-A A1-004 lands)
#   CB-CITY-002 CITY-G2-C6  C6 visual / street furniture (**after** G0c or parallel if file-safe)
#
# ### File ownership lock (do not cross-edit same session)
#
# | CHAIN-A (@coder_a) owns | CHAIN-B (@coder_b) owns |
# |-------------------------|-------------------------|
# | `gpu_fire_particle_raster.rs`, `gpu_water_particle_raster.rs` | `mig_a_adoption.rs` (audit split only) |
# | `fire_particle_draw.wgsl`, water draw WGSL | `dev/diagnostics/*`, `render_schedule_perf.rs` (event redirect) |
# | `extracted_camera_metrics.rs`, `gpu_instanced_quad.rs` (ViewUniform) | `engine_with_worldgen.rs` (plugin gates) |
# | `gui/map_camera.rs`, `gui/sim_map_rtt.rs`, `in_game_hud.rs` (A1 rename) | `src/construction/procedural/*` (BQ-F2/F3) |
# | `visual_readiness_witness.rs`, `tactical_map_debug.rs` (C-004/005) | `building_grammar.rs` split (CITY-G0-S1C) |
# | `sim_view_sync_debug.rs`, `debug_viewport_overlay.rs` | CLN-P0 targets (R1, S8, P10, …) |
#
# **Merge rule:** CHAIN-A merges first on shared witness paths (`stage5_full_app_live.json`);
# CHAIN-B rebases if A1-004 touches harness JSON schema same PR window.
#
# **@coder_a do NOT pick:** mig_a split · BQ-F* · building_grammar · CLN-P0 · RGR-T0-001/002 (CHAIN-B)
# **@coder_b do NOT pick:** RTT-B5-* · RTT-A1-* · RTT-C-* · ViewUniform · TacticalMapFillRect rename
