# Infrastructure corridor runbook `v1`

**Purpose:** Define **infrastructure corridor** planning — transport spines, utility trunks, industrial routes, and military supply axes — as **terrain-aware strategic corridors**, not isolated point placement.

**Parent:** [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md), [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md)

**Source draft:** [`base_ai_runbook_draft.md`](base_ai_runbook_draft.md)

Version: `v1.0.0`

---

## 1. Core philosophy

Infrastructure develops as **terrain-aware strategic corridors**, not click spam at disconnected points.

---

## 2. Corridor types

```rust
pub enum CorridorType {
    Logistics,
    Rail,
    Highway,
    PowerTransmission,
    Pipeline,
    MilitarySupply,
}
```

*(Illustrative — canonical types must match construction runbook graph enums when implemented.)*

---

## 3. Planning inputs

Corridor planning considers:

- Slope, geology, flood risk  
- Population density and land use *(when available)*  
- Strategic exposure and maintenance burden  

---

## 4. Scoring model

```rust
pub struct CorridorCost {
    pub construction: f32,
    pub maintenance: f32,
    pub vulnerability: f32,
    pub throughput: f32,
}
```

---

## 5. Redundancy corridors

Strategic redundancy may include:

- Bypass rails, alternate bridges  
- Distributed substations, parallel highways  

---

## 6. Vulnerability

Threats include:

- Sabotage, artillery  
- Landslides, flooding, fire  
- Congestion as operational threat  

---

## 7. Degradation

Corridors accumulate:

- Wear, erosion, overload damage, environmental decay  

(Resilience detail: [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md).)

---

## 8. AI corridor expansion

AI may expand corridors:

- Toward industrial growth  
- Toward strategic fronts  
- Toward extraction sites  

---

## 9. Acceptance (v1 drafting)

- [ ] Corridor abstraction **maps to** construction graph edges or an explicit corridor component — no orphan “preview only” forever.  
- [ ] Costs expose **why** a route wins (debug explanation or designer-facing breakdown).  
- [ ] Redundant routes interact with **rerouting** once resilience runbook phases exist.  

---

## 10. Long-term goal

Infrastructure networks resemble **adaptive territorial circulatory systems**.
