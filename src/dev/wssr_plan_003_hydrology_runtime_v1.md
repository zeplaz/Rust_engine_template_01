# WSS-PLAN-003 — Hydrology runtime authority `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-PLAN-003** |
| **Parent** | [`wssr_index_v1.md`](wssr_index_v1.md) (**WSS-PLAN-001**) |
| **Depends on** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) (**WSS-PLAN-002**) slab types |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Design input** | [`hydrology_v1.md`](../prompts/designer_questions/terrain_world/hydrology_v1.md) |
| **Coder unblock** | **WSS-HYDRO-RUNTIME-001** |

**No Rust in this deliverable.** Ocean, rivers, lakes, flooding, and coast foam are **hydrology substrate** — not a separate ocean renderer or `FX-WATER` silo.

---

## Summary

Hydrology **OWNS** all surface and subsurface water state per chunk inside **`ChunkSlab<HydrologyState>`** (via `WorldChunkState.hydrology`). Worldgen **`HydrologyResult`** hydrates initial slab data. **Runtime** uses **scheduled active-chunk simulation tasks** with boundary exchange — **NOT** a monolithic `FluidDomain` resource. GPU water particles, river shaders, and coast foam are **Layer 3** consumers of extraction — they never hold water depth authority.

---

## Current problems

| Issue | Evidence |
|:---|:---|
| Hydrology is **gen-time only** | `HydrologyResult`, `compute_hydrology_world` — no runtime slab |
| Water VFX closed as product track but **authority gap** remains | `gpu_water_*`, `water_ocean_tiles` witness vs no runtime ocean state |
| Rivers invisible before W1 was **presentation**; still no **flow state** at runtime | `RiverMarker` in gen, no `flow_velocity` persistence |
| Designer spec for event-driven deep solve **unsigned in code** | `hydrology_v1.md` triggers listed, no `HydrologySimulationTask` |
| Risk of `FluidDomain` anti-pattern | mentioned in designer Q — must explicitly reject |

---

## Target architecture

### Hydrology state (expanded per-chunk)

```rust
pub struct HydrologyState {
    /// Standing / flowing water depth per terrain cell [m].
    pub water_depth: Vec<f32>,
    /// D8 or vector flow per cell — downstream direction + magnitude.
    pub flow_velocity: Vec<Vec2>,
    /// Suspended sediment load — erosion/deposition coupling.
    pub sediment: Vec<f32>,
    /// Brackish / marine coupling — ocean cells high, rivers low.
    pub salinity: Vec<f32>,
    /// Volumetric soil saturation — feeds ecology + fire suppression gate.
    pub saturation: Vec<f32>,
    /// Deep ocean / shelf band — NOT a separate OceanSystem.
    pub ocean_mask: Vec<u8>,
    /// Channel mask from gen path + runtime incision.
    pub river_mask: Vec<u8>,
    /// Basin standing water.
    pub lake_mask: Vec<u8>,
    /// Groundwater table height (optional slow layer).
    pub groundwater: Vec<f32>,
    /// Event-driven solve metadata.
    pub solve: HydrologySolveMeta,
}

pub struct HydrologySolveMeta {
    pub last_background_tick: u64,
    pub last_deep_solve_tick: u64,
    pub deep_solve_active: bool,
    pub dirty_reason: HydrologyDirtyReason,
}

pub enum HydrologyDirtyReason {
    None,
    DamBreach { structure_id: u64 },
    ConstructionComplete { structure_id: u64 },
    Explosive { cell_index: u32 },
    ScenarioScript { script_id: String },
    UpstreamOverflow,
    ErosionThreshold,
    ManualEditor,
}
```

### Storage authority

```text
WorldSubstrateRegistry.chunks[key].hydrology  OWNS all water state
HydrologySimulationScheduler                  OWNS which keys run this tick
HydrologyBoundaryExchange                     OWNS inter-chunk flux at edges
gpu_water_* / water_surface_visual            CONSUME extraction only
```

**Forbidden:** `FluidDomain` as global mutable water authority; `OceanRendererPlugin`; water depth writes from particle systems.

---

## Simulation tiers

### Tier 0 — Static / gen baseline

- Worldgen pass 4 (`p4_hydrology`) → hydrate `HydrologyState` on chunk insert
- Rivers, lakes, ocean bands from `HydrologyResult` + height field
- No per-tick cost for distant chunks

### Tier 1 — Background tick (cheap, wide)

Runs on **resident** chunks in sim window:

- Evaporation / precipitation coupling from `AtmosphereState.local`
- Saturation diffusion (slow)
- River flow direction refresh from height + deformation delta
- Sediment transport (low fidelity)

**Schedule:** every N sim ticks; parallel over chunk keys with read-only neighbor slab peek.

### Tier 2 — Event-driven deep solve (localized)

Activated per [`hydrology_v1.md`](../prompts/designer_questions/terrain_world/hydrology_v1.md):

| Trigger | Example |
|:---|:---|
| Construction / destruction complete | dam, canal, culvert, pump |
| Damage threshold | breach, collapse |
| Scenario script | designer flood |
| Upstream overflow | pressure propagation |
| Player earthworks | redirect, drain |

**Scope:** active chunk set + 1-chunk boundary halo; time-sliced over multiple frames.

```rust
pub struct HydrologySimulationTask {
    pub active_keys: Vec<ChunkKey>,
    pub boundary_keys: Vec<ChunkKey>,  // halo for flux exchange
    pub tier: HydrologySimTier,
    pub tick_budget_ms: f32,
}

pub enum HydrologySimTier {
    Background,
    DeepEvent,
}
```

### Tier 3 — Extraction (L2)

```rust
pub struct HydrologyVisualExtract {
    pub river_polylines: Vec<RiverPolylineSegment>,  // existing type
    pub coast_lines: Vec<CoastSegment>,
    pub ocean_tiles: u32,
    pub foam_emitters: Vec<FoamEmitterHint>,
    pub strategic_ribbon: StrategicWaterRibbon,
}
```

Built from slab snapshot + `RenderProjectionGraph` node — **same pattern as fire per-view extract**.

---

## Terrain coupling (mandatory)

Hydrology reads/writes **must** interact with:

| Domain | Interaction |
|:---|:---|
| **TerrainState.height** | Flow direction, flooding extent |
| **DeformationState.height_delta** | Dam breach, earthworks, landslide block |
| **EcologyState** | Wetland biome, riparian stress |
| **ContaminationState.waterborne** | Industrial runoff, toxic flood plume |
| **ThermalState** | Evaporation, cooling water denial (power coupling) |
| **AtmosphereState.local** | Rain input, evaporation output |
| **Fire / SurfaceWaterFireGate** | Suppression — read saturation, not write fire |

```text
terrain height + deformation  →  hydrology flow
hydrology flood  →  contamination waterborne spread
hydrology reservoir  →  economy/power (event bus, not mega-type)
```

---

## Ocean = hydrology subsystem

| Wrong | Right |
|:---|:---|
| `OceanSystem` module | `ocean_mask` + `salinity` + deep water depth in `HydrologyState` |
| `water_ocean_tiles` as VFX-only witness | witness backed by slab `ocean_mask` count |
| Separate tile manager for sea | same chunk cell grid as terrain |
| Coast foam as particle silo | `FoamEmitterHint` from coast line extract → Layer 3 GPU |

**Coastline definition:** shallow/deep boundary from height + `ocean_mask` + `water_depth` gradient — extraction computes polyline, not hardcoded VFX.

---

## Implementation phases

### Phase W3-A — Gen hydrate into slab

| Goal | Exit |
|:---|:---|
| On chunk spawn, copy `HydrologyResult` into `WorldChunkState.hydrology` | lib test: river_mask non-zero on known fixture |
| Retire “ocean as VFX-only” language in active queues | queue doc update |

**Witness extension:** `wss_substrate_live.json` → `hydrology_hydrated: true`

---

### Phase W3-B — Background tick

| Goal | Exit |
|:---|:---|
| `HydrologyBackgroundTick` on resident keys | saturation changes under rain injection test |
| Boundary peek at chunk edges | flux continuity test (two adjacent chunks) |

**Acceptance:**
```powershell
cargo test -p proc_A_dine01 --lib hydrology_background
```

---

### Phase W3-C — Event-driven deep solve

| Goal | Exit |
|:---|:---|
| `HydrologySimulationTask` scheduler | dam breach fixture: `water_depth` redistribution |
| `HydrologyDirtyReason` from construction events | bridge hook (stub OK if event bus typed) |
| Time-slice over frames | budget test — no frame spike > threshold |

**Designer alignment:** triggers from `hydrology_v1.md` § Answers

---

### Phase W3-D — Extraction + GPU handoff

| Goal | Exit |
|:---|:---|
| `HydrologyVisualExtract` node in projection graph | `water_ocean_tiles > 0` from slab-backed extract |
| Coast / river foam hints → existing `gpu_water_*` | `river_foam`, `coast_foam` witness |
| Strategic ribbon from hydrology, not tile tint alone | `water_strategic_001_green` maintained |

**Do not rush:** until WSS-PLAN-002 snapshot builder exists — may use bridge from current catalog with slab validation witness.

---

### Phase W3-E — Persist dirty regions

| Goal | Exit |
|:---|:---|
| Mark `HydrologyDirtyReason` → `ChunkSlab.dirty` | save/load round-trip one flooded region |
| Cull recomputable scratch on load | designer persistence rule from hydrology v1 |

---

## ECS schedule plan

```text
SimControlSystemSet::AdvanceSimTick
  → ChunkPagingSystem
  → HydrologyBackgroundTick (Tier 1, resident keys)
  → HydrologyEventQueueDrain (apply dirty reasons)
  → schedule_hydrology_deep_tasks (Tier 2, may defer to PostSim job pool)

PostSim / AsyncTaskPool
  → run_hydrology_deep_solve (time-sliced)
  → apply_hydrology_boundary_exchange

PreExtract
  → build_hydrology_visual_extract (from substrate snapshot)
  → RenderProjectionGraph hydrology node
```

**Ordering:** Background tick **after** deformation apply (same frame) if earthworks changed height.

---

## Diagnostics

| Field | Meaning |
|:---|:---|
| `hydrology_resident_chunks` | paging |
| `deep_solve_active_tasks` | scheduler |
| `boundary_exchange_flux_max` | continuity |
| `ocean_tile_count` | slab-backed, not VFX-only |
| `river_channel_cells` | mask sum |
| `waterborne_contamination_max` | cross-domain coupling |

**Witness paths:**
- `debug_runs/wss_substrate_live.json` (substrate)
- `debug_runs/stage5_full_app_live.json` → water particle rows (Layer 3 rollup)

---

## Edge cases

- **Dry river bed:** `water_depth ≈ 0` but `river_mask` persists — ecology riparian strip remains
- **Frozen / snow:** saturation + atmosphere snow_depth — ice flow optional later
- **MP:** server owns deep solve; client interpolates preview (hydrology v1 decision)
- **Chunk boundary T-junction:** boundary exchange must run before extract same frame
- **Deformation during flood:** deep solve pauses, marks dirty, resumes next slice

---

## Anti-patterns (forbidden)

| Pattern | Why |
|:---|:---|
| Global `FluidDomain` Vec | sync hell, no streaming |
| Ocean as render module | authority drift |
| GPU readback of water depth into sim | causality violation without contract |
| Second terrain extract for water | WSS extraction graph rule |
| Skipping persist on “visual flood” | breaks save/load causality |

---

## Open questions (remaining)

1. **Power coupling:** shared event bus vs `HydraulicNode` component — prefer **event bus + tags** per hydrology v1; detail in economy plan
2. **Groundwater tier:** full 3D vs 2.5D table — defer v2; stub `groundwater` Vec in struct now
3. **Client flood prediction rollback** — gameplay plan, not hydrology authority

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Signed — slab hydrology + scheduled tasks, reject FluidDomain |
