# VM-11 preview vs main semantic audit `v1`

| Field | Value |
|:---|:---|
| **ID** | **VM-11-PREVIEW-AUDIT** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T8 INFRA-PERF |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` (+ `@coder` witness) |
| **Authority** | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) TRIAGE-VM-11 |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `view_isolation` |
| **Handoff** | VM-11 coder isolation audit |
| **Verdict** | **PASS** |

```text
VM-11-PREVIEW-AUDIT Q✓
Preview vs main semantic contract — not readiness flags alone
```

---

## 0. Scope

Audit **what the player sees** when WorldGen/preview chrome is active vs **Simulation** main map — semantic parity, not just green readiness booleans.

**Out of scope:** multiview editor parity · replay ring · construction ghosts.

---

## 1. Surfaces compared

| Surface | Preview / WorldGen | Simulation main | Must match |
|:---|:---|:---|:---:|
| Map projection | `SimulationMap` authority | same | ✓ |
| Terrain tint | world preview raster | tile fallback / clipmap | semantic class |
| Fire overlay | optional preview chrome | `FireVisualFrame` extract | heat band |
| Vegetation read | preview ecology tint | minimap + map stamp | topology kind |
| Infrastructure | dashed preview only | committed overlay graph | N/A |
| HUD chrome | full tools visible | collapsed sim defaults | intentional |

---

## 2. Semantic checks (operator)

| # | Check | Pass if |
|:---:|:---|:---|
| M1 | Same world XY click → same tile index | ±0 tiles |
| M2 | River / coast mask color family | same legend token |
| M3 | Burn scar vs clean canopy | scar visible in both when fire witness on |
| M4 | Preview dismiss → sim map no double exposure | single raster authority |
| M5 | Minimap thumbnail ↔ main map N/S alignment | landmark same quadrant |

---

## 3. Witness fields (coder)

Extend `stage5_full_app_live.json`:

```json
{
  "vm11_preview_audit": {
    "projection_parity": true,
    "ecology_legend_parity": true,
    "fire_band_parity": true,
    "double_raster": false
  }
}
```

---

## 4. Failure taxonomy

| Code | Meaning | Route |
|:---|:---|:---|
| VM11-PROJ | projection drift | `@coder` viewport |
| VM11-ECO | ecology tint mismatch | `@coder B` veg |
| VM11-FIRE | fire band mismatch | `@coder A` fire |
| VM11-CHROME | preview chrome leaked into sim | `@designer` + HUD |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
