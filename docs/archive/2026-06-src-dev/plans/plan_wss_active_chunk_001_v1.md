# PLAN-WSS-ACTIVE-CHUNK-001 — Hot-region activation criteria `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-ACTIVE-CHUNK-001** |
| **Paired exec** | [`plan_wss_slab_pr3_exec_001_v1.md`](plan_wss_slab_pr3_exec_001_v1.md) (**WSS-SLAB-PR-3**) |
| **Parent** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) § hybrid ECS |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). Policy doc only — implementation lives in PR-3 `activate_hot_chunks_system`.

---

## Summary

Define **when** a resident chunk key may spawn `ActiveChunkRuntime` and **when** it must despawn. Criteria are **sim-focus / substrate-paging** driven — never per-view camera or minimap zoom.

Slab remains authoritative; ECS entities are optional mirrors for hot systems only.

---

## Global gates (all reasons)

| Gate | Rule |
|:---|:---|
| **Residency** | `WorldSubstrateRegistry.paging.is_resident(key) == true` |
| **Cap** | `active_runtime_entity_count <= ACTIVE_CHUNK_CAP` (default **64**) per frame |
| **Dedup** | At most one `ActiveChunkRuntime` per `ChunkKey` |
| **Paging** | Activation reads **sim focus** + resident set only — not `ViewManager` / per-view residency |
| **Flush** | On despawn, `flush_active_runtime_to_slab(key)` before entity removal |

---

## Activation criteria by reason

Priority when multiple signals fire (single entity, highest wins):

`FloodSolve` > `FireFront` > `HydrologyEvent` > `Construction` > `PlayerProximity` > `Combat`

### FireFront

| Signal | Threshold |
|:---|:---|
| `ChunkSurfaceFire` on chunk entity for `key` | `surface_heat > FIRE_FRONT_HEAT_EPS` (default `0.05`) **or** spreading flag true |
| Slab mirror | `WorldChunkState.thermal.surface_heat` matches ECS within PR-2 drift ε |

**Deactivate:** heat below ε for `deactivate_after_ticks` (default **30** sim ticks).

### FloodSolve

| Signal | Threshold |
|:---|:---|
| `HydrologySolveMeta.deep_solve_active == true` on resident slab row | immediate activate |
| `HydrologyDirtyReason` deep event queued for `key` | activate until solve completes |

**Deactivate:** `deep_solve_active == false` and queue drained for `key`.

### HydrologyEvent

| Signal | Threshold |
|:---|:---|
| Drained `HydrologyDirtyReason` targets `key` (dam breach, upstream overflow) | activate for deep-solve window |

**Deactivate:** same as FloodSolve when solve completes.

### Construction

| Signal | Threshold |
|:---|:---|
| `CorridorConstructionBook` row on edge touching `key` | phase `InProgress` or `Planned` with sim tick writer armed |
| Site book / footprint | any `InProgress` site tile maps to `key` |
| Parametric commit | footprint tiles on `key` within **2 sim ticks** of execute (then drop unless still InProgress) |

**Deactivate:** no in-progress rows touching `key` for **30** ticks.

**Forbidden:** activate on ghost preview or validation-only plans.

### PlayerProximity

| Signal | Threshold |
|:---|:---|
| Sim focus chunk `key` | distance ≤ **1** chunk Manhattan from focus |
| Staging / play spawn | same window as sim focus authority |

**Deactivate:** focus moves > **2** chunks away for **60** ticks.

### Combat

| Signal | Threshold |
|:---|:---|
| Reserved | **disabled in v1** — enum present, no auto-spawn until combat book wired |

---

## Deactivate policy

```text
deactivate_candidate(key) :=
  NOT any_activation_signal(key)
  OR (ticks_since_last_signal(key) >= deactivate_after_ticks[reason])
```

| Reason | Default `deactivate_after_ticks` |
|:---|:---:|
| FloodSolve / HydrologyEvent | 0 (immediate when solve done) |
| FireFront | 30 |
| Construction | 30 |
| PlayerProximity | 60 |
| Combat | N/A (v1 off) |

---

## Budget / perf guards

| Guard | Limit |
|:---|:---|
| Max spawns per frame | **8** |
| Max despawns per frame | **16** (flush cost) |
| Headless proof | may use **3** fixed keys in lib fixture |

---

## Coder wiring (PR-3 consumption)

| PR-3 task | Uses this doc |
|:---|:---|
| PR3-2 `activate_hot_chunks_system` | § Activation criteria + global gates |
| PR3-2 `deactivate_stale_runtime_system` | § Deactivate policy |
| PR3-3 witness | `active_runtime_policy_wired: true` when criteria match table |

---

## Witness schema

**File:** `debug_runs/wss_substrate_live.json`

| Pointer | Type | Meaning |
|:---|:---|:---|
| `/active_runtime_policy_wired` | bool | criteria module registered |
| `/active_runtime_cap_respected` | bool | count ≤ cap in proof harness |
| `/active_runtime_entity_count` | number | from PR-3 |

**Policy green (nested under PR-3 rollup):**

```text
active_runtime_policy_wired == true
AND active_runtime_cap_respected == true
```

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate active_runtime
```

Fixture must prove: resident-only spawn; non-resident key never spawns; cap respected.

---

## Anti-patterns

- Per-view camera / minimap driving activation
- Planet-wide `ActiveChunkRuntime` per chunk
- Preview ghosts triggering Construction reason
- Reopening PR-3 authority map (this doc is criteria only)

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `WSS-SLAB-PR-3` PR3-2 policy wiring |
| **Depends on** | PR-3 types (`ChunkActivationReason`) |
| **Acceptance** | `active_runtime_policy_wired` + cap respected; PR-3 activate test still green |
