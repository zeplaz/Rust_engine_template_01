# PLAN-ECS-AI-SCHEDULE-FIT-v1 — perception stays a read, the sim tick stays the schedule

| | |
|---|---|
| **Id** | **PLAN-ECS-AI-SCHEDULE-FIT-v1** |
| **Status** | **OPEN** — slices **ECS-1**, **ECS-2** only |
| **Stance** | Industrial war: power, hauls, plants, factions, towns. `Agent` / `Faction` are a faction kernel. `TownRecord.population` is a `u32`. |
| **Not this program** | Squads, soldiers, formations, vision cones, `SquadObjective`, `UnitOrder`. HTN, blackboards, flow fields, utility-curve assets, a second graph — already cut in [`plan_attrition_systems_counter_v1.md`](plan_attrition_systems_counter_v1.md) and [`plan_macro_ai_counter_v1.md`](plan_macro_ai_counter_v1.md). Town rings and the one growth enqueue stay [`plan_town_growth_v1.md`](plan_town_growth_v1.md) **TWN-AI**. Site scoring stays [`plan_macro_ai_fit_v1.md`](plan_macro_ai_fit_v1.md). |

## Summary

Millington’s command chain has no owner in this tree. Perception already has owners: the strategic read models and `FactionGrowthLook`. Action already has owners: one logistics writer, the construction commit event, and the power `Update` chain. AI frequency is the existing sim tick (`StrategicFieldPipeline`, `TransportSchedule`, `HybridSimPipeline`, the settlement chain after `InfrastructureSiteSet::Planning`, `PowerRuntimePlugin`). That tick does not read the camera. This program adds two tests that lock those boundaries. It does not add entities, sets, or writers.

```text
⊚sync_logistics_graph_from_transport ═▶ ⊨ LogisticsGraph
⊚derive_frontline_from_control_system ═▶ ⊨ FrontlineState  ⊰ ChunkStrategicOverlay.faction_control
⊚world_read_snapshot_refresh_system ═▶ ⊨ WorldReadSnapshot
        ◂⊳[read] ChunkStrategicOverlay · FrontlineState · TransportEdgeDirectory
⊚faction_growth_step_system ◂⊳[FactionGrowthLook] TownBook · DistrictBook · FactionStanceBook
        · Faction.resources · ChunkStrategicOverlay.threat · ShotPulseBook · LogisticsGraph
        · UtilityGraph · PowerLineDamageBook
        ═▶ GrowthProposalQueue, then one pending row
⊚commit_growth_pending_to_funnel_system ═▶ CommitConstructionSiteEvent
        ═▶ commit_construction_site_system ⊨ the site
⊚PowerRuntimePlugin ═▶ ⊨ electrical load and plant output
        run_if power_simulation_running (SimControlState.should_tick)
```

Camera, `ViewManager`, and `ChunkSimLod` do not own the faction step.

## Fit table

| # | Proposal | Verdict | Why |
|---|---|---|---|
| 1 | Commander → squad → unit command chain | **DEFER** | No squad, unit, soldier, formation, vision cone, `SquadObjective`, or `UnitOrder` type exists. Building the chain needs new unit entities. |
| 2 | Perception queries and stimulus events, separate from systems that move or damage | **NARROW** | The read models and `FactionGrowthLook` are already the query. `ECS-1` bans a second writer and a stimulus bus. |
| 3 | AI LOD by component swap plus time-sliced commander / squad / unit schedules | **NARROW** | The sim sets already tick with no camera predicate. `ECS-2` bans a second schedule and an agent swap. The commander/squad/unit sets themselves stay **DEFER** (they need those entities, and an HTN was cut). |

**FIT** would require a missing boundary that one existing writer can grow. None of the three is that case.

## What the tree actually has

### Faction kernel, not a crew

`src/strategic/behavior_entities.rs`

- `Agent` — traits, emotions, `CognitiveState`, `AgentMode` (`Free` / `Scripted` / `Hybrid`). Runtime identity is the ECS `Entity`.
- `Faction` — cohesion, ideology, `resources`, `internal_blocks`, `control_strength`, `FactionInternalStage`, `sub_factions`. Those fields are fracture topology (unified → split), not an order chain.
- `AgentFactionLink` — on the agent entity: `loyalty`, `influence`, `autonomy`.
- `behavior_sync_entity_ids_system` — `PreUpdate` via `behavior_brain_plugin.rs`. It writes `id` fields to match the owning `Entity`.

`src/systems/agents/permissions.rs` has a second `Agent` component (`name`, `faction_id`, `AgentType`). `AgentType` is `HumanPlayer`, `AIPlayer`, `Delegate`, `Observer`. That component holds permission grants (`PermissionDomain::MilitaryCommand` is an access label). It is not a unit and it has no schedule in the strategic pipeline.

`src/strategic/settlement/town.rs` — `TownRecord.population`, `jobs`, `housing` are counts on `TownBook`.

`src/strategic/hybrid_brain.rs` — `perceive_world` / `perceive_world_biased` return `Perception { war_risk, opportunity, instability }` from `WorldFields` and traits. `apply_agent_intent` adds pressure to `WorldIntentField`. `HybridActionKind` is `War` / `Trade` / `Stabilize`. None of these move a body or damage a line.

### One growth site

`src/strategic/settlement/faction_step.rs` — `select_faction_growth` reads `FactionGrowthLook` and returns at most one `GrowthProposal`. `faction_growth_step_system` fills that look, enqueues the proposal, and calls `approve_growth_proposal_into_pending`. It does not take `ResMut<LogisticsGraph>` or `ResMut<UtilityGraph>`. The file header already says it does not write the logistics graph, occupation, or player rows.

`src/strategic/settlement/mod.rs` chains it after `growth_proposal_tick_system`, after `InfrastructureSiteSet::Planning`. `commit_growth_pending_to_funnel_system` emits `CommitConstructionSiteEvent` for `growth:` rows. Test `twn_ai_one_growth_site` already expects one commit and leaves the `player:house` row.

Preview: `src/construction/preview_commit_guard.rs` stamps validation. Gameplay sites are created only when the confirm path writes `CommitConstructionSiteEvent`. Ghost drag does not write occupation.

### Perception resources and their writers

| Read | Writer | Set |
|---|---|---|
| `LogisticsGraph` | `sync_logistics_graph_from_transport` only (`src/strategic/transport_bridge.rs`) | `StrategicFieldPipeline::GraphSync` after `TransportSchedule::CostCache` |
| `ChunkStrategicOverlay` fields | inject, logistics-net, `network_flow_chunk_local_solver_system`, zone apply, site emitters | `StrategicFieldPipeline` then `InfrastructureSiteSet::OperationalZones` |
| `FrontlineState` | `derive_frontline_from_control_system` | `ZoneAndReadModel` |
| `WorldReadSnapshot` | `world_read_snapshot_refresh_system` (means and counts; “not simulation truth”) | `ZoneAndReadModel`, after the frontline derive |
| `UtilityGraph` | utility hydration (`src/infrastructure/utility/graph.rs`) | power topology, not the faction step |
| `NarrativeObservationBus` | `narrative_observations_from_runtime_system` | text for the HUD, after coupling. It does not authorize a haul, a site, or damage |

`FactionGrowthLook` is a stack borrow inside the step. It is not a resource.

### Schedules that already tick without the camera

```text
AdvanceSimTick
  ═▶ TransportSchedule::Topology ═▶ FieldIntegrate ═▶ CostCache
  ═▶ StrategicFieldPipeline::EnsureOverlays ═▶ GraphSync
        ═▶ InjectTransportScalars ═▶ LogisticsNetInject
  ═▶ HybridSimPipeline::IntentReset ═▶ EmotionDrift ═▶ AgentIntent
        (after LogisticsNetInject; resolve in PostUpdate)
  ═▶ NetworkFlow ═▶ ZoneAndReadModel
  ═▶ InfrastructureSiteSet::Planning … OperationalZones
  ═▶ settlement Update chain (metrics → pressure → saturation
        → growth_proposal_tick → faction_growth_step → commit funnel)
PowerRuntimePlugin on Update, run_if(power_simulation_running)
```

`power_simulation_running` reads `SimControlState.should_tick()`. It does not read a view.

`ChunkSimLod` (`src/systems/chunk_sim_lod.rs`) is `Dormant` / `Normal` / `Active` from fire heat and weather, with `dt_scale` 0.4 / 1.0 / 2.0. `strategic_fields_coupling_tick` may skip overlay cell writes on a dormant chunk that is not dirty and not on the global refresh. That skip is an environment dt. It is not an agent swap and it does not wrap `faction_growth_step_system`.

## Current problems

The boundaries exist, and a later slice could still invent a parallel one:

- A stimulus message or `Blackboard` would copy `WorldReadSnapshot`, `FrontlineState`, `LogisticsGraph`, and `UtilityGraph`. **MAI-CUT-BOARD** already refuses that mirror.
- A commander/squad/unit set would be a new macro schedule. **ATR-CUT-STORE** and **ATR-CUT-HTN** already refuse a new schedule and an HTN.
- A camera `run_if`, or a swap that replaces `Agent` with a random attrition roll when the view looks away, would make the sim wrong off-screen. `ChunkSimLod::Dormant` skipping the faction step would do the same.

## Target architecture

Leave the owners above. Queries stay borrows of those resources. The faction step’s write stays `GrowthProposalQueue` plus the one pending row that the existing funnel turns into `CommitConstructionSiteEvent`. Logistics edges stay the output of `sync_logistics_graph_from_transport`. Power stays the `PowerRuntimePlugin` chain. The faction step, transport sets, strategic field sets, hybrid intent sets, and the power chain keep running when no camera resource is present.

## Rejected — not a slice

- `Squad`, `Unit`, `SquadObjective`, `UnitOrder`, formation buffers, vision cones, soldier entities.
- An HTN crate, a global blackboard, utility-curve assets, a flow field, a second logistics or power graph (**ATR-CUT-HTN**, **ATR-CUT-FLOW**, **ATR-CUT-STORE**, **MAI-CUT-GOAP**, **MAI-CUT-UTILITY**, **MAI-CUT-BT**, **MAI-CUT-BOARD**).
- Moving `process_build_order_queue_system` out of `ZoneAndReadModel`, or registering a new `SystemSet`. Both would edit `src/strategic/plugin.rs`, which this program does not touch. **ATR-CUT-STORE** already keeps macro work on the sets that exist.
- Using `ChunkSimLod` to drop or randomize the faction step.
- A stimulus event type. `NarrativeObservationBus` stays HUD text.

## Implementation phases

Serial: **ECS-1 → ECS-2**. Both are tests inside the existing `faction_step` test module. Neither registers a system on the game app.

### ECS-1 — lock the perception read against the logistics and power writers

**Goal.** Prove the faction step can read a live graph and still not write it, and that its only gameplay output is the existing one-site commit.

**Files.** `src/strategic/settlement/faction_step.rs` (`#[cfg(test)]` only).

**Change.** Add `ecs_perception_does_not_write_logistics`. Build the same minimal app as `twn_ai_one_growth_site` (one town, one overlay cell with threat, one `Faction`, the player `pending` row, `CommitConstructionSiteEvent`). Also insert:

- a `LogisticsGraph` with `revision` 7, one node, one edge (`capacity` 4, `disruption` 1, `traversal_cost` 2)
- a `UtilityGraph` with one node and one `power_edges` row (`throughput` 3)

Run one `Update` of `faction_growth_step_system` then `commit_growth_pending_to_funnel_system`. Assert:

- logistics `revision`, node count, edge count, `capacity`, `disruption`, and `traversal_cost` are unchanged
- utility node count, `power_edges` length, and `throughput` are unchanged
- commit count is 1 and the player `pending` row remains
- the app has no message registration for a new stimulus type (the test never calls `add_message` for one)

**Authority.** `⊚faction_growth_step_system` stays a reader of `LogisticsGraph` and `UtilityGraph`. `⊚sync_logistics_graph_from_transport` stays the logistics writer.

**Must not change.** `sync_logistics_graph_from_transport`, `world_read_snapshot_refresh_system`, `perceive_world`, `PowerRuntimePlugin`, `src/strategic/plugin.rs`, `src/construction/mod.rs`, `src/strategic/settlement/mod.rs`, occupation, preview.

**Test.** `cargo test -p proc_A_dine01 --lib ecs_perception_does_not_write_logistics`

**Rollback.** If that test fails because the step writes an edge, stop. Do not add a second writer to repair it.

### ECS-2 — lock the faction step to the sim tick

**Goal.** Prove a threat-in-range site still commits when the app has no camera and the chunk is `ChunkSimLod::Dormant`.

**Files.** `src/strategic/settlement/faction_step.rs` (`#[cfg(test)]` only).

**Change.** Add `ecs_faction_step_ignores_camera_lod`. Same app as **ECS-1**, plus `ChunkSimLod::Dormant` on the overlay entity. Do not insert a camera, `ViewManager`, or viewport resource. One `Update`. Assert commit count is 1. Assert the spawned `Agent` (if the test also spawns the faction kernel `Agent` from `behavior_entities.rs`) still has the same `AgentMode` it was given — the step must not remove or replace that component.

**Authority.** The settlement `Update` chain and `SimControlState.should_tick` remain the gate. `⊚ChunkSimLod` stays the writer of environment `dt_scale` only.

**Must not change.** `ChunkSimLod::dt_scale`, `chunk_sim_lod_refresh`, `strategic_fields_coupling_tick`’s dormant cell skip, `faction_growth_step_system`’s parameters (no new view or LOD argument), `src/strategic/plugin.rs`, `src/strategic/settlement/mod.rs`, `src/construction/mod.rs`. Do not add `run_if` on the settlement chain.

**Test.** `cargo test -p proc_A_dine01 --lib ecs_faction_step_ignores_camera_lod`

**Rollback.** If the commit disappears whenever `ChunkSimLod::Dormant` is present, the step has gained a LOD gate. Remove that gate. Do not replace it with a random roll.

## ECS schedule

No new set. Order stays:

```text
TransportSchedule ═▶ StrategicFieldPipeline::GraphSync
  ═▶ HybridSimPipeline ═▶ ZoneAndReadModel ═▶ InfrastructureSiteSet::Planning
  ═▶ faction_growth_step_system ═▶ commit_growth_pending_to_funnel_system
PowerRuntimePlugin on Update, gated by should_tick
```

## Diagnostics

No new witness file. **ECS-1** compares graph fields before and after one tick. **ECS-2** compares commit count and `AgentMode` with no view resource and a dormant chunk tag.

## Edge cases

- Empty `LogisticsGraph` (the live step treats empty edges as freight-missing). **ECS-1** uses a non-empty graph so a write would be visible. The empty case stays the existing behavior.
- `WorldReadSnapshot` stores means (`mean_threat_slot0`, `contested_chunk_count`). The faction step still walks overlay cells for a tile list because the snapshot has no per-cell list. Do not grow the snapshot into a second grid.
- `process_build_order_queue_system` shares `ZoneAndReadModel` with the snapshot refresh. Leave it there.
- `NarrativeObservationBus` may push a logistics sentence when congestion crosses its threshold. That string is not a stimulus the faction step reads.
- Two `Agent` types exist. Tests that spawn the faction kernel use `crate::strategic::behavior_entities::Agent`. Do not spawn `systems::agents::permissions::Agent` as a stand-in unit.
- Off-screen chunks still run the faction step. `ChunkSimLod::Dormant` may still skip coupling cell writes. Those are different writers.

## Open questions

- Who writes `FactionStanceBook` pairs during play is still open on the town plan. These slices do not add that writer.
- `HybridSimPipeline` contributes intent pressure and does not enqueue sites. Whether a later program should read `WorldIntentField` from the faction step is unanswered here. **ECS-1** and **ECS-2** do not pass `WorldIntentField` into `FactionGrowthLook`.

## Handoff

`@coder` implements **ECS-1**, then **ECS-2**. Both are tests in `src/strategic/settlement/faction_step.rs`. No slice writes `src/construction/mod.rs`, `src/strategic/plugin.rs`, or `src/strategic/settlement/mod.rs`. Verdicts: proposal 1 **DEFER**; proposal 2 **NARROW** (`ECS-1`); proposal 3 **NARROW** (`ECS-2`), with commander/squad/unit sets left **DEFER**.
