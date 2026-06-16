# PLAN-WSS-SLAB-PR-3-EXEC-001 — ActiveChunkRuntime hybrid entities `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-SLAB-PR-3-EXEC-001** |
| **Slice ID** | **WSS-SLAB-PR-3** |
| **Prior** | [`plan_wss_slab_pr2_dual_write_v1.md`](plan_wss_slab_pr2_dual_write_v1.md) (**SIGNED v1.0.0**) |
| **Parent** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) § W2-C |
| **Paired prep** | **PLAN-WSS-ACTIVE-CHUNK-001** (hot-region criteria — separate doc) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). **Prerequisite met:** PR-2 signed + `wss_chunk_slab_001.green` + `dual_write_shim_enabled`.

**Coder entry:** allowed after PR-2 drift witness stable in CI fixture.

---

## Summary

Introduce optional **`ActiveChunkRuntime`** ECS entities for **hot regions only**. Entities **mirror** slab state for query-friendly systems (fire front, flood solve, construction footprint, combat); **slab remains authoritative** for persist and cross-chunk exchange. On deactivate, flush ECS deltas → slab before despawn.

---

## Authority map

| Resource | Single writer | Allowed | Must NOT |
|:---|:---|:---|:---|
| `ActiveChunkRuntime` spawn/despawn | `src/substrate/active_runtime.rs` (**new**) | spawn on activation reasons; despawn after flush | render/minimap; construction preview |
| `ChunkActivationReason` | substrate types | set at spawn | duplicate in fire/construction crates |
| Slab `WorldChunkState` | `WorldSubstrateRegistry` domain writers | receive flush on deactivate | overwritten by ECS without flush |
| Activation policy | `activate_hot_chunks_system` | read fire front, hydro deep-solve flags, construction book | per-view camera residency |
| Witness `wss_substrate_live.json` | `write_wss_substrate_live_proof_system` | `active_runtime_*` counters | hand-edited JSON |

---

## Component contract (from WSS-PLAN-002)

```rust
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

**Rule:** At most **one** `ActiveChunkRuntime` per `ChunkKey`. Multiple reasons → OR into `activation_reason` priority: `FloodSolve` > `FireFront` > `Construction` > `PlayerProximity` > `Combat`.

---

## PR plan (≤3 files each)

### PR3-1 — Types + plugin wiring

| File | Change |
|:---|:---|
| `src/substrate/types.rs` | `ActiveChunkRuntime`, `ChunkActivationReason` |
| `src/substrate/active_runtime.rs` | **new** — spawn/despawn helpers |
| `src/substrate/mod.rs` | register systems in `SubstratePlugin` |

### PR3-2 — Activate / deactivate systems

| File | Change |
|:---|:---|
| `src/substrate/active_runtime.rs` | `activate_hot_chunks_system`, `deactivate_stale_runtime_system` |
| `src/systems/fire/chunk_surface_fire.rs` | mark keys with active fire front (read-only signal) |
| `src/substrate/registry.rs` | flush hook `flush_active_runtime_to_slab` |

**Activate / deactivate:** implement criteria from [`plan_wss_active_chunk_001_v1.md`](plan_wss_active_chunk_001_v1.md) (caps, priority, resident-only gates).

### PR3-3 — Witness + lib tests

| File | Change |
|:---|:---|
| `src/substrate/live_proof.rs` | `active_runtime_entity_count`, `active_runtime_wired` |
| `src/substrate/mod.rs` | lib test: activate 3 keys → deactivate → slab drift 0 |

---

## Witness schema

**File:** `debug_runs/wss_substrate_live.json`

| Pointer | Type | Meaning |
|:---|:---|:---|
| `/active_runtime_wired` | bool | systems registered |
| `/active_runtime_entity_count` | number | ECS count (may be 0 in headless proof) |
| `/active_runtime_activate_test_ok` | bool | lib fixture activate/deactivate/flush |
| `/wss_chunk_slab_001/green` | bool | must remain true (no PR-3 regression) |

**PR-3 green rollup:**

```text
active_runtime_wired == true
AND active_runtime_activate_test_ok == true
AND wss_chunk_slab_001.green == true
AND dual_write_shim_enabled == true  # PR-2 preserved
```

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate active_runtime
cargo test -p proc_A_dine01 --lib wss_substrate
```

---

## Anti-patterns

- ECS entity per chunk planet-wide (hot only)
- Skipping slab flush on despawn
- Render/extract spawning `ActiveChunkRuntime`
- Reopening PR-2 dual-write authority matrix
- Using per-view camera to drive activation (sim focus / paging only)

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `WSS-SLAB-PR-3`, **PLAN-WSS-ACTIVE-CHUNK-001** criteria wiring |
| **Witness** | `debug_runs/wss_substrate_live.json` |
| **Acceptance** | `active_runtime_wired` + `active_runtime_activate_test_ok`; PR-2 witness fields unchanged |
