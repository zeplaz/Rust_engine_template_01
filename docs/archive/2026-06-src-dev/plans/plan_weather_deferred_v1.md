# PLAN-WEATHER-DEFERRED-001 — Weather lane deferral + witness policy `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WEATHER-DEFERRED-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` |
| **Status** | **SUPERSEDED (queue)** — implementation on parallel downtime lane [`plan_weather_parallel_lane_v1.md`](plan_weather_parallel_lane_v1.md) |
| **Runbook (design only)** | [`weather_simulation_runbook_v2_plan_v1.md`](weather_simulation_runbook_v2_plan_v1.md) |
| **WSS atmos exec** | [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) |
| **OPS lane** | [`tools/orchestrator/queues/OPS_LANE_REGISTRY.json`](../../tools/orchestrator/queues/OPS_LANE_REGISTRY.json) `program_id: weather` |
| **Validation** | skill **validation-first** — **no weather ValidationReport until witness v1** |

---

## 1. Verdict

**Queue policy (2026-06-02):** Weather implementation moved to **parallel downtime lane** — see [`plan_weather_parallel_lane_v1.md`](plan_weather_parallel_lane_v1.md). This doc retains **validation-first** and **tile-generation** rules.

**Atmospheric weather simulation** is **not shippable** until `weather_sim_live.json` rollup green. There is **no operational green** yet.

| Question | Answer |
|:---|:---|
| Is weather sim shippable? | **No** — witness writer not landed |
| Is there a witness JSON? | **Schema signed** — [`plan_weather_witness_002_v1.md`](plan_weather_witness_002_v1.md); file not written yet |
| May agents claim weather green? | **Forbidden** until `weather_sim_live.json` rollup |
| May coders open WEATHER-* slices? | **Yes — downtime only** (Coder C or A/B when primary queue empty); start **WEATHER-WITNESS-001** |

---

## 2. Disambiguation (do not merge lanes)

| Term | Lane | Status | Witness |
|:---|:---|:---|:---|
| **Weather / atmosphere sim** | `WeatherPlugin`, clipmap, precip GPU, logistics traction | **DEFERRED** | none |
| **`weathering`** (APS / grammar) | Material age on assembly nodes (`clean`…`heavy`) | **Active** (procedural grammar) | APS / construction witnesses — **not** weather lane |
| **`weathered`** (APS tag) | `condition` tag for variants | **Active** | material / tile tags — **not** live sim |
| **Tile `damage` / `power` axes** | Tile variant state machine | **Active** (tile-generation skill) | tile atlas witnesses — **authored**, not driven by live weather yet |
| **WSS substrate atmosphere** | Slab + clipmap checkpoint | **Partial** | `wss_substrate_live.json` — **substrate**, not full weather sim sign-off |

**Rule G-WX-01:** `weathering` on `BuildingGrammar` / AssetSpec is **not** proof that atmospheric weather sim is wired.

---

## 3. What exists in repo today (honest)

| Piece | Location | Authority |
|:---|:---|:---|
| `WeatherPlugin` | `src/systems/weather/` | Runtime scaffold — **not** v2 three-tier sim |
| `GpuWeatherFireFieldPlugin` | render | Visual / field — **not** lane closure |
| `WeatherVisualSettings` | HUD | Presentation |
| `weather_penalty` on site staging | `strategic/site/components.rs` | Stub scalar — **no** regional sample |
| `GlobalRenewableWeatherFactors` | power | Partial hook — **not** weather witness |
| Runbook v2 waves W-SIM-1..4 | planner doc | **Not queued** in coder active |

**Do not** treat `--test weather` harness or lib fixture green as program lane green.

---

## 4. validation-first policy (agents)

Until witness v1 ships:

| Action | Allowed? |
|:---|:---:|
| `validate-report bevy` for construction / stage5 | yes |
| `validate-report asset_glb` for MCP art | yes |
| Claim `programs.weather[].operational_green` | **no** |
| Add fake `weather.green: true` to any JSON | **no** |
| Paste raw weather test logs into chat | **no** — no validator profile yet |

**When witness lands:** add `validate-report weather` + MCP `validate_weather_report` per [`plan_validation_runtime_v1.md`](plan_validation_runtime_v1.md).

---

## 5. tile-generation policy

Per **tile-generation** skill — tile variants are **state machines** with axes (`damage`, `power`, `lighting`, …).

| Topic | Policy while weather deferred |
|:---|:---|
| **Authoring** | Keyframe / tilemapgen spine — **manual or scripted** variant sets |
| **Live sim coupling** | **Forbidden** — no "rain → mud tile" auto bake from `ChunkWeather` |
| **`burning_*` / fire frames** | Fire lane (`fire_vfx`) — separate program |
| **Future** | **PLAN-WEATHER-TILE-COUPLING-003** after weather witness + traction overlay stub (W-SIM-4) |

**Do not** block PG-2 / tile production pilot on weather sim.

---

## 6. Reactivation gates (future planner slice)

Open weather implementation only when **all** true:

| # | Gate |
|:---:|:---|
| G1 | **PLAN-WEATHER-WITNESS-002** — **SIGNED**; implement **WEATHER-WITNESS-001** writer |
| G2 | WSS atmos clipmap checkpoint — **met** (`wss_atmos_clipmap_001.green`) |
| G3 | Downtime queue **WEATHER-CLIMATE-001** — see parallel lane doc |
| G4 | OPS registry `anchor_witnesses` includes `weather_sim_live.json` |
| G5 | `validate-report weather` green on fixture (after writer) |

**Signed runbook v2** remains **design authority**; this doc controls **queue + witness honesty**.

---

## 7. Future witness keys (draft — not active)

Do **not** implement writers until **PLAN-WEATHER-WITNESS-002** signs.

| Key | Meaning |
|:---|:---|
| `weather_sim_live.json` / `climate_seed_present` | L3 climate resource |
| `regional_weather_wired` | L2 clipmap sample → chunk |
| `weather_effects_traction_stub` | logistics sample hook |
| `weather_precip_gpu_authority` | mesh precip demoted |
| `green` | rollup — **false until above** |

Cross-link substrate only: `wss_substrate_live.json` atmosphere fields ≠ weather program closure.

---

## 8. OPS / orchestrator

| Artifact | Update |
|:---|:---|
| [`OPS_WITNESS_SPINE.md`](../../tools/orchestrator/queues/OPS_WITNESS_SPINE.md) | `weather` row → this doc |
| [`OPS_LANE_REGISTRY.json`](../../tools/orchestrator/queues/OPS_LANE_REGISTRY.json) | `status: deferred`, `owner: @planner`, `plan: plan_weather_deferred_v1.md` |
| `unified_witness_index.json` | omit `weather` program entries until first witness write |
| `coder_active_queue.json` | WEATHER-* in `weather_program.downtime_queue[]` / `coder_c.active[]` only |

---

## 9. Anti-patterns

| Don't | Do |
|:---|:---|
| Weather lane green from Stage 5 readiness alone | Wait for `weather_sim_live.json` |
| Conflate APS `weathering: medium` with sim | Separate docs / witnesses |
| Auto tile bake from undeclared weather ECS | Keyframe spine until coupling plan |
| Reopen W-SIM waves without witness schema | PLAN-WEATHER-WITNESS-002 first |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Signed deferral; validation-first + tile-generation policy; no witness |
| v1.1.0 | 2026-06-02 | Queue superseded by parallel downtime lane; witness-002 signed |
