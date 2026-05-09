# AI city planning runbook `v1`

**Purpose:** Define **AI-driven city formation** — industrial zoning, infrastructure placement, utility scaling, urban adaptation, and strategic settlement evolution. Cities are **emergent infrastructure ecosystems**, not static prefab clusters.

**Parent:** [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`settlement_growth_runbook_v1.md`](settlement_growth_runbook_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md), [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`logistics_ai_runbook_v1.md`](logistics_ai_runbook_v1.md)

**Source draft:** [`base_ai_runbook_draft.md`](base_ai_runbook_draft.md)

Version: `v1.0.0`

---

## 1. Core philosophy

City growth emerges from:

`terrain + resources + water + logistics + industry + defense + politics + population pressure`

---

## 2. AI planning layers

| Layer | Purpose |
|:---|:---|
| Strategic | Regional specialization |
| Urban | District planning |
| Infrastructure | Utility / transport |
| Tactical | Local placement |
| Recovery | Rebuilding / adaptation |

---

## 3. Settlement archetypes

```rust
pub enum SettlementArchetype {
    IndustrialHub,
    LogisticsJunction,
    MiningTown,
    AgriculturalRegion,
    MilitaryFortress,
    CoastalPort,
    ResearchCity,
    EnergyCluster,
}
```

---

## 4. Site evaluation

Score: water access, terrain & flood risk, rail access, resources, defensibility, climate, logistics connectivity.

---

## 5. City skeleton

Initial form:

`transport spine → utility backbone → industrial core → residential → service districts → expansion corridors`

---

## 6. District types

| District | Purpose |
|:---|:---|
| Industrial | Production |
| Logistics | Depots / warehouses |
| Residential | Labor |
| Military | Defense |
| Utility | Substations / trunk lines |
| Research | Institutions |
| Extraction | Raw materials |

---

## 7. Utility planning

Place substations, pumping stations, transformers, pipelines, backup generation using redundancy, terrain, load balancing, strategic resilience.

---

## 8. Congestion-aware planning

Avoid overloaded corridors, rail chokepoints, utility bottlenecks, flood-prone roads.

---

## 9. Defensive urbanism

| Threat | Adaptation |
|:---|:---|
| Artillery | Dispersal |
| Missiles | Hardened substations |
| Drones | Concealment |
| Flooding | Elevated infrastructure |
| Sabotage | Redundancy |

---

## 10. Adaptive rebuilding

Destroyed areas may rebuild differently, decentralize industry, relocate utilities, increase fortification.

---

## 11. GPU overlay inputs

AI consumes overlays for congestion, flood risk, fire spread, recon exposure, pollution, logistics throughput.

---

## 12. Acceptance (v1 drafting)

- [ ] City AI **does not** place structures without a **construction/planning** hook from [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md).  
- [ ] Overlay dependencies are **explicit** (which fields gate which district choices).  
- [ ] Rebuild logic respects **resilience** state (damage, maintenance debt) when available.  

---

## 13. Long-term goal

Cities become **adaptive, evolving infrastructural organisms** instead of static placement grids.
