# Status snapshot — viewport / views / construction map

**Last verified:** 2026-05-23

## §0 Witness vs structural debt (read this first)

| Lane | Witness / board | Status |
|------|-----------------|--------|
| **Stage 5 operational** | **CLOSED** — [`stage5_operational_signoff.md`](stage5_operational_signoff.md); witness `stage5_full_app_live.json` + 2026-05-23 visual session green | **Signed off** |
| **Next open lane** | [`stage5_5_open.md`](stage5_5_open.md) — view runtime / infra / Wave S (pick one track) | **Active** |
| View isolation (infra witness) | `debug_runs/infrastructure_view_isolation_live.json` — `infrastructure_view_isolation_green` | **Green** (not full VM checklist) |
| View isolation (infra) | `debug_runs/infrastructure_view_isolation_live.json` — `infrastructure_view_isolation_green` | **Green** |
| Construction | `debug_runs/construction_stage_live.json` — phase2 + `operational_green` | **Green** |
| Logistics / industrial | `logistics_throughput_live.json`, `industrial_activation_live.json` — `open_todos: 0` | **Green** |
| Session PLAY + post-PLAY | [`session_playback_issues_todos.md`](session_playback_issues_todos.md), [`post_play_followup_todos.md`](post_play_followup_todos.md) | **Closed** |
| Per-view `RepresentationResult` | §8 below — still global, not keyed by `ViewId` | **Debt** |
| VM-06…11 code-complete vs multiview ideal | §4 below — witnesses green; `proj-viewport-authority` writer sweep remains | **Hardening** |
| Frame p95 &lt; 33 ms | [`perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) | **Measure** |
| Construction phase GPU + undo/redo | `site_phase_tile_instances`, `history.rs` | **Done** |
| Replay/editor parity witness | `replay_editor_parity_live.json` | **Hardening** (stamp ring) |

**Archived boards:** [`next_action_todos.md`](next_action_todos.md) (signed off). **Deferred:** [`stage5_triage_backlog.md`](stage5_triage_backlog.md).

---

## 1. ViewManager, ViewId, viewport authority
ViewId / ViewManager (defined in view_authority.rs)
pub enum ViewId {
    WorldMain,
    WorldPreview,
    Minimap,
    SimulationMap,
}
ViewInstance: per-view camera, projection, viewport_rect, render_policy (LOD band, OverlayMask, filters).
ViewManager: HashMap<ViewId, ViewInstance> resource; populated each frame by sync_view_manager_bridge (not a long-lived mutable store writers fight over).
ViewAuthorityPlugin + ViewAuthoritySystemSet: RegisterViewCameras → SyncViewManager, ordered after ViewportPipelineSet::Resolve and after MapCameraSystemSet::ApplyInput.
Helpers: camera_translation, camera_zoom, view_surface_world_to_screen in c:\dev\github\Rust_engine_template_01\src\gui\view_projection_authority.rs.
Viewport authority types (split across GUI vs render)
Type	File	Purpose
ViewportAuthority (requests queue)
src/gui/viewport_authority.rs
UI submits ViewportRequest; cleared after resolve
ResolvedViewport / ResolvedViewports
src/render/viewport_pipeline.rs
Committed logical/physical extents per surface
SemanticViewportRect
src/gui/viewport_layout_solver.rs
Semantic sim-map fill from sim_map_fill measure
AuthoritativeViewport
src/gui/authoritative_viewport.rs
Sole dimension writer for sim-map hole
SimulationMapViewport
src/gui/mod.rs (re-exported)
Published sim-map AABB for camera scissor
ViewportAuthoritySource
src/gui/hud/viewport_authority_debug.rs
Trace enum (Boot, Hud, LayoutSolver, CameraLatch, ResolvedViewport, …)
ViewportAuthorityNode
src/gui/viewport_layout_solver.rs
HudRoot / CenterRow / SimMapFill / RescueFloor
Parallel ID spaces (intentional, easy to confuse)
ViewId — view authority spine (4 buckets).
MapViewInstanceId — map presentation (WorldPreview, Minimap, SimulationMap, TacticalMap, …) in src/gui/map_view/presentation/.
TileDebugViewId — GPU tile debug host tagging.
2. Minimap, WorldMain, world preview, GPU preview paths
WorldMain (tactical / primary Bevy camera)
Entity: MainWorldCamera component (src/gui/map_camera.rs).
Pose write surface: MapCameraDesired (RTS input) + mirror_map_camera_desired_to_world_main → ViewManager[WorldMain].
Viewport/scissor: sync_main_world_camera_viewport_and_projection uses SimulationMapViewport + MainWorldCameraViewportLatch (hole vs full window).
Render: primary window / sim-map hole; drives weather VFX children, construction egui overlays tied to SimulationMapViewport + MainWorldCamera.
Minimap
Stage	Module
Shell / panel size
MinimapShellState → resolve_minimap_panel_viewport in viewport_pipeline.rs
Per-view camera/zoom
MapViewInstances::minimap (MapViewState)
Resolved frame + texture
sync_resolved_map_view_frames → ResolvedMapViewFrames::minimap (map_view/projection/mod.rs)
Egui bind
consumers/minimap.rs → resolve_minimap_egui_texture
Raster source
Usually TileWorldFallbackState / SharedCpuRaster (MinimapPresentationSource in diagnostics)
ViewManager bridge
ViewId::Minimap in sync_view_manager_bridge
Camera intent to main
apply_minimap_camera_intent can write MapCameraDesired (shell focus jumps) before SyncViewManager
World preview
Path	Mode	Key files
CPU raster
PreviewRenderMode::CpuRaster
editor/world_preview/render_raster.rs, WorldPreviewTexture, swap buffers in view_representation.rs
GPU offscreen
PreviewRenderMode::GpuRenderTarget
editor/world_preview/gpu_preview.rs (WorldPreviewGpuCamera), preview_render_contract.rs, render_target_barrier.rs
Viewport requests
submit_viewport_request → ViewportAuthority → resolve_preview_viewport_requests
Extent sync back
sync_editor_viewport_from_resolved copies ResolvedViewports.world_preview → MapViewInstances.world_preview.viewport_size
Map-view consumer
consumers/world_preview.rs, update_world_preview_view (PostUpdate)
Live proof (debug_runs/stage5_full_app_live.json): minimap SharedCpuRaster, preview GPU target ~486×436 aligned with resolved viewport, shared_projection, low mismatch counts.

Simulation map (HUD hole, not “another window”)
ViewId::SimulationMap shares WorldMain camera entity and pose in bridge today (simulation_map_shares_main_camera: true in diagnostics).
ResolvedMapViewFrames::simulation_map uses shared CPU raster from tile fallback; TacticalMap / CommanderMap / etc. alias simulation_map (not world_preview) — explicit anti-bleed test in map_view/mod.rs.
3. Construction ghost / build overlays vs semantic vs render viewport
Construction (gameplay preview — separate lane from Stage 5 VM)
State: BuildGhostState, BuildOverlayVisibility, BuildPlacementPreview — src/construction/.
Interaction: build_interaction.rs (pointer → ghost origin); egui HUD reads ghost in in_game_hud.rs (terrain/net/cost overlay toggles).
World draw: BuildGhostRoot entity + rail/zone ghosts under src/construction/*/ghost.rs; phase labels use SimulationMapViewport + MainWorldCamera only (phase_visual.rs) — not ViewManager / per-view projection.
Invariant: construction is preview-only until commit funnel (construction_invariants.md); ghosts must not be confused with transport “authoring ghost” (documented in transport UX guides).
Semantic vs render viewport
Concern	Semantic	Render
What
Where the sim-map should be in window space
What GPU/egui actually sample
Writer
measure_sim_map_fill_viewport → commit_authority_from_semantic (authoritative_viewport.rs)
ViewportPipelineSet::Resolve (viewport_pipeline.rs)
Consumer
SimulationMapViewport, camera scissor adequacy
ResolvedViewports, ViewRepresentationSnapshot, map-view frames
Risk called out in recovery doc
Multiple historical authorities (measured, frozen, rescue, desired, stabilized) — migration collapsed several wrappers (2026-05 table in recovery_viewport.md)
Build overlays are HUD/egui semantic (hit-test copy in in_game_hud.rs references pointer + build overlays; camera scissor uses ResolvedViewports).

4. VM-06 through VM-11
These IDs live primarily in prompts/guides/base_finsh_5.md (backlog), not as separate rows in stage5_live_todos.rs (which uses TODO-01…TODO-13). Cross-reference: operational_readiness_vs_infrastructure_perf_v1.md, construction_active_progress.md P0.

ID	Intent (from code comments + base_finsh_5 appendix)	Status (editorial)
vm-06
Route world_to_screen / screen_to_world through ViewManager per ViewId; ViewIsolationDiagnostics for minimap↔main lockstep
Partial — helpers exist; diagnostics wired
vm-07
Input isolation: ActiveMapViewInput blocks main MapCameraDesired when preview/minimap focused
Partial — map_camera.rs, map_view/presentation/stability.rs
vm-08
Per-view overlay bitfields in MapViewInstances; separate UI id_prefix
Partial — vm-08 comments in view_authority.rs, hud_dev_overlay
vm-09 / vm-09b
Schedule: SyncViewManager after ApplyInput; MapCameraDesired compatibility write + mirror_map_camera_desired_to_world_main
Partial v1 shipped (TODO-04/05 in live todos)
vm-10
Minimap follow vs free — no accidental lockstep
Partial — apply_minimap_camera_intent ordering + diagnostics
vm-11
Preview vs main semantics; GPU preview parity (Phase D)
Partial — preview_render_contract, VT-4 (vt_ci_matrix.rs)
proj-viewport-authority
Sweep globals still reading MapCameraDesired
Open sweep
Stage 5 live todos that touch the same spine: TODO-04 (ViewManager vs MapCameraDesired), TODO-05 (no hidden second writer / VM-09B), TODO-11 (fire_view_extract / per-view visible chunks), TODO-12 (GPU preview authoritative).

5. src/gui/map_view and src/render viewport/projection files
src/gui/map_view/ (17 files)
Path	Role
mod.rs
MapViewPlugin, schedules, anti-alias tests
projection/mod.rs
ResolvedMapViewFrames, sync_resolved_map_view_frames
resolved.rs
ResolvedMapViewFrame (extent, texture source, revision)
view_state.rs
MapViewInstances, MapViewState (per-consumer camera, overlays, viewport_size)
presentation/*
Interaction, viewport suggestions, minimap/preview updates, stability
consumers/minimap.rs, consumers/world_preview.rs
Egui texture resolve
texture_cache/mod.rs
Per-MapViewInstanceId bindings
backend/mod.rs
MapTextureSource (CPU raster vs GPU RT)
debug/map_fit_validator.rs
Fit validation
src/render/ — viewport / projection spine
File	Role
viewport_pipeline.rs
ResolvedViewports, resolve chain, ViewportPipelineSet
extraction/render_projection_graph.rs
CPU projection graph (fire/logistics/ecology nodes)
extraction/fire_visual_extract.rs
Sim extract → per-view frames → run_render_projection_graph
fire_view_extract.rs
VisibleFireChunkSet, FireVisualFramesByView per ViewId
fire_chunk_runtime.rs
Active/visible chunk sets
tile_world_fallback.rs
Shared CPU world/minimap raster; sync after view authority
visual_diagnostics.rs, debug_viewport_overlay.rs
Diagnostics
full_render_diagnostic.rs, stage5_full_app_harness.rs
FULL_APP probes
vt_ci_matrix.rs
VT-4/5 surface parity (MinimapOverlay, WorldPreview)
view_representation_snapshot.rs (in gui)
Frame snapshot for GPU preview + harness
No ViewContext type found in the repo; search terms to use instead: ViewManager, MapViewInstances, ViewRepresentationSnapshot, ActiveMapViewInput.

6. recovery_viewport.md — key claims
Diagnosis: subsystem is “CRITICAL REFACTOR REQUIRED” — fragmented authorities, temp scaffolding, debug logic entangled with production, unclear camera lifecycle.

Failure patterns documented:

Multiple viewport authorities (semantic, measured, frozen, rescue, minimap, desired, …).
Migration layers never removed (deprecated solvers, compat adapters).
Debug systems became core logic.
Camera + viewport coupling undefined → jitter / order fragility.
Target architecture: INPUT → ViewportRequest events → single resolver → one commit → render extraction; debug read-only only.

Migration status (2026-05): merge_measured_with_solver / solve_sim_viewport_from_map_fill / solve_viewport_rescue_floor removed; canonical path semantic_viewport_from_map_fill → commit_authority_from_semantic → publish_simulation_map_viewport; drift witness debug_runs/viewport_drift.json, SIM_VIEW_SYNC_DEBUG=1.

Proposed (not fully landed) directory: src/viewport/{authority,arbitration,commit,debug,...} — still largely under src/gui/ + src/render/viewport_pipeline.rs.

7. Sync / shim / scaffold / repair systems
System	File	Role
sync_view_manager_bridge
view_authority.rs
Rebuilds all ViewManager entries from ResolvedViewports + MapViewInstances + MapCameraDesired
mirror_map_camera_desired_to_world_main
view_authority.rs
Shim: desired → WorldMain before bridge
sync_resolved_map_view_frames
map_view/projection/mod.rs
Maps resolved viewports → per-consumer frames; records viewport_sync_ms
sync_editor_viewport_from_resolved
viewport_pipeline.rs
Preview panel size from resolved contract
sync_map_follow_from_game_camera
tile_world_fallback.rs
After SyncViewManager
sim_view_sync_debug
hud/sim_view_sync_debug.rs
Window vs sim-hole vs scissor vs ortho trace (SIM_VIEW_SYNC_DEBUG)
viewport_authority_debug
hud/viewport_authority_debug.rs
Authority + drift + integrity (VIEWPORT_AUTHORITY_DEBUG=1)
ViewportIntegrityAssertPlugin
same
Replaces empty debug plugin
frozen_exceeds_semantic_authority
viewport_layout_solver.rs
Heal hud_root overshoot
ScaffoldContract
representation_governance.rs
Stage 5 transitional scaffold metadata (not viewport-specific)
apply_minimap_camera_intent
view_representation.rs
Can write global MapCameraDesired (infrastructure risk)
Orchestrator
tools/orchestrator/runbooks/viewport_pipeline.md
Witness viewport_authority_migration_witness.json
There is no centralized viewport_authority_resolver event bus yet (recovery doc’s aspirational design).

8. RenderProjectionGraph + RepresentationResult with views
RepresentationResult (representation_policy.rs)
Built from world representation frame + camera visual state; global resource.
Governs extract caps, overlay policy, GPU budget, particle instancing — not keyed by ViewId today.
Consumers: run_render_projection_graph, gpu_particle_draw, gpu_fire_particle_raster, stage5_readiness, atmosphere partial GPU, HUD metrics in view_representation.rs.
RenderProjectionGraph (render_projection_graph.rs)
Single graph per frame; RenderProjectionContext holds one RepresentationResult + one FireVisualFrame (tactical).
Domains: fire, logistics, ecology (TODO-03 wants all three on graph).
Per-view fire: upstream FireVisualFramesByView + VisibleFireChunkSet (from ViewInstance::visible_world_rect); projection graph still uses tactical_fire_visual → ViewId::WorldMain for GPU path.
Schedule: FireVisualFrameSet::BuildProfiles after ViewAuthoritySystemSet::SyncViewManager.
View coupling gap (important)
Per-view: fire visibility extraction, map camera pose, overlay masks, resolved extents.
Still global: representation band, projection graph evaluation, shared overlay buffers, particle dispatch policy.
Closing VM/fire backlog means widening per-view projection without duplicating RepresentationResult writers (representation_spine_audit.rs guards this).

Authority violation patterns (observed / documented)
Dual write on WorldMain camera — MapCameraDesired + ViewManager; writers must call mirror_*; TODO-04/05 / VM-09B track drift (stage5_map_camera_bridge witnesses).
Minimap shell → main camera — apply_minimap_camera_intent mutates MapCameraDesired for focus jumps (can look like bleed if mis-ordered).
SimulationMap ≡ WorldMain — intentional share of camera entity/pose; ViewIsolationDiagnostics.simulation_map_shares_main_camera always true.
Lockstep heuristics — minimap/preview camera matching main while not in follow mode (minimap_main_lockstep_suspect, preview_main_lockstep_suspect).
Revision coupling — fixed in map_view projection (preview/minimap revisions decoupled from resolved.revision churn); still watch ViewportPresentationMismatch.
Global projection for GPU fire — tactical path ignores per-view frames in RenderProjectionGraph (VM/fire-extract debt).
Construction overlays — bypass ViewId; use main camera + SimulationMapViewport only.
Multiple trace targets — viewport_authority, sim_view_sync, map_camera_desired::write — good for debug, bad if treated as authority.
Shared mutable state hotspots
Resource	Writers (multiple)	Readers
MapCameraDesired
RTS input, minimap intent, tile_world_fallback focus, weather?, diagnostics
Bridge, Bevy camera smooth, many HUD paths
ViewManager
sync_view_manager_bridge only (each frame rebuild)
Fire extract, projection helpers, readiness, diagnostics
MapViewInstances
Presentation/interaction systems, sync_editor_viewport_from_resolved
Bridge, map_view consumers
ResolvedViewports
viewport_pipeline resolve chain
Map frames, snapshot, GPU preview, readiness
SimulationMapViewport / AuthoritativeViewport
authoritative_viewport measure/commit
map_camera scissor, resolve primary/sim
ViewportAuthority.pending
UI submitters
Cleared each resolve
RepresentationResult
World representation / domain merge
Entire render spine
RenderProjectionGraph
run_render_projection_graph
GPU upload, weather field, readiness
SharedOverlayFieldBuffers
Fire overlay sync
Minimap/preview/world raster tint
MinimapShellState
HUD shell + presentation
Viewport resolve, minimap texture
Recommended module boundaries
viewport/                          # NEW (per recovery_viewport.md)
  request.rs                       # ViewportRequest events from UI
  semantic.rs                      # measure sim_map_fill → SemanticViewportRect
  resolve.rs                       # → ResolvedViewports (move from render/)
  debug.rs                         # trace only (viewport_authority + sim_view_sync)
view/
  authority.rs                     # ViewId, ViewManager, bridge, isolation diagnostics
  projection.rs                    # view_projection_authority helpers
  representation.rs                # CameraVisualState, budgets, snapshot
map_view/                          # KEEP — presentation-only
  consumers/                       # egui bind; no camera authority
  projection/                      # ResolvedMapViewFrames from resolved + snapshot
render/
  contracts/                       # ResolvedViewports, ViewRepresentationSnapshot
  extraction/                      # fire sim, per-view frames, projection graph
  surfaces/                        # preview GPU, tile fallback, minimap raster
construction/                      # KEEP separate — ghosts never write ViewManager
Rules to enforce:

UI emits requests; only viewport_pipeline + authoritative_viewport commit rects.
Only map_camera (or future commit_viewport_to_camera) mutates Bevy camera viewport/scissor.
Gameplay reads camera_translation(id) not raw MapCameraDesired unless in the RTS input shim.
RepresentationResult: one builder; per-view only via PerViewLodHints / future per-view policy tables.
Construction stays on SimulationMapViewport + confirm funnel until explicitly multiview-aware.
Quick reference — primary file paths
Topic	Path
ViewManager / ViewId
c:\dev\github\Rust_engine_template_01\src\gui\view_authority.rs
View projection API
c:\dev\github\Rust_engine_template_01\src\gui\view_projection_authority.rs
Viewport requests
c:\dev\github\Rust_engine_template_01\src\gui\viewport_authority.rs
Resolved viewports
c:\dev\github\Rust_engine_template_01\src\render\viewport_pipeline.rs
Semantic sim-map
c:\dev\github\Rust_engine_template_01\src\gui\authoritative_viewport.rs
Map camera / WorldMain
c:\dev\github\Rust_engine_template_01\src\gui\map_camera.rs
Map view spine
c:\dev\github\Rust_engine_template_01\src\gui\map_view\
Preview GPU
c:\dev\github\Rust_engine_template_01\src\gui\editor\world_preview\gpu_preview.rs
Preview contract
c:\dev\github\Rust_engine_template_01\src\gui\editor\world_preview\preview_render_contract.rs
Representation
c:\dev\github\Rust_engine_template_01\src\gui\representation_policy.rs
Projection graph
c:\dev\github\Rust_engine_template_01\src\render\extraction\render_projection_graph.rs
Per-view fire
c:\dev\github\Rust_engine_template_01\src\render\fire_view_extract.rs
VM backlog narrative
c:\dev\github\Rust_engine_template_01\prompts\guides\base_finsh_5.md
Recovery plan
c:\dev\github\Rust_engine_template_01\src\dev\recovery_viewport.md
Live FULL_APP proof
c:\dev\github\Rust_engine_template_01\debug_runs\stage5_full_app_live.json