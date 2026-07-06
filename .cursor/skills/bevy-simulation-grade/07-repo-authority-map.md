# Repo Authority Map — `Rust_engine_template_01`

**Authoritative for scheduling and writers.** Bevy **0.18**. Update this file when spine ownership changes.

## Update frame order (simplified)

```text
PreUpdate
  sync_visual_cadence_from_visual_budget_settings

Update (view / presentation spine — chained subsets)
  ViewRepresentationSystemSet::UiCollect
  ViewRepresentationSystemSet::ResolveViewport
    ├─ MapCameraSystemSet::ApplyInput        (RTS input → ViewProjectionAuthority)
    ├─ MapCameraSystemSet::DeriveDesired     (authority → MapCameraDesired mirror)
    ├─ MapCameraSystemSet::Smooth
    ├─ apply_minimap_camera_intent           (minimap shell → authority only)
    └─ ViewportPipelineSet::Resolve          → ResMut<ResolvedViewports>
  ViewRepresentationSystemSet::CameraSync
    ├─ ViewAuthoritySystemSet::RegisterViewCameras
    └─ ViewAuthoritySystemSet::SyncViewManager
         └─ sync_view_manager_bridge         → sole ResMut<ViewManager>
         └─ build_view_representation_snapshot
  ViewRepresentationSystemSet::RenderTargets
  ViewRepresentationSystemSet::WorldRender
  ViewRepresentationSystemSet::PostFX
    └─ validate_view_representation_snapshot

Update (fire / extraction — order relative to SyncViewManager)
  FireVisualFrameSet::BuildProfiles          (after SyncViewManager)
  FireVisualFrameSet::BuildClusters … EmitDomainOverlays
  FireVisualFrameSet::ProjectGpu
  ViewRepresentationSystemSet::SyncOverlayField (after BuildProfiles)
  tile / minimap consumers                   (.after BuildProfiles typical)

Simulation / logistics
  (domain SystemSets — chain via planner; avoid orphan .after global camera)

PostUpdate (construction — after camera scissor applied)
  SimulationViewportSyncSet::ApplyCameraScissor
    └─ build_pick_ghost_tile_system → footprint sync → placement_debug probe
       (.after ApplyCameraScissor — pick uses synced viewport + ortho trace)
```

## Sim spine (verified 2026-07-03 sweep — SCH-D1; full audit: `src/dev/plan_schedule_sync_v1.md`)

```text
Update (simulation — runs beside the view spine above)
  SimControlSystemSet::ApplyOperatorInput → AdvanceSimTick        (sim_control.rs:99; pause gate = should_tick()/dt_scale())
    ├─ ChunkEnvironmentSet: Lod → Weather → Ecology(⊃LandscapeBurnSet) → Fire   (chunk_environment_set.rs:26)
    │    └─ Fire → AtmospherePipelineSet::FieldFill → … → VisualExtract → Diagnostics (atmosphere/pipeline.rs:35-49)
    │                                    Diagnostics → TransportSchedule::Topology   (engine_with_worldgen.rs:196-200)
    ├─ SimEffectSystemSet::Drain (after AdvanceSimTick ONLY — ⚠ unordered vs fire tick, SCH-E2)
    ├─ TransportSchedule: Topology → FieldIntegrate → CostCache   (transport/mod.rs:47-49)
    │    ├─ NavSets::DamageSpeedAdjustment → MotionCalculation
    │    ├─ StrategicFieldPipeline::GraphSync … → InfrastructureSiteSet chain (strategic/plugin.rs:196-215)
    │    └─ LogisticsSimulationSet chain → HybridSimPipeline (strategic/sim.rs:230-232)
    └─ MapCameraSystemSet … (view spine above) → FireVisualFrameSet::BuildProfiles
         → WorldRepresentationSystemSet::ComputeFrame → ComputeDispatchSystemSet::Dispatch
         → FireVisualFrameSet::ProjectGpu → LocalLightExtractSet::Collect (engine:186-238,303-314)
```

Known gaps (do not "fix" ad-hoc — plan items): SCH-E2 effect-drain order · SCH-E3 fire→extract edge is
transitive-only via atmosphere · SCH-T1 strategic agents ignore pause · 233 bare-fn `.after` anchors (SCH-A1).
No FixedUpdate anywhere: sim is dt-scaled Update (frame-rate consistent, not independent — SCH-T5 ADR).

## SystemSet reference

| SystemSet | File | Role |
|-----------|------|------|
| `ViewRepresentationSystemSet` | `src/gui/view_representation.rs` | UI collect → viewport → camera → render targets → world render → post FX → overlay sync |
| `ViewportPipelineSet::Resolve` | `src/render/viewport_pipeline.rs` | Nested in `ResolveViewport`; writes **`ResolvedViewports`** |
| `MapCameraSystemSet` | `src/gui/map_camera.rs` | Input → derive desired → smooth; **before** `SyncViewManager` |
| `ViewAuthoritySystemSet` | `src/gui/view_authority.rs` | `RegisterViewCameras` → **`SyncViewManager`** |
| `FireVisualFrameSet` | `src/render/extraction/fire_visual_extract.rs` | Sim scan → clusters → GPU project; **after** view bridge |

## Single writers (production)

| Resource / surface | Sole writer (production) | Notes |
|--------------------|--------------------------|-------|
| `ViewManager` | `sync_view_manager_bridge` | `src/gui/view_authority.rs` — rebuild read model from authority |
| `MapCameraDesired` | `derive_map_camera_desired_from_view_authority` | Compatibility mirror; input commits **`ViewProjectionAuthority`** first |
| `ViewProjectionAuthority` poses | `commit_pose` / `commit_pose_traced` | Map camera input, minimap shell, session hooks |
| `ResolvedViewports` | `ViewportPipelineSet::Resolve` chain | `resolve_*` systems in `viewport_pipeline.rs` |
| `ViewportAuthority.pending` | UI via `submit_viewport_request` | Cleared after resolve |
| `ViewRepresentationSnapshot` | `build_view_representation_snapshot` | After `SyncViewManager` |

**Test-only partial sync:** `sync_view_manager_world_main_from_authority` — not scheduled in production.

## Core types (repo)

```rust
// src/gui/view_authority.rs
pub enum ViewId { WorldMain, WorldPreview, Minimap, SimulationMap }

// src/gui/viewport_authority.rs — requests (not final rects)
pub struct ViewportAuthority { pending, requested, resolved, revision }

// src/render/viewport_pipeline.rs — committed layout
pub struct ResolvedViewports {
    world_preview, minimap_panel, simulation_map, primary_window, revision
}

// src/render/view_runtime.rs (re-exported)
pub struct ViewProjectionAuthority { /* commit_pose per ViewSurfaceId */ }
```

## Correct dependency edges

When adding a system, prefer these anchors:

```rust
// After view bridge + snapshot exist
.after(ViewAuthoritySystemSet::SyncViewManager)

// After fire profiles built
.after(FireVisualFrameSet::BuildProfiles)

// Before bridge reads camera mirror
.before(ViewAuthoritySystemSet::SyncViewManager)

// Inside viewport resolve phase
.in_set(ViewportPipelineSet::Resolve)
```

**Anti-pattern:** `.after(sync_map_camera)` ad-hoc strings — use **SystemSet** names from this table.

## VM / migration notes (read code comments)

| Tag | Meaning |
|-----|---------|
| **VM-06** | `ViewManager` bridge sole writer |
| **VM-09** | `MapCameraDesired` mirror; `SyncViewManager` after map camera |
| **TRIAGE-VM-09-v2** | Input → `ViewProjectionAuthority` first |
| Minimap shell | `apply_minimap_camera_intent` — **no** `MapCameraDesired` writes |

## Diagnostics

| Witness / resource | When updated |
|--------------------|--------------|
| `ViewIsolationDiagnostics` | With `sync_view_manager_bridge` |
| `ViewportPresentationMismatch` | During viewport resolve |
| `debug_runs/stage5_full_app_live.json` | Stage 5 readiness harness |
| `ConstructionPlacementDebugProbe` | PostUpdate after camera scissor; `--test vfx` overlay |
| `MainWorldCameraOrthoTrace` | `sync_main_world_camera_viewport_and_projection` |

## GPU terrain render authority (P0-C′ / P2-D)

Simulation terrain display authority lives in `src/render/terrain_render_authority.rs` — **not** a parallel extraction path.

| Resource / fn | Role |
|---------------|------|
| `TerrainRenderAuthority` | `CpuFallback` · `GpuTilemap` (blocked) · `GpuInstancedAtlas` |
| `resolve_sim_default_authority()` | **Release Simulation:** `GpuInstancedAtlas`; **Debug:** `CpuFallback` unless `TERRAIN_GPU_INSTANCED=1` |
| `TERRAIN_CPU_FALLBACK=1` | Rollback env → force `CpuFallback` |
| `tile_world_fallback.rs` | CPU raster + stamp blit gated when authority uses GPU display |
| `terrain_instanced_draw.rs` | Interim GPU sprite-bake display (dirty-gated) |
| `minimap_compositor/pass.rs` | `minimap_terrain_source_label()` reflects authority in composite witness |
| `visual_perf_budget.rs` | `fire_extract_cadence_due()` — overlay extract cadence gate |

**Witness:** `debug_runs/gpu_terrain_p0c_prime_001_live.json` · `debug_runs/gpu_p1_p2_001_live.json` · harness `src/dev/gpu_terrain_witness.rs` / `gpu_p1_p2_witness.rs`.

**Anti-pattern:** ad-hoc CPU terrain blit in Simulation when `GpuInstancedAtlas` is active — route stamps through `TerrainGpuStampIndices` (`map_tile_atlas_stamp.rs`).

## Escalation

| Change touches | Read also |
|----------------|-----------|
| New view or surface | `view_authority.rs`, `view_representation.rs` |
| HUD viewport submit | `viewport_authority.rs`, `ui_layout_agent` playbook |
| Fire / overlay extract | `fire_visual_extract.rs`, Stage 5 convergence guide |
| Construction ghosts / placement pick | [05-construction-ghost-overlay.md](05-construction-ghost-overlay.md), [09-sim-map-projection-placement.md](09-sim-map-projection-placement.md), `construction_invariants.md` |
