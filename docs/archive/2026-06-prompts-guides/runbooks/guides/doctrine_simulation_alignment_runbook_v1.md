# Doctrine simulation alignment runbook `v1`

**Purpose:** Map **modern systems warfare doctrine** to **verify simulation and AI work** — so features reinforce logistics, infrastructure, recon/EW, and attrition instead of “WW2 mechanics with modern skins.”

**Parent:** [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md), [`ai_operational_warfare_runbook_v1.md`](ai_operational_warfare_runbook_v1.md), [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md)

**Source theory (archived prose):** [`base_doctrine_thoery.md`](base_doctrine_thoery.md) *(historical filename: “theory” spelling)*

Version: `v1.0.0`

---

## 1. Doctrine → simulation traceability

| Modern warfare reality | Simulation layer (target) |
|:---|:---|
| Drone reconnaissance | Intel / recon fields |
| EW interference | Signal degradation, jamming overlays |
| Missile strikes | Infrastructure disruption, not “flat HP” |
| Logistics attacks | Throughput collapse, rerouting |
| Trench systems | Terrain engineering, fortification networks |
| Economic / industrial war | Bottlenecks, replacement capacity |
| Information / morale | Population / stability pressure *(coarse)* |
| Satellite / strategic visibility | Strategic visibility field |
| Attritional artillery | Persistent area denial fields |
| Supply survivability | Rerouting graphs, crew jobs |

---

## 2. Core shift (design rule)

**Do not** model war primarily as unit HP combat, isolated tactical nodes only, or rigid front lines.

**Do** model **network warfare**: hubs, substations, repair throughput, industrial replenishment, sensor chains.

---

## 3. Drone ecosystem (not one unit type)

Roles to distinguish (conceptual): recon, loitering munition, FPV strike, EW relay, naval drone, logistics, mine detection, decoy.

**Detection chain:** detect → classify → communicate → authorize → strike → assess. Breaking any step delays or nullifies effects.

---

## 4. Electronic warfare (EW)

Effects include GPS denial, comms jamming, radar suppression, drone disruption, spoofing.

Sketch type:

```rust
pub struct SignalEnvironment {
    pub gps_strength: f32,
    pub radio_noise: f32,
    pub satellite_visibility: f32,
    pub ew_hostility: f32,
}
```

Prefer **chunk fields / faction visibility maps** per [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md).

---

## 5. Fortifications

Extensive, deformable, logistical, maintainable layers: mines, obstacles, trenches, camouflage, EW masking, nets, fallback routes — tied to [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) fortification types.

---

## 6. Missiles as infrastructure warfare

Targets: substations, rail hubs, bridges, depots, ports, factories, telecom. Couple to **cascades** in [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md).

---

## 7. Industrial attrition

Critical systems: shells, drones, repair throughput, power, fuel refining, transport networks. **Question modeled:** can the economy sustain attrition?

---

## 8. Fronts as dynamic pressure fields

Fronts are **pressure gradients**, contested corridors, mobility envelopes — **not** hard borders. Align with [`ai_operational_warfare_runbook_v1.md`](ai_operational_warfare_runbook_v1.md) and overlay composition.

---

## 9. Anti-pattern checklist (release gate)

Reject designs that:

- Paste “modern units” on WW2-style raw combat math  
- Treat drones as fast tanks  
- Treat missiles as generic ranged damage with no infrastructure coupling  
- Use economy as passive income unrelated to logistics  
- Hide supply / visibility entirely from systemic play  

---

## 10. Recommended implementation priority (from doctrine doc)

Use as **sequenced program pressure**, not rigid law:

| Phase | Focus |
|:---:|:---|
| 1 | Core infrastructure — logistics graphs, power, rerouting, maintenance, damage |
| 2 | Recon & intel — visibility fields, sensors, drone recon, confidence / fog |
| 3 | Attrition warfare — artillery logistics, repair throughput, ammunition supply, fortifications, terrain degradation |
| 4 | EW & drones — signal fields, jamming, drone classes, relays, spoofing |
| 5 | Strategic economy — bottlenecks, labor/energy shortages, sanctions/trade, replacement capacity |

**Execution plan overlap:** [`strategic_program_execution_plan_v1.md`](strategic_program_execution_plan_v1.md) (experience + overlay + construction first for engineering practicality).

---

## 11. Artillery doctrine alignment (summary)

- Artillery as **operational geography** — persistent effects, consumption, counter-battery, coupling with drones and logistics jobs (`assets`/ECS ownership TBD).  
- Urban destruction **states**, not binary rubble.  
- Munition diversity and **supply gravity** — shells as logistics outcomes.

*(Full prose: [`base_doctrine_thoery.md`](base_doctrine_thoery.md) mid-document artillery sections.)*

---

## 12. GPU field verification rubric

For each high-suitability field (recon, EW, artillery threat, traffic density, fire spread, blackout propagation):

| Field | Verification question |
|:---|:---|
| Recon | Does low confidence **delay** strike authorization or increase miss risk? |
| EW | Does jamming **break** a step in the detection chain for affected factions? |
| Artillery threat | Do units/routes **pay** measurable cost to cross high threat without field-aware behavior? |
| Logistics density | Do bottlenecks change **routing** outcomes AI/humans can see? |
| Fire / blackout | Does propagation **respect** infrastructure edges and repair state? |

---

## 13. Acceptance (v1 drafting)

- [ ] New war feature entry must cite **which doctrine row** it implements or explicitly mark “arcade exception.”  
- [ ] At least one **end-to-end** scenario (test or scripted) proves logistics + visibility + failure interaction *(when systems exist)*.  
- [ ] Blob / overlay direction preserved: **fields over polygons** for operational geography.  

---

## 14. Long-term architectural insight

Modern warfare simulation trends toward **continuous systems degradation** — consistent with field economics, hybrid agents, and chunked LOD ECS in [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md).
