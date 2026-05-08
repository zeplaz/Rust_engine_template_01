# Flora & ecology simulation runbook `v1`

> **STATUS:** Draft **v1** — design scaffold. Implementation **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

---

## 1. Purpose

Move trees / vegetation from **static decoration** to **ecology-coupled simulation**: terrain interaction, weather/hydrology coupling, fire, extraction economy — at **chunk scale** with **LOD detail** for local views.

---

## 2. Current problem

- Trees lack ecology succession, environmental feedback, and economy depth.  
- A single-entity-per-tree approach does not scale.

---

## 3. Target model

```
Terrain family / ecology profile
  → Vegetation succession state (fields)
  → Growth / death / spread
  → Environmental interaction (soil moisture, erosion, fire risk)
```

---

## 4. Scale rule (LOD)

| Scale | Representation |
|:---|:---|
| **Strategic / chunk** | Ecology **fields**: biomass density, species mix overlays, fire risk, regrowth. |
| **Local detail** | Full tree ECS only near **player**, **construction**, **combat**, **harvest**. |

**Truth** for forests lives primarily in **fields**, not in per-tree entities globally.

---

## 5. Key field variables (chunk / overlay)

| Variable | Purpose |
|:---|:---|
| `biomass` | Vegetation density |
| `moisture_need` | Drought resistance (species profile) |
| `root_strength` | Erosion stability coupling |
| `fire_risk` | Ignition / spread input |
| `regrowth_rate` | Recovery after disturbance |
| `shade_factor` | Local cooling / microclimate |
| `harvest_value` | Economy link |
| `disease_resistance` | Ecology events |

Exact names are **data-design**; keep stable once sims depend on them.

---

## 6. Ecology loop (integration)

```
weather
  → soil moisture (derived)
  → vegetation growth
  → erosion stability
  → hydrology feedback
```

---

## 7. Cross-links

- [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md) — moisture and storm forcing.  
- [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md) — when ecology fields tick.  
- Terrain / materials / tags: [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md).

---

## 8. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md) until a dedicated ecology matrix exists.
