# FIRE-ENTITY-1 — Site / unit / truck fire exposure

| Field | Value |
|:---|:---|
| **Id** | **FIRE-ENTITY-1** |
| **Status** | **SHIPPED** |
| **Parent** | [`plan_weather_wind_v1.md`](plan_weather_wind_v1.md) W2 · construction demolish funnel |

## Goal

Wildland `ChunkFireOverlay` heat damages **construction sites** (staged), **formations**, and **order-sim trucks** by susceptibility — collapse reuses demolish execute; presentation reads burn stages only.

## Exit

1. `SiteFireState` stages: Clear → Threatened → Disrupted → Deteriorating → Collapsing → Gone.
2. Collapse enqueues `execute_demolish_at_tile` (no second despawn path).
3. Formations take fire attrition via resolve (sole strength/morale writer).
4. Order-sim vehicles: delay → cargo loss → fleet remove under heat.
5. Footprint burn wash from `SiteFireState` (PT-5 Option B).
6. Dump series + lib tests; `validate_cargo_report` 0 errors.

## Authority

```text
ChunkFireOverlay → SiteFireState | FormationFireExposure | vehicle fire hits
  → consumers derate / attrit / delay
  → collapse → demolish funnel
  → presentation reads stages
```
