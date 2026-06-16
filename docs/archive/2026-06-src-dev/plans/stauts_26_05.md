# Operator run status — 2026-05-26


| Field        | Value                                                                            |
| ------------ | -------------------------------------------------------------------------------- |
| **Reporter** | Operator playtest                                                                |
| **Build**    | `cargo run -p proc_A_dine01` (dev profile typical)                               |
| **Verdict**  | **Playable but rough** — schedule startup fixed; runtime quality needs P0 polish |


---

## Symptoms (operator)


| #   | Symptom                                   | Severity | Notes                                                               |
| --- | ----------------------------------------- | -------- | ------------------------------------------------------------------- |
| 1   | **Frame rate horrible**                   | P0       | Dev + FULL_APP spine + readiness + visual harness overhead          |
| 2   | **World map blinks in/out**               | P0       | Overlay / fire extract / representation churn (see VR-05, PLAY-06c) |
| 3   | **Minimap weak** — few options, poor read | P1       | Witness green ≠ product polish; tray exists but shallow UX          |


---

## Likely causes (codebase cross-ref)

### FPS

- `Stage5ReadinessProfile::FULL_APP` runs heavy readiness eval **every frame** (`[stage5_readiness.rs](../render/stage5_readiness.rs)` — console logging alone can cost **~200 ms/frame** when verbose).
- **Dev** profile (no `--release`) + full plugin graph (world-gen, fire extract, minimap compositor, industrial, diagnostics).
- `--test visual` adds harness seeding, proof gates, UX-06 streak — not a “play” mode.
- Mitigations: `--release`, normal menu/demo (no `--test visual`), collapse F3 diagnostics, avoid stay-open unless inspecting.

### World map blink

- `[sync_shared_overlay_from_simulation](../render/extraction/fire_visual_extract.rs)` — **PLAY-06c** holds overlay when sim snapshot empty (anti-blink); still can flicker if heat map toggles every frame.
- `[visual_run_blockers.md](visual_run_blockers.md)` **VR-05** — `fire_inst` flicker (22 → 0) while eval passes.
- **VR-04** — VT-5 spatial invariants intermittent at low fire counts.

### Minimap

- Compositor + overlay mask wired (`[minimap_shell.rs](../gui/minimap_shell.rs)`, `[dock_shell.rs](../gui/hud/dock_shell.rs)` overlay tray).
- **M3 witness closed**; product depth (**zoom, bookmarks, layer UX, GPU compositor defaults**) = P2 — `[m3_minimap_product_depth_plan_v1.md](m3_minimap_product_depth_plan_v1.md)`.
- Default sim mask keeps **fire_heat off** on minimap (pink wash); world tint can still blink from main map overlay path.

---

## Recommended lanes (priority)


| Lane              | Owner              | Exit                                                                                       |                                                                         |     |     |
| ----------------- | ------------------ | ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- | --- | --- |
| **PERF-PLAY-001** | Coder              | `--release` play path ≥30 fps; readiness log throttled; no per-frame stdout in green state |                                                                         |     |     |
| **MAP-BLINK-001** | Coder + debug      | **FIX LANDED** — [`map_blink_001_repro_v1.md`](map_blink_001_repro_v1.md) (PLAY-06d + warmup + projection lag hold) | Verify in `--release` play |     |     |
| **MINIMAP-UX-001** | Designer + coder   | **SPEC PASS** — [`minimap_ux_v1.md`](minimap_ux_v1.md) (tray toggles, zoom, follow, bookmarks) | Coder polish open |     |     |


---

## Commands for next capture

```powershell
# PERF-PLAY-001: throttle targets — do not use bare `RUST_LOG=info` (enables stage5_live_todos spam with STAGE5_VERBOSE).
$env:RUST_LOG='warn,stage5_readiness::live=info,proc_A_dine01=info'
cargo run -p proc_A_dine01 --release
# Or layout test (stays open):
cargo run -p proc_A_dine01 --release -- --test frame
```

Paste stderr highlights (bootstrap, readiness, VT-5, overlay revision) under **Logs** below.

---

## Logs

*(paste terminal excerpts here)*

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:23.233143Z  INFO visual_diag: VISUAL_DIAG window frame=23 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:23.233143Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=23 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=23 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=15 resolved_rev=40 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:23.233293Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=23 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:23.233849Z  INFO visual_diag: VISUAL_DIAG camera frame=23 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=23 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:23.234021Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=23 resolved_rev=40 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:23.234180Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=23 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535055 sim_map_proj_rev=4367996742000

2026-05-26T20:52:23.234256Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=23 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086437940597534 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:23.234381Z  INFO visual_diag: VISUAL_DIAG render_spine frame=23 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=24 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:23.234635Z  INFO visual_diag: VISUAL_DIAG perf frame=23 tile_raster_ms=138.88209533691406 tile_raster_ran=true world_repr_ms=0.19950000941753387 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00010000000474974513 readiness_ms=0.15330000221729279 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:23.234776Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=23 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:23.234887Z  INFO perf: PERF wall=1067.65 instr=139.24 gap=928.41 | cpu_pre_egui=968.12 cpu_egui=97.15 cpu_post_egui=2.38 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.15 raster=138.88 | upd_attrib sum=906.72 pv_cpu=0.00 pv_gpu=0.01 fire=9.43 stream=897.28 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=138.88 hud=0.00 overlay=0.00 raster_b=138.88 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=97.15 | stall first+preupd=0.05 update=0.00 post_dom=139.19 post_vt=32.34 post→ready=0.96 ready=0.34 post→egui=63.62 egui=0.20 post_egui=0.32 | stall_hits=[after_tile_storage_apply:828.9,after_domain_merge:139.2,after_vt_ci:32.3,pre_egui:63.6]

2026-05-26T20:52:23.234992Z  INFO perf: PERF frame=1067.7ms update=968.1ms egui=97.1ms preview=0.0ms streaming=897.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=138.9ms

2026-05-26T20:52:23.235053Z  INFO stall: STALL culprit=after_tile_storage_apply duration=828.9ms frame=1067.7ms

2026-05-26T20:52:23.239165Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:23.240389Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:23.240578Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:23.240646Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:23.240707Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:23.241362Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:24.025833Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=25 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:24.042104Z  INFO stall: STALL after_tile_storage_apply: 804.79ms

2026-05-26T20:52:24.059644Z  INFO stall: STALL upd_streaming_reconstruct: 817.92ms

2026-05-26T20:52:24.194307Z  INFO stall: STALL after_domain_merge: 152.21ms

2026-05-26T20:52:24.194352Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:24.208603Z  INFO stall: STALL after_vt_ci: 14.30ms

2026-05-26T20:52:24.217232Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:24.218529Z  INFO stall: STALL post_egui: 9.46ms

2026-05-26T20:52:24.218690Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:24.218752Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:24.218814Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=25 sim_tick=25) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:24.224686Z  INFO stall: STALL last: 5.88ms

2026-05-26T20:52:24.224706Z  INFO visual_diag: VISUAL_DIAG window frame=24 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:24.224703Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=24 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=24 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=15 resolved_rev=42 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:24.224895Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=24 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:24.225434Z  INFO visual_diag: VISUAL_DIAG camera frame=24 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=24 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:24.225599Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=24 resolved_rev=42 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:24.225760Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=24 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535055 sim_map_proj_rev=4367996742000

2026-05-26T20:52:24.225838Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=24 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.708648443222046 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:24.225970Z  INFO visual_diag: VISUAL_DIAG render_spine frame=24 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=25 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:24.226224Z  INFO visual_diag: VISUAL_DIAG perf frame=24 tile_raster_ms=151.8863067626953 tile_raster_ran=true world_repr_ms=0.19770000874996185 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00010000000474974513 readiness_ms=0.13840000331401825 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:24.226364Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=24 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:24.226480Z  INFO perf: PERF wall=989.19 instr=152.23 gap=836.97 | cpu_pre_egui=957.04 cpu_egui=24.23 cpu_post_egui=7.92 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.14 raster=151.89 | upd_attrib sum=825.68 pv_cpu=0.00 pv_gpu=0.02 fire=7.74 stream=817.92 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=151.89 hud=0.00 overlay=0.00 raster_b=151.89 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=24.23 | stall first+preupd=0.05 update=0.00 post_dom=152.21 post_vt=14.30 post→ready=0.01 ready=0.28 post→egui=0.46 egui=9.46 post_egui=5.88 | stall_hits=[after_tile_storage_apply:804.8,after_domain_merge:152.2,after_vt_ci:14.3,post_egui:9.5,last:5.9]

2026-05-26T20:52:24.226567Z  INFO perf: PERF frame=989.2ms update=957.0ms egui=24.2ms preview=0.0ms streaming=817.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=151.9ms

2026-05-26T20:52:24.226627Z  INFO stall: STALL culprit=after_tile_storage_apply duration=804.8ms frame=989.2ms

2026-05-26T20:52:24.229673Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:24.230336Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:24.230421Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:24.230478Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:24.230537Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:24.231204Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:24.915434Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=26 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:24.915694Z  INFO stall: STALL after_tile_storage_apply: 687.84ms

2026-05-26T20:52:24.916604Z  INFO stall: STALL upd_streaming_reconstruct: 685.04ms

2026-05-26T20:52:25.040685Z  INFO stall: STALL after_domain_merge: 124.99ms

2026-05-26T20:52:25.040742Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:25.041264Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:25.050473Z  INFO stall: STALL post_egui: 9.67ms

2026-05-26T20:52:25.050840Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:25.050941Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:25.051052Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=26 sim_tick=26) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:25.051494Z  INFO visual_diag: VISUAL_DIAG window frame=25 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:25.051703Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=25 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:25.051952Z  INFO visual_diag: VISUAL_DIAG camera frame=25 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=25 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:25.052179Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=25 resolved_rev=42 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:25.052402Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=25 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535055 sim_map_proj_rev=4367996742000

2026-05-26T20:52:25.052526Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=25 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:25.052716Z  INFO visual_diag: VISUAL_DIAG render_spine frame=25 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=26 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:25.053048Z  INFO visual_diag: VISUAL_DIAG perf frame=25 tile_raster_ms=124.59910583496094 tile_raster_ran=true world_repr_ms=0.2092999964952469 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21780000627040863 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:25.053221Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=25 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:25.053360Z  INFO perf: PERF wall=825.60 instr=125.03 gap=700.57 | cpu_pre_egui=812.94 cpu_egui=9.81 cpu_post_egui=2.85 gpu_gap=0.00 | spine=0.00 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=124.60 | upd_attrib sum=694.32 pv_cpu=0.00 pv_gpu=0.01 fire=9.26 stream=685.04 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=124.60 hud=0.00 overlay=0.00 raster_b=124.60 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=9.81 | stall first+preupd=0.11 update=0.00 post_dom=124.99 post_vt=0.11 post→ready=0.00 ready=0.57 post→egui=0.00 egui=9.67 post_egui=0.43 | stall_hits=[after_tile_storage_apply:687.8,after_domain_merge:125.0,post_egui:9.7]

2026-05-26T20:52:25.053476Z  INFO perf: PERF frame=825.6ms update=812.9ms egui=9.8ms preview=0.0ms streaming=685.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=124.6ms

2026-05-26T20:52:25.053567Z  INFO stall: STALL culprit=after_tile_storage_apply duration=687.8ms frame=825.6ms

2026-05-26T20:52:25.057515Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:25.057822Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:25.057932Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:25.058032Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:25.058124Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:25.085545Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:25.748460Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=27 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:25.748675Z  INFO stall: STALL after_tile_storage_apply: 693.26ms

2026-05-26T20:52:25.749567Z  INFO stall: STALL upd_streaming_reconstruct: 663.60ms

2026-05-26T20:52:25.869194Z  INFO stall: STALL after_domain_merge: 120.52ms

2026-05-26T20:52:25.869241Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:25.869725Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:25.880113Z  INFO stall: STALL post_egui: 10.81ms

2026-05-26T20:52:25.880359Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:25.880473Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:25.880592Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=27 sim_tick=27) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:25.881080Z  INFO visual_diag: VISUAL_DIAG window frame=26 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:25.881079Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=26 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=26 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=16 resolved_rev=44 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:25.881256Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=26 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:25.881839Z  INFO visual_diag: VISUAL_DIAG camera frame=26 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=26 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:25.882032Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=26 resolved_rev=44 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:25.882223Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=26 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535055 sim_map_proj_rev=4367996742000

2026-05-26T20:52:25.882330Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=26 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:25.882487Z  INFO visual_diag: VISUAL_DIAG render_spine frame=26 raster_rev=16 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=27 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:25.882772Z  INFO visual_diag: VISUAL_DIAG perf frame=26 tile_raster_ms=118.5835952758789 tile_raster_ran=true world_repr_ms=0.24079999327659607 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.0003000000142492354 readiness_ms=0.24160000681877136 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:25.882941Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=26 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:25.883083Z  INFO perf: PERF wall=827.73 instr=119.07 gap=708.66 | cpu_pre_egui=813.87 cpu_egui=10.92 cpu_post_egui=2.94 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=118.58 | upd_attrib sum=664.19 pv_cpu=0.00 pv_gpu=0.01 fire=0.58 stream=663.60 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=118.58 hud=0.00 overlay=0.00 raster_b=118.58 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.92 | stall first+preupd=0.07 update=0.00 post_dom=120.52 post_vt=0.10 post→ready=0.00 ready=0.47 post→egui=0.00 egui=10.81 post_egui=0.48 | stall_hits=[after_tile_storage_apply:693.3,after_domain_merge:120.5,post_egui:10.8]

2026-05-26T20:52:25.883200Z  INFO perf: PERF frame=827.7ms update=813.9ms egui=10.9ms preview=0.0ms streaming=663.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=118.6ms

2026-05-26T20:52:25.883291Z  INFO stall: STALL culprit=after_tile_storage_apply duration=693.3ms frame=827.7ms

2026-05-26T20:52:25.886909Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:25.887480Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:25.887570Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:25.887652Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:25.887731Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:25.915108Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:26.554660Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=28 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:26.554865Z  INFO stall: STALL after_tile_storage_apply: 669.99ms

2026-05-26T20:52:26.555660Z  INFO stall: STALL upd_streaming_reconstruct: 640.05ms

2026-05-26T20:52:26.680454Z  INFO stall: STALL after_domain_merge: 125.59ms

2026-05-26T20:52:26.680501Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:26.681210Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:26.690917Z  INFO stall: STALL post_egui: 10.34ms

2026-05-26T20:52:26.691142Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:26.691233Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:26.691331Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=28 sim_tick=28) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:26.691711Z  INFO visual_diag: VISUAL_DIAG window frame=27 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:26.691709Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=27 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=27 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=17 resolved_rev=46 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:26.691891Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=27 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:26.692560Z  INFO visual_diag: VISUAL_DIAG camera frame=27 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=27 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:26.692787Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=27 resolved_rev=46 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:26.693012Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=27 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535056 sim_map_proj_rev=4367997742003

2026-05-26T20:52:26.693124Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=27 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:26.693280Z  INFO visual_diag: VISUAL_DIAG render_spine frame=27 raster_rev=17 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=28 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:26.693563Z  INFO visual_diag: VISUAL_DIAG perf frame=27 tile_raster_ms=123.33880615234375 tile_raster_ran=true world_repr_ms=0.30730000138282776 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00020000000949949026 readiness_ms=0.19629999995231628 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:26.693732Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=27 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:26.693847Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 242.4 ms)

2026-05-26T20:52:26.693949Z  INFO perf: PERF wall=809.05 instr=123.85 gap=685.20 | cpu_pre_egui=795.65 cpu_egui=10.48 cpu_post_egui=2.92 gpu_gap=0.00 | spine=0.00 world_repr=0.31 graph=0.00 merge=0.00 atm=0.00 readiness=0.20 raster=123.34 | upd_attrib sum=640.73 pv_cpu=0.00 pv_gpu=0.01 fire=0.67 stream=640.05 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=123.34 hud=0.00 overlay=0.00 raster_b=123.34 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.48 | stall first+preupd=0.07 update=0.00 post_dom=125.59 post_vt=0.12 post→ready=0.00 ready=0.41 post→egui=0.00 egui=10.34 post_egui=0.37 | stall_hits=[after_tile_storage_apply:670.0,after_domain_merge:125.6,post_egui:10.3]

2026-05-26T20:52:26.694065Z  INFO perf: PERF frame=809.1ms update=795.7ms egui=10.5ms preview=0.0ms streaming=640.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=123.3ms

2026-05-26T20:52:26.694154Z  INFO stall: STALL culprit=after_tile_storage_apply duration=670.0ms frame=809.1ms

2026-05-26T20:52:26.697737Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:26.697857Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:26.697972Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:26.698106Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:26.698252Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:26.726656Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:27.355408Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=29 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:27.355615Z  INFO stall: STALL after_tile_storage_apply: 660.14ms

2026-05-26T20:52:27.356176Z  INFO stall: STALL upd_streaming_reconstruct: 629.14ms

2026-05-26T20:52:27.484725Z  INFO stall: STALL after_domain_merge: 129.11ms

2026-05-26T20:52:27.484779Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:27.485647Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:27.495648Z  INFO stall: STALL post_egui: 10.82ms

2026-05-26T20:52:27.496065Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:27.496174Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:27.496286Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=29 sim_tick=29) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:27.496742Z  INFO visual_diag: VISUAL_DIAG window frame=28 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:27.496743Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=28 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=28 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=18 resolved_rev=48 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:27.496947Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=28 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:27.497558Z  INFO visual_diag: VISUAL_DIAG camera frame=28 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=28 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:27.497749Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=28 resolved_rev=48 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:27.497958Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=28 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535057 sim_map_proj_rev=4367998742006

2026-05-26T20:52:27.500507Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=28 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:27.500663Z  INFO visual_diag: VISUAL_DIAG render_spine frame=28 raster_rev=18 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=29 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:27.500951Z  INFO visual_diag: VISUAL_DIAG perf frame=28 tile_raster_ms=126.51960754394531 tile_raster_ran=true world_repr_ms=0.203000009059906 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00010000000474974513 readiness_ms=0.22580000758171082 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:27.501119Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=28 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:27.501258Z  INFO perf: PERF wall=805.82 instr=126.95 gap=678.87 | cpu_pre_egui=789.31 cpu_egui=10.94 cpu_post_egui=5.57 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=126.52 | upd_attrib sum=629.68 pv_cpu=0.00 pv_gpu=0.01 fire=0.52 stream=629.14 map_fit=0.01 hud=0.00 wgen=0.00 | budget_sum=126.52 hud=0.00 overlay=0.00 raster_b=126.52 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.94 | stall first+preupd=0.06 update=0.00 post_dom=129.11 post_vt=0.10 post→ready=0.00 ready=0.63 post→egui=0.00 egui=10.82 post_egui=0.45 | stall_hits=[after_tile_storage_apply:660.1,after_domain_merge:129.1,post_egui:10.8]

2026-05-26T20:52:27.501375Z  INFO perf: PERF frame=805.8ms update=789.3ms egui=10.9ms preview=0.0ms streaming=629.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=126.5ms

2026-05-26T20:52:27.501464Z  INFO stall: STALL culprit=after_tile_storage_apply duration=660.1ms frame=805.8ms

2026-05-26T20:52:27.504600Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:27.504738Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:27.504900Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:27.505001Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:27.505097Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:27.532024Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:28.168343Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=30 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:28.168542Z  INFO stall: STALL after_tile_storage_apply: 666.03ms

2026-05-26T20:52:28.169053Z  INFO stall: STALL upd_streaming_reconstruct: 636.55ms

2026-05-26T20:52:28.301883Z  INFO stall: STALL after_domain_merge: 133.34ms

2026-05-26T20:52:28.301939Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:28.302486Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:28.312533Z  INFO stall: STALL post_egui: 10.54ms

2026-05-26T20:52:28.313205Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:28.313295Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:28.313394Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=30 sim_tick=30) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:28.313771Z  INFO stage5_readiness::live: READINESS_FRAME_FENCE_OK eval_inv=30 frame_tick=30 passes=true

2026-05-26T20:52:28.313783Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=29 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=29 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=19 resolved_rev=50 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:28.313800Z  INFO visual_diag: VISUAL_DIAG window frame=29 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:28.314397Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=29 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:28.314607Z  INFO visual_diag: VISUAL_DIAG camera frame=29 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=29 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:28.314800Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=29 resolved_rev=50 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:28.314991Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=29 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535058 sim_map_proj_rev=4367999742009

2026-05-26T20:52:28.315098Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=29 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:28.315256Z  INFO visual_diag: VISUAL_DIAG render_spine frame=29 raster_rev=19 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=30 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:28.315542Z  INFO visual_diag: VISUAL_DIAG perf frame=29 tile_raster_ms=129.91128540039063 tile_raster_ran=true world_repr_ms=0.28679999709129333 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00020000000949949026 readiness_ms=0.19539999961853027 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:28.315714Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=29 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:28.315851Z  INFO perf: PERF wall=813.38 instr=130.40 gap=682.99 | cpu_pre_egui=799.42 cpu_egui=10.67 cpu_post_egui=3.29 gpu_gap=0.00 | spine=0.00 world_repr=0.29 graph=0.00 merge=0.00 atm=0.00 readiness=0.20 raster=129.91 | upd_attrib sum=637.18 pv_cpu=0.00 pv_gpu=0.01 fire=0.62 stream=636.55 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=129.91 hud=0.00 overlay=0.00 raster_b=129.91 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.67 | stall first+preupd=0.06 update=0.00 post_dom=133.34 post_vt=0.11 post→ready=0.00 ready=0.85 post→egui=0.00 egui=10.54 post_egui=0.38 | stall_hits=[after_tile_storage_apply:666.0,after_domain_merge:133.3,post_egui:10.5]

2026-05-26T20:52:28.315968Z  INFO perf: PERF frame=813.4ms update=799.4ms egui=10.7ms preview=0.0ms streaming=636.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=129.9ms

2026-05-26T20:52:28.316058Z  INFO stall: STALL culprit=after_tile_storage_apply duration=666.0ms frame=813.4ms

2026-05-26T20:52:28.318869Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:28.319146Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:28.319237Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:28.319318Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:28.319402Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:28.347111Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:28.998004Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=31 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:28.998192Z  INFO stall: STALL after_tile_storage_apply: 681.64ms

2026-05-26T20:52:28.999076Z  INFO stall: STALL upd_streaming_reconstruct: 651.56ms

2026-05-26T20:52:29.137320Z  INFO stall: STALL after_domain_merge: 139.13ms

2026-05-26T20:52:29.137366Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:29.137946Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:29.147446Z  INFO stall: STALL post_egui: 10.01ms

2026-05-26T20:52:29.148111Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:29.148216Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:29.148327Z  INFO stage5_readiness::truth:

========== STAGE5_FULL_APP_TRUTH (post_update_invocation=31 sim_tick=31) ==========

FULL_APP_PROFILE_ACTIVE: true

stage5_readiness_passes: true

--- AppStage5ReadinessReport (hard gates) ---

vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]

vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]

single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]

gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]

preview_render_target_active: true  [src: preview_authoritative_surface]

phase_d_ok: true  [derived: !require_preview || preview_render_target_active]

overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]

particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]

phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]

instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]

phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]

projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]

registered_producers: 2

duplicate_visual_scan_count: 0

--- violations (first = primary suspect) ---

first: (none)

all: []

--- input wiring (MISSING = UNKNOWN→operator FAIL) ---

RepresentationResult: true

WorldRepresentationFrame: true

RenderProjectionGraph: true

CommittedVisualSnapshotFence: true

SharedOverlayFieldBuffers: true

GpuRepresentationMetrics: true

VisualAgreementFrame: true

VtCiMatrixLiveReport: true

AtmospherePartialWriteMetrics: true

PreviewCameraState: true

WorldPreviewGpuRuntime: true

PhaseFLodProofReport: true

GpuIndirectDrawSpine: true

WorldFireParticleDrawDispatch: true

FireSimulationSnapshot: true

MISSING_WIRING_FULL_APP: none

================================================================

2026-05-26T20:52:29.148830Z  INFO visual_diag: VISUAL_DIAG window frame=30 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:29.148829Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=30 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=30 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=20 resolved_rev=52 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:29.149038Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=30 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:29.149738Z  INFO visual_diag: VISUAL_DIAG camera frame=30 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=30 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:29.149965Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=30 resolved_rev=52 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:29.150199Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=30 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535059 sim_map_proj_rev=4368000742012

2026-05-26T20:52:29.150306Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=30 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:29.150469Z  INFO visual_diag: VISUAL_DIAG render_spine frame=30 raster_rev=20 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=31 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:29.150755Z  INFO visual_diag: VISUAL_DIAG perf frame=30 tile_raster_ms=135.439697265625 tile_raster_ran=true world_repr_ms=0.1932000070810318 projection_graph_ms=0.000800000037997961 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2190999984741211 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:29.150926Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=30 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:29.151074Z  INFO perf: PERF wall=834.56 instr=135.86 gap=698.70 | cpu_pre_egui=820.82 cpu_egui=10.14 cpu_post_egui=3.59 gpu_gap=0.00 | spine=0.00 world_repr=0.19 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=135.44 | upd_attrib sum=652.11 pv_cpu=0.00 pv_gpu=0.00 fire=0.54 stream=651.56 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=135.44 hud=0.00 overlay=0.00 raster_b=135.44 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.14 | stall first+preupd=0.06 update=0.00 post_dom=139.13 post_vt=0.11 post→ready=0.00 ready=0.87 post→egui=0.00 egui=10.01 post_egui=0.50 | stall_hits=[after_tile_storage_apply:681.6,after_domain_merge:139.1,post_egui:10.0]

2026-05-26T20:52:29.151193Z  INFO perf: PERF frame=834.6ms update=820.8ms egui=10.1ms preview=0.0ms streaming=651.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=135.4ms

2026-05-26T20:52:29.151283Z  INFO stall: STALL culprit=after_tile_storage_apply duration=681.6ms frame=834.6ms

2026-05-26T20:52:29.157057Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:29.158871Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:29.158992Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:29.159120Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:29.159206Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:29.186216Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:29.841453Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=32 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:29.841635Z  INFO stall: STALL after_tile_storage_apply: 686.68ms

2026-05-26T20:52:29.842574Z  INFO stall: STALL upd_streaming_reconstruct: 655.95ms

2026-05-26T20:52:29.984464Z  INFO stall: STALL after_domain_merge: 142.83ms

2026-05-26T20:52:29.984499Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:29.985062Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:29.994785Z  INFO stall: STALL post_egui: 10.20ms

2026-05-26T20:52:29.995490Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:29.995603Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:29.995916Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=31 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=31 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=21 resolved_rev=54 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:29.995925Z  INFO visual_diag: VISUAL_DIAG window frame=31 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:29.996597Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=31 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:29.996847Z  INFO visual_diag: VISUAL_DIAG camera frame=31 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=31 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:29.997074Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=31 resolved_rev=54 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:29.997279Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=31 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535060 sim_map_proj_rev=4368001742015

2026-05-26T20:52:29.997385Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=31 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:29.997542Z  INFO visual_diag: VISUAL_DIAG render_spine frame=31 raster_rev=21 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=32 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:29.997831Z  INFO visual_diag: VISUAL_DIAG perf frame=31 tile_raster_ms=139.06790161132813 tile_raster_ran=true world_repr_ms=0.2393999993801117 projection_graph_ms=0.0010999999940395355 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2345999926328659 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:29.998000Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=31 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:29.998135Z  INFO perf: PERF wall=843.23 instr=139.55 gap=703.69 | cpu_pre_egui=829.58 cpu_egui=10.33 cpu_post_egui=3.32 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=139.07 | upd_attrib sum=656.52 pv_cpu=0.00 pv_gpu=0.01 fire=0.56 stream=655.95 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=139.07 hud=0.00 overlay=0.00 raster_b=139.07 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.33 | stall first+preupd=0.08 update=0.00 post_dom=142.83 post_vt=0.11 post→ready=0.00 ready=0.93 post→egui=0.00 egui=10.20 post_egui=0.19 | stall_hits=[after_tile_storage_apply:686.7,after_domain_merge:142.8,post_egui:10.2]

2026-05-26T20:52:29.998251Z  INFO perf: PERF frame=843.2ms update=829.6ms egui=10.3ms preview=0.0ms streaming=655.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=139.1ms

2026-05-26T20:52:29.998339Z  INFO stall: STALL culprit=after_tile_storage_apply duration=686.7ms frame=843.2ms

2026-05-26T20:52:30.004884Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:30.005017Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:30.005168Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:30.005271Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:30.005368Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:30.032584Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:30.686090Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=33 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:30.686292Z  INFO stall: STALL after_tile_storage_apply: 683.46ms

2026-05-26T20:52:30.687133Z  INFO stall: STALL upd_streaming_reconstruct: 654.10ms

2026-05-26T20:52:30.832540Z  INFO stall: STALL after_domain_merge: 146.25ms

2026-05-26T20:52:30.832597Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:30.833167Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:30.842866Z  INFO stall: STALL post_egui: 10.19ms

2026-05-26T20:52:30.843594Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:30.843700Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:30.844013Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=32 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=32 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=22 resolved_rev=56 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:30.844019Z  INFO visual_diag: VISUAL_DIAG window frame=32 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:30.844677Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=32 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:30.844924Z  INFO visual_diag: VISUAL_DIAG camera frame=32 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=32 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:30.845149Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=32 resolved_rev=56 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:30.845388Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=32 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535061 sim_map_proj_rev=4368002742018

2026-05-26T20:52:30.845510Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=32 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:30.845689Z  INFO visual_diag: VISUAL_DIAG render_spine frame=32 raster_rev=22 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=33 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:30.846024Z  INFO visual_diag: VISUAL_DIAG perf frame=32 tile_raster_ms=141.98338317871094 tile_raster_ran=true world_repr_ms=0.2565999925136566 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2264999896287918 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:30.846223Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=32 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:30.846361Z  INFO perf: PERF wall=843.59 instr=142.47 gap=701.12 | cpu_pre_egui=829.78 cpu_egui=10.34 cpu_post_egui=3.47 gpu_gap=0.00 | spine=0.00 world_repr=0.26 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=141.98 | upd_attrib sum=654.73 pv_cpu=0.00 pv_gpu=0.01 fire=0.60 stream=654.10 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=141.98 hud=0.00 overlay=0.00 raster_b=141.98 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.34 | stall first+preupd=0.08 update=0.00 post_dom=146.25 post_vt=0.12 post→ready=0.00 ready=0.94 post→egui=0.00 egui=10.19 post_egui=0.19 | stall_hits=[after_tile_storage_apply:683.5,after_domain_merge:146.2,post_egui:10.2]

2026-05-26T20:52:30.846478Z  INFO perf: PERF frame=843.6ms update=829.8ms egui=10.3ms preview=0.0ms streaming=654.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=142.0ms

2026-05-26T20:52:30.846567Z  INFO stall: STALL culprit=after_tile_storage_apply duration=683.5ms frame=843.6ms

2026-05-26T20:52:30.850365Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:30.852980Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:30.853085Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:30.853180Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:30.853272Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:30.880430Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:31.512161Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=34 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:31.512341Z  INFO stall: STALL after_tile_storage_apply: 664.38ms

2026-05-26T20:52:31.513154Z  INFO stall: STALL upd_streaming_reconstruct: 632.29ms

2026-05-26T20:52:31.662922Z  INFO stall: STALL after_domain_merge: 150.58ms

2026-05-26T20:52:31.662973Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:31.663509Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:31.673208Z  INFO stall: STALL post_egui: 10.16ms

2026-05-26T20:52:31.673941Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:31.674056Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:31.674366Z  INFO visual_diag: VISUAL_DIAG window frame=33 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:31.674367Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=33 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=33 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=23 resolved_rev=58 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:31.674571Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=33 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:31.675255Z  INFO visual_diag: VISUAL_DIAG camera frame=33 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=33 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:31.675477Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=33 resolved_rev=58 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:31.675700Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=33 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535062 sim_map_proj_rev=4368003742021

2026-05-26T20:52:31.675821Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=33 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:31.676001Z  INFO visual_diag: VISUAL_DIAG render_spine frame=33 raster_rev=23 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=34 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:31.676326Z  INFO visual_diag: VISUAL_DIAG perf frame=33 tile_raster_ms=146.0240936279297 tile_raster_ran=true world_repr_ms=0.24860000610351563 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2459000051021576 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:31.676496Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=33 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:31.676635Z  INFO perf: PERF wall=828.72 instr=146.52 gap=682.20 | cpu_pre_egui=815.02 cpu_egui=10.30 cpu_post_egui=3.39 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.25 raster=146.02 | upd_attrib sum=632.87 pv_cpu=0.00 pv_gpu=0.01 fire=0.56 stream=632.29 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=146.02 hud=0.00 overlay=0.00 raster_b=146.02 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=10.30 | stall first+preupd=0.07 update=0.00 post_dom=150.58 post_vt=0.12 post→ready=0.00 ready=0.96 post→egui=0.00 egui=10.16 post_egui=0.18 | stall_hits=[after_tile_storage_apply:664.4,after_domain_merge:150.6,post_egui:10.2]

2026-05-26T20:52:31.676755Z  INFO perf: PERF frame=828.7ms update=815.0ms egui=10.3ms preview=0.0ms streaming=632.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=146.0ms

2026-05-26T20:52:31.676845Z  INFO stall: STALL culprit=after_tile_storage_apply duration=664.4ms frame=828.7ms

2026-05-26T20:52:31.679729Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:31.680053Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:31.680158Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:31.680251Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:31.680345Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:31.708210Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:32.352250Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=35 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=true

2026-05-26T20:52:32.352513Z  INFO stall: STALL after_tile_storage_apply: 674.80ms

2026-05-26T20:52:32.353186Z  INFO stall: STALL upd_streaming_reconstruct: 644.54ms

2026-05-26T20:52:32.508957Z  INFO stall: STALL after_domain_merge: 156.45ms

2026-05-26T20:52:32.508999Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:32.509869Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:32.518932Z  INFO stall: STALL post_egui: 9.87ms

2026-05-26T20:52:32.519691Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:32.519794Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:32.520096Z  INFO visual_diag: VISUAL_DIAG window frame=34 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:32.520097Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=34 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=34 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=24 resolved_rev=60 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:32.520303Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=34 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:32.521001Z  INFO visual_diag: VISUAL_DIAG camera frame=34 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=34 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:32.521227Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=34 resolved_rev=60 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:32.521453Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=34 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535063 sim_map_proj_rev=4368004742024

2026-05-26T20:52:32.521574Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=34 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:32.521754Z  INFO visual_diag: VISUAL_DIAG render_spine frame=34 raster_rev=24 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=35 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:32.522092Z  INFO visual_diag: VISUAL_DIAG perf frame=34 tile_raster_ms=151.37159729003906 tile_raster_ran=true world_repr_ms=0.24269999563694 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00010000000474974513 readiness_ms=0.22179999947547913 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:32.522293Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=34 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:32.522447Z  INFO perf: PERF wall=844.77 instr=151.84 gap=692.93 | cpu_pre_egui=831.30 cpu_egui=9.99 cpu_post_egui=3.48 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=151.37 | upd_attrib sum=645.15 pv_cpu=0.00 pv_gpu=0.01 fire=0.60 stream=644.54 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=151.37 hud=0.00 overlay=0.00 raster_b=151.37 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=9.99 | stall first+preupd=0.06 update=0.00 post_dom=156.45 post_vt=0.10 post→ready=0.00 ready=0.97 post→egui=0.00 egui=9.87 post_egui=0.18 | stall_hits=[after_tile_storage_apply:674.8,after_domain_merge:156.4,post_egui:9.9]

2026-05-26T20:52:32.522577Z  INFO perf: PERF frame=844.8ms update=831.3ms egui=10.0ms preview=0.0ms streaming=644.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=151.4ms

2026-05-26T20:52:32.522674Z  INFO stall: STALL culprit=after_tile_storage_apply duration=674.8ms frame=844.8ms

2026-05-26T20:52:32.526111Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:32.526230Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:32.526344Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:32.526463Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:32.526614Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:32.555437Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:33.206853Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=36 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:33.207064Z  INFO stall: STALL after_tile_storage_apply: 683.43ms

2026-05-26T20:52:33.207606Z  INFO stall: STALL upd_streaming_reconstruct: 651.71ms

2026-05-26T20:52:33.367571Z  INFO stall: STALL after_domain_merge: 160.51ms

2026-05-26T20:52:33.367624Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:33.368177Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:33.377006Z  INFO stall: STALL post_egui: 9.31ms

2026-05-26T20:52:33.377843Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:33.377950Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:33.378250Z  INFO visual_diag: VISUAL_DIAG window frame=35 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:33.378251Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=35 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=35 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=25 resolved_rev=60 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:33.378458Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=35 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:33.379147Z  INFO visual_diag: VISUAL_DIAG camera frame=35 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=35 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:33.379374Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=35 resolved_rev=60 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:33.379595Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=35 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535064 sim_map_proj_rev=4368005742027

2026-05-26T20:52:33.379716Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=35 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:33.379897Z  INFO visual_diag: VISUAL_DIAG render_spine frame=35 raster_rev=25 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=36 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:33.380231Z  INFO visual_diag: VISUAL_DIAG perf frame=35 tile_raster_ms=154.97030639648438 tile_raster_ran=true world_repr_ms=0.2616000175476074 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2231999933719635 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:33.380414Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=35 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:33.380562Z  INFO perf: PERF wall=856.97 instr=155.46 gap=701.51 | cpu_pre_egui=844.01 cpu_egui=9.45 cpu_post_egui=3.51 gpu_gap=0.00 | spine=0.00 world_repr=0.26 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=154.97 | upd_attrib sum=652.36 pv_cpu=0.00 pv_gpu=0.03 fire=0.62 stream=651.71 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=154.97 hud=0.00 overlay=0.00 raster_b=154.97 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=9.45 | stall first+preupd=0.07 update=0.00 post_dom=160.51 post_vt=0.12 post→ready=0.00 ready=1.05 post→egui=0.00 egui=9.31 post_egui=0.18 | stall_hits=[after_tile_storage_apply:683.4,after_domain_merge:160.5,post_egui:9.3]

2026-05-26T20:52:33.380683Z  INFO perf: PERF frame=857.0ms update=844.0ms egui=9.4ms preview=0.0ms streaming=651.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=155.0ms

2026-05-26T20:52:33.380773Z  INFO stall: STALL culprit=after_tile_storage_apply duration=683.4ms frame=857.0ms

2026-05-26T20:52:33.385417Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:33.388360Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:33.388511Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:33.388674Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:33.388765Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:33.522156Z  INFO proc_A_dine01::terrain::generation::world_generator_enhanced: World generation completed (Full)

2026-05-26T20:52:33.550781Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:34.192587Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=37 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:34.192810Z  INFO stall: STALL after_tile_storage_apply: 809.74ms

2026-05-26T20:52:34.193272Z  INFO stall: STALL upd_streaming_reconstruct: 642.05ms

2026-05-26T20:52:34.354180Z  INFO stall: STALL after_domain_merge: 161.37ms

2026-05-26T20:52:34.354238Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:34.354828Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:34.363315Z  INFO stall: STALL post_egui: 8.98ms

2026-05-26T20:52:34.364315Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:34.364421Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:34.364724Z  INFO visual_diag: VISUAL_DIAG window frame=36 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:34.364724Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=36 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=36 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=27 resolved_rev=60 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:34.364928Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=36 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:34.365641Z  INFO visual_diag: VISUAL_DIAG camera frame=36 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=false render_hole=false latch_invalid_streak=36 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:34.365865Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=36 resolved_rev=60 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:34.366086Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=36 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535065 sim_map_proj_rev=4368006742030

2026-05-26T20:52:34.366208Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=36 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:34.366390Z  INFO visual_diag: VISUAL_DIAG render_spine frame=36 raster_rev=27 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=37 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:34.366674Z  INFO visual_diag: VISUAL_DIAG perf frame=36 tile_raster_ms=156.04791259765625 tile_raster_ran=true world_repr_ms=0.24369999766349792 projection_graph_ms=0.00139999995008111 domain_merge_ms=0.00020000000949949026 readiness_ms=0.22630000114440918 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:34.366842Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=36 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:34.366960Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 247.6 ms)

2026-05-26T20:52:34.367064Z  INFO perf: PERF wall=983.97 instr=156.52 gap=827.44 | cpu_pre_egui=971.18 cpu_egui=9.15 cpu_post_egui=3.64 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=156.05 | upd_attrib sum=642.67 pv_cpu=0.00 pv_gpu=0.01 fire=0.60 stream=642.05 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=156.05 hud=0.00 overlay=0.00 raster_b=156.05 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=9.15 | stall first+preupd=0.07 update=0.00 post_dom=161.37 post_vt=0.13 post→ready=0.01 ready=1.21 post→egui=0.01 egui=8.98 post_egui=0.18 | stall_hits=[after_tile_storage_apply:809.7,after_domain_merge:161.4,post_egui:9.0]

2026-05-26T20:52:34.367180Z  INFO perf: PERF frame=984.0ms update=971.2ms egui=9.1ms preview=0.0ms streaming=642.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=156.0ms

2026-05-26T20:52:34.367269Z  INFO stall: STALL culprit=after_tile_storage_apply duration=809.7ms frame=984.0ms

2026-05-26T20:52:34.369527Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="flow_full_ready" latch_dismissed=true world_gen_visible=false preview_window_open=false

2026-05-26T20:52:34.371405Z  INFO world_gen::flow: FullReady: auto-dismissed World Generator panel and World Preview window (reopen via Escape → pause or F8)

2026-05-26T20:52:34.372194Z  INFO proc_A_dine01::gui::editor::world_preview::preview_readiness: PREVIEW STATE: world=true cam=true tex=true proj=true state=Ready world_ready=true camera_ready=true texture_ready=true projection_ready=true missing=None contract_valid=true wp_half_x=606.09375 wp_half_y=297.15625 wp_logical_w=1212.1875 wp_logical_h=594.3125 viewport_rev=60

2026-05-26T20:52:34.373363Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:34.373652Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="ux_enter_world" latch_dismissed=true world_gen_visible=false preview_window_open=false

2026-05-26T20:52:34.373656Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:34.373913Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:34.374005Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:34.374095Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:34.374982Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:35.017778Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=38 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:35.017994Z  INFO stall: STALL after_tile_storage_apply: 648.82ms

2026-05-26T20:52:35.019067Z  INFO stall: STALL upd_streaming_reconstruct: 643.66ms

2026-05-26T20:52:35.146795Z  INFO test_harness::logistics: LOG-E01 visual proof: seeded transport_edges=2 logistics_edges=2 overlay_rows=2

2026-05-26T20:52:35.147287Z  INFO stall: STALL after_domain_merge: 129.30ms

2026-05-26T20:52:35.147339Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:35.147550Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:35.156145Z  INFO stall: STALL post_egui: 8.75ms

2026-05-26T20:52:35.156720Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:35.156826Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:35.157136Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(WorldGen)) worldgen=Res(State(Ready)) base=Res(State(Editor)) flow=Res(State(FullReady)) latch_dismissed=true world_gen_visible=false preview_window_open=false lifecycle=Uninitialized last_dismiss="never"

2026-05-26T20:52:35.157153Z  INFO visual_diag: VISUAL_DIAG window frame=37 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=1 base=1 flow=3 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:35.157154Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=37 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=37 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(22, 12) map_view_px=(1920, 1017) raster_rev=27 resolved_rev=60 app=1 base=1 flow=3 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(85.841194, 85.841194, 85.841194) minimap_visible=true

2026-05-26T20:52:35.157520Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=37 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)

2026-05-26T20:52:35.158217Z  INFO visual_diag: VISUAL_DIAG camera frame=37 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=85.84119415283203 latch_hole=false render_hole=false latch_invalid_streak=37 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=22 ortho_fixed_h=12 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:35.158439Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=37 resolved_rev=60 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:35.158638Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=37 world_preview_proj_rev=2551213575047 minimap_proj_rev=1374389535067 sim_map_proj_rev=4368008742036

2026-05-26T20:52:35.158743Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=37 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:35.158899Z  INFO visual_diag: VISUAL_DIAG render_spine frame=37 raster_rev=27 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.6739002466201782 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=38 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0

2026-05-26T20:52:35.159184Z  INFO visual_diag: VISUAL_DIAG perf frame=37 tile_raster_ms=124.47159576416016 tile_raster_ran=true world_repr_ms=0.2556000053882599 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2289000004529953 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:35.159352Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=37 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:35.159490Z  INFO perf: PERF wall=790.38 instr=124.96 gap=665.42 | cpu_pre_egui=778.19 cpu_egui=8.87 cpu_post_egui=3.32 gpu_gap=0.00 | spine=0.00 world_repr=0.26 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=124.47 | upd_attrib sum=644.27 pv_cpu=0.00 pv_gpu=0.01 fire=0.59 stream=643.66 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=124.47 hud=0.00 overlay=0.00 raster_b=124.47 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=8.87 | stall first+preupd=0.09 update=0.00 post_dom=129.30 post_vt=0.10 post→ready=0.00 ready=0.79 post→egui=0.01 egui=8.75 post_egui=0.20 | stall_hits=[after_tile_storage_apply:648.8,after_domain_merge:129.3,post_egui:8.7]

2026-05-26T20:52:35.159608Z  INFO perf: PERF frame=790.4ms update=778.2ms egui=8.9ms preview=0.0ms streaming=643.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=124.5ms

2026-05-26T20:52:35.159697Z  INFO stall: STALL culprit=after_tile_storage_apply duration=648.8ms frame=790.4ms

2026-05-26T20:52:35.161771Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="enter_simulation" latch_dismissed=true world_gen_visible=false preview_window_open=false

2026-05-26T20:52:35.163730Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="ux_on_enter_in_game" latch_dismissed=true world_gen_visible=false preview_window_open=false

2026-05-26T20:52:35.164932Z  INFO proc_A_dine01::render::viewport_pipeline: resolved viewport=minimap_panel revision=61 logical=(260.0,220.0) physical=260x220

2026-05-26T20:52:35.166201Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:35.167539Z  INFO test_harness::fire: spawned test scene chunk slabs nx=10 ny=10 (world 320x320)

2026-05-26T20:52:35.168203Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:35.168353Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=85.8412 world_main_xy=(160.00,160.00) zoom=85.8412 bridge_drift=0.0000

2026-05-26T20:52:35.168461Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:35.168717Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:35.711184Z  INFO test_harness::fire: test scene seeded shared overlay fire cells=28

2026-05-26T20:52:35.714706Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:35.715314Z  INFO world_representation::lod: WorldRepresentation: zoom=73.595 zoom_α=1.000 → LOD band LocalTactical (LT)

2026-05-26T20:52:35.716461Z  INFO stage5_readiness::live: READINESS_PROJECTION_GRAPH_BUILD dom=3 tick=39 order=fire+logistics+ecology fire_inst=0 fire_heat=0 log_rows=0 eco_rows=100 fire_snap=39 log_snap=39 eco_snap=39

2026-05-26T20:52:36.343305Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=39 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:36.343542Z  INFO stall: STALL after_tile_storage_apply: 1182.46ms

2026-05-26T20:52:36.344036Z  INFO stall: STALL upd_streaming_reconstruct: 628.59ms

2026-05-26T20:52:36.550461Z  INFO test_harness::industrial: IND-E02 visual seed: committed concrete_portland chain (mine → kiln → mixer)

2026-05-26T20:52:36.559564Z  INFO test_harness::logistics: S7P-LOG-001: spawned aluminum chain on road tiles [(0, 0), (1, 0), (2, 0)]

2026-05-26T20:52:36.560260Z  INFO stall: STALL after_domain_merge: 216.72ms

2026-05-26T20:52:36.560334Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.560526Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:36.560649Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.560781Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.560914Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.561051Z  WARN viewport_authority::drift: VIEWPORT_DRIFT measured vs committed delta=Vec2(-1920.0, -1017.0) measured=Vec2(0.0, 0.0) committed=Vec2(1920.0, 1017.0) hint="check AuthoritativeViewport vs SimulationMapViewport copy-through"

2026-05-26T20:52:36.561216Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraLatch valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.561355Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.569790Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:36.571669Z  INFO stall: STALL post_egui: 11.28ms

2026-05-26T20:52:36.591899Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:36.592023Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0

2026-05-26T20:52:36.592139Z  INFO stall: STALL after_readiness: 20.47ms

2026-05-26T20:52:36.599261Z  INFO stall: STALL last: 7.12ms

2026-05-26T20:52:36.599275Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(InGame)) worldgen=Res(State(Dismissed)) base=Res(State(Simulation)) flow=Res(State(PreviewReady)) latch_dismissed=true world_gen_visible=false preview_window_open=false lifecycle=Uninitialized last_dismiss="never"

2026-05-26T20:52:36.599287Z  WARN visual_diag::anomaly: RENDER_HOLE_FLIP frame=38 was=false now=true

2026-05-26T20:52:36.599286Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=38 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=1 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=1 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(26, 14) map_view_px=(1920, 1017) raster_rev=29 resolved_rev=61 app=2 base=2 flow=2 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(73.594986, 73.594986, 73.594986) minimap_visible=true

2026-05-26T20:52:36.599310Z  INFO worldgen_chrome::hud: HUD_SHELL_STATE (Editor/Simulation = HUD egui may run; MainMenu = player shell off) base=Res(State(Simulation)) minimap_visible=true overlay_tray=false command_shell=false transmission=false

2026-05-26T20:52:36.599657Z  WARN visual_diag::anomaly: CAMERA_SCISSOR_CHANGED frame=38 was=None now=Some((0, 0, 1920, 1017))

2026-05-26T20:52:36.600194Z  WARN sim_view_sync::anomaly: CAMERA_VIEWPORT_MODE_FLIP (full-window vs map-hole scissor) frame=38 was_hole=false now_hole=true

2026-05-26T20:52:36.600475Z  WARN visual_diag::anomaly: SIM_VIEWPORT_VALIDITY_CHANGED frame=38 was_valid=false now_valid=true

2026-05-26T20:52:36.600585Z  WARN sim_view_sync::anomaly: CAMERA_SCISSOR_CHANGED frame=38 was=None now=Some((0, 0, 1920, 1017))

2026-05-26T20:52:36.600689Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=38 was=0 now=2

2026-05-26T20:52:36.600797Z  WARN sim_view_sync::anomaly: SIM_MAP_VIEWPORT_VALIDITY_CHANGED frame=38 was_valid=false now_valid=true was_adequate=false now_adequate=true

2026-05-26T20:52:36.600901Z  INFO visual_diag: VISUAL_DIAG window frame=38 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=2 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:36.601057Z  WARN sim_view_sync::anomaly: RENDER_MODE_FLIP (map-hole scissor vs full-window — primary blink source) frame=38 was_render_hole=false now_render_hole=true was_scissor=None now_scissor=Some((0, 0, 1920, 1017))

2026-05-26T20:52:36.603083Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=38 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=false measured_wh=(0, 0) committed_wh=(1920, 1017) sim_held=false settle_streak=1 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:36.603424Z  INFO visual_diag: VISUAL_DIAG camera frame=38 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=1 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:36.603628Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=38 resolved_rev=61 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:36.603830Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=38 world_preview_proj_rev=2551213575047 minimap_proj_rev=944892805407 sim_map_proj_rev=4368008742036

2026-05-26T20:52:36.603940Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=38 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:36.604105Z  INFO visual_diag: VISUAL_DIAG render_spine frame=38 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.6739002466201782 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=39 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=2 gpu_draw=0

2026-05-26T20:52:36.604400Z  INFO visual_diag: VISUAL_DIAG perf frame=38 tile_raster_ms=202.5100860595703 tile_raster_ran=true world_repr_ms=0.21870000660419464 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2622999846935272 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:36.604578Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=38 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:36.604728Z  INFO perf: PERF wall=1443.71 instr=203.00 gap=1240.71 | cpu_pre_egui=1399.27 cpu_egui=11.41 cpu_post_egui=33.02 gpu_gap=0.00 | spine=0.01 world_repr=0.22 graph=0.00 merge=0.00 atm=0.00 readiness=0.26 raster=202.51 | upd_attrib sum=632.83 pv_cpu=0.00 pv_gpu=0.02 fire=4.21 stream=628.59 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=205.31 hud=0.00 overlay=0.00 raster_b=202.51 particles=0.00 residency=2.70 tex_reg=0.00 render_x=0.10 | egui_unbudgeted=11.41 | stall first+preupd=0.09 update=0.01 post_dom=216.72 post_vt=0.11 post→ready=0.00 ready=20.47 post→egui=0.01 egui=11.28 post_egui=7.12 | stall_hits=[after_tile_storage_apply:1182.5,after_domain_merge:216.7,post_egui:11.3,after_readiness:20.5,last:7.1]

2026-05-26T20:52:36.604852Z  INFO perf: PERF frame=1443.7ms update=1399.3ms egui=11.4ms preview=0.0ms streaming=628.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=202.5ms

2026-05-26T20:52:36.604947Z  INFO stall: STALL culprit=after_tile_storage_apply duration=1182.5ms frame=1443.7ms

2026-05-26T20:52:36.609337Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:36.609493Z  INFO proc_A_dine01::render::viewport_pipeline: resolved viewport=simulation_map revision=62 logical=(1920.0,1017.0) physical=1920x1017

2026-05-26T20:52:36.610855Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:36.610952Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:36.611035Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:36.611113Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:36.785126Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:36.785797Z  INFO economy::activation::ind_e03: IND-E03: spawned grid overload cluster for witness depth

2026-05-26T20:52:36.819952Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:36.820393Z  INFO world_representation::lod: WorldRepresentation: zoom=73.595 zoom_α=1.000 → LOD band Operational (OP)

2026-05-26T20:52:37.470809Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=40 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=1 atm_full_field_fallback=false

2026-05-26T20:52:37.471060Z  INFO stall: STALL after_tile_storage_apply: 863.26ms

2026-05-26T20:52:37.471997Z  INFO stall: STALL upd_streaming_reconstruct: 651.01ms

2026-05-26T20:52:37.647376Z  INFO stall: STALL after_domain_merge: 176.32ms

2026-05-26T20:52:37.647488Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:37.647937Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:37.648088Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:37.648246Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:37.648404Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:37.648585Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:37.656283Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:37.656937Z  INFO stall: STALL post_egui: 9.44ms

2026-05-26T20:52:37.658727Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:37.658830Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:37.665335Z  INFO stall: STALL last: 6.40ms

2026-05-26T20:52:37.665350Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(InGame)) worldgen=Res(State(Dismissed)) base=Res(State(Simulation)) flow=Res(State(FullReady)) latch_dismissed=true world_gen_visible=false preview_window_open=false lifecycle=Uninitialized last_dismiss="never"

2026-05-26T20:52:37.665365Z  WARN visual_diag::anomaly: RESOLVED_SIM_MAP_VALIDITY_CHANGED frame=39 was=false now=true

2026-05-26T20:52:37.665366Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=39 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=2 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=2 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(26, 14) map_view_px=(1920, 1017) raster_rev=29 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(73.594986, 73.594986, 73.594986) minimap_visible=true

2026-05-26T20:52:37.665718Z  INFO visual_diag: VISUAL_DIAG window frame=39 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:37.668435Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=39 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=false settle_streak=2 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:37.668703Z  INFO visual_diag: VISUAL_DIAG camera frame=39 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=2 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:37.668904Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=39 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:37.669107Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=39 world_preview_proj_rev=2551213575047 minimap_proj_rev=944892805410 sim_map_proj_rev=4368010742042

2026-05-26T20:52:37.669219Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=39 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:37.669383Z  INFO visual_diag: VISUAL_DIAG render_spine frame=39 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=40 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:37.669678Z  INFO visual_diag: VISUAL_DIAG perf frame=39 tile_raster_ms=171.04209899902344 tile_raster_ran=true world_repr_ms=0.19339999556541443 projection_graph_ms=0.001600000075995922 domain_merge_ms=0.00020000000949949026 readiness_ms=0.21979999542236328 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:37.669851Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=39 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:37.670000Z  INFO perf: PERF wall=1062.31 instr=171.49 gap=890.82 | cpu_pre_egui=1039.70 cpu_egui=9.58 cpu_post_egui=13.03 gpu_gap=0.00 | spine=0.03 world_repr=0.19 graph=0.00 merge=0.00 atm=0.03 readiness=0.22 raster=171.04 | upd_attrib sum=654.87 pv_cpu=0.00 pv_gpu=0.02 fire=3.83 stream=651.01 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=173.77 hud=0.00 overlay=0.00 raster_b=171.04 particles=0.00 residency=2.68 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=9.58 | stall first+preupd=0.13 update=0.00 post_dom=176.32 post_vt=0.11 post→ready=0.00 ready=2.00 post→egui=0.01 egui=9.44 post_egui=6.40 | stall_hits=[after_tile_storage_apply:863.3,after_domain_merge:176.3,post_egui:9.4,last:6.4]

2026-05-26T20:52:37.670122Z  INFO perf: PERF frame=1062.3ms update=1039.7ms egui=9.6ms preview=0.0ms streaming=651.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=171.0ms

2026-05-26T20:52:37.670218Z  INFO stall: STALL culprit=after_tile_storage_apply duration=863.3ms frame=1062.3ms

2026-05-26T20:52:37.673057Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:37.674591Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:37.674692Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:37.674787Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:37.674881Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:37.799484Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:37.800147Z  INFO economy::activation::ind_e03: IND-E03: grid overload witness depth green (overload_events_total=1)

2026-05-26T20:52:37.834126Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:38.466794Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=41 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:38.467031Z  INFO stall: STALL after_tile_storage_apply: 794.87ms

2026-05-26T20:52:38.467992Z  INFO stall: STALL upd_streaming_reconstruct: 633.37ms

2026-05-26T20:52:38.645614Z  INFO stall: STALL after_domain_merge: 178.58ms

2026-05-26T20:52:38.645651Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:38.646191Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:38.646319Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:38.646493Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:38.646633Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:38.646785Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:38.655473Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:38.656096Z  INFO stall: STALL post_egui: 10.36ms

2026-05-26T20:52:38.657392Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:38.657522Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:38.663996Z  INFO stall: STALL last: 6.36ms

2026-05-26T20:52:38.664024Z  INFO visual_diag: VISUAL_DIAG window frame=40 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:38.664297Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=40 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=false settle_streak=3 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:38.664549Z  INFO visual_diag: VISUAL_DIAG camera frame=40 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=3 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:38.664744Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=40 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:38.664935Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=40 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805413 sim_map_proj_rev=4368010742042

2026-05-26T20:52:38.665041Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=40 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:38.665206Z  INFO visual_diag: VISUAL_DIAG render_spine frame=40 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=41 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:38.665498Z  INFO visual_diag: VISUAL_DIAG perf frame=40 tile_raster_ms=173.56259155273438 tile_raster_ran=true world_repr_ms=0.2660999894142151 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2548999786376953 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:38.665673Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=40 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:38.665818Z  INFO perf: PERF wall=993.72 instr=174.09 gap=819.63 | cpu_pre_egui=973.54 cpu_egui=10.49 cpu_post_egui=9.69 gpu_gap=0.00 | spine=0.01 world_repr=0.27 graph=0.00 merge=0.00 atm=0.00 readiness=0.25 raster=173.56 | upd_attrib sum=637.03 pv_cpu=0.00 pv_gpu=0.01 fire=3.64 stream=633.37 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.59 hud=0.00 overlay=0.00 raster_b=173.56 particles=0.00 residency=2.98 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.49 | stall first+preupd=0.09 update=0.00 post_dom=178.58 post_vt=0.11 post→ready=0.00 ready=1.54 post→egui=0.00 egui=10.36 post_egui=6.36 | stall_hits=[after_tile_storage_apply:794.9,after_domain_merge:178.6,post_egui:10.4,last:6.4]

2026-05-26T20:52:38.665940Z  INFO perf: PERF frame=993.7ms update=973.5ms egui=10.5ms preview=0.0ms streaming=633.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=173.6ms

2026-05-26T20:52:38.666062Z  INFO stall: STALL culprit=after_tile_storage_apply duration=794.9ms frame=993.7ms

2026-05-26T20:52:38.668601Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:38.669953Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:38.670087Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:38.670190Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:38.670280Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:38.670368Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:38.796967Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:38.830952Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:39.487946Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=42 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:39.488152Z  INFO stall: STALL after_tile_storage_apply: 820.50ms

2026-05-26T20:52:39.489003Z  INFO stall: STALL upd_streaming_reconstruct: 657.54ms

2026-05-26T20:52:39.666837Z  INFO stall: STALL after_domain_merge: 178.68ms

2026-05-26T20:52:39.666862Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:39.667475Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:39.667612Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:39.667750Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:39.667886Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:39.668049Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:39.668213Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:39.677412Z  INFO stall: STALL post_egui: 10.45ms

2026-05-26T20:52:39.679886Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:39.680001Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:39.686595Z  INFO stall: STALL last: 6.49ms

2026-05-26T20:52:39.686622Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=41 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_settled" frozen=true pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=4 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(19, 10) map_view_px=(1920, 1017) raster_rev=29 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(100.12518, 100.12518, 100.12518) minimap_visible=true

2026-05-26T20:52:39.686632Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=41 was=2 now=3

2026-05-26T20:52:39.689371Z  INFO visual_diag: VISUAL_DIAG window frame=41 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:39.689575Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=41 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_settled" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:39.689836Z  INFO visual_diag: VISUAL_DIAG camera frame=41 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=100.12518310546875 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=4 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=19 ortho_fixed_h=10 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:39.690070Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=41 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:39.690306Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=41 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805413 sim_map_proj_rev=4368010742042

2026-05-26T20:52:39.690430Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=41 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:39.690620Z  INFO visual_diag: VISUAL_DIAG render_spine frame=41 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.786729097366333 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=42 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:39.690922Z  INFO visual_diag: VISUAL_DIAG perf frame=41 tile_raster_ms=174.02099609375 tile_raster_ran=true world_repr_ms=0.29749998450279236 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00020000000949949026 readiness_ms=0.24320000410079956 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:39.691101Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=41 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:39.691263Z  INFO perf: PERF wall=1023.67 instr=174.57 gap=849.10 | cpu_pre_egui=999.26 cpu_egui=10.60 cpu_post_egui=13.81 gpu_gap=0.00 | spine=0.00 world_repr=0.30 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=174.02 | upd_attrib sum=661.20 pv_cpu=0.00 pv_gpu=0.02 fire=3.62 stream=657.54 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=177.08 hud=0.00 overlay=0.00 raster_b=174.02 particles=0.00 residency=3.00 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.60 | stall first+preupd=0.09 update=0.00 post_dom=178.68 post_vt=0.13 post→ready=0.00 ready=2.69 post→egui=0.00 egui=10.45 post_egui=6.49 | stall_hits=[after_tile_storage_apply:820.5,after_domain_merge:178.7,post_egui:10.4,last:6.5]

2026-05-26T20:52:39.691391Z  INFO perf: PERF frame=1023.7ms update=999.3ms egui=10.6ms preview=0.0ms streaming=657.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=174.0ms

2026-05-26T20:52:39.691489Z  INFO stall: STALL culprit=after_tile_storage_apply duration=820.5ms frame=1023.7ms

2026-05-26T20:52:39.694608Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:39.695527Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:39.696089Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:39.696194Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:39.696284Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:39.696373Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:39.826626Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=100.1252 world_main_xy=(160.00,160.00) zoom=100.1252 bridge_drift=0.0000

2026-05-26T20:52:39.860604Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:40.513157Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=43 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:40.513368Z  INFO stall: STALL after_tile_storage_apply: 820.19ms

2026-05-26T20:52:40.514251Z  INFO stall: STALL upd_streaming_reconstruct: 653.19ms

2026-05-26T20:52:40.690779Z  INFO stall: STALL after_domain_merge: 177.41ms

2026-05-26T20:52:40.690812Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:40.691410Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:40.691551Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:40.691695Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:40.691826Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:40.691985Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:40.692145Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:40.701876Z  INFO stall: STALL post_egui: 10.97ms

2026-05-26T20:52:40.704130Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:40.704226Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:40.710554Z  INFO stall: STALL last: 6.23ms

2026-05-26T20:52:40.710572Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=42 was=3 now=4

2026-05-26T20:52:40.710577Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=42 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=5 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(26, 14) map_view_px=(1920, 1017) raster_rev=29 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(73.594986, 73.594986, 73.594986) minimap_visible=true

2026-05-26T20:52:40.710780Z  INFO visual_diag: VISUAL_DIAG window frame=42 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:40.711548Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=42 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:40.711812Z  INFO visual_diag: VISUAL_DIAG camera frame=42 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=5 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:40.712042Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=42 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:40.712268Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=42 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805413 sim_map_proj_rev=4368010742042

2026-05-26T20:52:40.712390Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=42 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:40.712572Z  INFO visual_diag: VISUAL_DIAG render_spine frame=42 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.786729097366333 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=43 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:40.712867Z  INFO visual_diag: VISUAL_DIAG perf frame=42 tile_raster_ms=172.7895965576172 tile_raster_ran=true world_repr_ms=0.20579999685287476 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00020000000949949026 readiness_ms=0.20319999754428864 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:40.713037Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=42 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:40.713177Z  INFO perf: PERF wall=1020.05 instr=173.21 gap=846.84 | cpu_pre_egui=997.66 cpu_egui=11.11 cpu_post_egui=11.27 gpu_gap=0.00 | spine=0.01 world_repr=0.21 graph=0.00 merge=0.00 atm=0.01 readiness=0.20 raster=172.79 | upd_attrib sum=654.27 pv_cpu=0.00 pv_gpu=0.04 fire=1.03 stream=653.19 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.54 hud=0.00 overlay=0.00 raster_b=172.79 particles=0.00 residency=2.70 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=11.11 | stall first+preupd=0.07 update=0.00 post_dom=177.41 post_vt=0.12 post→ready=0.00 ready=2.44 post→egui=0.00 egui=10.97 post_egui=6.23 | stall_hits=[after_tile_storage_apply:820.2,after_domain_merge:177.4,post_egui:11.0,last:6.2]

2026-05-26T20:52:40.713294Z  INFO perf: PERF frame=1020.0ms update=997.7ms egui=11.1ms preview=0.0ms streaming=653.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.8ms

2026-05-26T20:52:40.713385Z  INFO stall: STALL culprit=after_tile_storage_apply duration=820.2ms frame=1020.0ms

2026-05-26T20:52:40.718888Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:40.719785Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:40.720162Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:40.721301Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:40.721381Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:40.721462Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:40.845431Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:40.879565Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:41.524324Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=44 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:41.524536Z  INFO stall: STALL after_tile_storage_apply: 806.75ms

2026-05-26T20:52:41.525411Z  INFO stall: STALL upd_streaming_reconstruct: 645.33ms

2026-05-26T20:52:41.700789Z  INFO stall: STALL after_domain_merge: 176.25ms

2026-05-26T20:52:41.700817Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:41.701422Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:41.701564Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:41.701705Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:41.701838Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:41.702008Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:41.702167Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:41.711153Z  INFO stall: STALL post_egui: 10.24ms

2026-05-26T20:52:41.712648Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:41.712764Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:41.719021Z  INFO stall: STALL last: 6.15ms

2026-05-26T20:52:41.719044Z  INFO visual_diag: VISUAL_DIAG window frame=43 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:41.719305Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=43 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:41.719559Z  INFO visual_diag: VISUAL_DIAG camera frame=43 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=6 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:41.719788Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=43 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:41.720009Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=43 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805413 sim_map_proj_rev=4368010742042

2026-05-26T20:52:41.720130Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=43 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:41.720309Z  INFO visual_diag: VISUAL_DIAG render_spine frame=43 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=44 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:41.720652Z  INFO visual_diag: VISUAL_DIAG perf frame=43 tile_raster_ms=171.77938842773438 tile_raster_ran=true world_repr_ms=0.27889999747276306 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2410999983549118 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:41.720821Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=43 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:41.720958Z  INFO perf: PERF wall=1003.22 instr=172.30 gap=830.92 | cpu_pre_egui=983.07 cpu_egui=10.39 cpu_post_egui=9.77 gpu_gap=0.00 | spine=0.00 world_repr=0.28 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=171.78 | upd_attrib sum=648.90 pv_cpu=0.00 pv_gpu=0.01 fire=3.54 stream=645.33 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.71 hud=0.00 overlay=0.00 raster_b=171.78 particles=0.00 residency=2.88 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.39 | stall first+preupd=0.07 update=0.00 post_dom=176.25 post_vt=0.12 post→ready=0.00 ready=1.72 post→egui=0.00 egui=10.24 post_egui=6.15 | stall_hits=[after_tile_storage_apply:806.7,after_domain_merge:176.3,post_egui:10.2,last:6.1]

2026-05-26T20:52:41.721075Z  INFO perf: PERF frame=1003.2ms update=983.1ms egui=10.4ms preview=0.0ms streaming=645.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=171.8ms

2026-05-26T20:52:41.721164Z  INFO stall: STALL culprit=after_tile_storage_apply duration=806.7ms frame=1003.2ms

2026-05-26T20:52:41.723215Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:41.724131Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:41.724547Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:41.724649Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:41.724739Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:41.724826Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:41.848465Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:41.882412Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:42.535150Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=45 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:42.535378Z  INFO stall: STALL after_tile_storage_apply: 812.97ms

2026-05-26T20:52:42.536228Z  INFO stall: STALL upd_streaming_reconstruct: 653.42ms

2026-05-26T20:52:42.713107Z  INFO stall: STALL after_domain_merge: 177.73ms

2026-05-26T20:52:42.713142Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:42.713753Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:42.713882Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:42.714019Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:42.714159Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:42.714324Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:42.714483Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:42.723179Z  INFO stall: STALL post_egui: 9.92ms

2026-05-26T20:52:42.724639Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:42.724751Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:42.731267Z  INFO stall: STALL last: 6.24ms

2026-05-26T20:52:42.731296Z  INFO visual_diag: VISUAL_DIAG window frame=44 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:42.731545Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=44 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:42.731804Z  INFO visual_diag: VISUAL_DIAG camera frame=44 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=7 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:42.732032Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=44 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:42.732255Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=44 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805413 sim_map_proj_rev=4368010742042

2026-05-26T20:52:42.732376Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=44 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:42.732557Z  INFO visual_diag: VISUAL_DIAG render_spine frame=44 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=45 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:42.732908Z  INFO visual_diag: VISUAL_DIAG perf frame=44 tile_raster_ms=172.970703125 tile_raster_ran=true world_repr_ms=0.20569999516010284 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00010000000474974513 readiness_ms=0.3958999812602997 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:42.733107Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=44 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:42.733238Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 249.1 ms)

2026-05-26T20:52:42.733350Z  INFO perf: PERF wall=1010.91 instr=173.58 gap=837.33 | cpu_pre_egui=990.77 cpu_egui=10.09 cpu_post_egui=10.05 gpu_gap=0.00 | spine=0.00 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.40 raster=172.97 | upd_attrib sum=656.66 pv_cpu=0.00 pv_gpu=0.02 fire=3.22 stream=653.42 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.73 hud=0.00 overlay=0.00 raster_b=172.97 particles=0.00 residency=2.71 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.09 | stall first+preupd=0.07 update=0.01 post_dom=177.73 post_vt=0.13 post→ready=0.00 ready=1.85 post→egui=0.01 egui=9.92 post_egui=6.24 | stall_hits=[after_tile_storage_apply:813.0,after_domain_merge:177.7,post_egui:9.9,last:6.2]

2026-05-26T20:52:42.733483Z  INFO perf: PERF frame=1010.9ms update=990.8ms egui=10.1ms preview=0.0ms streaming=653.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=173.0ms

2026-05-26T20:52:42.733584Z  INFO stall: STALL culprit=after_tile_storage_apply duration=813.0ms frame=1010.9ms

2026-05-26T20:52:42.737837Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:42.738720Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:42.739280Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:42.739380Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:42.739468Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:42.739555Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:42.864941Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:42.898763Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:43.537123Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=46 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:43.537340Z  INFO stall: STALL after_tile_storage_apply: 800.50ms

2026-05-26T20:52:43.538244Z  INFO stall: STALL upd_streaming_reconstruct: 639.05ms

2026-05-26T20:52:43.715925Z  INFO stall: STALL after_domain_merge: 178.59ms

2026-05-26T20:52:43.715952Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:43.716546Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:43.716672Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:43.716807Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:43.716941Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:43.717105Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:43.717270Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:43.726188Z  INFO stall: STALL post_egui: 10.13ms

2026-05-26T20:52:43.727538Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:43.727651Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:43.734041Z  INFO stall: STALL last: 6.29ms

2026-05-26T20:52:43.734071Z  INFO visual_diag: VISUAL_DIAG window frame=45 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:43.734375Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=45 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:43.734674Z  INFO visual_diag: VISUAL_DIAG camera frame=45 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=73.59498596191406 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=8 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=26 ortho_fixed_h=14 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:43.734909Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=45 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:43.735134Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=45 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805413 sim_map_proj_rev=4368010742042

2026-05-26T20:52:43.735256Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=45 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:43.735444Z  INFO visual_diag: VISUAL_DIAG render_spine frame=45 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771677494049072 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=46 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:43.735789Z  INFO visual_diag: VISUAL_DIAG perf frame=45 tile_raster_ms=173.72950744628906 tile_raster_ran=true world_repr_ms=0.2442999929189682 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2360999882221222 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:43.735959Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=45 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:43.736102Z  INFO perf: PERF wall=999.32 instr=174.21 gap=825.11 | cpu_pre_egui=979.18 cpu_egui=10.26 cpu_post_egui=9.89 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=173.73 | upd_attrib sum=642.46 pv_cpu=0.00 pv_gpu=0.01 fire=3.38 stream=639.05 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.60 hud=0.00 overlay=0.00 raster_b=173.73 particles=0.00 residency=2.82 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.26 | stall first+preupd=0.08 update=0.00 post_dom=178.59 post_vt=0.12 post→ready=0.01 ready=1.57 post→egui=0.01 egui=10.13 post_egui=6.29 | stall_hits=[after_tile_storage_apply:800.5,after_domain_merge:178.6,post_egui:10.1,last:6.3]

2026-05-26T20:52:43.736221Z  INFO perf: PERF frame=999.3ms update=979.2ms egui=10.3ms preview=0.0ms streaming=639.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=173.7ms

2026-05-26T20:52:43.736310Z  INFO stall: STALL culprit=after_tile_storage_apply duration=800.5ms frame=999.3ms

2026-05-26T20:52:43.741409Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:43.741484Z  INFO test_harness::fire: test scene seeded shared overlay fire cells=28

2026-05-26T20:52:43.742309Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:43.742792Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:43.742881Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:43.742960Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:43.743050Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:43.866701Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=73.5950 world_main_xy=(160.00,160.00) zoom=73.5950 bridge_drift=0.0000

2026-05-26T20:52:43.901740Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:44.556591Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=47 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:44.556823Z  INFO stall: STALL after_tile_storage_apply: 816.28ms

2026-05-26T20:52:44.557690Z  INFO stall: STALL upd_streaming_reconstruct: 655.45ms

2026-05-26T20:52:44.763822Z  INFO stall: STALL after_domain_merge: 207.00ms

2026-05-26T20:52:44.763851Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:44.764509Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:44.764640Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:44.764751Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:44.764897Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:44.765063Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:44.773569Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:44.774207Z  INFO stall: STALL post_egui: 10.23ms

2026-05-26T20:52:44.775572Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:44.775746Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:44.782232Z  INFO stall: STALL last: 6.38ms

2026-05-26T20:52:44.782258Z  INFO visual_diag: VISUAL_DIAG window frame=46 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:44.782254Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=46 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=9 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(22, 12) map_view_px=(1920, 1017) raster_rev=30 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(85.841194, 85.841194, 85.841194) minimap_visible=true

2026-05-26T20:52:44.782566Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=46 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:44.783313Z  INFO visual_diag: VISUAL_DIAG camera frame=46 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=85.84119415283203 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=9 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=22 ortho_fixed_h=12 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:44.783552Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=46 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:44.783787Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=46 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805414 sim_map_proj_rev=4368010742042

2026-05-26T20:52:44.783899Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=46 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:44.784065Z  INFO visual_diag: VISUAL_DIAG render_spine frame=46 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771677494049072 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.6739002466201782 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=47 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:44.784365Z  INFO visual_diag: VISUAL_DIAG perf frame=46 tile_raster_ms=202.25350952148438 tile_raster_ran=true world_repr_ms=0.29019999504089355 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2953000068664551 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:44.784541Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=46 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:44.784693Z  INFO perf: PERF wall=1044.19 instr=202.84 gap=841.34 | cpu_pre_egui=1023.33 cpu_egui=10.40 cpu_post_egui=10.45 gpu_gap=0.00 | spine=0.00 world_repr=0.29 graph=0.00 merge=0.00 atm=0.00 readiness=0.30 raster=202.25 | upd_attrib sum=658.95 pv_cpu=0.00 pv_gpu=0.02 fire=3.48 stream=655.45 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=205.14 hud=0.00 overlay=0.00 raster_b=202.25 particles=0.00 residency=2.84 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.40 | stall first+preupd=0.07 update=0.00 post_dom=207.00 post_vt=0.15 post→ready=0.00 ready=1.64 post→egui=0.01 egui=10.23 post_egui=6.38 | stall_hits=[after_tile_storage_apply:816.3,after_domain_merge:207.0,post_egui:10.2,last:6.4]

2026-05-26T20:52:44.784819Z  INFO perf: PERF frame=1044.2ms update=1023.3ms egui=10.4ms preview=0.0ms streaming=655.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=202.3ms

2026-05-26T20:52:44.784909Z  INFO stall: STALL culprit=after_tile_storage_apply duration=816.3ms frame=1044.2ms

2026-05-26T20:52:44.792241Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:44.793251Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:44.793585Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:44.794041Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:44.794127Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:44.794207Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:44.923801Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=85.8412 world_main_xy=(160.00,160.00) zoom=85.8412 bridge_drift=0.0000

2026-05-26T20:52:44.958350Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:45.600389Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=48 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:45.600632Z  INFO stall: STALL after_tile_storage_apply: 809.75ms

2026-05-26T20:52:45.601440Z  INFO stall: STALL upd_streaming_reconstruct: 642.67ms

2026-05-26T20:52:45.777303Z  INFO stall: STALL after_domain_merge: 176.67ms

2026-05-26T20:52:45.777328Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:45.777866Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:45.777992Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:45.778127Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:45.778261Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:45.778434Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:45.787213Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:45.787759Z  INFO stall: STALL post_egui: 10.34ms

2026-05-26T20:52:45.788998Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:45.789100Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:45.795322Z  INFO stall: STALL last: 6.12ms

2026-05-26T20:52:45.795345Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=47 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=10 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=30 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:52:45.795352Z  INFO visual_diag: VISUAL_DIAG window frame=47 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:45.796073Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=47 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:45.796326Z  INFO visual_diag: VISUAL_DIAG camera frame=47 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=10 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:45.796553Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=47 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:45.796777Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=47 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:45.796887Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=47 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:45.797043Z  INFO visual_diag: VISUAL_DIAG render_spine frame=47 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.6739002466201782 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=48 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:45.797334Z  INFO visual_diag: VISUAL_DIAG perf frame=47 tile_raster_ms=172.0762939453125 tile_raster_ran=true world_repr_ms=0.20630000531673431 projection_graph_ms=0.0019000000320374966 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21809999644756317 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:45.797503Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=47 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:45.797651Z  INFO perf: PERF wall=1006.83 instr=172.51 gap=834.32 | cpu_pre_egui=986.49 cpu_egui=10.48 cpu_post_egui=9.86 gpu_gap=0.00 | spine=0.01 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=172.08 | upd_attrib sum=645.99 pv_cpu=0.00 pv_gpu=0.02 fire=3.29 stream=642.67 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.83 hud=0.00 overlay=0.00 raster_b=172.08 particles=0.00 residency=2.70 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.48 | stall first+preupd=0.09 update=0.00 post_dom=176.67 post_vt=0.11 post→ready=0.00 ready=1.44 post→egui=0.00 egui=10.34 post_egui=6.12 | stall_hits=[after_tile_storage_apply:809.7,after_domain_merge:176.7,post_egui:10.3,last:6.1]

2026-05-26T20:52:45.797782Z  INFO perf: PERF frame=1006.8ms update=986.5ms egui=10.5ms preview=0.0ms streaming=642.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.1ms

2026-05-26T20:52:45.797878Z  INFO stall: STALL culprit=after_tile_storage_apply duration=809.7ms frame=1006.8ms

2026-05-26T20:52:45.800808Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:45.801688Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:45.802178Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:45.804215Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:45.804295Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:45.804374Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:45.928125Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:45.961948Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:46.609790Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=49 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:46.610040Z  INFO stall: STALL after_tile_storage_apply: 810.30ms

2026-05-26T20:52:46.611040Z  INFO stall: STALL upd_streaming_reconstruct: 648.66ms

2026-05-26T20:52:46.786281Z  INFO stall: STALL after_domain_merge: 176.24ms

2026-05-26T20:52:46.786312Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:46.786881Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:46.787006Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:46.787139Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:46.787273Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:46.787434Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:46.787595Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:46.796320Z  INFO stall: STALL post_egui: 9.92ms

2026-05-26T20:52:46.797625Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:46.799000Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:46.805654Z  INFO stall: STALL last: 6.54ms

2026-05-26T20:52:46.805681Z  INFO visual_diag: VISUAL_DIAG window frame=48 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:46.805995Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=48 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:46.806291Z  INFO visual_diag: VISUAL_DIAG camera frame=48 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=11 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:46.806556Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=48 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:46.806821Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=48 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:46.806948Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=48 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:46.807130Z  INFO visual_diag: VISUAL_DIAG render_spine frame=48 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=49 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:46.807479Z  INFO visual_diag: VISUAL_DIAG perf frame=48 tile_raster_ms=171.4145965576172 tile_raster_ran=true world_repr_ms=0.2035999894142151 projection_graph_ms=0.002199999988079071 domain_merge_ms=0.00010000000474974513 readiness_ms=1.5051000118255615 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:46.807683Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=48 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:46.807839Z  INFO perf: PERF wall=1008.18 instr=173.13 gap=835.05 | cpu_pre_egui=986.63 cpu_egui=10.06 cpu_post_egui=11.49 gpu_gap=0.00 | spine=0.01 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=1.51 raster=171.41 | upd_attrib sum=651.95 pv_cpu=0.00 pv_gpu=0.03 fire=3.25 stream=648.66 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.14 hud=0.00 overlay=0.00 raster_b=171.41 particles=0.00 residency=2.68 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.06 | stall first+preupd=0.10 update=0.00 post_dom=176.24 post_vt=0.12 post→ready=0.00 ready=2.79 post→egui=0.00 egui=9.92 post_egui=6.54 | stall_hits=[after_tile_storage_apply:810.3,after_domain_merge:176.2,post_egui:9.9,last:6.5]

2026-05-26T20:52:46.807963Z  INFO perf: PERF frame=1008.2ms update=986.6ms egui=10.1ms preview=0.0ms streaming=648.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=171.4ms

2026-05-26T20:52:46.808052Z  INFO stall: STALL culprit=after_tile_storage_apply duration=810.3ms frame=1008.2ms

2026-05-26T20:52:46.810777Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:46.811569Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:46.812085Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:46.812186Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:46.812295Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:46.812406Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:46.936752Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:46.971417Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:47.609063Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=50 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:47.609292Z  INFO stall: STALL after_tile_storage_apply: 799.80ms

2026-05-26T20:52:47.610225Z  INFO stall: STALL upd_streaming_reconstruct: 638.37ms

2026-05-26T20:52:47.787005Z  INFO stall: STALL after_domain_merge: 177.71ms

2026-05-26T20:52:47.787036Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:47.787608Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:47.787740Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:47.787885Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:47.788018Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:47.788180Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:47.788342Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:47.796854Z  INFO stall: STALL post_egui: 9.73ms

2026-05-26T20:52:47.798383Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:47.798486Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:47.804800Z  INFO stall: STALL last: 6.21ms

2026-05-26T20:52:47.804824Z  INFO visual_diag: VISUAL_DIAG window frame=49 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:47.805089Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=49 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:47.805349Z  INFO visual_diag: VISUAL_DIAG camera frame=49 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=12 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:47.805580Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=49 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:47.805805Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=49 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:47.805926Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=49 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:47.806107Z  INFO visual_diag: VISUAL_DIAG render_spine frame=49 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=50 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:47.806500Z  INFO visual_diag: VISUAL_DIAG perf frame=49 tile_raster_ms=172.58541870117188 tile_raster_ran=true world_repr_ms=0.24300000071525574 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.22119998931884766 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:47.806700Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=49 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:47.806861Z  INFO perf: PERF wall=997.40 instr=173.05 gap=824.35 | cpu_pre_egui=977.56 cpu_egui=9.87 cpu_post_egui=9.96 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=172.59 | upd_attrib sum=642.02 pv_cpu=0.00 pv_gpu=0.01 fire=3.63 stream=638.37 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.72 hud=0.00 overlay=0.00 raster_b=172.59 particles=0.00 residency=3.08 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=9.87 | stall first+preupd=0.06 update=0.00 post_dom=177.71 post_vt=0.11 post→ready=0.00 ready=1.74 post→egui=0.00 egui=9.73 post_egui=6.21 | stall_hits=[after_tile_storage_apply:799.8,after_domain_merge:177.7,post_egui:9.7,last:6.2]

2026-05-26T20:52:47.806986Z  INFO perf: PERF frame=997.4ms update=977.6ms egui=9.9ms preview=0.0ms streaming=638.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.6ms

2026-05-26T20:52:47.807076Z  INFO stall: STALL culprit=after_tile_storage_apply duration=799.8ms frame=997.4ms

2026-05-26T20:52:47.810492Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:47.811324Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:47.811645Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:47.813726Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:47.813813Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:47.813897Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:47.938533Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:47.971988Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:48.628300Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=51 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=true

2026-05-26T20:52:48.628516Z  INFO stall: STALL after_tile_storage_apply: 819.14ms

2026-05-26T20:52:48.629428Z  INFO stall: STALL upd_streaming_reconstruct: 656.92ms

2026-05-26T20:52:48.805392Z  INFO stall: STALL after_domain_merge: 176.88ms

2026-05-26T20:52:48.805420Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:48.805990Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:48.806115Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:48.806250Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:48.806405Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:48.806572Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:48.806734Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:48.815333Z  INFO stall: STALL post_egui: 9.85ms

2026-05-26T20:52:48.816772Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:48.816874Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:48.823682Z  INFO stall: STALL last: 6.70ms

2026-05-26T20:52:48.823710Z  INFO visual_diag: VISUAL_DIAG window frame=50 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:48.823976Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=50 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:48.824286Z  INFO visual_diag: VISUAL_DIAG camera frame=50 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=13 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:48.824557Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=50 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:48.824820Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=50 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:48.824956Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=50 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:48.825135Z  INFO visual_diag: VISUAL_DIAG render_spine frame=50 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=51 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:48.825482Z  INFO visual_diag: VISUAL_DIAG perf frame=50 tile_raster_ms=172.25009155273438 tile_raster_ran=true world_repr_ms=0.2957000136375427 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00010000000474974513 readiness_ms=0.22220000624656677 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:48.825680Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=50 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:48.825833Z  INFO perf: PERF wall=1016.50 instr=172.77 gap=843.73 | cpu_pre_egui=996.08 cpu_egui=9.95 cpu_post_egui=10.47 gpu_gap=0.00 | spine=0.00 world_repr=0.30 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=172.25 | upd_attrib sum=660.34 pv_cpu=0.00 pv_gpu=0.01 fire=3.41 stream=656.92 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.06 hud=0.00 overlay=0.00 raster_b=172.25 particles=0.00 residency=2.76 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=9.95 | stall first+preupd=0.07 update=0.00 post_dom=176.88 post_vt=0.09 post→ready=0.00 ready=1.65 post→egui=0.00 egui=9.85 post_egui=6.70 | stall_hits=[after_tile_storage_apply:819.1,after_domain_merge:176.9,post_egui:9.8,last:6.7]

2026-05-26T20:52:48.825963Z  INFO perf: PERF frame=1016.5ms update=996.1ms egui=10.0ms preview=0.0ms streaming=656.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=172.3ms

2026-05-26T20:52:48.826056Z  INFO stall: STALL culprit=after_tile_storage_apply duration=819.1ms frame=1016.5ms

2026-05-26T20:52:48.829947Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:48.830685Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:48.831158Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:48.831255Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:48.831342Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:48.831429Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:48.956477Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:48.990656Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:49.666230Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=52 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:49.666449Z  INFO stall: STALL after_tile_storage_apply: 837.55ms

2026-05-26T20:52:49.667372Z  INFO stall: STALL upd_streaming_reconstruct: 676.24ms

2026-05-26T20:52:49.844838Z  INFO stall: STALL after_domain_merge: 178.39ms

2026-05-26T20:52:49.844873Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:49.845393Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:49.845520Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:49.845655Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:49.845788Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:49.845950Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:49.846115Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:49.854971Z  INFO stall: STALL post_egui: 10.02ms

2026-05-26T20:52:49.856365Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:49.856469Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:49.862782Z  INFO stall: STALL last: 6.21ms

2026-05-26T20:52:49.862813Z  INFO visual_diag: VISUAL_DIAG window frame=51 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:49.863121Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=51 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:49.863417Z  INFO visual_diag: VISUAL_DIAG camera frame=51 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=14 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:49.863683Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=51 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:49.863942Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=51 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:49.864065Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=51 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:49.864246Z  INFO visual_diag: VISUAL_DIAG render_spine frame=51 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=52 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:49.864595Z  INFO visual_diag: VISUAL_DIAG perf frame=51 tile_raster_ms=173.4967041015625 tile_raster_ran=true world_repr_ms=0.24990001320838928 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2199999988079071 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:49.864793Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=51 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:49.864955Z  INFO perf: PERF wall=1036.10 instr=173.97 gap=862.13 | cpu_pre_egui=1016.00 cpu_egui=10.16 cpu_post_egui=9.94 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=173.50 | upd_attrib sum=679.74 pv_cpu=0.00 pv_gpu=0.01 fire=3.48 stream=676.24 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.37 hud=0.00 overlay=0.00 raster_b=173.50 particles=0.00 residency=2.83 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.16 | stall first+preupd=0.07 update=0.00 post_dom=178.39 post_vt=0.11 post→ready=0.00 ready=1.60 post→egui=0.00 egui=10.02 post_egui=6.21 | stall_hits=[after_tile_storage_apply:837.6,after_domain_merge:178.4,post_egui:10.0,last:6.2]

2026-05-26T20:52:49.865074Z  INFO perf: PERF frame=1036.1ms update=1016.0ms egui=10.2ms preview=0.0ms streaming=676.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=173.5ms

2026-05-26T20:52:49.865163Z  INFO stall: STALL culprit=after_tile_storage_apply duration=837.6ms frame=1036.1ms

2026-05-26T20:52:49.868503Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:49.869372Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:49.869962Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:49.870064Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:49.870308Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:49.870400Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:49.995884Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:50.029567Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:50.668459Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=53 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:50.668686Z  INFO stall: STALL after_tile_storage_apply: 801.14ms

2026-05-26T20:52:50.669677Z  INFO stall: STALL upd_streaming_reconstruct: 639.66ms

2026-05-26T20:52:50.845990Z  INFO stall: STALL after_domain_merge: 177.30ms

2026-05-26T20:52:50.846020Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:50.846537Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:50.846669Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:50.846812Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:50.846952Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:50.847121Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:50.847284Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:50.856403Z  INFO stall: STALL post_egui: 10.29ms

2026-05-26T20:52:50.857746Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:50.857867Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:50.864244Z  INFO stall: STALL last: 6.27ms

2026-05-26T20:52:50.864277Z  INFO visual_diag: VISUAL_DIAG window frame=52 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:50.864583Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=52 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:50.864880Z  INFO visual_diag: VISUAL_DIAG camera frame=52 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=15 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:50.865124Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=52 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:50.865351Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=52 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:50.865476Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=52 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:50.865658Z  INFO visual_diag: VISUAL_DIAG render_spine frame=52 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=53 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:50.866012Z  INFO visual_diag: VISUAL_DIAG perf frame=52 tile_raster_ms=172.47129821777344 tile_raster_ran=true world_repr_ms=0.24550001323223114 projection_graph_ms=0.00139999995008111 domain_merge_ms=0.00020000000949949026 readiness_ms=0.24089999496936798 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:50.866181Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=52 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:50.866298Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 249.7 ms)

2026-05-26T20:52:50.866400Z  INFO perf: PERF wall=998.82 instr=172.96 gap=825.86 | cpu_pre_egui=978.51 cpu_egui=10.43 cpu_post_egui=9.89 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=172.47 | upd_attrib sum=643.13 pv_cpu=0.00 pv_gpu=0.01 fire=3.45 stream=639.66 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.42 hud=0.00 overlay=0.00 raster_b=172.47 particles=0.00 residency=2.90 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.43 | stall first+preupd=0.07 update=0.00 post_dom=177.30 post_vt=0.11 post→ready=0.00 ready=1.57 post→egui=0.01 egui=10.29 post_egui=6.27 | stall_hits=[after_tile_storage_apply:801.1,after_domain_merge:177.3,post_egui:10.3,last:6.3]

2026-05-26T20:52:50.866516Z  INFO perf: PERF frame=998.8ms update=978.5ms egui=10.4ms preview=0.0ms streaming=639.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=172.5ms

2026-05-26T20:52:50.866605Z  INFO stall: STALL culprit=after_tile_storage_apply duration=801.1ms frame=998.8ms

2026-05-26T20:52:50.870544Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:50.871242Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:50.871938Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:50.872047Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:50.872140Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:50.872227Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:50.996264Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:51.030965Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:51.699300Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=54 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:51.699540Z  INFO stall: STALL after_tile_storage_apply: 830.26ms

2026-05-26T20:52:51.700523Z  INFO stall: STALL upd_streaming_reconstruct: 669.15ms

2026-05-26T20:52:51.878154Z  INFO stall: STALL after_domain_merge: 178.62ms

2026-05-26T20:52:51.878209Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:51.878705Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:51.878830Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:51.878965Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:51.879103Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:51.879275Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:51.879435Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:51.888198Z  INFO stall: STALL post_egui: 9.93ms

2026-05-26T20:52:51.889532Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:51.889635Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:51.896304Z  INFO stall: STALL last: 6.57ms

2026-05-26T20:52:51.896336Z  INFO visual_diag: VISUAL_DIAG window frame=53 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:51.896589Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=53 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:51.896845Z  INFO visual_diag: VISUAL_DIAG camera frame=53 cam_desired_x=160.0 cam_desired_y=245.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=16 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:51.897079Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=53 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:51.897307Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=53 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:51.897430Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=53 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:51.897618Z  INFO visual_diag: VISUAL_DIAG render_spine frame=53 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=54 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:51.897965Z  INFO visual_diag: VISUAL_DIAG perf frame=53 tile_raster_ms=173.3568878173828 tile_raster_ran=true world_repr_ms=0.20559999346733093 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21880000829696655 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:51.898147Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=53 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:51.898288Z  INFO perf: PERF wall=1029.06 instr=173.79 gap=855.28 | cpu_pre_egui=1008.94 cpu_egui=10.07 cpu_post_egui=10.06 gpu_gap=0.00 | spine=0.00 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=173.36 | upd_attrib sum=672.42 pv_cpu=0.00 pv_gpu=0.02 fire=3.25 stream=669.15 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.12 hud=0.00 overlay=0.00 raster_b=173.36 particles=0.00 residency=2.71 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.07 | stall first+preupd=0.07 update=0.00 post_dom=178.62 post_vt=0.11 post→ready=0.00 ready=1.54 post→egui=0.00 egui=9.93 post_egui=6.57 | stall_hits=[after_tile_storage_apply:830.3,after_domain_merge:178.6,post_egui:9.9,last:6.6]

2026-05-26T20:52:51.898405Z  INFO perf: PERF frame=1029.1ms update=1008.9ms egui=10.1ms preview=0.0ms streaming=669.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=173.4ms

2026-05-26T20:52:51.898502Z  INFO stall: STALL culprit=after_tile_storage_apply duration=830.3ms frame=1029.1ms

2026-05-26T20:52:51.904191Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:51.905214Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:51.905635Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:51.905740Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:51.905831Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:51.905917Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:52.031848Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,245.00) zoom=108.1352 world_main_xy=(160.00,245.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:52.065639Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:52.721540Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=55 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:52.721762Z  INFO stall: STALL after_tile_storage_apply: 818.50ms

2026-05-26T20:52:52.722681Z  INFO stall: STALL upd_streaming_reconstruct: 656.59ms

2026-05-26T20:52:52.899254Z  INFO stall: STALL after_domain_merge: 177.49ms

2026-05-26T20:52:52.899287Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:52.899836Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:52.899963Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:52.900104Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:52.900241Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:52.900402Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:52.900560Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:52.909630Z  INFO stall: STALL post_egui: 10.26ms

2026-05-26T20:52:52.911049Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:52.911163Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:52.917577Z  INFO stall: STALL last: 6.30ms

2026-05-26T20:52:52.917597Z  INFO visual_diag: VISUAL_DIAG window frame=54 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:52.917862Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=54 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:52.918118Z  INFO visual_diag: VISUAL_DIAG camera frame=54 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=17 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:52.918352Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=54 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:52.918587Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=54 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:52.918714Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=54 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:52.918902Z  INFO visual_diag: VISUAL_DIAG render_spine frame=54 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=55 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:52.919256Z  INFO visual_diag: VISUAL_DIAG perf frame=54 tile_raster_ms=172.67279052734375 tile_raster_ran=true world_repr_ms=0.249099999666214 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.24310000240802765 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:52.919461Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=54 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:52.919631Z  INFO perf: PERF wall=1016.44 instr=173.17 gap=843.28 | cpu_pre_egui=996.09 cpu_egui=10.39 cpu_post_egui=9.97 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=172.67 | upd_attrib sum=660.31 pv_cpu=0.00 pv_gpu=0.01 fire=3.70 stream=656.59 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.83 hud=0.00 overlay=0.00 raster_b=172.67 particles=0.00 residency=3.11 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.39 | stall first+preupd=0.10 update=0.00 post_dom=177.49 post_vt=0.11 post→ready=0.00 ready=1.65 post→egui=0.00 egui=10.26 post_egui=6.30 | stall_hits=[after_tile_storage_apply:818.5,after_domain_merge:177.5,post_egui:10.3,last:6.3]

2026-05-26T20:52:52.919772Z  INFO perf: PERF frame=1016.4ms update=996.1ms egui=10.4ms preview=0.0ms streaming=656.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=172.7ms

2026-05-26T20:52:52.919879Z  INFO stall: STALL culprit=after_tile_storage_apply duration=818.5ms frame=1016.4ms

2026-05-26T20:52:52.925697Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:52.926617Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:52.927208Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:52.927308Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:52.927400Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:52.927488Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:53.051608Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:53.084750Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:53.745695Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=56 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:53.745943Z  INFO stall: STALL after_tile_storage_apply: 821.45ms

2026-05-26T20:52:53.746923Z  INFO stall: STALL upd_streaming_reconstruct: 661.77ms

2026-05-26T20:52:53.923927Z  INFO stall: STALL after_domain_merge: 177.99ms

2026-05-26T20:52:53.923948Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:53.924576Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:53.924724Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:53.924841Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:53.925000Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:53.925178Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:53.934046Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:53.934680Z  INFO stall: STALL post_egui: 10.60ms

2026-05-26T20:52:53.936553Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:53.936659Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:53.942834Z  INFO stall: STALL last: 6.07ms

2026-05-26T20:52:53.942861Z  INFO visual_diag: VISUAL_DIAG window frame=55 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:53.943153Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=55 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:53.943408Z  INFO visual_diag: VISUAL_DIAG camera frame=55 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=18 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:53.943635Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=55 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:53.943859Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=55 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:53.943980Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=55 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:53.944165Z  INFO visual_diag: VISUAL_DIAG render_spine frame=55 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=56 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:53.944513Z  INFO visual_diag: VISUAL_DIAG perf frame=55 tile_raster_ms=172.9219970703125 tile_raster_ran=true world_repr_ms=0.2021999955177307 projection_graph_ms=0.001600000075995922 domain_merge_ms=0.00010000000474974513 readiness_ms=0.22430001199245453 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:53.944712Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=55 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:53.944864Z  INFO perf: PERF wall=1020.42 instr=173.35 gap=847.07 | cpu_pre_egui=999.49 cpu_egui=10.77 cpu_post_egui=10.16 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=172.92 | upd_attrib sum=665.20 pv_cpu=0.00 pv_gpu=0.02 fire=3.40 stream=661.77 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.84 hud=0.00 overlay=0.00 raster_b=172.92 particles=0.00 residency=2.86 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.77 | stall first+preupd=0.07 update=0.00 post_dom=177.99 post_vt=0.13 post→ready=0.01 ready=2.08 post→egui=0.01 egui=10.60 post_egui=6.07 | stall_hits=[after_tile_storage_apply:821.4,after_domain_merge:178.0,post_egui:10.6,last:6.1]

2026-05-26T20:52:53.944999Z  INFO perf: PERF frame=1020.4ms update=999.5ms egui=10.8ms preview=0.0ms streaming=661.8ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.9ms

2026-05-26T20:52:53.945099Z  INFO stall: STALL culprit=after_tile_storage_apply duration=821.4ms frame=1020.4ms

2026-05-26T20:52:53.949199Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:53.950159Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:53.950654Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:53.953596Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:53.953676Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:53.953757Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:54.076461Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:54.109832Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:54.756891Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=57 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:54.757127Z  INFO stall: STALL after_tile_storage_apply: 809.13ms

2026-05-26T20:52:54.758061Z  INFO stall: STALL upd_streaming_reconstruct: 647.72ms

2026-05-26T20:52:54.933958Z  INFO stall: STALL after_domain_merge: 176.83ms

2026-05-26T20:52:54.933987Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:54.934574Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:54.934718Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:54.934871Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:54.934996Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:54.935174Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:54.935339Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:54.944008Z  INFO stall: STALL post_egui: 9.91ms

2026-05-26T20:52:54.945402Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:54.945505Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:54.951899Z  INFO stall: STALL last: 6.29ms

2026-05-26T20:52:54.951924Z  INFO visual_diag: VISUAL_DIAG window frame=56 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:54.952185Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=56 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:54.952405Z  INFO visual_diag: VISUAL_DIAG camera frame=56 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=19 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:54.952603Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=56 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:54.952795Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=56 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:54.952902Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=56 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:54.953058Z  INFO visual_diag: VISUAL_DIAG render_spine frame=56 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=57 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:54.953354Z  INFO visual_diag: VISUAL_DIAG perf frame=56 tile_raster_ms=171.90269470214844 tile_raster_ran=true world_repr_ms=0.2667999863624573 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2190999984741211 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:54.953524Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=56 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:54.953666Z  INFO perf: PERF wall=1005.71 instr=172.39 gap=833.32 | cpu_pre_egui=986.01 cpu_egui=10.08 cpu_post_egui=9.62 gpu_gap=0.00 | spine=0.00 world_repr=0.27 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=171.90 | upd_attrib sum=651.42 pv_cpu=0.00 pv_gpu=0.01 fire=3.68 stream=647.72 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.97 hud=0.00 overlay=0.00 raster_b=171.90 particles=0.00 residency=3.02 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.08 | stall first+preupd=0.06 update=0.00 post_dom=176.83 post_vt=0.13 post→ready=0.00 ready=1.60 post→egui=0.00 egui=9.91 post_egui=6.29 | stall_hits=[after_tile_storage_apply:809.1,after_domain_merge:176.8,post_egui:9.9,last:6.3]

2026-05-26T20:52:54.953787Z  INFO perf: PERF frame=1005.7ms update=986.0ms egui=10.1ms preview=0.0ms streaming=647.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=171.9ms

2026-05-26T20:52:54.953877Z  INFO stall: STALL culprit=after_tile_storage_apply duration=809.1ms frame=1005.7ms

2026-05-26T20:52:54.957757Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:54.958471Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:54.959032Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:54.959140Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:54.959231Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:54.959313Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:55.087415Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:55.121674Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:55.781423Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=58 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:55.781662Z  INFO stall: STALL after_tile_storage_apply: 824.94ms

2026-05-26T20:52:55.782508Z  INFO stall: STALL upd_streaming_reconstruct: 660.42ms

2026-05-26T20:52:55.959192Z  INFO stall: STALL after_domain_merge: 177.53ms

2026-05-26T20:52:55.959227Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:55.959739Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:55.959865Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:55.959999Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:55.960138Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:55.960311Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:55.960472Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:55.970014Z  INFO stall: STALL post_egui: 10.71ms

2026-05-26T20:52:55.971356Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:55.972082Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:55.978589Z  INFO stall: STALL last: 6.40ms

2026-05-26T20:52:55.978613Z  INFO visual_diag: VISUAL_DIAG window frame=57 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:55.978835Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=57 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:55.979051Z  INFO visual_diag: VISUAL_DIAG camera frame=57 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=20 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:55.979262Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=57 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:55.979493Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=57 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:55.979617Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=57 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:55.979803Z  INFO visual_diag: VISUAL_DIAG render_spine frame=57 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=58 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:55.980157Z  INFO visual_diag: VISUAL_DIAG perf frame=57 tile_raster_ms=172.64599609375 tile_raster_ran=true world_repr_ms=0.20600000023841858 projection_graph_ms=0.0020000000949949026 domain_merge_ms=0.00010000000474974513 readiness_ms=0.8503000140190125 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:55.980334Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=57 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:55.980474Z  INFO perf: PERF wall=1023.82 instr=173.71 gap=850.11 | cpu_pre_egui=1002.55 cpu_egui=10.84 cpu_post_egui=10.43 gpu_gap=0.00 | spine=0.01 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.85 raster=172.65 | upd_attrib sum=663.67 pv_cpu=0.00 pv_gpu=0.02 fire=3.22 stream=660.42 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=175.37 hud=0.00 overlay=0.00 raster_b=172.65 particles=0.00 residency=2.67 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.84 | stall first+preupd=0.08 update=0.00 post_dom=177.53 post_vt=0.11 post→ready=0.00 ready=2.18 post→egui=0.00 egui=10.71 post_egui=6.40 | stall_hits=[after_tile_storage_apply:824.9,after_domain_merge:177.5,post_egui:10.7,last:6.4]

2026-05-26T20:52:55.980592Z  INFO perf: PERF frame=1023.8ms update=1002.5ms egui=10.8ms preview=0.0ms streaming=660.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.6ms

2026-05-26T20:52:55.980683Z  INFO stall: STALL culprit=after_tile_storage_apply duration=824.9ms frame=1023.8ms

2026-05-26T20:52:55.985975Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:55.986847Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:55.987325Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:55.989833Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:55.989919Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:55.990004Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:56.112220Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:56.146747Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:56.819718Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=59 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:56.819942Z  INFO stall: STALL after_tile_storage_apply: 834.91ms

2026-05-26T20:52:56.820978Z  INFO stall: STALL upd_streaming_reconstruct: 673.85ms

2026-05-26T20:52:56.996295Z  INFO stall: STALL after_domain_merge: 176.35ms

2026-05-26T20:52:56.996325Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:56.996917Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:56.997047Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:56.997182Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:56.997319Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:56.997479Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:56.997638Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:57.006891Z  INFO stall: STALL post_egui: 10.48ms

2026-05-26T20:52:57.008262Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:57.008365Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:57.014732Z  INFO stall: STALL last: 6.26ms

2026-05-26T20:52:57.014759Z  INFO visual_diag: VISUAL_DIAG window frame=58 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:57.015082Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=58 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:57.016787Z  INFO visual_diag: VISUAL_DIAG camera frame=58 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=21 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:57.016987Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=58 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:57.017181Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=58 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:57.017288Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=58 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:57.017446Z  INFO visual_diag: VISUAL_DIAG render_spine frame=58 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=59 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:57.017738Z  INFO visual_diag: VISUAL_DIAG perf frame=58 tile_raster_ms=171.39710998535156 tile_raster_ran=true world_repr_ms=0.19740000367164612 projection_graph_ms=0.00139999995008111 domain_merge_ms=0.00020000000949949026 readiness_ms=0.21950000524520874 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:57.017907Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=58 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:57.018048Z  INFO perf: PERF wall=1033.06 instr=171.82 gap=861.24 | cpu_pre_egui=1011.32 cpu_egui=10.62 cpu_post_egui=11.12 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=171.40 | upd_attrib sum=677.54 pv_cpu=0.00 pv_gpu=0.02 fire=3.67 stream=673.85 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.59 hud=0.00 overlay=0.00 raster_b=171.40 particles=0.00 residency=3.14 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.62 | stall first+preupd=0.07 update=0.00 post_dom=176.35 post_vt=0.12 post→ready=0.00 ready=1.58 post→egui=0.00 egui=10.48 post_egui=6.26 | stall_hits=[after_tile_storage_apply:834.9,after_domain_merge:176.4,post_egui:10.5,last:6.3]

2026-05-26T20:52:57.018166Z  INFO perf: PERF frame=1033.1ms update=1011.3ms egui=10.6ms preview=0.0ms streaming=673.8ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=171.4ms

2026-05-26T20:52:57.018256Z  INFO stall: STALL culprit=after_tile_storage_apply duration=834.9ms frame=1033.1ms

2026-05-26T20:52:57.025436Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:57.026240Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:57.026548Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:57.026646Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:57.026736Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:57.026826Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:57.151607Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:57.186106Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:57.835541Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=60 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:57.835743Z  INFO stall: STALL after_tile_storage_apply: 811.69ms

2026-05-26T20:52:57.836606Z  INFO stall: STALL upd_streaming_reconstruct: 650.02ms

2026-05-26T20:52:58.013559Z  INFO stall: STALL after_domain_merge: 177.82ms

2026-05-26T20:52:58.013595Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:58.014090Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:58.014222Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:58.014366Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:58.014503Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:58.014672Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:58.014831Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:58.024186Z  INFO stall: STALL post_egui: 10.50ms

2026-05-26T20:52:58.025779Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:58.025881Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:58.032653Z  INFO stall: STALL last: 6.67ms

2026-05-26T20:52:58.032668Z  INFO stage5_readiness::live: READINESS_FRAME_FENCE_OK eval_inv=60 frame_tick=60 passes=true

2026-05-26T20:52:58.032701Z  INFO visual_diag: VISUAL_DIAG window frame=59 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:58.033080Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=59 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:58.033380Z  INFO visual_diag: VISUAL_DIAG camera frame=59 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=22 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:58.033610Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=59 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:58.033836Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=59 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:58.033958Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=59 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:58.034231Z  INFO visual_diag: VISUAL_DIAG render_spine frame=59 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=60 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:58.034543Z  INFO visual_diag: VISUAL_DIAG perf frame=59 tile_raster_ms=172.96031188964844 tile_raster_ran=true world_repr_ms=0.2441999912261963 projection_graph_ms=0.0017999999690800905 domain_merge_ms=0.00020000000949949026 readiness_ms=0.21770000457763672 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:58.034715Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=59 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:58.034861Z  INFO perf: PERF wall=1010.87 instr=173.43 gap=837.45 | cpu_pre_egui=989.60 cpu_egui=10.63 cpu_post_egui=10.64 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=172.96 | upd_attrib sum=653.68 pv_cpu=0.00 pv_gpu=0.01 fire=3.64 stream=650.02 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.06 hud=0.00 overlay=0.00 raster_b=172.96 particles=0.00 residency=3.04 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.63 | stall first+preupd=0.09 update=0.00 post_dom=177.82 post_vt=0.12 post→ready=0.00 ready=1.80 post→egui=0.00 egui=10.50 post_egui=6.67 | stall_hits=[after_tile_storage_apply:811.7,after_domain_merge:177.8,post_egui:10.5,last:6.7]

2026-05-26T20:52:58.035037Z  INFO perf: PERF frame=1010.9ms update=989.6ms egui=10.6ms preview=0.0ms streaming=650.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=173.0ms

2026-05-26T20:52:58.035127Z  INFO stall: STALL culprit=after_tile_storage_apply duration=811.7ms frame=1010.9ms

2026-05-26T20:52:58.042531Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:58.043597Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:58.044006Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:58.046603Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:58.046698Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:58.046792Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:58.166597Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:58.201026Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:58.867420Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=61 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:58.867637Z  INFO stall: STALL after_tile_storage_apply: 826.06ms

2026-05-26T20:52:58.869102Z  INFO stall: STALL upd_streaming_reconstruct: 667.60ms

2026-05-26T20:52:59.046523Z  INFO stall: STALL after_domain_merge: 178.89ms

2026-05-26T20:52:59.046553Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:59.047091Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:52:59.047221Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:59.047360Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:59.047493Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:59.047655Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:59.047826Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:52:59.056552Z  INFO stall: STALL post_egui: 9.92ms

2026-05-26T20:52:59.057514Z  INFO ui_layout_tree: UI_LAYOUT_TREE frame=60 root=275v1 target=102710v0

2026-05-26T20:52:59.057635Z  INFO ui_layout_tree: hud_root (hud_root) entity=275v1  size=(1920.0,1017.0) width=100.0% height=100.0% min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.057750Z  INFO ui_layout_tree:   unnamed () entity=274v1  size=(1920.0,38.0) width=100.0% height=38.0px min=(Auto,38.0px) max=(Auto,38.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.057863Z  INFO ui_layout_tree:     unnamed () entity=273v1  size=(186.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.057975Z  INFO ui_layout_tree:       unnamed () entity=272v1  size=(172.0,16.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058102Z  INFO ui_layout_tree:     unnamed () entity=271v1  size=(611.0,28.0) width=Auto height=Auto min=(100.0px,Auto) max=(Auto,Auto) flex_grow=1.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058214Z  INFO ui_layout_tree:       unnamed () entity=270v1  size=(22.0,22.0) width=22.0px height=22.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058351Z  INFO ui_layout_tree:         unnamed () entity=269v1  size=(11.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058462Z  INFO ui_layout_tree:       unnamed () entity=268v1  size=(71.0,16.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058569Z  INFO ui_layout_tree:     unnamed () entity=267v1  size=(206.0,22.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058677Z  INFO ui_layout_tree:       unnamed () entity=266v1  size=(196.0,16.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058784Z  INFO ui_layout_tree:     unnamed () entity=265v1  size=(213.0,22.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058892Z  INFO ui_layout_tree:       unnamed () entity=102700v0  size=(203.0,16.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.058999Z  INFO ui_layout_tree:     unnamed () entity=102701v0  size=(556.0,22.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.059111Z  INFO ui_layout_tree:       unnamed () entity=102702v0  size=(546.0,16.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.059225Z  INFO ui_layout_tree:     unnamed () entity=102703v0  size=(54.0,20.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.059339Z  INFO ui_layout_tree:       unnamed () entity=102704v0  size=(40.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.059435Z  INFO ui_layout_tree:   unnamed () entity=102705v0  size=(1920.0,26.0) width=100.0% height=26.0px min=(Auto,26.0px) max=(Auto,26.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.059529Z  INFO ui_layout_tree:     unnamed () entity=102706v0  size=(898.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.059626Z  INFO ui_layout_tree:   unnamed () entity=102707v0  size=(1920.0,22.0) width=100.0% height=22.0px min=(Auto,22.0px) max=(Auto,22.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.059719Z  INFO ui_layout_tree:     unnamed () entity=102708v0  size=(580.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.059814Z  INFO ui_layout_tree:   center_row (center_row) entity=102709v0  size=(1920.0,1017.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.059911Z  INFO ui_layout_tree:     sim_map_fill (sim_map_fill) entity=102710v0 <<< SIM_VIEWPORT size=(1920.0,1017.0) width=100.0% height=100.0% min=(400.0px,300.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.070531Z  INFO ui_layout_tree:       map_viewport_frame_inset () entity=102711v0  size=(1912.0,1009.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.070646Z  INFO ui_layout_tree:   left_stack_overlay (left_stack_overlay) entity=102712v0  size=(106.0,921.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Stretch align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.070753Z  INFO ui_layout_tree:     unnamed () entity=102713v0  size=(48.0,921.0) width=48.0px height=Auto min=(Auto,120.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=FlexStart overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.070855Z  INFO ui_layout_tree:       unnamed () entity=102714v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.070956Z  INFO ui_layout_tree:       unnamed () entity=102715v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071057Z  INFO ui_layout_tree:       unnamed () entity=102716v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071170Z  INFO ui_layout_tree:       unnamed () entity=102717v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071277Z  INFO ui_layout_tree:     build_rail () entity=102718v0  size=(52.0,921.0) width=52.0px height=100.0% min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071380Z  INFO ui_layout_tree:       unnamed () entity=102719v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071483Z  INFO ui_layout_tree:         unnamed () entity=102720v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071583Z  INFO ui_layout_tree:         unnamed () entity=102721v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071686Z  INFO ui_layout_tree:       unnamed () entity=102722v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071789Z  INFO ui_layout_tree:         unnamed () entity=102723v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071890Z  INFO ui_layout_tree:         unnamed () entity=102724v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.071992Z  INFO ui_layout_tree:       unnamed () entity=102725v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072098Z  INFO ui_layout_tree:         unnamed () entity=102726v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072204Z  INFO ui_layout_tree:         unnamed () entity=102727v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072310Z  INFO ui_layout_tree:       unnamed () entity=102728v0  size=(44.0,32.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072412Z  INFO ui_layout_tree:         unnamed () entity=102729v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072514Z  INFO ui_layout_tree:       unnamed () entity=102730v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072616Z  INFO ui_layout_tree:         unnamed () entity=102731v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072716Z  INFO ui_layout_tree:         unnamed () entity=102732v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072818Z  INFO ui_layout_tree:       unnamed () entity=102733v0  size=(44.0,32.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.072919Z  INFO ui_layout_tree:         unnamed () entity=102734v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073022Z  INFO ui_layout_tree:       unnamed () entity=102735v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073125Z  INFO ui_layout_tree:         unnamed () entity=102736v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073229Z  INFO ui_layout_tree:         unnamed () entity=102737v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073333Z  INFO ui_layout_tree:     unnamed () entity=102738v0  size=(0.0,0.0) width=400.0px height=100.0% min=(Auto,Auto) max=(Auto,100.0%) flex_grow=0.00 flex_shrink=0.00 flex_dir=Column align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }

2026-05-26T20:52:59.073434Z  INFO ui_layout_tree:       unnamed () entity=102739v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=FlexEnd justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073535Z  INFO ui_layout_tree:         unnamed () entity=102740v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073636Z  INFO ui_layout_tree:       unnamed () entity=102741v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073737Z  INFO ui_layout_tree:       unnamed () entity=102742v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073838Z  INFO ui_layout_tree:       unnamed () entity=102743v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.073939Z  INFO ui_layout_tree:       unnamed () entity=102744v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.074039Z  INFO ui_layout_tree:       unnamed () entity=102745v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.074140Z  INFO ui_layout_tree:       unnamed () entity=102746v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.074244Z  INFO ui_layout_tree:       unnamed () entity=102747v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.074346Z  INFO ui_layout_tree:       unnamed () entity=102748v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.074446Z  INFO ui_layout_tree:       unnamed () entity=102749v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.074549Z  INFO ui_layout_tree:   minimap_chrome_root () entity=102750v0  size=(262.0,222.0) width=262.0px height=222.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075065Z  INFO ui_layout_tree:     unnamed () entity=102751v0  size=(260.0,220.0) width=100.0% height=100.0% min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075185Z  INFO ui_layout_tree:   context_tray_root () entity=102752v0  size=(1814.0,32.0) width=Auto height=32.0px min=(Auto,32.0px) max=(Auto,32.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075290Z  INFO ui_layout_tree:     unnamed () entity=102753v0  size=(1814.0,32.0) width=100.0% height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075395Z  INFO ui_layout_tree:       unnamed () entity=102754v0  size=(59.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075495Z  INFO ui_layout_tree:         unnamed () entity=102755v0  size=(40.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075596Z  INFO ui_layout_tree:       unnamed () entity=102756v0  size=(51.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075700Z  INFO ui_layout_tree:         unnamed () entity=102757v0  size=(33.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075803Z  INFO ui_layout_tree:       unnamed () entity=102758v0  size=(78.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.075906Z  INFO ui_layout_tree:         unnamed () entity=102759v0  size=(60.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076008Z  INFO ui_layout_tree:       unnamed () entity=102760v0  size=(45.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076114Z  INFO ui_layout_tree:         unnamed () entity=102761v0  size=(27.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076233Z  INFO ui_layout_tree:     unnamed () entity=102762v0  size=(1814.0,71.0) width=100.0% height=96.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076336Z  INFO ui_layout_tree:       petroleum_panel_tab () entity=102763v0  size=(855.0,45.0) width=100.0% height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076438Z  INFO ui_layout_tree:         unnamed () entity=102764v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076537Z  INFO ui_layout_tree:         unnamed () entity=102765v0  size=(60.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076639Z  INFO ui_layout_tree:       logistics_vehicle_chips () entity=102766v0  size=(847.0,45.0) width=100.0% height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=FlexStart align_self=Auto justify=FlexStart overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076741Z  INFO ui_layout_tree:         unnamed () entity=102767v0  size=(37.0,45.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076842Z  INFO ui_layout_tree:           unnamed () entity=102768v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.076942Z  INFO ui_layout_tree:           unnamed () entity=102769v0  size=(27.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077042Z  INFO ui_layout_tree:         unnamed () entity=102770v0  size=(34.0,45.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077154Z  INFO ui_layout_tree:           unnamed () entity=102771v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077257Z  INFO ui_layout_tree:           unnamed () entity=102772v0  size=(22.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077362Z  INFO ui_layout_tree:         unnamed () entity=102773v0  size=(34.0,45.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077464Z  INFO ui_layout_tree:           unnamed () entity=102774v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077565Z  INFO ui_layout_tree:           unnamed () entity=102775v0  size=(17.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077674Z  INFO ui_layout_tree:       unnamed () entity=102776v0  size=(92.0,51.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }

2026-05-26T20:52:59.077816Z  INFO ui_layout_tree::chain: MAP_LAYOUT_CHAIN:

  Window: 1920x1017

  RootHud: 1920x1017

  center_row: 1920x1017

  sim_map_fill: 1920x1017

  MapFill: 1920x1017

  Measured: 1920x1017

  Committed: 1920x1017

  Solver(SimMapFill): 1920x1017

  CommittedResource: 1920x1017 last_commit=hole_hold frame=60

2026-05-26T20:52:59.077973Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:52:59.078066Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:52:59.078165Z  INFO stall: STALL after_readiness: 21.61ms

2026-05-26T20:52:59.084552Z  INFO stall: STALL last: 6.39ms

2026-05-26T20:52:59.084576Z  INFO visual_diag: VISUAL_DIAG window frame=60 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:52:59.085833Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=60 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:52:59.086257Z  INFO visual_diag: VISUAL_DIAG camera frame=60 cam_desired_x=160.0 cam_desired_y=245.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=23 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:52:59.086468Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=60 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:52:59.086697Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=60 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:52:59.086810Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=60 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:52:59.087042Z  INFO visual_diag: VISUAL_DIAG render_spine frame=60 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=61 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:52:59.087355Z  INFO visual_diag: VISUAL_DIAG perf frame=60 tile_raster_ms=174.00338745117188 tile_raster_ran=true world_repr_ms=0.26100000739097595 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00010000000474974513 readiness_ms=0.20880000293254852 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:52:59.087531Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=60 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:52:59.087657Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 249.9 ms)

2026-05-26T20:52:59.087769Z  INFO perf: PERF wall=1046.18 instr=174.48 gap=871.70 | cpu_pre_egui=1005.04 cpu_egui=10.04 cpu_post_egui=31.10 gpu_gap=0.00 | spine=0.00 world_repr=0.26 graph=0.00 merge=0.00 atm=0.00 readiness=0.21 raster=174.00 | upd_attrib sum=671.07 pv_cpu=0.00 pv_gpu=0.01 fire=3.45 stream=667.60 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.89 hud=0.00 overlay=0.00 raster_b=174.00 particles=0.00 residency=2.83 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.04 | stall first+preupd=0.09 update=0.00 post_dom=178.89 post_vt=0.11 post→ready=0.00 ready=21.61 post→egui=0.00 egui=9.92 post_egui=6.39 | stall_hits=[after_tile_storage_apply:826.1,after_domain_merge:178.9,post_egui:9.9,after_readiness:21.6,last:6.4]

2026-05-26T20:52:59.087891Z  INFO perf: PERF frame=1046.2ms update=1005.0ms egui=10.0ms preview=0.0ms streaming=667.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=174.0ms

2026-05-26T20:52:59.087980Z  INFO stall: STALL culprit=after_tile_storage_apply duration=826.1ms frame=1046.2ms

2026-05-26T20:52:59.094816Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:52:59.095678Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:52:59.096161Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:52:59.096258Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:52:59.096352Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:52:59.096441Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:52:59.220736Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,245.00) zoom=108.1352 world_main_xy=(160.00,245.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:52:59.254459Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:52:59.912506Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=62 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:52:59.912713Z  INFO stall: STALL after_tile_storage_apply: 819.11ms

2026-05-26T20:52:59.914094Z  INFO stall: STALL upd_streaming_reconstruct: 659.17ms

2026-05-26T20:53:00.091035Z  INFO stall: STALL after_domain_merge: 178.31ms

2026-05-26T20:53:00.091113Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:00.091856Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:53:00.091988Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:00.092128Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:00.092270Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:00.092456Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:00.092635Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:53:00.101708Z  INFO stall: STALL post_egui: 10.47ms

2026-05-26T20:53:00.103313Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:53:00.103423Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:53:00.110068Z  INFO stall: STALL last: 6.55ms

2026-05-26T20:53:00.110096Z  INFO visual_diag: VISUAL_DIAG window frame=61 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:53:00.111165Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=61 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:53:00.111492Z  INFO visual_diag: VISUAL_DIAG camera frame=61 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=24 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:53:00.111780Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=61 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:53:00.112062Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=61 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:53:00.112212Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=61 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:53:00.112451Z  INFO visual_diag: VISUAL_DIAG render_spine frame=61 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=62 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:53:00.112831Z  INFO visual_diag: VISUAL_DIAG perf frame=61 tile_raster_ms=173.1031036376953 tile_raster_ran=true world_repr_ms=0.2452000081539154 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00020000000949949026 readiness_ms=0.22830000519752502 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:53:00.113006Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=61 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:53:00.113165Z  INFO perf: PERF wall=1019.62 instr=173.58 gap=846.04 | cpu_pre_egui=997.49 cpu_egui=10.71 cpu_post_egui=11.42 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=173.10 | upd_attrib sum=662.76 pv_cpu=0.00 pv_gpu=0.01 fire=3.57 stream=659.17 map_fit=0.01 hud=0.00 wgen=0.00 | budget_sum=176.12 hud=0.00 overlay=0.00 raster_b=173.10 particles=0.00 residency=2.96 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.71 | stall first+preupd=0.09 update=0.00 post_dom=178.31 post_vt=0.21 post→ready=0.00 ready=1.81 post→egui=0.01 egui=10.47 post_egui=6.55 | stall_hits=[after_tile_storage_apply:819.1,after_domain_merge:178.3,post_egui:10.5,last:6.5]

2026-05-26T20:53:00.113290Z  INFO perf: PERF frame=1019.6ms update=997.5ms egui=10.7ms preview=0.0ms streaming=659.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=173.1ms

2026-05-26T20:53:00.113384Z  INFO stall: STALL culprit=after_tile_storage_apply duration=819.1ms frame=1019.6ms

2026-05-26T20:53:00.119918Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:00.120842Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:53:00.121257Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:53:00.121359Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:53:00.121466Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:53:00.121559Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:53:00.248357Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:53:00.282443Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:53:00.963708Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=63 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:53:00.963962Z  INFO stall: STALL after_tile_storage_apply: 845.13ms

2026-05-26T20:53:00.965466Z  INFO stall: STALL upd_streaming_reconstruct: 682.53ms

2026-05-26T20:53:01.142579Z  INFO stall: STALL after_domain_merge: 178.62ms

2026-05-26T20:53:01.142602Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:01.143149Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:53:01.143291Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:01.143440Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:01.143574Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:01.143736Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:01.143895Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:53:01.153209Z  INFO stall: STALL post_egui: 10.50ms

2026-05-26T20:53:01.158793Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:53:01.158904Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:53:01.159012Z  INFO stall: STALL after_readiness: 5.80ms

2026-05-26T20:53:01.165381Z  INFO stall: STALL last: 6.37ms

2026-05-26T20:53:01.165406Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=62 was=4 now=2

2026-05-26T20:53:01.165417Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=62 win_logical=(1920, 1017) win_physical=(1920, 1017) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=1 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=25 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=30 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=false map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:53:01.165615Z  INFO visual_diag: VISUAL_DIAG window frame=62 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:53:01.166386Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=62 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=false settle_streak=1 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:53:01.166645Z  INFO visual_diag: VISUAL_DIAG camera frame=62 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=25 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:53:01.166877Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=62 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:53:01.167104Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=62 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:53:01.167226Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=62 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:53:01.167412Z  INFO visual_diag: VISUAL_DIAG render_spine frame=62 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=63 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:53:01.167705Z  INFO visual_diag: VISUAL_DIAG perf frame=62 tile_raster_ms=173.70150756835938 tile_raster_ran=true world_repr_ms=0.24950000643730164 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00020000000949949026 readiness_ms=0.24210000038146973 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:53:01.167904Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=62 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:53:01.168049Z  INFO perf: PERF wall=1049.27 instr=174.20 gap=875.07 | cpu_pre_egui=1023.82 cpu_egui=10.65 cpu_post_egui=14.80 gpu_gap=0.00 | spine=0.01 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=173.70 | upd_attrib sum=686.21 pv_cpu=0.00 pv_gpu=0.03 fire=3.64 stream=682.53 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=176.78 hud=0.00 overlay=0.00 raster_b=173.70 particles=0.00 residency=3.03 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.65 | stall first+preupd=0.08 update=0.00 post_dom=178.62 post_vt=0.12 post→ready=0.00 ready=5.80 post→egui=0.00 egui=10.50 post_egui=6.37 | stall_hits=[after_tile_storage_apply:845.1,after_domain_merge:178.6,post_egui:10.5,after_readiness:5.8,last:6.4]

2026-05-26T20:53:01.168169Z  INFO perf: PERF frame=1049.3ms update=1023.8ms egui=10.7ms preview=0.0ms streaming=682.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=173.7ms

2026-05-26T20:53:01.168259Z  INFO stall: STALL culprit=after_tile_storage_apply duration=845.1ms frame=1049.3ms

2026-05-26T20:53:01.175423Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:01.176426Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:53:01.176831Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:53:01.176926Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:53:01.177013Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:53:01.177100Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:53:01.301171Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:53:01.336028Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:53:01.974141Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=64 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:53:01.974376Z  INFO stall: STALL after_tile_storage_apply: 799.89ms

2026-05-26T20:53:01.975564Z  INFO stall: STALL upd_streaming_reconstruct: 639.11ms

2026-05-26T20:53:02.151394Z  INFO stall: STALL after_domain_merge: 177.02ms

2026-05-26T20:53:02.151406Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:02.151977Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:53:02.152124Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:02.152284Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:02.152438Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:02.152612Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:02.161127Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:53:02.161702Z  INFO stall: STALL post_egui: 10.18ms

2026-05-26T20:53:02.163029Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:53:02.163136Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:53:02.169521Z  INFO stall: STALL last: 6.28ms

2026-05-26T20:53:02.169552Z  INFO visual_diag: VISUAL_DIAG window frame=63 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:53:02.169845Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=63 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=false settle_streak=2 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:53:02.170144Z  INFO visual_diag: VISUAL_DIAG camera frame=63 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=26 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:53:02.170410Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=63 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:53:02.170637Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=63 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:53:02.170758Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=63 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:53:02.170938Z  INFO visual_diag: VISUAL_DIAG render_spine frame=63 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=64 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:53:02.171283Z  INFO visual_diag: VISUAL_DIAG perf frame=63 tile_raster_ms=172.02040100097656 tile_raster_ran=true world_repr_ms=0.20189999043941498 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00020000000949949026 readiness_ms=0.22459998726844788 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:53:02.171479Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=63 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:53:02.171618Z  INFO perf: PERF wall=997.21 instr=172.45 gap=824.76 | cpu_pre_egui=976.98 cpu_egui=10.34 cpu_post_egui=9.89 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=172.02 | upd_attrib sum=642.57 pv_cpu=0.00 pv_gpu=0.02 fire=3.44 stream=639.11 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.95 hud=0.00 overlay=0.00 raster_b=172.02 particles=0.00 residency=2.88 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.34 | stall first+preupd=0.09 update=0.00 post_dom=177.02 post_vt=0.12 post→ready=0.00 ready=1.54 post→egui=0.00 egui=10.18 post_egui=6.28 | stall_hits=[after_tile_storage_apply:799.9,after_domain_merge:177.0,post_egui:10.2,last:6.3]

2026-05-26T20:53:02.171735Z  INFO perf: PERF frame=997.2ms update=977.0ms egui=10.3ms preview=0.0ms streaming=639.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.0ms

2026-05-26T20:53:02.171824Z  INFO stall: STALL culprit=after_tile_storage_apply duration=799.9ms frame=997.2ms

2026-05-26T20:53:02.521568Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:02.522311Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:53:02.522738Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:53:02.524044Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:53:02.524125Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:53:02.524220Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:53:02.650405Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:53:02.684636Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:53:03.334441Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=65 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:53:03.334703Z  INFO stall: STALL after_tile_storage_apply: 813.95ms

2026-05-26T20:53:03.335350Z  INFO stall: STALL upd_streaming_reconstruct: 650.16ms

2026-05-26T20:53:03.514271Z  INFO stall: STALL after_domain_merge: 179.57ms

2026-05-26T20:53:03.514315Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:03.515239Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1920.0 h=1017.0

2026-05-26T20:53:03.515365Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:03.515510Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:03.515651Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:03.515824Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:03.515988Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:53:03.524985Z  INFO stall: STALL post_egui: 10.58ms

2026-05-26T20:53:03.526530Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:53:03.526636Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:53:03.532909Z  INFO stall: STALL last: 6.17ms

2026-05-26T20:53:03.532945Z  INFO visual_diag: VISUAL_DIAG window frame=64 periodic=false win_logical=(1920, 1017) win_physical=(1920, 1017) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:53:03.533954Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=64 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=false settle_streak=3 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:53:03.534212Z  INFO visual_diag: VISUAL_DIAG camera frame=64 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=27 cam_scissor=Some((0, 0, 1920, 1017)) ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:53:03.534456Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=64 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:53:03.534685Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=64 world_preview_proj_rev=2551213575047 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742045

2026-05-26T20:53:03.534806Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=64 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:53:03.534986Z  INFO visual_diag: VISUAL_DIAG render_spine frame=64 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=65 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:53:03.535333Z  INFO visual_diag: VISUAL_DIAG perf frame=64 tile_raster_ms=174.6632080078125 tile_raster_ran=true world_repr_ms=0.29019999504089355 projection_graph_ms=0.0020000000949949026 domain_merge_ms=0.00010000000474974513 readiness_ms=0.22619999945163727 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:53:03.535533Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=64 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:53:03.535682Z  INFO perf: PERF wall=1015.01 instr=175.19 gap=839.83 | cpu_pre_egui=993.61 cpu_egui=10.74 cpu_post_egui=10.66 gpu_gap=0.00 | spine=0.01 world_repr=0.29 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=174.66 | upd_attrib sum=653.70 pv_cpu=0.00 pv_gpu=0.02 fire=3.51 stream=650.16 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=177.53 hud=0.00 overlay=0.00 raster_b=174.66 particles=0.00 residency=2.82 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=10.74 | stall first+preupd=0.10 update=0.00 post_dom=179.57 post_vt=0.13 post→ready=0.00 ready=1.75 post→egui=0.00 egui=10.58 post_egui=6.17 | stall_hits=[after_tile_storage_apply:813.9,after_domain_merge:179.6,post_egui:10.6,last:6.2]

2026-05-26T20:53:03.535802Z  INFO perf: PERF frame=1015.0ms update=993.6ms egui=10.7ms preview=0.0ms streaming=650.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=174.7ms

2026-05-26T20:53:03.535892Z  INFO stall: STALL culprit=after_tile_storage_apply duration=813.9ms frame=1015.0ms

2026-05-26T20:53:03.552486Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1920.0, 1017.0) w=1920.0 h=1017.0

2026-05-26T20:53:03.553292Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN

2026-05-26T20:53:03.553899Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8

2026-05-26T20:53:03.553990Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15

2026-05-26T20:53:03.554070Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27

2026-05-26T20:53:03.554146Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN

2026-05-26T20:53:03.678000Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=108.1352 world_main_xy=(160.00,160.00) zoom=108.1352 bridge_drift=0.0000

2026-05-26T20:53:03.712535Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)

2026-05-26T20:53:04.363559Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=66 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false

2026-05-26T20:53:04.363812Z  INFO stall: STALL after_tile_storage_apply: 812.53ms

2026-05-26T20:53:04.364355Z  INFO stall: STALL upd_streaming_reconstruct: 651.32ms

2026-05-26T20:53:04.540485Z  INFO stall: STALL after_domain_merge: 176.67ms

2026-05-26T20:53:04.540487Z  INFO bevy_window::system: No windows are open, exiting

2026-05-26T20:53:04.549158Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)

2026-05-26T20:53:04.549760Z  INFO stall: STALL post_egui: 9.14ms

2026-05-26T20:53:04.551249Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1

2026-05-26T20:53:04.551362Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0

2026-05-26T20:53:04.558040Z  INFO stall: STALL last: 6.57ms

2026-05-26T20:53:04.558062Z  WARN visual_diag::anomaly: CAMERA_SCISSOR_CHANGED frame=65 was=Some((0, 0, 1920, 1017)) now=None

2026-05-26T20:53:04.558064Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=65 win_logical=(1, 1) win_physical=(1, 1) sim_valid=false sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) measured_valid=true measured_wh=Vec2(1920.0, 1017.0) committed_wh=Vec2(1920.0, 1017.0) sim_wh=Vec2(1920.0, 1017.0) settle_streak=3 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1920.0, 1017.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=27 cam_scissor=None ortho_fixed_wh=(18, 9) map_view_px=(1920, 1017) raster_rev=30 resolved_rev=62 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=false map_cam_scale=Vec3(108.1352, 108.1352, 108.1352) minimap_visible=true

2026-05-26T20:53:04.558260Z  WARN visual_diag::anomaly: SIM_VIEWPORT_VALIDITY_CHANGED frame=65 was_valid=true now_valid=false

2026-05-26T20:53:04.558774Z  WARN sim_view_sync::anomaly: CAMERA_SCISSOR_CHANGED frame=65 was=Some((0, 0, 1920, 1017)) now=None

2026-05-26T20:53:04.558883Z  INFO visual_diag: VISUAL_DIAG window frame=65 periodic=false win_logical=(1, 1) win_physical=(1, 1) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }

2026-05-26T20:53:04.558987Z  WARN sim_view_sync::anomaly: SIM_MAP_VIEWPORT_VALIDITY_CHANGED frame=65 was_valid=true now_valid=false was_adequate=true now_adequate=true

2026-05-26T20:53:04.559182Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=65 sim_valid=false sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1920.0, 1017.0) sim_wh=(1920, 1017) measured_valid=true measured_wh=(1920, 1017) committed_wh=(1920, 1017) sim_held=false settle_streak=3 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1920, 1017) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1920.0, 1017.0)

2026-05-26T20:53:04.559563Z  INFO visual_diag: VISUAL_DIAG camera frame=65 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=108.13520050048828 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=27 cam_scissor=None ortho_fixed_w=18 ortho_fixed_h=9 map_view_px_w=1920 map_view_px_h=1017 world_w=320 world_h=320

2026-05-26T20:53:04.559758Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=65 resolved_rev=62 primary_valid=true primary_wh=(1920, 1017) sim_resolved_valid=true sim_resolved_wh=(1920, 1017) preview_valid=true preview_wh=(1212, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false

2026-05-26T20:53:04.559951Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=65 world_preview_proj_rev=2551210575038 minimap_proj_rev=944893805415 sim_map_proj_rev=4368011742042

2026-05-26T20:53:04.560057Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=65 wp_fit=Contain wp_viewport=UVec2(1212, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0

2026-05-26T20:53:04.560215Z  INFO visual_diag: VISUAL_DIAG render_spine frame=65 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=1.0 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=66 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0

2026-05-26T20:53:04.560505Z  INFO visual_diag: VISUAL_DIAG perf frame=65 tile_raster_ms=171.68711853027344 tile_raster_ran=true world_repr_ms=0.2493000030517578 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2441999912261963 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0

2026-05-26T20:53:04.560674Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=65 cmd_shell=false overlay_tray=false transmission=false

2026-05-26T20:53:04.560791Z  INFO bevy_winit::system: Closing window 0v0

2026-05-26T20:53:04.560817Z  INFO perf: PERF wall=1009.59 instr=172.18 gap=837.41 | cpu_pre_egui=989.27 cpu_egui=9.30 cpu_post_egui=11.03 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=171.69 | upd_attrib sum=655.00 pv_cpu=0.00 pv_gpu=0.02 fire=3.65 stream=651.32 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=174.76 hud=0.00 overlay=0.00 raster_b=171.69 particles=0.00 residency=3.02 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=9.30 | stall first+preupd=0.08 update=0.00 post_dom=176.67 post_vt=0.13 post→ready=0.00 ready=1.71 post→egui=0.00 egui=9.14 post_egui=6.57 | stall_hits=[after_tile_storage_apply:812.5,after_domain_merge:176.7,post_egui:9.1,last:6.6]

2026-05-26T20:53:04.560982Z  INFO perf: PERF frame=1009.6ms update=989.3ms egui=9.3ms preview=0.0ms streaming=651.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=171.7ms

2026-05-26T20:53:04.561073Z  INFO stall: STALL culprit=after_tile_storage_apply duration=812.5ms frame=1009.6ms

# read
2026-05-27T03:24:28.724025Z  INFO visual_diag: VISUAL_DIAG window frame=24 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:28.724024Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=24 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=24 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=43 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:28.724435Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=24 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:28.725046Z  INFO visual_diag: VISUAL_DIAG camera frame=24 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=24 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:28.725218Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=24 resolved_rev=43 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:28.725386Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=24 world_preview_proj_rev=2551212574558 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:28.725466Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=24 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:28.725598Z  INFO visual_diag: VISUAL_DIAG render_spine frame=24 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=25 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:28.725929Z  INFO visual_diag: VISUAL_DIAG perf frame=24 tile_raster_ms=157.9027099609375 tile_raster_ran=true world_repr_ms=0.22370000183582306 projection_graph_ms=0.0021000001579523087 domain_merge_ms=0.00010000000474974513 readiness_ms=86.06350708007813 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:28.726080Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=24 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:28.726209Z  INFO perf: PERF wall=1787.36 instr=244.20 gap=1543.17 | cpu_pre_egui=1175.97 cpu_egui=510.01 cpu_post_egui=101.38 gpu_gap=0.00 | spine=0.01 world_repr=0.22 graph=0.00 merge=0.00 atm=0.00 readiness=86.06 raster=157.90 | upd_attrib sum=849.42 pv_cpu=0.00 pv_gpu=0.02 fire=171.20 stream=678.19 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=157.90 hud=0.00 overlay=0.00 raster_b=157.90 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=510.01 | stall first+preupd=0.10 update=89.09 post_dom=259.68 post_vt=74.71 post→ready=123.12 ready=86.79 post→egui=223.06 egui=11.95 post_egui=0.47 | stall_hits=[after_tile_storage_apply:916.2,after_domain_merge:259.7,after_vt_ci:74.7,pre_egui:223.1,before_readiness:123.1,postupdate_begin:89.1,post_egui:12.0,after_readiness:86.8]
2026-05-27T03:24:28.726307Z  INFO perf: PERF frame=1787.4ms update=1176.0ms egui=510.0ms preview=0.0ms streaming=678.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=157.9ms
2026-05-27T03:24:28.726370Z  INFO stall: STALL culprit=after_tile_storage_apply duration=916.2ms frame=1787.4ms
2026-05-27T03:24:28.736510Z  INFO stall: STALL preupdate_end: 7.24ms
2026-05-27T03:24:28.741460Z  INFO proc_A_dine01::gui::editor::world_preview::preview_readiness: PREVIEW STATE: world=false cam=true tex=true proj=true state=Loading world_ready=false camera_ready=true texture_ready=true projection_ready=true missing=Some("world_generation") contract_valid=true wp_half_x=363.28125 wp_half_y=297.15625 wp_logical_w=726.5625 wp_logical_h=594.3125 viewport_rev=45
2026-05-27T03:24:28.742543Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:28.743348Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:28.746734Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:28.746824Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:28.747623Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:28.748384Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:29.227890Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=26 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:29.246727Z  INFO stall: STALL after_tile_storage_apply: 510.21ms
2026-05-27T03:24:29.269121Z  INFO stall: STALL upd_streaming_reconstruct: 520.35ms
2026-05-27T03:24:29.425063Z  INFO stall: STALL after_domain_merge: 178.34ms
2026-05-27T03:24:29.425106Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:29.435996Z  INFO stall: STALL after_vt_ci: 10.93ms
2026-05-27T03:24:29.453101Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:29.453969Z  WARN worldgen_chrome::preview_egui: PREVIEW_EGUI_REBIND (egui texture id churn — flicker) rebinds_frame=1 rebinds_total=2 projection_revision=2551212574559 window_open=true preview_ready=false texture_bound=true lifecycle=GeneratingWorld
2026-05-27T03:24:29.454540Z  INFO stall: STALL post_egui: 18.03ms
2026-05-27T03:24:29.454828Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:29.454902Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:29.454968Z  INFO stage5_readiness::truth:
========== STAGE5_FULL_APP_TRUTH (post_update_invocation=26 sim_tick=26) ==========
FULL_APP_PROFILE_ACTIVE: true
stage5_readiness_passes: true
--- AppStage5ReadinessReport (hard gates) ---
vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]
vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]
single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]
gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]
preview_render_target_active: true  [src: preview_authoritative_surface]
phase_d_ok: true  [derived: !require_preview || preview_render_target_active]
overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]
particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]
phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]
instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]
phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]
projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]
registered_producers: 2
duplicate_visual_scan_count: 0
--- violations (first = primary suspect) ---
first: (none)
all: []
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---
RepresentationResult: true
WorldRepresentationFrame: true
RenderProjectionGraph: true
CommittedVisualSnapshotFence: true
SharedOverlayFieldBuffers: true
GpuRepresentationMetrics: true
VisualAgreementFrame: true
VtCiMatrixLiveReport: true
AtmospherePartialWriteMetrics: true
PreviewCameraState: true
WorldPreviewGpuRuntime: true
PhaseFLodProofReport: true
GpuIndirectDrawSpine: true
WorldFireParticleDrawDispatch: true
FireSimulationSnapshot: true
MISSING_WIRING_FULL_APP: none

# 2026-05-27T03:24:29.455302Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(WorldGen)) worldgen=Res(State(Ready)) base=Res(State(Editor)) flow=Res(State(PreviewReady)) latch_dismissed=false world_gen_visible=true preview_window_open=true lifecycle=GeneratingWorld last_dismiss="never"
2026-05-27T03:24:29.455317Z  INFO visual_diag: VISUAL_DIAG window frame=25 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:29.455721Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=25 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:29.455317Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=25 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=25 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=45 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:29.455970Z  INFO visual_diag: VISUAL_DIAG camera frame=25 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=25 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:29.456554Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=25 resolved_rev=45 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:29.456727Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=25 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:29.456806Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=25 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:29.456937Z  INFO visual_diag: VISUAL_DIAG render_spine frame=25 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=26 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:29.457200Z  INFO visual_diag: VISUAL_DIAG perf frame=25 tile_raster_ms=177.97030639648438 tile_raster_ran=true world_repr_ms=0.20069999992847443 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.00010000000474974513 readiness_ms=0.1542000025510788 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:29.457343Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=25 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:29.457473Z  INFO perf: PERF wall=728.16 instr=178.33 gap=549.83 | cpu_pre_egui=695.79 cpu_egui=29.49 cpu_post_egui=2.89 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.15 raster=177.97 | upd_attrib sum=521.19 pv_cpu=0.00 pv_gpu=0.01 fire=0.82 stream=520.35 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=177.97 hud=0.00 overlay=0.00 raster_b=177.97 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=29.49 | stall first+preupd=7.24 update=0.00 post_dom=178.34 post_vt=10.93 post→ready=0.00 ready=0.42 post→egui=0.50 egui=18.03 post_egui=0.34 | stall_hits=[preupdate_end:7.2,after_tile_storage_apply:510.2,after_domain_merge:178.3,after_vt_ci:10.9,post_egui:18.0]
2026-05-27T03:24:29.457648Z  INFO perf: PERF frame=728.2ms update=695.8ms egui=29.5ms preview=0.0ms streaming=520.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=178.0ms
2026-05-27T03:24:29.457717Z  INFO stall: STALL culprit=after_tile_storage_apply duration=510.2ms frame=728.2ms
2026-05-27T03:24:29.461856Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:29.461967Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:29.462099Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:29.462157Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:29.462226Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:29.462928Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:29.480063Z  INFO stall: STALL upd_fire_pipeline: 17.23ms
2026-05-27T03:24:29.916672Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=27 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:29.986588Z  INFO stall: STALL after_tile_storage_apply: 527.46ms
2026-05-27T03:24:30.057789Z  INFO stall: STALL upd_streaming_reconstruct: 594.48ms
2026-05-27T03:24:30.151517Z  INFO stall: STALL after_domain_merge: 164.93ms
2026-05-27T03:24:30.151611Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:30.246731Z  INFO stall: STALL after_vt_ci: 95.21ms
2026-05-27T03:24:30.395048Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:30.440225Z  INFO stall: STALL pre_egui: 193.49ms
2026-05-27T03:24:30.441701Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:30.441786Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:30.441868Z  INFO stage5_readiness::truth:
========== STAGE5_FULL_APP_TRUTH (post_update_invocation=27 sim_tick=27) ==========
FULL_APP_PROFILE_ACTIVE: true
stage5_readiness_passes: true
--- AppStage5ReadinessReport (hard gates) ---
vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]
vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]
single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]
gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]
preview_render_target_active: true  [src: preview_authoritative_surface]
phase_d_ok: true  [derived: !require_preview || preview_render_target_active]
overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]
particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]
phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]
instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]
phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]
projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]
registered_producers: 2
duplicate_visual_scan_count: 0
--- violations (first = primary suspect) ---
first: (none)
all: []
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---
RepresentationResult: true
WorldRepresentationFrame: true
RenderProjectionGraph: true
CommittedVisualSnapshotFence: true
SharedOverlayFieldBuffers: true
GpuRepresentationMetrics: true
VisualAgreementFrame: true
VtCiMatrixLiveReport: true
AtmospherePartialWriteMetrics: true
PreviewCameraState: true
WorldPreviewGpuRuntime: true
PhaseFLodProofReport: true
GpuIndirectDrawSpine: true
WorldFireParticleDrawDispatch: true
FireSimulationSnapshot: true
MISSING_WIRING_FULL_APP: none

# 2026-05-27T03:24:30.442214Z  INFO visual_diag: VISUAL_DIAG window frame=26 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:30.442213Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=26 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=26 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=47 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:30.442363Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=26 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:30.442889Z  INFO visual_diag: VISUAL_DIAG camera frame=26 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=26 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:30.443050Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=26 resolved_rev=47 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:30.443213Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=26 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:30.443290Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=26 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:30.443416Z  INFO visual_diag: VISUAL_DIAG render_spine frame=26 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=27 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:30.443672Z  INFO visual_diag: VISUAL_DIAG perf frame=26 tile_raster_ms=149.33938598632813 tile_raster_ran=true world_repr_ms=0.20080000162124634 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00020000000949949026 readiness_ms=0.17960000038146973 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:30.443812Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=26 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:30.443919Z  INFO perf: PERF wall=984.82 instr=149.72 gap=835.10 | cpu_pre_egui=692.43 cpu_egui=289.98 cpu_post_egui=2.41 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.18 raster=149.34 | upd_attrib sum=611.72 pv_cpu=0.00 pv_gpu=0.01 fire=17.23 stream=594.48 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=149.34 hud=0.00 overlay=0.00 raster_b=149.34 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=289.98 | stall first+preupd=0.05 update=0.01 post_dom=164.93 post_vt=95.21 post→ready=1.04 ready=0.38 post→egui=193.49 egui=0.22 post_egui=0.33 | stall_hits=[after_tile_storage_apply:527.5,after_domain_merge:164.9,after_vt_ci:95.2,pre_egui:193.5]
2026-05-27T03:24:30.444006Z  INFO perf: PERF frame=984.8ms update=692.4ms egui=290.0ms preview=0.0ms streaming=594.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=149.3ms
2026-05-27T03:24:30.444066Z  INFO stall: STALL culprit=after_tile_storage_apply duration=527.5ms frame=984.8ms
2026-05-27T03:24:30.452365Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:30.493834Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:30.493935Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:30.494039Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:30.494109Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:30.494860Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:30.937004Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=28 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:30.961511Z  INFO stall: STALL after_tile_storage_apply: 512.81ms
2026-05-27T03:24:30.986069Z  INFO stall: STALL upd_streaming_reconstruct: 490.86ms
2026-05-27T03:24:31.128076Z  INFO stall: STALL after_domain_merge: 166.57ms
2026-05-27T03:24:31.128155Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:31.157587Z  INFO stall: STALL after_vt_ci: 29.51ms
2026-05-27T03:24:31.216442Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:31.239266Z  INFO stall: STALL pre_egui: 81.68ms
2026-05-27T03:24:31.260179Z  INFO stall: STALL before_readiness: 20.91ms
2026-05-27T03:24:31.272058Z  INFO stall: STALL postupdate_begin: 11.88ms
2026-05-27T03:24:31.292196Z  INFO stall: STALL post_egui: 20.14ms
2026-05-27T03:24:31.292371Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:31.292445Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:31.292510Z  INFO stage5_readiness::truth:
========== STAGE5_FULL_APP_TRUTH (post_update_invocation=28 sim_tick=28) ==========
FULL_APP_PROFILE_ACTIVE: true
stage5_readiness_passes: true
--- AppStage5ReadinessReport (hard gates) ---
vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]
vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]
single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]
gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]
preview_render_target_active: true  [src: preview_authoritative_surface]
phase_d_ok: true  [derived: !require_preview || preview_render_target_active]
overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]
particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]
phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]
instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]
phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]
projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]
registered_producers: 2
duplicate_visual_scan_count: 0
--- violations (first = primary suspect) ---
first: (none)
all: []
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---
RepresentationResult: true
WorldRepresentationFrame: true
RenderProjectionGraph: true
CommittedVisualSnapshotFence: true
SharedOverlayFieldBuffers: true
GpuRepresentationMetrics: true
VisualAgreementFrame: true
VtCiMatrixLiveReport: true
AtmospherePartialWriteMetrics: true
PreviewCameraState: true
WorldPreviewGpuRuntime: true
PhaseFLodProofReport: true
GpuIndirectDrawSpine: true
WorldFireParticleDrawDispatch: true
FireSimulationSnapshot: true
MISSING_WIRING_FULL_APP: none

# 2026-05-27T03:24:31.292841Z  INFO visual_diag: VISUAL_DIAG window frame=27 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:31.292841Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=27 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=27 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=49 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:31.292993Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=27 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:31.293521Z  INFO visual_diag: VISUAL_DIAG camera frame=27 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=27 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:31.293685Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=27 resolved_rev=49 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:31.293847Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=27 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:31.293925Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=27 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:31.294050Z  INFO visual_diag: VISUAL_DIAG render_spine frame=27 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=28 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:31.294305Z  INFO visual_diag: VISUAL_DIAG perf frame=27 tile_raster_ms=166.27809143066406 tile_raster_ran=true world_repr_ms=0.19910000264644623 projection_graph_ms=0.000800000037997961 domain_merge_ms=0.00010000000474974513 readiness_ms=0.15160000324249268 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:31.294445Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=27 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:31.294535Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 242.5 ms)
2026-05-27T03:24:31.294611Z  INFO perf: PERF wall=845.88 instr=166.63 gap=679.25 | cpu_pre_egui=679.42 cpu_egui=143.99 cpu_post_egui=22.46 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.15 raster=166.28 | upd_attrib sum=491.91 pv_cpu=0.00 pv_gpu=0.29 fire=0.76 stream=490.86 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=166.28 hud=0.00 overlay=0.00 raster_b=166.28 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=143.99 | stall first+preupd=0.05 update=11.88 post_dom=166.57 post_vt=29.51 post→ready=20.91 ready=0.31 post→egui=81.68 egui=20.14 post_egui=0.32 | stall_hits=[after_tile_storage_apply:512.8,after_domain_merge:166.6,after_vt_ci:29.5,pre_egui:81.7,before_readiness:20.9,postupdate_begin:11.9,post_egui:20.1]
2026-05-27T03:24:31.294715Z  INFO perf: PERF frame=845.9ms update=679.4ms egui=144.0ms preview=0.3ms streaming=490.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=166.3ms
2026-05-27T03:24:31.294777Z  INFO stall: STALL culprit=after_tile_storage_apply duration=512.8ms frame=845.9ms
2026-05-27T03:24:31.313480Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:31.313552Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:31.313602Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:31.313649Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:31.314016Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:31.315031Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:31.331060Z  INFO stall: STALL upd_fire_pipeline: 16.19ms
2026-05-27T03:24:31.748871Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=29 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:31.785891Z  INFO stall: STALL after_tile_storage_apply: 479.52ms
2026-05-27T03:24:31.822757Z  INFO stall: STALL upd_streaming_reconstruct: 506.78ms
2026-05-27T03:24:31.958412Z  INFO stall: STALL after_domain_merge: 172.52ms
2026-05-27T03:24:31.958451Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:31.972425Z  INFO stall: STALL after_vt_ci: 14.01ms
2026-05-27T03:24:31.989159Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:32.010697Z  INFO stall: STALL post_egui: 38.11ms
2026-05-27T03:24:32.041940Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:32.073802Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:32.105913Z  INFO stall: STALL after_readiness: 95.21ms
2026-05-27T03:24:32.105925Z  INFO stage5_readiness::truth:
========== STAGE5_FULL_APP_TRUTH (post_update_invocation=29 sim_tick=29) ==========
FULL_APP_PROFILE_ACTIVE: true
stage5_readiness_passes: true
--- AppStage5ReadinessReport (hard gates) ---
vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]
vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]
single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]
gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]
preview_render_target_active: true  [src: preview_authoritative_surface]
phase_d_ok: true  [derived: !require_preview || preview_render_target_active]
overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]
particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]
phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]
instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]
phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]
projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]
registered_producers: 2
duplicate_visual_scan_count: 0
--- violations (first = primary suspect) ---
first: (none)
all: []
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---
RepresentationResult: true
WorldRepresentationFrame: true
RenderProjectionGraph: true
CommittedVisualSnapshotFence: true
SharedOverlayFieldBuffers: true
GpuRepresentationMetrics: true
VisualAgreementFrame: true
VtCiMatrixLiveReport: true
AtmospherePartialWriteMetrics: true
PreviewCameraState: true
WorldPreviewGpuRuntime: true
PhaseFLodProofReport: true
GpuIndirectDrawSpine: true
WorldFireParticleDrawDispatch: true
FireSimulationSnapshot: true
MISSING_WIRING_FULL_APP: none

# 2026-05-27T03:24:32.174419Z  INFO stall: STALL last: 68.51ms
2026-05-27T03:24:32.174436Z  INFO visual_diag: VISUAL_DIAG window frame=28 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:32.174436Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=28 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=28 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=51 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:32.206351Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=28 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:32.266574Z  INFO visual_diag: VISUAL_DIAG camera frame=28 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=28 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:32.299438Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=28 resolved_rev=51 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:32.308833Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=28 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:32.337430Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=28 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:32.364899Z  INFO visual_diag: VISUAL_DIAG render_spine frame=28 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=29 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:32.397136Z  INFO visual_diag: VISUAL_DIAG perf frame=28 tile_raster_ms=172.2075958251953 tile_raster_ran=true world_repr_ms=0.19839999079704285 projection_graph_ms=0.0008999999845400453 domain_merge_ms=0.00010000000474974513 readiness_ms=63.98749923706055 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:32.429560Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=28 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:32.466613Z  INFO perf: PERF wall=1160.25 instr=236.40 gap=923.85 | cpu_pre_egui=652.08 cpu_egui=52.30 cpu_post_egui=455.87 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=63.99 raster=172.21 | upd_attrib sum=523.28 pv_cpu=0.00 pv_gpu=0.30 fire=16.19 stream=506.78 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=172.21 hud=0.00 overlay=0.00 raster_b=172.21 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=52.30 | stall first+preupd=0.04 update=0.00 post_dom=172.52 post_vt=14.01 post→ready=0.00 ready=95.21 post→egui=0.16 egui=38.11 post_egui=68.51 | stall_hits=[after_tile_storage_apply:479.5,after_domain_merge:172.5,after_vt_ci:14.0,post_egui:38.1,after_readiness:95.2,last:68.5]
2026-05-27T03:24:32.473420Z  INFO perf: PERF frame=1160.2ms update=652.1ms egui=52.3ms preview=0.3ms streaming=506.8ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=172.2ms
2026-05-27T03:24:32.500359Z  INFO stall: STALL culprit=after_tile_storage_apply duration=479.5ms frame=1160.2ms
2026-05-27T03:24:32.536103Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:32.632470Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:32.665917Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:32.777628Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:32.872621Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:32.938030Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:33.321529Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=30 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:33.352154Z  INFO stall: STALL after_tile_storage_apply: 818.04ms
2026-05-27T03:24:33.541429Z  INFO stall: STALL upd_streaming_reconstruct: 603.04ms
2026-05-27T03:24:33.630023Z  INFO stall: STALL after_domain_merge: 277.87ms
2026-05-27T03:24:33.630060Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:33.630277Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:33.647896Z  INFO stall: STALL post_egui: 17.75ms
2026-05-27T03:24:33.665028Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:33.665401Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:33.665467Z  INFO stall: STALL after_readiness: 17.57ms
2026-05-27T03:24:33.665485Z  INFO stage5_readiness::truth:
========== STAGE5_FULL_APP_TRUTH (post_update_invocation=30 sim_tick=30) ==========
FULL_APP_PROFILE_ACTIVE: true
stage5_readiness_passes: true
--- AppStage5ReadinessReport (hard gates) ---
vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]
vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]
single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]
gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]
preview_render_target_active: true  [src: preview_authoritative_surface]
phase_d_ok: true  [derived: !require_preview || preview_render_target_active]
overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]
particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]
phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]
instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]
phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]
projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]
registered_producers: 2
duplicate_visual_scan_count: 0
--- violations (first = primary suspect) ---
first: (none)
all: []
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---
RepresentationResult: true
WorldRepresentationFrame: true
RenderProjectionGraph: true
CommittedVisualSnapshotFence: true
SharedOverlayFieldBuffers: true
GpuRepresentationMetrics: true
VisualAgreementFrame: true
VtCiMatrixLiveReport: true
AtmospherePartialWriteMetrics: true
PreviewCameraState: true
WorldPreviewGpuRuntime: true
PhaseFLodProofReport: true
GpuIndirectDrawSpine: true
WorldFireParticleDrawDispatch: true
FireSimulationSnapshot: true
MISSING_WIRING_FULL_APP: none

# 2026-05-27T03:24:33.665976Z  INFO stage5_readiness::live: READINESS_FRAME_FENCE_OK eval_inv=30 frame_tick=30 passes=true
2026-05-27T03:24:33.665990Z  INFO visual_diag: VISUAL_DIAG window frame=29 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:33.665989Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=29 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=29 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=53 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:33.666203Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=29 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:33.666739Z  INFO visual_diag: VISUAL_DIAG camera frame=29 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=29 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:33.666903Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=29 resolved_rev=53 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:33.667066Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=29 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:33.667144Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=29 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:33.667272Z  INFO visual_diag: VISUAL_DIAG render_spine frame=29 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=30 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:33.667531Z  INFO visual_diag: VISUAL_DIAG perf frame=29 tile_raster_ms=189.0838165283203 tile_raster_ran=true world_repr_ms=0.20440000295639038 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00010000000474974513 readiness_ms=0.45660001039505005 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:33.667673Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=29 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:33.667783Z  INFO perf: PERF wall=1133.70 instr=189.75 gap=943.95 | cpu_pre_egui=1095.96 cpu_egui=17.88 cpu_post_egui=19.86 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.46 raster=189.08 | upd_attrib sum=603.56 pv_cpu=0.00 pv_gpu=0.01 fire=0.49 stream=603.04 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=189.08 hud=0.00 overlay=0.00 raster_b=189.08 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=17.88 | stall first+preupd=0.06 update=0.00 post_dom=277.87 post_vt=0.12 post→ready=0.00 ready=17.57 post→egui=0.00 egui=17.75 post_egui=0.51 | stall_hits=[after_tile_storage_apply:818.0,after_domain_merge:277.9,post_egui:17.8,after_readiness:17.6]
2026-05-27T03:24:33.667884Z  INFO perf: PERF frame=1133.7ms update=1096.0ms egui=17.9ms preview=0.0ms streaming=603.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=189.1ms
2026-05-27T03:24:33.667945Z  INFO stall: STALL culprit=after_tile_storage_apply duration=818.0ms frame=1133.7ms
2026-05-27T03:24:33.742067Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:33.770378Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:33.797347Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:33.813347Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:33.813428Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:33.814106Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:34.269407Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=31 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:34.269600Z  INFO stall: STALL after_tile_storage_apply: 533.21ms
2026-05-27T03:24:34.269978Z  INFO stall: STALL upd_streaming_reconstruct: 455.53ms
2026-05-27T03:24:34.411176Z  INFO stall: STALL after_domain_merge: 141.57ms
2026-05-27T03:24:34.411244Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:34.428009Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:34.429718Z  INFO stall: STALL post_egui: 18.40ms
2026-05-27T03:24:34.430527Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:34.430664Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:34.430801Z  INFO stage5_readiness::truth:
========== STAGE5_FULL_APP_TRUTH (post_update_invocation=31 sim_tick=31) ==========
FULL_APP_PROFILE_ACTIVE: true
stage5_readiness_passes: true
--- AppStage5ReadinessReport (hard gates) ---
vt4_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]
vt5_ok: true  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]
single_fire_extract: true  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]
gpu_field_authoritative: true  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]
preview_render_target_active: true  [src: preview_authoritative_surface]
phase_d_ok: true  [derived: !require_preview || preview_render_target_active]
overlay_from_shared_buffers_only: true  [src: SharedOverlayFieldBuffers resource exists]
particle_lod_scales: true  [src: GpuRepresentationMetrics vs RepresentationResult band]
phase_f_lod_proof_ok: true  [src: PhaseFLodProofReport]
instanced_dispatch_ok: true  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]
phase_f_ok: true  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]
projection_domains (report): 3  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]
registered_producers: 2
duplicate_visual_scan_count: 0
--- violations (first = primary suspect) ---
first: (none)
all: []
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---
RepresentationResult: true
WorldRepresentationFrame: true
RenderProjectionGraph: true
CommittedVisualSnapshotFence: true
SharedOverlayFieldBuffers: true
GpuRepresentationMetrics: true
VisualAgreementFrame: true
VtCiMatrixLiveReport: true
AtmospherePartialWriteMetrics: true
PreviewCameraState: true
WorldPreviewGpuRuntime: true
PhaseFLodProofReport: true
GpuIndirectDrawSpine: true
WorldFireParticleDrawDispatch: true
FireSimulationSnapshot: true
MISSING_WIRING_FULL_APP: none

2026-05-27T03:24:34.431416Z  INFO visual_diag: VISUAL_DIAG window frame=30 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:34.431417Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=30 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=30 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=15 resolved_rev=55 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:34.431685Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=30 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:34.432423Z  INFO visual_diag: VISUAL_DIAG camera frame=30 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=30 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:34.432617Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=30 resolved_rev=55 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:34.432810Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=30 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:34.432917Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=30 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:34.433076Z  INFO visual_diag: VISUAL_DIAG render_spine frame=30 raster_rev=15 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=31 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:34.433385Z  INFO visual_diag: VISUAL_DIAG perf frame=30 tile_raster_ms=141.15829467773438 tile_raster_ran=true world_repr_ms=0.20009998977184296 projection_graph_ms=0.0010999999940395355 domain_merge_ms=0.00010000000474974513 readiness_ms=0.28360000252723694 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:34.433592Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=30 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:34.433757Z  INFO perf: PERF wall=697.39 instr=141.65 gap=555.74 | cpu_pre_egui=674.83 cpu_egui=18.56 cpu_post_egui=4.00 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.28 raster=141.16 | upd_attrib sum=456.02 pv_cpu=0.00 pv_gpu=0.01 fire=0.48 stream=455.53 map_fit=0.01 hud=0.00 wgen=0.00 | budget_sum=141.16 hud=0.00 overlay=0.00 raster_b=141.16 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.56 | stall first+preupd=0.05 update=0.01 post_dom=141.57 post_vt=0.13 post→ready=0.01 ready=1.06 post→egui=0.01 egui=18.40 post_egui=0.60 | stall_hits=[after_tile_storage_apply:533.2,after_domain_merge:141.6,post_egui:18.4]
2026-05-27T03:24:34.433894Z  INFO perf: PERF frame=697.4ms update=674.8ms egui=18.6ms preview=0.0ms streaming=455.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=141.2ms
2026-05-27T03:24:34.434001Z  INFO stall: STALL culprit=after_tile_storage_apply duration=533.2ms frame=697.4ms
2026-05-27T03:24:34.437845Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:34.438138Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:34.438248Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:34.438522Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:34.438608Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:34.466897Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:34.804005Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=32 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:34.804193Z  INFO stall: STALL after_tile_storage_apply: 368.85ms
2026-05-27T03:24:34.804935Z  INFO stall: STALL upd_streaming_reconstruct: 337.60ms
2026-05-27T03:24:34.937207Z  INFO stall: STALL after_domain_merge: 133.02ms
2026-05-27T03:24:34.937265Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:34.937728Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:34.955203Z  INFO stall: STALL post_egui: 17.86ms
2026-05-27T03:24:34.955432Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:34.955538Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:34.955879Z  INFO visual_diag: VISUAL_DIAG window frame=31 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:34.955881Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=31 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=31 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=16 resolved_rev=55 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:34.956092Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=31 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:34.956762Z  INFO visual_diag: VISUAL_DIAG camera frame=31 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=31 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:34.956986Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=31 resolved_rev=55 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:34.957209Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=31 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535055 sim_map_proj_rev=3092391454447
2026-05-27T03:24:34.957338Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=31 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:34.957492Z  INFO visual_diag: VISUAL_DIAG render_spine frame=31 raster_rev=16 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=32 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:34.957774Z  INFO visual_diag: VISUAL_DIAG perf frame=31 tile_raster_ms=130.9864044189453 tile_raster_ran=true world_repr_ms=0.25380000472068787 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.22050000727176666 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:34.957939Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=31 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:34.958077Z  INFO perf: PERF wall=522.80 instr=131.46 gap=391.33 | cpu_pre_egui=501.95 cpu_egui=18.01 cpu_post_egui=2.84 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=130.99 | upd_attrib sum=338.22 pv_cpu=0.00 pv_gpu=0.01 fire=0.61 stream=337.60 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=130.99 hud=0.00 overlay=0.00 raster_b=130.99 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.01 | stall first+preupd=0.08 update=0.00 post_dom=133.02 post_vt=0.12 post→ready=0.01 ready=0.44 post→egui=0.01 egui=17.86 post_egui=0.22 | stall_hits=[after_tile_storage_apply:368.9,after_domain_merge:133.0,post_egui:17.9]
2026-05-27T03:24:34.958194Z  INFO perf: PERF frame=522.8ms update=501.9ms egui=18.0ms preview=0.0ms streaming=337.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=131.0ms
2026-05-27T03:24:34.958286Z  INFO stall: STALL culprit=after_tile_storage_apply duration=368.9ms frame=522.8ms
2026-05-27T03:24:34.961103Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:34.961228Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:34.961378Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:34.961471Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:34.961553Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:34.988733Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:35.326419Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=33 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:35.326580Z  INFO stall: STALL after_tile_storage_apply: 367.57ms
2026-05-27T03:24:35.326952Z  INFO stall: STALL upd_streaming_reconstruct: 337.76ms
2026-05-27T03:24:35.465936Z  INFO stall: STALL after_domain_merge: 139.36ms
2026-05-27T03:24:35.465984Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:35.466434Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:35.484104Z  INFO stall: STALL post_egui: 18.06ms
2026-05-27T03:24:35.484773Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:35.484870Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:35.485272Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=32 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=32 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=17 resolved_rev=57 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:35.485279Z  INFO visual_diag: VISUAL_DIAG window frame=32 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:35.485928Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=32 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:35.486145Z  INFO visual_diag: VISUAL_DIAG camera frame=32 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=32 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:35.486343Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=32 resolved_rev=57 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:35.486545Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=32 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535056 sim_map_proj_rev=3092392454450
2026-05-27T03:24:35.486652Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=32 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:35.486813Z  INFO visual_diag: VISUAL_DIAG render_spine frame=32 raster_rev=17 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=33 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:35.487101Z  INFO visual_diag: VISUAL_DIAG perf frame=32 tile_raster_ms=137.15139770507813 tile_raster_ran=true world_repr_ms=0.26440000534057617 projection_graph_ms=0.0008999999845400453 domain_merge_ms=0.00020000000949949026 readiness_ms=0.20499999821186066 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:35.487275Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=32 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:35.487437Z  INFO perf: PERF wall=528.45 instr=137.62 gap=390.83 | cpu_pre_egui=506.98 cpu_egui=18.18 cpu_post_egui=3.29 gpu_gap=0.00 | spine=0.00 world_repr=0.26 graph=0.00 merge=0.00 atm=0.00 readiness=0.20 raster=137.15 | upd_attrib sum=338.39 pv_cpu=0.00 pv_gpu=0.01 fire=0.61 stream=337.76 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=137.15 hud=0.00 overlay=0.00 raster_b=137.15 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.18 | stall first+preupd=0.06 update=0.00 post_dom=139.36 post_vt=0.10 post→ready=0.00 ready=0.86 post→egui=0.00 egui=18.06 post_egui=0.30 | stall_hits=[after_tile_storage_apply:367.6,after_domain_merge:139.4,post_egui:18.1]
2026-05-27T03:24:35.487567Z  INFO perf: PERF frame=528.4ms update=507.0ms egui=18.2ms preview=0.0ms streaming=337.8ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=137.2ms
2026-05-27T03:24:35.487667Z  INFO stall: STALL culprit=after_tile_storage_apply duration=367.6ms frame=528.4ms
2026-05-27T03:24:35.490692Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:35.490806Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:35.490966Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:35.491058Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:35.491137Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:35.518859Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:35.848971Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=34 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:35.849150Z  INFO stall: STALL after_tile_storage_apply: 360.73ms
2026-05-27T03:24:35.850002Z  INFO stall: STALL upd_streaming_reconstruct: 330.74ms
2026-05-27T03:24:35.989808Z  INFO stall: STALL after_domain_merge: 140.66ms
2026-05-27T03:24:35.989858Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:35.990373Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:36.008050Z  INFO stall: STALL post_egui: 18.12ms
2026-05-27T03:24:36.008472Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:36.008575Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:36.008842Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=33 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=33 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=18 resolved_rev=59 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:36.008850Z  INFO visual_diag: VISUAL_DIAG window frame=33 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:36.009849Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=33 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:36.010054Z  INFO visual_diag: VISUAL_DIAG camera frame=33 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=33 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:36.010241Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=33 resolved_rev=59 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:36.010430Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=33 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535057 sim_map_proj_rev=3092393454453
2026-05-27T03:24:36.010549Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=33 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:36.010732Z  INFO visual_diag: VISUAL_DIAG render_spine frame=33 raster_rev=18 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=34 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:36.011074Z  INFO visual_diag: VISUAL_DIAG perf frame=33 tile_raster_ms=138.05239868164063 tile_raster_ran=true world_repr_ms=0.23999999463558197 projection_graph_ms=0.0010000000474974513 domain_merge_ms=0.0003000000142492354 readiness_ms=0.21610000729560852 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:36.011276Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=33 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:36.011433Z  INFO perf: PERF wall=523.09 instr=138.51 gap=384.58 | cpu_pre_egui=501.48 cpu_egui=18.24 cpu_post_egui=3.36 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=138.05 | upd_attrib sum=331.30 pv_cpu=0.00 pv_gpu=0.00 fire=0.55 stream=330.74 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=138.05 hud=0.00 overlay=0.00 raster_b=138.05 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.24 | stall first+preupd=0.09 update=0.00 post_dom=140.66 post_vt=0.10 post→ready=0.01 ready=0.62 post→egui=0.01 egui=18.12 post_egui=0.16 | stall_hits=[after_tile_storage_apply:360.7,after_domain_merge:140.7,post_egui:18.1]
2026-05-27T03:24:36.011568Z  INFO perf: PERF frame=523.1ms update=501.5ms egui=18.2ms preview=0.0ms streaming=330.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=138.1ms
2026-05-27T03:24:36.011668Z  INFO stall: STALL culprit=after_tile_storage_apply duration=360.7ms frame=523.1ms
2026-05-27T03:24:36.014517Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:36.014619Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:36.014766Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:36.014888Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:36.015033Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:36.041882Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:36.369998Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=35 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=true
2026-05-27T03:24:36.370163Z  INFO stall: STALL after_tile_storage_apply: 357.63ms
2026-05-27T03:24:36.370575Z  INFO stall: STALL upd_streaming_reconstruct: 328.33ms
2026-05-27T03:24:36.514460Z  INFO stall: STALL after_domain_merge: 144.30ms
2026-05-27T03:24:36.514512Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:36.514946Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:36.532883Z  INFO stall: STALL post_egui: 18.32ms
2026-05-27T03:24:36.533473Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:36.533576Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:36.533854Z  INFO visual_diag: VISUAL_DIAG window frame=34 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:36.533854Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=34 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=34 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=19 resolved_rev=61 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:36.534060Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=34 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:36.534729Z  INFO visual_diag: VISUAL_DIAG camera frame=34 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=34 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:36.534925Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=34 resolved_rev=61 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:36.535119Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=34 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535058 sim_map_proj_rev=3092394454456
2026-05-27T03:24:36.535230Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=34 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:36.535391Z  INFO visual_diag: VISUAL_DIAG render_spine frame=34 raster_rev=19 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=35 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:36.535719Z  INFO visual_diag: VISUAL_DIAG perf frame=34 tile_raster_ms=141.31700134277344 tile_raster_ran=true world_repr_ms=0.2078000009059906 projection_graph_ms=0.0010999999940395355 domain_merge_ms=0.0003000000142492354 readiness_ms=0.21580000221729279 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:36.538545Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=34 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:36.538709Z  INFO perf: PERF wall=526.22 instr=141.74 gap=384.48 | cpu_pre_egui=501.98 cpu_egui=18.45 cpu_post_egui=5.79 gpu_gap=0.00 | spine=0.00 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=141.32 | upd_attrib sum=328.85 pv_cpu=0.00 pv_gpu=0.00 fire=0.51 stream=328.33 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=141.32 hud=0.00 overlay=0.00 raster_b=141.32 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.45 | stall first+preupd=0.06 update=0.00 post_dom=144.30 post_vt=0.10 post→ready=0.00 ready=0.79 post→egui=0.00 egui=18.32 post_egui=0.16 | stall_hits=[after_tile_storage_apply:357.6,after_domain_merge:144.3,post_egui:18.3]
2026-05-27T03:24:36.538865Z  INFO perf: PERF frame=526.2ms update=502.0ms egui=18.4ms preview=0.0ms streaming=328.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=141.3ms
2026-05-27T03:24:36.538978Z  INFO stall: STALL culprit=after_tile_storage_apply duration=357.6ms frame=526.2ms
2026-05-27T03:24:36.541654Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:36.542114Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:36.542214Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:36.542292Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:36.542374Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:36.570247Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:36.903919Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=36 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:36.904085Z  INFO stall: STALL after_tile_storage_apply: 364.50ms
2026-05-27T03:24:36.904853Z  INFO stall: STALL upd_streaming_reconstruct: 334.19ms
2026-05-27T03:24:37.051687Z  INFO stall: STALL after_domain_merge: 147.60ms
2026-05-27T03:24:37.051734Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:37.052207Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:37.070264Z  INFO stall: STALL post_egui: 18.44ms
2026-05-27T03:24:37.070898Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:37.071001Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:37.071299Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=35 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=35 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=20 resolved_rev=63 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:37.071306Z  INFO visual_diag: VISUAL_DIAG window frame=35 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:37.071888Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=35 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:37.072129Z  INFO visual_diag: VISUAL_DIAG camera frame=35 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=35 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:37.072324Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=35 resolved_rev=63 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:37.072515Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=35 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535059 sim_map_proj_rev=3092395454459
2026-05-27T03:24:37.072623Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=35 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:37.072781Z  INFO visual_diag: VISUAL_DIAG render_spine frame=35 raster_rev=20 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=36 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:37.073062Z  INFO visual_diag: VISUAL_DIAG perf frame=35 tile_raster_ms=144.1787872314453 tile_raster_ran=true world_repr_ms=0.24089999496936798 projection_graph_ms=0.0008999999845400453 domain_merge_ms=0.0003000000142492354 readiness_ms=0.2176000028848648 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:37.073283Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=35 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:37.076156Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 247.3 ms)
2026-05-27T03:24:37.076507Z  INFO perf: PERF wall=536.64 instr=144.64 gap=392.00 | cpu_pre_egui=512.17 cpu_egui=18.59 cpu_post_egui=5.88 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=144.18 | upd_attrib sum=334.75 pv_cpu=0.00 pv_gpu=0.00 fire=0.55 stream=334.19 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=144.18 hud=0.00 overlay=0.00 raster_b=144.18 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.59 | stall first+preupd=0.06 update=0.00 post_dom=147.60 post_vt=0.13 post→ready=0.00 ready=0.84 post→egui=0.01 egui=18.44 post_egui=0.18 | stall_hits=[after_tile_storage_apply:364.5,after_domain_merge:147.6,post_egui:18.4]
2026-05-27T03:24:37.076656Z  INFO perf: PERF frame=536.6ms update=512.2ms egui=18.6ms preview=0.0ms streaming=334.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=144.2ms
2026-05-27T03:24:37.076756Z  INFO stall: STALL culprit=after_tile_storage_apply duration=364.5ms frame=536.6ms
2026-05-27T03:24:37.079733Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:37.079998Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:37.080088Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:37.080167Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:37.080246Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:37.107658Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:37.442019Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=37 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:37.442195Z  INFO stall: STALL after_tile_storage_apply: 364.65ms
2026-05-27T03:24:37.442581Z  INFO stall: STALL upd_streaming_reconstruct: 334.55ms
2026-05-27T03:24:37.595653Z  INFO stall: STALL after_domain_merge: 153.46ms
2026-05-27T03:24:37.595697Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:37.596195Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:37.613942Z  INFO stall: STALL post_egui: 18.18ms
2026-05-27T03:24:37.614616Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:37.614721Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:37.615007Z  INFO visual_diag: VISUAL_DIAG window frame=36 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:37.615007Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=36 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=36 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=21 resolved_rev=65 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:37.615215Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=36 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:37.615842Z  INFO visual_diag: VISUAL_DIAG camera frame=36 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=36 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:37.616031Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=36 resolved_rev=65 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:37.616219Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=36 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535060 sim_map_proj_rev=3092396454462
2026-05-27T03:24:37.616323Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=36 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:37.616483Z  INFO visual_diag: VISUAL_DIAG render_spine frame=36 raster_rev=21 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=37 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:37.616777Z  INFO visual_diag: VISUAL_DIAG perf frame=36 tile_raster_ms=149.84390258789063 tile_raster_ran=true world_repr_ms=0.19950000941753387 projection_graph_ms=0.000800000037997961 domain_merge_ms=0.0003000000142492354 readiness_ms=0.21620000898838043 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:37.616945Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=36 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:37.617082Z  INFO perf: PERF wall=539.58 instr=150.26 gap=389.32 | cpu_pre_egui=518.17 cpu_egui=18.30 cpu_post_egui=3.10 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=149.84 | upd_attrib sum=335.07 pv_cpu=0.00 pv_gpu=0.01 fire=0.50 stream=334.55 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=149.84 hud=0.00 overlay=0.00 raster_b=149.84 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.30 | stall first+preupd=0.07 update=0.00 post_dom=153.46 post_vt=0.10 post→ready=0.00 ready=0.88 post→egui=0.00 egui=18.18 post_egui=0.17 | stall_hits=[after_tile_storage_apply:364.6,after_domain_merge:153.5,post_egui:18.2]
2026-05-27T03:24:37.617197Z  INFO perf: PERF frame=539.6ms update=518.2ms egui=18.3ms preview=0.0ms streaming=334.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=149.8ms
2026-05-27T03:24:37.617285Z  INFO stall: STALL culprit=after_tile_storage_apply duration=364.6ms frame=539.6ms
2026-05-27T03:24:37.620110Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:37.620199Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:37.620293Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:37.620433Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:37.620602Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:37.649302Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:37.975410Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=38 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:37.975595Z  INFO stall: STALL after_tile_storage_apply: 357.63ms
2026-05-27T03:24:37.976220Z  INFO stall: STALL upd_streaming_reconstruct: 326.48ms
2026-05-27T03:24:38.135904Z  INFO stall: STALL after_domain_merge: 160.31ms
2026-05-27T03:24:38.135941Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:38.136432Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:38.153979Z  INFO stall: STALL post_egui: 17.96ms
2026-05-27T03:24:38.154579Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:38.154678Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:38.154953Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=37 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=37 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=22 resolved_rev=67 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:38.154959Z  INFO visual_diag: VISUAL_DIAG window frame=37 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:38.155546Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=37 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:38.155788Z  INFO visual_diag: VISUAL_DIAG camera frame=37 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=37 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:38.155984Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=37 resolved_rev=67 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:38.156178Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=37 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535061 sim_map_proj_rev=3092397454465
2026-05-27T03:24:38.156287Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=37 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:38.156446Z  INFO visual_diag: VISUAL_DIAG render_spine frame=37 raster_rev=22 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=38 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:38.156733Z  INFO visual_diag: VISUAL_DIAG perf frame=37 tile_raster_ms=155.6822052001953 tile_raster_ran=true world_repr_ms=0.2378000020980835 projection_graph_ms=0.0058999997563660145 domain_merge_ms=0.00010000000474974513 readiness_ms=0.20979999005794525 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:38.156902Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=37 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:38.157041Z  INFO perf: PERF wall=539.14 instr=156.14 gap=383.00 | cpu_pre_egui=518.03 cpu_egui=18.08 cpu_post_egui=3.03 gpu_gap=0.00 | spine=0.01 world_repr=0.24 graph=0.01 merge=0.00 atm=0.00 readiness=0.21 raster=155.68 | upd_attrib sum=327.13 pv_cpu=0.00 pv_gpu=0.03 fire=0.62 stream=326.48 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=155.68 hud=0.00 overlay=0.00 raster_b=155.68 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.08 | stall first+preupd=0.08 update=0.00 post_dom=160.31 post_vt=0.10 post→ready=0.00 ready=0.79 post→egui=0.01 egui=17.96 post_egui=0.17 | stall_hits=[after_tile_storage_apply:357.6,after_domain_merge:160.3,post_egui:18.0]
2026-05-27T03:24:38.157183Z  INFO perf: PERF frame=539.1ms update=518.0ms egui=18.1ms preview=0.0ms streaming=326.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=155.7ms
2026-05-27T03:24:38.157282Z  INFO stall: STALL culprit=after_tile_storage_apply duration=357.6ms frame=539.1ms
2026-05-27T03:24:38.160292Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:38.160440Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:38.160602Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:38.160682Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:38.160758Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:38.188526Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:38.523432Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=39 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:38.523598Z  INFO stall: STALL after_tile_storage_apply: 365.36ms
2026-05-27T03:24:38.523980Z  INFO stall: STALL upd_streaming_reconstruct: 335.05ms
2026-05-27T03:24:38.686375Z  INFO stall: STALL after_domain_merge: 162.78ms
2026-05-27T03:24:38.686416Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:38.686877Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:38.704191Z  INFO stall: STALL post_egui: 17.70ms
2026-05-27T03:24:38.704940Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:38.705030Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:38.705292Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=38 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=38 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=23 resolved_rev=69 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:38.705301Z  INFO visual_diag: VISUAL_DIAG window frame=38 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:38.705888Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=38 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:38.706136Z  INFO visual_diag: VISUAL_DIAG camera frame=38 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=38 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:38.706362Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=38 resolved_rev=69 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:38.706577Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=38 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535062 sim_map_proj_rev=3092398454468
2026-05-27T03:24:38.706681Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=38 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:38.706835Z  INFO visual_diag: VISUAL_DIAG render_spine frame=38 raster_rev=23 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=39 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:38.707112Z  INFO visual_diag: VISUAL_DIAG perf frame=38 tile_raster_ms=158.37939453125 tile_raster_ran=true world_repr_ms=0.2354000061750412 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.1917000114917755 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:38.707278Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=38 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:38.707413Z  INFO perf: PERF wall=549.21 instr=158.81 gap=390.40 | cpu_pre_egui=528.19 cpu_egui=17.82 cpu_post_egui=3.20 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.19 raster=158.38 | upd_attrib sum=335.61 pv_cpu=0.00 pv_gpu=0.01 fire=0.54 stream=335.05 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=158.38 hud=0.00 overlay=0.00 raster_b=158.38 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=17.82 | stall first+preupd=0.05 update=0.00 post_dom=162.78 post_vt=0.10 post→ready=0.01 ready=0.93 post→egui=0.01 egui=17.70 post_egui=0.16 | stall_hits=[after_tile_storage_apply:365.4,after_domain_merge:162.8,post_egui:17.7]
2026-05-27T03:24:38.707566Z  INFO perf: PERF frame=549.2ms update=528.2ms egui=17.8ms preview=0.0ms streaming=335.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=158.4ms
2026-05-27T03:24:38.707680Z  INFO stall: STALL culprit=after_tile_storage_apply duration=365.4ms frame=549.2ms
2026-05-27T03:24:38.710169Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:38.710266Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:38.710363Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:38.710483Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:38.710633Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:38.737771Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:39.070106Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=40 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:39.070255Z  INFO stall: STALL after_tile_storage_apply: 362.02ms
2026-05-27T03:24:39.070613Z  INFO stall: STALL upd_streaming_reconstruct: 332.49ms
2026-05-27T03:24:39.237161Z  INFO stall: STALL after_domain_merge: 166.91ms
2026-05-27T03:24:39.237192Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:39.237668Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:39.255419Z  INFO stall: STALL post_egui: 18.15ms
2026-05-27T03:24:39.256188Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:39.256291Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:39.256595Z  INFO visual_diag: VISUAL_DIAG window frame=39 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:39.256596Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=39 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=39 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=24 resolved_rev=71 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:39.256773Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=39 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:39.257362Z  INFO visual_diag: VISUAL_DIAG camera frame=39 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=39 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:39.257551Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=39 resolved_rev=71 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:39.257740Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=39 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535063 sim_map_proj_rev=3092399454471
2026-05-27T03:24:39.257844Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=39 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:39.257999Z  INFO visual_diag: VISUAL_DIAG render_spine frame=39 raster_rev=24 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=40 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:39.258280Z  INFO visual_diag: VISUAL_DIAG perf frame=39 tile_raster_ms=162.04010009765625 tile_raster_ran=true world_repr_ms=0.19099999964237213 projection_graph_ms=0.000699999975040555 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21559999883174896 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:39.258446Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=39 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:39.258589Z  INFO perf: PERF wall=550.40 instr=162.45 gap=387.95 | cpu_pre_egui=528.99 cpu_egui=18.27 cpu_post_egui=3.15 gpu_gap=0.00 | spine=0.00 world_repr=0.19 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=162.04 | upd_attrib sum=332.98 pv_cpu=0.00 pv_gpu=0.01 fire=0.47 stream=332.49 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=162.04 hud=0.00 overlay=0.00 raster_b=162.04 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.27 | stall first+preupd=0.07 update=0.00 post_dom=166.91 post_vt=0.10 post→ready=0.00 ready=0.97 post→egui=0.00 egui=18.15 post_egui=0.19 | stall_hits=[after_tile_storage_apply:362.0,after_domain_merge:166.9,post_egui:18.2]
2026-05-27T03:24:39.258706Z  INFO perf: PERF frame=550.4ms update=529.0ms egui=18.3ms preview=0.0ms streaming=332.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=162.0ms
2026-05-27T03:24:39.258796Z  INFO stall: STALL culprit=after_tile_storage_apply duration=362.0ms frame=550.4ms
2026-05-27T03:24:39.261924Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:39.262970Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:39.263059Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:39.263137Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:39.263214Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:39.291728Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:39.632026Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=41 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:39.632188Z  INFO stall: STALL after_tile_storage_apply: 372.21ms
2026-05-27T03:24:39.632592Z  INFO stall: STALL upd_streaming_reconstruct: 340.50ms
2026-05-27T03:24:39.804652Z  INFO stall: STALL after_domain_merge: 172.46ms
2026-05-27T03:24:39.804695Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:39.805573Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:39.823078Z  INFO stall: STALL post_egui: 18.30ms
2026-05-27T03:24:39.823923Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:39.824024Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:39.824319Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=40 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=40 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=25 resolved_rev=73 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:39.824325Z  INFO visual_diag: VISUAL_DIAG window frame=40 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:39.824974Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=40 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:39.825187Z  INFO visual_diag: VISUAL_DIAG camera frame=40 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=40 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:39.825382Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=40 resolved_rev=73 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:39.825580Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=40 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535064 sim_map_proj_rev=3092400454474
2026-05-27T03:24:39.825686Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=40 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:39.825854Z  INFO visual_diag: VISUAL_DIAG render_spine frame=40 raster_rev=25 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=41 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:39.826134Z  INFO visual_diag: VISUAL_DIAG perf frame=40 tile_raster_ms=167.19400024414063 tile_raster_ran=true world_repr_ms=0.2117999941110611 projection_graph_ms=0.000800000037997961 domain_merge_ms=0.00020000000949949026 readiness_ms=0.21699999272823334 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:39.826301Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=40 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:39.826436Z  INFO perf: PERF wall=566.50 instr=167.63 gap=398.87 | cpu_pre_egui=544.73 cpu_egui=18.44 cpu_post_egui=3.33 gpu_gap=0.00 | spine=0.00 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=167.19 | upd_attrib sum=341.02 pv_cpu=0.00 pv_gpu=0.01 fire=0.51 stream=340.50 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=167.19 hud=0.00 overlay=0.00 raster_b=167.19 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.44 | stall first+preupd=0.06 update=0.00 post_dom=172.46 post_vt=0.12 post→ready=0.00 ready=1.04 post→egui=0.01 egui=18.30 post_egui=0.18 | stall_hits=[after_tile_storage_apply:372.2,after_domain_merge:172.5,post_egui:18.3]
2026-05-27T03:24:39.826557Z  INFO perf: PERF frame=566.5ms update=544.7ms egui=18.4ms preview=0.0ms streaming=340.5ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=167.2ms
2026-05-27T03:24:39.826650Z  INFO stall: STALL culprit=after_tile_storage_apply duration=372.2ms frame=566.5ms
2026-05-27T03:24:39.832673Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:39.832978Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:39.833109Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:39.833191Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:39.833267Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:39.962759Z  INFO proc_A_dine01::terrain::generation::world_generator_enhanced: World generation completed (Full)
2026-05-27T03:24:39.981413Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:40.310643Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=42 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:40.310816Z  INFO stall: STALL after_tile_storage_apply: 480.40ms
2026-05-27T03:24:40.311112Z  INFO stall: STALL upd_streaming_reconstruct: 329.27ms
2026-05-27T03:24:40.486801Z  INFO stall: STALL after_domain_merge: 175.98ms
2026-05-27T03:24:40.486861Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:40.503440Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:40.504795Z  INFO stall: STALL post_egui: 17.87ms
2026-05-27T03:24:40.505535Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:40.505640Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:40.505948Z  INFO visual_diag: VISUAL_DIAG window frame=41 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=2 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:40.505949Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=41 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=41 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=27 resolved_rev=75 app=1 base=1 flow=2 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:40.506152Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=41 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:40.508061Z  INFO visual_diag: VISUAL_DIAG camera frame=41 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=41 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:40.508248Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=41 resolved_rev=75 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:40.508434Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=41 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535065 sim_map_proj_rev=3092401454477
2026-05-27T03:24:40.508546Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=41 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:40.508710Z  INFO visual_diag: VISUAL_DIAG render_spine frame=41 raster_rev=27 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=42 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:40.508991Z  INFO visual_diag: VISUAL_DIAG perf frame=41 tile_raster_ms=170.84799194335938 tile_raster_ran=true world_repr_ms=0.23980000615119934 projection_graph_ms=0.0008999999845400453 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2232999950647354 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:40.509162Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=41 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:40.509305Z  INFO perf: PERF wall=678.94 instr=171.31 gap=507.63 | cpu_pre_egui=656.45 cpu_egui=18.01 cpu_post_egui=4.48 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=170.85 | upd_attrib sum=329.87 pv_cpu=0.00 pv_gpu=0.00 fire=0.58 stream=329.27 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=170.85 hud=0.00 overlay=0.00 raster_b=170.85 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=18.01 | stall first+preupd=0.08 update=0.00 post_dom=175.98 post_vt=0.12 post→ready=0.00 ready=0.95 post→egui=0.00 egui=17.87 post_egui=0.19 | stall_hits=[after_tile_storage_apply:480.4,after_domain_merge:176.0,post_egui:17.9]
2026-05-27T03:24:40.509426Z  INFO perf: PERF frame=678.9ms update=656.5ms egui=18.0ms preview=0.0ms streaming=329.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=170.8ms
2026-05-27T03:24:40.509520Z  INFO stall: STALL culprit=after_tile_storage_apply duration=480.4ms frame=678.9ms
2026-05-27T03:24:40.510576Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="flow_full_ready" latch_dismissed=true world_gen_visible=false preview_window_open=false
2026-05-27T03:24:40.510701Z  INFO world_gen::flow: FullReady: auto-dismissed World Generator panel and World Preview window (reopen via Escape → pause or F8)
2026-05-27T03:24:40.511477Z  INFO proc_A_dine01::gui::editor::world_preview::preview_readiness: PREVIEW STATE: world=true cam=true tex=true proj=true state=Ready world_ready=true camera_ready=true texture_ready=true projection_ready=true missing=None contract_valid=true wp_half_x=363.28125 wp_half_y=297.15625 wp_logical_w=726.5625 wp_logical_h=594.3125 viewport_rev=77
2026-05-27T03:24:40.512767Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:40.512943Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:40.513087Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:40.513094Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="ux_enter_world" latch_dismissed=true world_gen_visible=false preview_window_open=false
2026-05-27T03:24:40.513182Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:40.513402Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:40.514301Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:40.855545Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=43 world_frame_present=true overlay_rev=0 overlay_chunk_cells=0 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:40.855742Z  INFO stall: STALL after_tile_storage_apply: 345.51ms
2026-05-27T03:24:40.856525Z  INFO stall: STALL upd_streaming_reconstruct: 341.88ms
2026-05-27T03:24:40.999558Z  INFO test_harness::logistics: LOG-E01 visual proof: seeded transport_edges=2 logistics_edges=2 overlay_rows=2
2026-05-27T03:24:41.000069Z  INFO stall: STALL after_domain_merge: 144.33ms
2026-05-27T03:24:41.000123Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:41.000316Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:41.017499Z  INFO stall: STALL post_egui: 17.31ms
2026-05-27T03:24:41.018025Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:41.018120Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:41.018421Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(WorldGen)) worldgen=Res(State(Ready)) base=Res(State(Editor)) flow=Res(State(FullReady)) latch_dismissed=true world_gen_visible=false preview_window_open=false lifecycle=Uninitialized last_dismiss="never"
2026-05-27T03:24:41.018428Z  INFO visual_diag: VISUAL_DIAG window frame=42 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=1 base=1 flow=3 worldgen=3 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:41.018425Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=42 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(0.0, 0.0) sim_wh=Vec2(0.0, 0.0) settle_streak=0 layout_settled=false sim_held=false last_commit="" frozen=false pending_wh=Vec2(0.0, 0.0) cam_hole=false render_hole=false cam_invalid_streak=42 cam_valid_streak=0 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=27 resolved_rev=77 app=1 base=1 flow=3 worldgen=3 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:41.018761Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=42 sim_valid=false sim_adequate=false sim_min=Vec2(0.0, 0.0) sim_max=Vec2(0.0, 0.0) sim_wh=(0, 0) measured_valid=false measured_wh=(0, 0) committed_wh=(0, 0) sim_held=false settle_streak=0 layout_settled=false frozen=false last_commit="" pending_wh=(0, 0) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(0.0, 0.0)
2026-05-27T03:24:41.019426Z  INFO visual_diag: VISUAL_DIAG camera frame=42 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=42 latch_valid_streak=0 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:41.019625Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=42 resolved_rev=77 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=false minimap_wh=(0, 0) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:41.019820Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=42 world_preview_proj_rev=2551212574559 minimap_proj_rev=1374389535067 sim_map_proj_rev=3092403454483
2026-05-27T03:24:41.019931Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=42 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:41.020093Z  INFO visual_diag: VISUAL_DIAG render_spine frame=42 raster_rev=27 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=0 fire_spark_rows=0 fire_spark_phase="A+B" fire_spark_scatter_slots=0 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=43 overlay_rev=0 overlay_chunk_cells=0 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=0 gpu_draw=0
2026-05-27T03:24:41.020386Z  INFO visual_diag: VISUAL_DIAG perf frame=42 tile_raster_ms=139.90550231933594 tile_raster_ran=true world_repr_ms=0.1932000070810318 projection_graph_ms=0.000800000037997961 domain_merge_ms=0.0003000000142492354 readiness_ms=0.20189999043941498 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:41.020588Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=42 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:41.020729Z  INFO perf: PERF wall=510.57 instr=140.30 gap=370.27 | cpu_pre_egui=489.93 cpu_egui=17.44 cpu_post_egui=3.20 gpu_gap=0.00 | spine=0.00 world_repr=0.19 graph=0.00 merge=0.00 atm=0.00 readiness=0.20 raster=139.91 | upd_attrib sum=342.37 pv_cpu=0.00 pv_gpu=0.01 fire=0.47 stream=341.88 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=139.91 hud=0.00 overlay=0.00 raster_b=139.91 particles=0.00 residency=0.00 tex_reg=0.00 render_x=0.00 | egui_unbudgeted=17.44 | stall first+preupd=0.09 update=0.00 post_dom=144.33 post_vt=0.11 post→ready=0.01 ready=0.71 post→egui=0.01 egui=17.31 post_egui=0.20 | stall_hits=[after_tile_storage_apply:345.5,after_domain_merge:144.3,post_egui:17.3]
2026-05-27T03:24:41.020848Z  INFO perf: PERF frame=510.6ms update=489.9ms egui=17.4ms preview=0.0ms streaming=341.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=139.9ms
2026-05-27T03:24:41.020938Z  INFO stall: STALL culprit=after_tile_storage_apply duration=345.5ms frame=510.6ms
2026-05-27T03:24:41.022403Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="enter_simulation" latch_dismissed=true world_gen_visible=false preview_window_open=false
2026-05-27T03:24:41.024545Z  INFO worldgen_chrome::dismiss: CHROME_DISMISS reason="ux_on_enter_in_game" latch_dismissed=true world_gen_visible=false preview_window_open=false
2026-05-27T03:24:41.025592Z  INFO proc_A_dine01::render::viewport_pipeline: resolved viewport=minimap_panel revision=78 logical=(260.0,220.0) physical=260x220
2026-05-27T03:24:41.026959Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:41.027065Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:41.027158Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:41.027246Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:41.028265Z  INFO test_harness::fire: spawned test scene chunk slabs nx=10 ny=10 (world 320x320)
2026-05-27T03:24:41.028372Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:41.582505Z  INFO test_harness::fire: test scene seeded shared overlay fire cells=28
2026-05-27T03:24:41.586039Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:41.586691Z  INFO world_representation::lod: WorldRepresentation: zoom=76.556 zoom_α=1.000 → LOD band LocalTactical (LT)
2026-05-27T03:24:41.587806Z  INFO stage5_readiness::live: READINESS_PROJECTION_GRAPH_BUILD dom=3 tick=44 order=fire+logistics+ecology fire_inst=0 fire_heat=0 log_rows=0 eco_rows=100 fire_snap=44 log_snap=44 eco_snap=44
2026-05-27T03:24:42.220622Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=44 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=0 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:42.220832Z  INFO stall: STALL after_tile_storage_apply: 1199.04ms
2026-05-27T03:24:42.221666Z  INFO stall: STALL upd_streaming_reconstruct: 634.84ms
2026-05-27T03:24:42.439686Z  INFO test_harness::industrial: IND-E02 visual seed: committed concrete_portland chain (mine → kiln → mixer)
2026-05-27T03:24:42.448259Z  INFO test_harness::logistics: S7P-LOG-001: spawned aluminum chain on road tiles [(0, 0), (1, 0), (2, 0)]
2026-05-27T03:24:42.448924Z  INFO stall: STALL after_domain_merge: 228.09ms
2026-05-27T03:24:42.448990Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:42.449171Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:42.449298Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:42.449431Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:42.449570Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:42.449705Z  WARN viewport_authority::drift: VIEWPORT_DRIFT measured vs committed delta=Vec2(-1280.0, -720.0) measured=Vec2(0.0, 0.0) committed=Vec2(1280.0, 720.0) hint="check AuthoritativeViewport vs SimulationMapViewport copy-through"
2026-05-27T03:24:42.449856Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:42.466694Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:42.468446Z  INFO stall: STALL post_egui: 19.40ms
2026-05-27T03:24:42.488827Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:42.488942Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=2/6 footprint_ok=false readability=false icons=0
2026-05-27T03:24:42.489372Z  INFO stall: STALL after_readiness: 20.93ms
2026-05-27T03:24:42.496313Z  INFO stall: STALL last: 6.94ms
2026-05-27T03:24:42.496325Z  INFO worldgen_chrome::hud: HUD_SHELL_STATE (Editor/Simulation = HUD egui may run; MainMenu = player shell off) base=Res(State(Simulation)) minimap_visible=true overlay_tray=false command_shell=false transmission=false
2026-05-27T03:24:42.496325Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(InGame)) worldgen=Res(State(Dismissed)) base=Res(State(Simulation)) flow=Res(State(PreviewReady)) latch_dismissed=true world_gen_visible=false preview_window_open=false lifecycle=Uninitialized last_dismiss="never"
2026-05-27T03:24:42.496346Z  WARN visual_diag::anomaly: SIM_VIEWPORT_VALIDITY_CHANGED frame=43 was_valid=false now_valid=true
2026-05-27T03:24:42.496373Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=43 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=false measured_wh=Vec2(0.0, 0.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=1 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1280.0, 720.0) cam_hole=false render_hole=false cam_invalid_streak=0 cam_valid_streak=1 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=29 resolved_rev=78 app=2 base=2 flow=2 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:42.496812Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=43 was=0 now=2
2026-05-27T03:24:42.497264Z  WARN sim_view_sync::anomaly: SIM_MAP_VIEWPORT_VALIDITY_CHANGED frame=43 was_valid=false now_valid=true was_adequate=false now_adequate=true
2026-05-27T03:24:42.497363Z  INFO visual_diag: VISUAL_DIAG window frame=43 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=2 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:42.497659Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=43 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=false measured_wh=(0, 0) committed_wh=(1280, 720) sim_held=false settle_streak=1 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:42.497873Z  INFO visual_diag: VISUAL_DIAG camera frame=43 cam_desired_x=160.0 cam_desired_y=245.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=0 latch_valid_streak=1 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:42.498061Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=43 resolved_rev=78 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=false sim_resolved_wh=(0, 0) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:42.498250Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=43 world_preview_proj_rev=2551212574560 minimap_proj_rev=944892805407 sim_map_proj_rev=3092403454483
2026-05-27T03:24:42.498355Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=43 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:42.498509Z  INFO visual_diag: VISUAL_DIAG render_spine frame=43 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=44 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=2 gpu_draw=0
2026-05-27T03:24:42.498798Z  INFO visual_diag: VISUAL_DIAG perf frame=43 tile_raster_ms=213.73001098632813 tile_raster_ran=true world_repr_ms=0.26589998602867126 projection_graph_ms=0.00139999995008111 domain_merge_ms=0.00020000000949949026 readiness_ms=0.5704999566078186 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:42.498968Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=43 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:42.499092Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 249.0 ms)
2026-05-27T03:24:42.499210Z  INFO perf: PERF wall=1477.36 instr=214.57 gap=1262.78 | cpu_pre_egui=1427.18 cpu_egui=19.54 cpu_post_egui=30.63 gpu_gap=0.00 | spine=0.01 world_repr=0.27 graph=0.00 merge=0.00 atm=0.00 readiness=0.57 raster=213.73 | upd_attrib sum=638.90 pv_cpu=0.00 pv_gpu=0.01 fire=4.04 stream=634.84 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=216.54 hud=0.00 overlay=0.00 raster_b=213.73 particles=0.00 residency=2.71 tex_reg=0.00 render_x=0.10 | egui_unbudgeted=19.54 | stall first+preupd=0.06 update=0.00 post_dom=228.09 post_vt=0.11 post→ready=0.00 ready=20.93 post→egui=0.00 egui=19.40 post_egui=6.94 | stall_hits=[after_tile_storage_apply:1199.0,after_domain_merge:228.1,post_egui:19.4,after_readiness:20.9,last:6.9]
2026-05-27T03:24:42.499344Z  INFO perf: PERF frame=1477.4ms update=1427.2ms egui=19.5ms preview=0.0ms streaming=634.8ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=213.7ms
2026-05-27T03:24:42.499445Z  INFO stall: STALL culprit=after_tile_storage_apply duration=1199.0ms frame=1477.4ms
2026-05-27T03:24:42.506266Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:42.506403Z  INFO proc_A_dine01::render::viewport_pipeline: resolved viewport=simulation_map revision=79 logical=(1280.0,720.0) physical=1280x720
2026-05-27T03:24:42.507936Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:42.508038Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:42.508120Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:42.508197Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:42.677533Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,245.00) zoom=76.5559 world_main_xy=(160.00,245.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:42.678209Z  INFO economy::activation::ind_e03: IND-E03: spawned grid overload cluster for witness depth
2026-05-27T03:24:42.715591Z  WARN stage5_fire_view::live: STAGE5_FIRE_VIEW_CROSSCHECK view=WorldMain visible_chunks_not_in_active=1 active_total=0 visible_wm=1
2026-05-27T03:24:42.723602Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:42.724780Z  INFO world_representation::lod: WorldRepresentation: zoom=76.556 zoom_α=1.000 → LOD band Operational (OP)
2026-05-27T03:24:42.736626Z  INFO stall: STALL upd_fire_pipeline: 21.15ms
2026-05-27T03:24:43.364007Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=45 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=1 atm_full_field_fallback=false
2026-05-27T03:24:43.364222Z  INFO stall: STALL after_tile_storage_apply: 859.18ms
2026-05-27T03:24:43.365578Z  INFO stall: STALL upd_streaming_reconstruct: 630.16ms
2026-05-27T03:24:43.553947Z  INFO stall: STALL after_domain_merge: 189.73ms
2026-05-27T03:24:43.554068Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:43.554542Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:43.554672Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:43.554812Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:43.554973Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:43.555144Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=false min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:43.570602Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:43.571262Z  INFO stall: STALL post_egui: 17.20ms
2026-05-27T03:24:43.574080Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:43.574176Z  INFO stage5_live_todos: STAGE5_TODO_BOARD_RECONCILE marked_done=0 reopened=1 done_count=12/13 inv=45
2026-05-27T03:24:43.574265Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:43.580675Z  INFO stall: STALL last: 6.31ms
2026-05-27T03:24:43.580688Z  INFO worldgen_chrome::trace: CHROME_STATE app=Res(State(InGame)) worldgen=Res(State(Dismissed)) base=Res(State(Simulation)) flow=Res(State(FullReady)) latch_dismissed=true world_gen_visible=false preview_window_open=false lifecycle=Uninitialized last_dismiss="never"
2026-05-27T03:24:43.580709Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=44 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=2 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1280.0, 720.0) cam_hole=false render_hole=false cam_invalid_streak=0 cam_valid_streak=2 cam_scissor=None ortho_fixed_wh=(17, 9) map_view_px=(1280, 720) raster_rev=29 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:24:43.580720Z  WARN visual_diag::anomaly: RESOLVED_SIM_MAP_VALIDITY_CHANGED frame=44 was=false now=true
2026-05-27T03:24:43.581463Z  INFO visual_diag: VISUAL_DIAG window frame=44 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:43.581696Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=44 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=false settle_streak=2 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:43.581953Z  INFO visual_diag: VISUAL_DIAG camera frame=44 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=false render_hole=false latch_invalid_streak=0 latch_valid_streak=2 cam_scissor=None ortho_fixed_w=17 ortho_fixed_h=9 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:43.582154Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=44 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:43.582350Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=44 world_preview_proj_rev=2551212574560 minimap_proj_rev=944892805410 sim_map_proj_rev=3092405454489
2026-05-27T03:24:43.582461Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=44 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:43.582623Z  INFO visual_diag: VISUAL_DIAG render_spine frame=44 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.8500000238418579 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=45 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:43.582918Z  INFO visual_diag: VISUAL_DIAG perf frame=44 tile_raster_ms=184.77830505371094 tile_raster_ran=true world_repr_ms=0.20120000839233398 projection_graph_ms=0.0019000000320374966 domain_merge_ms=0.0003000000142492354 readiness_ms=0.3003999888896942 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:43.583091Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=44 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:43.583241Z  INFO perf: PERF wall=1078.27 instr=185.31 gap=892.96 | cpu_pre_egui=1048.99 cpu_egui=17.33 cpu_post_egui=11.95 gpu_gap=0.00 | spine=0.03 world_repr=0.20 graph=0.00 merge=0.00 atm=0.03 readiness=0.30 raster=184.78 | upd_attrib sum=651.35 pv_cpu=0.00 pv_gpu=0.02 fire=21.15 stream=630.16 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=187.55 hud=0.00 overlay=0.00 raster_b=184.78 particles=0.00 residency=2.72 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=17.33 | stall first+preupd=0.09 update=0.00 post_dom=189.73 post_vt=0.10 post→ready=0.01 ready=3.10 post→egui=0.01 egui=17.20 post_egui=6.31 | stall_hits=[after_tile_storage_apply:859.2,after_domain_merge:189.7,post_egui:17.2,last:6.3]
2026-05-27T03:24:43.583364Z  INFO perf: PERF frame=1078.3ms update=1049.0ms egui=17.3ms preview=0.0ms streaming=630.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=184.8ms
2026-05-27T03:24:43.583458Z  INFO stall: STALL culprit=after_tile_storage_apply duration=859.2ms frame=1078.3ms
2026-05-27T03:24:43.586352Z  INFO stage5_live_todos: STAGE5_ACTIVE_TODO subsystem=First(render_frame_start) spine_gate_seq=["TODO-01", "TODO-04", "TODO-06"] idx=10 id=TODO-11 status=InProgress file=src/render/fire_view_extract.rs system=VisibleFireChunkSet, build_fire_visual_frames_by_view goal=VisibleFireChunkSet derived from view projection only, matches sim snapshot. runtime_check=RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet. failure_mode=Ghost or missing fire chunks.
2026-05-27T03:24:43.590567Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:43.591891Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:43.591993Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:43.592088Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:43.592170Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:43.714160Z  INFO stage5_live_todos: STAGE5_ACTIVE_TODO subsystem=map_camera_mirror_chain spine_gate_seq=["TODO-01", "TODO-04", "TODO-06"] idx=10 id=TODO-11 status=InProgress file=src/render/fire_view_extract.rs system=VisibleFireChunkSet, build_fire_visual_frames_by_view goal=VisibleFireChunkSet derived from view projection only, matches sim snapshot. runtime_check=RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet. failure_mode=Ghost or missing fire chunks.
2026-05-27T03:24:43.714304Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=76.5559 world_main_xy=(160.00,160.00) zoom=76.5559 bridge_drift=0.0000
2026-05-27T03:24:43.715233Z  INFO economy::activation::ind_e03: IND-E03: grid overload witness depth green (overload_events_total=1)
2026-05-27T03:24:43.751915Z  INFO stage5_live_todos: STAGE5_ACTIVE_TODO subsystem=fire_extract_post spine_gate_seq=["TODO-01", "TODO-04", "TODO-06"] idx=10 id=TODO-11 status=InProgress file=src/render/fire_view_extract.rs system=VisibleFireChunkSet, build_fire_visual_frames_by_view goal=VisibleFireChunkSet derived from view projection only, matches sim snapshot. runtime_check=RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet. failure_mode=Ghost or missing fire chunks.
2026-05-27T03:24:43.752043Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:44.090818Z  INFO stage5_live_todos: STAGE5_ACTIVE_TODO subsystem=representation_spine_post_merge spine_gate_seq=["TODO-01", "TODO-04", "TODO-06"] idx=10 id=TODO-11 status=InProgress file=src/render/fire_view_extract.rs system=VisibleFireChunkSet, build_fire_visual_frames_by_view goal=VisibleFireChunkSet derived from view projection only, matches sim snapshot. runtime_check=RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet. failure_mode=Ghost or missing fire chunks.
2026-05-27T03:24:44.090966Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=46 world_frame_present=true overlay_rev=1 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:44.091560Z  INFO stall: STALL after_tile_storage_apply: 501.93ms
2026-05-27T03:24:44.091664Z  INFO stall: STALL upd_streaming_reconstruct: 338.13ms
2026-05-27T03:24:44.285705Z  INFO stall: STALL after_domain_merge: 194.14ms
2026-05-27T03:24:44.285758Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.286336Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:44.286469Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.286619Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.286767Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.286922Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraLatch valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.287044Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.304736Z  INFO stage5_live_todos: STAGE5_ACTIVE_TODO subsystem=gpu_indirect_sync_post spine_gate_seq=["TODO-01", "TODO-04", "TODO-06"] idx=10 id=TODO-11 status=InProgress file=src/render/fire_view_extract.rs system=VisibleFireChunkSet, build_fire_visual_frames_by_view goal=VisibleFireChunkSet derived from view projection only, matches sim snapshot. runtime_check=RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet. failure_mode=Ghost or missing fire chunks.
2026-05-27T03:24:44.304859Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:44.305684Z  INFO stall: STALL post_egui: 19.81ms
2026-05-27T03:24:44.306985Z  INFO stage5_live_todos: STAGE5_ACTIVE_TODO subsystem=evaluate_app_stage5_readiness spine_gate_seq=["TODO-01", "TODO-04", "TODO-06"] idx=10 id=TODO-11 status=InProgress file=src/render/fire_view_extract.rs system=VisibleFireChunkSet, build_fire_visual_frames_by_view goal=VisibleFireChunkSet derived from view projection only, matches sim snapshot. runtime_check=RUST_LOG=stage5_fire_view::live=warn: STAGE5_FIRE_VIEW_CROSSCHECK if visible chunks escape ActiveFireChunkSet. failure_mode=Ghost or missing fire chunks.
2026-05-27T03:24:44.307119Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:44.307214Z  INFO stage5_live_todos: STAGE5_TODO_BOARD_RECONCILE marked_done=1 reopened=0 done_count=13/13 inv=46
2026-05-27T03:24:44.307306Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:44.314063Z  INFO stall: STALL last: 6.65ms
2026-05-27T03:24:44.314089Z  WARN visual_diag::anomaly: RENDER_HOLE_FLIP frame=45 was=false now=true
2026-05-27T03:24:44.314091Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=45 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=3 layout_settled=false sim_held=false last_commit="hole_settling" frozen=false pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=3 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(25, 14) map_view_px=(1280, 720) raster_rev=29 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(52.102642, 52.102642, 52.102642) minimap_visible=true
2026-05-27T03:24:44.314246Z  WARN visual_diag::anomaly: CAMERA_SCISSOR_CHANGED frame=45 was=None now=Some((0, 0, 1280, 720))
2026-05-27T03:24:44.314700Z  WARN sim_view_sync::anomaly: CAMERA_VIEWPORT_MODE_FLIP (full-window vs map-hole scissor) frame=45 was_hole=false now_hole=true
2026-05-27T03:24:44.314802Z  INFO visual_diag: VISUAL_DIAG window frame=45 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:44.314908Z  WARN sim_view_sync::anomaly: CAMERA_SCISSOR_CHANGED frame=45 was=None now=Some((0, 0, 1280, 720))
2026-05-27T03:24:44.315079Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=45 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=false settle_streak=3 layout_settled=false frozen=false last_commit="hole_settling" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:44.315188Z  WARN sim_view_sync::anomaly: RENDER_MODE_FLIP (map-hole scissor vs full-window — primary blink source) frame=45 was_render_hole=false now_render_hole=true was_scissor=None now_scissor=Some((0, 0, 1280, 720))
2026-05-27T03:24:44.315405Z  INFO visual_diag: VISUAL_DIAG camera frame=45 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=3 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:44.315736Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=45 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:44.315932Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=45 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805413 sim_map_proj_rev=3092405454489
2026-05-27T03:24:44.316035Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=45 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:44.316190Z  INFO visual_diag: VISUAL_DIAG render_spine frame=45 raster_rev=29 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.8500000238418579 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=46 overlay_rev=1 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:44.316474Z  INFO visual_diag: VISUAL_DIAG perf frame=45 tile_raster_ms=189.44580078125 tile_raster_ran=true world_repr_ms=0.1981000006198883 projection_graph_ms=0.0021000001579523087 domain_merge_ms=0.00020000000949949026 readiness_ms=0.4316999912261963 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:44.316638Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=45 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:44.316783Z  INFO perf: PERF wall=730.42 instr=190.08 gap=540.34 | cpu_pre_egui=699.37 cpu_egui=19.97 cpu_post_egui=11.09 gpu_gap=0.00 | spine=0.01 world_repr=0.20 graph=0.00 merge=0.00 atm=0.01 readiness=0.43 raster=189.45 | upd_attrib sum=342.59 pv_cpu=0.00 pv_gpu=0.01 fire=4.44 stream=338.13 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=192.18 hud=0.00 overlay=0.00 raster_b=189.45 particles=0.00 residency=2.69 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.97 | stall first+preupd=3.29 update=0.00 post_dom=194.14 post_vt=0.13 post→ready=0.01 ready=1.75 post→egui=0.01 egui=19.81 post_egui=6.65 | stall_hits=[after_tile_storage_apply:501.9,after_domain_merge:194.1,post_egui:19.8,last:6.7]
2026-05-27T03:24:44.316903Z  INFO perf: PERF frame=730.4ms update=699.4ms egui=20.0ms preview=0.0ms streaming=338.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=189.4ms
2026-05-27T03:24:44.316995Z  INFO stall: STALL culprit=after_tile_storage_apply duration=501.9ms frame=730.4ms
2026-05-27T03:24:44.319995Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:44.320192Z  INFO test_harness::fire: test scene seeded shared overlay fire cells=28
2026-05-27T03:24:44.321056Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:44.321452Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:44.321544Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:44.321639Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:44.321729Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:44.446464Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:44.482973Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:44.809729Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=47 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:44.809938Z  INFO stall: STALL after_tile_storage_apply: 490.99ms
2026-05-27T03:24:44.810571Z  INFO stall: STALL upd_streaming_reconstruct: 327.19ms
2026-05-27T03:24:45.030320Z  INFO stall: STALL after_domain_merge: 220.38ms
2026-05-27T03:24:45.030349Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.030961Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:45.031096Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.031235Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.031378Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.031544Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.048814Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:45.049478Z  INFO stall: STALL post_egui: 19.01ms
2026-05-27T03:24:45.052790Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:45.052886Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:45.059932Z  INFO stall: STALL last: 6.95ms
2026-05-27T03:24:45.059963Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=46 was=2 now=3
2026-05-27T03:24:45.059958Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=46 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_settled" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=4 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(25, 14) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(52.102642, 52.102642, 52.102642) minimap_visible=true
2026-05-27T03:24:45.060144Z  INFO visual_diag: VISUAL_DIAG window frame=46 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:45.060775Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=46 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_settled" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:45.061054Z  INFO visual_diag: VISUAL_DIAG camera frame=46 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=4 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:45.061288Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=46 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:45.061484Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=46 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805414 sim_map_proj_rev=3092405454489
2026-05-27T03:24:45.061593Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=46 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:45.061753Z  INFO visual_diag: VISUAL_DIAG render_spine frame=46 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=47 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:45.062048Z  INFO visual_diag: VISUAL_DIAG perf frame=46 tile_raster_ms=215.66578674316406 tile_raster_ran=true world_repr_ms=0.19849999248981476 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00010000000474974513 readiness_ms=0.20310001075267792 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:45.062215Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=46 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:45.062358Z  INFO perf: PERF wall=743.49 instr=216.07 gap=527.42 | cpu_pre_egui=711.45 cpu_egui=19.18 cpu_post_egui=12.86 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.20 raster=215.67 | upd_attrib sum=330.59 pv_cpu=0.00 pv_gpu=0.01 fire=3.38 stream=327.19 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=218.58 hud=0.00 overlay=0.00 raster_b=215.67 particles=0.00 residency=2.87 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.18 | stall first+preupd=0.10 update=0.00 post_dom=220.38 post_vt=0.14 post→ready=0.00 ready=3.50 post→egui=0.00 egui=19.01 post_egui=6.95 | stall_hits=[after_tile_storage_apply:491.0,after_domain_merge:220.4,post_egui:19.0,last:7.0]
2026-05-27T03:24:45.062476Z  INFO perf: PERF frame=743.5ms update=711.5ms egui=19.2ms preview=0.0ms streaming=327.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=215.7ms
2026-05-27T03:24:45.062564Z  INFO stall: STALL culprit=after_tile_storage_apply duration=491.0ms frame=743.5ms
2026-05-27T03:24:45.065580Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.066496Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:45.066819Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:45.069244Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:45.069349Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:45.069442Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:45.189200Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:45.225903Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:45.560448Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=48 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:45.560699Z  INFO stall: STALL after_tile_storage_apply: 496.25ms
2026-05-27T03:24:45.561144Z  INFO stall: STALL upd_streaming_reconstruct: 334.86ms
2026-05-27T03:24:45.755898Z  INFO stall: STALL after_domain_merge: 195.19ms
2026-05-27T03:24:45.755914Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.756803Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:45.756941Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.757073Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.757216Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.757374Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.774168Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:45.774757Z  INFO stall: STALL post_egui: 18.73ms
2026-05-27T03:24:45.776242Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:45.776354Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:45.782628Z  INFO stall: STALL last: 6.17ms
2026-05-27T03:24:45.782655Z  WARN visual_diag::anomaly: SIM_COMMIT_BRANCH_CHANGED frame=47 was=3 now=4
2026-05-27T03:24:45.782652Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=47 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=5 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(25, 14) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(52.102642, 52.102642, 52.102642) minimap_visible=true
2026-05-27T03:24:45.782816Z  INFO visual_diag: VISUAL_DIAG window frame=47 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:45.783367Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=47 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:45.783582Z  INFO visual_diag: VISUAL_DIAG camera frame=47 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=5 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:45.783777Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=47 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:45.783969Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=47 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:45.784073Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=47 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:45.784227Z  INFO visual_diag: VISUAL_DIAG render_spine frame=47 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=48 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:45.784515Z  INFO visual_diag: VISUAL_DIAG perf frame=47 tile_raster_ms=190.67210388183594 tile_raster_ran=true world_repr_ms=0.2019999921321869 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00020000000949949026 readiness_ms=0.23499999940395355 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:45.784682Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=47 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:45.784841Z  INFO perf: PERF wall=720.43 instr=191.11 gap=529.31 | cpu_pre_egui=691.52 cpu_egui=18.86 cpu_post_egui=10.04 gpu_gap=0.00 | spine=0.01 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=190.67 | upd_attrib sum=338.06 pv_cpu=0.00 pv_gpu=0.02 fire=3.17 stream=334.86 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=193.39 hud=0.00 overlay=0.00 raster_b=190.67 particles=0.00 residency=2.67 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.86 | stall first+preupd=0.07 update=0.00 post_dom=195.19 post_vt=0.13 post→ready=0.00 ready=1.70 post→egui=0.00 egui=18.73 post_egui=6.17 | stall_hits=[after_tile_storage_apply:496.3,after_domain_merge:195.2,post_egui:18.7,last:6.2]
2026-05-27T03:24:45.784964Z  INFO perf: PERF frame=720.4ms update=691.5ms egui=18.9ms preview=0.0ms streaming=334.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=190.7ms
2026-05-27T03:24:45.785053Z  INFO stall: STALL culprit=after_tile_storage_apply duration=496.3ms frame=720.4ms
2026-05-27T03:24:45.787530Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:45.788337Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:45.788854Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:45.788949Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:45.789041Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:45.789140Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:45.910131Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:45.947379Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:46.277535Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=49 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:46.277729Z  INFO stall: STALL after_tile_storage_apply: 491.12ms
2026-05-27T03:24:46.278180Z  INFO stall: STALL upd_streaming_reconstruct: 330.36ms
2026-05-27T03:24:46.470731Z  INFO stall: STALL after_domain_merge: 193.00ms
2026-05-27T03:24:46.470755Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:46.471317Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:46.471431Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:46.471552Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:46.471672Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:46.471822Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:46.471964Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:46.489797Z  INFO stall: STALL post_egui: 18.92ms
2026-05-27T03:24:46.491306Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:46.491764Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:46.498708Z  INFO stall: STALL last: 6.85ms
2026-05-27T03:24:46.498735Z  INFO visual_diag: VISUAL_DIAG window frame=48 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:46.499042Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=48 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:46.499336Z  INFO visual_diag: VISUAL_DIAG camera frame=48 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=6 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:46.499603Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=48 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:46.499853Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=48 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:46.499973Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=48 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:46.500152Z  INFO visual_diag: VISUAL_DIAG render_spine frame=48 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=49 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:46.500494Z  INFO visual_diag: VISUAL_DIAG perf frame=48 tile_raster_ms=188.1591033935547 tile_raster_ran=true world_repr_ms=0.2443999946117401 projection_graph_ms=0.00139999995008111 domain_merge_ms=0.00010000000474974513 readiness_ms=0.572700023651123 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:46.500691Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=48 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:46.500844Z  INFO perf: PERF wall=714.29 instr=188.98 gap=525.31 | cpu_pre_egui=684.18 cpu_egui=19.10 cpu_post_egui=11.01 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.57 raster=188.16 | upd_attrib sum=333.61 pv_cpu=0.00 pv_gpu=0.01 fire=3.23 stream=330.36 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=190.88 hud=0.00 overlay=0.00 raster_b=188.16 particles=0.00 residency=2.67 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.10 | stall first+preupd=0.07 update=0.00 post_dom=193.00 post_vt=0.13 post→ready=0.01 ready=2.07 post→egui=0.01 egui=18.92 post_egui=6.85 | stall_hits=[after_tile_storage_apply:491.1,after_domain_merge:193.0,post_egui:18.9,last:6.8]
2026-05-27T03:24:46.500961Z  INFO perf: PERF frame=714.3ms update=684.2ms egui=19.1ms preview=0.0ms streaming=330.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=188.2ms
2026-05-27T03:24:46.501050Z  INFO stall: STALL culprit=after_tile_storage_apply duration=491.1ms frame=714.3ms
2026-05-27T03:24:46.503557Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:46.504441Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:46.504968Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:46.505066Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:46.505157Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:46.505243Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:46.627001Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:46.664034Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:47.002977Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=50 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:47.003336Z  INFO stall: STALL after_tile_storage_apply: 500.64ms
2026-05-27T03:24:47.004785Z  INFO stall: STALL upd_streaming_reconstruct: 340.31ms
2026-05-27T03:24:47.200059Z  INFO stall: STALL after_domain_merge: 196.72ms
2026-05-27T03:24:47.200084Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.200674Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:47.200788Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.200911Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.201030Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.201180Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.201326Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:47.218364Z  INFO stall: STALL post_egui: 18.17ms
2026-05-27T03:24:47.219859Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:47.219968Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:47.226345Z  INFO stall: STALL last: 6.28ms
2026-05-27T03:24:47.226368Z  INFO visual_diag: VISUAL_DIAG window frame=49 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:47.227221Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=49 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:47.227498Z  INFO visual_diag: VISUAL_DIAG camera frame=49 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=7 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:47.227729Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=49 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:47.227956Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=49 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:47.228076Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=49 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:47.228257Z  INFO visual_diag: VISUAL_DIAG render_spine frame=49 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=50 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:47.228599Z  INFO visual_diag: VISUAL_DIAG perf frame=49 tile_raster_ms=192.1558074951172 tile_raster_ran=true world_repr_ms=0.24120000004768372 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00020000000949949026 readiness_ms=0.23109999299049377 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:47.228794Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=49 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:47.228961Z  INFO perf: PERF wall=726.33 instr=192.63 gap=533.70 | cpu_pre_egui=697.45 cpu_egui=18.34 cpu_post_egui=10.54 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=192.16 | upd_attrib sum=343.59 pv_cpu=0.00 pv_gpu=0.02 fire=3.25 stream=340.31 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=194.89 hud=0.00 overlay=0.00 raster_b=192.16 particles=0.00 residency=2.69 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.34 | stall first+preupd=0.10 update=0.00 post_dom=196.72 post_vt=0.14 post→ready=0.00 ready=1.71 post→egui=0.00 egui=18.17 post_egui=6.28 | stall_hits=[after_tile_storage_apply:500.6,after_domain_merge:196.7,post_egui:18.2,last:6.3]
2026-05-27T03:24:47.229102Z  INFO perf: PERF frame=726.3ms update=697.4ms egui=18.3ms preview=0.0ms streaming=340.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=192.2ms
2026-05-27T03:24:47.229204Z  INFO stall: STALL culprit=after_tile_storage_apply duration=500.6ms frame=726.3ms
2026-05-27T03:24:47.232024Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.232863Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:47.233485Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:47.233582Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:47.233669Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:47.233754Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:47.358442Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:47.395954Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:47.727674Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=51 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=true
2026-05-27T03:24:47.727856Z  INFO stall: STALL after_tile_storage_apply: 496.83ms
2026-05-27T03:24:47.728248Z  INFO stall: STALL upd_streaming_reconstruct: 331.92ms
2026-05-27T03:24:47.919128Z  INFO stall: STALL after_domain_merge: 191.27ms
2026-05-27T03:24:47.919156Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.919636Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:47.919758Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.919878Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.920000Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.920143Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.920281Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:47.937458Z  INFO stall: STALL post_egui: 18.20ms
2026-05-27T03:24:47.939814Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:47.939957Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:47.946530Z  INFO stall: STALL last: 6.46ms
2026-05-27T03:24:47.946562Z  INFO visual_diag: VISUAL_DIAG window frame=50 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:47.946784Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=50 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:47.947020Z  INFO visual_diag: VISUAL_DIAG camera frame=50 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=8 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:47.947225Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=50 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:47.947424Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=50 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:47.947538Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=50 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:47.947705Z  INFO visual_diag: VISUAL_DIAG render_spine frame=50 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=51 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:47.948021Z  INFO visual_diag: VISUAL_DIAG perf frame=50 tile_raster_ms=186.66310119628906 tile_raster_ran=true world_repr_ms=0.19920000433921814 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2987000048160553 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:47.948199Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=50 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:47.948363Z  INFO perf: PERF wall=717.37 instr=187.16 gap=530.20 | cpu_pre_egui=688.16 cpu_egui=18.35 cpu_post_egui=10.86 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.30 raster=186.66 | upd_attrib sum=335.09 pv_cpu=0.00 pv_gpu=0.02 fire=3.15 stream=331.92 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=189.38 hud=0.00 overlay=0.00 raster_b=186.66 particles=0.00 residency=2.67 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.35 | stall first+preupd=0.07 update=0.00 post_dom=191.27 post_vt=0.12 post→ready=0.01 ready=2.61 post→egui=0.01 egui=18.20 post_egui=6.46 | stall_hits=[after_tile_storage_apply:496.8,after_domain_merge:191.3,post_egui:18.2,last:6.5]
2026-05-27T03:24:47.948490Z  INFO perf: PERF frame=717.4ms update=688.2ms egui=18.4ms preview=0.0ms streaming=331.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=186.7ms
2026-05-27T03:24:47.948587Z  INFO stall: STALL culprit=after_tile_storage_apply duration=496.8ms frame=717.4ms
2026-05-27T03:24:47.952487Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:47.953469Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:47.954008Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:47.956791Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:47.956878Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:47.956997Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:48.079121Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:48.116450Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:48.454125Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=52 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:48.454299Z  INFO stall: STALL after_tile_storage_apply: 503.07ms
2026-05-27T03:24:48.454708Z  INFO stall: STALL upd_streaming_reconstruct: 337.82ms
2026-05-27T03:24:48.662322Z  INFO stall: STALL after_domain_merge: 208.02ms
2026-05-27T03:24:48.662351Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:48.663130Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:48.663242Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:48.663359Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:48.663480Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:48.663628Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:48.663781Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:48.680940Z  INFO stall: STALL post_egui: 18.47ms
2026-05-27T03:24:48.682261Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:48.682358Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:48.688596Z  INFO stall: STALL last: 6.14ms
2026-05-27T03:24:48.688615Z  INFO visual_diag: VISUAL_DIAG window frame=51 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:48.688856Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=51 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:48.689085Z  INFO visual_diag: VISUAL_DIAG camera frame=51 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=9 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:48.689290Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=51 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:48.689488Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=51 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:48.689599Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=51 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:48.689762Z  INFO visual_diag: VISUAL_DIAG render_spine frame=51 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=52 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:48.690060Z  INFO visual_diag: VISUAL_DIAG perf frame=51 tile_raster_ms=203.45339965820313 tile_raster_ran=true world_repr_ms=0.2441999912261963 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00020000000949949026 readiness_ms=0.20890000462532043 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:48.690239Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=51 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:48.690358Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 249.7 ms)
2026-05-27T03:24:48.690471Z  INFO perf: PERF wall=739.22 instr=203.91 gap=535.31 | cpu_pre_egui=711.17 cpu_egui=18.64 cpu_post_egui=9.41 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.21 raster=203.45 | upd_attrib sum=341.07 pv_cpu=0.00 pv_gpu=0.01 fire=3.23 stream=337.82 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=206.15 hud=0.00 overlay=0.00 raster_b=203.45 particles=0.00 residency=2.65 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.64 | stall first+preupd=0.09 update=0.00 post_dom=208.02 post_vt=0.15 post→ready=0.00 ready=1.51 post→egui=0.00 egui=18.47 post_egui=6.14 | stall_hits=[after_tile_storage_apply:503.1,after_domain_merge:208.0,post_egui:18.5,last:6.1]
2026-05-27T03:24:48.690591Z  INFO perf: PERF frame=739.2ms update=711.2ms egui=18.6ms preview=0.0ms streaming=337.8ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=203.5ms
2026-05-27T03:24:48.690689Z  INFO stall: STALL culprit=after_tile_storage_apply duration=503.1ms frame=739.2ms
2026-05-27T03:24:48.693551Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:48.694357Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:48.694933Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:48.695042Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:48.695121Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:48.695199Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:48.819304Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:48.856810Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:49.188028Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=53 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:49.188201Z  INFO stall: STALL after_tile_storage_apply: 495.64ms
2026-05-27T03:24:49.188623Z  INFO stall: STALL upd_streaming_reconstruct: 331.37ms
2026-05-27T03:24:49.383244Z  INFO stall: STALL after_domain_merge: 195.04ms
2026-05-27T03:24:49.383268Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:49.383846Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:49.383968Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:49.384088Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:49.384230Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:49.384396Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:49.384542Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:49.401431Z  INFO stall: STALL post_egui: 18.05ms
2026-05-27T03:24:49.402873Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:49.402981Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:49.409384Z  INFO stall: STALL last: 6.31ms
2026-05-27T03:24:49.409409Z  INFO visual_diag: VISUAL_DIAG window frame=52 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:49.409679Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=52 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:49.409931Z  INFO visual_diag: VISUAL_DIAG camera frame=52 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=10 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:49.410161Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=52 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:49.410358Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=52 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:49.410468Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=52 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:49.410627Z  INFO visual_diag: VISUAL_DIAG render_spine frame=52 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=53 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:49.410917Z  INFO visual_diag: VISUAL_DIAG perf frame=52 tile_raster_ms=190.50209045410156 tile_raster_ran=true world_repr_ms=0.2465999871492386 projection_graph_ms=0.0020000000949949026 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2134999930858612 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:49.411090Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=52 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:49.411249Z  INFO perf: PERF wall=718.74 instr=190.97 gap=527.77 | cpu_pre_egui=690.76 cpu_egui=18.22 cpu_post_egui=9.76 gpu_gap=0.00 | spine=0.01 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.21 raster=190.50 | upd_attrib sum=334.86 pv_cpu=0.00 pv_gpu=0.02 fire=3.46 stream=331.37 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=193.42 hud=0.00 overlay=0.00 raster_b=190.50 particles=0.00 residency=2.86 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.22 | stall first+preupd=0.09 update=0.00 post_dom=195.04 post_vt=0.13 post→ready=0.00 ready=1.64 post→egui=0.01 egui=18.05 post_egui=6.31 | stall_hits=[after_tile_storage_apply:495.6,after_domain_merge:195.0,post_egui:18.0,last:6.3]
2026-05-27T03:24:49.411371Z  INFO perf: PERF frame=718.7ms update=690.8ms egui=18.2ms preview=0.0ms streaming=331.4ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=190.5ms
2026-05-27T03:24:49.411464Z  INFO stall: STALL culprit=after_tile_storage_apply duration=495.6ms frame=718.7ms
2026-05-27T03:24:49.414447Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:49.415084Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:49.415804Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:49.415912Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:49.416000Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:49.416077Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:49.537056Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:49.573272Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:49.903632Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=54 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:49.903839Z  INFO stall: STALL after_tile_storage_apply: 490.63ms
2026-05-27T03:24:49.904475Z  INFO stall: STALL upd_streaming_reconstruct: 330.74ms
2026-05-27T03:24:50.100204Z  INFO stall: STALL after_domain_merge: 196.36ms
2026-05-27T03:24:50.100236Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.100849Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:50.100991Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.101108Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.101225Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.101370Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.101528Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:50.118738Z  INFO stall: STALL post_egui: 18.37ms
2026-05-27T03:24:50.120033Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:50.120148Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:50.126420Z  INFO stall: STALL last: 6.17ms
2026-05-27T03:24:50.126442Z  INFO visual_diag: VISUAL_DIAG window frame=53 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:50.126671Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=53 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:50.126886Z  INFO visual_diag: VISUAL_DIAG camera frame=53 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=11 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:50.127085Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=53 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:50.127286Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=53 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:50.127392Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=53 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:50.127548Z  INFO visual_diag: VISUAL_DIAG render_spine frame=53 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=54 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:50.127837Z  INFO visual_diag: VISUAL_DIAG perf frame=53 tile_raster_ms=191.5980987548828 tile_raster_ran=true world_repr_ms=0.24449999630451202 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00020000000949949026 readiness_ms=0.233800008893013 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:50.128007Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=53 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:50.128168Z  INFO perf: PERF wall=714.99 instr=192.08 gap=522.91 | cpu_pre_egui=687.05 cpu_egui=18.55 cpu_post_egui=9.39 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=191.60 | upd_attrib sum=334.43 pv_cpu=0.00 pv_gpu=0.02 fire=3.67 stream=330.74 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=194.74 hud=0.00 overlay=0.00 raster_b=191.60 particles=0.00 residency=3.09 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.55 | stall first+preupd=0.06 update=0.00 post_dom=196.36 post_vt=0.16 post→ready=0.00 ready=1.51 post→egui=0.00 egui=18.37 post_egui=6.17 | stall_hits=[after_tile_storage_apply:490.6,after_domain_merge:196.4,post_egui:18.4,last:6.2]
2026-05-27T03:24:50.128290Z  INFO perf: PERF frame=715.0ms update=687.0ms egui=18.6ms preview=0.0ms streaming=330.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=191.6ms
2026-05-27T03:24:50.128384Z  INFO stall: STALL culprit=after_tile_storage_apply duration=490.6ms frame=715.0ms
2026-05-27T03:24:50.130995Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.132129Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:50.132133Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:50.135329Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:50.135409Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:50.135492Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:50.257922Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:50.294915Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:50.637979Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=55 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:50.638222Z  INFO stall: STALL after_tile_storage_apply: 507.92ms
2026-05-27T03:24:50.638676Z  INFO stall: STALL upd_streaming_reconstruct: 343.35ms
2026-05-27T03:24:50.833211Z  INFO stall: STALL after_domain_merge: 194.99ms
2026-05-27T03:24:50.833240Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.834014Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:50.834121Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.834233Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.834346Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.834487Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.834630Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:50.852107Z  INFO stall: STALL post_egui: 18.77ms
2026-05-27T03:24:50.853336Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:50.853443Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:50.860218Z  INFO stall: STALL last: 6.68ms
2026-05-27T03:24:50.860247Z  INFO visual_diag: VISUAL_DIAG window frame=54 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:50.860547Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=54 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:50.860846Z  INFO visual_diag: VISUAL_DIAG camera frame=54 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=12 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:50.861100Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=54 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:50.861322Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=54 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:50.861439Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=54 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:50.861601Z  INFO visual_diag: VISUAL_DIAG render_spine frame=54 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=55 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:50.861894Z  INFO visual_diag: VISUAL_DIAG perf frame=54 tile_raster_ms=190.3278045654297 tile_raster_ran=true world_repr_ms=0.19869999587535858 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00010000000474974513 readiness_ms=0.2176000028848648 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:50.862069Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=54 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:50.862216Z  INFO perf: PERF wall=732.03 instr=190.75 gap=541.28 | cpu_pre_egui=703.04 cpu_egui=18.91 cpu_post_egui=10.07 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=190.33 | upd_attrib sum=346.78 pv_cpu=0.00 pv_gpu=0.02 fire=3.41 stream=343.35 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=193.22 hud=0.00 overlay=0.00 raster_b=190.33 particles=0.00 residency=2.85 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.91 | stall first+preupd=0.15 update=0.00 post_dom=194.99 post_vt=0.12 post→ready=0.01 ready=1.43 post→egui=0.01 egui=18.77 post_egui=6.68 | stall_hits=[after_tile_storage_apply:507.9,after_domain_merge:195.0,post_egui:18.8,last:6.7]
2026-05-27T03:24:50.862336Z  INFO perf: PERF frame=732.0ms update=703.0ms egui=18.9ms preview=0.0ms streaming=343.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=190.3ms
2026-05-27T03:24:50.862426Z  INFO stall: STALL culprit=after_tile_storage_apply duration=507.9ms frame=732.0ms
2026-05-27T03:24:50.865537Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:50.866418Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:50.867078Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:50.867211Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:50.867291Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:50.867369Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:50.989642Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:51.027711Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:51.360537Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=56 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:51.360705Z  INFO stall: STALL after_tile_storage_apply: 495.92ms
2026-05-27T03:24:51.361101Z  INFO stall: STALL upd_streaming_reconstruct: 332.92ms
2026-05-27T03:24:51.559490Z  INFO stall: STALL after_domain_merge: 198.78ms
2026-05-27T03:24:51.559517Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:51.560082Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:51.560196Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:51.560320Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:51.560441Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:51.560582Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:51.577449Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:51.578014Z  INFO stall: STALL post_egui: 18.40ms
2026-05-27T03:24:51.579243Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:51.579340Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:51.586026Z  INFO stall: STALL last: 6.58ms
2026-05-27T03:24:51.586047Z  INFO visual_diag: VISUAL_DIAG window frame=55 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:51.586298Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=55 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:51.586523Z  INFO visual_diag: VISUAL_DIAG camera frame=55 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=13 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:51.586727Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=55 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:51.586924Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=55 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:51.587038Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=55 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:51.587214Z  INFO visual_diag: VISUAL_DIAG render_spine frame=55 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=56 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:51.587525Z  INFO visual_diag: VISUAL_DIAG perf frame=55 tile_raster_ms=194.3083038330078 tile_raster_ran=true world_repr_ms=0.25829997658729553 projection_graph_ms=0.0017999999690800905 domain_merge_ms=0.00020000000949949026 readiness_ms=0.21480000019073486 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:51.589813Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=55 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:51.589956Z  INFO perf: PERF wall=725.20 instr=194.79 gap=530.41 | cpu_pre_egui=694.73 cpu_egui=18.55 cpu_post_egui=11.91 gpu_gap=0.00 | spine=0.00 world_repr=0.26 graph=0.00 merge=0.00 atm=0.00 readiness=0.21 raster=194.31 | upd_attrib sum=336.40 pv_cpu=0.00 pv_gpu=0.02 fire=3.45 stream=332.92 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=197.21 hud=0.00 overlay=0.00 raster_b=194.31 particles=0.00 residency=2.85 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.55 | stall first+preupd=0.05 update=0.00 post_dom=198.78 post_vt=0.12 post→ready=0.00 ready=1.43 post→egui=0.00 egui=18.40 post_egui=6.58 | stall_hits=[after_tile_storage_apply:495.9,after_domain_merge:198.8,post_egui:18.4,last:6.6]
2026-05-27T03:24:51.590080Z  INFO perf: PERF frame=725.2ms update=694.7ms egui=18.6ms preview=0.0ms streaming=332.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=194.3ms
2026-05-27T03:24:51.590169Z  INFO stall: STALL culprit=after_tile_storage_apply duration=495.9ms frame=725.2ms
2026-05-27T03:24:51.592632Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:51.593374Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:51.593892Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:51.593989Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:51.594078Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:51.594157Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:51.720636Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:51.757892Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:52.100549Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=57 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:52.100740Z  INFO stall: STALL after_tile_storage_apply: 508.89ms
2026-05-27T03:24:52.101172Z  INFO stall: STALL upd_streaming_reconstruct: 342.86ms
2026-05-27T03:24:52.295848Z  INFO stall: STALL after_domain_merge: 195.11ms
2026-05-27T03:24:52.295893Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:52.296330Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:52.296436Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:52.296553Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:52.296677Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:52.296821Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:52.296960Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:52.314505Z  INFO stall: STALL post_egui: 18.54ms
2026-05-27T03:24:52.315918Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:52.316024Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:52.322488Z  INFO stall: STALL last: 6.37ms
2026-05-27T03:24:52.322515Z  INFO visual_diag: VISUAL_DIAG window frame=56 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:52.322785Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=56 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:52.323046Z  INFO visual_diag: VISUAL_DIAG camera frame=56 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=14 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:52.323279Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=56 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:52.323502Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=56 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:52.323633Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=56 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:52.323817Z  INFO visual_diag: VISUAL_DIAG render_spine frame=56 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=57 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:52.324146Z  INFO visual_diag: VISUAL_DIAG perf frame=56 tile_raster_ms=190.13619995117188 tile_raster_ran=true world_repr_ms=0.20009998977184296 projection_graph_ms=0.0019000000320374966 domain_merge_ms=0.00020000000949949026 readiness_ms=0.22260001301765442 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:52.324316Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=56 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:52.324459Z  INFO perf: PERF wall=732.68 instr=190.56 gap=542.11 | cpu_pre_egui=704.09 cpu_egui=18.68 cpu_post_egui=9.92 gpu_gap=0.00 | spine=0.01 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=190.14 | upd_attrib sum=346.11 pv_cpu=0.00 pv_gpu=0.02 fire=3.22 stream=342.86 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=192.86 hud=0.00 overlay=0.00 raster_b=190.14 particles=0.00 residency=2.68 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.68 | stall first+preupd=0.09 update=0.00 post_dom=195.11 post_vt=0.11 post→ready=0.00 ready=1.62 post→egui=0.00 egui=18.54 post_egui=6.37 | stall_hits=[after_tile_storage_apply:508.9,after_domain_merge:195.1,post_egui:18.5,last:6.4]
2026-05-27T03:24:52.324577Z  INFO perf: PERF frame=732.7ms update=704.1ms egui=18.7ms preview=0.0ms streaming=342.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=190.1ms
2026-05-27T03:24:52.324671Z  INFO stall: STALL culprit=after_tile_storage_apply duration=508.9ms frame=732.7ms
2026-05-27T03:24:52.327472Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:52.328332Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:52.328785Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:52.328882Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:52.328968Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:52.329046Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:52.450049Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:52.487028Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:52.822986Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=58 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:52.823203Z  INFO stall: STALL after_tile_storage_apply: 496.71ms
2026-05-27T03:24:52.824048Z  INFO stall: STALL upd_streaming_reconstruct: 336.58ms
2026-05-27T03:24:53.019456Z  INFO stall: STALL after_domain_merge: 196.25ms
2026-05-27T03:24:53.019474Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.020093Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:53.020224Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.020353Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.020474Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.020619Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.020765Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:53.038813Z  INFO stall: STALL post_egui: 19.20ms
2026-05-27T03:24:53.040123Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:53.040683Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:53.047102Z  INFO stall: STALL last: 6.31ms
2026-05-27T03:24:53.047130Z  INFO visual_diag: VISUAL_DIAG window frame=57 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:53.047362Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=57 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:53.047580Z  INFO visual_diag: VISUAL_DIAG camera frame=57 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=15 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:53.047777Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=57 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:53.047969Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=57 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:53.048077Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=57 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:53.048238Z  INFO visual_diag: VISUAL_DIAG render_spine frame=57 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=58 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:53.048530Z  INFO visual_diag: VISUAL_DIAG perf frame=57 tile_raster_ms=191.62879943847656 tile_raster_ran=true world_repr_ms=0.24320000410079956 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00010000000474974513 readiness_ms=0.6811000108718872 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:53.048701Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=57 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:53.048846Z  INFO perf: PERF wall=722.39 instr=192.56 gap=529.84 | cpu_pre_egui=693.02 cpu_egui=19.38 cpu_post_egui=10.00 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.68 raster=191.63 | upd_attrib sum=340.14 pv_cpu=0.00 pv_gpu=0.01 fire=3.54 stream=336.58 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=194.67 hud=0.00 overlay=0.00 raster_b=191.63 particles=0.00 residency=2.99 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.38 | stall first+preupd=0.06 update=0.00 post_dom=196.25 post_vt=0.16 post→ready=0.00 ready=1.98 post→egui=0.01 egui=19.20 post_egui=6.31 | stall_hits=[after_tile_storage_apply:496.7,after_domain_merge:196.2,post_egui:19.2,last:6.3]
2026-05-27T03:24:53.048982Z  INFO perf: PERF frame=722.4ms update=693.0ms egui=19.4ms preview=0.0ms streaming=336.6ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=191.6ms
2026-05-27T03:24:53.049087Z  INFO stall: STALL culprit=after_tile_storage_apply duration=496.7ms frame=722.4ms
2026-05-27T03:24:53.051805Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.052791Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:53.053182Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:53.053272Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:53.053362Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:53.053449Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:53.175390Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:53.212595Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:53.545211Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=59 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:53.545381Z  INFO stall: STALL after_tile_storage_apply: 494.44ms
2026-05-27T03:24:53.545888Z  INFO stall: STALL upd_streaming_reconstruct: 332.87ms
2026-05-27T03:24:53.740967Z  INFO stall: STALL after_domain_merge: 195.58ms
2026-05-27T03:24:53.740998Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.741803Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:53.741930Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.742065Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.742188Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.742336Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.742481Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:53.759657Z  INFO stall: STALL post_egui: 18.57ms
2026-05-27T03:24:53.761082Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:53.762099Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:53.768763Z  INFO stall: STALL last: 6.57ms
2026-05-27T03:24:53.768787Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=58 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=16 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(14, 8) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(90.0, 90.0, 90.0) minimap_visible=true
2026-05-27T03:24:53.768796Z  INFO visual_diag: VISUAL_DIAG window frame=58 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:53.769463Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=58 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:53.769720Z  INFO visual_diag: VISUAL_DIAG camera frame=58 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=90.0 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=16 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=14 ortho_fixed_h=8 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:53.769946Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=58 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:53.770173Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=58 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:53.770284Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=58 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:53.770443Z  INFO visual_diag: VISUAL_DIAG render_spine frame=58 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=1.0 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=59 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:53.770730Z  INFO visual_diag: VISUAL_DIAG perf frame=58 tile_raster_ms=190.68899536132813 tile_raster_ran=true world_repr_ms=0.24040000140666962 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00010000000474974513 readiness_ms=1.1344000101089478 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:53.770901Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=58 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:53.771049Z  INFO perf: PERF wall=720.15 instr=192.07 gap=528.08 | cpu_pre_egui=690.08 cpu_egui=18.71 cpu_post_egui=11.36 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=1.13 raster=190.69 | upd_attrib sum=336.51 pv_cpu=0.00 pv_gpu=0.02 fire=3.61 stream=332.87 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=193.80 hud=0.00 overlay=0.00 raster_b=190.69 particles=0.00 residency=3.06 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.71 | stall first+preupd=0.07 update=0.00 post_dom=195.58 post_vt=0.12 post→ready=0.00 ready=2.54 post→egui=0.01 egui=18.57 post_egui=6.57 | stall_hits=[after_tile_storage_apply:494.4,after_domain_merge:195.6,post_egui:18.6,last:6.6]
2026-05-27T03:24:53.771174Z  INFO perf: PERF frame=720.2ms update=690.1ms egui=18.7ms preview=0.0ms streaming=332.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=190.7ms
2026-05-27T03:24:53.771262Z  INFO stall: STALL culprit=after_tile_storage_apply duration=494.4ms frame=720.2ms
2026-05-27T03:24:53.774786Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:53.775650Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:53.776069Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:53.776180Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:53.776264Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:53.776343Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:53.898447Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=90.0000 world_main_xy=(160.00,160.00) zoom=90.0000 bridge_drift=0.0000
2026-05-27T03:24:53.938207Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:53.959662Z  INFO stall: STALL upd_fire_pipeline: 21.57ms
2026-05-27T03:24:54.300958Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=60 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:54.301167Z  INFO stall: STALL after_tile_storage_apply: 527.43ms
2026-05-27T03:24:54.301595Z  INFO stall: STALL upd_streaming_reconstruct: 344.66ms
2026-05-27T03:24:54.513381Z  INFO stall: STALL after_domain_merge: 212.21ms
2026-05-27T03:24:54.513410Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:54.514049Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:54.514177Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:54.514295Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:54.514418Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:54.514571Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:54.514736Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:54.532421Z  INFO stall: STALL post_egui: 18.90ms
2026-05-27T03:24:54.533696Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:54.533794Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:54.540214Z  INFO stall: STALL last: 6.32ms
2026-05-27T03:24:54.540225Z  INFO stage5_readiness::live: READINESS_FRAME_FENCE_OK eval_inv=60 frame_tick=60 passes=true
2026-05-27T03:24:54.540239Z  INFO visual_diag: VISUAL_DIAG window frame=59 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:54.540547Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=59 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:54.540767Z  INFO visual_diag: VISUAL_DIAG camera frame=59 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=89.2947998046875 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=17 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=14 ortho_fixed_h=8 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:54.540966Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=59 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:54.541164Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=59 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:54.541269Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=59 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:54.541423Z  INFO visual_diag: VISUAL_DIAG render_spine frame=59 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=1.0 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.9921318888664246 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=60 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:54.541705Z  INFO visual_diag: VISUAL_DIAG perf frame=59 tile_raster_ms=207.67759704589844 tile_raster_ran=true world_repr_ms=0.20839999616146088 projection_graph_ms=0.0021000001579523087 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21329998970031738 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:54.541871Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=59 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:54.541991Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 249.9 ms)
2026-05-27T03:24:54.542101Z  INFO perf: PERF wall=768.33 instr=208.11 gap=560.23 | cpu_pre_egui=739.71 cpu_egui=19.06 cpu_post_egui=9.56 gpu_gap=0.00 | spine=0.01 world_repr=0.21 graph=0.00 merge=0.00 atm=0.00 readiness=0.21 raster=207.68 | upd_attrib sum=366.27 pv_cpu=0.00 pv_gpu=0.02 fire=21.57 stream=344.66 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=210.43 hud=0.00 overlay=0.00 raster_b=207.68 particles=0.00 residency=2.70 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.06 | stall first+preupd=0.08 update=0.00 post_dom=212.21 post_vt=0.14 post→ready=0.00 ready=1.47 post→egui=0.00 egui=18.90 post_egui=6.32 | stall_hits=[after_tile_storage_apply:527.4,after_domain_merge:212.2,post_egui:18.9,last:6.3]
2026-05-27T03:24:54.542219Z  INFO perf: PERF frame=768.3ms update=739.7ms egui=19.1ms preview=0.0ms streaming=344.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=207.7ms
2026-05-27T03:24:54.542310Z  INFO stall: STALL culprit=after_tile_storage_apply duration=527.4ms frame=768.3ms
2026-05-27T03:24:54.545903Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:54.546809Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:54.547163Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:54.549377Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:54.549469Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:54.549559Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:54.674390Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=89.2948 world_main_xy=(160.00,160.00) zoom=89.2948 bridge_drift=0.0000
2026-05-27T03:24:54.711413Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:55.051571Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=61 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:55.051800Z  INFO stall: STALL after_tile_storage_apply: 506.88ms
2026-05-27T03:24:55.052773Z  INFO stall: STALL upd_streaming_reconstruct: 340.96ms
2026-05-27T03:24:55.246425Z  INFO stall: STALL after_domain_merge: 194.63ms
2026-05-27T03:24:55.246455Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.246980Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:55.247092Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.247216Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.247359Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.247527Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.247678Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:55.264824Z  INFO stall: STALL post_egui: 18.28ms
2026-05-27T03:24:55.265682Z  INFO ui_layout_tree: UI_LAYOUT_TREE frame=60 root=22653v1 target=102709v0
2026-05-27T03:24:55.265808Z  INFO ui_layout_tree: hud_root (hud_root) entity=22653v1  size=(1280.0,720.0) width=100.0% height=100.0% min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.265970Z  INFO ui_layout_tree:   unnamed () entity=22345v1  size=(1280.0,38.0) width=100.0% height=38.0px min=(Auto,38.0px) max=(Auto,38.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.266091Z  INFO ui_layout_tree:     unnamed () entity=22344v1  size=(167.0,40.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266208Z  INFO ui_layout_tree:       unnamed () entity=22343v1  size=(153.0,32.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266312Z  INFO ui_layout_tree:     unnamed () entity=22342v1  size=(100.0,38.0) width=Auto height=Auto min=(100.0px,Auto) max=(Auto,Auto) flex_grow=1.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266431Z  INFO ui_layout_tree:       unnamed () entity=22341v1  size=(20.0,22.0) width=22.0px height=22.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266539Z  INFO ui_layout_tree:         unnamed () entity=22340v1  size=(11.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266657Z  INFO ui_layout_tree:       unnamed () entity=22339v1  size=(64.0,32.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266765Z  INFO ui_layout_tree:     unnamed () entity=22338v1  size=(184.0,38.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266872Z  INFO ui_layout_tree:       unnamed () entity=22025v1  size=(174.0,32.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.266979Z  INFO ui_layout_tree:     unnamed () entity=22024v1  size=(190.0,38.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.267086Z  INFO ui_layout_tree:       unnamed () entity=22023v1  size=(180.0,32.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.267190Z  INFO ui_layout_tree:     unnamed () entity=102700v0  size=(495.0,38.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.267286Z  INFO ui_layout_tree:       unnamed () entity=102701v0  size=(485.0,32.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.267396Z  INFO ui_layout_tree:     unnamed () entity=102702v0  size=(50.0,33.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.269380Z  INFO ui_layout_tree:       unnamed () entity=102703v0  size=(36.0,27.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.269502Z  INFO ui_layout_tree:   unnamed () entity=102704v0  size=(1280.0,26.0) width=100.0% height=26.0px min=(Auto,26.0px) max=(Auto,26.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.269609Z  INFO ui_layout_tree:     unnamed () entity=102705v0  size=(898.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.269718Z  INFO ui_layout_tree:   unnamed () entity=102706v0  size=(1280.0,22.0) width=100.0% height=22.0px min=(Auto,22.0px) max=(Auto,22.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.269824Z  INFO ui_layout_tree:     unnamed () entity=102707v0  size=(580.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.269932Z  INFO ui_layout_tree:   center_row (center_row) entity=102708v0  size=(1280.0,720.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.270042Z  INFO ui_layout_tree:     sim_map_fill (sim_map_fill) entity=102709v0 <<< SIM_VIEWPORT size=(1280.0,720.0) width=100.0% height=100.0% min=(400.0px,300.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.270152Z  INFO ui_layout_tree:       map_viewport_frame_inset () entity=102710v0  size=(1272.0,712.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270258Z  INFO ui_layout_tree:   left_stack_overlay (left_stack_overlay) entity=102711v0  size=(106.0,624.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Stretch align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.270360Z  INFO ui_layout_tree:     unnamed () entity=102712v0  size=(48.0,624.0) width=48.0px height=Auto min=(Auto,120.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=FlexStart overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270459Z  INFO ui_layout_tree:       unnamed () entity=102713v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270558Z  INFO ui_layout_tree:       unnamed () entity=102714v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270656Z  INFO ui_layout_tree:       unnamed () entity=102715v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270755Z  INFO ui_layout_tree:       unnamed () entity=102716v0  size=(9.0,17.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270855Z  INFO ui_layout_tree:     build_rail () entity=102717v0  size=(52.0,624.0) width=52.0px height=100.0% min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.270956Z  INFO ui_layout_tree:       unnamed () entity=102718v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271055Z  INFO ui_layout_tree:         unnamed () entity=102719v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271154Z  INFO ui_layout_tree:         unnamed () entity=102720v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271254Z  INFO ui_layout_tree:       unnamed () entity=102721v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271359Z  INFO ui_layout_tree:         unnamed () entity=102722v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271462Z  INFO ui_layout_tree:         unnamed () entity=102723v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271562Z  INFO ui_layout_tree:       unnamed () entity=102724v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271662Z  INFO ui_layout_tree:         unnamed () entity=102725v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271760Z  INFO ui_layout_tree:         unnamed () entity=102726v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271860Z  INFO ui_layout_tree:       unnamed () entity=102727v0  size=(44.0,32.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.271959Z  INFO ui_layout_tree:         unnamed () entity=102728v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272059Z  INFO ui_layout_tree:       unnamed () entity=102729v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272159Z  INFO ui_layout_tree:         unnamed () entity=102730v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272256Z  INFO ui_layout_tree:         unnamed () entity=102731v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272357Z  INFO ui_layout_tree:       unnamed () entity=102732v0  size=(44.0,32.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272456Z  INFO ui_layout_tree:         unnamed () entity=102733v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272556Z  INFO ui_layout_tree:       unnamed () entity=102734v0  size=(44.0,52.0) width=100.0% height=Auto min=(Auto,32.0px) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Center overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272655Z  INFO ui_layout_tree:         unnamed () entity=102735v0  size=(32.0,32.0) width=32.0px height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272753Z  INFO ui_layout_tree:         unnamed () entity=102736v0  size=(12.0,12.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.272853Z  INFO ui_layout_tree:     unnamed () entity=102737v0  size=(0.0,0.0) width=400.0px height=100.0% min=(Auto,Auto) max=(Auto,100.0%) flex_grow=0.00 flex_shrink=0.00 flex_dir=Column align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Clip, y: Clip }
2026-05-27T03:24:55.272957Z  INFO ui_layout_tree:       unnamed () entity=102738v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=FlexEnd justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273056Z  INFO ui_layout_tree:         unnamed () entity=102739v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273154Z  INFO ui_layout_tree:       unnamed () entity=102740v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273256Z  INFO ui_layout_tree:       unnamed () entity=102741v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273354Z  INFO ui_layout_tree:       unnamed () entity=102742v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273453Z  INFO ui_layout_tree:       unnamed () entity=102743v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273552Z  INFO ui_layout_tree:       unnamed () entity=102744v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273851Z  INFO ui_layout_tree:       unnamed () entity=102745v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.273957Z  INFO ui_layout_tree:       unnamed () entity=102746v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274056Z  INFO ui_layout_tree:       unnamed () entity=102747v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274174Z  INFO ui_layout_tree:       unnamed () entity=102748v0  size=(0.0,0.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274267Z  INFO ui_layout_tree:   minimap_chrome_root () entity=102749v0  size=(262.0,222.0) width=262.0px height=222.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274435Z  INFO ui_layout_tree:     unnamed () entity=102750v0  size=(260.0,220.0) width=100.0% height=100.0% min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274530Z  INFO ui_layout_tree:   context_tray_root () entity=102751v0  size=(1174.0,32.0) width=Auto height=32.0px min=(Auto,32.0px) max=(Auto,32.0px) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274625Z  INFO ui_layout_tree:     unnamed () entity=102752v0  size=(1174.0,32.0) width=100.0% height=32.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274717Z  INFO ui_layout_tree:       unnamed () entity=102753v0  size=(59.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274808Z  INFO ui_layout_tree:         unnamed () entity=102754v0  size=(40.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274900Z  INFO ui_layout_tree:       unnamed () entity=102755v0  size=(51.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.274992Z  INFO ui_layout_tree:         unnamed () entity=102756v0  size=(33.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275084Z  INFO ui_layout_tree:       unnamed () entity=102757v0  size=(78.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275177Z  INFO ui_layout_tree:         unnamed () entity=102758v0  size=(60.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275277Z  INFO ui_layout_tree:       unnamed () entity=102759v0  size=(45.0,24.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275376Z  INFO ui_layout_tree:         unnamed () entity=102760v0  size=(27.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275477Z  INFO ui_layout_tree:     unnamed () entity=102761v0  size=(1174.0,71.0) width=100.0% height=96.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275577Z  INFO ui_layout_tree:       petroleum_panel_tab () entity=102762v0  size=(536.0,45.0) width=100.0% height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275677Z  INFO ui_layout_tree:         unnamed () entity=102763v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275775Z  INFO ui_layout_tree:         unnamed () entity=102764v0  size=(60.0,14.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.275885Z  INFO ui_layout_tree:       logistics_vehicle_chips () entity=102765v0  size=(529.0,45.0) width=100.0% height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=FlexStart align_self=Auto justify=FlexStart overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276002Z  INFO ui_layout_tree:         unnamed () entity=102766v0  size=(37.0,45.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276111Z  INFO ui_layout_tree:           unnamed () entity=102767v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276206Z  INFO ui_layout_tree:           unnamed () entity=102768v0  size=(27.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276298Z  INFO ui_layout_tree:         unnamed () entity=102769v0  size=(34.0,45.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276402Z  INFO ui_layout_tree:           unnamed () entity=102770v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276494Z  INFO ui_layout_tree:           unnamed () entity=102771v0  size=(22.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276586Z  INFO ui_layout_tree:         unnamed () entity=102772v0  size=(34.0,45.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Column align_items=Center align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276679Z  INFO ui_layout_tree:           unnamed () entity=102773v0  size=(24.0,24.0) width=24.0px height=24.0px min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=0.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276771Z  INFO ui_layout_tree:           unnamed () entity=102774v0  size=(17.0,11.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276863Z  INFO ui_layout_tree:       unnamed () entity=102775v0  size=(89.0,51.0) width=Auto height=Auto min=(Auto,Auto) max=(Auto,Auto) flex_grow=0.00 flex_shrink=1.00 flex_dir=Row align_items=Default align_self=Auto justify=Default overflow=Overflow { x: Visible, y: Visible }
2026-05-27T03:24:55.276990Z  INFO ui_layout_tree::chain: MAP_LAYOUT_CHAIN:
  Window: 1280x720
  RootHud: 1280x720
  center_row: 1280x720
  sim_map_fill: 1280x720
  MapFill: 1280x720
  Measured: 1280x720
  Committed: 1280x720
  Solver(SimMapFill): 1280x720
  CommittedResource: 1280x720 last_commit=hole_hold frame=60
2026-05-27T03:24:55.277138Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:55.277240Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:55.277341Z  INFO stall: STALL after_readiness: 12.52ms
2026-05-27T03:24:55.283823Z  INFO stall: STALL last: 6.48ms
2026-05-27T03:24:55.283851Z  INFO visual_diag: VISUAL_DIAG window frame=60 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:55.285064Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=60 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:55.285279Z  INFO visual_diag: VISUAL_DIAG camera frame=60 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=89.2947998046875 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=18 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=14 ortho_fixed_h=8 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:55.285470Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=60 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:55.285659Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=60 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:55.285764Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=60 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:55.285918Z  INFO visual_diag: VISUAL_DIAG render_spine frame=60 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.9921318888664246 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.9921318888664246 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=61 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:55.286204Z  INFO visual_diag: VISUAL_DIAG perf frame=60 tile_raster_ms=190.07040405273438 tile_raster_ran=true world_repr_ms=0.19930000603199005 projection_graph_ms=0.0012000000569969416 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21699999272823334 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:55.286370Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=60 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:55.286512Z  INFO perf: PERF wall=741.66 instr=190.49 gap=551.17 | cpu_pre_egui=701.58 cpu_egui=18.42 cpu_post_egui=21.65 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=190.07 | upd_attrib sum=344.41 pv_cpu=0.00 pv_gpu=0.02 fire=3.42 stream=340.96 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=193.03 hud=0.00 overlay=0.00 raster_b=190.07 particles=0.00 residency=2.91 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.42 | stall first+preupd=0.09 update=0.00 post_dom=194.63 post_vt=0.11 post→ready=0.01 ready=12.52 post→egui=0.01 egui=18.28 post_egui=6.48 | stall_hits=[after_tile_storage_apply:506.9,after_domain_merge:194.6,post_egui:18.3,after_readiness:12.5,last:6.5]
2026-05-27T03:24:55.286630Z  INFO perf: PERF frame=741.7ms update=701.6ms egui=18.4ms preview=0.0ms streaming=341.0ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=190.1ms
2026-05-27T03:24:55.286717Z  INFO stall: STALL culprit=after_tile_storage_apply duration=506.9ms frame=741.7ms
2026-05-27T03:24:55.290730Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.291649Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:55.292007Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:55.292100Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:55.292182Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:55.292260Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:55.414940Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=89.2948 world_main_xy=(160.00,160.00) zoom=89.2948 bridge_drift=0.0000
2026-05-27T03:24:55.451563Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:55.786260Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=62 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:55.786442Z  INFO stall: STALL after_tile_storage_apply: 497.05ms
2026-05-27T03:24:55.787214Z  INFO stall: STALL upd_streaming_reconstruct: 335.27ms
2026-05-27T03:24:55.988912Z  INFO stall: STALL after_domain_merge: 202.46ms
2026-05-27T03:24:55.988942Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.989519Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:55.989664Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.989793Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.989914Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.990057Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:55.990220Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:56.007575Z  INFO stall: STALL post_egui: 18.53ms
2026-05-27T03:24:56.008845Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:56.008954Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:56.015320Z  INFO stall: STALL last: 6.27ms
2026-05-27T03:24:56.015341Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=61 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=19 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(25, 14) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(52.102642, 52.102642, 52.102642) minimap_visible=true
2026-05-27T03:24:56.015346Z  INFO visual_diag: VISUAL_DIAG window frame=61 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:56.016003Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=61 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:56.016226Z  INFO visual_diag: VISUAL_DIAG camera frame=61 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=19 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:56.016427Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=61 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:56.016619Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=61 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:56.016728Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=61 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:56.016892Z  INFO visual_diag: VISUAL_DIAG render_spine frame=61 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.9921318888664246 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=62 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:56.017192Z  INFO visual_diag: VISUAL_DIAG perf frame=61 tile_raster_ms=197.90670776367188 tile_raster_ran=true world_repr_ms=0.19519999623298645 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2281000018119812 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:56.017367Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=61 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:56.017513Z  INFO perf: PERF wall=728.15 instr=198.33 gap=529.82 | cpu_pre_egui=699.57 cpu_egui=18.68 cpu_post_egui=9.90 gpu_gap=0.00 | spine=0.00 world_repr=0.20 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=197.91 | upd_attrib sum=338.89 pv_cpu=0.00 pv_gpu=0.01 fire=3.61 stream=335.27 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=201.02 hud=0.00 overlay=0.00 raster_b=197.91 particles=0.00 residency=3.06 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.68 | stall first+preupd=0.06 update=0.00 post_dom=202.46 post_vt=0.13 post→ready=0.00 ready=1.48 post→egui=0.00 egui=18.53 post_egui=6.27 | stall_hits=[after_tile_storage_apply:497.0,after_domain_merge:202.5,post_egui:18.5,last:6.3]
2026-05-27T03:24:56.017635Z  INFO perf: PERF frame=728.2ms update=699.6ms egui=18.7ms preview=0.0ms streaming=335.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=197.9ms
2026-05-27T03:24:56.017728Z  INFO stall: STALL culprit=after_tile_storage_apply duration=497.0ms frame=728.2ms
2026-05-27T03:24:56.021454Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.022328Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:56.022699Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:56.023412Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:56.023491Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:56.023569Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:56.145805Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:56.183114Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:56.519120Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=63 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:56.519293Z  INFO stall: STALL after_tile_storage_apply: 498.63ms
2026-05-27T03:24:56.519713Z  INFO stall: STALL upd_streaming_reconstruct: 336.14ms
2026-05-27T03:24:56.715921Z  INFO stall: STALL after_domain_merge: 196.62ms
2026-05-27T03:24:56.715958Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.717110Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:56.717222Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.717340Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.717462Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.717608Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.717765Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:56.735419Z  INFO stall: STALL post_egui: 19.37ms
2026-05-27T03:24:56.736835Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:56.736930Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:56.743372Z  INFO stall: STALL last: 6.35ms
2026-05-27T03:24:56.743395Z  INFO visual_diag: VISUAL_DIAG window frame=62 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:56.743620Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=62 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:56.743896Z  INFO visual_diag: VISUAL_DIAG camera frame=62 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=20 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:56.744111Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=62 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:56.744307Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=62 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:56.744418Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=62 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:56.744580Z  INFO visual_diag: VISUAL_DIAG render_spine frame=62 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=63 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:56.744875Z  INFO visual_diag: VISUAL_DIAG perf frame=62 tile_raster_ms=191.547607421875 tile_raster_ran=true world_repr_ms=0.24449999630451202 projection_graph_ms=0.001600000075995922 domain_merge_ms=0.00020000000949949026 readiness_ms=0.20329999923706055 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:56.745047Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=62 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:56.745197Z  INFO perf: PERF wall=724.58 instr=192.00 gap=532.58 | cpu_pre_egui=695.32 cpu_egui=19.52 cpu_post_egui=9.74 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.20 raster=191.55 | upd_attrib sum=339.87 pv_cpu=0.00 pv_gpu=0.02 fire=3.70 stream=336.14 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=194.72 hud=0.00 overlay=0.00 raster_b=191.55 particles=0.00 residency=3.12 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.52 | stall first+preupd=0.07 update=0.00 post_dom=196.62 post_vt=0.12 post→ready=0.00 ready=1.60 post→egui=0.00 egui=19.37 post_egui=6.35 | stall_hits=[after_tile_storage_apply:498.6,after_domain_merge:196.6,post_egui:19.4,last:6.3]
2026-05-27T03:24:56.745322Z  INFO perf: PERF frame=724.6ms update=695.3ms egui=19.5ms preview=0.0ms streaming=336.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=191.5ms
2026-05-27T03:24:56.745410Z  INFO stall: STALL culprit=after_tile_storage_apply duration=498.6ms frame=724.6ms
2026-05-27T03:24:56.749887Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:56.750561Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:56.751227Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:56.751316Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:56.751394Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:56.751471Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:56.876317Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:56.912927Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:57.244468Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=64 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:57.244656Z  INFO stall: STALL after_tile_storage_apply: 495.89ms
2026-05-27T03:24:57.245142Z  INFO stall: STALL upd_streaming_reconstruct: 331.75ms
2026-05-27T03:24:57.448062Z  INFO stall: STALL after_domain_merge: 203.40ms
2026-05-27T03:24:57.448109Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:57.448722Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:57.448828Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:57.448976Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:57.449063Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:57.449177Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:57.466988Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:57.467582Z  INFO stall: STALL post_egui: 19.35ms
2026-05-27T03:24:57.469865Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:57.470499Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:57.476841Z  INFO stall: STALL last: 6.25ms
2026-05-27T03:24:57.476865Z  INFO visual_diag: VISUAL_DIAG window frame=63 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:57.477088Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=63 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:57.477309Z  INFO visual_diag: VISUAL_DIAG camera frame=63 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=21 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:57.477509Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=63 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:57.477710Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=63 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:57.477815Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=63 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:57.477981Z  INFO visual_diag: VISUAL_DIAG render_spine frame=63 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=64 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:57.478301Z  INFO visual_diag: VISUAL_DIAG perf frame=63 tile_raster_ms=198.91009521484375 tile_raster_ran=true world_repr_ms=0.24640001356601715 projection_graph_ms=0.00139999995008111 domain_merge_ms=0.00010000000474974513 readiness_ms=0.7434000372886658 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:57.478479Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=63 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:57.478643Z  INFO perf: PERF wall=729.90 instr=199.90 gap=530.00 | cpu_pre_egui=699.34 cpu_egui=19.53 cpu_post_egui=11.02 gpu_gap=0.00 | spine=0.00 world_repr=0.25 graph=0.00 merge=0.00 atm=0.00 readiness=0.74 raster=198.91 | upd_attrib sum=335.43 pv_cpu=0.00 pv_gpu=0.02 fire=3.65 stream=331.75 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=201.98 hud=0.00 overlay=0.00 raster_b=198.91 particles=0.00 residency=3.02 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.53 | stall first+preupd=0.06 update=0.00 post_dom=203.40 post_vt=0.17 post→ready=0.00 ready=3.01 post→egui=0.00 egui=19.35 post_egui=6.25 | stall_hits=[after_tile_storage_apply:495.9,after_domain_merge:203.4,post_egui:19.3,last:6.3]
2026-05-27T03:24:57.478769Z  INFO perf: PERF frame=729.9ms update=699.3ms egui=19.5ms preview=0.0ms streaming=331.7ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=198.9ms
2026-05-27T03:24:57.478868Z  INFO stall: STALL culprit=after_tile_storage_apply duration=495.9ms frame=729.9ms
2026-05-27T03:24:57.481024Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:57.481893Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:57.482438Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:57.482528Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:57.482608Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:57.485637Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:57.608599Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:57.644782Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:57.985181Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=65 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:57.985348Z  INFO stall: STALL after_tile_storage_apply: 505.00ms
2026-05-27T03:24:57.986197Z  INFO stall: STALL upd_streaming_reconstruct: 340.95ms
2026-05-27T03:24:58.178493Z  INFO stall: STALL after_domain_merge: 193.14ms
2026-05-27T03:24:58.178495Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.179026Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:58.179141Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.179262Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.179384Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.179527Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.179666Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:58.196521Z  INFO stall: STALL post_egui: 17.92ms
2026-05-27T03:24:58.198823Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:58.198926Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:58.205131Z  INFO stall: STALL last: 6.10ms
2026-05-27T03:24:58.205151Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=64 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=22 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(20, 11) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(65.63433, 65.63433, 65.63433) minimap_visible=true
2026-05-27T03:24:58.205158Z  INFO visual_diag: VISUAL_DIAG window frame=64 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:58.205872Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=64 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:58.206170Z  INFO visual_diag: VISUAL_DIAG camera frame=64 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=65.63433074951172 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=22 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=20 ortho_fixed_h=11 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:58.206396Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=64 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:58.206585Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=64 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:58.206689Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=64 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:58.206845Z  INFO visual_diag: VISUAL_DIAG render_spine frame=64 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.7281448841094971 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=65 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:58.207132Z  INFO visual_diag: VISUAL_DIAG perf frame=64 tile_raster_ms=188.5937042236328 tile_raster_ran=true world_repr_ms=0.24269999563694 projection_graph_ms=0.001600000075995922 domain_merge_ms=0.00010000000474974513 readiness_ms=0.21850000321865082 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:58.207303Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=64 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:58.207461Z  INFO perf: PERF wall=727.19 instr=189.06 gap=538.13 | cpu_pre_egui=698.21 cpu_egui=18.07 cpu_post_egui=10.90 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.22 raster=188.59 | upd_attrib sum=344.46 pv_cpu=0.00 pv_gpu=0.02 fire=3.47 stream=340.95 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=191.53 hud=0.00 overlay=0.00 raster_b=188.59 particles=0.00 residency=2.88 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.07 | stall first+preupd=0.10 update=0.00 post_dom=193.14 post_vt=0.10 post→ready=0.00 ready=2.51 post→egui=0.00 egui=17.92 post_egui=6.10 | stall_hits=[after_tile_storage_apply:505.0,after_domain_merge:193.1,post_egui:17.9,last:6.1]
2026-05-27T03:24:58.207585Z  INFO perf: PERF frame=727.2ms update=698.2ms egui=18.1ms preview=0.0ms streaming=340.9ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=188.6ms
2026-05-27T03:24:58.207678Z  INFO stall: STALL culprit=after_tile_storage_apply duration=505.0ms frame=727.2ms
2026-05-27T03:24:58.212913Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.213792Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:58.214125Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:58.215716Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:58.215793Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:58.215880Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:58.336708Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=65.6343 world_main_xy=(160.00,160.00) zoom=65.6343 bridge_drift=0.0000
2026-05-27T03:24:58.373475Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:58.731569Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=66 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:24:58.731829Z  INFO stall: STALL after_tile_storage_apply: 519.99ms
2026-05-27T03:24:58.733047Z  INFO stall: STALL upd_streaming_reconstruct: 359.16ms
2026-05-27T03:24:58.932118Z  INFO stall: STALL after_domain_merge: 200.28ms
2026-05-27T03:24:58.932144Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.932702Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:58.932827Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.932959Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.933099Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.933262Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:58.933418Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:58.951547Z  INFO stall: STALL post_egui: 19.27ms
2026-05-27T03:24:58.952957Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:58.953075Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:58.960181Z  INFO stall: STALL last: 7.00ms
2026-05-27T03:24:58.960202Z  INFO visual_diag: VISUAL_DIAG window frame=65 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:58.960206Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=65 win_logical=(1280, 720) win_physical=(1280, 720) sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=23 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_wh=(25, 14) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(52.102642, 52.102642, 52.102642) minimap_visible=true
2026-05-27T03:24:58.960521Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=65 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:58.961360Z  INFO visual_diag: VISUAL_DIAG camera frame=65 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=23 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:58.961590Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=65 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:58.961814Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=65 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:58.961935Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=65 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:58.962121Z  INFO visual_diag: VISUAL_DIAG render_spine frame=65 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.7281448841094971 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=66 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:58.962421Z  INFO visual_diag: VISUAL_DIAG perf frame=65 tile_raster_ms=195.7139892578125 tile_raster_ran=true world_repr_ms=0.22169999778270721 projection_graph_ms=0.0013000000035390258 domain_merge_ms=0.00020000000949949026 readiness_ms=0.24199999868869781 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:58.962589Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=65 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:58.962729Z  INFO perf: PERF wall=750.94 instr=196.18 gap=554.76 | cpu_pre_egui=720.33 cpu_egui=19.46 cpu_post_egui=11.14 gpu_gap=0.00 | spine=0.00 world_repr=0.22 graph=0.00 merge=0.00 atm=0.00 readiness=0.24 raster=195.71 | upd_attrib sum=362.48 pv_cpu=0.00 pv_gpu=0.02 fire=3.30 stream=359.16 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=198.53 hud=0.00 overlay=0.00 raster_b=195.71 particles=0.00 residency=2.77 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=19.46 | stall first+preupd=0.07 update=0.01 post_dom=200.28 post_vt=0.14 post→ready=0.01 ready=1.63 post→egui=0.01 egui=19.27 post_egui=7.00 | stall_hits=[after_tile_storage_apply:520.0,after_domain_merge:200.3,post_egui:19.3,last:7.0]
2026-05-27T03:24:58.962848Z  INFO perf: PERF frame=750.9ms update=720.3ms egui=19.5ms preview=0.0ms streaming=359.2ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=195.7ms
2026-05-27T03:24:58.962937Z  INFO stall: STALL culprit=after_tile_storage_apply duration=520.0ms frame=750.9ms
2026-05-27T03:24:59.078484Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.079352Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:59.079872Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:59.079988Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:59.080086Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:59.080178Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:59.202412Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:59.239334Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:24:59.578385Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=67 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=true
2026-05-27T03:24:59.578566Z  INFO stall: STALL after_tile_storage_apply: 501.31ms
2026-05-27T03:24:59.578898Z  INFO stall: STALL upd_streaming_reconstruct: 339.11ms
2026-05-27T03:24:59.769541Z  INFO stall: STALL after_domain_merge: 190.97ms
2026-05-27T03:24:59.769575Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiMeasured valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.770448Z  INFO viewport_authority::solver: VIEWPORT_SOLVER_TARGET node=SimMapFill valid=true rect.source=SimMapFill w=1280.0 h=720.0
2026-05-27T03:24:59.770571Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.770703Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=LayoutSolver valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.770834Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=UiCommitted valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.771000Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=CameraApplied valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.771162Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:24:59.788125Z  INFO stall: STALL post_egui: 18.45ms
2026-05-27T03:24:59.789546Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:24:59.789660Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:24:59.796288Z  INFO stall: STALL last: 6.53ms
2026-05-27T03:24:59.796320Z  INFO visual_diag: VISUAL_DIAG window frame=66 periodic=false win_logical=(1280, 720) win_physical=(1280, 720) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:24:59.796549Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=66 sim_valid=true sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:24:59.796762Z  INFO visual_diag: VISUAL_DIAG camera frame=66 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=52.10264205932617 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=24 cam_scissor=Some((0, 0, 1280, 720)) ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:24:59.796954Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=66 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:24:59.797152Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=66 world_preview_proj_rev=2551212574560 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454492
2026-05-27T03:24:59.797263Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=66 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:24:59.797428Z  INFO visual_diag: VISUAL_DIAG render_spine frame=66 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=0.5771676898002625 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=67 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:24:59.797716Z  INFO visual_diag: VISUAL_DIAG perf frame=66 tile_raster_ms=186.16839599609375 tile_raster_ran=true world_repr_ms=0.24289999902248383 projection_graph_ms=0.0017000000225380063 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2287999987602234 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:24:59.797884Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=66 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:24:59.798022Z  INFO perf: PERF wall=720.83 instr=186.64 gap=534.18 | cpu_pre_egui=692.36 cpu_egui=18.60 cpu_post_egui=9.87 gpu_gap=0.00 | spine=0.00 world_repr=0.24 graph=0.00 merge=0.00 atm=0.00 readiness=0.23 raster=186.17 | upd_attrib sum=342.68 pv_cpu=0.00 pv_gpu=0.02 fire=3.54 stream=339.11 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=189.17 hud=0.00 overlay=0.00 raster_b=186.17 particles=0.00 residency=2.95 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=18.60 | stall first+preupd=0.08 update=0.00 post_dom=190.97 post_vt=0.12 post→ready=0.01 ready=1.64 post→egui=0.01 egui=18.45 post_egui=6.53 | stall_hits=[after_tile_storage_apply:501.3,after_domain_merge:191.0,post_egui:18.5,last:6.5]
2026-05-27T03:24:59.798144Z  INFO perf: PERF frame=720.8ms update=692.4ms egui=18.6ms preview=0.0ms streaming=339.1ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.2ms raster=186.2ms
2026-05-27T03:24:59.798236Z  INFO stall: STALL culprit=after_tile_storage_apply duration=501.3ms frame=720.8ms
2026-05-27T03:24:59.809383Z  INFO viewport_authority: VIEWPORT_AUTHORITY source=ResolvedViewport valid=true min=Vec2(0.0, 0.0) max=Vec2(1280.0, 720.0) w=1280.0 h=720.0
2026-05-27T03:24:59.810112Z  INFO industrial_activation_todos: INDUSTRIAL_ACTIVATION_GREEN
2026-05-27T03:24:59.810691Z  INFO construction_finish_todos: CONSTRUCTION_FINISH_COMPLETE done=8/8
2026-05-27T03:24:59.810783Z  INFO construction_round2_todos: CONSTRUCTION_ROUND2_COMPLETE done=15/15
2026-05-27T03:24:59.810863Z  INFO construction_round3_todos: CONSTRUCTION_ROUND3_COMPLETE done=27/27
2026-05-27T03:24:59.810940Z  INFO construction_operational_todos: CONSTRUCTION_OPERATIONAL_GREEN
2026-05-27T03:24:59.931200Z  INFO stage5_live_todos: STAGE5_MAP_CAMERA_HOOK post_mirror desired_xy=(160.00,160.00) zoom=52.1026 world_main_xy=(160.00,160.00) zoom=52.1026 bridge_drift=0.0000
2026-05-27T03:24:59.967565Z  INFO stage5_live_todos: STAGE5_FIRE_HOOK fire_visual_producer_count=1 (expect 1 for FIRE-01)
2026-05-27T03:25:00.301744Z  INFO stage5_live_todos: STAGE5_SPINE_HOOK policy_present=true graph_present=true fence_committed=true fence_fire_tick=68 world_frame_present=true overlay_rev=2 overlay_chunk_cells=28 atm_partial_dispatch=1 atm_gpu_tex_uploads=0 atm_full_field_fallback=false
2026-05-27T03:25:00.301966Z  INFO stall: STALL after_tile_storage_apply: 493.71ms
2026-05-27T03:25:00.302402Z  INFO stall: STALL upd_streaming_reconstruct: 334.34ms
2026-05-27T03:25:00.501681Z  INFO bevy_window::system: No windows are open, exiting
2026-05-27T03:25:00.501680Z  INFO stall: STALL after_domain_merge: 199.71ms
2026-05-27T03:25:00.518881Z  INFO stage5_live_todos: STAGE5_GPU_HOOK indirect_instances=0 dispatch_count=0 draw_instances=0 (PHF-01 alignment)
2026-05-27T03:25:00.519479Z  INFO stall: STALL post_egui: 17.25ms
2026-05-27T03:25:00.520829Z  INFO stage5_live_todos: STAGE5_READINESS_HOOK passes=true violations_first=(none) producer_count=1
2026-05-27T03:25:00.520927Z  INFO visual_aidv2_live_todos: VISUAL_AID_V2_BOARD done=3/6 footprint_ok=false readability=true icons=0
2026-05-27T03:25:00.527189Z  INFO stall: STALL last: 6.17ms
2026-05-27T03:25:00.527210Z  WARN visual_diag::anomaly: CAMERA_SCISSOR_CHANGED frame=67 was=Some((0, 0, 1280, 720)) now=None
2026-05-27T03:25:00.527207Z  INFO sim_view_sync: SIM_VIEW_SYNC frame=67 win_logical=(1, 1) win_physical=(1, 1) sim_valid=false sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) measured_valid=true measured_wh=Vec2(1280.0, 720.0) committed_wh=Vec2(1280.0, 720.0) sim_wh=Vec2(1280.0, 720.0) settle_streak=4 layout_settled=true sim_held=true last_commit="hole_hold" frozen=true pending_wh=Vec2(1280.0, 720.0) cam_hole=true render_hole=true cam_invalid_streak=0 cam_valid_streak=24 cam_scissor=None ortho_fixed_wh=(25, 14) map_view_px=(1280, 720) raster_rev=30 resolved_rev=79 app=2 base=2 flow=3 worldgen=5 cmd_shell=false overlay_tray=false transmission=false rect_sanity_issues=0 left_stack_collapsed=true map_cam_scale=Vec3(76.55589, 76.55589, 76.55589) minimap_visible=true
2026-05-27T03:25:00.527371Z  WARN visual_diag::anomaly: SIM_VIEWPORT_VALIDITY_CHANGED frame=67 was_valid=true now_valid=false
2026-05-27T03:25:00.527773Z  WARN sim_view_sync::anomaly: CAMERA_SCISSOR_CHANGED frame=67 was=Some((0, 0, 1280, 720)) now=None
2026-05-27T03:25:00.527869Z  INFO visual_diag: VISUAL_DIAG window frame=67 periodic=false win_logical=(1, 1) win_physical=(1, 1) scale=1.0 app=2 base=2 flow=3 worldgen=5 readiness_profile=Stage5ReadinessProfile { require_vt4: true, require_vt5: true, require_preview: true, require_partial_metrics: true, require_world_frame: true, require_phase_f_proof: true, require_instanced_draw: true }
2026-05-27T03:25:00.527963Z  WARN sim_view_sync::anomaly: SIM_MAP_VIEWPORT_VALIDITY_CHANGED frame=67 was_valid=true now_valid=false was_adequate=true now_adequate=true
2026-05-27T03:25:00.528135Z  INFO visual_diag: VISUAL_DIAG sim_viewport frame=67 sim_valid=false sim_adequate=true sim_min=Vec2(0.0, 0.0) sim_max=Vec2(1280.0, 720.0) sim_wh=(1280, 720) measured_valid=true measured_wh=(1280, 720) committed_wh=(1280, 720) sim_held=true settle_streak=4 layout_settled=true frozen=true last_commit="hole_hold" pending_wh=(1280, 720) pending_min=Vec2(0.0, 0.0) pending_max=Vec2(1280.0, 720.0)
2026-05-27T03:25:00.528461Z  INFO visual_diag: VISUAL_DIAG camera frame=67 cam_desired_x=160.0 cam_desired_y=160.0 cam_zoom=76.55589294433594 latch_hole=true render_hole=true latch_invalid_streak=0 latch_valid_streak=24 cam_scissor=None ortho_fixed_w=25 ortho_fixed_h=14 map_view_px_w=1280 map_view_px_h=720 world_w=320 world_h=320
2026-05-27T03:25:00.528699Z  INFO visual_diag: VISUAL_DIAG resolved_viewports frame=67 resolved_rev=79 primary_valid=true primary_wh=(1280, 720) sim_resolved_valid=true sim_resolved_wh=(1280, 720) preview_valid=true preview_wh=(727, 594) minimap_valid=true minimap_wh=(260, 220) mismatch_preview=false mismatch_minimap=false mismatch_sim_map=false mismatch_stale_tex=false
2026-05-27T03:25:00.528929Z  INFO visual_diag: VISUAL_DIAG map_view_frames frame=67 world_preview_proj_rev=2551210574554 minimap_proj_rev=944893805415 sim_map_proj_rev=3092406454490
2026-05-27T03:25:00.529055Z  INFO visual_diag: VISUAL_DIAG map_presentation frame=67 wp_fit=Contain wp_viewport=UVec2(727, 594) wp_fit_scale=1.8322265148162842 wp_expected_fit_scale=1.8322265148162842 wp_zoom=1.7086485624313354 mm_fit=Contain mm_viewport=UVec2(1, 1) mm_fit_scale=1.0
2026-05-27T03:25:00.529252Z  INFO visual_diag: VISUAL_DIAG render_spine frame=67 raster_rev=30 repr_band=0 repr_lod="LocalTactical" particle_rows_cap=262144 visual_intent="Strategic" visual_zoom_alpha=0.5771676898002625 fire_particle_rows=336 fire_spark_rows=336 fire_spark_phase="A+B" fire_spark_scatter_slots=308 fire_spark_scatter_max=14 fire_spark_zoom_alpha=1.0 fire_spark_additive_blend=true fire_spark_budget_capped=false fire_spark_compute_enabled=true fire_particle_view_culled=false fire_particle_stamp=68 overlay_rev=2 overlay_chunk_cells=28 graph_fire_inst=0 graph_fire_heat=0 gpu_instance_rows=0 gpu_dispatch=1 gpu_draw=0
2026-05-27T03:25:00.529622Z  INFO visual_diag: VISUAL_DIAG perf frame=67 tile_raster_ms=195.18460083007813 tile_raster_ran=true world_repr_ms=0.2912999987602234 projection_graph_ms=0.001500000013038516 domain_merge_ms=0.00020000000949949026 readiness_ms=0.2084999978542328 cpu_pre_egui_ms=0.0 cpu_egui_ms=0.0 cpu_post_egui_ms=0.0 gpu_gap_ms=0.0
2026-05-27T03:25:00.529825Z  INFO visual_diag: VISUAL_DIAG hud_shell frame=67 cmd_shell=false overlay_tray=false transmission=false
2026-05-27T03:25:00.529957Z  INFO bevy_winit::system: Closing window 0v0
2026-05-27T03:25:00.529959Z  WARN proc_A_dine01::gui::hud::frame_budget: frame budget anomaly FrameSpike: frame 250.0 ms (avg 250.0 ms)
2026-05-27T03:25:00.530161Z  INFO perf: PERF wall=721.78 instr=195.69 gap=526.09 | cpu_pre_egui=693.50 cpu_egui=17.82 cpu_post_egui=10.47 gpu_gap=0.00 | spine=0.00 world_repr=0.29 graph=0.00 merge=0.00 atm=0.00 readiness=0.21 raster=195.18 | upd_attrib sum=337.90 pv_cpu=0.00 pv_gpu=0.02 fire=3.53 stream=334.34 map_fit=0.00 hud=0.00 wgen=0.00 | budget_sum=198.16 hud=0.00 overlay=0.00 raster_b=195.18 particles=0.00 residency=2.93 tex_reg=0.00 render_x=0.05 | egui_unbudgeted=17.82 | stall first+preupd=0.08 update=0.00 post_dom=199.71 post_vt=0.55 post→ready=0.00 ready=1.54 post→egui=0.00 egui=17.25 post_egui=6.17 | stall_hits=[after_tile_storage_apply:493.7,after_domain_merge:199.7,post_egui:17.2,last:6.2]
2026-05-27T03:25:00.530283Z  INFO perf: PERF frame=721.8ms update=693.5ms egui=17.8ms preview=0.0ms streaming=334.3ms tile_apply=0.0ms viewport=0.0ms map_fit=0.0ms repr=0.3ms raster=195.2ms
2026-05-27T03:25:00.530384Z  INFO stall: STALL culprit=after_tile_storage_apply duration=493.7ms frame=721.8ms