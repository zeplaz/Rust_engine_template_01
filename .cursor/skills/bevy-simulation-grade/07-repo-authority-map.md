# Repo Authority Map — `Rust_engine_template_01`

**Authoritative for scheduling and writers.** Bevy **0.20** (migrated from 0.19 — PLAN-BEVY-020-MIG-v1). Update this file when spine ownership changes.

**0.20 note:** several built-in schedules order SystemSets **weakly**. Prefer ECS-visible `Res`/`ResMut` edges; where systems communicate via interior mutability / channels / atomics, restore **strict** `.before` / `.after` on named sets (see migration guide “weakly ordered sets”).

## Four-layer graph (Slice A — 2026-11-14)

```text
⊚Sim ═▶ ⊚View ═▶ ⊚Render ═▶ ⊚UI
  AdvanceSimTick → AttentionSet::Build → ChunkEnvironment (Lod→Weather→Ecology→Fire)
    → Transport / Strategic / Logistics / Combat / MFG
  ViewRepresentation / MapCamera / ViewAuthority → sole ViewManager
  FireVisualFrame / atmos extract / world representation (read snaps)
  HUD / picker / overlays (read snaps · intent ingress only)
  AudioMixSet::Build (after SyncViewManager — presentation; never writes sim)
```

### Dual-writer risk list (watch)

| Surface | Risk | Mitigation |
|:---|:---|:---|
| `SimAttentionFrame` | Everyone pushes interest | **Sole** `build_sim_attention_frame` |
| `AudioMixFrame` | Sim systems play SFX directly | Enqueue `AudioOneShotQueue` only; mix drains |
| `ViewManager` | Parallel rebuilds | `sync_view_manager_bridge` only |
| `MissionBook` | UI / AI dual apply | `MissionIntentQueue` → drone step sole apply |
| `LogisticsOrderSim` | Panel mutates sim | `LogisticsOrderRequest` ingress only |
| `ChunkSimLod` | Attention overwrites fire Active | Attention may **upgrade** Dormant→Normal only |

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
  SimControlSystemSet::AdvanceSimTick
    → AttentionSet::Build → ResMut<SimAttentionFrame>
    → ChunkEnvironmentSet::Lod (reads attention RO) → Weather → Ecology → Fire
    → Transport / Strategic / Logistics / Combat / MFG
  (avoid orphan .after global camera)

PostUpdate (construction — after camera scissor applied)
  SimulationViewportSyncSet::ApplyCameraScissor
    └─ build_pick_ghost_tile_system → footprint sync → placement_debug probe
       (.after ApplyCameraScissor — pick uses synced viewport + ortho trace)
```

## Sim spine (verified 2026-07-03 sweep — SCH-D1; full audit: `src/dev/plan_schedule_sync_v1.md`)

```text
Update (simulation — runs beside the view spine above)
  SimControlSystemSet::ApplyOperatorInput → AdvanceSimTick        (sim_control.rs; pause gate = should_tick()/dt_scale())
    ├─ AttentionSet::Build → SimAttentionFrame (sim_attention.rs; sole writer)
    ├─ ChunkEnvironmentSet: Lod → Weather → Ecology(⊃LandscapeBurnSet) → Fire   (chunk_environment_set.rs)
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
| `AttentionSet` | `src/systems/sim_attention.rs` | Build `SimAttentionFrame` after AdvanceSimTick; before Lod |
| `AudioMixSet` | `src/audio/mix_frame.rs` | Build `AudioMixFrame` after `SyncViewManager` |

## Single writers (production)

| Resource / surface | Sole writer (production) | Notes |
|--------------------|--------------------------|-------|
| `SimAttentionFrame` | `build_sim_attention_frame` | Slice A3 — RO consumers only after Build |
| `AudioMixFrame` | `build_audio_mix_frame` | Slice C1 — presentation; never writes sim |
| `AudioOneShotQueue` | UI / combat emitters enqueue; mix drains | Never play audio from sim systems directly |
| `BallisticFlightBook` | `step_ballistic_flights` (+ launches from `apply_missile_sortie_launches_system`) | Combat; impacts enqueue `BlastIntent` |
| `MissileSortieLaunchQueue` | `step_drone_missions_system` enqueues; `apply_missile_sortie_launches_system` drains | Keeps mission step under Bevy 0.20 SystemParam arity |
| `BlastIntentQueue` / `BlastCraterBook` | `execute_blast_intents_system` | Construction sole demolish/crater |
| `BlastCraterDump` | `publish_blast_crater_dump` | Presentation/score RO mirror |
| `TruckPresentation` entities | `sync_truck_presentation` | Presentation; reads order-sim RO |
| Missile catalog plants (`missile_*`) | building JSON + `activate_industrial_facilities_system` → MFG | Discrete MFG; no ResourceFlow; picker via `industrial_menu` |

### FIDELITY-1 attention consumers (RO `SimAttentionFrame`)

| Consumer | Gate |
|:---|:---|
| `step_drone_missions_system` / missile pulse | Skip sortie pulse for `AttentionTier::Dormant` factions |
| `faction_ai_tick_system` | Dormant defer; Aggregate stretches `decision_interval` |
| `score_pressure_points_system` | Cheap / skip when faction slot Dormant |
| `step_logistics_orders_system` | Cadence: Full every step; Aggregate every 2nd; Dormant skip tick (requests still apply) |
| `auto_supply_unserved_system` | Interval ×2 when player theater inactive |
| `tick_manufacturing_nodes` | Throughput scale from player-theater attention |
| `run_compute_dispatch_graph` / heat | Truncate eligible rows to `budget_path_units` |
| `build_audio_mix_frame` | Spatial from trucks when zoomed + player theater |
| `ViewManager` | `sync_view_manager_bridge` | `src/gui/view_authority.rs` — rebuild read model from authority |
| `MapCameraDesired` | `derive_map_camera_desired_from_view_authority` | Compatibility mirror; input commits **`ViewProjectionAuthority`** first |
| `ViewProjectionAuthority` poses | `commit_pose` / `commit_pose_traced` | Map camera input, minimap shell, session hooks |
| `ResolvedViewports` | `ViewportPipelineSet::Resolve` chain | `resolve_*` systems in `viewport_pipeline.rs` |
| `ViewportAuthority.pending` | UI via `submit_viewport_request` | Cleared after resolve |
| `ViewRepresentationSnapshot` | `build_view_representation_snapshot` | After `SyncViewManager` |
| `AtmosphereClipmapStack` L0 | FieldFill/WindAdvect (+ `sync_atmos_clipmap_witness_system`) | **ES-6 / DEBT-011** — smoke+fog+toxicity+ash+ember+heat authority; Field resource deleted; smoke extract bridge `L0→ChunkSmoke→SimChunkSmokeVisualExtract` |
| ~~`AtmosphereField` (128²)~~ | — | **DEBT-011 deleted** — use `atmos_chunk_to_tile` + L0 `sample_tactical_*` |
| `LogisticsGraph` | `sync_logistics_graph_from_transport` (`StrategicFieldPipeline::GraphSync`) | **LOG-A-01** derived cache only; portals via `PortalAttachmentMap`; one-shot play/harness seeds call same `rebuild_logistics_graph_from_transport` then GraphSync replaces |

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
| `TerrainRenderAuthority` | `CpuRaster` (alias `CpuFallback`) · `GpuTilemap` (deferred, never constructed) · `GpuBake` (alias `GpuInstancedAtlas`) |
| `resolve_sim_default_authority()` | **Simulation:** `GpuBake` (debug+release); `TERRAIN_CPU_FALLBACK=1` forces `CpuRaster` |
| `TERRAIN_CPU_FALLBACK=1` | Rollback env → force `CpuRaster` |
| `tile_world_fallback.rs` | CPU raster + stamp blit gated when authority uses GPU display |
| `terrain_instanced_draw.rs` | Interim GPU sprite-bake display (dirty-gated) |
| `minimap_compositor/pass.rs` | `minimap_terrain_source_label()` reflects authority in composite witness |
| `visual_perf_budget.rs` | `fire_extract_cadence_due()` — overlay extract cadence gate |

**Witness:** `debug_runs/gpu_terrain_p0c_prime_001_live.json` · `debug_runs/gpu_p1_p2_001_live.json` · harness `src/dev/gpu_terrain_witness.rs` / `gpu_p1_p2_witness.rs`.

**Anti-pattern:** ad-hoc CPU terrain blit in Simulation when `GpuBake` is active — route stamps through `TerrainGpuStampIndices` (`map_tile_atlas_stamp.rs`).

## Staleness (LOOP-07 · E2)

Documented one-tick / cadence boundaries for visual and strategic **read models** (writers in other modules; types in `sim_visual_extract.rs`, `world_read_snapshot.rs`).

| Reader (snapshot) | Source | Lag | Reason |
|-------------------|--------|-----|--------|
| `FireVisualFrame` | `FireSimulationSnapshot` / `extract_fire_simulation_snapshot` | This sim tick, or **previous** when extract cadence skips | Visual-only; `fire_projection_stamp_aligned` permits ≤1 tick vs `CommittedVisualSnapshotFence` |
| `SimFireEmitterVisualExtract` | Tactical `FireVisualFrame` in `FireVisualFramesByView` | Same as parent fire frame | Legacy CPU/debug mirror; GPU uses `RenderProjectionGraph` |
| `SimChunkSmokeVisualExtract` | `ChunkSmokeField` / substrate contamination | **This sim tick** after `ChunkEnvironmentSet::Fire` | Fog/volume presentation; render extract ≤1 render frame, not an extra sim step |
| `ClimateVisualAggregate` | `ChunkWeather`, `ChunkEcology`, `LandscapeProgramOnChunk` | **This sim tick** after environment weather/ecology | World-mean uniforms; no gameplay write-back |
| `WorldReadSnapshot` | `ChunkStrategicOverlay`, `FrontlineState`, `TransportEdgeDirectory` | **This frame** on refresh; else **up to 11 display frames**; transport topology may be **1 frame** behind network flow | Planner/HUD digest only; chained after frontline derive in `ZoneAndReadModel` |

## Escalation

| Change touches | Read also |
|----------------|-----------|
| New view or surface | `view_authority.rs`, `view_representation.rs` |
| HUD viewport submit | `viewport_authority.rs`, `ui_layout_agent` playbook |
| Fire / overlay extract | `fire_visual_extract.rs`, Stage 5 convergence guide |
| Construction ghosts / placement pick | [05-construction-ghost-overlay.md](05-construction-ghost-overlay.md), [09-sim-map-projection-placement.md](09-sim-map-projection-placement.md), `construction_invariants.md` |
