# Research capability ecosystem runbook `v1`

**Purpose:** Replace linear **tech trees** with **capability emergence** — industrial dependency, organizational development, institutions, and knowledge ecosystems. Research is **infrastructure + maturity + doctrine pressure + experience**, not isolated unlock buttons.

**Parent:** [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`concrete_industry_sim_runbook_v1.md`](concrete_industry_sim_runbook_v1.md), [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md), [`doctrine_simulation_alignment_runbook_v1.md`](doctrine_simulation_alignment_runbook_v1.md), [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md)

**Source draft:** [`base_reserch_draft.md`](base_reserch_draft.md) *(historical filename)*

Version: `v1.0.0`

---

## 1. Core philosophy

Avoid: `Mining II → Steel III → Tank IV`.

Prefer capability formed from:

`material capability + manufacturing sophistication + doctrine + resources + experience → new possibilities`

---

## 2. System layers

| Layer | Purpose |
|:---|:---|
| Knowledge domains | Broad theory |
| Industrial capability | Mass production |
| Practical experience | Operational feedback |
| Institutions | Generation of research / education |
| Doctrine | Strategic prioritization |
| Discovery graph | Emergent unlock conditions |
| Production maturity | Quality / reliability scaling |

---

## 3. Knowledge domains

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnowledgeDomain {
    Metallurgy,
    Chemistry,
    Agriculture,
    Hydrology,
    Logistics,
    CivilEngineering,
    MaterialsScience,
    Electronics,
    Thermodynamics,
    Combustion,
    Aerodynamics,
    Communications,
    Medicine,
    EnergySystems,
    Computing,
}
```

---

## 4. Capability progression

```rust
pub struct KnowledgeProgress {
    pub theory: f32,
    pub practical: f32,
    pub industrialization: f32,
}
```

Example: high metallurgy theory + low industrialization ⇒ prototypes yes, mass production unreliable.

---

## 5. Institutions

| Institution | Contribution |
|:---|:---|
| Universities | Theory |
| Factories | Practical knowledge |
| Laboratories | Experimentation |
| Military | Doctrine pressure |
| Logistics networks | Operational optimization |
| Hospitals | Medical advancement |
| Power grids | Electrification support |

---

## 6. Discovery graph

Technologies are **emergent possibilities** from combined predicates (tooling + metallurgy + stable electricity + labor surplus ⇒ turbine manufacturing **possible**), not a single “unlock turbine” node.

---

## 7. Research pressure

| Pressure | Result |
|:---|:---|
| Warfare | Rapid weapons innovation |
| Famine | Agriculture focus |
| Logistics collapse | Transport optimization |
| Fuel shortages | Energy alternatives |
| Disasters | Engineering adaptation |

---

## 8. Reverse engineering

```rust
pub struct ReverseEngineeringSource {
    pub source_faction: Entity,
    pub technology_complexity: f32,
    pub compatibility: f32,
}
```

---

## 9. Knowledge diffusion

Spread via trade, migration, espionage, education, alliances, captured infrastructure.

---

## 10. Production maturity

```rust
pub struct ProductionMaturity {
    pub defect_rate: f32,
    pub throughput_efficiency: f32,
    pub maintenance_burden: f32,
}
```

Unlocking capability ≠ reliable production.

---

## 11. Organizational friction

Model bureaucracy, inertia, political resistance, retraining costs *(coarse first)*.

---

## 12. Doctrine systems

Doctrine modifies funding, priorities, deployment patterns, specialization.

```rust
pub enum StrategicDoctrine {
    DefensiveEngineering,
    MechanizedWarfare,
    NavalProjection,
    IndustrialExpansion,
    EcologicalStability,
}
```

---

## 13. Gameplay UX

Focus on planning ecosystems, balancing priorities, crisis response, institutions, specialization — **not** clicking static nodes. **UI:** [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md) + [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md).

---

## 14. Strategic overlays

Visualize industrial concentration, educational density, logistics sophistication, electrification coverage, production maturity.

---

## 15. Acceptance (v1 drafting)

- [ ] No new “tech unlock” path that bypasses **institutions + capability dimensions** without an explicit legacy bridge.  
- [ ] Discovery conditions are **data-driven** and testable (determinism contract).  
- [ ] Production maturity couples to **existing industry** runbooks where possible.  

---

## 16. Long-term goals

Support asymmetric development, industrial divergence, emergent technological eras, infrastructure dependency, doctrine-driven economies.
