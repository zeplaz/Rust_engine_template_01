# AI operational warfare runbook `v1`

**Purpose:** Define **operational-scale warfare AI** — strategic shaping, logistics warfare, infrastructure targeting, corridor analysis, attritional pressure — **not** unit micro management.

**Parent:** [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`doctrine_simulation_alignment_runbook_v1.md`](doctrine_simulation_alignment_runbook_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md), [`logistics_ai_runbook_v1.md`](logistics_ai_runbook_v1.md), [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md)

**Source draft:** [`base_ai_runbook_draft.md`](base_ai_runbook_draft.md)

Version: `v1.0.0`

---

## 1. Core philosophy

Modern warfare AI reasons about:

`sustainability + exposure + attrition + logistics + recon + industrial endurance`

---

## 2. Warfare layers

| Layer | Purpose |
|:---|:---|
| Strategic | National / coalition priorities |
| Operational | Front shaping |
| Tactical | Local engagements *(scoped — not micro)* |
| Logistics | Sustainment |
| Information | Recon / EW |

---

## 3. Operational maps

AI uses threat fields, recon confidence, artillery coverage, logistics throughput, terrain mobility, EW intensity.

---

## 4. Front representation

**Not** rigid lines. Prefer:

- Pressure gradients  
- Contested corridors  
- Mobility envelopes  
- Supply dominance zones  

---

## 5. Offensive logic

Before committing, evaluate ammo reserves, rail throughput, weather, recon quality, bridge survivability, EW conditions.

---

## 6. Strategic strike logic

High-value targets include substations, depots, rail bridges, telecom hubs, drone relays, fuel storage.

---

## 7. Attritional warfare

Track replacement rates, industrial losses, shell expenditure, repair throughput, population fatigue *(coarse)*.

---

## 8. Defensive logic

Layered trenches, fallback corridors, drone denial zones, artillery belts, minefields — tie to construction / fortification systems when present.

---

## 9. Drone doctrine (sketch)

```rust
pub enum DroneDoctrine {
    ReconHeavy,
    SaturationStrike,
    EWSuppression,
    LogisticsInterdiction,
}
```

Expand per [`doctrine_simulation_alignment_runbook_v1.md`](doctrine_simulation_alignment_runbook_v1.md) drone ecosystem rules.

---

## 10. Operational goals

| Goal | Behavior |
|:---|:---|
| Encirclement | Corridor severance |
| Attrition | Artillery pressure |
| Disruption | Utility targeting |
| Exhaustion | Economic attacks |
| Denial | Infrastructure destruction |

---

## 11. Acceptance (v1 drafting)

- [ ] Operational decisions **read** overlays + logistics graph; **write** intents (strikes, movement axes, logistics jobs).  
- [ ] No “attack-move wins” shortcuts that ignore supply / recon fields (doctrine anti-pattern).  
- [ ] Strike targeting respects **ownership** of infrastructure components (what exists in world state).  

---

## 12. Long-term goal

AI warfare resembles **adaptive operational planning**, **systems warfare**, and **logistics competition** — not simplistic attack-move behavior.
