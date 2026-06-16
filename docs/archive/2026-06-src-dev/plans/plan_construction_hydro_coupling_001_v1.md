# PLAN-CONSTRUCTION-HYDRO-COUPLING-001 — Construction → hydrology event bus `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-HYDRO-COUPLING-001** |
| **Coder lane** | **WSS-HYDRO-BOUNDARY-001** / matrix **B-H2** |
| **Parent** | [`wssr_plan_003_hydrology_runtime_v1.md`](wssr_plan_003_hydrology_runtime_v1.md) (**WSS-PLAN-003**, SIGNED) |
| **Exec sibling** | [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md) (**HY-005** event drain) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). Docs-only coupling contract — no hydrology sim in construction crate.

---

## Summary

Construction **never** writes `WorldChunkState.hydrology` (or `water_depth`, `flow_velocity`, masks). After an **execute** commit, construction publishes **`HydrologyDirtyReason`** events on a single bus; hydrology runtime (**Tier 2 deep solve**) is the sole consumer.

Ghost preview, validation, and queue rows remain hydrology-neutral per [`construction_invariants.md`](construction_invariants.md).

---

## Authority map

| Resource | Single writer | Allowed | Forbidden |
|:---|:---|:---|:---|
| `HydrologyDirtyReason` enum | `src/substrate/hydrology/` (types) | extend variants only via WSS-PLAN-003 amendment | construction-local duplicate enum |
| `HydrologyEventQueue` | `src/systems/hydrology/event_queue.rs` | enqueue, drain, dedupe per tick | construction direct push to slab |
| Construction bridge | `src/construction/hydro_coupling.rs` (**new**) | `emit_construction_hydro_dirty(...)` after execute | any `get_mut` on `WorldSubstrateRegistry.chunks[].hydrology` |
| Deep solve scheduler | `HydrologySimulationTask` / HY-005 | react to drained events | construction calling deep solve inline |
| Witness `wss_hydro_runtime_001` | `src/substrate/live_proof.rs` | `construction_hydro_coupling_wired` flag | manual JSON |

---

## Event bus contract

### Message shape

```rust
/// Published by construction; consumed by hydrology drain (HY-005).
pub struct HydrologyDirtyEvent {
    pub key: ChunkKey,
    pub reason: HydrologyDirtyReason,
    pub structure_id: Option<u64>,
    pub affected_cells: Option<SmallVec<[u32; 8]>>,
}
```

Use existing **`HydrologyDirtyReason`** from WSS-PLAN-003:

| Variant | Construction emit when |
|:---|:---|
| `ConstructionComplete { structure_id }` | Building/site/road **execute** committed (footprint tiles finalized) |
| `DamBreach { structure_id }` | Demolish execute removes dam/levee structure (future) |
| `None` | Never emitted by construction |

**Do not emit** from preview, ghost, plan queue insert, or validation failure paths.

### Bus API (construction-facing)

```rust
pub fn emit_construction_hydro_dirty(
    queue: &mut HydrologyEventQueue,
    key: ChunkKey,
    reason: HydrologyDirtyReason,
    structure_id: u64,
    affected_cells: impl IntoIterator<Item = u32>,
);
```

- **Dedup:** same `(key, reason, structure_id)` coalesced within one sim tick (hydrology drain owns dedupe table).
- **Ordering:** events processed FIFO per tick after `execute_construction_plans_system` / site commit systems.

---

## Emit hooks (execute funnel only)

| Construction path | Emit | `structure_id` source |
|:---|:---:|:---|
| `execute_construction_plans_system` (road/corridor completed) | `ConstructionComplete` | plan `id` or transport edge id |
| `commit_construction_site_system` / `queue_commit_construction_site` | `ConstructionComplete` | site entity / book row id |
| Parametric `Enter` commit (buildings) | `ConstructionComplete` | staged ghost `structure_id` |
| Demolish execute (dam class, when typed) | `DamBreach` | pending demolish target id |
| Ghost / preview / `allows_commit == false` | — | **no emit** |

**Chunk key resolution:** map footprint world cells → `ChunkKey` via existing chunk matrix / strategic tile book (same helper hydrology hydrate uses).

---

## Coder task list (≤3 files per PR)

### HC-1 — Types + queue resource

| File | Change |
|:---|:---|
| `src/substrate/hydrology/state.rs` | `HydrologyDirtyReason`, `HydrologyEventQueue` |
| `src/systems/hydrology/event_queue.rs` | `HydrologyEventQueueDrain` system |
| `src/substrate/mod.rs` | plugin registration |

### HC-2 — Construction bridge

| File | Change |
|:---|:---|
| `src/construction/hydro_coupling.rs` | **new** — `emit_construction_hydro_dirty` |
| `src/construction/construction_pipeline.rs` | hook after successful road execute |
| `src/construction/build_commit.rs` | hook after site commit |

### HC-3 — Witness + lib test

| File | Change |
|:---|:---|
| `src/substrate/live_proof.rs` | `construction_hydro_coupling_wired` under `wss_hydro_runtime_001` |
| `src/construction/integration_tests.rs` | execute plan → event queued; preview → no event |

---

## Witness schema

**File:** `debug_runs/wss_substrate_live.json`  
**Block:** `wss_hydro_runtime_001` (nested)

| Pointer | Type | Meaning |
|:---|:---|:---|
| `/wss_hydro_runtime_001/construction_hydro_coupling_wired` | bool | bridge registered + test emit observed |
| `/wss_hydro_runtime_001/deep_solve_wired` | bool | HY-005 (unchanged) |
| `/wss_hydro_runtime_001/green` | bool | rollup includes coupling when product gate requires B-H2 |

**Coupling green predicate:**

```text
construction_hydro_coupling_wired :=
  execute_path_emits_construction_complete == true
  AND preview_path_emits_zero == true
  AND hydrology_drain_observed_construction_event == true
```

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib hydrology_hydro_coupling construction::integration
cargo test -p proc_A_dine01 --lib wss_substrate
```

---

## Anti-patterns

- Preview or ghost writing `water_depth` / hydrology masks
- Second event bus in `src/construction/`
- Inline deep solve from construction execute (must be async per hydrology scheduler)
- Reopening **WSS-PLAN-003** enum without planner amendment
- Reopening archived R4/M3/replay exec plans

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `WSS-HYDRO-BOUNDARY-001` (**B-H2**) |
| **Depends on** | `WSS-HYDRO-RUNTIME-001` HY-005 stub drain (may ship coupling before deep solve green) |
| **Acceptance** | `construction_hydro_coupling_wired: true` in `wss_substrate_live.json`; construction lib test: execute emits, preview does not |
