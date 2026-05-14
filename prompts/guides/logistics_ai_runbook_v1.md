# Logistics AI runbook `v1`

**Purpose:** Define **AI logistics management** — routing, prioritization, throughput balancing, redundancy, stockpiles, and emergency response — as **continuous infrastructure optimization**.

**Parent:** [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) → [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Related:** [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md), [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md), [`infrastructure_corridor_runbook_v1.md`](infrastructure_corridor_runbook_v1.md), [`legacy_cpp_repos_agent_communication_maps_v1.md`](legacy_cpp_repos_agent_communication_maps_v1.md) §8.3–§8.4 (hub vs line comms, delayed dispatch)

**Source draft:** [`base_ai_runbook_draft.md`](base_ai_runbook_draft.md)

Version: `v1.0.0`

---

## Execution rounds (run tests after each round)

| Round | Goal | Verify |
|:---:|:---|:---|
| **R1** | Priority weighting stub (military under crisis) | `cargo test -p proc_A_dine01 strategic::runbook_rounds::logistics_ai_policy::tests::round1_military_weight_rises_under_crisis` |
| **R2** | Reroute when congestion + damage exceed threshold | `cargo test -p proc_A_dine01 strategic::runbook_rounds::logistics_ai_policy::tests::round2_reroute_when_congestion_and_damage_stack` |
| **R3** | Demand forecast stub (offense + weather) | `cargo test -p proc_A_dine01 strategic::runbook_rounds::logistics_ai_policy::tests::round3_forecast_rises_with_offense_and_weather` |

**Code (stubs):** [`strategic::runbook_rounds::logistics_ai_policy`](../../src/strategic/runbook_rounds.rs). **Integrates with:** [`logistics_net_inject_into_overlays`](../../src/strategic/logistics_net.rs).

**Live sim:** [`LogisticsAiRuntime`](../../src/strategic/sim.rs).`congestion_proxy` from overlay throughput in [`strategic_fields_coupling_tick`](../../src/strategic/sim.rs).

---

## 1. Scope

Logistics AI controls:

- Routing and prioritization  
- Throughput balancing and redundancy  
- Stockpile allocation  
- Emergency response  

---

## 2. Core philosophy

Logistics is **continuous infrastructure optimization**, not one-shot pathfinding.

---

## 3. Priorities

| Priority | Example |
|:---|:---|
| Military | Frontline ammunition |
| Civilian | Hospitals, water |
| Industrial | Fuel delivery to plants |
| Emergency | Disaster response |

---

## 4. Routing

AI dynamically routes around:

- Congestion, floods, fire  
- Sabotage, bridge collapse  
- Artillery threat *(overlay-driven)*  

---

## 5. Supply graph sketch

```rust
pub struct LogisticsCorridor {
    pub throughput: f32,
    pub vulnerability: f32,
    pub congestion: f32,
}
```

Align fields with corridor + construction graph types when implemented.

---

## 6. Redundancy logic

AI may plan:

- Bypass roads, secondary rail  
- Backup substations, alternate depots  

---

## 7. Strategic stockpiles

Distribute reserves (fuel, shells, food, spares, medical supplies) using:

- Operational threat, disaster risk  
- Seasonal / forecast signals *(weather, industry)*  

---

## 8. Emergency rerouting

Examples:

- Flooded rail → truck reroute  
- Blackout → generator deployment  
- Bridge collapse → pontoon / detour logistics jobs  

---

## 9. Forecasting

Predict demand spikes, offensives, fuel shortages, weather disruption, industrial bottlenecks — outputs should be **inspectable** (policy resource or debug), not magic.

---

## 10. Acceptance (v1 drafting)

- [ ] AI **reads** congestion/threat from overlays + graph integrity from construction/resilience — no duplicate “phantom cost” maps.  
- [ ] Reroute produces **explainable** job intents (for UI or logs).  
- [ ] Stockpile policy respects **UI boundary** — panels set policy, not graph internals.  

---

## 11. Long-term goal

Logistics AI should feel like **living infrastructure management** and **adaptive supply engineering**.
