# WSS-PLAN-004 — Atmosphere unification & clipmaps `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-PLAN-004** |
| **Parent** | [`wssr_index_v1.md`](wssr_index_v1.md) (**WSS-PLAN-001**) |
| **Depends on** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) (**WSS-PLAN-002**) |
| **Related** | [`wssr_plan_003_hydrology_runtime_v1.md`](wssr_plan_003_hydrology_runtime_v1.md) (evaporation / toxic rain) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Prior art** | [`base_fire2_smoke.md`](../prompts/guides/base_fire2_smoke.md) · [`weather_simulation_runbook_v1.md`](../prompts/guides/weather_simulation_runbook_v1.md) |
| **Coder unblock** | **WSS-ATMOS-CLIPMAP-001** |

**No Rust in this deliverable.** Replace fixed 128² `AtmosphereField` thinking with **hierarchical simulation clipmaps**. **Contamination is separate** from `AtmosphereCell`. Hanabi: **spike now**, integrate later.

---

## Summary

Atmosphere **simulation authority** spans **regional clipmaps** (L0–L3) plus **per-chunk local state** in `WorldChunkState.atmosphere`. **`ContaminationState`** is a **separate domain** with **`AtmosphereCoupling`** for transport — supporting toxic rain, soil pollution, waterborne plumes, and radiation without polluting core atmosphere logic. **Smoke** splits into Layer A persistent field + Layer B GPU representation. **Dust** is atmosphere transport + contamination soil/airborne — not vehicle VFX silo. **Simulation clipmaps ≠ render clipmaps.**

---

## Current problems

| Issue | Evidence |
|:---|:---|
| Fixed 128² `AtmosphereField` | `systems/atmosphere/field.rs` — will not scale planetary |
| Three partial weather/smoke authorities | `ChunkWeather`, `AtmosphereField`, `gpu_weather_fire_field` |
| `fire_visual_emit_smoke_stub` | incomplete Layer B bridge |
| CPU mesh precip (`WeatherVisualPlugin`) | Layer 3 scaffold acting like authority |
| No contamination domain | toxicity only in `AtmosphereCell` — insufficient |
| GPU field visual-only but adjacent to sim | readback hazard |

---

## Target architecture

### Contamination (separate domain)

```rust
/// Per-chunk contamination — NOT merged into AtmosphereCell.
pub struct ContaminationState {
    pub airborne: Vec<f32>,      // respirable / plume
    pub soil: Vec<f32>,          // industrial spill, pesticides
    pub waterborne: Vec<f32>,    // runoff, algal bloom proxy
    pub bioactive: Vec<f32>,     // organic toxin / epidemic proxy
    pub radiation: Vec<f32>,     // fallout / waste
}

/// Cross-domain coupling — atmosphere moves airborne; hydrology moves waterborne.
pub struct AtmosphereCoupling {
    pub wind_transport: f32,     // airborne advection strength
    pub humidity_binding: f32,   // wet deposition / toxic rain
    pub thermal_exchange: f32,   // heat ↔ smoke buoyancy
    pub precipitation_washout: f32,
}
```

**Coupling rules:**

```text
ContaminationState.airborne  ←→  atmosphere clipmap advection (via AtmosphereCoupling)
ContaminationState.soil      ←   deposition from airborne + industrial events
ContaminationState.waterborne ←→  hydrology flow (WSS-PLAN-003)
EcologyState.stress          ←   f(soil + waterborne contamination)
AtmosphereCell               ←   derived visibility/toxic hazard sample, NOT storage of all toxin types
```

### Atmosphere clipmap hierarchy (simulation)

**NOT fixed 128².** Planetary-ish scale + tactical zoom + minimap + weather fronts require multi-resolution.

```rust
pub enum AtmosphereClipLevelId {
    L0, // local high detail   — ~256m–1km effective cell (tunable)
    L1, // regional            — storms, smoke columns
    L2, // continental         — fronts, drought bands
    L3, // planetary           — climate background, orbital preview
}

pub struct AtmosphereClipLevel {
    pub id: AtmosphereClipLevelId,
    pub resolution: UVec2,
    pub cell_size_world: f32,    // meters per cell at this level
    pub origin_world: DVec2,
    pub fields: AtmosphereFieldGrid,
}

pub struct AtmosphereFieldGrid {
    pub smoke_density: Vec<f32>,
    pub fog_density: Vec<f32>,
    pub heat: Vec<f32>,
    pub ash_density: Vec<f32>,   // dust/ash transport (airborne particulate)
    pub humidity: Vec<f32>,
    pub pressure: Vec<f32>,      // optional slow field
    pub wind: Vec<Vec2>,
    pub visibility: Vec<f32>,
    /// Gameplay hazard sample — NOT full contamination storage.
    pub toxic_hazard: Vec<f32>,
}

#[derive(Resource)]
pub struct AtmosphereClipmapStack {
    pub levels: [AtmosphereClipLevel; 4],
    pub active_focus: DVec2,
    pub last_advect_tick: u64,
}
```

**Per-chunk local (fast):**

```rust
pub struct ChunkWeatherLocal {
    pub rain_intensity: f32,
    pub fog_density: f32,
    pub snow_depth: f32,
    pub wind_speed: f32,
    pub lightning_risk: f32,
    pub visibility_factor: f32,
    pub soil_moisture: f32,
}

pub struct AtmosphereState {
    pub local: ChunkWeatherLocal,
    /// Indices / weights for sampling L0–L3 at chunk center.
    pub clipmap_sample: ClipmapSampleRef,
}
```

### Render clipmaps (separate from sim)

```rust
#[derive(Resource)]
pub struct AtmosphereRenderClipmap {
    pub levels: Vec<GpuAtmosphereClipLevel>,
    /// May downsample, temporal filter, or omit pressure entirely.
    pub last_upload_tick: u64,
}
```

**CRITICAL:** `AtmosphereRenderClipmap` **DERIVES** from `AtmosphereClipmapStack` + extraction — different resolution, different upload cadence, optional fields stripped.

```text
AtmosphereClipmapStack (sim)     OWNS simulation truth
AtmosphereRenderClipmap (GPU)    DERIVES for visualization
gpu_weather_fire_field           CONSUMES render clipmap / partial uploads
WeatherVisualPlugin mesh precip  TRANSITIONAL Layer 3 — retire as authority
```

### Smoke — Layer A / Layer B

```text
Layer A — Simulation (persistent)
  ChunkSmokeField (chunk component → migrate to slab thermal/smoke gen)
  → fold into AtmosphereClipmapStack L0/L1 smoke_density
  → ContaminationState.airborne coupling on toxic burns
  → saveable, sensor visibility, AI hazard

Layer B — GPU representation (transient)
  RenderProjectionGraph atmosphere/smoke node
  → AtmosphereRenderClipmap upload
  → gpu_weather_fire_field ping-pong (visual diffusion OK)
  → volumetric haze / ground fog composite
  → Hanabi: local wisps, ember puffs (Layer 3 embellishment)
```

**Replace:** `fire_visual_emit_smoke_stub` → real smoke extract node feeding Layer B.

### Dust — not a silo

```text
Sources:
  erosion (terrain + hydrology sediment)
  vehicles / convoys (impulse events → ash_density increment)
  desert biome wind pickup
  collapse / explosion events
  battlefield disturbance

Transport:
  AtmosphereClipmapStack.ash_density + wind advection
  deposit → ContaminationState.soil

Visualization:
  ground haze composite + strategic overlay
  Hanabi: kick-up puff on event only
```

### Weather sim expansion (runbook alignment)

```text
Climate (L3, slow) → Regional cells (L2/L1) → ChunkWeatherLocal (L0)
  → feeds hydrology evaporation (WSS-PLAN-003)
  → feeds contamination washout
  → extraction → ClimateVisualAggregate → precip / fog overlays
```

**Do not** implement weather as Hanabi rain system at world scale.

---

## Hanabi — research spike (now) vs adoption (later)

### Phase H-A — Non-blocking research spike (schedule immediately)

Create **`experiments/hanabi_validation/`** (standalone crate or `--example`):

| Test | Pass criteria |
|:---|:---|
| Bevy 0.18 pin compatibility | compiles with engine wgpu stack |
| Custom extraction coexistence | Hanabi + `gpu_weather_fire_field` same frame |
| Multiview | particles respect `ViewId` / render layers |
| Minimap bleed | no Hanabi draw on minimap unless intended |
| GPU bandwidth | N particles @ budget (document threshold) |
| Volumetric coexistence | no pipeline conflict with field compute |
| Determinism | Hanabi does not write sim state |

**Output doc:** `experiments/hanabi_validation/REPORT.md` → routes to `@coder` or defer with pinned version.

**Do NOT:** merge Hanabi into main `EnginePlugin` until:
- projection graph stable (F-T02)
- per-view extraction landed
- atmosphere clipmap substrate (W4-B) started

### Phase H-B — Scoped adoption (later)

Hanabi **ONLY** for: embers, sparks, debris, explosion puffs, local smoke wisps.

Hanabi **NEVER** for: weather sim, smoke authority, terrain state, strategic atmosphere.

---

## Authority map

| Resource | Writer | Layer |
|:---|:---|:---:|
| `AtmosphereClipmapStack` | `AtmosphereAdvectSet` | L1 |
| `ContaminationState` (per chunk slab) | `ContaminationTickSet` | L1 |
| `AtmosphereCoupling` (per chunk) | `AtmosphereCouplingSet` | L1 |
| `ChunkWeatherLocal` | `ChunkEnvironmentSet::Weather` | L1 |
| `ClimateVisualAggregate` | atmosphere visual extract | L2 |
| `SimChunkSmokeVisualExtract` | smoke extract node | L2 |
| `AtmosphereRenderClipmap` | render clipmap builder | L2 |
| `gpu_weather_fire_field` textures | GPU upload / compute | L3 |
| `WeatherVisualPlugin` | **transitional** L3 | L3 |
| Hanabi instances | event VFX systems | L3 |

**Forbidden:** `ChunkWeather` query in `render/extraction/*`; merging `ContaminationState` into `AtmosphereCell`; GPU field → sim readback without contract.

---

## Implementation phases

### Phase W4-A — Contamination domain + coupling

| Goal | Exit |
|:---|:---|
| Add `ContaminationState` + `AtmosphereCoupling` to `WorldChunkState` | types in world_substrate |
| `ContaminationTickSet` — deposition / washout stubs | lib test: rain reduces airborne |
| Wire ecology stress read | single test chunk |

---

### Phase W4-B — Clipmap stack (sim)

| Goal | Exit |
|:---|:---|
| `AtmosphereClipmapStack` with L0–L3 struct | replace single 128² default |
| Fold `ChunkSurfaceFire` + `ChunkSmokeField` → L0 smoke | non-zero smoke after fire tick |
| Semi-Lagrangian advect on L0/L1 (migrate from `atmosphere/advect.rs`) | wind transport test |
| Sample clipmap at chunk center → update hazard | visibility attenuation sample |

**Migration bridge:** keep legacy `AtmosphereField` as **alias to L1** until witness green — then remove.

---

### Phase W4-C — Render clipmap + GPU bridge

| Goal | Exit |
|:---|:---|
| `AtmosphereRenderClipmap` builder from sim stack | upload metrics in witness |
| `AtmosphereGpuFieldBridge` sole L1→L3 path | no direct ChunkWeather in bridge |
| Retire full-field dispatch every idle frame | partial upload counters (P2-H intent) |

---

### Phase W4-D — Smoke stub → extraction spine

| Goal | Exit |
|:---|:---|
| Remove `fire_visual_emit_smoke_stub` | smoke node in projection graph |
| Layer B volumetric/haze composite reads render clipmap | tactical harness smoke visible |
| Hanabi wisps optional after H-A report | not blocking |

---

### Phase W4-E — Weather runbook v2 wiring

| Goal | Exit |
|:---|:---|
| L3 climate slow tick → L2 regional | seasonal drift test |
| Regional → chunk local interpolation | chunk rain matches regional storm cell |
| `GlobalRenewableWeatherFactors` from clipmap | existing renewables test green |
| CPU mesh precip demoted to fallback flag | `WeatherVisualSettings` default off when GPU precip ready |

---

### Phase W4-F — Dust transport

| Goal | Exit |
|:---|:---|
| Vehicle/event impulse → L0 ash_density increment | convoy kick test |
| Deposition → `ContaminationState.soil` | dust storm fixture |
| Strategic haze overlay via extraction | minimap readability check |

---

## ECS schedule plan

```text
ChunkEnvironmentSet::Weather          → ChunkWeatherLocal WRITER
ChunkEnvironmentSet::Fire             → smoke gen → fold input
ContaminationTickSet                  → ContaminationState WRITER
AtmospherePipelineSet::FoldSources    → clipmap L0 source terms
AtmospherePipelineSet::Advect         → AtmosphereClipmapStack WRITER
AtmospherePipelineSet::Coupling       → AtmosphereCoupling + cross-domain
AtmospherePipelineSet::DepositWashout → airborne ↔ soil ↔ hydrology

PreExtract
  → build_climate_visual_aggregate
  → build_smoke_visual_extract
  → build_atmosphere_render_clipmap
  → RenderProjectionGraph atmosphere nodes

Render Extract
  → AtmosphereGpuFieldBridge
  → gpu_weather_fire_field compute
  → composites + (later) Hanabi embellishment
```

---

## Diagnostics

| Metric | Source |
|:---|:---|
| `clipmap_l0_smoke_max` | sim stack |
| `sim_vs_render_resolution_ratio` | clipmap builder |
| `contamination_airborne_max` | slab |
| `toxic_hazard_sample` | L0 cell at focus |
| `gpu_partial_upload_count` | bridge |
| `smoke_stub_removed` | bool witness |
| `hanabi_spike_report_present` | experiments/ |

**Witness:** `debug_runs/wss_substrate_live.json` + extend `fire_ecology_live.json` atmosphere rows

---

## Edge cases

- **Clipmap focus shift / streaming:** re-center L0/L1 origins on camera/sim focus — like terrain paging
- **L3 planetary + L0 local mismatch:** sample weights must blend smoothly at boundaries
- **Toxic rain:** `humidity_binding` + `ContaminationState.airborne` → `waterborne` via hydrology rain event
- **Nuclear fallout:** `radiation` half-life decay in `ContaminationTickSet` — separate from smoke advection
- **Minimap:** compositor reads **compressed L2/L3** strategic channel — not L0 sim grid
- **Strategic cull for particles:** Layer 3 only — sim clipmap still advances

---

## Library additions (parallel)

| Library | When | Purpose |
|:---|:---|:---|
| `bevy_vector_shapes` | W4-D or tactical overlay slice | wind vectors, authority debug, magenta wire aesthetic |
| `bevy_mod_outline` | selection pass | tactical readability |
| Hanabi | after H-A report + W4-C | ember/wisp Layer 3 |

---

## Anti-patterns (forbidden)

| Pattern | Why |
|:---|:---|
| Fixed single 128² forever | planetary scale failure |
| Tight sim/render resolution coupling | cannot optimize independently |
| Merging contamination into `AtmosphereCell` | loses soil/water/structure toxins |
| Hanabi as weather authority | wrong abstraction |
| Disabling strategic cull globally for witness | breaks D-F09 / D-W09 |
| `WeatherVisualPlugin` as long-term rain authority | CPU mesh does not scale |

---

## Open questions (remaining)

1. **L3 climate data source:** procedural only vs saveable climate seed — recommend saveable seed in world header
2. **Lightning:** event bus vs clipmap `lightning_risk` threshold — event bus for strikes, field for risk
3. **Orbital view:** L3 render-only clip — sim may skip L0 when focus in orbit mode

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Signed — clipmaps, contamination split, Hanabi spike policy |
