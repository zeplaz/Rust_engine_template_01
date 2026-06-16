# Weather — player readability charter `v1` (DESIGN-WEATHER-PLAYER-READ-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-WEATHER-PLAYER-READ-001** · impl slice **DESIGN-WX-HUD-IMPL-001** |
| **Date** | 2026-06-03 |
| **Owner** | `@designer` (charter + §Implementation) · `@coder C` (wire) |
| **Verdict** | **PASS** (charter) · **PASS** (§Implementation spec) |
| **Plan** | [`plan_weather_parallel_lane_v1.md`](plan_weather_parallel_lane_v1.md) |
| **Runbook** | [`weather_simulation_runbook_v2_plan_v1.md`](weather_simulation_runbook_v2_plan_v1.md) |
| **Unblocks** | **WEATHER-GPU-PRECIP-001** UX review · **DESIGN-WX-HUD-IMPL-001** @coder C |
| **Impl witness** | [`debug_runs/weather_hud_player_read_live.json`](../../debug_runs/weather_hud_player_read_live.json) |

---

## Mission

Players must tell **weather state at a glance** in simulation — rain vs snow vs fog vs clear, tactical vs strategic zoom, and how weather differs from **fire smoke** and **WSS contamination**. This doc charters HUD, overlay, and minimap reads only.

**Acceptance test:** *At tactical zoom, a player can name active precip and whether visibility is reduced — without opening F3 diagnostics.*

---

## Authority model (presentation)

```text
ChunkWeather / ClimateVisualAggregate   → sim + extract (Coder C)
WeatherVisualPlugin / GPU precip        → tactical + background bands
Ops strip WX zone                       → numeric summary (always on in sim)
F3 diagnostics                          → engineer toggles — not player-critical path
```

**Rule:** Never use diagnostics-only toggles as the only weather read. Ops strip + world overlay must suffice.

---

## 1. Precip overlay — rain / snow / mesh fallback

| Band | Zoom signal | Player sees | Implementation hook |
|:---|:---|:---|:---|
| **Tactical** | `map_zoom_alpha > 0.45` | Directional rain/snow streaks on camera child mesh | `weather_precip_show_tactical` · `WEATHER_TACTICAL_PRECIP_ZOOM_ALPHA` |
| **Strategic / zoomed out** | `map_zoom_alpha ≤ 0.45` | Screen-space “digital AE” background streaks — not tactical flakes | `weather_precip_show_background` · `background_aesthetic` flag |
| **Fallback** | GPU off or `enabled == false` | Flat tint only from `ClimateVisualAggregate` — no silent “clear” when sim has rain | Overlay tint must track mean sample |

**Copy (ops strip):** `WX  r {rain:.2}  s {snow:.2}  f {fog:.2}` — locked in [`format_ops_wx_line`](../../src/gui/hud/simulation_shell_phase2.rs). **Do not** relabel `s` as wind; wind lives in renewables / future HUD extension.

**Minimap:** No particle precip on minimap — **color wash only** when mean rain/snow/fog > threshold (see §2). Same rule as hydrology D-W09 particles culled at strategic scale.

---

## 2. Fog / visibility — distinct from fire smoke and contamination

| Phenomenon | Hue family | Motion | Ops / HUD noun |
|:---|:---|:---|:---|
| **Weather fog** | Cool gray-blue desaturate | Slow uniform veil; rises with `fog_density` | `f` in WX line + optional `VIS` suffix when `< 0.85` |
| **Fire smoke** | Warm amber / orange particulate | Directional plumes, localized | **SMOKE** alerts — never reuse WX line |
| **WSS contamination** | Warm desaturate haze + channel patterns | Stipple / contour per [`wss_contamination_visual_language_v1.md`](wss_contamination_visual_language_v1.md) | Channel nouns (plume, spill) — not “fog” |

**Designer rule:** If fog and smoke share the same tint at tactical zoom, fix **hue separation** before adding density. Fog = **cool**; smoke = **warm**; contamination = **patterned overlay**, not full-screen tint alone.

**Minimap scale:** Fog reads as **soft edge darkening** on terrain base — max 35% alpha. No animated particles.

**Visibility stub:** When `visibility_factor < 0.85`, append ops text `VIS low` (future slice) — text required; no red-only flash.

---

## 3. Wind / solar hint — `GlobalRenewableWeatherFactors`

| Question | Answer |
|:---|:---|
| Show wind/solar on default player HUD? | **No full-time bands** in v1 — overloads ops strip |
| Where player may see coupling? | **PWR zone** when renewable derate active: `PWR  62%  (wind)` or `PWR  71%  (solar)` — only when factor `< 0.95` or `> 1.05` |
| F3 / debug | May show `wind_capacity_factor` / `solar_capacity_factor` numeric for engineers |
| Minimap | **No** wind arrows in v1 |

**Rationale:** Renewables are **gameplay consequence** of weather, not meteorology education. Pair with PWR, not WX.

---

## 4. Terrain wetness — subtle default

| State | Tactical read | Strategic / minimap |
|:---|:---|:---|
| **Dry default** | Base terrain albedo unchanged | Same |
| **Light rain** (`soil_moisture` ↑) | Slight specular darkening on soil tiles — no mirror flood | 5–10% darker green/brown band |
| **Heavy rain / flood coupling** | Pulse with hydrology depth — see [`wss_hydrology_player_read_v1.md`](wss_hydrology_player_read_v1.md) | Dim ribbon only — no shimmer |
| **Snow** | White accumulation on `snow_depth` — distinct from rain wetness | Pale cap on minimap cells |

**Default:** Wetness is **subtle** until witness green. Gameplay-readable threshold: moisture visible when `soil_moisture > 0.65` or `snow_depth > 0.1`.

---

## 5. Construction — `weather_penalty` badge (future)

| Rule | Spec |
|:---|:---|
| **When hidden** | `weather_penalty == 1.0` (default) — no badge, no tray row |
| **When shown** | Scalar ≠ 1.0 → site staging badge `Weather delay ×{scalar:.1}` on construction **ghost read-only** chrome |
| **Forbidden** | Weather gates on execute funnel · weather buttons in build rail |
| **Scope** | Presentation on staging overlay only — Coder C publishes scalar; construction owns execute |

---

## 6. Accessibility

| # | Requirement |
|:---:|:---|
| A1 | WX line always includes **letters + numbers** — `WX  r 0.20  s 0.10  f 0.05` |
| A2 | Precip active state duplicated: overlay tint **and** ops text (not color-only world) |
| A3 | Fog vs smoke vs contamination use **different words** in any player-facing label |
| A4 | Minimap weather wash includes **legend entry** in sim minimap tray when overlays on: `Wx wash = mean precip` |
| A5 | Diagnostics toggles (`Enable weather VFX`) — engineer-only; sim entry leaves player settings unchanged |

---

## 7. Non-goals

- Tile weather frames / MCP atlas / APS grammar `weathering` material age
- Transport graph weather edges
- Construction execute blocking
- Full meteorology panel (pressure fronts, isobars) — world-gen static climate only in editor
- Hanabi-scale particle storms at world zoom

---

## Band summary table

| Surface | Clear | Rain | Snow | Fog | Smoke (fire) |
|:---|:---|:---|:---|:---|:---|
| **Ops strip** | `r≈0 s≈0 f≈0` | `r↑` | `s↑` | `f↑` | **ALERTS** / fire channel — not WX |
| **Tactical view** | No overlay | Streaks + wet tint | White flakes + cap | Cool veil | Warm plumes |
| **Strategic view** | — | BG aesthetic streaks | BG aesthetic | Darkened base | Localized warm plume |
| **Minimap** | Base map | Dim cool wash | Pale cap | Edge darken | Separate fire overlay layer |
| **PWR coupling** | — | Solar derate optional | — | — | — |

---

## §Implementation — widget tree (sim session)

**Scope:** `BaseState::Simulation` only — same gate as PLAY-01 ops strip ([`simulation_session.rs`](../../src/gui/hud/simulation_session.rs)). Editor / WorldGen may show WX in diagnostics; player-critical path is sim.

```text
SimulationSession (OnEnter)
└─ HudCommandShellLayout (Bevy native, z≈1200)
   └─ Operations strip — full width
      ├─ OpsStripTime      ← SimTick + SimControlState
      ├─ OpsStripAlerts    ← ActiveMissions (fire channel — NOT WX)
      ├─ OpsStripIntel     ← StrategicOverlayDisplayPolicy
      ├─ OpsStripWeather   ← WeatherPrecipVisualSample  ★ primary player read
      ├─ OpsStripPower     ← GlobalRenewableWeatherFactors (derate) + scarcity fallback
      └─ OpsStripTray      ← ContextTrayState

WorldMain camera child (WeatherVisualPlugin)
├─ WeatherPrecipOverlay  ← cool tint from sample.rain/fog (fallback when GPU off)
└─ PrecipParticle mesh   ← tactical band OR background_aesthetic (zoom split)

Minimap chrome (egui overlay shell)
├─ MinimapCompositor raster
└─ minimap_legend line   ← append Wx wash legend when overlays on

F3 diagnostics (engineer-only — not player path)
└─ Weather section       ← WeatherVisualSettings toggles + sample dump
```

**Session rule:** `apply_simulation_hud_defaults` leaves ops strip **visible**; weather VFX use `WeatherVisualSettings::default()` (enabled) — do not require F3 toggle for player read.

---

## §Implementation — data bindings

| Surface | Source resource / fn | Consumer | Binding rule |
|:---|:---|:---|:---|
| **WX ops text** | `WeatherPrecipVisualSample` | `update_ops_strip_zone_lines_system` → `OpsStripWeather` | `format_ops_wx_line(rain, snow, fog)` — two decimals |
| **WX VIS suffix** | `WeatherEffectsSample.visibility_sample` | extend `format_ops_wx_line` or post-append | Append `  VIS low` when `< 0.85` — **text only** |
| **PWR derate** | `GlobalRenewableWeatherFactors` | `OpsStripPower` via `format_ops_power_derate_line` | `PWR  {pct}%  (wind)` or `(solar)` when factor `< 0.95` or `> 1.05`; else existing scarcity proxy |
| **Precip sample** | `ClimateVisualAggregate` + local `ChunkWeather` at camera focus | `sync_precip_sample_at_camera_focus` → `WeatherPrecipVisualSample` | **Already wired** — do not re-query chunks in HUD |
| **Overlay tint** | `WeatherPrecipVisualSample` | `update_weather_overlay_tint` | Cool `srgba(0.52, 0.58, 0.78, α)` — α from rain+fog; active when `enabled && overlay` |
| **Tactical streaks** | `map_zoom_alpha` + sample | `weather_precip_show_tactical` | `zoom_alpha > 0.45` |
| **Background streaks** | same + `background_aesthetic` | `weather_precip_show_background` | `zoom_alpha ≤ 0.45` |
| **Mesh fallback** | `WeatherVisualSettings` | particle visibility | When `gpu_precip_authority && mesh_precip_demoted` → mesh hidden; tint + ops must still reflect sim |
| **Minimap wash** | `ClimateVisualAggregate.mean_*` | minimap compositor tint pass | No particles; wash when `max(mean_rain, mean_snow, mean_fog_density) > 0.08` |
| **Minimap legend** | wash active flag | `HudAsyncTask::MinimapLegend` cache | Append ` · Wx wash = mean precip` when wash on |
| **Wetness tint** | `ChunkWeather.soil_moisture` / `snow_depth` via aggregate | terrain / slab visual extract | Subtle — threshold §4; **P1** after P0 witness green |
| **Construction badge** | `SiteStaging.weather_penalty` | ghost read-only chrome | Hidden at `1.0` — **P2** stub publish only |
| **Fire smoke** | `ChunkSmokeGpu` / fire extract | ALERTS + warm plumes | **Never** bind to WX zone |

**Forbidden:** `ChunkWeather` queries inside `src/gui/` except through published samples above. Render extract uses `ClimateVisualAggregate` only ([`plan_weather_parallel_lane_v1.md`](plan_weather_parallel_lane_v1.md)).

---

## §Implementation — copy helpers (@coder C)

Add or extend in [`simulation_shell_phase2.rs`](../../src/gui/hud/simulation_shell_phase2.rs):

```rust
// WX base (locked)
format_ops_wx_line(rain, snow, fog) → "WX  r {rain:.2}  s {snow:.2}  f {fog:.2}"

// VIS suffix (P0)
format_ops_wx_line_with_vis(rain, snow, fog, visibility_sample) → base + "  VIS low" when vis < 0.85

// PWR derate (P0)
format_ops_power_derate_line(pct, wind_factor, solar_factor) →
  "PWR  {pct:.0}%  (wind)" | "(solar)" | "PWR  {pct:.0}%" idle
```

| String | When |
|:---|:---|
| `WX  r 0.00  s 0.00  f 0.00` | Clear / no sample |
| `WX  r 0.20  s 0.10  f 0.05  VIS low` | Fog/rain reducing visibility |
| `PWR  62%  (wind)` | `wind_capacity_factor < 0.95` |
| `PWR  71%  (solar)` | `solar_capacity_factor < 0.95` (wind idle) |
| `Wx wash = mean precip` | Minimap legend suffix when wash active |

---

## §Implementation — slice order

| Priority | Slice | Files (max 1 cross-system per PR) | Exit |
|:---:|:---|:---|:---|
| **P0** | WX zone live + VIS suffix | `simulation_shell_phase2.rs` | `ops_wx_wired: true` |
| **P0** | PWR renewable derate copy | `simulation_shell_phase2.rs` | `pwr_renewable_derate_wired: true` |
| **P0** | Minimap wash + legend | minimap compositor + `hud_async_task_queue.rs` | `minimap_wx_wash_wired: true` |
| **P0** | Witness writer | `src/systems/weather/` or `src/dev/` proof hook | `weather_hud_player_read_live.json` |
| **P1** | Wetness / snow cap on terrain | visual extract consumer | `terrain_wetness_subtle: true` |
| **P2** | Construction `weather_penalty` badge | construction ghost chrome | `construction_wx_badge_wired: false` until scalar wired |

**Regression:** maintain `debug_runs/weather_sim_live.json` `green` — presentation slice must not flip sim rollup false.

---

## §Implementation — witness (`weather_hud_player_read_live.json`)

**Profile:** `WEATHER_HUD_PLAYER_READ` · **Gate:** `DESIGN-WX-HUD-IMPL-001`

```json
{
  "gate": "DESIGN-WX-HUD-IMPL-001",
  "green": false,
  "program_id": "DESIGN-WX-HUD-IMPL-001",
  "charter_doc": "docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md",
  "ops_wx_wired": false,
  "ops_wx_vis_suffix_wired": false,
  "ops_wx_sample_from_precip_visual": false,
  "pwr_renewable_derate_wired": false,
  "precip_overlay_fallback_wired": false,
  "precip_tactical_band_wired": false,
  "precip_background_band_wired": false,
  "minimap_wx_wash_wired": false,
  "minimap_wx_legend_wired": false,
  "f3_not_required_for_player_read": true,
  "terrain_wetness_subtle": false,
  "construction_wx_badge_wired": false,
  "weather_sim_live_maintained": false,
  "acceptance_player_read_at_glance": false
}
```

### Rollup rules

| Key | True when |
|:---|:---|
| `ops_wx_wired` | `OpsStripWeather` updates from `WeatherPrecipVisualSample` in sim (not static placeholder) |
| `ops_wx_vis_suffix_wired` | `VIS low` appended when `WeatherEffectsSample.visibility_sample < 0.85` |
| `ops_wx_sample_from_precip_visual` | Sample resource non-default after climate extract in proof harness |
| `pwr_renewable_derate_wired` | PWR line shows `(wind)` or `(solar)` per §3 when factors exceed band |
| `precip_overlay_fallback_wired` | Overlay tint tracks sample when `mesh_precip_demoted` |
| `precip_tactical_band_wired` | `weather_precip_show_tactical` true at tactical zoom in lib test |
| `precip_background_band_wired` | `weather_precip_show_background` true at strategic zoom in lib test |
| `minimap_wx_wash_wired` | Compositor applies cool wash above threshold — no particles |
| `minimap_wx_legend_wired` | Legend includes `Wx wash = mean precip` when wash active |
| `weather_sim_live_maintained` | `weather_sim_live.json` `green` unchanged or true |
| `acceptance_player_read_at_glance` | lib test or harness: non-zero rain → WX `r > 0` + overlay α > 0 |
| **`green`** | all **P0** keys true && `weather_sim_live_maintained` && `acceptance_player_read_at_glance` |

---

## §Implementation — coder handoff (@coder C)

```text
DESIGN-WX-HUD-IMPL-001 — implement per docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md §Implementation

Read:  design_weather_player_read_v1.md (§1–7 + §Implementation)
       plan_weather_parallel_lane_v1.md — cross_system_max_consumer_files_per_pr = 1
       debug_runs/weather_sim_live.json (maintain green)

Touch (presentation only):
  src/gui/hud/simulation_shell_phase2.rs   — WX + PWR bindings, format helpers
  src/systems/weather/weather_visual.rs      — verify bands (likely no change)
  src/gui/hud/hud_async_task_queue.rs        — minimap legend suffix
  minimap compositor path                  — wx wash tint (one consumer file outside weather/)

Do:
  P0 witness keys in debug_runs/weather_hud_player_read_live.json
  cargo test -p proc_A_dine01 --lib weather

Do NOT:
  src/construction/ execute funnel
  ChunkWeather in render extract
  Tile / MCP coupling
  F3-only weather read

Verify:
  cargo test -p proc_A_dine01 --lib weather
  weather_sim_live.json green preserved

Witness: debug_runs/weather_hud_player_read_live.json
```

**ΔWF→@coder C:** bindings in §Implementation table — confirm `VIS low` + PWR derate + minimap legend land in **one or two PRs** (respect cross-system file cap).

---

## Sign-off

```text
DESIGN-WEATHER-PLAYER-READ-001 complete
Verdict: PASS
Doc: docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md

DESIGN-WX-HUD-IMPL-001 designer spec complete
Verdict: PASS (§Implementation)
ΔWF→@coder C — bindings clear?
Witness target: debug_runs/weather_hud_player_read_live.json
```

**Consumer:** Coder C — `weather_visual.rs`, `simulation_shell_phase2.rs` WX/PWR extensions, minimap compositor wash + legend.
