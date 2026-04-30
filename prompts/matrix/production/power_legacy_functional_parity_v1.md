# Legacy `power_systems.rs` — functional parity & accountability (v1)

**Purpose:** Every **legacy system**, **grid behavior**, and **operational × plant** concept from the old module is **listed here** with: **current support**, **gap**, and **what must be built** (code, data, or tools). This is **not** a code-structure map — it is **behavior and terms** so nothing is “just deleted” without a trail.

**Canonical implementation today:** `src/entities/production/power/*`, `PlantDefinition` JSON: `assets/config/power/plant_definitions.json`, registry: `PlantDefinitionRegistry`.

**Legacy file path (comment-only pointer):** `src/systems/production/power_systems.rs`

---

## 1. Legacy scheduling (`PowerSysPlugin`)

| Legacy behavior | Parity status | Notes |
|:---|:---|:---|
| Run only when `BaseState::Simulation` **and** `SimulationState::Running` | **Partial** | `PowerRuntimePlugin` uses **`SimControlState::should_tick()`** (Diagnostics pause/step — see `SimControlPlugin`). Matches playable loop today. **Tighter parity:** add `init_state::<BaseState>()` + `SimulationState` on the app and extend `run_if` per §1 notes below. |
| Ordered **before** `ProductionPlugin::building_activity_system` | **Gap** | No explicit ordering vs manufacturing. **Support needed:** `before/after` on shared system sets once production tick order is fixed in `systems/production/runtime.rs`. |

---

## 2. Legacy grid / electrical systems

| Legacy symbol | Intended behavior | Parity status | Support needed |
|:---|:---|:---|:---|
| `manage_electrical_grids_system` | Clear grids; for each transformer/line, attach nearby buildings with **load** (`ElectricalComponent` on building); accumulate load/capacity; insert `ElectricalGrid` on host if missing | **Partial** | `rebuild_electrical_grid_topology` rebuilds from **all** buildings with `Building` + `ElectricalComponent` in radius. Legacy required `load_option` (building had optional load). **Support:** optional rule “member must have consumptive load” via definition flag or `ConsumptionComponent` filter. |
| `update_grid_totals_system` | Recompute `total_load` / `total_capacity` from `grid.members` + `ElectricalComponent` | **Partial** | `recalculate_grid_totals_from_members` (added) runs after membership; keeps totals consistent when **only** loads change. |
| `update_grid_system` | Mixed transformer + line + `ElectricalLoad` + `ElectricalCapacity` on **same** entity, `connected_grids` transfer | **Gap** | Types `ElectricalLoad` / `ElectricalCapacity` as in legacy snippet are **not** in active ECS. **Support:** model A/C buses + inter-tie as components + a dedicated **multi-bus** system, or document as **superseded** by `ElectricalComponent` + future graph. |
| `update_on_power_infrastructure_destroyed` / `remove_from_grid_system` | On `RemovedComponents<ElectricalComponent>`, remove entity from all `grid.members` | **Implemented** | `purge_removed_entities_from_grids` removes stale member IDs. |
| `add_to_grid_system` | On `Added<ElectricalComponent>`, insert into grid within `grid.radius` | **Superseded** | Full **rebuild** each frame (or on interval) covers adds; **optimization:** event-driven incremental add later. |
| `check_for_overloads_system` | If `total_load > total_capacity`, log / alert | **Partial** | `emit_grid_overload_signals` sends **`GridOverloadEvent`** (Bevy message). **Support:** subscribe in UI / damage (`EmergencyType::SubstationOverload`) per `04_power_damage_repair.md`. |

---

## 3. Legacy `update_load_system`

| Legacy behavior | Parity status | Support needed |
|:---|:---|:---|
| Drive `ElectricalLoad.current_load` from `ProductionComponent` + `ConsumptionComponent` ratios and overload penalty | **Gap** | `ElectricalLoad` type not in graph; production uses `ManufacturingNode` / other paths. **Support:** bridge system `manufacturing_or_production_to_building_electrical_load` + tunable penalty coefficients in **plant / grid JSON**, not hardcoded. |

---

## 4. Legacy `power_plant_operational_system` (global `State<OperationalStatus>`)

| Issue | Parity |
|:---|:---|
| Used **`ResMut<State<OperationalStatus>>`** — **one** global state for **all** plants | **Rejected** | Wrong model. **Current:** `PowerPlant.status: OperationalStatus` **per entity**. All legacy branches must be reinterpreted as **per-plant** behavior driven by **data** (`PlantDefinition.operational`) + systems, not a cartesian `match status × plant_type` in code. |

---

## 5. Operational status × plant type — semantic checklist

Legacy had empty branches for **every** `OperationalStatus` × `PowerPlantType`. Below: **what should happen** (design intent) and **where it lives**.

Legend: **D** = `PlantDefinition` JSON (`operational.*`), **E** = ECS/systems, **T** = tools/UI.

| Status | Coal / Oil / Gas / Biomass | Nuclear | Hydro | Solar | Wind | Geothermal |
|:---|:---|:---|:---|:---|:---|:---|
| **Standby** | Ready; minimal aux load; fuel inventory static | Ready; decay heat / cooling **per definition**; may require `StandbyService` power | Head/flow from sim; output 0; spill optional **D** | Inverter idle; tracker parked **D** | Pitch/feather **D** | Circulation / brine idle **D** |
| **Operational** | Full **output_model** thermal; emissions hooks **D** | Thermal + containment systems; power cap = `instance.reactor_units × unit_rating` **D/E` | Power from **head × flow × efficiency** curves **D** | Irradiance × area × panel η **D** | Wind × swept area × Cp curve **D** | Resource temperature / binary cycle **D** |
| **Maintenance** | Output 0; crew slots **T** | Refuel outage / SCRAM maintenance windows **D** | Gate/turbine dewater **D** | Array isolation **D** | Climb-down service **D** | Heat exchanger service **D** |
| **OutOfFuel** | Trip; **Fuel** component **E** | Misnomer → map to cycle limit / no coolant? **D** (rename in tools) | Low pond / intake — often **EnvironmentalShutdown** instead **D** | N/A (map to **ReducedCapacity** dust/snow if needed) **D** | N/A | Brine chemistry / recharge **D** |
| **StartingUp** | Boiler ramp **D** | Critical path hours; power import **D** | Fill ramp / wicket gate **D** | MPPT soft-start **D** | Cut-in ramp **D** | Pressure stability **D** |
| **ShuttingDown** | Cool down **D** | SCRAM curve **D** | Load rejection / wicket **D** | Disconnect **D** | Brake / cut-out **D** | Gradual thermal ramp **D** |
| **Decommissioned** | Output 0; dismantling jobs **T** | Long tail; SAFSTOR/ENTOMB **D** | Dam safety regime **D** | Remediation **D** | Blade disposal **D** | Well abandonment **D** |
| **ExternalShutdown** | Grid / market / strike | Regulatory / grid | Spill / fish / flow mandate | Curtailment | Curtailment | Land-use / permit |
| **ReducedCapacity** | Slagging, fouling, partial mills **D** | Xenon / boron / partial rod **D** | Low flow / head band **D** | Soiling / partial inverter **D** | Wake / icing **D** | Scaling / partial ORC **D** |
| **OverCapacity** | Rare; stress **D** | Thermal limits **D** | Flood discharge (may not export) **D** | Tracker fault overshoot **D** | Overspeed protection **D** | Thermal overload **D** |
| **EnvironmentalShutdown** | Cooling water limits | Thermal pollution limits | Flood / drought / fish window | Snow / eclipse / hail | Cut-out wind / lightning | Seismic / thermal source drop |

**Accountability:** No Rust `match` is required to **enumerate** every cell; tools + `PlantDefinition.operational.status_modifiers` + capability markers (`SteamCycle`, `ContainmentBuilding`, `VariableRenewable`) supply **parameters**. Rust evaluates **curves and caps** from loaded defs.

---

## 6. PlantDefinition & JSON

| Deliverable | Path |
|:---|:---|
| Full schema & examples | `assets/config/power/plant_definitions.json` |
| Rust mirror + registry | `src/entities/production/power/plant_definition.rs`, `plant_registry.rs` |
| Spec (scripting strategy) | `prompts/designer_questions/production_economy/spec/06_power_plants_data_scripting_v1.md` |

---

## 7. Review cadence

When changing power behavior: update **this file** (gap rows), **JSON**, and **serialization matrix** if save format includes `definition_id` or instance params.
