# Plan: weather and wind field v1

Status: W1 SHIPPED (2026-11-13: `src/systems/weather/{field,systems,tuning}.rs`, `assets/configs/weather/weather_v1.ron`, save overlay `src/io/save/weather_overlay.rs`, dump `src/dev/sim_run_dump_weather.rs`); W2–W5 PLANNED. Owner: `coder` (sim), `designer` (overlay/HUD/clouds).
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
2. **W2 Fire + smoke:**
   - anisotropic spread and slope, embers and spot fires from the sampled wind, L0 advection per cell
   - fire VFX use local wind
   - delete `GlobalWind` and `PREVAILING_WIND_DIR`
3. **W3 Visuals + player read:**
   - local precip, the cloud cover layer (map and voxel), the weather map mode, the HUD readout
4. **W4 Combat / logistics / power:**
   - artillery drift and dispersion, visibility ranges, mud mobility, truck weather speed, per-plant renewables
5. **W5 Air / missile hooks:**
   - grounding thresholds and the headwind model, landing with those units

Budget: one step costs well under 1 ms even at 64×64 cells. `sample()` is O(1). Nothing weather-related runs every frame except interpolated sampling by the consumers that need it.

## 5. Decisions

1. **Weather cell size:** 1 km (recommended; zones are visibly distinct and cheap) or 512 m (= 1 sim chunk, a direct 1:1 with `ChunkWeather`).
2. **Fire spread:** add diagonal neighbours (recommended; wind at 45° otherwise makes staircase fronts) or keep 4 cardinal neighbours.
3. **Day/night and seasons:** keep the season phase only, or add a diurnal cycle (fog at dawn, convective storms in the afternoon). The recommendation is to defer diurnal to after W4.
