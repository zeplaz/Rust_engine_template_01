# Infrastructure resilience and failure runbook `v1`

**Purpose:** Define **degradation, repair logistics, network failure, sabotage, cascading collapse, disaster response, and ecological recovery** — infrastructure as **persistent physical systems under stress**, not indestructible scenery.

**Parent:** [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) (program parent: [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md))

**Related:** [`logistics_ai_runbook_v1.md`](logistics_ai_runbook_v1.md), [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md), [`fire_ecology_simulation_runbook_v1.md`](fire_ecology_simulation_runbook_v1.md), [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md)

**Source draft:** [`base_reserch_draft.md`](base_reserch_draft.md)

Version: `v1.0.0`

---

## 1. Core philosophy

Lifecycle: `construction → operation → degradation → overload → damage → repair → adaptation → replacement`.

---

## 2. Failure categories

| Type | Examples |
|:---|:---|
| Mechanical | Wear, fatigue |
| Environmental | Erosion, flood, fire |
| Operational | Overload, congestion |
| Human | Sabotage, neglect |
| Military | Bombing, shelling |
| Ecological | Landslides, root damage |

---

## 3. Health model

```rust
#[derive(Clone, Copy, Debug)]
pub struct InfrastructureCondition {
    pub structural_integrity: f32,
    pub operational_efficiency: f32,
    pub maintenance_debt: f32,
    pub overload_stress: f32,
    pub environmental_damage: f32,
}
```

---

## 4. Aging

Cumulative fatigue, corrosion, thermal stress, erosion, vegetation intrusion, foundation instability — driven by weather, usage, overload, moisture, salt, neglect.

---

## 5. Maintenance crews

```rust
pub enum MaintenanceCrewType {
    RoadRepair,
    RailRepair,
    Electrical,
    PipelineRepair,
    StructuralEngineering,
    FireRecovery,
    FloodRecovery,
}
```

Crews need transport, fuel, spares, labor, safety. Workflow: detect → dispatch → route → deliver → repair → restore.

---

## 6. Prioritization

Examples: military frontline rail, civilian hospitals, industrial grid, economic ports, ecological levees / firebreaks.

---

## 7. Deferred maintenance

Rising repair cost, reduced throughput, rising catastrophic failure probability.

---

## 8. Sabotage

Targets: bridges, substations, rail junctions, pipelines, depots, telecom nodes.

```rust
pub struct SabotageEvent {
    pub severity: f32,
    pub stealth: f32,
    pub infrastructure_target: Entity,
    pub faction: Option<Entity>,
}
```

---

## 9. Counter-sabotage

Patrols, surveillance, loyalty, intel networks, redundancy.

---

## 10. Utility overload

| Cause | Result |
|:---|:---|
| Power demand spike | Transformer stress |
| Rail congestion | Routing collapse |
| Truck overload | Road damage |
| Flood pumping | Grid overload |

```rust
pub struct UtilityLoadState {
    pub current_load: f32,
    pub safe_capacity: f32,
    pub overload_factor: f32,
}
```

---

## 11. Cascading failure

Failures propagate (e.g. substation overload → reroute → neighbor overload → rolling blackout → pumping loss → urban disruption).

---

## 12. Cascading blackouts

Dynamic load balancing, rolling outages, brownouts, emergency isolation.

---

## 13. Dynamic rerouting

Roads (pathfinding), rail (graph dispatch), power (flow balancing), pipelines (pressure) — goals: maintain service, reduce overload, avoid damage zones, preserve strategic supply.

---

## 14. Redundancy

Dual rail, ring grids, bypass roads, reserve substations.

---

## 15. Bridges

Failure causes: overload, sabotage, flood erosion, fire, bombing.

```rust
pub enum BridgeCondition {
    Operational,
    Damaged,
    StructurallyUnsafe,
    Collapsed,
}
```

Collapse may sever logistics, isolate regions, increase congestion, disrupt utilities.

---

## 16. Tunnel flooding

Couple groundwater, rainfall, rivers, pumping, power.

```rust
pub struct TunnelHydrologyState {
    pub water_level: f32,
    pub seepage_rate: f32,
    pub pump_capacity: f32,
}
```

---

## 17. Ecological restoration

Abandoned infrastructure → corridors for regrowth; damage → wetlands, succession stages.

```rust
pub enum EcologicalRecoveryStage {
    Sterile,
    PioneerGrowth,
    Grassland,
    YoungForest,
    MatureForest,
}
```

---

## 18. Infrastructure ↔ ecology feedback

Infrastructure affects fragmentation, runoff, pollution, fire, migration. Ecology affects roots, erosion, flood mitigation, fire risk, visibility.

---

## 19. Fire recovery integration

Burned areas may destabilize slopes, alter hydrology, increase sediment, reduce road stability.

---

## 20. GPU simulation opportunities

Blackout propagation, flood spread, wildfire, traffic density, erosion (high); vegetation recovery (medium).

---

## 21. ECS vs GPU rule

**Authoritative on CPU/ECS:** logistics state, ownership, strategic routing, repair jobs, economy.

**GPU:** propagation fields, visualization, hazard diffusion, large-scale environmental simulation.

---

## 22. Overlay visualization

Maintenance debt, blackout zones, rerouting congestion, flood risk to tunnels, sabotage risk, ecological recovery.

---

## 23. UX principles

Players anticipate failures, build redundancy, prioritize repairs, understand cascades. Avoid invisible failures, arbitrary destruction, opaque routing.

---

## 24. AI strategic behavior

Target bottlenecks, defend substations, repair key corridors, build redundancy, exploit ecological damage *(when aligned with faction AI policy)*.

---

## 25. Critical design rule

Failures create **emergent strategic geography**: bridges reshape fronts, blackouts cripple industry, floods isolate regions, recovery alters terrain — **not** only scripted map events.

---

## 26. Acceptance (v1 drafting)

- [ ] Every failure mode attaches to a **construction-owned** entity or edge.  
- [ ] Cascades have **stopping conditions** and determinism tests.  
- [ ] Rerouting does not bypass [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md) (policy vs mutation).  

---

## 27. Long-term simulation goals

Wartime collapse, industrial decay, environmental disasters, climate stress, strategic bombing resilience, resilient city planning.
