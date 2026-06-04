# PLAN-WSS-HYDRO-RUNTIME-EXEC-001 — WSS-HYDRO-RUNTIME-001 execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-HYDRO-RUNTIME-EXEC-001** |
| **Slice ID** | **WSS-HYDRO-RUNTIME-001** |
| **Prior** | [`wssr_plan_003_hydrology_runtime_v1.md`](wssr_plan_003_hydrology_runtime_v1.md) (**WSS-PLAN-003 SIGNED**) |
| **Parent** | [`wssr_index_v1.md`](wssr_index_v1.md) (**WSS-PLAN-001**) |
| **Design** | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) · [`hydrology_v1.md`](../prompts/designer_questions/terrain_world/hydrology_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **READY** — blocked on **WSS-CHUNK-SLAB-001** hydrate path |
| **Suggested owner** | `@coder` B (terrain / worldgen boundary) |

**Prereq gates:**

| Gate | Path | Status |
|:---|:---|:---:|
| **WSS-DESIGN-GATE-001** | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) | **PASS (qualified)** ☑ |
| **WSS-CHUNK-SLAB-001** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) | `hydrate_wired` + full `HydrologyState` in `WorldChunkState` |

---

## Scope

Land **WSS-PLAN-003** phases **W3-A → W3-D** (gen hydrate, background tick, event deep solve, visual extract + GPU handoff). **W3-E** (persist dirty hydrology regions) is a follow-on unless merged by planner waiver.

**In scope:**

- `HydrologyState` + `HydrologySolveMeta` authoritative on `WorldChunkState.hydrology`
- Gen `HydrologyResult` → slab hydrate on chunk spawn (extend CS-003)
- `HydrologyBackgroundTick` on resident keys (Tier 1)
- `HydrologySimulationTask` scheduler + `HydrologyDirtyReason` event bus (Tier 2)
- `HydrologyBoundaryExchange` at chunk edges
- `HydrologyVisualExtract` projection-graph node → existing `gpu_water_*` consumers
- Witness block `wss_hydro_runtime_001` in `debug_runs/wss_substrate_live.json`
- `ocean_tile_count` slab-backed (not VFX-only language)

**Out of scope:**

- `FluidDomain` global resource (**forbidden**)
- `OceanSystem` / separate ocean renderer module
- GPU readback of water depth into sim
- Full groundwater 3D solve (stub `groundwater` Vec only)
- MP client prediction rollback (gameplay plan)

**Regression guards:** `stage5_full_app_live.json` → `water_w1_green`, `water_w2_green`, D-W09 strategic band, [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) tactical look.

---

## Hybrid migration matrix

| Incumbent | WSS target | Authoritative until | This slice |
|:---|:---|:---|:---|
| `HydrologyResult` (gen only) | hydrate → slab | gen still **source at spawn** | copy on spawn |
| `water_surface_visual` from gen hydro | reads slab snapshot | extract from slab | W3-D |
| `gpu_water_*` / particles | L3 consumer | **YES** — no depth writeback | extract hints only |
| `RiverMarker` / gen masks | `river_mask` in slab | slab after hydrate | W3-A |
| `water_ocean_tiles` witness | `ocean_mask` sum | slab-backed count | W3-D |

---

## Agreed module layout

| Path | Responsibility |
|:---|:---|
| `src/substrate/hydrology/mod.rs` | exports, plugin |
| `src/substrate/hydrology/state.rs` | `HydrologyState`, `HydrologySolveMeta`, `HydrologyDirtyReason` |
| `src/substrate/hydrology/hydrate.rs` | `HydrologyResult` → slab (extend `substrate/hydrate.rs`) |
| `src/substrate/hydrology/background_tick.rs` | Tier 1 resident tick |
| `src/substrate/hydrology/scheduler.rs` | `HydrologySimulationTask`, deep solve queue |
| `src/substrate/hydrology/boundary.rs` | inter-chunk flux at edges |
| `src/substrate/hydrology/live_proof.rs` | witness fields for hydro block |
| `src/systems/hydrology/event_queue.rs` | drain `HydrologyDirtyReason` from construction/scenario |
| `src/render/extraction/hydrology_visual_extract.rs` | `HydrologyVisualExtract` node |
| `src/terrain/generation/hydrology/` | **read-only** — no authority move |

**Forbidden new modules:** `ocean_system.rs`, `fluid_domain.rs`, `OceanRendererPlugin`.

---

## Authority map

| Domain | Single writer |
|:---|:---|
| `WorldChunkState.hydrology` | hydrate (spawn), `HydrologyBackgroundTick`, deep solve apply |
| `HydrologySimulationScheduler` | `schedule_hydrology_deep_tasks` |
| `HydrologyBoundaryExchange` | post-solve / pre-extract same frame |
| `HydrologyVisualExtract` | PreExtract snapshot (read-only slab) |
| `gpu_water_*` | L3 — consumes extract hints only |
| `wss_hydro_runtime_001` witness | `write_wss_substrate_live_proof_system` |

**Forbidden:** particle systems writing `water_depth`; second terrain extract for water; global `Vec` water state outside registry.

---

## Task list (HY-001 … HY-008)

### HY-001 — Hydrology state types (W3-A)

1. Full `HydrologyState` fields per WSS-PLAN-003 in `types.rs` / `state.rs`.
2. All per-cell `Vec` lengths = `CELL_COUNT`.
3. `HydrologyDirtyReason` enum + `HydrologySolveMeta`.

**Exit:** types compile; `hydrology_state_present: true`.

---

### HY-002 — Gen hydrate bridge (W3-A)

1. Extend `hydrate_chunk_into_substrate` to copy `HydrologyResult` when available on chunk/worldgen context.
2. Map: `water_depth`, masks, `flow_velocity` stub from D8 gen, `ocean_mask`, `salinity` defaults.
3. **Do not** mutate gen resource after copy.

**Exit:** `hydrology_hydrated: true`; lib test `river_mask_nonzero_on_fixture`.

---

### HY-003 — Background tick (W3-B)

1. `HydrologyBackgroundTick` after `ChunkPaging` / resident sync.
2. Resident keys only; evaporation/rain coupling from `ChunkWeather` or slab `atmosphere.local`.
3. Saturation diffusion stub + flow direction refresh from `TerrainState.height`.

**Exit:** `hydrology_background_wired: true`; test `saturation_changes_under_rain`.

---

### HY-004 — Boundary exchange (W3-B)

1. `HydrologyBoundaryExchange` — peek neighbor slab across ±X/±Y chunk keys.
2. Flux continuity test two adjacent chunks.

**Exit:** `boundary_exchange_wired: true`; `boundary_exchange_flux_max` in witness.

---

### HY-005 — Event scheduler + deep solve (W3-C)

1. `HydrologySimulationTask` with `Background` / `DeepEvent` tiers.
2. `HydrologyEventQueueDrain` — accept `HydrologyDirtyReason` from construction bridge (stub OK).
3. Time-sliced deep solve over frames — dam breach fixture redistributes `water_depth`.
4. Budget guard — no single frame > threshold in test harness.

**Exit:** `deep_solve_wired: true`; `deep_solve_active_tasks` diagnostic.

---

### HY-006 — Visual extract + GPU handoff (W3-D)

1. `HydrologyVisualExtract` in projection graph: polylines, coast, `ocean_tiles`, `strategic_ribbon`.
2. Feed existing `gpu_water_*` / `water_surface_visual` from extract — **no** parallel hydro query in render.
3. `ocean_tile_count` from `ocean_mask` sum on resident slabs.

**Exit:** `hydrology_extract_wired: true`; `water_w1_green` / `water_w2_green` unchanged in stage5 JSON.

---

### HY-007 — Witness block

Nested in `wss_substrate_live.json`:

```json
"wss_hydro_runtime_001": { ... }
```

Cross-link `stage5_full_app_live.json` water rows in envelope `related_proofs`.

---

### HY-008 — Lib tests

```powershell
cargo test -p proc_A_dine01 --lib hydrology_hydrate hydrology_background hydrology_boundary hydrology_deep
```

---

## Test predicates

### `hydrate_copies_river_mask`

```text
GIVEN fixture chunk with known HydrologyResult.river_mask sum > 0
WHEN hydrate_chunk_into_substrate
THEN slab.hydrology.river_mask sum matches gen within epsilon
```

### `background_tick_resident_only`

```text
GIVEN keys A resident, B not resident
WHEN HydrologyBackgroundTick
THEN only A.hydrology.saturation may change
```

### `boundary_flux_continuous`

```text
GIVEN two adjacent chunks with water_depth gradient at shared edge
WHEN HydrologyBoundaryExchange
THEN flux at edge finite AND no NaN
```

### `deep_solve_dam_breach`

```text
GIVEN HydrologyDirtyReason::DamBreach on key K
WHEN deep solve completes (may be multi-frame)
THEN water_depth downstream of K increases
```

### `extract_ocean_tiles_from_slab`

```text
GIVEN slab with ocean_mask cells
WHEN build_hydrology_visual_extract
THEN extract.ocean_tiles == mask count AND stage5 water witnesses pass compare
```

### `gpu_water_no_depth_writeback`

```text
GIVEN rg water_depth assignment in gpu_water*
WHEN only extract/build paths
THEN no write to WorldSubstrateRegistry or HydrologyState from L3 modules
```

---

## Witness JSON — `wss_hydro_runtime_001`

**Path:** `debug_runs/wss_substrate_live.json` (nested block)

| JSON pointer | Type | Semantics |
|:---|:---|:---|
| `/wss_hydro_runtime_001/gate` | string | `"WSS-HYDRO-RUNTIME-001"` |
| `/wss_hydro_runtime_001/green` | bool | rollup |
| `/wss_hydro_runtime_001/hydrology_state_present` | bool | full struct on slab |
| `/wss_hydro_runtime_001/hydrology_hydrated` | bool | gen copy observed |
| `/wss_hydro_runtime_001/hydrology_background_wired` | bool | Tier 1 ran |
| `/wss_hydro_runtime_001/boundary_exchange_wired` | bool | edge flux |
| `/wss_hydro_runtime_001/deep_solve_wired` | bool | Tier 2 path |
| `/wss_hydro_runtime_001/hydrology_extract_wired` | bool | projection node |
| `/wss_hydro_runtime_001/ocean_tile_count` | number | slab-backed |
| `/wss_hydro_runtime_001/river_channel_cells` | number | mask sum |
| `/wss_hydro_runtime_001/deep_solve_active_tasks` | number | scheduler |
| `/wss_hydro_runtime_001/boundary_exchange_flux_max` | number | diagnostic |
| `/wss_hydro_runtime_001/waterborne_contamination_max` | number | optional coupling read |

### Green predicate (slice v1)

```text
wss_hydro_runtime_001_green :=
  gate == "WSS-HYDRO-RUNTIME-001"
  AND hydrology_hydrated
  AND hydrology_background_wired
  AND boundary_exchange_wired
  AND deep_solve_wired
  AND hydrology_extract_wired
  AND ocean_tile_count > 0 on default fixture
  AND stage5 water_w1_green AND water_w2_green unchanged
```

---

## ECS schedule (this slice)

```text
SimControlSystemSet::AdvanceSimTick
  → sync_substrate_paging
  → HydrologyBackgroundTick          [resident keys, WRITER hydrology]
  → HydrologyEventQueueDrain           [dirty reasons]
  → schedule_hydrology_deep_tasks

PostSim / task pool
  → run_hydrology_deep_solve           [time-sliced]
  → apply_hydrology_boundary_exchange

PreExtract
  → build_hydrology_visual_extract     [read slab snapshot]
  → RenderProjectionGraph hydrology node
```

**Ordering:** Background tick **after** deformation apply when earthworks same frame.

---

## Diagnostics (F3 / witness alignment)

| Key | Label |
|:---|:---|
| `hydrology_resident_chunks` | resident slab keys with hydro |
| `deep_solve_active_tasks` | scheduler queue depth |
| `ocean_tile_count` | slab `ocean_mask` sum |
| `river_channel_cells` | `river_mask` sum |

Optional designer doc: extend [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) in P2.

---

## Verification commands

```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib hydrology_hydrate hydrology_background hydrology_boundary
cargo test -p proc_A_dine01 --lib stage5
# water-specific rows in stage5 witness
```

---

## Anti-patterns (forbidden)

| Anti-pattern | Why |
|:---|:---|
| `FluidDomain` | WSS-PLAN-003 rejection |
| `OceanSystem` module | authority silo |
| GPU → sim depth readback | causality |
| Skipping persist on visual-only flood | save/load (W3-E) |
| Second terrain extract for water | extraction graph rule |

---

## Rollback

- Disable hydrology tick via env `RUST_ENGINE_HYDRO_RUNTIME=0` — gen hydrate copy may remain read-only
- `water_w1_green` false → revert extract node, keep slab hydrate only

---

## Coder assignment

| Field | Value |
|:---|:---|
| **Slice** | WSS-HYDRO-RUNTIME-001 |
| **Blocked until** | `/hydrate_wired` + `/hydrology_hydrated` from slab PR-1 |
| **Budget** | ≤10 files; split HY-001..004 then HY-005..006 |
| **Playbook** | `render_pipeline_agent` + terrain worldgen playbook |
| **Parallel** | **FIRE-F2-EXTRACT-001** (disjoint) · **not** `src/construction/*` writers to hydro without event bus |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Full exec plan — mirrors chunk-slab exec pattern |
