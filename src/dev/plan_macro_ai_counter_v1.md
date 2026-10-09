# PLAN-MACRO-AI-COUNTER-v1 — cut the Millington expansion

| Field | Value |
|:---|:---|
| **Id** | **PLAN-MACRO-AI-COUNTER-v1** |
| **Status** | **REFUSAL** — no new systems |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` (read-only review) |
| **Not this file** | Do not edit `plan_macro_ai_fit_v1.md`. Do not edit `src/strategic/`, `src/construction/`, or town-growth code from this plan. |

Modern industrial multi-state war: logistics, power, industrial output, faction conflict, settlements. Castle, knight, quarry, sawmill, timber, granary, keep, and town-hall examples are the wrong game even when a framework lists them.

`MAI-CUT-*` ids are refusal gates. They do not add systems. `plan_macro_ai_fit_v1.md` must not schedule one until its cut condition is met. Each condition below is already met in the named file, so the fit plan does not schedule them.

```text
⊚ChunkStrategicOverlay ═▶ ⊨threat, fire_risk, power_flow, logistics_strength, logistics_flow
⊚ChunkFireOverlay.heat ═▶ ⊨cell fire heat
⊚LogisticsGraph ═▶ ⊨macro haul edges   ⊚sync_logistics_graph_from_transport ═▶ ⊨its only writer
⊚GrowthProposalQueue ═▶ ⊨next site intent
⊚commit_growth_pending_to_funnel_system ═▶ ⊨CommitConstructionSiteEvent for growth: rows
⊚FrontlineState ⊰ faction_control on ChunkStrategicOverlay
⊚TownBook ═▶ ⊨towns    ⊚DistrictBook ═▶ ⊨zones
```

One growth site per faction step stays **TWN-AI** in `src/dev/plan_town_growth_v1.md`. This plan does not raise that cap. Polygon town limits stay **TWN-OV1** / **TWN-OV2**. This plan does not redesign them.

## CUT

| Proposal | Already does the job | Failure if added |
|:---|:---|:---|
| **MAI-CUT-INFLUENCE** — a new influence heatmap | `ChunkStrategicOverlay` (`src/strategic/mod.rs`) is the grid: per-cell `threat`, `artillery_danger`, `fire_risk`, `power_flow`, `logistics_strength`, `logistics_flow`, `logistics_throughput`. `ChunkFireOverlay.heat` (`src/systems/fire/types.rs`) is the fire heat grid; `chunk_fire_overlay_tick` is its writer. `logistics_net_inject_into_overlays` writes logistics into the overlay. `network_flow` writes `power_flow`. A build-suitability read already exists as `site_score` in `src/strategic/runbook_rounds.rs` (`city_planning`): it adds `logistics_strength`, `flood_risk`, and `mean_threat` with fixed weights and returns a scalar. | A second grid writer of the same numbers. Nondeterministic dual decay. A suitability map that drifts from the overlay. |
| **MAI-CUT-GOAP** — GOAP / STRIPS tech tree | `growth_proposal_tick_system` (`src/strategic/settlement/growth.rs`) enqueues `GrowthProposal`. `growth_actor_may_enqueue` allows only `GrowthActorLayer::Growth`. `pending_blueprint_from_growth_proposal` labels `growth:{archetype}`. `commit_growth_pending_to_funnel_system` (`src/strategic/settlement/execute.rs`) emits `CommitConstructionSiteEvent` for those rows only. `GrowthReasonCode` is already the precondition vocabulary: `TransportHigh`, `EmploymentDemand`, `HousingDeficit`, `MarketSaturated`, `UtilityMissing`, `PolicyBlocked`. | A second planner that picks the next site beside the queue. Nondeterministic action search. A medieval tech ladder (quarry, sawmill, timber, granary, keep, town hall). |
| **MAI-CUT-UTILITY** — utility curves and a utility-AI crate | A faction step can rank three reads with plain weights. Power: `sample_network_flow_at_world_tile` → `NetworkFlowFieldSample.power_flow` (`src/strategic/network_flow.rs`), and `plant_reaches_consumer` on `UtilityGraph` (`src/infrastructure/utility/activation_link.rs`). Stock: `LogisticsNode.stockpile` on `LogisticsGraph`, plus `ChunkFieldCell.supply` / `demand` (`src/strategic/world_field_layers.rs`). Threat: `ChunkStrategicOverlay.threat` and `ShotPulseBook` (`shot_in_range`, `SHOT_MAX_RANGE` in `src/strategic/shot_pulse.rs`). `score_agent_cpu` (`src/strategic/agent_batch_scoring.rs`) and `score_proposal` already apply plain weights. | Curve assets and a second score that disagrees with `score_proposal`. A crate the tick does not need. |
| **MAI-CUT-BT** — behavior trees and citizen FSMs | There is no citizen and no worker entity. `Agent` / `Faction` / `AgentFactionLink` (`src/strategic/behavior_entities.rs`) are faction kernels, not residents. Town population is a `u32` on the settlement record, not an entity. Vegetation “population” in ecology is plants. | Inventing people so a tree has someone to tick. Wrong game. A second brain next to `Agent`. |
| **MAI-CUT-HPA** — hierarchical A* | Macro routes already exist. `rebuild_logistics_graph_from_transport` builds `LogisticsGraph` from `TransportEdgeDirectory` (`src/strategic/transport_bridge.rs`). `sync_logistics_graph_from_transport` is the only writer. `LogisticsEdge` connects anchored nodes (district and corridor cells) with `traversal_cost`. Cell cost search is `logistics_path_dijkstra` (`src/systems/navigation/logistics_floodfill.rs`), not a cluster hierarchy. | HPA* clusters as a second route authority over the same edges. Stale cluster handles. |
| **MAI-CUT-BOARD** — a Blackboard resource | ECS resources are the board. A manager reads `WorldReadSnapshot`, `FrontlineState`, `LogisticsGraph`, `ThroughputSolverState`, `UtilityGraph`, `TransportEdgeDirectory`, `GrowthProposalQueue`, `PendingConstructionQueue`, `TownBook`, `DistrictBook`, `ShotPulseBook`, `PressureField`, and the `ChunkStrategicOverlay` / `ChunkFireOverlay` queries. `Faction` is a component (`resources`, `cohesion`), not a bag to copy. | A mirror of those resources. A second writer of threat, power, or stock. |
| **MAI-CUT-WALL** — chokepoints and wall kits | Defensive logistics is the existing catalog id `warehouse` from `archetype_for_zoning` when the zone is `ZoningClass::Industrial` (`src/strategic/settlement/execute.rs`). It enters play only as a `GrowthProposal` (`catalog_id` = that archetype), a `growth:` pending row, then `CommitConstructionSiteEvent`, and only when overlay threat (or a `ShotPulse` target) is in range of the town. Player in-situ wall / trench / bunker tools stay on their current commit path. They are not an AI kit. | A wall-kit catalog, a keep, a chokepoint graph, and a second writer beside the growth funnel. `BuildOrderQueue` / `StructureType::Fortification` (`src/strategic/build_order.rs`) must not grow into that kit. |

## NARROW

| Id | Only behavior allowed | Cut condition (already met — do not schedule) |
|:---|:---|:---|
| **MAI-CUT-INFLUENCE** | Read `site_score(logistics_strength, flood_risk, mean_threat)`. Sample `logistics_strength` and `threat` from `ChunkStrategicOverlay` and pass fire or flood only as the existing `flood_risk` argument. Do not store the score on a map. | `ChunkStrategicOverlay` plus `site_score` already cover a build-suitability query. |
| **MAI-CUT-GOAP** | Preconditions stay data the existing tick already understands: `GrowthReasonCode` and `DistrictStyleRules.allowed_archetypes`. No planner engine, no STRIPS operator list, no tech-tree asset. | `CommitConstructionSiteEvent` plus the `growth:` enqueue already sequences the next site. |
| **MAI-CUT-UTILITY** | In the one faction step from **TWN-AI**, rank at most power shortfall (`1 - power_flow`, or plant not reaching a consumer), stock shortfall (`demand - supply` or a low `stockpile`), and threat in range, with three plain weights. First legal match still enqueues one site. No curve asset. | `score_proposal` and `score_agent_cpu` already weight scalars. |
| **MAI-CUT-FRONT** | The zero line may label the next site as forward or internal. Read `FrontlineState.contested_chunks` from `derive_frontline_from_control_system` (`src/strategic/frontline.rs`), which already marks where `faction_control[0]` and `faction_control[1]` are both high and close. A label may also read `faction_control[slot] - faction_control[other]` on the existing overlay. It must not allocate a map, call `set_faction_threat`, write `fire_risk`, or create a town book. The site still goes through one `GrowthProposal` and the growth funnel. | `FrontlineState` already derives the contested set from `faction_control`. |

Behavior trees, hierarchical A*, the blackboard, and wall kits have no remainder. They stay on the CUT list only.

## Do not build

A coder follows this list. None of it is a task.

- Do not add an influence map, heatmap resource, or GPU influence field.
- Do not add a GOAP, STRIPS, behavior-tree, or utility-AI crate. Do not add curve RON or JSON.
- Do not add a citizen, worker, resident, or population entity. Do not tick a tree because `Agent` exists.
- Do not add HPA* clusters, a chokepoint graph, or a second writer of `LogisticsGraph`.
- Do not add a `Blackboard` resource or a copy of the resources listed under **MAI-CUT-BOARD**.
- Do not add wall, trench, bunker, keep, castle, palisade, or gate kits for faction growth. Do not enqueue them from the faction step. Do not extend `StructureType::Fortification` or `BuildOrderQueue` into that kit.
- Do not spawn a frontline map, a danger writer, or a second `TownBook` / `DistrictBook`.
- Do not enqueue more than one growth site in a faction step. That cap is **TWN-AI**. `growth_proposal_tick_system` may still score districts; the faction step does not get a higher cap from this plan.
- Do not redesign polygon membership or the ring draw. Those are **TWN-OV1** and **TWN-OV2**.
- Do not add catalog tokens for knight, quarry, sawmill, timber, granary, keep, or town hall. `warehouse`, `house`, `corner_shop`, `civic_hall`, `harbor`, and `oil_rig` stay the tokens `archetype_for_zoning` and the water rule already name.
- Do not schedule `MAI-CUT-INFLUENCE`, `MAI-CUT-GOAP`, `MAI-CUT-UTILITY`, `MAI-CUT-BT`, `MAI-CUT-HPA`, `MAI-CUT-BOARD`, `MAI-CUT-WALL`, or `MAI-CUT-FRONT`. Their cut conditions are met.
