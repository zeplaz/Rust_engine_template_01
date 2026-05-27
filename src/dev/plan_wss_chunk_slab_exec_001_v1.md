# PLAN-WSS-CHUNK-SLAB-EXEC-001 — WSS-CHUNK-SLAB-001 execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-CHUNK-SLAB-EXEC-001** |
| **Slice ID** | **WSS-CHUNK-SLAB-001** |
| **Prior** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) (**WSS-PLAN-002 SIGNED**) |
| **Parent** | [`wssr_index_v1.md`](wssr_index_v1.md) (**WSS-PLAN-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **READY** — **WSS-DESIGN-GATE-001 PASS (qualified)** 2026-05-26 |
| **Coder assignment** | **WSS-CHUNK-SLAB-001** on `coder_active_queue.json` |

**Gate record (required before coder):**

| Gate | Path | Status |
|:---|:---|:---:|
| **WSS-DESIGN-GATE-001** (parent) | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) + deliverable [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) | **OPEN** |
| **WSS-DESIGN-GATE-001** (slab preflight) | [`wss_design_gate_001_v1.md`](wss_design_gate_001_v1.md) — G1–G6 operational checklist | **PENDING** |

Both must PASS before `@coder` assignment.

---

## Scope

Land **Phase W2-A** from WSS-PLAN-002:

- `ChunkKey`, `ChunkSlab<T>`, full `WorldChunkState` types
- `WorldSubstrateRegistry` resource + `SubstratePlugin`
- Gen hydrate bridge (terrain + hydrology masks from existing worldgen)
- Lib tests for **dirty / resident / paging** predicates
- `debug_runs/wss_substrate_live.json` witness writer (skeleton green)

**Explicitly out of scope for this slice:**

- Dual-write shim ( **WSS-SLAB-PR-2** )
- `ActiveChunkRuntime` ECS hybrid ( **WSS-SLAB-PR-3** )
- RON persist round-trip ( **WSS-SLAB-PR-4** )
- Removing legacy ECS components ( **WSS-SLAB-PR-5+** )
- Atmosphere clipmaps (WSS-PLAN-004)
- Hydrology runtime tick (WSS-PLAN-003)

**Regression guard:** `ChunkWeather`, `ChunkSurfaceFire`, and all existing `ChunkEnvironmentSet` systems **remain authoritative** until their coupling PR lands — see § Hybrid migration matrix.

---

## Design gate (hard blocker)

```text
WSS-DESIGN-GATE-001 PASS  →  @coder may pick up WSS-CHUNK-SLAB-001
WSS-DESIGN-GATE-001 FAIL  →  planner/designer revise; NO coder queue row
```

Gate checklist lives in [`wss_design_gate_001_v1.md`](wss_design_gate_001_v1.md). Minimum PASS requires:

| # | Check | Owner |
|:---:|:---|:---|
| G1 | Hybrid migration UX documented — operators/devs know ECS components still live | `@designer` |
| G2 | Diagnostics overlay names for slab `resident` / `dirty` / `chunk_count` | `@designer` |
| G3 | No player-facing regression from plugin init (empty world, editor, sim) | `@sim-steward` |
| G4 | `ChunkEnvironmentSet` ordering preserved — witness or lib test | `@sim-steward` |
| G5 | Module path **`src/substrate/`** confirmed (this exec plan) | `@planner` ☑ |

**Orchestrator rule:** `coder_active_queue.json` must **not** contain `WSS-CHUNK-SLAB-001` until `/gate == "WSS-DESIGN-GATE-001"` and `/pass == true` in gate record.

---

## Agreed module layout

**Module root:** `src/substrate/` (agreed over WSS-PLAN-002 draft `world_substrate/`).

| Path | Responsibility |
|:---|:---|
| `src/substrate/mod.rs` | Module exports, `SubstratePlugin`, `configure_sets` hook (stub) |
| `src/substrate/key.rs` | `ChunkKey`, conversions `IVec2` ↔ `ChunkKey` |
| `src/substrate/slab.rs` | `ChunkSlab<T>`, dirty/resident/paging API |
| `src/substrate/types.rs` | `WorldChunkState` + domain structs (full expanded shapes from WSS-PLAN-002) |
| `src/substrate/registry.rs` | `WorldSubstrateRegistry`, `ChunkPagingState` |
| `src/substrate/persist.rs` | `SubstratePersistBook` API stub (no RON yet) |
| `src/substrate/hydrate.rs` | Gen → slab hydrate from `Chunk` spawn / matrix |
| `src/substrate/live_proof.rs` | `WssSubstrateWitness`, JSON writer |
| `src/substrate/tests.rs` | `#[cfg(test)]` predicates (or inline `mod tests` per file) |

**Engine wiring (2 files outside module):**

| Path | Change |
|:---|:---|
| `src/lib.rs` | `pub mod substrate;` |
| `src/engine/mod.rs` or root plugin | `app.add_plugins(substrate::SubstratePlugin)` — **feature-flag or env gate** for rollback |

**Cell grid contract (v1):** all per-cell `Vec` fields in `WorldChunkState` use **`ChunkCellMatrix::CELL_COUNT`** (same as terrain height within chunk). Document constant in `types.rs`.

---

## Authority map (this slice only)

| Resource | Single writer | Allowed mutation |
|:---|:---|:---|
| `WorldSubstrateRegistry` | `SubstratePlugin` init + `hydrate_chunk_into_substrate` + `sync_substrate_paging` | insert slab, set resident, mark dirty via `ChunkSlab` API |
| `ChunkSlab::dirty` | only via `ChunkSlab::get_mut`, `insert`, `mark_dirty` | keys touched by hydrate |
| `ChunkSlab::resident` | `sync_substrate_paging` | read Stage 6 / focus stub for v1 |
| `debug_runs/wss_substrate_live.json` | `write_wss_substrate_live_proof_system` | envelope + witness payload |
| `ChunkWeather` | **unchanged** — `weather_chunk_tick` | **still authoritative** |
| `ChunkSurfaceFire` | **unchanged** — `chunk_surface_fire_tick` | **still authoritative** |

**Forbidden this slice:** dual-write to slab from weather/fire ticks; render/extract reads of `WorldSubstrateRegistry`; second `HashMap` for chunk state outside registry.

---

## Hybrid migration matrix (ECS remains until PR N)

Legacy ECS components **stay live and authoritative** for sim + extract. Slab is **hydrate + witness only** in WSS-CHUNK-SLAB-001.

| Legacy ECS / resource | Slab field (target) | Still authoritative until PR | Coupling PR |
|:---|:---|:---|:---|
| `ChunkWeather` component | `WorldChunkState.atmosphere.local` | **YES** — all weather sim + visuals | **WSS-SLAB-PR-2** (dual-write shim) |
| `ChunkSurfaceFire` component | `WorldChunkState.thermal` | **YES** — fire sim + ecology witness | **WSS-SLAB-PR-2** |
| `ChunkSmokeField` component | fold → atmosphere (later clipmap) | **YES** | **WSS-SLAB-PR-5** |
| `ChunkFireOverlay` component | `thermal` + overlay heat | **YES** | **WSS-SLAB-PR-5** |
| `FireFuelField` / `ChunkEcology` | `ecology` | **YES** | **WSS-SLAB-PR-5** |
| `DynamicTerrainOverlay` resource | `dynamic` per chunk | **YES** | **WSS-SLAB-PR-4** |
| `AtmosphereField` grid | clipmap L1 (WSS-PLAN-004) | **YES** | **WSS-ATMOS-CLIPMAP-001** |
| Gen `HydrologyResult` | `hydrology` hydrate on spawn | Slab **read-only mirror** after hydrate in PR-1 | runtime tick **WSS-HYDRO-RUNTIME-001** |

**PR-1 (this exec plan):** spawn/hydrate slab; **no** sim tick writes to slab from weather/fire.  
**PR-2:** bidirectional shim + drift witness.  
**PR-3:** `ActiveChunkRuntime` hybrid entities.  
**PR-4:** persist book + dynamic overlay migration.  
**PR-5:** retire ECS components when drift `0` for N frames in CI fixture.

---

## Task list (CS-001 … CS-006)

### CS-001 — Core types + slab API

1. Implement `ChunkKey`, `ChunkSlab<T>` with `chunks`, `resident`, `dirty`.
2. Public API:
   - `insert`, `get`, `get_mut` (marks dirty)
   - `mark_dirty`, `clear_dirty`, `dirty_count`
   - `set_resident`, `clear_resident`, `is_resident`, `resident_count`
   - `contains`, `len`
3. `WorldChunkState` + nested domain structs — **full fields from WSS-PLAN-002**, not truncated stubs.

**Files:** `key.rs`, `slab.rs`, `types.rs`

---

### CS-002 — Registry + plugin

1. `WorldSubstrateRegistry { chunks: ChunkSlab<WorldChunkState>, paging, persist }`
2. `SubstratePlugin`: init resources, register live proof systems
3. Env rollback: `RUST_ENGINE_SUBSTRATE=0` skips plugin systems (pattern: other feature gates in repo)

**Files:** `registry.rs`, `persist.rs`, `mod.rs`, engine plugin wiring

---

### CS-003 — Hydrate bridge (PR-1)

1. On `Chunk` entity spawn (or matrix batch), call `hydrate_chunk_into_substrate`.
2. Copy height/material from `ChunkCellMatrix` → `TerrainState`.
3. Copy river/lake/ocean markers from gen hooks if present on entity — else zero masks.
4. Initialize `atmosphere.local` from `ChunkWeather::default()` (**do not** read live component yet).
5. Mark key `resident` if inside default paging radius (stub: single-chunk test uses `{key}`).

**Files:** `hydrate.rs`

**Must not:** mutate `ChunkWeather` / `ChunkSurfaceFire` from hydrate.

---

### CS-004 — Paging stub

1. `sync_substrate_paging` system after `SimControlSystemSet::AdvanceSimTick` (or `ChunkEnvironmentSet::Lod` **before** weather — read-only resident set).
2. v1: resident = all keys in slab with chunk entity present (simple); optional read `CameraFocusDebug` if available without new authority.
3. Evict from `resident` only — **never** remove from `chunks` in this slice.

**Files:** `registry.rs`, `mod.rs`

---

### CS-005 — Live witness

1. `WssSubstrateWitness` resource — rolling metrics.
2. `write_wss_substrate_live_proof_system` — writes `debug_runs/wss_substrate_live.json` with envelope.
3. Register path in `KNOWN_LIVE_PROOF_PATHS` (`debug_run_envelope.rs`).

**Files:** `live_proof.rs`, `debug_run_envelope.rs`

---

### CS-006 — Lib tests

Implement tests in `src/substrate/tests.rs` (see § Test predicates).

**Files:** `tests.rs`

---

## Test predicates

Run:

```powershell
cargo test -p proc_A_dine01 --lib substrate
cargo test -p proc_A_dine01 --lib chunk_environment_set
```

### `slab_insert_marks_dirty`

```text
GIVEN empty ChunkSlab
WHEN insert(key, state)
THEN dirty.contains(key) AND resident is unchanged AND len == 1
```

### `slab_get_mut_marks_dirty`

```text
GIVEN slab with key inserted and clear_dirty() called
WHEN get_mut(key) succeeds
THEN dirty.contains(key)
```

### `slab_get_readonly_no_dirty`

```text
GIVEN slab with key, dirty cleared
WHEN get(key) only
THEN NOT dirty.contains(key)
```

### `slab_resident_independent_of_dirty`

```text
GIVEN key in slab, not resident
WHEN set_resident(key)
THEN resident.contains(key) AND dirty unchanged unless get_mut called
WHEN clear_resident(key)
THEN NOT resident.contains(key)
```

### `slab_clear_dirty_flushes_bookkeeping`

```text
GIVEN multiple dirty keys
WHEN clear_dirty() on one key
THEN dirty_count decrements; other keys remain
WHEN clear_all_dirty()
THEN dirty_count == 0
```

### `slab_paging_resident_subset`

```text
GIVEN slab with keys A, B, C
WHEN set_resident only {A, B}
THEN resident_count == 2 AND chunks.len() == 3
```

### `registry_hydrate_on_chunk_spawn`

```text
GIVEN minimal app with SubstratePlugin + one Chunk entity at coord (2,3)
WHEN one Update
THEN registry.chunks.contains(ChunkKey {2,3})
AND terrain.height.len() == CELL_COUNT
AND hydrology masks len == CELL_COUNT
```

### `hydrate_does_not_touch_chunk_weather_component`

```text
GIVEN Chunk + ChunkWeather with rain_intensity = 0.7
WHEN hydrate runs
THEN component rain still 0.7
AND slab atmosphere.local.rain_intensity == default (0.0) — proves no premature coupling
```

### `chunk_environment_order_unchanged_with_plugin`

```text
GIVEN app with SubstratePlugin + chunk_environment_set fixtures
WHEN update
THEN order Lod → Weather → Ecology → Fire preserved (existing test pattern)
```

### `witness_writer_produces_schema`

```text
GIVEN proof system ran in test harness
THEN JSON has gate, green, slab_registry_present, chunk_count, runtime_writer
AND _agent_meta.schema == debug_run_envelope_v1
```

---

## Witness JSON schema

**Path:** `debug_runs/wss_substrate_live.json`

### Envelope

Uses [`debug_run_envelope_v1`](debug_run_envelope.rs):

```json
{
  "_agent_meta": {
    "schema": "debug_run_envelope_v1",
    "profile": "WSS_SUBSTRATE",
    "source_system": "write_wss_substrate_live_proof_system",
    "relative_path": "debug_runs/wss_substrate_live.json",
    "agent_commands": ["cargo test -p proc_A_dine01 --lib substrate"],
    "related_proofs": [
      "debug_runs/fire_ecology_live.json",
      "debug_runs/stage6_virtualization_live.json"
    ]
  }
}
```

### Payload (required fields)

| JSON pointer | Type | Semantics |
|:---|:---|:---|
| `/gate` | string | Always `"WSS-CHUNK-SLAB-001"` |
| `/pass` | bool | Rollup green (same as `/green` in v1) |
| `/green` | bool | All predicates below |
| `/runtime_writer` | bool | `true` when written by proof system, not hand-edited |
| `/slab_registry_present` | bool | `WorldSubstrateRegistry` exists |
| `/chunk_count` | number | `registry.chunks.len()` |
| `/resident_count` | number | `registry.chunks.resident_count()` |
| `/dirty_count` | number | current dirty set size |
| `/hydrate_wired` | bool | at least one hydrate on chunk spawn observed |
| `/paging_wired` | bool | `sync_substrate_paging` ran ≥1 frame |
| `/hybrid_ecs_weather_authoritative` | bool | **must be `true`** until PR-2 |
| `/hybrid_ecs_fire_authoritative` | bool | **must be `true`** until PR-2 |
| `/dual_write_shim_enabled` | bool | **`false`** in PR-1 |
| `/dual_write_drift_max` | number | **`0.0`** in PR-1 (shim absent) |
| `/substrate_plugin_enabled` | bool | env gate state |
| `/cell_grid_matches_terrain` | bool | `height.len() == CELL_COUNT` for sampled chunk |
| `/chunk_environment_order_preserved` | bool | lib test rollup |

### Optional (PR-2+)

| JSON pointer | Type | When |
|:---|:---|:---|
| `/dual_write_shim_enabled` | bool | PR-2 |
| `/dual_write_drift_max` | number | PR-2 — must be `< 1e-5` for green |
| `/active_runtime_entity_count` | number | PR-3 |

### Green predicate (PR-1)

```text
wss_chunk_slab_001_green :=
  gate == "WSS-CHUNK-SLAB-001"
  AND runtime_writer == true
  AND slab_registry_present == true
  AND chunk_count > 0
  AND hydrate_wired == true
  AND paging_wired == true
  AND hybrid_ecs_weather_authoritative == true
  AND hybrid_ecs_fire_authoritative == true
  AND dual_write_shim_enabled == false
  AND cell_grid_matches_terrain == true
  AND chunk_environment_order_preserved == true
  AND fire_ecology_live.json f1_green unchanged (manual/regression compare)
```

### Example minimal green body

```json
{
  "gate": "WSS-CHUNK-SLAB-001",
  "pass": true,
  "green": true,
  "runtime_writer": true,
  "slab_registry_present": true,
  "chunk_count": 1,
  "resident_count": 1,
  "dirty_count": 1,
  "hydrate_wired": true,
  "paging_wired": true,
  "hybrid_ecs_weather_authoritative": true,
  "hybrid_ecs_fire_authoritative": true,
  "dual_write_shim_enabled": false,
  "dual_write_drift_max": 0.0,
  "substrate_plugin_enabled": true,
  "cell_grid_matches_terrain": true,
  "chunk_environment_order_preserved": true
}
```

---

## ECS schedule (this slice)

```text
Startup
  → SubstratePlugin init WorldSubstrateRegistry

Update
  → (existing) spawn Chunk entities
  → hydrate_chunk_into_substrate     [after chunk spawn sets, WRITER slab]
  → SimControlSystemSet::AdvanceSimTick
  → sync_substrate_paging            [WRITER resident set]
  → ChunkEnvironmentSet::Lod → Weather → Ecology → Fire   [UNCHANGED — ECS authoritative]
  → write_wss_substrate_live_proof_system   [WRITER JSON, throttled]
```

**Do not** insert substrate hydrate **after** weather/fire ticks in PR-1.

---

## Verification commands

```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib substrate
cargo test -p proc_A_dine01 --lib chunk_environment_set
# regression:
cargo test -p proc_A_dine01 --lib fire_ecology
```

After gate PASS + implementation:

```powershell
cargo test -p proc_A_dine01 --lib substrate -- --nocapture
# confirm debug_runs/wss_substrate_live.json refreshed
```

---

## Anti-patterns (forbidden)

| Anti-pattern | Why |
|:---|:---|
| Queue `@coder` before **WSS-DESIGN-GATE-001 PASS** | User/planner policy |
| Hand-edit `wss_substrate_live.json` | witness authority |
| Dual-write weather/fire in PR-1 | premature coupling |
| Remove `ChunkWeather` / `ChunkSurfaceFire` in PR-1 | breaks hybrid matrix |
| Render systems query `WorldSubstrateRegistry` | L2 not ready |
| `FluidDomain` or parallel chunk HashMap | WSS-PLAN-003 rejection |
| Truncate `WorldChunkState` to placeholder structs | user: expand now or lose fields |

---

## Rollback trigger

- `fire_ecology_live.json` `f1_green` → false after substrate plugin enabled
- `chunk_environment_set` ordering test fails
- Stage 5 FULL_APP failure traced to `SubstratePlugin` → disable via `RUST_ENGINE_SUBSTRATE=0`, file steward ticket

---

## Coder assignment (after gate only)

| Field | Value |
|:---|:---|
| **Slice** | WSS-CHUNK-SLAB-001 |
| **Owner** | `@coder` A (suggested — render/sim boundary) |
| **Budget** | ≤8 files listed in § Module layout + 2 engine wires |
| **Playbook** | `stage5_readiness_agent` (regression only) + WSS-PLAN-002 |
| **Unblocks** | WSS-SLAB-PR-2, WSS-HYDRO-RUNTIME-001 hydrate path, WSS-ATMOS-CLIPMAP-001 types |

**Until gate PASS:** row status in `coder_active_queue.json` = **`blocked_design_gate`**.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Exec plan — blocked on WSS-DESIGN-GATE-001 |
