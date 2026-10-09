# PLAN-AI-FLEET-v1 — faction strategic AI that plays through the player's orders

| Field | Value |
|:---|:---|
| **Id** | **PLAN-AI-FLEET-v1** |
| **Status** | **OPEN** — slices **FAI-0 → FAI-9** |
| **Date** | 2026-10-08 |
| **Owner** | fleet `ai` (charter `fleet_charter_20261008_v1.md`) |
| **Code home** | `src/ai/faction/` (new) · `src/strategic/hybrid_brain.rs` · `src/gui/ai_explainability_ui.rs` |
| **Reconciles** | `plan_macro_ai_fit/counter_v1`, `plan_ecs_ai_schedule_fit/counter_v1`, `plan_prediction_fit/counter_v1`, `plan_vehicular_threat_fit/counter_v1`, `plan_attrition_systems_fit/counter_v1` |
| **Partners** | `plan_logistics_orders_v1.md` (fleet `logistics`), `plan_front_combat_v1.md` (fleet `front`) |
| **Units** | Metric. Tiles via `FrontBalance.tile_size_m` (25 m default). Time in sim ticks / sim seconds. |

## Summary

The five fit/counter pairs (2026-09-27) were written against a pasted Millington catalogue full of
castles, soldiers, and parallel grids. They cut almost everything, and the narrow remainders they
allowed are **already shipped**: `sample_overlay_site` (MAI-1), `rank_town_need` (MAI-2), the ring
search in `faction_step.rs` (MAI-3), `ecs_perception_does_not_write_logistics` (ECS-1),
`ecs_faction_step_ignores_camera_lod` (ECS-2), `sum_path_traversal_cost` + the enqueue gate
(PRD-CUT-PATH), the own-side shot refuse (PRD-CUT-OWN), the role-ordered shed (ATR-1,
`overload_sheds_fabrication_before_mine`), threat-closed hauls (ATR-2, `threat_drops_the_open_edge`),
and the idle solver skip (ATR-3, `solver_skips_an_unchanged_revision`).

What none of them built is the thing the game now needs: **AI factions that decide**. Today the
only faction "decision" is `select_faction_growth`, which is faction-agnostic (it folds every
`Faction.resources` with `max`) and only places one growth site. The behavior brain
(`hybrid_brain`, `behavior_pipeline`) produces world-intent weather and an explainability sample,
but no faction ever issues an order. `ConstructionAiPlugin` is a disabled probe.

The counters' **principles** stay binding. Their **"no new systems"** verdict is lifted for exactly
one thing — a faction commander that is a *client* of the player's command APIs — because the
partner fleets are now creating those APIs (`LogisticsOrderCommand`, `DeploymentOrderQueue`) and
both plans name `ai` as a consumer.

### Ground truth (survey, 2026-10-08)

- No player logistics, front, theater, or deployment **order** API exists yet. The live player
  command surfaces are construction (`CommitConstructionSiteEvent` via `queue_commit_construction_site`
  and `PendingConstructionQueue`) and zone paint (`spawn_zone_at_tile`, `faction_slot: 0` hardcoded).
  `StrategicCommandQueue` is a delayed dispatch queue (8 ticks) the Stage-7 playtest buttons push
  Move/Secure corridor strings into; nothing consumes `delivered` yet.
- Factions are identified four ways (overlay slot `u8`, `Zone.faction_slot`/`TownRecord.faction`,
  `Faction` component entity, dormant `systems::agents::Agent.faction_id`). **This program uses the
  overlay slot `u8` as the faction key** and puts the controller on a `Faction` entity.
- `Faction` entities are **never spawned in the running app** (tests only). FAI-4 spawns one kernel
  per active AI slot at `OnEnter(Simulation)`; the player keeps slot 0.
- `CommitConstructionSiteEvent.owner` is an opaque `Entity` (`BuildCommandActor`,
  `ConstructionAiOwner`, `Entity::PLACEHOLDER` for growth). AI commits carry the faction entity.
- Best existing headless harness: `perception_commit_app` in `faction_step.rs` tests and
  `dev/city_faction_progress.rs` (multi-tick, two factions).

## Reconciliation — decisions per pair

| Pair | Counter's core rule | Decision for this program |
|:---|:---|:---|
| **Macro AI** | No influence book, no GOAP/HTN, no utility crate or curve assets, no blackboard, no citizens | **Kept.** Perception is a pure fold over `ChunkStrategicOverlay` returning a value (`perceive_faction`); nothing is stored as a grid. Goal choice is plain-weight scoring with named terms (`score_goals`) — the same shape `score_proposal` / `rank_town_need` already use. No planner search. The growth step stays the only growth enqueue; the AI's Develop goal feeds it per faction (FAI-6) instead of adding a second build queue. |
| **ECS schedule** | No commander/squad/unit entities *invented*; no stimulus bus; no camera/LOD gate; no third cadence | **Kept, one amendment.** The AI does not spawn units — formations are the `front` fleet's (`Formation`, FC-5). The AI is one controller component on the existing faction entity and three chained systems on the existing `Update` sim tick, gated by `SimControlState::should_tick` and a per-faction **sim-tick interval** (not `FixedUpdate`, not the camera, not `ChunkSimLod`). `FactionAiSet` is an ordering label for partners, not a cadence. |
| **Prediction** | No integrators, arcs, dead reckoning, footprint mini-sims | **Kept.** The AI never predicts arrival; it reads `LogisticsOrdersSnapshot` ETAs and draw-demand health the logistics sim publishes. |
| **Vehicular threat** | No second threat/frag grid, no steering, no `Transform` writes | **Kept.** Threat is read from the overlay (and `PressurePointBook` once FC-4 lands). The AI writes no `Transform`; movement is `execute_deployment_orders_system` + `advance_formations_system` (front). |
| **Attrition** | No second power/logistics graph, no HTN for supply chains, no max-flow aim planner | **Kept.** Supply decisions become `LogisticsOrderCommand`s; the logistics sim allocates vehicles. Targeting stays the front fleet's pressure points. |

**Lifted:** the counters' blanket "no new systems / no new resource". Allowed additions, each with a
single writer: `FactionAiController` (component), `FactionAiDecisionLog` (explainability record —
an audit trail, never read back by the sim), and the three AI systems below.

## What the AI does

Each AI faction slot (overlay `faction_control` slot `u8`; the player keeps its slot without a
controller) runs a loop every `decision_interval_ticks` sim ticks:

```text
perceive  ⊰ ChunkStrategicOverlay (control, threat, logistics_strength) · FrontlineState
            · [FC-3/4] TheaterBook · PressurePointBook · [LO-3] LogisticsOrdersSnapshot
          ═▶ FactionPerception (value; not stored)
decide    ⊰ FactionPerception · FactionDoctrine · current goal
          ═▶ GoalChoice { chosen, all scores with named terms, kept_by_commitment }
plan      ⊰ chosen goal · perception
          ═▶ Vec<AiOrder>  (≤ MAX_ORDERS_PER_DECISION, deterministic targets)
act       AiOrder ═▶ the player's command API (adapter per API) ═▶ OrderOutcome
log       ═▶ FactionAiDecisionLog (record: tick, goal, previous, scores, digest, orders+outcomes)
```

Goals (`StrategicGoal`): **DefendFront** (reinforce the most-threatened front cell / Threatened
pressure point), **Advance** (attack the best push point / Opportunity pressure point),
**SecureSupply** (standing line from the rear anchor to the worst-supplied front sector),
**Develop** (calm rear → one growth site through the growth funnel for *this* faction),
**Hold**. Doctrine (`FactionDoctrine`: aggression, caution, logistics focus, development focus,
commitment) is plain weights, derivable from the faction's lead `Agent` traits
(`FactionDoctrine::from_traits`) so authored personalities show up as different strategies.
Hysteresis: the current goal gets a `commitment` term, so factions do not flip each decision.

### Orders through the player's APIs (no AI bypass)

| `AiOrder` | Player API it goes through | Owner | Until the API lands |
|:---|:---|:---|:---|
| `Reinforce { sector, tile, enemy }` | `DeploymentOrderQueue` ← `DeploymentOrder{Reinforce(pressure point)}` | `front` FC-6 | `StrategicCommandQueue::enqueue_strategic` (`SecureCorridor`), the queue the player's Move/Secure playtest buttons use |
| `Attack { sector, tile, enemy }` | `DeploymentOrder{Attack(tile)}` | `front` FC-6 | `StrategicCommandQueue` (`MoveCorridor`) |
| `Fortify { sector }` | `DeploymentOrder{Fortify(theater)}` → pending construction rows | `front` FC-6/7 | not issued (outcome `Deferred`) |
| `SupplyLine { from, to }` | `LogisticsOrderCommand::CreateStandingLine{owner, source, sink, cargo, priority}` | `logistics` LO-3 | `StrategicCommandQueue` summary (outcome `Fallback`) |
| `SupplyRun { from, to, qty }` | `LogisticsOrderCommand::CreateOneOffRun` | `logistics` LO-3 | same fallback |
| `ServeDemand { demand }` | `CreateStandingLine` for an unserved `DrawDemand` (logistics §4: "AI auto-creates lines for unserved demands") | `logistics` LO-2/3 | — |
| `Develop { tile }` | per-faction growth proposal → `approve_growth_proposal_into_pending` → `CommitConstructionSiteEvent` | settlement (shared) | issued once FAI-6 lands |

Every adapter records an `OrderOutcome` (`Issued{api, id}`, `Fallback{api}`, `Refused{reason}`,
`Deferred{reason}`) in the decision record. A refused order is logged, never retried blindly, and
never forced through a side door. Validation is the API's, not the AI's.

## Schedule placement

```text
Update
  SimControlSystemSet::AdvanceSimTick
  ═▶ StrategicFieldPipeline::ZoneAndReadModel      (FrontlineState, WorldReadSnapshot fresh)
  ═▶ [front] theater + pressure passes             (when present)
  ═▶ FactionAiSet::Decide   faction_ai_decide_system      run_if should_tick
        writes FactionAiController.{goal, next_decision_tick}, FactionAiDecisionLog, PendingAiOrders
  ═▶ FactionAiSet::Act      faction_ai_issue_orders_system (one adapter system per API, chained)
        writes only the API's own enqueue surface (message / queue push)
  ═▶ [front] execute_deployment_orders · [logistics] step_logistics_orders  (consume next tick or same)
PostUpdate: hybrid_resolve_and_feedback_system (seeded RNG after FAI-2)
EguiPrimaryContextPass: ai_explainability_egui_system reads FactionAiDecisionLog
```

No camera, `ViewManager`, or `ChunkSimLod` read anywhere in `src/ai/faction/` (locked by test).

## Data read / written

| Item | AI access |
|:---|:---|
| `ChunkStrategicOverlay`, `FrontlineState`, `OperationalTheaterSummary`, `TheaterBook`, `PressurePointBook`, `LogisticsOrdersSnapshot`, `Faction`, `Agent` traits | **read** |
| `FactionAiController` (component) | **write** — sole writer `faction_ai_decide_system` |
| `FactionAiDecisionLog` | **write** — sole writer `faction_ai_decide_system` (outcomes appended by the act system into the same record via `PendingAiOrders` hand-off) |
| `PendingAiOrders` | decide writes, act drains |
| `StrategicCommandQueue`, `DeploymentOrderQueue`, `LogisticsOrderCommand`, growth pending | **enqueue only**, same calls the player UI makes |
| `LogisticsGraph`, `UtilityGraph`, overlay fields, `Transform`, occupation | **never** |

## Explainable

- Every decision record has all goal scores as named terms (`label`, `input`, `weight`), the
  previous goal, whether commitment kept it, a perception digest, and each order with its API
  outcome. `FactionAiDecisionRecord::explain()` renders plain lines.
- HUD: the existing AI explainability window (`src/gui/ai_explainability_ui.rs`) gains a
  **Faction AI** section — one block per AI faction with its latest record's lines (FAI-5).
- Witness: `debug_runs/faction_ai_live.json` (`_agent_meta` envelope) from the headless scenario.

## Testable headless

- Pure layers (`perception`, `decide`, `plan`) are unit-tested without an `App`.
- `src/ai/faction/scenario.rs`: `MinimalPlugins` app, two overlay chunks, two `Faction` entities
  (one with a controller), sim control resources, the command queues. Runs N ticks and asserts:
  a decision every interval, goals react to a threat stamp, orders reach the APIs, the log
  explains them, and two runs with the same inputs produce identical logs (determinism).

## Phases

| Slice | What | Accept |
|:---|:---|:---|
| **FAI-0** | This plan | committed |
| **FAI-1** | `src/ai/faction/`: perception, decide, log, orders, plan; controller + plugin (decide + act via `StrategicCommandQueue`) registered in `SimulationPlugin` | `cargo test -p proc_A_dine01 --lib ai::faction` |
| **FAI-2** | Determinism: seed `hybrid_resolve_and_feedback_system` from `SimTick` + a seed resource (no `thread_rng`) | `hybrid_resolve_is_seeded` |
| **FAI-3** | Headless scenario + witness JSON; camera/LOD independence lock | `ai::faction::scenario` |
| **FAI-4** | Session seeding: AI controllers on non-player factions at `OnEnter(Simulation)` (doctrine from lead `Agent` traits) so the running app has live AI | app smoke + log non-empty |
| **FAI-5** | Explainability UI: Faction AI section | compiles; reads log only |
| **FAI-6** | Per-faction growth: `select_faction_growth` / step reads the faction's own resources and Develop goal (coordinate settlement owners on board) | `twn_ai_one_growth_site` stays green + new per-faction test |
| **FAI-7** | Front adapter: `DeploymentOrder` via `DeploymentOrderQueue`; targets from `PressurePointBook` (when FC-4/6 land) | adapter test |
| **FAI-8** | Logistics adapter: `LogisticsOrderCommand` standing lines / runs / serve unserved demands (when LO-3 lands) | adapter test |
| **FAI-9** | Tactical layer: per-formation posture advice inside the theater (retreat broken units, hold fortified cells) — through `DeploymentOrder` only | scenario |

FAI-7..9 wait on partner APIs; until then the fallback adapter keeps decisions observable.

## Must not change

- No second grid / heatmap / blackboard; perception is recomputed, the log is never read by sim.
- No unit spawning, no `Transform` write, no camera/LOD gate, no `FixedUpdate` cadence.
- No bypass of any partner validation; one enqueue surface per API, same as the player.
- No unseeded randomness in the AI path.
- `src/strategic/plugin.rs`, `src/construction/mod.rs` untouched.

## Open questions (board)

- `logistics`: order `owner` type — faction slot `u8`? Site ids for front sectors / rear depots?
- `front`: `DeploymentOrder.issued_by` — faction slot? Formation ownership read for an AI slot?
- Who owns `src/strategic/settlement/faction_step.rs` (FAI-6)? Not in the charter table.
