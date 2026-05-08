# Weather simulation runbook `v1`

> **STATUS:** Draft **v1** — design scaffold. Implementation **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

---

## 1. Purpose

Replace **visual-only** weather with simulation that connects **terrain**, **infrastructure**, **logistics**, **buildings**, **agents**, and **economy** — via **derived metrics and overlays**, not by editing base terrain ontology.

---

## 2. Current problem

- Weather is mostly **cosmetic** or stubbed.  
- No consistent coupling to **traction**, **throughput**, **power output**, **combat visibility**, or **agriculture**.

---

## 3. Target model (dataflow)

```
Climate
  → Regional weather cells
  → Chunk weather state
  → Terrain interaction (overlays / derived fields)
  → Infrastructure effects
  → Logistics effects
  → Economic effects
  → Agent behavior
```

---

## 4. Weather state model

### 4.1 Global climate layer (slow)

- Seasons, temperature bands, humidity circulation, oceanic effects.

### 4.2 Regional weather layer (medium)

- Storms, droughts, cold fronts, rainfall systems, snow bands.

### 4.3 Chunk weather layer (fast)

- Local rain, fog, mud accumulation, snow depth, lightning risk, visibility modifiers.

---

## 5. Effect categories (must map to systems)

| Category | Examples |
|:---|:---|
| Hydrology | Flooding, erosion pressure |
| Mobility | Traction, stuck risk |
| Logistics | Throughput loss, delay |
| Power | Solar/wind efficiency |
| Combat | Visibility, morale modifiers |
| Agriculture | Crop growth / failure |
| Buildings | Degradation, shutdown risk |
| Fires | Spread chance |
| Disease | Population pressure (if applicable) |

---

## 6. Recommended module layout (future code)

```
systems/weather/
  climate/
  regional_cells/
  chunk_weather/
  weather_events/
  weather_effects/
  weather_persistence/
```

Naming is indicative; align with actual crate layout when implementing.

---

## 7. Critical design rule

**Weather never directly edits terrain ontology** (immutable base classification / materials registry meaning).  

**Allowed path:** weather → **dynamic overlays** or **derived chunk fields** → mobility / power / AI **interpretation**.

---

## 8. Cross-links

- [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md) — when chunk weather updates run.  
- [`infrastructure_environment_integration_v1.md`](infrastructure_environment_integration_v1.md) — grid and building response.  
- [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) — chunk fields and material/mobility chain.  
- Dynamic overlay patterns: `DynamicTerrainOverlay` / stubs in engine (extend consistently).

---

## 9. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md) · **S2:** [`../matrix/simulation_expansion/runbook/s2_steps_v1.md`](../matrix/simulation_expansion/runbook/s2_steps_v1.md)
