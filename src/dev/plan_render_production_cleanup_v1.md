# PLAN-RENDER-PROD-CLEANUP-v1 — production-grade render pipeline cleanup

**Status:** ACTIVE · **Priority:** P1 · **Owner:** @coder (exec) / @planner (Phase 2) / operator (Phase 1 gate)
**Queue:** `tools/orchestrator/queues/render_production_cleanup_queue.json`
**Origin:** operator directive 2026-07-06 ("get rid of subs, hacks and sloppy fallbacks; move forward with
our real render pipeline") + cleanup-intelligence audit F1–F16 + VISUAL-REGRESSION-HUNT closure
(see RGR queue RGR-V2-001 `visual_gate_closed` note for the four root causes fixed that day).

## Locks (do not touch)
- `ViewProjectionAuthority` (REVIEW NOTE #3, 15+ dependents) and `stage6_virtualization` (production streaming spine).
- `sim_map_rtt` is the RTT standard; `MINIMAP_GPU_SHADER_FAILED` degrade-with-log mechanism is correct (only its witness sync is F4).
- Verified NON-smells (audit F10): map_camera → ViewProjectionAuthority → extracted_camera_metrics is single-writer/multi-reader; do not "deduplicate".

## Already landed (2026-07-06, VISUAL-REGRESSION-HUNT)
F1 minimap label+routing (world_raster honest input) · MIG-A2 NoCpuCulling retirement (bevy 0.19 invisibility) ·
startup-zoom component write · zoom-band hysteresis (dirty-loop) · witness detector fixes (GPU_RTT_VOID / view_proj_degenerate) ·
day/night clear revival + SimulationMapRttPlugin consolidation · real capture probes (VFX_CAPTURE, RTT_SPRITE_TRACE).

## Phases

### Phase 0 — honesty fixes (mechanical, unblocked)
| id | audit | goal | exit |
|:---|:---|:---|:---|
| RPC-0-001 | F4 | Unify minimap GPU-enablement predicate: `run_minimap_compositor_pass` consults `minimap_gpu_compositor_runtime_enabled()` (incl. `MINIMAP_GPU_SHADER_FAILED`); `composite_path` flips to CpuBridge the frame the shader-failed flag trips | witness `composite_path` can never claim GpuCompute after a shader failure |
| RPC-0-002 | F6/F12/F15 | Hygiene: collapse dead identical if/else in `apply_editor_terrain_authority` (terrain_render_authority.rs:83-89); drop stale `#[allow(dead_code)]` on `arm_visual_test_graceful_exit` (gpu_surface_teardown.rs:29); fix stale doc ref `enable_tile_gpu_instanced_authoritative` (tile_debug_types.rs:72) | cargo check clean, no behavior change |
| RGR-V2-004 | — | (runs in RGR queue, tandem OK) remove deprecated latch witness fields from visual_readiness | render_hole_steady_flip_count sourced from RTT valid streak |

### Phase 1 — GPU terrain decision (BLOCKED_ON_OPERATOR_DECISION)
F2/F3: `TerrainRenderAuthority::GpuInstancedAtlas` promises a GPU world-texture path that does not exist
(terrain pixels are always CPU-rastered; `GpuTilemap` never constructed; per-tile instanced pass permanently
dormant behind `uses_gpu_sprite_display()`).
- **Option A:** build a real GPU world-terrain bake (render-thread sample of `TerrainMaterialAtlasGpu` into a
  world texture; minimap + tactical consume it). Bigger effort; real perf/quality win at large worlds.
- **Option B:** honest rename + simplify: `TerrainRenderAuthority` → `TerrainSourceMode`, delete or
  milestone-gate the dormant per-tile pass (`pipelines/terrain_instanced_draw.rs`), keep CPU raster as the
  one world-texture source. Small effort; removes ambient complexity + label lies.
Doc refs claiming `gpu_atlas` as target: `gpu_todos_v1.md:43`, `plan_gpu_terrain_production_exec_001_v1.md:171,287`,
`visual_test_runbook_v1.md:141` — update per chosen option.

### Phase 2 — pose authority completion (TODO-04)
Finish "ViewManager sole authority for WorldMain camera pose; MapCameraDesired only mirrored" (stage5 TODO-04).
Kills the whole clobber class (startup-zoom bug family; frame-1 `ZOOM_REVERT 1.0→0.02` alpha-state default).
@planner architecture pass first: enumerate writers (map_camera_apply_input, focus_main_camera_on_world_params,
view_representation::apply_minimap_camera_intent), define single commit point + mirror direction, then @coder.

### Phase 3 — structure (after Phase 1 decision)
| id | audit | goal |
|:---|:---|:---|
| RPC-3-001 | F16 | Split `tile_world_fallback.rs` (~1900 LOC): raster / minimap-image / atlas-stamp-queue / egui-panel modules (after Phase 0+1 land — avoid double churn) |
| RPC-3-002 | F7 | Retire ~46 internal-only path shims in render/mod.rs by rewriting ~277 internal imports; keep 4 external pub shims until 9 external call sites repointed (= RGR-M1-002/003 tail) |
| RPC-3-003 | F8/F11 | Renames: `RttDiagCameraMode::Production` → `Standard`; add `test_harness_active` witness field beside `fire_debug_force_visible`; consider `TerrainRenderAuthority`→`TerrainSourceMode` (folds into Phase 1 Option B) |

### Phase 4 — probe/harness tidy
Retire the stub `render/probes/vfx_capture_hook.rs` (writes text files pretending to be PNG capture) in favor
of `dev/diagnostics/visual_capture_probe.rs`; keep `RTT_SPRITE_TRACE` probe (env-gated, zero cost off).
Sweep dead `SIMULATION_MAP_RTT_RENDER_LAYER=1` const (declared, never enforced — latent trap flagged by trace).

## Watch items
- `raster_applied_revision` vs `raster_revision` gap should now converge post-hysteresis; assert in witness (audit F9 / cleanup P7).
- Frame budget: ~74 ms avg in release vfx run with diagnostics on — profile after Phase 0 (dirty-loop removal should already help).
