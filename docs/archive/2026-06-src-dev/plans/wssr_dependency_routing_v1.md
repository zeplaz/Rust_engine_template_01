# WSS dependency routing `v1` (post slab types)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-ROUTING-001** |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **ACTIVE** |

**Do not re-open:** WSS-PLAN-001..004 sign-off text.

---

## Spine tree (authoritative)

```text
WSS-CHUNK-SLAB-001  (types + registry + witness)
  │
  ├─► WSS-ATMOS-CLIPMAP-001   needs: ChunkKey, WorldChunkState, WorldSubstrateRegistry
  │                             does NOT need: hydrate_wired on sim spawn (for AC-001..003)
  │
  └─► WSS-HYDRO-RUNTIME-001    needs: hydrate_wired + HydrologyState populated from gen
                                blocked until: CS-003 / HY-001 on Chunk entity spawn
```

---

## Gate matrix

| Slice | Coder | Start when | Hard block |
|:---|:---|:---|:---|
| **WSS-CHUNK-SLAB-001** | A (done partial) | types in `src/substrate/` | — |
| **WSS-ATMOS-CLIPMAP-001** | A | `WorldChunkState` + `ContaminationState` on disk | sim `hydrate_wired` |
| **WSS-HYDRO-RUNTIME-001** | B | `hydrate_wired: true` in **sim** + `HydrologyResult` copy | types only |

---

## WSS-CHUNK-SLAB-001 — what “ready now” means

| Layer | Status | Evidence |
|:---|:---:|:---|
| **Types + registry** | ☑ READY | `src/substrate/{mod,slab,types,registry}.rs` |
| **Lib witness green** | ☑ | `cargo test -p proc_A_dine01 --lib wss_substrate_refresh_green` |
| **Sim spawn hydrate** | ◐ PENDING | `hydrate_skeleton_chunk` only — **not** wired to `Chunk` spawn yet |
| **Witness on disk (sim)** | ◐ | Refresh via lib test; live sim may show `chunk_count: 0` until CS-003 |

**Exec:** [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) — remaining: **CS-003** `hydrate_chunk_from_matrix` on spawn.

---

## WSS-ATMOS-CLIPMAP-001 — unblocked (parallel)

**Plan:** [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md)

| PR block | May start now | Touches |
|:---|:---:|:---|
| AC-001 contamination on slab | ☑ | `src/substrate/types.rs` (fields exist) + new `src/substrate/atmos_clipmap.rs` or `systems/atmosphere/clipmap.rs` |
| AC-002 `AtmosphereClipmapStack` L0–L3 | ☑ | new resources — **no** Chunk spawn hook |
| AC-003 legacy `AtmosphereField` → L1 alias | ☑ | `systems/atmosphere/field.rs` bridge |

**Mutex:** coordinate with **FIRE-F2-EXTRACT-001** on `fire_visual_extract.rs` / `render_projection_graph.rs` — one coder per PR.

**Witness extension:** `wss_atmos_clipmap_001.green` in `wss_substrate_live.json` (per atmos exec plan).

---

## WSS-HYDRO-RUNTIME-001 — blocked on hydrate

**Plan:** [`plan_wss_hydro_runtime_exec_001_v1.md`](plan_wss_hydro_runtime_exec_001_v1.md)

| PR block | Start when |
|:---|:---|
| HY-001 gen → `WorldChunkState.hydrology` | `witness.hydrate_wired` set from **real** spawn hook, not skeleton only |
| HY-002 background tick | resident keys non-empty in sim |
| HY-003 event solve | HY-001 green |

**Coder B:** may prep types/tests in `src/substrate/hydrate.rs` **after** A lands CS-003 or share PR with slab completion.

**Green predicate:**

```text
hydrate_wired == true
AND chunk_count > 0
AND hydrology_hydrated == true   # new witness field HY-001
AND ocean_mask_sum >= 0          # fixture-dependent
```

---

## Recommended session split

| Session | Coder A | Coder B |
|:---|:---|:---|
| **Now** | Finish **CS-003** spawn hydrate **or** **WSS-ATMOS** AC-001 | Wait **or** **B-C4** construction (disjoint) |
| **Next** | AC-002 clipmap stack | **HY-001** once `hydrate_wired` sim-green |
| **Parallel OK** | Atmos types + F2 extract (different files) | Hydro after hydrate |

---

## Regression (all WSS slices)

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate
cargo test -p proc_A_dine01 --lib chunk_environment_set fire_ecology
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Post slab types landed; routing tree |
