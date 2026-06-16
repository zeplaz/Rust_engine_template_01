# PLAN-WEATHER-WITNESS-002 — Weather program witness schema `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WEATHER-WITNESS-002** |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Lane policy** | [`plan_weather_parallel_lane_v1.md`](plan_weather_parallel_lane_v1.md) |
| **First coder slice** | **WEATHER-WITNESS-001** (implement writer) |

---

## 1. Anchor file

| Path | Role |
|:---|:---|
| `debug_runs/weather_sim_live.json` | **Program** witness — weather lane closure only |

**Not substitutes:** `wss_substrate_live.json` (`wss_post_spine_001`, `wss_atmos_clipmap_001`) prove substrate bridge, not weather program rollup.

Envelope: [`debug_run_envelope.rs`](debug_run_envelope.rs) + `_agent_meta` per [`debug_runs/README.md`](../../debug_runs/README.md).

---

## 2. Schema (v1 keys)

```json
{
  "gate": "WEATHER-SIM-LIVE-001",
  "green": false,
  "climate_seed_present": false,
  "climate_state_wired": false,
  "regional_weather_wired": false,
  "chunk_weather_from_regional": false,
  "weather_effects_traction_stub": false,
  "weather_precip_gpu_authority": false,
  "renewables_from_clipmap": false,
  "weather_sim_ticks": 0,
  "regional_weather_sample": 0.0,
  "cross_system_hooks": {
    "renewable_factors_read": false,
    "visual_extract_only": true,
    "construction_penalty_published": false,
    "tile_coupling_forbidden": true
  }
}
```

### Rollup rules

| Key | True when |
|:---|:---|
| `climate_seed_present` | `ClimateState` resource exists with non-default seed / season phase |
| `climate_state_wired` | `climate_slow_tick` runs under `SimControlState::should_tick` |
| `regional_weather_wired` | L2 clipmap sample drives regional tick (not witness-only) |
| `chunk_weather_from_regional` | `ChunkWeather` lerp targets from regional sample, not cell-matrix heuristic alone |
| `weather_effects_traction_stub` | W-SIM-4 slab traction mirror active |
| `renewables_from_clipmap` | `GlobalRenewableWeatherFactors` updated from clipmap path |
| `weather_precip_gpu_authority` | mesh precip demoted; GPU field authoritative |
| **`green`** | `climate_state_wired` && `regional_weather_wired` && `chunk_weather_from_regional` && `renewables_from_clipmap` |

`green` may stay **false** while downtime train progresses — partial keys are honest progress.

---

## 3. Writer placement

| Option | Location |
|:---|:---|
| Preferred | `src/systems/weather/witness.rs` + register in test harness / visual proof path |
| Cross-link | Index entry in `debug_runs/agent_debug_index.json` on write |

**Slice WEATHER-WITNESS-001:** implement writer with all keys present (booleans false until later slices flip them).

---

## 4. Validator (future)

Add profile `WEATHER` to validation-first when writer exists — see [`plan_validation_runtime_v1.md`](plan_validation_runtime_v1.md). Until then: lib test asserting JSON shape + rollup logic.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Signed schema; unblocks WEATHER-WITNESS-001 |
