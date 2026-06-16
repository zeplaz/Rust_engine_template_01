# Infrastructure construction runbook `v1`

**Purpose:** Define **infrastructure, construction, planning, and spatial engineering** — territorial systems engineering rather than isolated building placement or abstract economy screens.

**Parent:** [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Next-layer orchestration (sites, zones, schedule, GPU, AI):** [`territorial_infrastructure_orchestration_v1.md`](territorial_infrastructure_orchestration_v1.md)

**Related:** [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md), [`infrastructure_corridor_runbook_v1.md`](infrastructure_corridor_runbook_v1.md), [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md), [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md), [`fire_ecology_simulation_runbook_v1.md`](fire_ecology_simulation_runbook_v1.md), [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md), [`experience_layer_ux_hud_designer_brief_v1.md`](experience_layer_ux_hud_designer_brief_v1.md) §4 (**BQ-121** blueprint UX)

**Source draft:** [`base_reserch_draft.md`](base_reserch_draft.md)

Version: `v1.0.1`

**Alignment:** §10 **`ConstructionState`** matches [`territorial_infrastructure_orchestration_v1.md`](territorial_infrastructure_orchestration_v1.md) §3 (single canonical site lifecycle).

---

## 1. Core philosophy

Infrastructure is **spatial, physical, logistical, terrain-dependent, damageable, maintainable, interconnected**.

---

## 2. Major categories

| Category | Examples |
|:---|:---|
| Transport | Roads, rails, bridges |
| Utilities | Power lines, substations, pipelines |
| Industry | Factories, refineries |
| Civil | Housing, hospitals, schools |
| Military | Forts, trenches, depots |
| Ecology | Firebreaks, forestry zones |
| Hydrology | Canals, levees, drainage |

---

## 3. Planning-first workflow

`plan → survey → allocate logistics → construct → activate → maintain → degrade → repair`  
**Activate** = network attachment + readiness (supplies, workforce, utilities) — distinct from “physics complete.” **Not** instant spawning.

Operational sites (buildings, forts-as-sites) follow the **same** lifecycle as network spans (roads, rail, pipes, power); see **IX** in [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md).

---

## 4. Infrastructure graph layer

```rust
pub enum InfrastructureNetworkType {
    Roads,
    Rail,
    Power,
    Pipelines,
    Communications,
}
```

---

## 5. Graph structure

```rust
pub struct InfrastructureNode {
    pub id: u64,
    pub position: Vec2,
}

pub struct InfrastructureEdge {
    pub from: u64,
    pub to: u64,
    pub throughput: f32,
    pub integrity: f32,
    pub maintenance_cost: f32,
}
```

---

## 6. Road system

```rust
pub enum RoadClass {
    DirtTrack,
    GravelRoad,
    PavedRoad,
    Highway,
    MilitaryRoad,
}
```

Properties: throughput, weather resistance, mud susceptibility, maintenance, width, speed modifier.

---

## 7. Rail system

High throughput, rigid routing, strategic vulnerability; depots, yards, switching, bridges, tunnels.

---

## 8. Power grid

```rust
pub enum PowerLineClass {
    Local,
    Regional,
    HighVoltage,
}
```

Components: generation, transformers, substations, transmission, storage, switchyards.

---

## 9. Utility placement UX

Splines, snapping, terrain-aware routing, obstacle detection, corridor planning.

---

## 10. Construction states (canonical site / span lifecycle)

**Authoritative definition** (keep in sync with [`territorial_infrastructure_orchestration_v1.md`](territorial_infrastructure_orchestration_v1.md) §3):

```rust
pub enum ConstructionState {
    Planned,
    Surveying,
    Clearing,
    Foundation,
    UnderConstruction,
    Provisioning, // structure may exist; lacks power, workers, pipes, and/or network linkage
    Operational,
    Damaged,
    Offline,      // deliberately or forcibly non-functioning; distinct from Abandoned
    Abandoned,
}
```

**Semantics:**

- **Foundation** — substructure / earthwork / pad phase before vertical **UnderConstruction**.
- **Provisioning** — **built ≠ operational**: sieges, sabotage, logistics isolation, and “factory shell with no inputs” live here.
- **Offline** — controlled or forced shutdown (fuel cut, switch open, crew evacuated) without declaring the site **Abandoned**.

Corridors today may use a reduced phase set in code (`CorridorConstructionPhase`); **new site work** should converge on this enum or an explicit mapping table — **no** “roads stateful / buildings instant spawn” split.

---

## 11. Logistics coupling

Construction consumes labor, fuel, concrete, steel, machinery, transport capacity — **no free placement**.

**Progress model:** prefer **delivered ÷ required** (per resource manifest, reachable by logistics) over raw elapsed time. Time may modulate **rates** (weather, crew), not replace shortfalls.

---

## 12. Terrain coupling

Affects road speed, rail grades, landslide risk, flood exposure, erosion, bridge requirements.

---

## 13. Weather coupling

Mud, flooding, visibility, construction speed, road degradation, outages, fire risk.

---

## 14. Fortification system

```rust
pub enum FortificationType {
    Trench,
    Berm,
    Bunker,
    DragonTeeth,
    Minefield,
    RazorWire,
    ArtilleryPit,
    HardenedDepot,
}
```

Fortifications are **terrain engineering**.

---

## 15. Terrain modification

May alter drainage, erosion, mobility, vegetation, visibility.

---

## 16. Urban development

Cities emerge from transport access, utilities, industry, labor, terrain suitability, defense, economy — **not** only direct building placement.

---

## 17. Resource nodes

Extraction depends on geology, transport, power, labor, environmental conditions.

---

## 18. Suggested flow systems

Engine may suggest efficient logistics routes, rail/utility corridors, firebreaks, congestion mitigation **without** removing player agency.

---

## 19. Build modes / tool context

Player-facing tools should converge on a **single vocabulary** (e.g. `ToolContext` in territorial IX); legacy `BuildMenuState` / `EntityMenuState` should retire in favor of one strip.

| Mode | Purpose |
|:---|:---|
| Transport | Roads / rail |
| Utilities | Power / pipes |
| Industry | Factories |
| Military | Defenses |
| Ecology | Forestry / firebreaks |
| Civil | Housing / services |

---

## 20. Strategic overlay visualization

Prefer world overlays, GPU compositing, contours — **not** excessive floating windows (see [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md)).

---

## 21. Construction UX rules

- Planning: draft corridors, districts, networks, fortification lines before execution.  
- Ghost placement: phased construction, priorities, budgeting, deferred execution.  
- Contextual tools: costs, throughput, terrain penalties, congestion predictions, logistics requirements.

---

## 22. Icons and visual language

Readable silhouettes; linear roads, segmented rail, grid/ lightning power, arrow logistics, angular military, organic ecology, heat fire, pipe/cable utilities.

---

## 23. UI direction

Emphasize overlays, contextual inspectors, strategic readability, minimal clutter. **Boundary:** [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md).

---

## 24. Critical design rule

Infrastructure is **not** decorative. It is simulated, vulnerable, logistical, strategic, environmentally coupled — and must affect warfare, economy, ecology, population, industry, political stability *(when those domains exist)*.

---

## 25. Acceptance (v1 drafting)

- [ ] Graph types are **single canonical** representation for roads/rail/power/pipes/comms *(no shadow ad-hoc graphs)*.  
- [ ] Construction consumes **logistics resources** or explicit “cheat/test” bypass is documented.  
- [ ] Weather / fire coupling follows expansion orchestrator path (no direct terrain ontology mutation from weather).  
- [ ] **Operational sites** use the **same** plan → validate → logistics → phases → **activate** pattern as corridors (no instant building spawn in sim).  
- [ ] Runtime **`ConstructionState`** (or mapped equivalent) implements **Foundation**, **Provisioning**, **Offline** per §10.  
- [ ] **`PlannedSite` / planned corridor** (or equivalent) are **world-serializable**, not UI-only ghosts when committed to planning layer.  

---

## 26. Related step packs

When this domain gets matrices per [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md), add phased step packs under `prompts/matrix/` *(path TBD — do not invent until anchor matrix exists)*.

**Orchestration index:** [`territorial_infrastructure_orchestration_v1.md`](territorial_infrastructure_orchestration_v1.md) — runtime layers, site state machine (`Offline`, `Provisioning`), chunk/preview dirty splits, recommended plugin/schedule order.

---

**Document history:** `v1.0.1` — §10 canonical `ConstructionState` aligned with territorial IX; §3 **activate**; §11 delivered/required; §19 `ToolContext`; §25 extended acceptance; coding kickoff via infra orchestrator §6.
