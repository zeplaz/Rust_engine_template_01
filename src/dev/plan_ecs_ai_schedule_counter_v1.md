# PLAN-ECS-AI-SCHEDULE-COUNTER-v1 — cut the soldier stack

| Field | Value |
|:---|:---|
| **Id** | **PLAN-ECS-AI-SCHEDULE-COUNTER-v1** |
| **Status** | **REFUSAL** — no new entities, events, or schedules |
| **Date** | 2026-09-27 |
| **Owner** | `@planner` (read-only review) |
| **Not this file** | Do not edit `plan_ecs_ai_schedule_fit_v1.md`, `plan_attrition_systems_fit_v1.md`, `src/strategic/`, or `src/construction/` from this plan. Do not commit. |

Industrial sim: power, hauls, plants, growth sites. A commander, a squad, and a soldier who takes a hill are the wrong game. There is no soldier to wake, steer, or cheap out off-screen.

`ECS-CUT-*` ids are refusal gates. They do not add systems. A fit plan must not schedule one. Each cut condition below is already met in the named file.

```text
⊚strategic::behavior_entities::Agent ═▶ ⊨faction kernel (traits, cognition, mode)
⊚Faction ═▶ ⊨pressure aggregator          ⊚AgentFactionLink ⊰ Agent ∧ Faction
⊚systems::agents::Agent ═▶ ⊨player identity (HumanPlayer, AIPlayer, Delegate, Observer)
⊚WorldReadSnapshot ⊰ ChunkStrategicOverlay ∧ FrontlineState
⊚ChunkStrategicOverlay ═▶ ⊨threat, logistics, power_flow
⊚LogisticsGraph ═▶ ⊨haul edges             ⊚sync_logistics_graph_from_transport ═▶ ⊨sole LogisticsEdge writer
⊚UtilityGraph ═▶ ⊨power topology           ⊚damage_power_line_segment ═▶ ⊨segment hp
⊚faction_growth_step_system ═▶ ⊨one GrowthProposal
⊚commit_construction_site_system ═▶ ⊨site commit
⊚TransportSchedule ∧ StrategicFieldPipeline ∧ PowerRuntimePlugin ═▶ ⊨macro cadence
```

One growth site per faction step stays **TWN-AI** in `src/dev/plan_town_growth_v1.md`. Skipping a solver when the tick fingerprint did not change stays **ATR-3** in `src/dev/plan_attrition_systems_fit_v1.md`. This plan does not restate either slice and does not tie either to the camera.

## CUT

| Proposal | Already does the job | Failure if added |
|:---|:---|:---|
| **ECS-CUT-HIERARCHY** — a Commander entity, a Squad entity, a soldier Unit entity, Formation, SquadObjective, UnitOrder, or an HTN that writes “take the hill” | The kernel is `Agent` in `src/strategic/behavior_entities.rs` (`traits`, `emotional_state`, `cognition`, `mode`). `Faction` in that file is the pressure aggregator (`cohesion`, `resources`, `control_strength`). `AgentFactionLink` points one agent entity at one faction entity. `Faction.internal_blocks` is an optional entity list on that component. It is not a squad roster and it is not filled with soldiers. The other `Agent`, in `src/systems/agents/permissions.rs`, is a player identity: `AgentType::{HumanPlayer, AIPlayer, Delegate, Observer}` plus `faction_id`. `MapViewInstanceId::CommanderMap` (`src/gui/map_view/presentation/mod.rs`) is a map view. It is not a commander to spawn. The faction action that exists is `select_faction_growth` / `faction_growth_step_system` (`src/strategic/settlement/faction_step.rs`): one `GrowthProposal`. | A commander, squad, soldier unit, formation, objective, unit order, or HTN. Inventing squads so the hierarchy has something to hold. |
| **ECS-CUT-STIMULUS** — a vision-cone query and `EventReader<SoundStimulus>` | Perception that exists is a read. `world_read_snapshot_refresh_system` (`src/strategic/world_read_snapshot.rs`) writes `WorldReadSnapshot` from `ChunkStrategicOverlay`, `FrontlineState`, and `TransportEdgeDirectory` inside `StrategicFieldPipeline::ZoneAndReadModel`. `derive_frontline_from_control_system` (`src/strategic/frontline.rs`) writes contested chunks from overlay `faction_control`. `faction_growth_step_system` already reads `ChunkStrategicOverlay.threat`, `LogisticsGraph`, and `UtilityGraph` (plus `plant_reaches_consumer` and the damage book). Overlay `visibility` and `recon_confidence` are already on the chunk. No `SoundStimulus` type exists under `src/`. | A cone query. A sound event. A gunshot message whose job is to wake idle soldiers. |
| **ECS-CUT-CAMERA** — `kinematic_steering_system` on screen and `stochastic_attrition_system` off screen, swapped by camera distance | Power, hauls, and damage already resolve without the camera. `PowerRuntimePlugin` (`src/entities/production/power/systems.rs`) chains the grid on `Update` under `power_simulation_running` (`SimControlState::should_tick`). Haul edges are `sync_logistics_graph_from_transport` (`src/strategic/transport_bridge.rs`) in `StrategicFieldPipeline::GraphSync`. Segment hp is `damage_power_line_segment` (`src/construction/power_lines/damage.rs`). Sites commit in `commit_construction_site_system` (`src/strategic/site/systems.rs`, `InfrastructureSiteSet::Planning`). View stays `ViewManager`. The only solver skip already named is **ATR-3**: return when the tick fingerprint did not change. It is not a camera test. | Steering components. A random fight that runs only when the chunk is off-screen. Any sim branch gated on camera distance. |
| **ECS-CUT-CADENCE** — `FixedUpdate` at 2.0s, 0.5s, and every frame as three new schedules | Macro work already time-slices on `Update`. `TransportSchedule` (`src/systems/transport/mod.rs`): `Topology` after `SimControlSystemSet::AdvanceSimTick`, then `FieldIntegrate`, then `CostCache`. `StrategicFieldPipeline` (`src/strategic/plugin.rs`): `EnsureOverlays` → `GraphSync` (also `.after(TransportSchedule::CostCache)` in `src/engine/engine_with_worldgen.rs`) → `InjectTransportScalars` → `LogisticsNetInject` → `NetworkFlow` → `ZoneAndReadModel`, then `InfrastructureSiteSet`. `PowerRuntimePlugin` is the power chain on `Update`, `.run_if(power_simulation_running)`. `tick_strategic_command_queue_system` (`src/strategic/stage7_behavioral.rs`) already runs `.after(AdvanceSimTick)`. `faction_growth_step_system` already sits in the settlement `Update` chain after `growth_proposal_tick_system` and before `commit_growth_pending_to_funnel_system` (`src/strategic/settlement/mod.rs`). `src/` has no `FixedUpdate`. | A third cadence. Three new schedules. Moving those systems onto 2.0s / 0.5s / frame sets. |

## NARROW

| Id | Only behavior allowed | Cut condition (already met — do not schedule) |
|:---|:---|:---|
| **ECS-CUT-HIERARCHY** | No commander, squad, soldier, formation, objective, unit order, or HTN. The kernel stays `Agent` in `src/strategic/behavior_entities.rs`. The faction step may write only the one growth proposal (`GrowthProposalQueue`, then the existing `growth:` pending row). It does not write a hill order. | `select_faction_growth` already returns at most one proposal. |
| **ECS-CUT-STIMULUS** | Perception stays a read of `WorldReadSnapshot`, `ChunkStrategicOverlay`, `FrontlineState`, `LogisticsGraph`, and `UtilityGraph`. No vision cone. No `SoundStimulus`. No gunshot event. | Those five reads already exist. No stimulus type exists. |
| **ECS-CUT-CAMERA** | Damage stays `damage_power_line_segment`. Haul movement stays `sync_logistics_graph_from_transport`. Site placement stays `commit_construction_site_system` after `commit_growth_pending_to_funnel_system`. AI does not write `Transform`. A solver may skip only when the tick fingerprint did not change. That skip is **ATR-3**. Do not restate it here. Do not tie it to the camera. On-screen and off-screen chunks use the same power, haul, and damage path. | Those three writers already apply damage, hauls, and commits. ATR-3 already names the fingerprint skip. |
| **ECS-CUT-CADENCE** | No remainder. Macro systems stay on `TransportSchedule`, `StrategicFieldPipeline`, and the `PowerRuntimePlugin` chain. Do not add a third cadence. | Those three schedules already time-slice the macro work. |

## Do not build

A coder follows this list. None of it is a task. Refuse the work from this file alone.

- Do not add a Commander entity, a Squad entity, a soldier Unit entity, a Formation, a SquadObjective, a UnitOrder, or an HTN that writes “take the hill.” Do not spawn soldiers so `Faction.internal_blocks` has a roster. Do not turn `MapViewInstanceId::CommanderMap` into an entity.
- Do not add a vision-cone query, `EventReader<SoundStimulus>`, or a gunshot event that wakes idle soldiers.
- Do not add `kinematic_steering_system`, a steering component, or `stochastic_attrition_system`. Do not swap them by camera distance. Do not write `Transform` from AI.
- Do not add `FixedUpdate` at 2.0s, 0.5s, and frame rate. Do not add a third cadence beside `TransportSchedule`, `StrategicFieldPipeline`, and `PowerRuntimePlugin`.
- Do not let the faction step write anything except the one growth proposal. Damage stays `damage_power_line_segment`. Hauls stay `sync_logistics_graph_from_transport`. Commits stay `commit_construction_site_system`.
- Do not restate **ATR-3** as a new slice, and do not attach that skip to the camera.
- Do not enqueue more than one growth site in a faction step. That cap is **TWN-AI**.
- Do not schedule `ECS-CUT-HIERARCHY`, `ECS-CUT-STIMULUS`, `ECS-CUT-CAMERA`, or `ECS-CUT-CADENCE`. Their cut conditions are met.
