# PLAN-GPU-TERRAIN-PRODUCTION-EXEC-001 — Retire CPU fallback · strip render graph · ship perf `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-GPU-TERRAIN-EXEC-001** |
| **Artifact** | `plan_gpu_terrain_production_exec_001_v1.md` |
| **Supersedes** | [`plan_visual_perf_production_v1.md`](plan_visual_perf_production_v1.md) Phase 5 (“longer-term GPU terrain”) — **now in scope** |
| **Parent** | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) Phases 1–4 (budget/diag — partial) |
| **Index** | [`development_plan_index.md`](development_plan_index.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-07-02 |
| **Owner** | `@planner` → **`@coder`** (render + gui + terrain) |
| **Status** | **ACTIVE — single-lane; no partial sign-off** |
| **Deferrals** | [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-MIG-TILEMAP** / **DR-GPU-TERRAIN-P0C** block P0-C tilemap default until crates.io 0.19 |

---

## 0. Ship rule (read first)

**Do not report this program “done” until every row in §10 Program exit gate is green.**

Partial phase completion is tracked internally but **not** communicated to operators as success. Diagnostics-only work does not close this program.

**Target:** `--test demo` and `--test visual` on a **320×320** world, **release** build, **no** `RASTER_*` / `PERF_*` env overrides:

| Metric | Gate |
|:---|:---|
| p95 frame time | **≤ 33 ms** (60 s window) |
| `spine.tile_raster_ms` | **== 0** in Simulation default path |
| `render_schedule.render_and_present_ms` p95 | **≤ 16 ms** |
| `update_attrib.fire_pipeline_ms` p95 | **≤ 8 ms** |
| Stall mismatch (`attribution_honesty.stall_checkpoint_mismatch`) | **false** on steady-state frames |

Proof bundle: `debug_runs/sim_spectrum_analytics_live.json` + `debug_runs/stage5_full_app_live.json` + `debug_runs/minimap_compositor_live.json`.

---

## 1. Summary (architectural stance)

The lag is **structural**: default Simulation still uses **CPU `tile_world_fallback`** (RGBA paint + GPU upload) as the main map authority, while production GPU lanes (tilemap adapter, atlas stamps, minimap compositor, instanced debug tiles) were built **beside** it, not **instead** of it.

**Target architecture:**

```text
ChunkCellMatrix / MaterializedChunk  ──▶  GPU terrain (tilemap OR instanced atlas)  ──▶  MainWorldCamera
                                              │
                                              ├─ dirty tile indices only (no CPU RGBA upload)
                                              └─ minimap reads downsampled GPU snapshot / shared atlas

CPU tile_world_fallback  ──▶  editor / witness / explicit DEV flag ONLY

Fire / projection / particles  ──▶  early-out when inst=0 or overlay matrix off

WorldRepresentation / overlays  ──▶  dirty-revision gates, not every sim tick
```

**Single terrain authority:** `TerrainRenderAuthority` resource — one enum, one commit path per frame. No parallel CPU+GPU main-map truth.

---

## 2. Current problems (honest)

| Problem | Evidence | Why it hurts |
|:---|:---|:---|
| CPU fallback is **default** | `TileWorldFallbackPlugin` always registered; `bevy_tilemap_adapter` optional/off | Wrong job for a tile map; PCIe upload + full render graph for one sprite |
| Tilemap adapter **ECS-only** | `Cargo.toml` `default-features = false`; no GPU draw | Feature flip alone changes nothing visible |
| Atlas stamps **CPU blit** | `map_tile_atlas_stamp.rs` → fallback RGBA | Building art path fights GPU terrain |
| Full render graph with **zero work** | Particle/spark dispatch scaffolding; projection graph on empty overlays | GPU bubbles + prepare cost |
| Monolithic Update chain | Fire build → world repr → streaming in one breath | Coupling (CPU cost is small today; blocks future decoupling) |
| Diagnostic sprawl | Custom stall + render spans without Tracy | Necessary triage; not a substitute for fixes |

---

## 3. Authority map (must not violate)

| Domain | Authority | Consumers | DO NOT |
|:---|:---|:---|:---|
| **Terrain pixels (sim)** | `TerrainRenderAuthority` → GPU path | Main camera, minimap compositor terrain layer | CPU fallback + GPU tilemap both active |
| **Chunk material indices** | `MaterializedChunk` / `MaterialRegistry` | GPU tilemap UV or instanced atlas | Re-scan `TileMarker` spam in chunk-authoritative mode |
| **Minimap widget** | GPU compositor RT (`MinimapRenderTargetRegistry`) | Bevy UI `ImageNode` | Second 3D camera; CPU `minimap_image` in sim default |
| **Fire particles** | `WorldFireParticleFrame` / projection graph | Tactical views only | Minimap / strategic views |
| **Overlays** | `SharedOverlayFieldBuffers` + `RepresentationResult.overlay_matrix` | Raster/compositor when toggles on | Full ECS scan when all overlays off |
| **View layout** | `ViewManager` + `ResolvedViewports` | All surfaces | Ad-hoc viewport writes |

Read first: [`.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md`](../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md).

---

## 4. DO NOT TOUCH (coder)

- Stage 5 / Stage 6 **closure witnesses** — extend fields, do not weaken gates.
- `RepresentationResult` schema — add fields only via existing merge paths.
- Construction execute funnel / `src/construction/` invariants.
- MCP art pipeline (`tools/mcp/`) — consume atlases, do not redesign bake.
- `FireVisualFrameSet` ordering vs `MapCameraSystemSet::Smooth` (already fixed).
- Witness JSON **filenames** under `debug_runs/` (append fields OK).

---

## 5. Implementation phases (strict DAG)

Each phase **blocks** the next. **Rollback trigger:** any Phase N gate red for 2 consecutive CI runs → revert Phase N only, fix forward.

---

### Phase P0 — GPU terrain default (BLOCKER) · ~5–8 days

**Goal:** Simulation entry never CPU-paints main map terrain.

#### P0-A — Terrain render authority

| Task | Files | Detail |
|:---|:---|:---|
| **P0-A1** | `src/render/terrain_render_authority.rs` (new), `render/mod.rs` | `TerrainRenderAuthority` enum: `CpuFallback`, `GpuTilemap`, `GpuInstancedAtlas`. Resource + `TerrainRenderAuthorityPlugin`. |
| **P0-A2** | `src/engine/engine_with_worldgen.rs` | `OnEnter(Simulation)`: set `GpuTilemap` when adapter+atlas ready, else `GpuInstancedAtlas` interim, else `CpuFallback` only under `cfg(debug_assertions)` + explicit env. |
| **P0-A3** | `src/render/tile_world_fallback.rs` | `tile_world_fallback_sync_spawner` + `tile_world_fallback_rasterize`: **no-op main map** when authority ≠ `CpuFallback`. Keep editor/witness path. |
| **P0-A4** | Unit tests | Authority gating: sim + GpuTilemap ⇒ `tile_raster_ran == false`. |

**Exit P0-A:** `cargo test -p proc_A_dine01 --lib terrain_render_authority` green; sim enter sets non-CPU authority in release default build.

#### P0-B — Material terrain atlas (single GPU texture)

| Task | Files | Detail |
|:---|:---|:---|
| **P0-B1** | `src/render/terrain_material_atlas.rs` (new) | Build **one** RGBA atlas from `MaterialRegistry` family colors (+ default biomes). Handle: `TerrainMaterialAtlasGpu`. Deterministic layout (stable sort by material id). |
| **P0-B2** | `src/systems/terrain/material_plugin.rs` | After registry load, (re)build atlas; bump `revision` on registry hot reload. |
| **P0-B3** | Tests | Atlas dimension > 0; UV lookup round-trip for sample materials. |

**Exit P0-B:** Atlas image exists in `Assets<Image>`; no per-tile PNG handles for base terrain.

#### P0-C — GPU tilemap render path (preferred production path)

**BLOCKED — DR-MIG-TILEMAP / DR-GPU-TERRAIN-P0C:** `bevy_ecs_tilemap` crates.io latest is **0.18.1** (Bevy 0.18). Do **not** enable `bevy_tilemap_adapter` in default features until upstream 0.19 + compat witness. **Pick P0-C′ instanced path** or P0-A/B while blocked. Re-evaluate when `defer_registry.json` → `DR-MIG-TILEMAP` predicates all true.

| Task | Files | Detail |
|:---|:---|:---|
| **P0-C1** | `Cargo.toml` | **WAIT** — Enable `bevy_ecs_tilemap` render features + `bevy_tilemap_adapter` default only after **DR-MIG-TILEMAP** unblock. Until then: feature stays OFF. |
| **P0-C2** | `src/render/tilemap_adapter.rs` | Bind `TilemapTexture` to `TerrainMaterialAtlasGpu` handle. Map `MaterializedChunk.materials` → `TileTextureIndex` (already partial). |
| **P0-C3** | `src/render/tilemap_render_plugin.rs` (new) | Camera-visible tilemap layer on `MainWorldCamera`; chunk world offset from `Chunk` transform. |
| **P0-C4** | Hydration | On `DenseTerrainHydrationGate` complete + chunk slabs: ensure `spawn_chunk_tilemaps` runs before sim HUD visible. |
| **P0-C5** | CI | `.github/workflows/ci.yml` — default job builds **with** tilemap adapter render. |

**Exit P0-C:** Visual: main map visible with **`tile_world_fallback` sprite despawned**; `ChunkTilemaps` populated; **zero** `tile_world_fallback_rasterize` main pass.

**Fallback if P0-C blocked (bevy_ecs_tilemap render API):** P0-C′ instanced atlas terrain (extend pattern from `gpu_tile_debug_draw.rs`):

| Task | Files | Detail |
|:---|:---|:---|
| **P0-C′1** | `src/render/terrain_instanced_draw.rs` (new) | Storage buffer: per-chunk or per-tile instance { world_xy, atlas_uv, material_index }. |
| **P0-C′2** | `assets/shaders/terrain/terrain_instanced.wgsl` (new) | Sample `TerrainMaterialAtlasGpu`; one draw per chunk or one global draw. |
| **P0-C′3** | Register in `RenderApp` like `register_tile_debug_instanced_draw`. |

Use C′ only if C fails after 1 day — document choice in plan footer.

#### P0-D — Building stamps off CPU blit

| Task | Files | Detail |
|:---|:---|:---|
| **P0-D1** | `src/gui/map_tile_atlas_stamp.rs` | Dual path: when `TerrainRenderAuthority::Gpu*`, write **tile variant indices** into tilemap layer (or instanced overlay), not `apply_atlas_stamps_to_rgba_subregion`. |
| **P0-D2** | `src/construction/procedural/tile_atlas_index.rs` | Map variant_key → atlas tile index (existing registry). |
| **P0-D3** | Tests | Stamp request updates tilemap index, not CPU buffer. |

**Exit P0-D:** Construction sites visible on GPU terrain; CPU stamp blit **not** called in sim default.

#### P0-E — Minimap terrain decouple

| Task | Files | Detail |
|:---|:---|:---|
| **P0-E1** | `src/render/minimap_compositor/pass.rs` | Terrain source: GPU downsample pass OR shared atlas sample — **not** `fallback.image` CPU repaint dependency. |
| **P0-E2** | `src/render/minimap_compositor/composite.rs` | Refresh terrain layer on cadence + dirty revision only. |
| **P0-E3** | Witness | `minimap_compositor_live.json`: `terrain_source: gpu_atlas` (new field). |

**Exit P0-E:** Minimap compositor works with `cpu_minimap_pass: false` and `tile_raster_ms: 0`.

#### Phase P0 gate (ALL required)

```powershell
cargo test -p proc_A_dine01 --lib terrain_render tilemap terrain_instanced minimap_shell --release
cargo run -p proc_A_dine01 --release -- --test demo --stay-open
# 60s: sim_spectrum last_frame.spine.tile_raster_ms == 0
# render_schedule.render_and_present_ms p95 <= 16ms (initial baseline — may need P1)
```

---

### Phase P1 — Strip dead render work · ~2–3 days

**Goal:** Render thread does only work that has visible instances.

| ID | Task | Files | Exit |
|:---|:---|:---|:---|
| **P1-A** | Fire particle compute **zero skip** | `gpu_particle_draw.rs`, `gpu_spark_compute.rs`, `gpu_fire_particle_raster.rs` | When `instance_count == 0`: no bind group rebuild, **no** `dispatch_workgroups`, node returns immediately. Witness: `dispatch_count == 0`. |
| **P1-B** | Water/weather particle same | `gpu_water_particle_draw.rs`, `gpu_weather_fire_field.rs` | Same early-out contract. |
| **P1-C** | Projection graph overlay skip | `extraction/render_projection_graph.rs` | If `RepresentationResult.overlay_matrix` all-false **and** no fire instances: skip graph evaluate; stamp-only fence (existing pattern ~L327). |
| **P1-D** | Fire extract cadence (replace spike-only) | `extraction/fire_visual_extract.rs` | Full scan on sim tick **or** overlay revision **or** residency dirty — not every frame. Remove sole reliance on `UxFrameSpikeGuard` skip. |
| **P1-E** | Minimap overlay default | `minimap_shell.rs`, `test_harness.rs` | Demo/default: `simulation_minimap_overlay_defaults()` — not witness harness mask. Visual `--test visual` keeps harness via `full_capture_active()` only. |

**Phase P1 gate:**

```powershell
cargo test -p proc_A_dine01 --lib fire_view_extract gpu_particle render_projection_graph --release
# sim_spectrum: fire_extract.extract_ms p95 < 1ms when no fire active
# render_schedule.prepare_ms drop vs P0 baseline (record in witness)
```

---

### Phase P2 — Dirty-gate representation · ~2 days

**Goal:** Stop doing representation work when inputs unchanged.

| ID | Task | Files | Exit |
|:---|:---|:---|
| **P2-A** | `WorldRepresentationFrame` dirty gate | `gui/world_representation.rs` | Fingerprint: camera zoom band, viewport revision, sim tick bucket. Skip `ComputeFrame` when unchanged. |
| **P2-B** | Overlay buffer push gate | `overlay_field_buffers.rs`, `fire_visual_extract.rs` | Push heat buffers only when fire chunks dirty or overlay toggle on. |
| **P2-C** | Streaming reconstruct gate | `io/streaming/mod.rs` | Already partial — ensure not called on idle frames (witness `streaming_reconstruct_ms` p95 < 5ms idle). |
| **P2-D** | Schedule documentation | `src/dev/plan_gpu_terrain_production_exec_001_v1.md` §7 | Update authority map if new sets added. |

**Phase P2 gate:** 60s idle sim (no camera move): `upd_world_repr_frame` perf_scope **not** in top 8 scopes.

---

### Phase P3 — Profiling consolidation · ~1 day

**Goal:** Keep CI witness, drop duplicate human-facing noise; optional Tracy for deep dives.

| ID | Task | Files | Exit |
|:---|:---|:---|
| **P3-A** | Tracy optional feature | `Cargo.toml`, `src/dev/tracy_integration.md` (new) | Feature `tracy` → `tracing-tracy` on Bevy; doc how to run. **Not** required for ship gate. |
| **P3-B** | Witness contract | `sim_spectrum_analytics.rs`, `render_schedule_perf.rs` | `render_schedule` block required when instrumentation on. Triage **must** list `render_thread_draw_and_present` before stall substages. |
| **P3-C** | Trim redundant logs | `stall_watch.rs`, `test_run_instrumentation.rs` | Default `--test demo`: no `STALL` spam; disk witness only. |
| **P3-D** | Runbook | `visual_test_runbook_v1.md` | Single “perf truth” recipe: release, no env, read `render_schedule` + `spine.tile_raster_ms`. |

**Phase P3 gate:** Doc merged; one-command repro in runbook; no behavior change to P0–P2 gates.

---

## 6. ECS schedule (target end state)

```text
Update (Simulation):
  MapCameraSystemSet (input → smooth)
  → ViewAuthoritySystemSet::SyncViewManager
  → [dirty] FireVisualFrameSet::BuildProfiles … ProjectGpu
  → [dirty] WorldRepresentationSystemSet::ComputeFrame
  → TerrainGpuSyncSet (tilemap index patches only — no CPU raster)
  → Streaming (residency dirty only)

Render thread:
  ExtractSchedule → RenderSystems chain
  → terrain tilemap/instanced pass in Core2d
  → [if inst>0] fire particle compute
  → present

PostUpdate:
  Minimap compositor (cadence Hz, overlay dirty)
  → egui HUD (static textures cached)
```

**Forbidden:** `tile_world_fallback_rasterize` in `Simulation` when `TerrainRenderAuthority != CpuFallback`.

---

## 7. File ownership matrix

| File / area | Phase | Owner |
|:---|:---|:---|
| `terrain_render_authority.rs`, `terrain_material_atlas.rs`, `terrain_instanced_draw.rs` | P0 | render |
| `tilemap_adapter.rs`, `tilemap_render_plugin.rs` | P0 | render |
| `tile_world_fallback.rs` | P0 (gate), P1 | render |
| `map_tile_atlas_stamp.rs` | P0-D | gui + construction |
| `minimap_compositor/*` | P0-E, P1-E | render |
| `gpu_particle*.rs`, `gpu_spark_compute.rs` | P1 | render |
| `render_projection_graph.rs`, `fire_visual_extract.rs` | P1 | render/extraction |
| `world_representation.rs`, `overlay_field_buffers.rs` | P2 | gui/render |
| `sim_spectrum_analytics.rs`, `render_schedule_perf.rs`, `stall_watch.rs` | P3 | dev/render |
| `Cargo.toml`, `ci.yml` | P0-C | infra |

---

## 8. Diagnostics / witnesses (required artifacts)

| Witness | New / updated fields | Phase |
|:---|:---|:---|
| `sim_spectrum_analytics_live.json` | `render_schedule.*`, `spine.tile_raster_ms == 0`, `terrain_authority` | P0+ |
| `stage5_full_app_live.json` | `perf.terrain_gpu_authoritative: true` | P0 |
| `minimap_compositor_live.json` | `terrain_source: gpu_atlas` | P0-E |
| `engine_deep_debug_live.json` | `images` count **down** (no duplicate terrain textures) | P0 |
| `debug_runs/perf_attribution_60s.md` | P3 runbook section | P3 |

Add lib tests (not only manual runs):

- `terrain_render_authority::sim_default_not_cpu`
- `tilemap_adapter::material_index_maps_to_atlas_uv`
- `gpu_particle_draw::zero_instances_zero_dispatch` (extend existing)

---

## 9. Risks + mitigations

| Risk | Mitigation |
|:---|:---|
| `bevy_ecs_tilemap` render API mismatch 0.19 | **DR-MIG-TILEMAP** — use P0-C′ instanced fallback until upstream 0.19; timebox C to 1 day after unblock |
| Editor still uses `PerTileEntities` | P0: editor may keep `CpuFallback` until editor migration (document); sim/demo must not |
| Stage 5 regression | Run `cargo test -p proc_A_dine01 --lib stage5` each phase |
| Atlas rebuild hitch on registry load | Async prepare one frame early; show loading chrome |
| Minimap black during transition | Bootstrap GPU terrain before hiding fallback sprite (same frame commit) |

**Rollback:** Flip `TerrainRenderAuthority::CpuFallback` via env `TERRAIN_CPU_FALLBACK=1` (debug only) until P0 stable — remove env before program sign-off.

---

## 10. Program exit gate (ALL must pass)

| # | Check | Command / witness |
|:---|:---|:---|
| 1 | Lib tests green | `cargo test -p proc_A_dine01 --lib --release` |
| 2 | Stage 5 regression | `cargo test -p proc_A_dine01 --lib stage5 --release` |
| 3 | Demo 60s p95 frame ≤ 33ms | `cargo run -p proc_A_dine01 --release -- --test demo --stay-open` + sim_spectrum |
| 4 | Visual 60s p95 frame ≤ 33ms | `cargo run -p proc_A_dine01 --release -- --test visual --stay-open` |
| 5 | CPU tile raster off | `last_frame.spine.tile_raster_ms == 0` steady sim |
| 6 | Render present bounded | `render_schedule.render_and_present_ms` p95 ≤ 16ms |
| 7 | No stall mismatch idle | `attribution_honesty.stall_checkpoint_mismatch == false` idle 60s |
| 8 | Minimap witness green | `minimap_compositor_live.json` green |
| 9 | FULL_APP readiness | `stage5_full_app_live.json` green |
| 10 | Runbook updated | `visual_test_runbook_v1.md` perf truth section |

**Sign-off record:** append row to [`post_stage6_active_todos.md`](post_stage6_active_todos.md) Phase F or new `PERF-GPU-TERRAIN-001` with witness paths + date.

---

## 11. @coder handoff (single session instruction)

```text
Implement PLAN-GPU-TERRAIN-EXEC-001 in order P0 → P1 → P2 → P3.
Do not return for review until §10 all green.
Read first: bevy-simulation-grade 07-repo-authority-map, validation-first skill.
After each phase: cargo test filters listed in phase gate — fix before continuing.
If P0-C blocked >1 day: implement P0-C′ and document in commit message.
No new profiling systems unless P3 — use existing RenderScheduleWitness.
Minimize scope: no refactors outside file ownership matrix.
```

Invoke: `@coder` with link to this file + `tools/orchestrator/agents/stage5_readiness_agent.md` + `render_pipeline_agent.md`.

---

## 12. Open questions (resolve in Phase P0-A, do not block start)

| # | Question | Default if unresolved |
|:---|:---|:---|
| Q1 | `bevy_tilemap_adapter` in default features vs release-only? | **Yes** default features for `proc_A_dine01` |
| Q2 | Editor world gen stays `PerTileEntities` this program? | **Yes** — sim/demo chunk-authoritative only |
| Q3 | Interim `GpuInstancedAtlas` acceptable for P0 sign-off if C lags? | **Yes**, if visual parity + perf gates met |

---

## 13. Tracking

| Board ID | Phase | Status |
|:---|:---|:---|
| **PERF-GPU-TERRAIN-001** | P0 | **SCAFFOLDED** — P0-C′ files + opt-in env; **PRIME + §10 gate open** → [`gpu_todos_v1.md`](gpu_todos_v1.md) |
| **PERF-GPU-TERRAIN-002** | P1 | **PARTIAL** — verify P1-A..E in gpu_todos |
| **PERF-GPU-TERRAIN-003** | P2 | **PARTIAL** — verify P2-A..D in gpu_todos |
| **PERF-GPU-TERRAIN-004** | P3 | **OPEN** — runbook + tracy optional |

Update [`plan_visual_perf_production_v1.md`](plan_visual_perf_production_v1.md) Phase 5 row: **→ moved to PLAN-GPU-TERRAIN-EXEC-001**.
