# PLAN-WSS-ATMOS-CLIPMAP-EXEC-001 — WSS-ATMOS-CLIPMAP-001 execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-ATMOS-CLIPMAP-EXEC-001** |
| **Slice ID** | **WSS-ATMOS-CLIPMAP-001** |
| **Prior** | [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) (**WSS-PLAN-004 SIGNED**) |
| **Parent** | [`wssr_index_v1.md`](wssr_index_v1.md) (**WSS-PLAN-001**) |
| **Design** | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) · [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **READY** — blocked on **WSS-CHUNK-SLAB-001** types + registry |
| **Suggested owner** | `@coder` A (atmosphere / render boundary) |

**Prereq gates:**

| Gate | Path | Status |
|:---|:---|:---:|
| **WSS-DESIGN-GATE-001** (parent) | [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) | **PASS (qualified)** ☑ |
| **WSS-CHUNK-SLAB-001** (types) | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) | `slab_registry_present` + `WorldChunkState` full types |

---

## Scope

Land **WSS-PLAN-004** phases **W4-A → W4-D** (contamination domain, sim clipmap stack, render clipmap bridge, smoke stub removal). **W4-E/F** (full climate runbook v2, dust transport) are follow-on slices unless planner expands this queue row.

**In scope:**

- `ContaminationState` + `AtmosphereCoupling` on `WorldChunkState` (slab)
- `AtmosphereClipmapStack` L0–L3 sim resource + advect on L0/L1
- Legacy `AtmosphereField` **bridge alias → L1** (hybrid — no delete)
- `AtmosphereRenderClipmap` + `AtmosphereGpuFieldBridge` sole L1→L3 path
- Replace `fire_visual_emit_smoke_stub` with projection-graph smoke extract node (Layer B spine)
- Witness block `wss_atmos_clipmap_001` in `debug_runs/wss_substrate_live.json`
- Designer diagnostics keys (optional P2): `clipmap_l0_smoke_max`, `sim_vs_render_resolution_ratio`

**Out of scope (this slice):**

- Hanabi merge into `EnginePlugin` (experiments spike only — **H-A**)
- Full L3 climate slow tick / seasonal drift (**W4-E**)
- Vehicle dust impulse + soil deposition (**W4-F**)
- Retiring `ChunkWeather` ECS component (**WSS-SLAB-PR-5+**)
- CPU mesh precip as sole authority (demote via flag only)

**Regression guards:** `fire_streaming_live.json`, `fire_ecology_live.json` atmosphere rows, `stage5_full_app_live.json` → `tactical_vfx_witness`, D-F09/D-W09 strategic cull unchanged.

---

## Hybrid migration matrix

| Incumbent | WSS target | Authoritative until | This slice |
|:---|:---|:---|:---|
| `AtmosphereField` 128² | `AtmosphereClipmapStack` L1 alias | **YES** — bridge reads/writes both | dual-write L1 alias |
| `ChunkWeather` component | `ChunkWeatherLocal` in slab | **YES** | sample clipmap → local hazard only |
| `ChunkSmokeField` | fold → L0 `smoke_density` | **YES** | fold input wired, ECS retained |
| `gpu_weather_fire_field` | consumes `AtmosphereRenderClipmap` | L3 unchanged writer | bridge upload only |
| `WeatherVisualPlugin` CPU precip | fallback flag | **YES** | default on until GPU precip signed |
| `fire_visual_emit_smoke_stub` | smoke extract node | **REMOVE** when node green | W4-D exit |

---

## Agreed module layout

| Path | Responsibility |
|:---|:---|
| `src/substrate/atmosphere/mod.rs` | exports, plugin hook |
| `src/substrate/atmosphere/clipmap.rs` | `AtmosphereClipLevel`, `AtmosphereClipmapStack`, `AtmosphereFieldGrid` |
| `src/substrate/atmosphere/contamination.rs` | `ContaminationState`, `AtmosphereCoupling` |
| `src/substrate/atmosphere/bridge_legacy.rs` | `AtmosphereField` ↔ L1 alias sync (hybrid) |
| `src/substrate/atmosphere/render_clipmap.rs` | `AtmosphereRenderClipmap` builder types |
| `src/substrate/atmosphere/live_proof.rs` | extend `WssSubstrateWitness` atmos fields |
| `src/systems/atmosphere/clipmap_advect.rs` | migrate semi-Lagrangian from `advect.rs` |
| `src/systems/atmosphere/contamination_tick.rs` | `ContaminationTickSet` stubs |
| `src/systems/atmosphere/fold_sources.rs` | chunk smoke/fire → L0 sources |
| `src/render/atmosphere_gpu_bridge.rs` | `AtmosphereGpuFieldBridge` (or extend existing bridge module) |
| `src/render/extraction/smoke_visual_extract.rs` | replace stub path |

**Engine wiring:** register `AtmosphereClipmapPlugin` after `SubstratePlugin`; env `RUST_ENGINE_ATMOS_CLIPMAP=0` rollback.

**Schedule sets (new):**

```text
AtmospherePipelineSet::FoldSources
AtmospherePipelineSet::Advect
AtmospherePipelineSet::Coupling
AtmospherePipelineSet::DepositWashout
```

Insert **after** `ChunkEnvironmentSet::Weather` fold inputs, **before** render extract.

---

## Authority map

| Resource | Single writer | Layer |
|:---|:---|:---:|
| `AtmosphereClipmapStack` | `AtmospherePipelineSet::Advect` | L1 sim |
| `ContaminationState` (per chunk in slab) | `ContaminationTickSet` | L1 sim |
| `AtmosphereCoupling` (per chunk) | `AtmospherePipelineSet::Coupling` | L1 sim |
| `ChunkWeather` / `ChunkWeatherLocal` | `ChunkEnvironmentSet::Weather` | L1 (ECS + slab mirror read) |
| `AtmosphereField` (legacy) | bridge sync from L1 | transitional |
| `AtmosphereRenderClipmap` | PreExtract builder | L2 |
| `ClimateVisualAggregate` / smoke extract | projection graph nodes | L2 |
| `gpu_weather_fire_field` textures | GPU upload | L3 |
| `wss_substrate_live.json` → `wss_atmos_clipmap_001` | `write_wss_substrate_live_proof_system` | witness |

**Forbidden:** render `extract/*` queries `ChunkWeather` directly; merge contamination into `AtmosphereCell`; GPU field → sim readback without contract; global strategic cull disable for witness greens.

---

## Task list (AC-001 … AC-008)

### AC-001 — Contamination + coupling types (W4-A)

1. Add `ContaminationState`, `AtmosphereCoupling` to `WorldChunkState` in `src/substrate/types.rs`.
2. Per-cell `Vec` lengths = `CELL_COUNT` for airborne/soil/waterborne/bioactive/radiation.
3. `ContaminationTickSet` — deposition + rain washout **stubs** (lib test: rain reduces airborne).

**Exit:** `contamination_domain_present: true` in witness.

---

### AC-002 — Clipmap stack scaffold (W4-B partial)

1. `AtmosphereClipmapStack` with L0–L3 `AtmosphereClipLevel` structs.
2. Default resolutions documented in module doc (tunable constants).
3. `active_focus: DVec2` from sim focus / camera stub (reuse Stage 6 focus where possible).

**Exit:** `clipmap_levels_present: true`, `clipmap_level_count == 4`.

---

### AC-003 — Legacy bridge (hybrid)

1. `bridge_legacy.rs`: on advect tick end, copy L1 grid ↔ existing `AtmosphereField` resource.
2. Existing `systems/atmosphere/update.rs` fold paths **unchanged** entrypoints — call into clipmap fold.
3. Witness: `legacy_atmosphere_field_bridged: true`.

**Exit:** existing atmosphere lib tests pass without deleting `AtmosphereField`.

---

### AC-004 — Advect + fold sources (W4-B)

1. Migrate semi-Lagrangian advect to `clipmap_advect.rs` for L0/L1.
2. `fold_sources`: `ChunkSmokeField` + fire smoke gen → L0 `smoke_density`.
3. Sample clipmap at chunk center → update `toxic_hazard` sample on local state (display scalar only).

**Exit:** lib test `clipmap_advect_preserves_mass_approximately`; `clipmap_l0_smoke_max > 0` after fire fixture tick.

---

### AC-005 — Render clipmap + GPU bridge (W4-C)

1. `AtmosphereRenderClipmap` builder in PreExtract — may downsample, strip pressure.
2. `AtmosphereGpuFieldBridge` — sole path from sim/render clipmap to `gpu_weather_fire_field`.
3. Witness: `sim_vs_render_resolution_ratio` documented (render ≤ sim per level).
4. `gpu_partial_upload_count` counter (partial upload intent).

**Exit:** tactical harness still shows atmosphere; no new extract pass for weather field.

---

### AC-006 — Smoke stub removal (W4-D)

1. Implement `build_smoke_visual_extract` node in projection graph.
2. Remove or no-op `fire_visual_emit_smoke_stub`.
3. Layer B composite reads render clipmap smoke channel.

**Exit:** `smoke_stub_removed: true`; `stage5_full_app_live.json` tactical smoke rows unchanged or improved.

---

### AC-007 — Live witness extension

Extend `src/substrate/live_proof.rs` (or `atmosphere/live_proof.rs`) with nested block:

```json
"wss_atmos_clipmap_001": { ... }
```

Register related proofs in envelope `related_proofs`.

---

### AC-008 — Lib tests

```powershell
cargo test -p proc_A_dine01 --lib atmosphere_clipmap contamination_tick atmosphere_bridge
```

Predicates: see § Test predicates.

---

## Test predicates

### `contamination_rain_washout_stub`

```text
GIVEN chunk with airborne=1.0, soil=0
WHEN ContaminationTickSet runs with rain_intensity > 0.5
THEN airborne decreases AND soil increases OR washout flag set
```

### `clipmap_levels_initialized`

```text
GIVEN app with AtmosphereClipmapPlugin
WHEN startup
THEN stack.levels.len() == 4 AND each fields.smoke_density.len() > 0
```

### `legacy_field_l1_alias_roundtrip`

```text
GIVEN AtmosphereField with non-zero cell
WHEN bridge sync to L1 and back
THEN legacy field cell within epsilon of original
```

### `smoke_fold_after_fire_tick`

```text
GIVEN chunk with active surface fire
WHEN ChunkEnvironmentSet::Fire + FoldSources
THEN clipmap L0 smoke_density max > 0
```

### `render_clipmap_resolution_lte_sim`

```text
GIVEN sim L0 resolution R
WHEN render clipmap built
THEN render L0 resolution <= R per axis
```

### `no_chunk_weather_in_extract`

```text
GIVEN rg ChunkWeather src/render/extraction/
WHEN smoke/weather extract nodes
THEN no direct Query<ChunkWeather> in extract (bridge only)
```

---

## Witness JSON — `wss_atmos_clipmap_001`

**Path:** `debug_runs/wss_substrate_live.json` (nested block)

| JSON pointer | Type | PR-1 slice semantics |
|:---|:---|:---|
| `/wss_atmos_clipmap_001/gate` | string | `"WSS-ATMOS-CLIPMAP-001"` |
| `/wss_atmos_clipmap_001/green` | bool | rollup |
| `/wss_atmos_clipmap_001/contamination_domain_present` | bool | types + tick wired |
| `/wss_atmos_clipmap_001/clipmap_levels_present` | bool | L0–L3 exist |
| `/wss_atmos_clipmap_001/clipmap_level_count` | number | `4` |
| `/wss_atmos_clipmap_001/legacy_atmosphere_field_bridged` | bool | alias sync on |
| `/wss_atmos_clipmap_001/clipmap_advect_wired` | bool | L0/L1 advect ran |
| `/wss_atmos_clipmap_001/render_clipmap_wired` | bool | builder registered |
| `/wss_atmos_clipmap_001/sim_vs_render_resolution_ratio` | number | render/sim cell ratio ≤1 |
| `/wss_atmos_clipmap_001/smoke_stub_removed` | bool | stub gone / no-op |
| `/wss_atmos_clipmap_001/gpu_partial_upload_count` | number | ≥0 |
| `/wss_atmos_clipmap_001/clipmap_l0_smoke_max` | number | diagnostic |
| `/wss_atmos_clipmap_001/toxic_hazard_sample` | number | focus sample |
| `/wss_atmos_clipmap_001/hanabi_spike_report_present` | bool | `experiments/hanabi_validation/REPORT.md` exists (optional) |

### Green predicate (slice v1)

```text
wss_atmos_clipmap_001_green :=
  gate == "WSS-ATMOS-CLIPMAP-001"
  AND contamination_domain_present
  AND clipmap_levels_present
  AND legacy_atmosphere_field_bridged
  AND clipmap_advect_wired
  AND render_clipmap_wired
  AND smoke_stub_removed
  AND fire_ecology_live.json unchanged (manual compare)
  AND stage5 tactical_vfx_witness all_green (manual compare)
```

---

## ECS schedule (this slice)

```text
ChunkEnvironmentSet::Weather          → ChunkWeather WRITER (unchanged)
ChunkEnvironmentSet::Fire             → smoke gen WRITER (unchanged)
AtmospherePipelineSet::FoldSources    → clipmap L0 sources
AtmospherePipelineSet::Advect         → AtmosphereClipmapStack WRITER
AtmospherePipelineSet::Coupling       → ContaminationState + coupling
AtmospherePipelineSet::DepositWashout → cross-domain stubs

PreExtract
  → build_atmosphere_render_clipmap
  → build_smoke_visual_extract
  → RenderProjectionGraph atmosphere nodes

Render
  → AtmosphereGpuFieldBridge → gpu_weather_fire_field
```

**Do not** run advect after render extract in same frame before snapshot.

---

## Verification commands

```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib atmosphere_clipmap contamination_tick
cargo test -p proc_A_dine01 --lib stage5 fire_streaming
cargo test -p proc_A_dine01 --lib fire_ecology
```

---

## Anti-patterns (forbidden)

| Anti-pattern | Why |
|:---|:---|
| Delete `AtmosphereField` before bridge green | hybrid violation |
| Merge `ContaminationState` into `AtmosphereCell` | WSS-PLAN-004 |
| Hanabi as weather authority | design gate |
| `ChunkWeather` in extract | L2 contract |
| Disable D-F09/D-W09 globally | witness cheat |
| Second full-grid GPU upload every idle frame | perf regression |

---

## Rollback

- `RUST_ENGINE_ATMOS_CLIPMAP=0` disables new systems; legacy field path only
- `fire_ecology_live.json` regression → disable bridge, file steward ticket

---

## Coder assignment

| Field | Value |
|:---|:---|
| **Slice** | WSS-ATMOS-CLIPMAP-001 |
| **Blocked until** | `wss_substrate_live.json` → `/slab_registry_present` + `/chunk_count > 0` |
| **Budget** | ≤10 new/modified files per PR; split AC-001..004 then AC-005..006 |
| **Playbook** | `render_pipeline_agent` + bevy-simulation-grade |
| **Unblocks** | W4-E climate, W4-F dust, Hanabi H-B (after spike) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Full exec plan — mirrors chunk-slab exec pattern |
