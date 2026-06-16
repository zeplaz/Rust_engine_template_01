# DESIGN-WEATHER-PLAYER-READ-001 — Designer brief (weather presentation)

**Program:** [`plan_weather_parallel_lane_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_weather_parallel_lane_v1.md)  
**Runbook:** [`weather_simulation_runbook_v2_plan_v1.md`](../../docs/archive/2026-06-src-dev/plans/weather_simulation_runbook_v2_plan_v1.md)  
**Owner:** `@designer`  
**Parallel with:** APS-UX-AUDIT-001 lead sign-off (does not block Coder C witness)

---

## Mission

Charter how **players read weather** in simulation — presentation tier only. Coder C owns `src/systems/weather/` sim; you deliver a design doc only.

**Not in scope:** sim tick math, clipmap writers, tile variants, APS atlas, transport graph UI.

---

## Deliverable

**File:** `docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md`

### Cover

1. **Precip overlay** — rain/snow active vs mesh fallback; tactical vs strategic map
2. **Fog / visibility** — distinct from fire/smoke tint; readability at minimap scale
3. **Wind / solar hint** — should HUD show `GlobalRenewableWeatherFactors` bands (0.05–1.2)?
4. **Terrain wetness** — subtle default; gameplay-readable when witness green
5. **Construction** — future `weather_penalty` badge (hidden until scalar ≠ 1.0); no execute funnel UX
6. **Accessibility** — status not color/glyph alone; consistent legends
7. **Non-goals** — tile weather frames, MCP/APS, graph edits

### Acceptance

- ≤2 pages · PASS / PASS WITH NOTES / FAIL verdict: *player can tell weather state at a glance*
- Sign-off: `tools/orchestrator/queues/designer_signoff_registry.json` → `DESIGN-WEATHER-PLAYER-READ-001`
- Unblocks UX review for **WEATHER-GPU-PRECIP-001** (later)

---

## Paste back to orchestrator

```text
DESIGN-WEATHER-PLAYER-READ-001 complete
Verdict: PASS | PASS WITH NOTES | FAIL
Doc: docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md
```
