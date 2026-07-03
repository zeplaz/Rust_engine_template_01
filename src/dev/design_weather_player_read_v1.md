# Weather — player readability charter `v1` (DESIGN-WEATHER-PLAYER-READ-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-WEATHER-PLAYER-READ-001** · impl slice **DESIGN-WX-HUD-IMPL-001** |
| **Canonical archive** | [`docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md) |
| **Date** | 2026-06-03 |
| **Owner** | `@designer` (charter) · `@coder C` (wire) |
| **Verdict** | **PASS** (charter + §Implementation spec) |
| **Impl witness** | [`debug_runs/weather_hud_player_read_live.json`](../debug_runs/weather_hud_player_read_live.json) |

```text
DESIGN-WEATHER-PLAYER-READ-001 Q✓
Player reads precip/fog/smoke/contamination without F3
```

---

## Mission (summary)

Players name **rain vs snow vs fog vs clear** at tactical/strategic zoom and distinguish weather from **fire smoke** and **WSS contamination** via ops strip + overlay — not F3 diagnostics.

**Acceptance:** At tactical zoom, player names active precip and visibility reduction without opening diagnostics.

---

## Authority (presentation)

| Source | Consumer |
|:---|:---|
| `WeatherPrecipVisualSample` | Ops strip WX zone |
| `ClimateVisualAggregate` | Overlay tint, minimap wash |
| Fire extract | ALERTS channel — never WX line |
| F3 | Engineer-only toggles |

---

## Key copy (locked)

| Surface | Format |
|:---|:---|
| WX ops | `WX  r {rain:.2}  s {snow:.2}  f {fog:.2}` |
| VIS suffix | append `VIS low` when visibility `< 0.85` |
| PWR derate | `PWR  {pct}%  (wind)` or `(solar)` when factor out of band |
| Minimap legend | `Wx wash = mean precip` when wash active |

Full band tables, hue separation (fog cool / smoke warm / contamination patterned), and widget tree: see **canonical archive** above.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-03 |

**Consumer:** Coder C — `simulation_shell_phase2.rs`, minimap compositor wash, `weather_hud_player_read_live.json` P0 keys.
