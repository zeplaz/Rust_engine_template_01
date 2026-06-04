# PLAN-WSS-HYBRID-RETIRE-PR4-001 — Hybrid ECS retirement (PR-4 / PR-5) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-HYBRID-RETIRE-PR4-001** |
| **Slice** | **WSS-SLAB-PR-4** (persist) + **WSS-SLAB-PR-5** (ECS retire) |
| **Parent** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) § PR-4/PR-5 |
| **Prereq** | PR-3 **CLOSED** on disk (`active_runtime_wired`); PR-2 dual-write stable |
| **UX** | [`wss_dual_write_transition_ux_001.md`](wss_dual_write_transition_ux_001.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). **Do not** start PR-5 component removal until PR-4 persist witness green.

---

## Summary

After hybrid activation (PR-3) and stable dual-write (PR-2), migrate **persist book** and **dynamic overlay** into slab (PR-4), then retire legacy ECS weather/fire components when drift stays zero for **N** CI frames (PR-5).

---

## Entry gates (all required)

| Gate | Witness |
|:---|:---|
| Slab green | `wss_substrate_live.json` → `green`, `hydrate_wired` |
| Dual-write | `dual_write_shim_enabled: true`, `dual_write_drift_max < 1e-5` |
| Active runtime | `active_runtime_wired`, `active_runtime_activate_test_ok` |
| Atmos + hydro | `wss_atmos_clipmap_001.green`, `wss_hydro_runtime_001.green` |

---

## PR-4 — Persist book + dynamic overlay (≤3 files per PR)

| Task | Deliverable |
|:---|:---|
| PR4-1 | `SubstratePersistBook` flush dirty slab keys on save slot |
| PR4-2 | Migrate `DynamicTerrainOverlay` HashMaps → `WorldChunkState.dynamic` |
| PR4-3 | Witness: `substrate_persist_roundtrip_ok`, `dynamic_overlay_migrated` |

**Authority:** `WorldSubstrateRegistry.persist` sole writer for slab persist slices.

---

## PR-5 — ECS component retirement (≤3 files per PR)

| Component | Retire when |
|:---|:---|
| `ChunkWeather` | drift 0 for **N=120** sim ticks in CI fixture **and** extract reads slab snapshot path |
| `ChunkSurfaceFire` | same + fire extract uses thermal slab |
| `ChunkSmokeField` | after atmosphere fold complete (smoke bridge closed) |

**Witness rollup:**

```text
hybrid_ecs_weather_authoritative == false  # only after cutover
hybrid_ecs_fire_authoritative == false
ecs_retire_fixture_green == true
wss_chunk_slab_001.green == true  # no regression
```

---

## Anti-patterns

- Retire ECS before persist round-trip green
- Render writing slab during PR-4/5
- Reopening PR-2/PR-3 exec plan docs (regression only)
- Removing `ActiveChunkRuntime` before hot-region policy satisfied

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `WSS-SLAB-PR-4`, `WSS-SLAB-PR-5` |
| **Witness** | `debug_runs/wss_substrate_live.json` |
| **Acceptance** | PR-4: `substrate_persist_roundtrip_ok`; PR-5: `ecs_retire_fixture_green` with authorities flipped per plan |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate
```
