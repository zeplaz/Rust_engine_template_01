# WSS-PLAN-002 — Chunk authority & substrate storage `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-PLAN-002** |
| **Parent** | [`wssr_index_v1.md`](wssr_index_v1.md) (**WSS-PLAN-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Coder unblock** | **WSS-CHUNK-SLAB-001** — exec [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md); design gate **PASS (qualified)** |
| **Downstream** | [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) · [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md) |

**No Rust in this deliverable.** Structs below are **expanded target shapes** — implement fully, not as stubs-to-forget.

---

## Summary

Persistent world-state lives in **resource slabs keyed by chunk** (`ChunkSlab<T>`), not one ECS entity per chunk during substrate evolution. **Hot/active regions** later gain optional **`ActiveChunkRuntime`** ECS entities for combat, construction, fire propagation, and nearby AI — a **hybrid** that scales to planetary sim without archetype churn today.

---

## Current problems

| Issue | Evidence | Severity |
|:---|:---|:---:|
| Domain state scattered across components + resources | `ChunkWeather`, `ChunkSurfaceFire`, `DynamicTerrainOverlay`, `AtmosphereField` grid | HIGH |
| No unified persist key for save/load dirty regions | Hydrology v1 spec mentions dirty regions; no central chunk key | MED |
| Existing `Chunk` ECS entity used for environment components | Works for local sim; breaks at planetary entity counts if expanded naively | MED |
| Terrain treated as rendering + gen, not simulation substrate | `gpu_water_*` vs `HydrologyResult` split | HIGH |
| Transitional overlays correct concept, wrong home long-term | `DynamicTerrainOverlay` HashMaps vs slab integration | LOW |

---

## Target architecture

### Core keys and storage

```rust
/// Canonical chunk coordinate for ALL substrate domains.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
}

impl From<IVec2> for ChunkKey {
    fn from(v: IVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// Generic paged storage — ONE pattern for all persistent domains.
#[derive(Resource, Debug, Default)]
pub struct ChunkSlab<T> {
    pub chunks: HashMap<ChunkKey, T>,
    /// Paging / streaming: chunks present in sim window this tick.
    pub resident: HashSet<ChunkKey>,
    /// Save/load: keys dirtied since last persist flush.
    pub dirty: HashSet<ChunkKey>,
}

impl<T> ChunkSlab<T> {
    pub fn get(&self, key: ChunkKey) -> Option<&T> { self.chunks.get(&key) }
    pub fn get_mut(&mut self, key: ChunkKey) -> Option<&mut T> {
        self.dirty.insert(key);
        self.chunks.get_mut(&key)
    }
    pub fn insert(&mut self, key: ChunkKey, value: T) {
        self.dirty.insert(key);
        self.chunks.insert(key, value);
    }
}
```

### Full world chunk state (expanded — not minimal)

```rust
/// Static / slow-changing substrate facts (from worldgen + geology passes).
pub struct TerrainState {
    pub height: Vec<f32>,              // cell grid within chunk
    pub material_ids: Vec<u16>,
    pub geology_class: GeologyClass,
    pub biome_id: u16,
    pub porosity: Vec<f32>,
    pub hardness: Vec<f32>,
}

/// Runtime hydrology — ocean is a band/mask HERE, not a separate system.
/// See WSS-PLAN-003.
pub struct HydrologyState {
    pub water_depth: Vec<f32>,
    pub flow_velocity: Vec<Vec2>,      // per cell or downsampled
    pub sediment: Vec<f32>,
    pub salinity: Vec<f32>,
    pub saturation: Vec<f32>,          // soil moisture coupling
    pub ocean_mask: Vec<u8>,           // deep water / coastal band
    pub river_mask: Vec<u8>,
    pub lake_mask: Vec<u8>,
}

/// Fast local weather scalars + clipmap sample hooks.
pub struct AtmosphereState {
    pub local: ChunkWeatherLocal,      // rain, fog, snow_depth, wind, visibility
    pub clipmap_sample: AtmosphereSampleRefs, // indices into regional clipmaps
}

/// SEPARATE from atmosphere grid — see WSS-PLAN-004.
pub struct ContaminationState {
    pub airborne: Vec<f32>,
    pub soil: Vec<f32>,
    pub waterborne: Vec<f32>,
    pub bioactive: Vec<f32>,
    pub radiation: Vec<f32>,
}

pub struct AtmosphereCoupling {
    pub wind_transport: f32,
    pub humidity_binding: f32,
    pub thermal_exchange: f32,
}

pub struct DeformationState {
    pub height_delta: Vec<f32>,          // persistent terrain mutation
    pub compaction: Vec<f32>,
    pub landslide_risk: Vec<f32>,
    pub last_mutation_tick: u64,
}

pub struct EcologyState {
    pub biomass: Vec<f32>,
    pub fuel_load: Vec<f32>,
    pub vegetation_class: Vec<u8>,
    pub fire_risk: Vec<f32>,
    pub stress: Vec<f32>,              // drought, pollution, overgrazing
}

pub struct ThermalState {
    pub surface_heat: Vec<f32>,
    pub subsurface_heat: Vec<f32>,
    pub ash_cover: Vec<f32>,
}

/// Transient fast-decay scalars (mud, congestion) — still persistable per game mode.
pub struct DynamicOverlaySlice {
    pub mud: Vec<f32>,
    pub snow_accum: Vec<f32>,
    pub danger: Vec<f32>,
    pub congestion: Vec<f32>,
}

/// Aggregated per-chunk persistent state.
pub struct WorldChunkState {
    pub key: ChunkKey,
    pub terrain: TerrainState,
    pub hydrology: HydrologyState,
    pub ecology: EcologyState,
    pub atmosphere: AtmosphereState,
    pub contamination: ContaminationState,
    pub coupling: AtmosphereCoupling,
    pub deformation: DeformationState,
    pub thermal: ThermalState,
    pub dynamic: DynamicOverlaySlice,
    pub sim_lod: u8,
    pub version: u32,                  // schema migration
}
```

### Resource registry (single authority entry point)

```rust
#[derive(Resource)]
pub struct WorldSubstrateRegistry {
    pub chunks: ChunkSlab<WorldChunkState>,
    pub paging: ChunkPagingState,      // focus, radius, residency window tie-in
    pub persist: SubstratePersistBook, // dirty flush, save slots
}
```

**`WorldSubstrateRegistry` OWNS** slab insert/mutate for cross-domain chunk state. Domain systems receive **scoped writers** via system params — not direct HashMap mutation elsewhere.

### Hybrid ECS (phase 2 — after slabs land)

```rust
/// ONLY for hot/active runtime — combat, construction, fire spread front, nearby AI.
#[derive(Component)]
pub struct ActiveChunkRuntime {
    pub key: ChunkKey,
    pub activation_reason: ChunkActivationReason,
    pub deactivate_after_ticks: Option<u64>,
}

pub enum ChunkActivationReason {
    FireFront,
    FloodSolve,
    Construction,
    Combat,
    PlayerProximity,
    HydrologyEvent,
}
```

**Rule:** `ActiveChunkRuntime` entities **mirror** slab state for systems that benefit from ECS queries — they do **not** replace slab authority. On deactivate, flush ECS-side deltas → slab.

---

## Authority map

| Domain | Sole writer (L1) | Storage | Readers (read-only) |
|:---|:---|:---|:---|
| Terrain facts | Worldgen persist + deformation apply | `WorldChunkState.terrain` | hydrology, ecology, fire fuel, extraction |
| Hydrology | `HydrologySimulationTask` scheduler | `WorldChunkState.hydrology` | weather, contamination.waterborne, extraction |
| Ecology | `ChunkEnvironmentSet::Ecology` (migrate) | `WorldChunkState.ecology` | fire, logistics readability |
| Atmosphere local | `ChunkEnvironmentSet::Weather` (migrate) | `WorldChunkState.atmosphere.local` | atmosphere clipmap fold |
| Contamination | `ContaminationTickSet` (new) | `WorldChunkState.contamination` | ecology stress, logistics, sensors |
| Deformation | `DeformationApplySet` (new) | `WorldChunkState.deformation` | hydrology redirect, terrain render |
| Thermal / fire residue | `ChunkEnvironmentSet::Fire` (migrate) | `WorldChunkState.thermal` | atmosphere fold, extraction |
| Dynamic overlay | `DynamicOverlaySet` | `WorldChunkState.dynamic` | traction, pathfinding |
| Slab paging | `ChunkPagingSystem` | `WorldSubstrateRegistry.paging` | all sim systems |
| Persist flush | `SubstratePersistSystem` | `WorldSubstrateRegistry.persist` | save/load |

**L2 extraction** reads **`WorldSubstrateRegistry`** (or immutable snapshot `Arc<WorldSubstrateSnapshot>`) — **never** mutates slabs.

**Forbidden:** Render systems writing any `WorldChunkState` field; second parallel `HashMap` for hydrology or contamination outside registry.

---

## Migration from current code

| Current | Target | Phase |
|:---|:---|:---:|
| `Chunk` entity + `ChunkWeather` component | slab `atmosphere.local` + optional `ActiveChunkRuntime` | M1 |
| `ChunkSurfaceFire`, `ChunkSmokeField` components | `thermal` + atmosphere fold inputs | M2 |
| `DynamicTerrainOverlay` resource HashMaps | `WorldChunkState.dynamic` per chunk | M1 |
| `terrain/generation` outputs | hydrate `TerrainState` + initial `HydrologyState` on gen | M1 |
| `AtmosphereField` global grid | **regional clipmap** tiles referencing chunk keys (WSS-PLAN-004) | M2 |
| `FireFuelField`, `ChunkEcology` components | `EcologyState` slab fields | M2 |

**Compatibility bridge:** During M1, dual-write shim `ChunkWeather` ↔ slab with `#[deprecated]` witness flag — remove when `wss_substrate_live.json` green.

---

## Implementation phases

### Phase W2-A — Types + registry skeleton

| Goal | Deliverable |
|:---|:---|
| Introduce `ChunkKey`, `ChunkSlab`, `WorldChunkState` (full struct) | `src/world_substrate/` module |
| `WorldSubstrateRegistry` resource init | `WorldSubstratePlugin` |
| Hydrate terrain + hydrology from existing gen into slab on chunk spawn | bridge from `Chunk` entity |

**Files (initial):**
- `src/world_substrate/mod.rs`
- `src/world_substrate/types.rs`
- `src/world_substrate/registry.rs`
- `src/world_substrate/persist.rs` (stub API)

**Acceptance:**
```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib world_substrate
```

**Witness:** `wss_substrate_live.json` → `slab_registry_present: true`, `chunk_count > 0`

---

### Phase W2-B — Dual-write shim + paging

| Goal | Deliverable |
|:---|:---|
| Sync `ChunkWeather` ↔ slab for resident chunks | shim with drift metric |
| Tie `ChunkPagingState` to Stage 6 residency / camera focus | `PerViewResidencyConsumerWindow` read |

**Acceptance:** witness `dual_write_drift_max < epsilon` in test fixture

---

### Phase W2-C — ActiveChunkRuntime hybrid

| Goal | Deliverable |
|:---|:---|
| Spawn `ActiveChunkRuntime` on fire front / flood event | activation/deactivation systems |
| Flush on deactivate | no orphaned ECS state |

**Acceptance:** lib test — activate 3 chunks, deactivate, slab equals pre-ECS truth

---

### Phase W2-D — Persist book

| Goal | Deliverable |
|:---|:---|
| Dirty region tracking | `SubstratePersistBook` |
| RON save slice for one chunk domain (hydrology first) | round-trip test |

**Acceptance:** save/load one `ChunkKey` hydrology slice; `dirty` clears on flush

---

## ECS schedule plan

```text
SimControlSystemSet::AdvanceSimTick
  → ChunkPagingSystem                    [WorldSubstrateRegistry.paging — WRITER]
  → ChunkEnvironmentSet (existing order)
       Lod → Weather → Ecology → Fire
  → ContaminationTickSet                 [ContaminationState — WRITER]
  → DeformationApplySet                  [DeformationState — WRITER]
  → SubstratePersistSystem (every N ticks) [persist book — WRITER]

PostSim (optional parallel jobs)
  → HydrologySimulationTask schedule     [see WSS-PLAN-003]

PreExtract
  → build_world_substrate_snapshot       [Arc snapshot — WRITER, once/frame]
  → RenderProjectionGraph consumes snapshot [READ ONLY]
```

---

## Diagnostics

| Metric | Source |
|:---|:---|
| `slab_chunk_count` | registry |
| `resident_count` | paging |
| `dirty_count` | persist book |
| `dual_write_drift` | shim compare |
| `active_runtime_entity_count` | ECS query |
| `single_writer_violations` | debug assert (dev only) |

**Witness path:** `debug_runs/wss_substrate_live.json`

---

## Edge cases

- **Empty chunk / ocean-only chunk:** slab entry exists with `ocean_mask` dominant; sim LOD may skip ecology tick
- **Streaming boundary:** paging evicts slab from `resident` but **retains** in `chunks` until memory pressure policy
- **Schema migration:** `WorldChunkState.version` bump with loader shim — never silent truncate
- **Multiview:** paging uses **simulation focus**, not per-view camera — views derive via extraction
- **Editor undo:** deformation dirty must participate in editor transaction book (future coordination with construction)

---

## Open questions (remaining)

1. Cell resolution within chunk vs downsampled hydrology grid — planner recommends **same cell grid as terrain height** for v1; downsample later for perf
2. Memory cap eviction policy — LRU vs distance-from-focus (tie to Stage 6)
3. MP authority — server owns slab writes; client snapshot read-only (defer to MP plan)

---

## Rollback trigger

- Dual-write drift exceeds threshold in CI
- Measurable regression in `fire_ecology_live.json` or chunk environment ordering tests
- Stage 5 FULL_APP failure traceable to substrate plugin — **disable plugin**, not revert FIRE7 spine

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Signed — hybrid slab-first architecture |
