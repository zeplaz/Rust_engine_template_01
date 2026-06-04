# PLAN-WSS-SLAB-PR-2-EXEC-001 — Dual-write shim (weather + fire) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-SLAB-PR-2-EXEC-001** |
| **Slice ID** | **WSS-SLAB-PR-2** |
| **Prior** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) (**WSS-CHUNK-SLAB-001**) |
| **Parent** | [`wssr_plan_002_chunk_authority_v1.md`](wssr_plan_002_chunk_authority_v1.md) (reference only) |
| **Version** | `1.0.0` (**SIGNED**) |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — slab checkpoint satisfied (`wss_chunk_slab_001.green`) |

**Sign trigger:** `debug_runs/wss_substrate_live.json` → `green: true` AND `hydrate_wired: true`

**Coder entry:** allowed — plan is signed and slab checkpoint is green.

---

## Summary

Bidirectional sync between **legacy ECS components** (`ChunkWeather`, `ChunkSurfaceFire`) and **`WorldSubstrateRegistry` slab** for resident chunks — with drift metrics in witness. **ECS remains gameplay authority** until PR-5 retirement; slab is mirror + persist target.

---

## Authority during PR-2

| Direction | When | Writer |
|:---|:---|:---|
| ECS → slab | After `weather_chunk_tick` / `chunk_surface_fire_tick` | `sync_ecs_to_substrate_shim` |
| slab → ECS | **Forbidden** in PR-2 except test fixtures | — |
| Drift metric | End of frame | `compare_weather_fire_shim` |

---

## PR plan (≤3 files each)

### PR2-1 — Weather dual-write

| File | Change |
|:---|:---|
| `src/substrate/shim.rs` | **new** — `sync_chunk_weather_to_slab` |
| `src/systems/weather/chunk_weather.rs` | hook after tick (or observer) |
| `src/substrate/live_proof.rs` | `dual_write_drift_max` for rain/fog fields |

**Predicate:** `|ecs.rain - slab.local.rain| < 1e-5` for resident keys

---

### PR2-2 — Fire / thermal dual-write

| File | Change |
|:---|:---|
| `src/substrate/shim.rs` | `sync_surface_fire_to_thermal` |
| `src/systems/fire/chunk_surface_fire.rs` | post-tick hook |
| `src/substrate/types.rs` | map heat → `thermal.surface_heat` |

---

### PR2-3 — Witness + flag

| File | Change |
|:---|:---|
| `src/substrate/live_proof.rs` | `dual_write_shim_enabled: true` |
| `debug_runs/wss_substrate_live.json` | schema fields from exec 001 |

**Green:**

```text
dual_write_shim_enabled == true
AND dual_write_drift_max < 1e-5
AND hybrid_ecs_weather_authoritative == true
AND hybrid_ecs_fire_authoritative == true
```

---

## Hybrid matrix (unchanged authority)

| Component | Authoritative | Slab field |
|:---|:---:|:---|
| `ChunkWeather` | **YES** | `atmosphere.local` |
| `ChunkSurfaceFire` | **YES** | `thermal` |
| Extract / render | reads **ECS** until PR-5 | — |

---

## Designer

**DESIGN-DUAL-WRITE-UX-001** when `dual_write_drift > ε` in dev — diagnostics copy per [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md) § checkpoints.

---

## Sign-off

| Role | Status | Date | Evidence |
|:---|:---|:---|:---|
| `@planner` | **PASS** | 2026-05-27 | `debug_runs/wss_substrate_live.json` → `green: true`, `hydrate_wired: true` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | SIGNED after slab gate green checkpoint |
| v0.9.0 | 2026-05-26 | Draft pre `wss_chunk_slab_001.green` |
