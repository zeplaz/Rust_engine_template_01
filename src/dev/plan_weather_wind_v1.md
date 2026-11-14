# Plan: weather and wind field v1

Status: W1–W5 SHIPPED; **§4e D0 OPEN** (2026-11-13: drone Link/Sensor/Flight envelopes on air_ops — see [`plan_drone_subsystem_fit_v1.md`](plan_drone_subsystem_fit_v1.md)). Owner: `coder` (sim), `designer` (overlay/HUD/clouds).
Units: metric throughout (m/s, mm/h, °C, m visibility).

## 1. What exists today (and why wind looks fake)

| Piece | File | Problem |
|---|---|---|
| `ChunkWeather` (rain, fog, snow, wind_speed 0..1, lightning, visibility, soil) | `src/systems/weather/chunk_weather.rs` | Wind is derived from the **temperature spread inside one chunk**. It has no direction and no movement, and it is re-lerped every frame using real time. |
| `GlobalWind { direction, speed }` | `src/systems/atmosphere/field.rs` | Only the test harness writes it. In real play the speed is 0, so smoke **never drifts**. |
| `PREVAILING_WIND_DIR` const | `src/systems/fire/ember_physics.rs` | Ember and smoke direction is hard-coded. |
| Fire spread | `src/systems/fire/cell_spread.rs` | Spreads the same in all directions. Wind only burns fuel faster. |
| Fire/rain VFX | `render/fire_vfx`, `render/weather_vfx` | Read one **world-mean** wind/rain value. |
| Renewables | `GlobalRenewableWeatherFactors` | Use a world mean, not each plant's location. |
| `climate.rs`, `regional_sample.rs` | `src/systems/weather/` | **Orphans**: not in `mod.rs`, so never compiled. |
| Combat / logistics / AI | — | Read no weather at all. Artillery is a `Zone{Fire}` stamp with no shell flight. |

Keep the good parts:
- the clipmap L0 smoke/ash/ember advection (`advect_l0_with_wind`)
- the `ChunkWeather` consumers in ecology, fire and LOD
- the precip VFX pipeline

## 2. Model: a coarse weather grid driven by a few moving pressure systems

One authority: **`WeatherField` resource**, the only writer of weather truth.

- **Grid:** weather cells of a fixed **1 km × 1 km** (data-driven in RON). The grid grows with the map, so larger maps get more zones for free.
  - 8 km default map → 8×8 cells. A 32 km map → 32×32 cells.
  - Cost per step is cells × systems, which is trivial.
- **Climate (slow, about once per game hour):**
  - season phase and base temperature
  - **prevailing wind** (direction plus mean speed), from the world seed
- **Weather systems (2–8 structs):** highs and lows, each with:
  - centre, radius, strength (hPa), velocity and humidity load
  - lifetime: born, drift with the prevailing steering wind, strengthen and fade
  - seeded RNG from the world seed, so runs are deterministic
- **Field solve, every `weather_step_secs` (default 60 s game time = 60 sim ticks):**
  1. **Pressure** = baseline + sum of the systems' Gaussians.
  2. **Wind** = prevailing wind + geostrophic wind (perpendicular to the pressure gradient, turned about 20° toward low pressure for surface friction). Then terrain adjusts it, using factors **baked once** from `ChunkCellMatrix` per weather cell:
     - **Height exposure:** ridges ×1.3, valleys ×0.7.
     - **Roughness:** forest and city ×0.6.
     - **Valley channelling:** the wind turns toward the valley's axis.
  3. **Humidity** is advected by the wind on the coarse grid (semi-Lagrangian, cheap):
     - sources: water cells from `WorldWaterGrid`, plus the systems' humidity load
     - sink: rain
  4. **Cloud cover** comes from humidity and lift:
     - low pressure, plus **orographic** lift on windward slopes
     - downwind of ridges this gives a rain shadow
  5. **Rain** (mm/h) where cloud is above a threshold. It becomes snow when temperature < 0 °C.
     - **Fog** where humidity is high, wind is low and it is cool.
     - **Lightning** where it is warm, humid and pressure is low.
  6. **Visibility** (m) comes from fog, rain and smoke (L0 smoke is sampled).
- **No per-frame weather compute.** The field keeps two snapshots (`prev`, `next`) and the step time. Consumers call:

```rust
WeatherField::sample(pos_m: Vec2, sim_time: SimTimeMicros) -> WeatherSample {
    wind_mps: Vec2, gust_mps: f32, temp_c: f32, humidity: f32,
    cloud: f32, rain_mm_h: f32, snow: bool, fog: f32, visibility_m: f32, lightning: f32,
}
```

- `sample` interpolates bilinearly in space and linearly in time between `prev` and `next`, so changes are smooth with no per-frame cost.
- **Gusts** are a deterministic hash of the cell and a 5 s time bucket, scaled by speed and roughness. They are computed on demand and stored nowhere.
- **Save/load:** climate, systems, both snapshots and the RNG state go in the RON snapshot.
- **Scenario override:** RON `weather_preset` (calm / windy / storm front / fire weather). This replaces the test-harness wind poking.

## 3. Integration per system

| Consumer | Change |
|---|---|
| **`ChunkWeather`** | Becomes a **projection** written once per weather step from `WeatherField`, which keeps existing readers working. Adds `wind_mps: Vec2`. Keeps `wind_speed` 0..1 as derived (`|wind| / 20 m/s`) until readers migrate. Its own target formula is deleted. |
| **Smoke / ash / embers (clipmap L0)** | `advect_l0_with_wind` takes the per-cell wind from the field instead of `GlobalWind`. **Delete `GlobalWind`.** Smoke moves in real play for the first time. |
| **Fire spread** | Anisotropic heat gain to a neighbour: `gain × (1 + k_w·max(0, d̂·ŵ)·|w|/10) × (1 − u_w·max(0, −d̂·ŵ))` plus an uphill slope term. Diagonal neighbours are optional (decision 2). Rain (mm/h) and humidity drive fuel moisture; this replaces the 0..1 proxy. |
| **Embers / spot fires** | `ember_physics` launches with the sampled wind vector, replacing `PREVAILING_WIND_DIR`. Spot fires land downwind at a distance ∝ wind × loft time, so a strong wind jumps rivers and roads. |
| **Fire VFX** | `FireVisualFrame` carries the local wind per emitter, taken from the extract sample, not the world mean. Flame lean and smoke direction match the sim. |
| **Rain / snow VFX, clouds** | Precip streaks appear only where it is raining (local sample). **Clouds:** a low-res cloud-cover texture with drifting shadows on the map. In the voxel view, cloud sheets sit at altitude and move with the wind. This attaches to the projection graph with no parallel extraction. |
| **Artillery (now)** | At shot resolution, sample the wind at the midpoint of the path. The impact offset is `drift_coef × wind × flight_time`, and dispersion grows with gusts. Fog and rain reduce observed accuracy, which widens the fire-zone stamp radius. Data lives in `front_balance` RON per unit: `wind_drift_coef`, `max_wind_mps`. |
| **Missiles / aircraft / drones (when they exist)** | Use the same `sample()`. Guided missiles take a small drift. Aircraft use headwind and tailwind for ground speed and fuel, and have crosswind limits. **Grounding thresholds** come from RON: wind, visibility, lightning. The API is defined now; the units come later. |
| **Combat pressure** | Visibility scales detection and engagement range. Mud (soil moisture from rain accumulation) cuts off-road mobility in the front `mobility` term. |
| **Logistics** | Truck speed is multiplied by the rain/snow/mud factor (off-road worse than road). Optionally a high-wind hold for future airlift. |
| **Power** | Wind and solar plants sample **their own cell** (wind cubed toward rated, cloud cover for solar). Delete `GlobalRenewableWeatherFactors`. |
| **Ecology** | `soil_moisture` and `vegetation_field` use real rain accumulation. Dryness feeds fire risk. |
| **AI** | Reads the forecast (the `next` snapshot): it avoids air ops and pushes in storms, and is aware of fire weather. Low priority. |
| **UI (play only)** | "Weather" map mode (wind arrows, rain, cloud; `ViewRepresentation::Wind` already exists). The HUD shows the weather under the cursor and camera via `player_read_hud`: `12 m/s NW, gusts 18, rain 4 mm/h, vis 2.1 km`. |
| **Review path** | The sim dump adds a weather series: mean/max wind, spatial stddev, rain coverage %, number of systems, the fire spread downwind:upwind ratio, the smoke centroid shift, and artillery drift. The matrix checks that wind varies across cells and over time, fire spreads downwind more than upwind, and smoke moves downwind. |

## 4. Phases (each ends with one playcheck)

1. **W1 Field core:**
   - `WeatherField`, climate, pressure systems, the step at 60 s game time, `sample()`, terrain bake, save/load, the RON preset
   - `ChunkWeather` becomes a projection
   - dump series
   - delete the orphans `climate.rs` and `regional_sample.rs`
2. **W2 Fire + smoke:** see §4a for the full scope, including the user-reported bugs.
   - anisotropic spread and slope, embers and spot fires from the sampled wind, L0 advection per cell
   - fire VFX use local wind
   - delete `GlobalWind` and `PREVAILING_WIND_DIR`
3. **W3 Visuals + player read:** see §4b.
4. **W4 Combat / logistics / power:** see §4c.
5. **W5 Air / missile hooks:** see §4d.

## 4a. W2 scope: fire behaviour (user-reported, 2026-11-13)

The user saw three problems in play. W2 is not done until all three are fixed and visible in the dump.

**Bug 1: fire spreads as a circle.**
- **Cause:** `src/systems/fire/cell_spread.rs` deposits the same heat on every cardinal neighbour (`spread_onto_neighbor`). Wind only burns fuel faster (`wind_fuel_delta`), and nothing depends on direction. On uniform fuel this gives a diamond or circle that grows evenly.
- **Target:** spread follows real fire behaviour, a simplified Rothermel / Huygens model:
  - **Wind:** the rate of spread goes up sharply downwind. The burnt area becomes an elongated ellipse or teardrop; the length:breadth ratio is about 1 + 0.25·U (U in m/s), so roughly 3:1 at 8 m/s.
    - The head fire runs fast, the flanks are slow and the back creeps upwind.
    - Use 8 neighbours (diagonals at ×1/√2 distance) so 45° wind doesn't give staircase fronts.
  - **Slope:** spread is faster uphill and slower downhill. Rate of spread roughly doubles every ~10° of upslope.
  - **Fuel:** rate and intensity depend on fuel kind and load.
    - Grass: fast, low intensity, short residence.
    - Brush: medium.
    - Timber: slow surface fire with a crown-fire transition when hot and windy.
    - Fuel moisture comes from humidity, rain accumulation and soil moisture.
  - **Uneven, not uniform:** per-cell fuel variation (seeded, from the stand), gusts from `WeatherField::sample` (5 s buckets) and a small stochastic ignition probability instead of a deterministic threshold. This gives fingers, unburnt islands and an irregular perimeter.
  - **Spotting:** embers are lofted by fire intensity and carried downwind by `sample()` wind, then ignite new spot fires ahead of the front. At high wind they jump roads and rivers.
  - **Fire weather:** rain and high humidity slow or stop spread; dry, hot, windy conditions accelerate it.
- **Dump/matrix:**
  - perimeter length:breadth ratio
  - downwind:upwind spread distance (expect > 2 when wind > 5 m/s)
  - unburnt islands count
  - spot fires count
  - burnt-area growth per minute

**Bug 2: trees "regrow" right after burning.**
- **Cause:** `ChunkCellStand` timings in `src/systems/ecology/cell_stand.rs` (`StandCfg::default`) are compressed to minutes. Snags stand for 6 / 25 / 90 s and regrow takes 25 / 120 / 600 s (grass / brush / timber), so a burnt forest is back to a full live stand in about **11.5 game minutes**.
  - The fire tick also refills the fuel reservoir up to `fuel_cap` ("clamp down, refill up").
  - `ChunkCellStand` is **not saved** (an open item), so a reload can also resurrect burnt stands.
- **Target:** burns are long-lived landscape changes.
  - **Burn scar (ground):** stays charred/ash for game days, then grass and herbs green up over weeks.
  - **Brush:** returns over months to a few game years.
  - **Timber:** snags stand for months to years. A young forest takes many game years and a mature one decades. Within a normal match a burnt forest stays burnt; it can only be replanted by a player or AI action, which W2 doesn't add.
  - Put the timings in RON in **game days**, tied to the sim clock and its time scale, not raw seconds.
  - Fuel refill is limited by the regrowth stage, never instant.
- **Save/load:** persist `ChunkCellStand` (phase, age and burnt flags) in the save snapshot.
- **Visuals:** the tree layer shows charred ground → snags → sparse regrowth by stage, and never pops back to full canopy.
- **Dump:** a burnt-cell series. The burnt fraction must not decrease within a playcheck run without a replant action.

**Feature 3: firelines, firebreaks and suppression.**
- **Natural breaks:** roads, rails, rivers/water, bare rock, built-up cells and previously burnt cells have no or little fuel and stop surface spread. Spotting can still jump them at high wind.
- **Player/AI firebreaks:** a construction tool clears a line of fuel (bulldozed break) through the existing construction funnel (`src/construction/`, single execute path).
  - It takes time and workers.
  - A wider break is needed against crown fire.
- **Backburn (later):** a controlled burn set ahead of the front to consume fuel. This depends on the W2 spread model and comes after it.
- **Suppression hooks (later):** rain, and later units or aircraft (W5), that reduce heat or add moisture along a line.
- **Dump/matrix:** a spread-blocked-at-break count. A fire against a road or river with wind under a threshold must not cross.

Budget: one step costs well under 1 ms even at 64×64 cells. `sample()` is O(1). Nothing weather-related runs every frame except interpolated sampling by the consumers that need it.

### Post-W2 fire fidelity (do **not** expand the stencil to 16+)

W2 locks an **8-neighbour square stencil** weighted by a **continuous** wind vector (`WeatherField::sample` → `ChunkWeather.wind_mps`) and slope. Agents must not “upgrade” to 16/32 discrete headings on this grid — that is the wrong lever.

| Layer | W2 | Higher fidelity (later) |
|---|---|---|
| Wind input | Continuous `Vec2` m/s + gust hash | Same; never quantize wind to N buckets |
| Deposit topology | 8-neigh (diagonals ×1/√2) | Keep 8 on the square surface grid |
| Rate of spread | Anisotropic gain on deposits (head/flank/back) | Full **elliptical ROS / Huygens**: compute head–flank–back rates from Rothermel-like wind+slope+fuel, then weight the same 8 deposits (or integrate along the front) |
| Front geometry | Irregular via fuel jitter + gust + ignition hash | Optional **level-set / distance-field** perimeter for smooth firelines and break contact |
| Smoke / ash flow | Per-texel L0 advection from `sample()` | Same continuous field; flows are not neighbour-count problems |
| Firebreaks | Fuel mask + spotting jump | Continuous front vs strip width; crown needs a wider clear |
| Voxel / 3D | Out of W2 | Separate **vertical plume + buoyancy** on voxels; surface fire can stay 2D 8-neigh with continuous wind. Do not invent a 16-dir 2D stencil “for voxels.” |

**Rejected:** 16+ direction stencils on square cells (knight/long-range offsets, uneven distance weights, diminishing return vs elliptical ROS).

**Next fidelity step after W2 ships:** elliptical ROS kernel driving the existing 8-neigh path in `cell_spread` / `chunk_fire_overlay`, then level-set if fireline tools need sub-cell contact tests.

## 4b. W3 scope: visuals + player read (2026-11-13)

W3 makes the field **readable** without inventing a second weather extract.

| Deliverable | Attach point | Done when |
|---|---|---|
| Local precip | `CameraLocalWeatherSample` ← `WeatherField::sample(camera)`; `weather_vfx` streaks + shader lean from `wind_mps` | Rain density follows camera cell; lean matches sim wind |
| Cloud / wind bake | `bake_weather_overlay_from_field` → `OverlayFieldFrame.fields[Wind]` (R=cloud G=rain B/A=wind dir) | `WeatherOverlayBakeState.published` |
| Weather map mode | `MinimapOverlayMask.weather` + tray; wash via ecology compositor slot from Wind bake | Toggle shows cool cloud/rain wash |
| Voxel cloud sheets | Procedural quads under weather VFX root; opacity from `sample().cloud` | Sheets appear when cloud high; drift with wind |
| HUD readout | `WeatherPlayerReadFrame` + `format_tactical_wx_line` → ops strip | `12 m/s NW, gusts …, rain … mm/h, vis … km` (compact strip keeps `r/s/f`) |
| Projection | `WeatherProjectionNode` on `RenderProjectionGraph` | Bake revision mirrored; no parallel scan |
| Dump / matrix | `overlay_wind_published`, `local_rain_vs_mean_abs`, `mean_cloud` | Matrix rows green under rainy/storm runs |

**Authority:** `WeatherField::sample` only. `ClimateVisualAggregate` stays world means for legacy GPU fire paths.

**Plan wording note:** older text said `ViewRepresentation::Wind` — the live enum is `OverlayChannel::Wind`.

### Post-W3 fidelity (do not invent parallel clouds)

| Layer | W3 | Higher fidelity (later) |
|---|---|---|
| Precip | Camera + per-streak `sample` | Full screen-space rain from field texture |
| Cloud sheets | Few drifting quads | Soft volumetric / sky dome; map shadows from cloud texture |
| Map mode | Wash + encoded wind in bake | Sparse chevron glyphs at weather-cell centres |
| HUD | Camera + voxel cursor pick | Top-down cursor via `ConstructionMapProjection` |

**Rejected:** second ECS weather scan for VFX; AI-generated cloud art; volumetric voxels as a W3 requirement.

## 4c. W4 scope: combat / logistics / power (2026-11-13)

| Deliverable | Attach point | Notes |
|---|---|---|
| Artillery drift | `artillery_weather_stamp` → `sync_formation_zones_system` | Offset `Zone{Fire}` centre downwind; widen radius with fog/rain/gust. RON: `UnitProfile.wind_drift_coef` / `max_wind_mps` |
| Engagement visibility | `Combatant.visibility_scale` in `resolve_step` | `0.35 + 0.65·vis·fog`; sampled from `WeatherField` at formation tile |
| Mud mobility | `strategic_fields_coupling_tick` | `mobility_cost += 0.35·soil_moisture + 0.20·snow` (same writer as throughput) |
| Truck haul speed | `LogisticsOrderSim.weather_haul_mul` | `weather_haul_speed_mul(sample)` before `travel_ticks` |
| Renewables | `variable_renewable_placeholder` (already per-plant) | `RenewableWeatherFactors` + ops PWR derate; **no** `GlobalRenewableWeatherFactors` in Rust |
| Dump / matrix | `artillery_drift_tiles`, `haul_weather_speed_mul`, `wind_cf_*` / `solar_cf_*` | Soft thresholds (calm maps may show zero arty drift) |

**Authority:** `WeatherField::sample` / `ChunkWeather` projection only. Keep single `Zone{Fire}` writer.

### Post-W4 fidelity

| Layer | W4 | Later (W5+) |
|---|---|---|
| Artillery | Stamp drift (no shell flight entity) | True ballistic flight + impact events |
| Haul | World-centre mul | Per-leg midpoint / road vs off-road graph |
| Air | — | Grounding thresholds RON + headwind model (see §4d) |

## 4d. W5 scope: air / missile hooks (2026-11-13)

| Deliverable | Attach point | Notes |
|---|---|---|
| RON profiles | `assets/configs/weather/air_ops_v1.ron` → `AirOpsTuning` | FixedWing / Rotor / Missile / Airlift: wind, crosswind, visibility, lightning, cruise, drift |
| Grounding | `evaluate_grounding(sample, profile, runway_dir?)` | Reasons: Wind, Crosswind, Visibility, Lightning |
| Headwind model | `air_ground_speed_mul` / `air_fuel_mul` | Ground speed `0.35..=1.25`; fuel `1 + k·headwind` |
| Missile drift | `missile_weather_drift` | Pure metres offset; no Zone stamp (no missile entities yet) |
| Airlift hold | `airlift_hold` → `AirOpsWeatherStatus.airlift_hold` | High wind/vis/lightning hold for future airlift |
| Status | `sync_air_ops_weather_status` | Sole writer; centre-sample projection for dump/AI |
| Dump / matrix | `air_ops_grounded`, `airlift_hold`, `air_ground_speed_mul`, `missile_drift_m` | Soft on calm maps (grounded/hold often 0) |

**Authority:** `WeatherField::sample` only. **No** aircraft / missile / drone ECS entities in W5 — API landing pad for later units.

**Out of W5:** AI forecast consumer, true ballistic artillery, fire-suppression aircraft lines, elliptical ROS.

## 4e. Drone envelopes (D0 — primary air_ops consumer)

Full program: [`plan_drone_subsystem_fit_v1.md`](plan_drone_subsystem_fit_v1.md). Weather owns the envelopes; combat/logistics/mission UX consume them.

| Deliverable | Attach point | Notes |
|---|---|---|
| Drone* RON profiles | `air_ops_v1.ron` → `AirOpsTuning` | DroneFpv / DroneRecon / DroneCruise / DroneInterceptor |
| Link envelope | `link_quality(sample, smoke01, modality, resist)` | Radio fragile to smoke+lightning; Fiber resists RF; Sat partial; Autonomy resists C2 loss |
| Sensor envelope | `sensor_range_mul(sample, smoke01, sensor, resist)` | EO hard vs fog/smoke; IR partial; Radar softer on smoke |
| Flight | reuse grounding / `air_ground_speed_mul` | Launch/recover abort under wind/vis |
| Status probes | `AirOpsWeatherStatus` | `link_quality_radio`, `link_quality_fiber`, `sensor_range_mul_eo`, `drone_cruise_grounded` |
| Dump / matrix | those keys | Soft on calm maps |

**Authority:** one `WeatherField::sample` + `smoke01` from [`ChunkSmokeField`](../systems/fire/chunk_smoke_field.rs) (L0-pulled) with L0 clipmap fallback. No drone ECS entities in D0.

**Out of §4e / D0:** MissionBook, formation packages, production plants, presentation LOD (D1–D8).

## 5. Decisions

1. **Weather cell size:** 1 km (recommended; zones are visibly distinct and cheap) or 512 m (= 1 sim chunk, a direct 1:1 with `ChunkWeather`).
2. **Fire spread (locked for W2):** **8 neighbours** + continuous wind/slope weighting. Not 4-only. Not 16+. See post-W2 fidelity above.
3. **Day/night and seasons:** keep the season phase only, or add a diurnal cycle (fog at dawn, convective storms in the afternoon). The recommendation is to defer diurnal to after W4.
