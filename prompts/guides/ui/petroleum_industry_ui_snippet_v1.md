# Petroleum industry UI / UX snippet `v1`

> **STATUS:** Draft **v1** — UX + architecture rules. Implementation **Pending**.

Version: `v1.0.0`  
Simulation runbook: [`../petroleum_industry_simulation_runbook_v1.md`](../petroleum_industry_simulation_runbook_v1.md)

---

## 1. Purpose

Petroleum gameplay should feel like **industrial planning**, **strategic allocation**, **infrastructure balancing**, and **wartime logistics** — not a static recipe clicker.

Expose **priorities**, **allocations**, **reserve policy**, **export policy**, **doctrine**, and **warnings** with **predictive feedback**.

---

## 2. Core UI principle

**Player defines strategic intent. Simulation executes operational consequences.**

---

## 3. Refinery output allocation panel

| UI element | Purpose |
|:---|:---|
| Sliders | Output mix (% per fraction) |
| Lock toggles | Minimum reserves |
| Priority arrows | Wartime bias |
| Warnings | Predicted shortage |
| Throughput graphs | Bottlenecks |
| Storage bars | Inventories |

Example display:

```
Diesel     [=====|---] 55%
Jet fuel   [===|-----] 30%
Gasoline   [==|------] 15%
```

Changing sliders should **preview** military readiness, civilian stability, export revenue, shortages, logistics pressure (exact metrics TBD in sim spec).

---

## 4. Fuel priority controls

Checkboxes / modes (illustrative): prioritize military diesel, maintain aviation reserves, civilian-first, wartime rationing. These feed **policy resources**, not individual convoys.

---

## 5. Strategic reserve UI

Slider for target reserve level; optional **“never draw below X”** lock. Simulation handles **auto-balancing**, **rerouting**, **export throttling** from policy.

---

## 6. Export terminal UI

Controls: export quota, domestic-priority toggle, military emergency export cut, alliance routing preferences.

---

## 7. Power + weather warnings (examples)

- Storm reducing **offshore** extraction efficiency  
- Refinery **overload** risk  
- Pipeline **freeze** risk  
- Strategic reserve **critically low**

Bind to [`weather_simulation_runbook_v1.md`](../weather_simulation_runbook_v1.md) and infrastructure runbook.

---

## 8. Military fuel doctrine panel

Modes (illustrative): civilian growth, balanced, wartime mobilization, air superiority, naval projection — each remaps **refinery weights**, **logistics priorities**, **stockpile targets**, **AI dispatch** hints.

---

## 9. Debug / advanced overlays

Pipeline pressure, reserve heatmap, throughput stress, shortage map, refinery utilization.

---

## 10. UI architecture rules (mandatory)

**UI must not** directly mutate routing graphs, agent pathing internals, or low-level simulation state.

```
UI → policy / settings resources → simulation systems react
```

### Example policy resource

```rust
#[derive(Resource)]
pub struct PetroleumPolicySettings {
    pub diesel_priority: f32,
    pub jet_fuel_priority: f32,
    pub reserve_target: f32,
    pub export_priority: f32,
    pub wartime_mode: bool,
}
```

Exact fields are **design**; placement and naming follow [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md).

---

## 11. Runbook integration

| Doc | Role |
|:---|:---|
| [`gui_runbook_v1.md`](../gui_runbook_v1.md) | Panel standards |
| [`petroleum_industry_simulation_runbook_v1.md`](../petroleum_industry_simulation_runbook_v1.md) | Sim behavior |
| [`weather_simulation_runbook_v1.md`](../weather_simulation_runbook_v1.md) | Environmental penalties |
| [`chunk_scheduler_runbook_v1.md`](../chunk_scheduler_runbook_v1.md) | Large-scale updates |

---

## 12. UX target

Player identity: **energy minister**, **wartime logistics planner**, **industrial strategist** — not **per-tick crafting babysitter**.
