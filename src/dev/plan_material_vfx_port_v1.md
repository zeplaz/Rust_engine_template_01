# MATERIAL2d + BUFFER VFX PORT — fire/water particle systems modernization

# Generated 2026-07-04 from GPU fire/water architecture dependency audit.
# Companion: [`plan_render_gui_refactor_v1.md`](plan_render_gui_refactor_v1.md) (Phase 2 RGR-V2) · [`plan_cleanup_v1.md`](plan_cleanup_v1.md) Phase 3 defer

# Issue codes: MVP-M# (Material2d adoption) · MVP-B# (buffer migration) · MVP-C# (Camera/orthographic)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-MATERIAL-VFX-PORT-v1
# status:       DEFERRED — gate line: RGR-V2-001 lands + RGR-G4 unblocks (operator visual sign-off)
# priority:     P2 — depends RGR-V2 operator acceptance · blocks GPU terrain perf optimization
# owner:        @coder render/vfx · @sim-steward for Material2d visual signoff
# regression:   cargo test -p proc_A_dine01 --lib stage5 vfx · validate-report cargo ·
#               fire_particle_live.json + water_particle_live.json witness suite refresh
# depends:      RGR-V2-001 (fire/water raster ViewProjectionAuthority adoption) LANDED
#               RGR-G4 (BufferVec architecture design) APPROVED
# territory:    src/render/gpu_fire_particle_raster.rs · src/render/gpu_water_particle_raster.rs ·
#               src/render/gpu_particle_draw.rs · src/render/gpu_buffer_registry.rs (cross-ref)
# done_bar:     Material2d fire + water rasters signed off visually · BufferVec teardown path validated ·
#               RenderLayers preserved (minimap/tactical cameras) · WGSL fire/water shaders compile clean

# ═════════════════════════════════════════════════════════════════════
# GATE LINE & BLOCKING CONTEXT
# ═════════════════════════════════════════════════════════════════════
#
# **DO NOT START THIS PROGRAM until:**
# 1. RGR-V2-001 lands: GPU fire/water raster queries ViewProjectionAuthority SimulationMap surface
#    (not raw MainWorldCamera). Operator visual sign-off + isolation_tests in [`debug_runs/chain_b_witness_live.json`]
# 2. RGR-G4 unlocks: BufferVec architecture decision approved; no MIG-V1 conflicts
#
# **Reason:** Material2d adoption requires stable authority source (V2-001 guarantees) + buffer migration
# roadmap clarity (G4 scope definition). Premature work = rework on authority or registry changes.

# ═════════════════════════════════════════════════════════════════════
# PHASED EXECUTION
# ═════════════════════════════════════════════════════════════════════

## Phase 1 — Material2d adoption (3–4 sessions)
# Implement `Material2d<FireParticleMaterial>` + `Material2d<WaterParticleMaterial>` render plugins.
# **COMPLETE-THEN-RETIRE protocol:** both old (legacy `gpu_fire_particle_raster`) and new coexist with feature flag.
# Fire on custom raster nodes ONLY after Material replacement is visually signed off in isolation_tests.

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| MVP-M1-001 | MVP-M | @coder | Implement `FireParticleMaterial` for Bevy 0.19 Material2d; WGSL shader port from `fire_particle.wgsl` | fire_particle_material.rs: compile clean + unit tests |
| MVP-M1-002 | MVP-M | @coder | Implement `WaterParticleMaterial` for Bevy 0.19 Material2d; WGSL shader port from `water_particle_draw.wgsl` | water_particle_material.rs: compile clean + unit tests |
| MVP-M1-003 | MVP-M | @coder | Wire Material2d fire/water plugins into RenderApp; feature-gate legacy raster imports (not delete) | legacy rasters still callable (feature `legacy_particle_raster`); isolation_tests pass |

**Do not:** delete or move `gpu_fire_particle_raster.rs`, `gpu_water_particle_raster.rs` files until Phase 1 visual signoff.
**Strategy:** parallel render with both paths (legacy + Material2d flagged); unit tests + isolated visual proof before switching.

## Phase 2 — GPUBufferRegistry → BufferVec migration (2–3 sessions)
# Coordinate with RGR-G4 architecture decision. Cross-ref 45 buffer usages across 17 src/ files.

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| MVP-B2-001 | MVP-B | @coder | Audit GPUBufferRegistry usage in fire/water draw modules; enumerate layout + lifetime boundaries | dependency map: 45 refs × 17 files (priority ranked) |
| MVP-B2-002 | MVP-B | @coder | Pilot `BufferVec<T>` for fire particle instance data; retire `gpu_surface_teardown.rs` workarounds only after BufferVec teardown path proves clean in isolation | teardown_witness_live.json shows no orphan allocations |

**Defer:** full registry replacement until all pilots (tile_debug MVP-G4-002 equivalent) prove buffer lifetime. Do NOT delete registry in Phase 2.

## Phase 3 — Camera3d orthographic 2.5D depth sort spike (optional, 1–2 sessions)
# Low-priority unlock for depth-sorted particle rendering if needed post-Material2d.
# **CRITICAL:** RenderLayers must NOT be removed wholesale — minimap RTT / tactical cameras depend on layer separation.

| ID | Issue | Owner | Action | Exit |
|:---|:---|:---|:---|:---|
| MVP-C3-001 | MVP-C | @coder | Spike: evaluate Camera3d orthographic 2.5D depth sorting vs current RenderLayers layering strategy | design doc + performance estimate (defer full impl if <5% ROI) |

**Constraint:** If depth sort adopted, ensure RenderLayers remain — do not flatten to single pass.

# ═════════════════════════════════════════════════════════════════════
# DEPENDENCY SURFACE & HOTSPOTS
# ═════════════════════════════════════════════════════════════════════
#
# | File | LOC | Issue | Phase | Action |
# |------|-----|-------|-------|--------|
# | gpu_fire_particle_raster.rs | ~600 | Material2d migrate | M1 | Complete Material impl first; retire after visual green |
# | gpu_water_particle_raster.rs | ~500 | Material2d migrate | M1 | Complete Material impl first; retire after visual green |
# | gpu_particle_draw.rs | 643 | Registry refactor | B2 | Audit buffer binds; pilot BufferVec in isolated slice |
# | gpu_buffer_registry.rs | ~700 | Architecture anchor | B2 | Keep until BufferVec architecture locked (RGR-G4) |
# | assets/shaders/fire/fire_particle.wgsl | ~200 | Material2d port | M1 | Port to Material2d WGSL entry point + uniforms |
# | assets/shaders/fire/fire_particle_draw.wgsl | ~150 | Material2d port | M1 | Consolidate into fire_particle.wgsl if single-target |
# | assets/shaders/water/water_particle_draw.wgsl | ~180 | Material2d port | M1 | Material2d entry point + uniform bindings |
#
# **Cross-file buffer references:** 45 direct references to GPUBufferRegistry in:
#   src/render/gpu_particle_draw.rs (8) · gpu_fire_particle_raster.rs (7) · gpu_water_particle_raster.rs (6) ·
#   gpu_buffer_registry.rs (4 self-refs) · gpu_terrain_instanced.rs (5) · gpu_debug_tile.rs (6) · 
#   other draw modules (3)

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES — corrections & constraints from PLAN-RENDER-GUI-REFACTOR-v1 evaluation
# ═════════════════════════════════════════════════════════════════════

# 1. **Rejected Phase-1 deletion targets (DO NOT REVISIT):**
#    - `mig_a_adoption.rs` — MIG-A10 spine runtime production authority (static bulk tagging).
#      Lives in `src/dev/` now; render/ copy is witness audit only. Do not confuse audit mirror with production deletion.
#    - `visual_readiness_witness.rs` + `perf_attribution_witness.rs` — already in `src/dev/diagnostics/`.
#      Render→dev extract is COMPLETE (CHAIN-B closed 2026-07-04). Do not re-propose file deletions.
#    - `src/render/mig_a_static.rs` — production runtime authority. MIG-A10 is active spine dependency (construction 15+ files).
#      Do NOT delete; references are legitimate, not accidental.
#
# 2. **"Witness stalls render thread" measurement artifact (KNOWN & DOCUMENTED):**
#    - Stall visible in perf_attribution after checkpoint_interval wall absorption. NOT a witness system fault.
#    - Trust `PerfScope upd_*` metrics + `gpu_gap_ms` probe output (PERF-INSTR-VFX-002 baseline witness).
#    - Probe with `PERF_NO_VSYNC=1` if needed; check iGPU vs dGPU in `visual_readiness_witness.json` `device_tier`.
#
# 3. **Material2d adoption is a render modernization, NOT a cleanup pass:**
#    - Raster nodes stay as long as needed for fallback or isolation testing.
#    - Complete-then-retire pattern prevents flicker + visual regression risk.
#    - No aggressive file deletion; this is a staged adoption path.
#
# 4. **BufferVec migration is perf-driven, not hygiene:**
#    - RGR-G4 addresses cross-pass lifetime safety + allocation pooling.
#    - Phase 2 pilots (fire/water pilot + tile_debug pilot) must prove teardown before registry replacement.
#    - Premature deletion = UAF risk or flawed architecture design.
#
# 5. **RenderLayers MUST remain (HARD CONSTRAINT):**
#    - Minimap RTT, tactical_map_rtt, and other non-main-world cameras depend on layer separation.
#    - Do NOT flatten or wholesale remove RenderLayers when adopting 2.5D depth sorting.
#    - Phase 3 spike is optional — only if depth-sort unlocks >10% visible ROI.

# ═════════════════════════════════════════════════════════════════════
# CONFLICT MATRIX
# ═════════════════════════════════════════════════════════════════════
# Lane | Item | Rule |
# -----|------|------|
# PLAN-RENDER-GUI-REFACTOR-v1 | RGR-V2-001 must land first (operator visual green) | gate line: blocking until complete |
# PLAN-RENDER-GUI-REFACTOR-v1 | RGR-G4 architecture must unlock BufferVec scope | gate line: blocking until approved |
# PLAN-BEVY-019-MIG-v1 | MIG-V1 closed; no Material2d conflicts w/ core migration | parallel OK after RGR-V2-001 |
# plan_cleanup_v1 | Phase 3+ may include GPU spine refactors — coordinate with B2 pilot scope | defer large cleanup until MVP-B2 witness |
# PERF-INSTR-VFX-002 | Baseline perf before Material2d adoption accepted | M1 exit witness references baseline |

# ═════════════════════════════════════════════════════════════════════
# ACTIVE PHASE + QUEUE SEED
# ═════════════════════════════════════════════════════════════════════
# status:    DEFERRED — awaits RGR-V2-001 landing + RGR-G4 unlocking
# next_pick: MVP-M1-001 (Material2d FireParticleMaterial) when RGR-V2-001 witness green
# queue:     (no queue file — plan-only registration per orchestrator policy)
# blocked:   entire program until gate line clears
# regression_witness: fire_particle_live.json + water_particle_live.json + stage5_full_app_live.json

# Note: This program is registered in development_plan_index.md and may appear in
# cross_front_pick_queue_v1.md when gate line unlocks. Do not seed HANDOFF rows until
# `git show RGR-V2-001` confirms ViewProjectionAuthority adoption + operator sign-off.

# ═════════════════════════════════════════════════════════════════════
# REFERENCE ARCHITECTURE
# ═════════════════════════════════════════════════════════════════════
#
# Current (pre-Material2d):
# ```
# Main World RenderApp
# ───────────────────
# fire_entity {  FireParticleSystemResource }
#   ├─ update (main)
#   └─ gpu_fire_particle_raster (render, CustomRenderNode)
#       ├─ fetch uniforms from ViewProjectionAuthority (RGR-V2-001)
#       └─ draw via gpu_buffer_registry
#
# water_entity { WaterParticleSystemResource }
#   ├─ update (main)
#   └─ gpu_water_particle_raster (render, CustomRenderNode)
#       ├─ fetch uniforms from ViewProjectionAuthority (RGR-V2-001)
#       └─ draw via gpu_buffer_registry
# ```
#
# Target (post-Material2d, Phase 1 complete):
# ```
# Main World RenderApp
# ───────────────────
# fire_entity { FireParticleMaterial }
#   ├─ Material2d plugin routes to raster node (Bevy core)
#   └─ render: fire_particle_draw via Material2d
#       ├─ fetch uniforms from ViewProjectionAuthority
#       └─ draw via Material bindings (BufferVec-ready in Phase 2)
#
# water_entity { WaterParticleMaterial }
#   ├─ Material2d plugin routes to raster node (Bevy core)
#   └─ render: water_particle_draw via Material2d
#       ├─ fetch uniforms from ViewProjectionAuthority
#       └─ draw via Material bindings (BufferVec-ready in Phase 2)
# ```
#
# Post-Phase 2 (BufferVec):
# ```
# FireParticleMaterial
#   └─ bind_group: BufferVec<FireParticleInstance>
# WaterParticleMaterial
#   └─ bind_group: BufferVec<WaterParticleInstance>
# # gpu_buffer_registry retired (or repurposed as thin bridge)
# ```
