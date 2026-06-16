# WEATHER-SIM-PLAN-001 — Weather simulation runbook v2 `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **WEATHER-SIM-PLAN-001** |
| **Prior** | [`prompts/guides/weather_simulation_runbook_v1.md`](../prompts/guides/weather_simulation_runbook_v1.md) (draft scaffold) |
| **WSS coupling** | [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) (reference — **do not re-sign**) |
| **Exec dependency** | [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) |
| **Version** | `1.0.0` (**SIGNED**) |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — checkpoint satisfied (`wss_atmos_clipmap_001.green`) · **implementation PARALLEL DOWNTIME** — [`plan_weather_parallel_lane_v1.md`](plan_weather_deferred_v1.md) |

**No Rust.** Signed at checkpoint: `wss_atmos_clipmap_001.green`.

---

## Summary

Elevate weather from **cosmetic** `ChunkWeather` + mesh precip to a **three-tier simulation** (climate → regional clipmap → chunk local) that feeds hydrology, fire, logistics, and power — while keeping **presentation** on separate render clipmaps and retiring CPU mesh precip as authority.

---

## North star (from runbook v1, expanded)

```text
Climate (L3, slow, saveable seed)
  → Regional weather cells (L2/L1 clipmap)
  → ChunkWeatherLocal (L0, fast)
  → Derived overlays (traction, visibility, mud)
  → Hydrology evaporation / saturation
  → Infrastructure + logistics + economy samples
  → Agent behavior hooks (stubs v2)
```

**Not:** Hanabi rain at world scale · `WeatherVisualPlugin` as sim truth · fixed 128² global grid.

---

## Authority map (v2)

| Tier | Resource | Writer | Tick rate |
|:---|:---|:---|:---|
| L3 Climate | `ClimateState` resource | `climate_slow_tick` | every N thousand sim ticks |
| L2 Regional | `AtmosphereClipmapStack` L1–L3 | `regional_weather_tick` | medium |
| L0 Chunk | `ChunkWeather` component (hybrid until PR-2) | `weather_chunk_tick` | every sim tick |
| L0 Slab mirror | `WorldChunkState.atmosphere.local` | dual-write after PR-2 | same |
| L2 Extract | `ClimateVisualAggregate` | atmosphere visual extract | per frame |
| L3 Visual | `WeatherVisualPlugin`, GPU field | render only | per frame |

---

## Effect routing (must map to systems)

| Category | Consumer | v2 deliverable |
|:---|:---|:---|
| Hydrology | `HydrologyBackgroundTick` | rain → saturation; evap ← heat/wind |
| Fire | `derive_fire_fuel_from_vegetation` | moisture + wind |
| Logistics | throughput solver sample | traction overlay stub |
| Power | `GlobalRenewableWeatherFactors` | **exists** — wire to clipmap L2 |
| Combat | visibility | `AtmosphereCell.visibility` sample |
| Construction | none direct | mud via `DynamicOverlaySlice` |
| Agriculture | ecology stress | future |

---

## Implementation waves (planner — post-sign)

| Wave | ID | Goal | Coder slice |
|:---|:---|:---|:---|
| W-SIM-1 | **WEATHER-CLIMATE-001** | `ClimateState` + slow tick | after clipmap types |
| W-SIM-2 | **WEATHER-REGIONAL-001** | L2 cells interpolate storms/fronts | with atmos exec W4-B |
| W-SIM-3 | **WEATHER-CHUNK-001** | Chunk local ← regional sample | dual-write PR-2 |
| W-SIM-4 | **WEATHER-EFFECTS-001** | logistics traction + visibility stubs | optional |
| W-SIM-VFX-1 | **WEATHER-GPU-PRECIP-001** | demote mesh precip to fallback flag | after render clipmap |

---

## Module layout (target — do not create until SIGNED)

```text
src/systems/weather/
  climate.rs           # L3
  regional_sample.rs   # L2 clipmap interface
  chunk_weather.rs     # L0 (existing, extend)
  weather_effects.rs   # derived gameplay samples
  weather_visual.rs    # L3 only (existing)
```

**Substrate:** regional fields live in `AtmosphereClipmapStack` under `src/substrate/` or `systems/atmosphere/` — **one** clipmap owner per WSS-PLAN-004.

---

## Witness (future — **blocked by PLAN-WEATHER-DEFERRED-001**)

**No witness writers until PLAN-WEATHER-WITNESS-002.** See [`plan_weather_deferred_v1.md`](plan_weather_deferred_v1.md) §7.

---

## Hybrid (until WSS-SLAB-PR-2)

| Incumbent | v2 behavior |
|:---|:---|
| `ChunkWeather` ECS | **authoritative** for sim tick |
| `WeatherVisualPlugin` mesh | remains L3 until W-SIM-VFX-1 |
| `ChunkWeather` in render extract | **forbidden** — use `ClimateVisualAggregate` only |

---

## Open questions (v2 sign blockers)

1. Climate save slot in world header vs procedural-only  
2. Lightning: event bus vs field threshold  
3. Disease / population — defer to Stage 7 behavioral plan  

---

## Sign-off criteria (v1.0.0)

- [x] Designer: readability at strategic vs tactical ([`designer_parallel_workboard_v1.md`](designer_parallel_workboard_v1.md) contamination/smoke rows)  
- [x] Steward: `chunk_environment_set` order preserved  
- [x] Planner: waves W-SIM-1..4 accepted in [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md)  

---

## Sign-off

| Role | Status | Date | Evidence |
|:---|:---|:---|:---|
| `@planner` | **PASS** | 2026-05-27 | `debug_runs/wss_substrate_live.json` → `wss_atmos_clipmap_001.green: true`, `clipmap_levels_present: true`, `contamination_domain_present: true` |

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | SIGNED after atmos clipmap checkpoint satisfied |
| v0.9.0 | 2026-05-26 | DRAFT from runbook v1 — parallel planner wave |
