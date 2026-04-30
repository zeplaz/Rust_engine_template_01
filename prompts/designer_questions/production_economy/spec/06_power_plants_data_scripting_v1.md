# Power plants: data-driven archetypes, research, and scripting (v1)

**Goal:** Cover **all** plant kinds and **operational statuses** (standby, maintenance, environmental shutdown, etc.) while keeping **specifics** (hydro head/flow, nuclear unit count, solar array + panel tech) **designer-editable** and **tool-friendly**. Rust owns **ECS shape**, **deterministic hot paths**, and **grid coupling**; **numbers, curves, gates, and infrequent rules** live in **data** (and optionally a **small script layer**).

**Related code:** `src/entities/production/power/*`, `OperationalStatus` in `src/entities/types/e_flagz.rs`, manifest: `src/systems/production/manifest.rs`.

---

## 1. What must *not* stay hardcoded

| Concern | Direction |
|:---|:---|
| Plant categories, efficiency curves, derate coefficients | **Asset configs** (JSON/RON), keyed by `plant_archetype_id` + optional `panel_tech_id`, `hydro_crest_id`, etc. |
| Research unlocks / branches | **Data**: tech id → list of allowed archetype params, max MW caps, new curve ids |
| Hydro “depends on river” | **Instance** links: `river_reach_id`, `head_m`, `installed_capacity_mw`; **live** flow from hydrology/worldgen or save snapshot; **curve** flow → output from config |
| Nuclear “reactors + size” | **Instance**: `reactor_unit_count`, `rated_thermal_mw_per_unit`, fuel cycle **tech id**; caps/modifiers from research row in data |
| Solar “field + panels” | **Instance**: `array_area_m2`, `panel_tech_id`, `tilt_deg`, optional shading; **panel tech** row = efficiency vs irradiance curve id |
| Operational transitions | **Per-entity** `PowerPlant.status` + optional **data-driven FSM table** (from → event → to) for tools; heavy logic stays rare |

Rust keeps: component definitions, queries, `ElectricalGrid` topology, ordering in `PowerRuntimePlugin`, and **one internal API** like `sample_output_curve(plant_instance, external_inputs) -> f32` that **only reads** loaded tables + instance fields.

---

## 2. JSON vs Lua vs “something else”

### 2.1 JSON (or RON) — **default for definitions**

**Use for:** archetypes, curves as sampled arrays, research gates, string ids, defaults for tools.

**Pros:** trivial to edit in VS Code / custom tools, `serde` validation, hot-reload with Bevy assets, diff-friendly, no sandbox problem.

**Cons:** not a programming language — no arbitrary per-tick code unless you add a **separate** expression layer (below).

**Fit:** **Most** of what you described (dam vs run-of-river, panel types, reactor count) is **structured parameters + curves**, not general scripts.

### 2.2 Lua

**Use for:** **optional modding** or **rare** complex rules (e.g. custom economic dispatch) where designers write real functions.

**Pros:** familiar to many, large ecosystem.

**Cons:** FFI boundary, sandboxing, embedding (`mlua` etc.), harder static validation, **easy to hurt performance** if called every tick on thousands of entities.

**Guideline:** If Lua is added, run it **low frequency** (on build/upgrade, policy change, or chunked every *N* sim ticks), **never** per-entity per-frame unless heavily batched and capped.

### 2.3 Rhai (or tiny expression DSL)

**Use for:** **short formulas** in strings, evaluated with a **fixed** variable map (`head`, `flow`, `efficiency`, …) bound from Rust.

**Pros:** Rust-native, sandboxed, safer than Lua for small expressions.

**Cons:** still a dependency and a skill curve for designers; debug story needs tooling.

**Guideline:** Prefer **JSON curve ids** first; add Rhai only where “one formula per archetype” beats maintaining huge curve tables.

### 2.4 WASM / plug-in (future)

For **maximum performance** with custom logic: compile user logic to WASM, run in pooled workers. High engineering cost — only if mods demand **near-native** speed.

---

## 3. Recommended stack for *this* engine template

1. **`assets/config/power/`** (or similar):  
   - `plant_archetypes.json` — family tags (steam, nuclear, renewable…), default capability markers, curve id references.  
   - `curves/` — e.g. `head_flow_to_power.json` (piecewise or LUT).  
   - `research/power_tech.json` — unlocks, multipliers, allowed `panel_tech_id` sets.

2. **Bevy `Asset` types** mirroring those files; `PlantArchetype` in Rust becomes **“runtime handle + cached floats”** loaded from asset id, not a giant `match` on enum only.

3. **Instances** (save / ECS):  
   - `PowerPlant` + **small components** or **blob component** for instance params (`HydroInstance`, `SolarInstance`, …) *or* a single `PowerPlantParams` map keyed by field name if tools generate it — pick one pattern and serialize it in snapshots.

4. **Operational status:** keep **enum in Rust** for type safety; **allowed transitions** and **repair durations** can live in JSON keyed by status + damage type (ties into `04_power_damage_repair.md`).

5. **Lua:** defer until there is a concrete modding requirement; document that **JSON + optional Rhai** is the first escalation path.

---

## 4. Tooling / performance

- **Tools** edit JSON (and later visualize curves); **game** loads assets once; Rust uses **LUT interpolation** for hot paths.  
- **Validation:** CI step: `serde` parse all configs + golden tests for monotonic curves.  
- **Max performance:** no script in the inner loop; batch **external inputs** (weather, river) into **chunk resources** updated on a coarser schedule, then multiply per plant from cached coefficients.

---

## 5. Follow-ups (implementation)

- [ ] Add `PowerPlantDefinition` / `PlantArchetypeId` asset and loader; replace enum-only `PlantArchetype::for_type` with **id → loaded row** (enum can remain for backward compat migration).  
- [ ] Extend manifest / serialization matrix for new DTO fields.  
- [ ] Production tools UI: pick archetype, edit instance params, assign research id.  
- [ ] Revisit **Lua** only after modding spec in `prompts/matrix` or designer_questions explicitly requires it.
