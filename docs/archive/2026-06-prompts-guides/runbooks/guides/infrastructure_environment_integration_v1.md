# Infrastructure, power & building environment integration `v1`

> **STATUS:** Draft **v1** — design scaffold. Implementation **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

---

## 1. Purpose

Connect **buildings** and **power/infrastructure** to **environment** (weather, terrain-derived stress, maintenance) so **efficiency** and **failures** emerge from simulation — not isolated prefab stats.

---

## 2. Current problem

- Buildings tend to be **isolated** from weather and terrain dynamics.  
- Power models are often **oversimplified** (single “weather dependent” flag).  
- Missing **grid stress**, **export limits**, and **cascading failure** semantics.

---

## 3. Target model

```
Environment (weather + terrain interpretation + overlays)
  → Infrastructure stress (grid, pipes, rails, roads)
  → Building efficiency / availability
  → Economic throughput
```

---

## 4. Example: solar plant inputs

Solar output should depend on **multiple simulated factors**, for example:

| Factor | Influence |
|:---|:---|
| Cloud cover | Generation |
| Dust accumulation | Efficiency loss |
| Snow coverage | Severe reduction |
| Maintenance quality | Degradation / recovery |
| Grid stability | Export efficiency |
| Heat | Panel degradation |

Implementation binds to **chunk/regional weather** and **overlay/derived** fields, not a boolean flag alone.

---

## 5. Power system responsibilities

| System | Purpose |
|:---|:---|
| Grid topology | Routing, partitions |
| Load balancing | Shortage response |
| Infrastructure stress | Failures, brownouts |
| Weather impacts | Output and mechanical stress |
| Fuel logistics | Non-electrical plants |
| Reserves / stabilization | Storage, peaking |

---

## 6. Building environmental interface (sketch)

Design-time or runtime profile (names indicative):

```rust
pub struct EnvironmentalResponseProfile {
    pub temperature_tolerance: f32,
    pub flood_resistance: f32,
    pub wind_resistance: f32,
    pub moisture_sensitivity: f32,
    pub dust_sensitivity: f32,
}
```

**Systems** apply weather + environment to these profiles; assets supply parameters.

---

## 7. Cross-links

- [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md)  
- [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md)  
- Production/power matrices: `prompts/matrix/production/` and designer specs under `production_economy/`  
- UI boundary: [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md)

---

## 8. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md)
