# Territorial infrastructure orchestration `v1`

> **STATUS:** Canon **v1** — next-layer schedule tying **city/base development**, **logistics**, **networks** (roads / power / pipes / rail / comms), **defenses**, **underground layers**, **faction pressure**, **operational zones**, **AI construction**, **player planning UX**, **world preview**, **chunk invalidation**, and **future GPU acceleration**.
>
> **Parent:** [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md) · **Construction detail:** [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) · **UI:** [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md), [`base_ui_direction_principls.md`](base_ui_direction_principls.md)

Version: `v1.0.1`  
Audience: engine agents aligning **sites**, **graphs**, **terrain**, and **scheduling** — buildings are **not** isolated entities.

**Coding entry:** [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md) §6 — read this doc **step 3** in that table.

---

## 1. Core architectural rule

Think in:

**territory → infrastructure → logistics → operational capability → population / economy / warfare**

**Not:** click building → building exists.

**Buildings** are **territorial operational nodes** inside a **living infrastructure graph**, same philosophy as corridors (plan → validate → logistics → construct → activate → maintain).

---

## 2. Canonical runtime layers

### Layer A — Terrain reality

Authoritative world partition (conceptually: chunk/cell matrix) holds terrain family, moisture, elevation, **slope**, hydrology, hazards, geology, tags, visibility, movement cost, **underground suitability**.

This remains **source truth** for validation and progression. *(Implementation anchors: terrain / chunk runbooks and material pipeline — names in code may differ from this conceptual matrix.)*

### Layer B — Infrastructure networks

Canonical `NetworkType` examples: Road, Rail, Power, Pipe, Comms, Sewer, MilitarySupply — backed by **graph + chunk overlays**.

Same lifecycle for all: **plan → validate → allocate logistics → construct → activate → maintain → degrade → repair** — **not** instant placement.

### Layer C — Operational sites

Sites are the true **buildings** (factories, depots, civil blocks, fortifications-as-sites):

- Identity, **footprint**, **construction state**
- **Logistics access** and **terrain validation**
- **Supplied / workforce** ratios (operational readiness)
- **Owner** (faction / agent context)

### Layer D — Strategic zones

Not physical meshes — **pressure-field overlays**: supply zones, fire-control, sensor coverage, civil authority, industrial districts, contamination, fortified lines → e.g. `ChunkStrategicOverlay` class semantics.

### Layer E — Agent / faction pressure

Factions and agents read/write all prior layers. Example chain: oligarch faction controls fuel depots → power shortages → civil unrest → insurgent recruitment. **Fracture** adds nuance and instability; it is **seasoning**, not the only core loop (avoid “nation explodes every five minutes”).

---

## 3. Construction lifecycle — site state machine

```rust
pub enum ConstructionState {
    Planned,
    Surveying,
    Clearing,
    Foundation,
    UnderConstruction,
    Provisioning, // built but not necessarily functioning
    Operational,
    Damaged,
    Offline,
    Abandoned,
}
```

**Mirrored in:** [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) §10 (keep these in lockstep).

**Provisioning** is mandatory: structure may **physically exist** but lack power, workers, pipes, or network linkage → **construction complete ≠ operational**. Drives sieges, sabotage, infrastructure warfare, isolation, collapse.

**Progress:** prefer **delivered vs required resources** (and reachable logistics), not raw elapsed time alone — see [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) §11.

---

## 4. Network primitives (unified ECS shape)

Illustrative — align with a **single** canonical graph representation in implementation:

```rust
pub struct InfrastructureNode {
    pub network: NetworkType,
    pub integrity: f32,
    pub throughput: f32,
}

pub struct InfrastructureEdge {
    pub from: Entity,
    pub to: Entity,
    pub congestion: f32,
    pub damage: f32,
}
```

Corridors today use `CorridorConstructionBook`-style **per-edge** state; **sites** should mirror **plan → book/resource → phases → operational attachment** without a second “instant spawn” model.

---

## 5. Underground / layered space

**Not** a fake separate game — **layered infrastructure** on the same world:

```rust
pub enum SpatialLayer {
    Surface,
    Elevated,
    UndergroundShallow,
    UndergroundDeep,
}
```

Couple **terrain** (rock, water table, flood, collapse, geology), **infrastructure** (power, air, pipes, comms, supply shafts), and **visibility** (thermal, sensor, acoustic). Fortified facilities may carry depth / blast resistance / ventilation integrity as site attributes.

---

## 6. Defensive engineering

Defenses are **terrain engineering** — specialized **construction sites**, not a disconnected military minigame:

- Trench, berm, bunker, dragon’s teeth, minefield, sensor post, radar site, artillery pit, hardened depot, shielded power node, etc.

Completed defensive **sites** emit **zones** (detection field, fire-support field, command cohesion field) into strategic overlays.

---

## 7. City / industrial development

Cities **emerge** from inputs (road access, power, water, jobs, safety, terrain, pollution, war risk, supply, resources); player **guides**, not micromanages every house. Factories are **operational sites** with power, workers, transport, raw materials, maintenance; outputs feed the logistics graph.

---

## 8. Player UX

Player = **operational planner**, not spam-click RTS builder.

Flow: **select build mode → ghost placement → terrain validation → logistics prediction → cost projection → commit plan** → simulation owns survey, delivery, construction, activation, maintenance.

**Bevy** for player-facing placement and inspectors per [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md); **operations/build strip** per [`base_ui_direction_principls.md`](base_ui_direction_principls.md).

---

## 9. Tooling architecture

Retire fragmented **`BuildMenuState` / `EntityMenuState`** in favor of one **tool context** vocabulary, e.g.:

```rust
pub enum ToolContext {
    Roads,
    Rail,
    Utilities,
    Military,
    Industry,
    Ecology,
    Civil,
}
```

Shared **placement ghost** (`footprint`, `validity`) and **validation pipeline**: terrain, network access, clearance, hydrology, slope, supply — **one path** for roads, pipes, and sites.

---

## 10. Planning as world state

Planning is a **world layer**, not only ephemeral UI:

- `PlannedSite`, `PlannedCorridor` (or equivalent) — **inspectable, serializable, replayable, AI-readable** for factions, forecasting, multiplayer, campaign scripting.

---

## 11. Chunk / preview integration

Edits (tiles, roads, sites, utilities) emit **dirty** signals: chunk, preview, network as needed — **independent invalidation** for layers (terrain, roads, power, pipes, supply, fortifications, sensors, pollution, ownership). Align with [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md) and world preview runbooks.

---

## 12. GPU acceleration (hybrid)

- **Worldgen GPU:** parallel passes (erosion, moisture, temperature, hydrology, materials); **CPU** remains orchestration authority.
- **Agent / strategic GPU:** batch scoring (threat, desire propagation, path utility, economic weights, pressure diffusion) — **not** full cognitive AI. **CPU:** identity, memory, emotion, narrative, social reasoning.

---

## 13. AI construction

Operational loop: **detect pressure → evaluate shortages → score candidate sites → allocate logistics → issue construction plan → monitor completion.**

Example: frontline fuel shortage → AI scores depot sites → rail access + terrain safety → commits **site** construction plan.

---

## 14. Recommended plugin boundaries

Illustrative split (may merge as the codebase matures):

| Plugin (conceptual) | Role |
|:---|:---|
| `InfrastructurePlugin` | Planning, validation attachment |
| `ConstructionPlugin` | Site/corridor phase progression |
| `OperationalZonesPlugin` | Strategic overlay zones from sites |
| `NetworkFlowPlugin` | Solve / throughput / congestion |
| `StrategicOverlayPlugin` | Composite strategic fields |
| `UndergroundInfrastructurePlugin` | Layered subsurface rules |
| `FactionPressurePlugin` | Pressure reads/writes |
| `AgentBehaviorPlugin` | Decisions, missions |
| `MissionPressurePlugin` | Mission-level pressure packages *(if distinct)* |

Goal: **one territorial operational engineering domain**, not unrelated economy/city/military silos.

---

## 15. Recommended ECS phase order (high level)

1. Input  
2. Planning  
3. Validation  
4. Network solve  
5. Logistics  
6. Construction  
7. Operational zones  
8. Agent decisions  
9. Faction pressure  
10. Preview dirty  
11. Rendering  

Fine-grained ordering must respect existing **FixedUpdate** / scheduler contracts in [`engine_architecture_human_map_v1.md`](engine_architecture_human_map_v1.md) and chunk runbooks.

---

## 16. Design rule (summary)

The player should feel they are **engineering a living territory** — spatial logic, logistics, environmental coupling, social pressure, military vulnerability, and **infrastructure dependence** — **not** placing disconnected objects.

---

## 17. Related documents

| Doc | Relation |
|:---|:---|
| [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) | Construction UX, fortifications, urban emergence |
| [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md) | I1/I2 sequencing |
| [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) | Overlays, corridors, logistics AI |
| [`strategic_program_execution_plan_v1.md`](strategic_program_execution_plan_v1.md) | Execution order |
| [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md) | HUD / operations table direction |
