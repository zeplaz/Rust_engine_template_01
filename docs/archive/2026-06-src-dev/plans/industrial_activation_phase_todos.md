# Phase 4 — Industrial activation & infrastructure causality

> **Source assessment:** [`recovery_construction.md`](recovery_construction.md) from line **1811** (granularity restored, resource flow next).  
> **Live board:** [`industrial_activation_todos.rs`](industrial_activation_todos.rs) — `INDUSTRIAL_ACTIVATION_GREEN` when all rows reconcile **Done**.  
> **Pipeline:** [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md).

**Prerequisite:** `CONSTRUCTION_OPERATIONAL_GREEN`. **Not** Stage 5 FULL_APP.

---

## North star (from assessment)

The engine is moving toward **infrastructure causality simulation**:

- Geography, staging, and zoning matter because chains are **spatially placeable**
- Power asymmetry (mine 22 vs smelter 200) creates **grid strategy**
- Role-based activation (`supply_chain_role`) drives runtime, power, and future logistics
- **Do not** collapse chains back into “Advanced Industrial Complex” mega-buildings

---

## Maturity snapshot

| Area | Maturity | Registry block |
|------|----------|----------------|
| Construction authority | Mature | (construction lane — done) |
| Industrial entity ontology | Surprisingly mature | `INDUSTRIAL-SC-*` |
| Activation semantics | Improved (role → bundle) | `INDUSTRIAL-I1-*`, `INDUSTRIAL-SC-*` |
| Economic flow | Early | `INDUSTRIAL-I2-*` |
| Power distribution | Medium | `INDUSTRIAL-I3-*` |
| Strategic logistics | Conceptual | `INDUSTRIAL-I4-*` |
| Chain governance | Policy | `INDUSTRIAL-GOV-*` |

---

## Work order (priority)

1. **SC** — Supply-chain physicalization — **done**.
2. **I1** — Bridge + proof JSON + coal plant utility — **done** (`I1-05`, `I1-06` via `utilities_coal_plant.json`).
3. **I2** — Resource flow graph stub — **done** (`I2-01`…`I2-04`); propagation/starvation **open**.
4. **I3** — Placeable transformers/substations — **done** (`I3-03`, `I3-04`); grid overload **open**.
5. **GOV** — Anti-collapse at load — **done** (`INDUSTRIAL-GOV-01`).
6. **I4** — Logistics physicalization — **done** (anchors, batch stub, path gate, spatial districts).

---

## I1 — Activation bridge

| Id | Goal | Runtime check |
|----|------|----------------|
| `INDUSTRIAL-I1-01` | `catalog_id` on commit → `BuildingDefinitionRef` | Event + site entity carry ref |
| `INDUSTRIAL-I1-02` | Activate on `Operational` | `activate_industrial_facilities_system` runs after provisioning |
| `INDUSTRIAL-I1-03` | Power from JSON | `ElectricalComponent` scales with `power_consumption` |
| `INDUSTRIAL-I1-04` | Unit tests | `cargo test -p proc_A_dine01 economy:: --lib` |
| `INDUSTRIAL-I1-05` | Proof JSON | `debug_runs/industrial_activation_live.json` |
| `INDUSTRIAL-I1-06` | Power plants | Utilities pick → `PowerPlant` + `plant_definitions.json` |

---

## SC — Supply-chain granularity (assessment §1811–1940)

| Id | Goal | Runtime check |
|----|------|----------------|
| `INDUSTRIAL-SC-01` | Chain index | `assets/configs/industrial_supply_chains.json` |
| `INDUSTRIAL-SC-02` | Per-step catalog | All chain `catalog_id`s load in `BuildingDefinitionRegistry` |
| `INDUSTRIAL-SC-03` | Role activation | `src/economy/supply_chain.rs` maps role → single runtime bundle |
| `INDUSTRIAL-SC-04` | Industrial UI | Submenu grouped by `supply_chain` + power label |
| `INDUSTRIAL-SC-05` | Geopolymer path | `concrete_cement_kiln_geopolymer` + `concrete_mixer_geopolymer` |
| `INDUSTRIAL-SC-06` | Aluminum four steps | mine → refinery → smelter → fabrication distinct activation |
| `INDUSTRIAL-SC-07` | Chain membership | `IndustrialSupplyChainMembership` on operational sites |
| `INDUSTRIAL-SC-08` | Power asymmetry test | Smelter `ElectricalComponent.base_load` ≫ mine (unit test) |

**Concrete chain (placeable):** aggregate mine → kiln → mixer (+ legacy `integrated_plant`).  
**Aluminum chain:** bauxite mine → alumina refinery → smelter → fabrication.

---

## I2 — Resource flow (assessment §2026–2093)

Target shapes:

```rust
pub struct ResourceFlowNode {
    pub inventory: HashMap<ResourceId, f32>,
    pub throughput_limit: f32,
    pub production: Vec<ResourceRate>,
    pub consumption: Vec<ResourceRate>,
}

pub struct ResourceFlowEdge {
    pub from: Entity,
    pub to: Entity,
    pub transport_mode: TransportMode,
    pub max_rate: f32,
    pub latency: f32,
}
```

| Id | Goal |
|----|------|
| `INDUSTRIAL-I2-01` | `ResourceFlowNode` resource + registry |
| `INDUSTRIAL-I2-02` | `ResourceFlowEdge` + `TransportMode` |
| `INDUSTRIAL-I2-03` | Register node at activation from JSON produces/consumes |
| `INDUSTRIAL-I2-04` | JSON strings → `ResourceType` where aligned |
| `INDUSTRIAL-I2-05` | Per-facility inventory buffer on node |
| `INDUSTRIAL-I2-06` | Tick propagation with `throughput_limit` |
| `INDUSTRIAL-I2-07` | Starvation cascade test (refinery starved → smelter stall witness) |

**Emergent loop to prove:** refinery starved → smelter stalls → fabrication backlog → grid expansion delayed → construction slowdown.

---

## I3 — Power distribution (assessment §1939–2133)

| Id | Goal |
|----|------|
| `INDUSTRIAL-I3-01` | Industrial loads join `ElectricalGrid` rebuild |
| `INDUSTRIAL-I3-02` | `GridOverloadEvent` / brownout when bus exceeded — **DONE** · plan [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) |
| `INDUSTRIAL-I3-03` | Transformer + substation **catalog JSON** (placeable utilities) |
| `INDUSTRIAL-I3-04` | Activation spawns `TransformerComponent` / substation from catalog |
| `INDUSTRIAL-I3-05` | Capacity bottleneck gameplay (not decorative transformers) |

---

## I4 — Logistics physicalization

| Id | Goal |
|----|------|
| `INDUSTRIAL-I4-01` | Facility registers `LogisticsGraph` node on activate |
| `INDUSTRIAL-I4-02` | Concrete batch / cure stub (runbook) |
| `INDUSTRIAL-I4-03` | `ResourceFlowEdge` requires logistics path (not teleport) |
| `INDUSTRIAL-I4-04` | Spatial district test: clustered smelters stress one transformer host |

---

## GOV — Do not undevelop chains

| Id | Goal |
|----|------|
| `INDUSTRIAL-GOV-01` | Lint/test: new industrial JSON must declare `supply_chain_role` OR explicit `integrated_plant`; ban unnamed mega-factory rows |

---

## Proof commands

```powershell
cargo test -p proc_A_dine01 economy:: --lib
cargo test -p proc_A_dine01 construction:: --lib
```

---

## Exit gate — `INDUSTRIAL_ACTIVATION_GREEN`

All rows in [`industrial_activation_todos.rs`](industrial_activation_todos.rs) reconcile to **Done** via witness predicates (not static text alone).

Minimum bar for “phase playable” before full green:

- **SC** block green (granular chains placeable + activate correctly)
- **I1** green (proof JSON + power plants)
- **I2-01..03** at least stubbed (nodes registered from JSON)

Full green includes I2 propagation, I3 overload + transformers, I4 logistics edges, GOV-01.

---

## Post-green — **high priority** next lanes

1. **Infrastructure hardening** — VM multiview / per-view isolation ([`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) §1B). Not part of `INDUSTRIAL_ACTIVATION_GREEN`.
2. **Logistics depth** — `ResourceFlowEdge` validity from live **transport** topology (`TransportEdgeDirectory` → `LogisticsGraph` edges), not chunk-anchor proximity alone (`src/strategic/transport_bridge.rs`, `src/economy/resource_flow.rs` `path_open`).
