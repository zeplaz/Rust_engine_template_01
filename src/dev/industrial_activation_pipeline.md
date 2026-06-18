# Industrial activation pipeline

> **STATUS:** Active — **Phase 4** after `CONSTRUCTION_OPERATIONAL_GREEN`.  
> **Prerequisite:** Construction spine green ([`construction_operational_gate.md`](construction_operational_gate.md)).  
> **Assessment source:** [`recovery_construction.md`](recovery_construction.md) § Current real architecture (line 1323+).

---

## North star

JSON building assets and ECS production types must drive **live** simulation — not flavor text.

```text
tool → intent → preview → validation → queue → execute → site lifecycle
                                                      ↓
                                            commissioning / provisioning
                                                      ↓
                                            IndustrialActivationBridge
                                                      ↓
                         production ECS + grid + logistics + resource graph
```

Construction proves **authority**. Industrial activation proves **the world comes alive**.

---

## Three-layer maturity (honest)

| Layer | State | Location |
|-------|--------|----------|
| **L1 Construction** | Mature | `src/construction/` — toolbox, catalog, commit, proof |
| **L2 Industrial definitions** | Advanced prototype | `assets/configs/buildings/*.json`, `src/entities/production/{concrete,aluminum,power}/` |
| **L3 Live economic sim** | Early stub | Missing cross-system lifecycle |

The gap is **not** “rewrite industry in one file”. It is **integration spines**:

- construction → operational facility  
- resource registration / throughput  
- grid + logistics membership  

---

## Required lifecycle (canonical)

| Stage | Strategic phase | What happens |
|-------|-----------------|--------------|
| **Place** | `Planned` … `UnderConstruction` | `CommitConstructionSiteEvent`; `BuildingDefinitionRef` stored on site entity |
| **Build** | Survey → Foundation → UnderConstruction | `SiteConstructionBook` progression ([`advance_early_construction_phases_system`](../construction/history.rs) + site systems) |
| **Commission** | `Provisioning` | Networks + manifest delivery; `SiteOperationalStats` ratios |
| **Activate** | `Operational` | **`activate_industrial_facilities_system`** spawns production bundle |
| **Register** | post-activate | Resource flow node, grid load, logistics node (phased) |

Today: commit stops at `ConstructionSite` + phases; production plugins tick **only if** runtime components were spawned manually.

---

## Priority 1 — `IndustrialActivationBridge` (highest leverage)

**Module:** `src/economy/activation/`  
**Plugin:** `IndustrialActivationPlugin`  
**System:** `activate_industrial_facilities_system`

**Trigger:** `ConstructionSite.phase == Operational` and entity has `BuildingDefinitionRef` and lacks `IndustrialFacilityActivated`.

**Supply-chain authority:** `assets/configs/industrial_supply_chains.json` + per-step JSON under `assets/configs/buildings/`. Roles in `src/construction/supply_chain_role.rs`; activation in `src/economy/supply_chain.rs`.

**Visual grammar binding (2026-06):** [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) — join building grammars + site zones + catalog power/IO in APS iterate loop.

**Power line construction UX (2026-06):** [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md) — curved/90° draw · voltage types · islanding/repair read · [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md).

**Role → ECS (one building per step — do not collapse chains):**

| `supply_chain_role` | Example `catalog_id` | Runtime components | Typical power |
|---------------------|----------------------|--------------------|---------------|
| `aggregate_mine` | `concrete_aggregate_mine` | `AggregateMineRuntime` | 18 |
| `cement_kiln` | `concrete_cement_kiln` | `CementKilnRuntime` | 72 |
| `concrete_mixer` | `concrete_mixer_plant` | `ConcreteMixerRuntime` | 28 |
| `integrated_plant` | `concrete_basic_production_plant` | kiln + mixer (legacy monolith) | 50 |
| `bauxite_mine` | `aluminum_bauxite_mine` | `BauxiteMineRuntime` | 22 |
| `alumina_refinery` | `aluminum_alumina_refinery` | `AluminaRefineryRuntime` | 85 |
| `aluminum_smelter` | `aluminum_smelter1` | `AluminumSmelterRuntime` | 200 |
| `aluminum_fabrication` | `aluminum_fabrication_plant` | `AluminumFabricationPlantRuntime` | 48 |
| `builtin:*` | — | skip | — |
| power plant | utilities catalog | future: `plant_definitions.json` | — |

**Power draw:** `BuildingDefinition.power_consumption` → `ElectricalComponent` via `electrical_from_power_units` (designer units / 100 = base load until grid MW alignment).

**Marker:** `IndustrialFacilityActivated` — idempotent; witness + tests.

**Schedule:** run **after** [`site_provisioning_system`](../strategic/site/provisioning.rs) in `InfrastructureSiteSet`.

---

## Priority 2 — Resource flow registry

**Goal:** authoritative “who produces / consumes what” graph — not global magic counters.

```rust
// Target shape (economy/resource_flow.rs — not yet implemented)
pub struct ResourceFlowNode {
    pub inputs: Vec<ResourceRate>,
    pub outputs: Vec<ResourceRate>,
}
```

**Inputs:** `BuildingDefinition.produces` / `consumes` string tags from JSON (Concrete, Aluminum, Electricity, Labour, …).  
**Link:** attach to site entity at activation; sync with [`ResourceType`](../entities/production/core/production_utils.rs) where names match.

**Todo board:** `INDUSTRIAL-I2-*` in [`industrial_activation_todos.rs`](industrial_activation_todos.rs).

---

## Priority 3 — Grid stress gameplay

**Existing:** `PowerRuntimePlugin`, `ElectricalGrid`, `TransformerComponent`, `rebuild_electrical_grid_topology`, overload events.

**Activation must:** insert `ElectricalComponent` on industrial buildings so grid rebuild associates loads within `GridConnectionRadiusSq` (~48 m).

**IND-E03 witness (done):** [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) (**PLAN-IND-E03-001** / **IND-E03-CODER-A**) — `GridOverloadEvent` → `ind_e03_green` in `industrial_activation_live.json`.

**Gameplay target:** smelter +220 MW equivalent → transformer stress → brownouts (design in [`power_damage_ui_persistence_v1.md`](../../prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md)).

---

## Priority 4 — Physical logistics

Concrete: batching, move cost, expiry (see [`concrete_industry_sim_runbook_v1.md`](../../prompts/guides/concrete_industry_sim_runbook_v1.md)).  
Aluminum: bauxite → alumina → smelter chain tied to [`LogisticsGraph`](../../strategic/mod.rs) — not inventory UI fiction.

---

## Data already in repo (use it)

| Domain | Assets | Runtime |
|--------|--------|---------|
| Concrete | `concrete_*_production_plant.json` | `ConcreteProductionConfig`, kiln/mine/mixer |
| Aluminum | `aluminum_smelter1.json` | `AluminumProductionConfig`, mine/refinery/smelter/fabrication |
| Power plants | `assets/config/power/plant_definitions.json` | `PowerPlant`, defs registry, output systems |
| Distribution | asset editor substation/transformer tabs | `TransformerComponent`, `SubstationComponent`, grid topology |

**Warning:** do not let JSON become disconnected flavor — every field should eventually drive runtime or validation.

---

## Exit gate — `INDUSTRIAL_ACTIVATION_GREEN`

Separate from Stage 5 and construction operational green.

| # | Requirement | Todo id |
|---|-------------|---------|
| 1 | `catalog_id` on commit path (event → `PlannedSite` / `BuildingDefinitionRef`) | `INDUSTRIAL-I1-01` |
| 2 | Activation on `Operational` for concrete + aluminum JSON ids | `INDUSTRIAL-I1-02` |
| 3 | `ElectricalComponent` load from def; appears in grid totals | `INDUSTRIAL-I1-03` |
| 4 | Unit test: phase Operational + ref → components present | `INDUSTRIAL-I1-04` |
| 5 | Proof JSON or witness row in `debug_runs/industrial_activation_live.json` | `INDUSTRIAL-I1-05` |
| 6 | Resource flow registry stub resource | `INDUSTRIAL-I2-01` |
| 7 | Power plant activation from utilities pick (definition_id) | `INDUSTRIAL-I1-06` |

Registry: [`industrial_activation_todos.rs`](industrial_activation_todos.rs) (**31 rows**).  
Human spec (assessment §1811+): [`industrial_activation_phase_todos.md`](industrial_activation_phase_todos.md).

---

## Work order

1. **SC** — Supply-chain granularity (placeable steps, role activation) — **mostly done**.  
2. **I1** — Close proof JSON + power-plant activation.  
3. **I2** — Resource flow graph + inventory + starvation cascade.  
4. **I3** — Grid stress + **placeable** transformers/substations.  
5. **I4** — Logistics physical movement + spatial districts.  
6. **GOV** — Anti mega-factory collapse invariant.

---

## Related docs

- [`recovery_construction.md`](recovery_construction.md) — construction + Round 3 complete; architecture assessment §1323+  
- [`construction_invariants.md`](construction_invariants.md) — placement authority (unchanged)  
- [`construction_active_progress.md`](construction_active_progress.md) — construction lane status  
- [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) — do not collapse with Stage 5  
