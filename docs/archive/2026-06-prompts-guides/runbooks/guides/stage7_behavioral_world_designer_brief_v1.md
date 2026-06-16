# Stage-7 behavioral world graph — designer brief `v1`

> **STATUS:** Locked product input for **behavior / command / comms / belief / utility masks** — **no Rust**. Engineering implements via contracts first, full sim after spine + Wave **S → P → C** gates.

Version: `v1.0.0`  
**Full plan:** [`../../docs/archive/2026-06-src-dev/plans/stage7_behavioral_full_plan_v1.md`](../../docs/archive/2026-06-src-dev/plans/stage7_behavioral_full_plan_v1.md) (**PLAN-STAGE7-BEHAVIORAL-001**) · worksheet [`stage7_behavioral_decision_worksheet_v1.md`](stage7_behavioral_decision_worksheet_v1.md)  
**Parent:** [`strategic_program_execution_plan_v1.md`](strategic_program_execution_plan_v1.md) Track F · [`legacy_cpp_repos_agent_communication_maps_v1.md`](legacy_cpp_repos_agent_communication_maps_v1.md) §8  
**Companions:** [`base_behav_a.md`](base_behav_a.md), [`simulation_explainability_runbook_v1.md`](simulation_explainability_runbook_v1.md), [`mission_authoring_framework_v1.md`](mission_authoring_framework_v1.md), [`experience_layer_ux_hud_designer_brief_v1.md`](experience_layer_ux_hud_designer_brief_v1.md) §5 (**UX-D**), [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4 (**BQ-114+**)

---

## 1. Program placement and gates

**Relative order (non-negotiable for production-ready Stage-7):**

```text
VISUAL SPINE EXIT
    → Wave S
    → Wave P
    → Wave C
    → Stage-6 virtualization
    → Stage-7 behavioral world graph
```

**P2 construction** and **P2-H atmosphere** are **not** blocked by Stage-7. **Typed contracts and stubs** may land earlier.

**Safe pre–Stage-7 contracts (enums, resources, DTOs, save schemas, ownership rules, queue types, explainability record types):**

- `CommunicationPlane`, `DispatchMessage`, `BeliefRecord`, `IntelConfidence`, `UtilityChannel`, `StrategicOverlayType`, `MissionIntent`

**Not yet:** full strategic AI, coalition planners, theater simulation, EW propagation solvers.

---

## 2. Minimum shippable slice (MVP)

| Dimension | v1 choice |
|:---|:---|
| Communication plane | **StrategicCommand** only (prove authority + delay) |
| Overlay family | **Recon + logistics stress** |
| Mission type | **Move + secure corridor** |
| Utility channels | **Threat**, **Logistics**, **Visibility** |

**Proves without full warfare AI:** delayed dispatch, stale intel, routing, overlays, explainability, command authority.

**Track F — player-visible in v1:** dispatch delay, stale intel, ghost contacts, logistics lag, congestion overlays, EW/jamming zones, orders-pending state, command queue timeline.

**Sim-only in v1:** low-level belief confidence math, hidden routing heuristics, probabilistic utility internals, internal coalition merge arbitration.

---

## 3. Communications and command

### 3.1 Planes (v1 scope)

| Plane | v1 | Role |
|:---|:---:|:---|
| StrategicCommand | YES | **Orders** |
| LogisticsHub | YES | **Orders + routing** |
| SensorRelay | YES | **Informational** |
| TacticalLine | LIMITED | Local |
| Civilian | NO | Later |

**Authoritative (orders):** StrategicCommand, LogisticsHub. **Informational:** SensorRelay, recon feeds, EW reports.

### 3.2 Hub-isolated storage

**Hard rule** for settlements, depots, rail hubs, ports, warehouses: **local storage + dispatch contracts + transport delay** — not a global magical inventory pool.

### 3.3 Commands

| Command | Delayable |
|:---|:---:|
| halt, retreat, emergency unload, hard stop | NO (immediate) |
| move, logistics reroute, reconnaissance requests | YES |
| secure corridor, report status, change logistics priority | per plane contract |

**Presentation:** explicit diegetic lag — orders in transit, ghost path previews, pending command queue, delayed acknowledgement. Avoid fully hidden lag.

### 3.4 Degradation (v1)

| Source | Supported |
|:---|:---:|
| Terrain, EW, faction tech | YES |
| Weather | LATER |

| Effect | v1 |
|:---|:---:|
| Delay, loss | YES |
| Corruption | LIMITED |
| Duplication | NO |

**Coalition picture:** **not** global omniscience — per faction → per theater → merged by comms/intel contracts.

**Logistics queue:** **parallel** to unit command dispatch — same contracts, different queue ownership.

---

## 4. Belief, intel, and AI layers

**Core rule:** world truth ≠ agent belief ≠ command picture. **Enforced** except in editor, replay, cheat/debug, benchmark.

**Decay (v1):** confidence half-life + refresh on contact/comms (not instant forget).

**Misidentification (v1):** wrong strength band, stale location, false contact, wrong unit category.

**Operational AI:** reads **belief fields only**; dev/debug may use optional truth access; difficulty scales via **better intel**, not omniscience.

**Relationship scalars (v1):** danger, trust, stability, logistics reliability — for routing, mission priority, diplomacy later, overlay coloration.

**Stack:** Layer 1 — fractional/statistical local behavior ([`base_behav_a.md`](base_behav_a.md)). Layer 2 — strategic field planners. Strategic emits **intent**; local resolves **execution**.

**Fog of war:** overlay recon ≠ unit belief — strategic map may show uncertainty while units know less locally.

---

## 5. Utility maps and masks

### 5.1 Authoritative channels

**v1:** threat, logistics, visibility, congestion, instability. **Later:** moisture, heat, subsystem damage, civilian sentiment.

### 5.2 Mask ownership

**Both:** asset-owned (vehicle facing, armor, subsystem zones) and world-owned strategic fields (artillery threat, EW pressure, minefields, congestion).

**Legend:** registry-driven, mod-stable — e.g. `(channel: "Threat", color: [255,0,0])` — not hardcoded gameplay RGB.

**Authoring timing:** **offline first** (bulk mask, utility preprocessing) until Wave **S** and **C** stable and field ownership stabilized; then in-engine layered editing.

### 5.3 One owner per channel

| Channel | Owner |
|:---|:---|
| Threat | strategic AI |
| Logistics | logistics sim |
| Visibility | recon / intel |
| Congestion | transport system |

Consumers read only. Required for multiplayer/modding.

---

## 6. Strategic overlays and warfare

**Mandatory before “real” operational warfare AI:** recon, logistics stress, congestion, threat, EW coverage. **Later:** morale, instability, propaganda.

**EW:** scalar degradation fields (not binary-only); per-plane modifiers for command loss, sensor noise, delay.

**Fronts:** **both** gradient fields **and** named corridors/objectives for mission anchors.

**Warfare targets** must exist in world state: depots, rail hubs, bridges, relay stations, fuel storage — not abstract penalties only.

---

## 7. Logistics, transport, construction

**Separate graphs:** rail, road, pipeline (sea later) with **hub handoff rules**.

**Crisis priorities (v1):** military, humanitarian, industry (export later).

**Construction phases** affect: throughput, vulnerability, routing preference, congestion.

---

## 8. Experience layer (build order)

1. Overlay toggles  
2. Command tray  
3. Intel timeline  
4. Command table  

**Explainability visibility:** default strategic HUD with expanded debug detail and replay export.

**Required contributor buckets:** recon, logistics, EW, doctrine, terrain, mission pressure. Use cases: AI mission assignment, logistics reroutes, command delay, failed offensives, EW disruption.

---

## 9. Persistence and tooling

### 9.1 Save vs recompute

| Save | Recompute |
|:---|:---|
| dispatch queues, command state, mission state, faction intel summaries, authored utility masks | derived overlays, temporary GPU buffers, composited fields, transient visibility caches |

### 9.2 Tooling split

| Python / external | In-engine first |
|:---|:---|
| bulk mask authoring, terrain generation, utility preprocessing | overlays, debug, mission validation |

---

## 10. Acceptance — “done” fixtures

| Fixture | Purpose |
|:---|:---|
| Named scenario RON | mission + pressure injection |
| Overlay snapshot | field owner agreement |
| Dispatch timeline | delay + queue ordering |
| Explainability report | contributor buckets |
| Golden replay | deterministic replay + explainability record |

---

## 11. Explicit non-goals (this wave)

- Unit micro, full diplomacy sim, coalition theater planners, EW propagation solvers at production fidelity  
- Runtime MDJ/UML, legacy C++ singleton buses, `void*` payloads  
- Civilian comms plane, export logistics priority class, weather on message degradation (v1)

---

**Document history:** `v1.0.0` (2026-05-14) — designer lock from Stage-7 / Track F Q&A.
