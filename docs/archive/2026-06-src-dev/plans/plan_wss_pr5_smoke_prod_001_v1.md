# PLAN-WSS-PR5-SMOKE-PROD-001 — Smoke authority production cutover `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-PR5-SMOKE-PROD-001** |
| **Slice ID** | **WSS-PR5-SMOKE-PROD-001** |
| **Parent** | [`plan_wss_hybrid_retire_pr4_001_v1.md`](plan_wss_hybrid_retire_pr4_001_v1.md) § PR-5 `ChunkSmokeField` |
| **Prereq** | PR-4 **CLOSED**; PR-5 fixture **QUALIFIED CLOSED** (`ecs_retire_fixture_green`, weather/fire authority false) |
| **Smoke bridge** | [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md) |
| **UX** | [`wss_pr4_retire_cutover_ux_v1.md`](wss_pr4_retire_cutover_ux_v1.md) (**DESIGN-PR4-RETIRE-UX-001** PASS qualified) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). Fixture cutover (`ecs_retire.rs`) intentionally **excludes** smoke — this exec plan is **live Simulation** authority flip only.

---

## Summary

| Lane | `hybrid_ecs_smoke_authoritative` | Scope |
|:---|:---:|:---|
| PR-5 fixture (landed) | may stay `true` in lib fixture | weather + fire cutover only |
| **PR5 smoke prod (this plan)** | must be **`false`** in **live** `wss_substrate_live.json` | `ChunkSmokeField` → slab/clipmap extract path |

**Disk baseline (2026-05-27):** weather/fire `false`; smoke `true`; `ecs_retire_fixture_green: true` — **smoke prod OPEN**.

---

## Entry gates

| Gate | Witness |
|:---|:---|
| PR-4 | `substrate_persist_roundtrip_ok`, `dynamic_overlay_migrated` |
| PR-5 fixture | `ecs_retire_fixture_green`, `ecs_retire_stable_ticks >= 120` |
| Smoke bridge | `smoke_extract_wired`, `smoke_stub_removed` |
| Atmos clipmap | `wss_atmos_clipmap_001.green` |
| Dual-write | `dual_write_shim_enabled`, drift `< 1e-5` |

---

## Authority map

| Resource | Single writer | After cutover |
|:---|:---|:---|
| `ChunkSmokeField` | `chunk_smoke_field_tick` (today) | **read-only mirror** or despawn in hot path |
| Slab / clipmap smoke sample | atmosphere advect + `AtmosphereClipmapStack` | **authoritative** for extract |
| `SimChunkSmokeVisualExtract` | `publish_sim_visual_extract` | populated from slab/clipmap, **not** `ChunkSmokeField` query |
| `hybrid_ecs_smoke_authoritative` | `EcsRetireState` / live proof | `false` in Simulation writer |
| `wss_substrate_live.json` | `write_wss_substrate_live_proof_system` | smoke prod fields |

---

## PR plan (≤3 files each)

### PR5-SM-1 — Slab smoke sample + dual-write

| File | Change |
|:---|:---|
| `src/substrate/types.rs` | ensure `AtmosphereState` / clipmap ref exposes smoke density for extract (if missing) |
| `src/substrate/shim.rs` | dual-write `ChunkSmokeField` ↔ slab atmosphere smoke scalar |
| `src/systems/fire/chunk_smoke_field.rs` | when `!hybrid_ecs_smoke_authoritative`, tick writes slab only |

**Drift:** extend dual-write compare to include smoke scalar; stable **N=120** ticks before cutover (reuse `ECS_RETIRE_DRIFT_WINDOW_TICKS` or dedicated counter).

### PR5-SM-2 — Extract path reads slab

| File | Change |
|:---|:---|
| `src/systems/atmosphere/visual_extract.rs` | `publish_sim_visual_extract` reads clipmap/slab when smoke authority false |
| `src/render/extraction/smoke_visual_extract.rs` | assert no `ChunkSmokeField` query when cutover complete |
| `src/substrate/ecs_retire.rs` | extend cutover: `hybrid_smoke_authoritative`, `smoke_extract_reads_slab` |

**Regression:** `gpu_bridge_reads_extract_not_ecs` test must stay green.

### PR5-SM-3 — Live witness + lib tests

| File | Change |
|:---|:---|
| `src/substrate/mod.rs` | `build_wss_substrate_payload`: `ecs_retire_smoke_extract_slab` |
| `src/substrate/mod.rs` | lib test: live-style retire pass flips `hybrid_ecs_smoke_authoritative` false |
| `src/substrate/ecs_retire.rs` | include smoke in `ecs_retire_fixture_green` **only** when smoke prod flag set (split fixture vs prod witnesses) |

**Important:** Keep **fixture** test green without requiring smoke cutover; add **`ecs_retire_smoke_prod_green`** rollup for live JSON.

---

## Witness schema

**File:** `debug_runs/wss_substrate_live.json`

| Pointer | Type | Production exit |
|:---|:---|:---|
| `/hybrid_ecs_smoke_authoritative` | bool | **`false`** |
| `/ecs_retire_smoke_extract_slab` | bool | **`true`** |
| `/ecs_retire_smoke_prod_green` | bool | **`true`** (rollup) |
| `/smoke_extract_wired` | bool | `true` (preserved) |
| `/smoke_density_sum` | number | `> 0` in tactical fixture |
| `/ecs_retire_fixture_green` | bool | `true` (no regression) |
| `/green` | bool | `true` |

**PR5 smoke prod green rollup:**

```text
hybrid_ecs_smoke_authoritative == false
AND ecs_retire_smoke_extract_slab == true
AND smoke_extract_wired == true
AND smoke_density_sum > 0
AND ecs_retire_fixture_green == true
AND green == true
```

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate ecs_retire smoke_visual_extract atmosphere
cargo test -p proc_A_dine01 --lib wss_substrate
```

Refresh live JSON via Simulation proof writer (not lib-only fixture).

---

## Anti-patterns

- Flipping `hybrid_ecs_smoke_authoritative` in lib fixture only while live JSON stays `true`
- Removing `ChunkSmokeField` component before extract reads slab
- Render/extract querying ECS smoke after cutover
- Reopening PR-4 exec or smoke bridge exec docs
- Breaking `tactical_vfx_witness.all_green`

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `WSS-PR5-SMOKE-PROD-001` |
| **Witness** | `debug_runs/wss_substrate_live.json` |
| **Mutex** | `src/substrate/*` — no `src/construction/*` |
| **Designer** | **DESIGN-PR4-RETIRE-UX-001** — signed; smoke-pending rows in retire UX doc |
| **Acceptance** | PR5 smoke prod green rollup above |
