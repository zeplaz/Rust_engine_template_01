# PLAN-WEATHER-PARALLEL-001 — Weather parallel downtime lane `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WEATHER-PARALLEL-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` |
| **Status** | **SIGNED — PARALLEL DOWNTIME** |
| **Supersedes queue policy in** | [`plan_weather_deferred_v1.md`](plan_weather_deferred_v1.md) §1 (implementation no longer frozen) |
| **Runbook (design)** | [`weather_simulation_runbook_v2_plan_v1.md`](weather_simulation_runbook_v2_plan_v1.md) |
| **Witness** | [`plan_weather_witness_002_v1.md`](plan_weather_witness_002_v1.md) |
| **OPS lane** | `program_id: weather` in [`OPS_LANE_REGISTRY.json`](../../tools/orchestrator/queues/OPS_LANE_REGISTRY.json) |

---

## 1. Verdict

**Atmospheric weather** is its **own system** and may advance on a **parallel downtime lane** — **Coder C** (dedicated) or **Coder A/B in downtime** — without blocking or preempting construction, infrastructure, MCP art, or procedural growth.

| Question | Answer |
|:---|:---|
| Is weather sim shippable today? | **No** — witness v1 not written yet |
| May coders work weather now? | **Yes** — downtime pull only; first slice **WEATHER-WITNESS-001** |
| Does weather block CON-P2 / INFRA / MCP? | **No** — hard boundary; minimal read hooks only |
| Operational green? | **Only** when `debug_runs/weather_sim_live.json` rollup green per witness-002 |

---

## 2. Pull policy (downtime)

```text
Primary:   Coder A / B → coder_a.active[] / coder_b.active[]  (CON + INFRA + fleet)
Downtime:  Coder C OR any coder with empty/blocked primary active[] → weather_program.downtime_queue[]
Never:     Weather PR preempts CON-P2, INFRA-E*, PROC-PG*, ECON-OG*, or tools/mcp/*
```

| Rule | Detail |
|:---|:---|
| **D-PULL-01** | Finish or unblock primary `active[]` before first weather slice unless assigned **Coder C** |
| **D-PULL-02** | Weather PRs **≤3 files**; default territory `src/systems/weather/` + witness writer |
| **D-PULL-03** | One cross-system consumer touch per PR max — steward review if outside weather territory |
| **D-PULL-04** | No edits to `src/construction/`, `src/infrastructure/`, `tools/mcp/` in weather PRs |

**Machine mirror:** [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) → `weather_program` + `coder_c`.

---

## 3. System boundary (weather owns sim; others read scalars)

Weather is **authoritative** for climate → regional clipmap sample → chunk local → presentation extract. Other programs **must not** duplicate weather state or write `ChunkWeather`.

### 3.1 Owned territory (Coder C default)

| Path | Role |
|:---|:---|
| `src/systems/weather/` | Sim tick, diagnostics, precip VFX plugin |
| `ClimateState` (future) | L3 slow climate resource |
| `regional_weather_tick` (future) | L2 clipmap writers (weather fields only) |
| `debug_runs/weather_sim_live.json` | Program witness writer |
| Render: `WeatherVisualPlugin`, GPU weather field extract | Presentation only |

**WSS coordination:** `src/substrate/atmosphere/` clipmap **types** are shared substrate — weather lane adds **weather field writers**; contamination/smoke domain stays WSS-owned. Dual-write to slab: [`post_spine.rs`](../substrate/post_spine.rs) — weather PRs may extend **weather scalars only** with `@sim-steward` ack.

### 3.2 Approved cross-system hooks (read-mostly, minimal impact)

| Consumer program | Hook | Direction | Allowed impact |
|:---|:---|:---|:---|
| **Power / renewables** | `GlobalRenewableWeatherFactors` | weather → read | `wind_capacity_factor`, `solar_capacity_factor` (0.05–1.2) |
| **Render / lighting** | `ClimateVisualAggregate`, `WeatherVisualSettings` | weather → extract | Precip overlay, fog tint, exposure bias — **visual only** |
| **Terrain materials** | optional `ChunkWeather` read in `material_plugin` | read | Wetness / snow tint — **no** ontology mutation |
| **Construction staging** | `weather_penalty: f32` on site components | weather → publish scalar (future) | Staging delay multiplier stub — **default 1.0** until wired |
| **Logistics / infra** | slab `dynamic.congestion` traction stub (W-SIM-4) | weather reads transport; mirrors scalar | Traction sample only — **no** graph topology changes |
| **MCP / APS / tiles** | — | **none** | Zero until [`PLAN-WEATHER-TILE-COUPLING-003`](plan_weather_deferred_v1.md) |
| **Construction execute** | — | **none** | No weather gates on commit funnel |
| **Transport graph** | — | **none** | No edge/corridor mutation from weather |

### 3.3 Forbidden (anti-patterns)

| Don't | Why |
|:---|:---|
| Rain → auto tile variant bake | Tile-generation skill — keyframe until coupling plan |
| Weather PR edits `execute_construction_plans_system` | Construction authority |
| Weather PR edits `TransportGraph` / R8 snapshot | Infrastructure authority |
| `ChunkWeather` in render extract as sim truth | Runbook v2 — use `ClimateVisualAggregate` |
| Claim weather green from `wss_substrate_live.json` alone | Substrate ≠ weather program closure |

---

## 4. Implementation train (downtime queue)

Order from runbook v2 — **one slice at a time**:

| # | ID | Deliverable | Files (typical) |
|:---:|:---|:---|:---|
| 0 | **WEATHER-WITNESS-001** | `weather_sim_live.json` writer + schema | witness module, envelope |
| 1 | **WEATHER-CLIMATE-001** | `ClimateState` + `climate_slow_tick` | `systems/weather/climate.rs`, mod |
| 2 | **WEATHER-REGIONAL-001** | L2 sample → `ChunkWeather` authority | chunk_weather.rs, post_spine (steward) |
| 3 | **WEATHER-EFFECTS-001** | traction + visibility gameplay stubs | post_spine, weather effects |
| 4 | **WEATHER-GPU-PRECIP-001** | demote mesh precip; GPU authority flag | weather_visual.rs, render |

**Dependency:** W-SIM-1..3 may use existing `AtmosphereClipmapStack` (substrate checkpoint green). No wait on INFRA-E2+.

---

## 5. Disambiguation (unchanged)

| Term | Lane |
|:---|:---|
| **Weather / atmosphere sim** | This doc + runbook v2 |
| **`weathering`** (APS grammar material age) | Procedural grammar — **not** this lane |
| **Tile `damage` / `power` axes** | Tile-generation — authored, not live weather |

---

## 6. validation-first

| Until witness-002 signed | After first writer lands |
|:---|:---|
| No fake `weather.green: true` | `validate-report weather` on fixture |
| Substrate keys ≠ weather program green | OPS `anchor_witnesses` includes `weather_sim_live.json` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Signed parallel downtime lane; Coder C + A/B downtime; cross-system boundary table |
