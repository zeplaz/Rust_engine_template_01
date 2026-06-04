# Construction economy + organic growth vision `v1`

| Field | Value |
|:---|:---|
| **ID** | **CONSTRUCTION-ECON-GROWTH-001** |
| **Parent** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| **Exec (growth)** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) |
| **Exec (assembly)** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) |
| **Industrial spine** | [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) |
| **Date** | 2026-06-02 |

---

## Player fantasy (one paragraph)

You build **state capacity** — factories, utilities, corridors, civic anchors. Zoning and infrastructure make districts **legible**. Over time **private and tertiary** actors propose shops, housing, warehouses, and services that **fill niches**, add supply-chain depth, and **compete** for foot traffic and labor — not as 500 JSON buildings, but as **archetype-driven** infill through the same construction pipeline you use.

---

## Actor layers (who builds what)

| Layer | Who commits | Examples | Pipeline |
|:---|:---|:---|:---|
| **State / player** | Player execute | Factory, power, water, rail, admin, military | Zone tool + parametric commit → **CON-P2** stages |
| **Growth (sim)** | Approve / auto policy | Corner shop, apartments, warehouse, school | **OG-2** proposal → `ConstructionPlanQueue` |
| **Market pressure** | Sim scoring only | Competing commercial, redundant services | **OG-1** metrics → proposal priority / suppress |
| **Catalog legacy** | Migration alias | Existing `BuildingDefinition` rows | PG-1 `catalog_id` alias until retired |

**Invariant:** No layer spawns `Operational` on commit. State factories and growth proposals both → **Planned** → tick → **Operational**.

---

## Metrics that drive infill (OG-1 extension)

| Signal | Drives | Compete / niche |
|:---|:---|:---|
| `employment_demand` | Commercial / industrial proposals | Too many shops → lower priority |
| `housing_deficit` | Residential infill | Competes with industrial zoning |
| `freight_access` | Warehouse, logistics | Tied to **INFRA-E1** graph (gated) |
| `utility_service` | All usages | State-built power/water unlocks growth |
| `civic_policy` | Schools, clinics | Government archetypes |
| `market_saturation` | Suppress duplicate commercial | Same archetype in district cap |

**v1 stubs OK:** boolean service flags + scalar pressure; full economy solver is Phase 7 logistics alignment.

---

## Sequence (back to implementation)

```text
1. CON-P2-001..003     staged sites (state + player builds)
2. PG-1                BuildingArchetype + Usage (State/Industrial/Residential/…)
3. INFRA E0–E2         graph + utilities (unlocks freight + service metrics)
4. PLAN-SETTLEMENT-005 Town/District/Block books
5. OG-1                pressure + saturation
6. OG-2                proposals → queue (private infill)
7. PG-2 + MCP lod0     greybox modules for assembly
8. OG-3                approve UI (designer UX PASS)
9. Phase 7             logistics throughput on graph (supply competition)
```

**MCP art** runs **parallel** at step 7 — does not block steps 1–6.

---

## Archetype ↔ usage (PG-1 seed set)

| Usage | State / growth | Notes |
|:---|:---|:---|
| `Government` | Player | Civic anchors |
| `Industrial` | Player + growth | Factories (state), workshops (growth) |
| `Commercial` | Growth | Shops, services — **market saturation** |
| `Residential` | Growth | Housing infill |
| `Office` | Growth | Employment sinks |
| `Military` | Player | Policy-gated |

Style packs differentiate **Soviet heavy / Western industrial / Victorian residential** — same grammar, different modules ([`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md)).

---

## Planner slices still needed

| ID | Delivers |
|:---|:---|
| **PLAN-SETTLEMENT-HIERARCHY-005** | Town/District/Block schema + RON |
| **PLAN-ECON-GROWTH-ACTORS-001** | [`plan_econ_growth_actors_exec_001_v1.md`](plan_econ_growth_actors_exec_001_v1.md) **SIGNED** |
| **PLAN-CONSTRUCTION-SCALING-AUDIT-003** | P3 witness (designer scaling PASS paired) |

---

## Witness targets (future)

| Key | Meaning |
|:---|:---|
| `construction_site_stage_pipeline_001.green` | P2 closed |
| `construction_procedural_build_001.green` | PG-1/2 |
| `construction_organic_growth_001.green` | OG-2 queue, no instant spawn |
| `growth_market_saturation_active` | duplicate commercial suppressed |

---

## Anti-patterns

- 500 per-building JSON as content strategy
- Growth spawning operational entities
- MCP greybox promoted as `production` without validation contract
- Duplicate `Town` type in infra vs construction (G-TOWN-ONE)

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | State vs private infill + market niche framing |
