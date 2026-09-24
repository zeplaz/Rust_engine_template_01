# Military deployable defense — architecture `v1`

| Field | Value |
|:---|:---|
| **ID** | **PLAN-MIL-DEPLOYABLE-DEFENSE-v1** |
| **Priority** | **P2** (construction QoP after place spine) |
| **Owner** | `@planner` (this packet) · `@designer` DES · `@coder` COD · `@sim-steward` witness/authority |
| **Date** | 2026-09-24 |
| **Status** | **machine ★closed** — wall concrete + deployable recipe/logistics/place (WIT-HON) · residual packet synced |
| **Prereq shipped** | **COD-MIL-DEFENSE-PLACE-001** · **COD-WALL-ARCHETYPE-001** · **COD-P3-PLACE-ANIM-001** · **COD-WALL-CONCRETE-GATE-001** · **COD-DEPLOYABLE-*** |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Parent place** | [`design_mil_defense_place_v1.md`](design_mil_defense_place_v1.md) |
| **Index codes** | CB-TOOL · CB-PIPE · CB-CAT · EC-LOG · AI-CON · STR-SIT |
| **Packet** | [`debug_runs/des_mil_deployable_defense_planner_packet.json`](../debug_runs/des_mil_deployable_defense_planner_packet.json) |
| **Residual rollup** | [`debug_runs/des_mil_defense_place_residual_packet.json`](../debug_runs/des_mil_defense_place_residual_packet.json) |

**No production ECS in this turn.** Catalog IA + authority map + phased slices only.

---

## Summary

Defense splits into two product classes that share **one** place funnel (`ActiveBuildTool` → validation → `CommitConstructionSiteEvent`), but differ in **how mass arrives at the tile**:

| Class | Examples | Mass model | Commit gate |
|:---|:---|:---|:---|
| **In-situ material** | Defensive wall (similar poured fortifications) | Bulk **concrete** from stockpile / portland chain | `allows_commit` requires concrete available; consume on commit |
| **Manufactured deployable** | Dragon’s teeth · landmines / minefields | Prefab units: **manufacture → `InTransitLedger` haul → site staging → deploy** | `allows_commit` requires staged unit stock ≥ footprint need |

Operator intent is correct: dragon’s teeth and mines must **not** free-stamp. Concrete walls are **not** the same class — they are poured/cast with material on commit, still two-click, still the same commit event. Mines are lighter unitized cargo than dragon’s teeth but ride the **same** authority chain.

---

## Critique (operator model vs system)

### Accept

1. **Walls = material-backed in-situ** — aligns with existing concrete ManufacturingDomain (`mfg_concrete_batch_v1`), industrial portland chain, and building `concrete_type` / supply-chain tags. Extending **consume-on-commit** for `DefenseKind::DefensiveWall` is a thin gate on the existing funnel, not a new industry tree.
2. **Dragon’s teeth = heavy prefab deployable** — stamp place would violate logistics causality (LOG-B-02: freight must enter `InTransitLedger`, no teleport) and construction north star (preview ≠ commit; execute only after validation).
3. **Mines / minefields = same deployable chain, lighter cargo** — unitized lots via `FreightLot.buffer_tag`; throughput/amount differ; place FSM identical. Research draft already named `DragonTeeth` / `Minefield` under fortification types — product names exist; runtime must not invent a parallel enum outside `DefenseKind` / `SiteArchetype`.

### Reject / correct

| Shortcut | Why rejected |
|:---|:---|
| Instant stamp `DefenseKind` place for teeth/mines | Teleports stock; skips manufacture + ledger |
| New place event bus / UI-only “deploy” | Dual funnel vs `CommitConstructionSiteEvent` — violates invariants §2 |
| Corridor / `ConstructionPlanQueue` for deployables | Roads/rail only (§3); corridor book is transport-edge ledger |
| Wholesale Phase-9 military industry | Industrial activation L3 is early stub; use thin `ManufacturingDomain::Custom` blueprints + existing solver/ledger |
| AI free-spawn sites | `AI-CON` already commits via `CommitConstructionSiteEvent` — keep that; gate on staged stock |
| Mines as “different authority because lighter” | Cargo class ≠ authority class |

### Complexity budget

- **EV high** for QoP honesty (defense feels like logistics + industry, not a stamp palette).
- **Cx medium** if phased; **Cx catastrophic** if Phase-9 invent.
- Verdict: **SHIP_THIN** phased DAG; defer full munitions plants / detection UX / art kits.

---

## Current problems

1. Military picker only places wall / trench / bunker as **immediate** site commits — no material consume, no prefab stock.
2. No `DefenseKind` / `SiteArchetype` for DragonTeeth or Minefield.
3. Manufacturing blueprints cover Concrete / Aluminum / Power only (plus unused `Custom`) — no deployable recipes.
4. Logistics freight is commodity-agnostic (`buffer_tag` + amount) but **no construction staging stock** tied to defense place validation.
5. Residual packet `DES-MIL-DEFENSE-PLACE` marked residuals_closed and scoped out logistics — QoP lane needs new open residuals.

---

## Target architecture

### Authority map (mandatory)

```text
⊚ActiveBuildTool ═▶ ⊨ armed tool (sole construction input source)
⊚DefenseKind ⊰ ⊚ActiveBuildTool          catalog row → SiteArchetype + DeployClass
⊚allows_commit ⊰ ⊚DefenseKind + stock     validation before mutation
⊚CommitConstructionSiteEvent ═▶ ⊨ site spawn   sole defense commit funnel
⊚ManufacturingNode ═▶ ⊨ unit production     blueprint_id → buffer_tag output
⊚ThroughputSolverState ═▶ ⊨ freight alloc
⊚InTransitLedger ⊰ ⊚ThroughputSolver       lots in motion (no teleport)
⊚SiteStagingStock ⊰ ⊚InTransitLedger       arrivals credit staging at portal/site
⊚ConstructionAiConfig ⊰ same commit path    AI place = CommitConstructionSiteEvent + same gates

Preview ghosts ◂⊳[snapshot] ActiveBuildTool     ⛔ never mutate stock / sites
UI / HUD ◂⊳[snapshot] staging + allows_commit   ⛔ not authority
```

### Deploy class (design enum — coder wires thin)

```text
DeployClass::InSituMaterial { material: Concrete }     → DefensiveWall
DeployClass::ManufacturedPrefab { cargo: HeavyUnit }   → DragonTeeth
DeployClass::ManufacturedPrefab { cargo: LightUnit }   → Minefield
```

Both prefab variants share pipeline phases; only recipe id, `buffer_tag`, amount-per-tile, and footprint differ.

### Catalog IA (Military picker — additive)

| Section | Row | SiteArchetype | DeployClass | Place FSM |
|:---|:---|:---|:---|:---|
| Walls | Defensive wall | `DefensiveWall` | InSituMaterial(concrete) | Two-click |
| Trenches | Trench line | `TrenchLine` | *(defer material — keep as today)* | Two-click |
| Fortification | Bunker | `BunkerComplex` | *(defer)* | Two-click |
| **Obstacles** *(new)* | Dragon’s teeth | `DragonTeeth` *(new)* | ManufacturedPrefab heavy | Two-click **gated on staging** |
| **Emplacement** *(new)* | Minefield | `Minefield` *(new)* | ManufacturedPrefab light | Two-click **gated on staging** |
| Editing | Demolish | — | — | pending → confirm |

### Pipeline (prefab only)

```text
ManufacturingNode.tick
  → credit plant/depot output buffer (tag: dragon_teeth_unit | mine_unit)
ThroughputSolver + FacilityPortal
  → dispatch FreightLot { buffer_tag, amount, route… }
InTransitLedger
  → commit_freight_arrivals → SiteStagingStock at destination site/depot
User/AI place request (DefenseKind)
  → allows_commit iff staging ≥ need
  → CommitConstructionSiteEvent
  → debit staging · spawn ConstructionSite
```

### Explicit non-goals (this program)

- Phase-9 military industry tree / munitions plants wholesale
- Mine detection, clearing, or warfare combat loop
- Module kits / APS bake / G-PLAY operator rubric
- Continuous corridor wall topology (see wall archetype `future_corridor_wall_unblock_when`)
- Changing road/rail `ConstructionPlanQueue` semantics

---

## Implementation phases (DAG)

```text
DES-0 ──▶ COD-W1 (wall concrete) ──┐
   │                                ├──▶ COD-P3 (staging+ledger tags) ──▶ COD-P4 (place gate + AI)
   └──▶ COD-P2 (recipes) ───────────┘
```

### ▢DES-0 — Catalog + copy (`DES-MIL-DEPLOYABLE-CATALOG`)

| | |
|:---|:---|
| **Goal** | Charter Obstacles + Emplacement sections; locked labels; contrast table; reject stamp UX |
| **Owner** | `@designer` |
| **Files** | `src/dev/design_mil_deployable_defense_v1.md` (designer charter) · HUD copy registry note · residual update |
| **Authority** | Catalog IA only — no ECS |
| **Witness** | `debug_runs/des_mil_deployable_defense_live.json` |
| **Acceptance** | PASS IA table; player strings locked; Demolish remains footer; walls remain Walls section |
| **Rollback** | Revert charter; keep place spine untouched |

### ▢COD-W1 — Wall concrete gate (`COD-WALL-CONCRETE-GATE-001`)

| | |
|:---|:---|
| **Goal** | `DefenseKind::DefensiveWall` commit requires concrete stock; consume on successful commit |
| **Owner** | `@coder` |
| **Files (expected)** | `build_interaction` / commit path · stock read from existing concrete/industrial buffers · witness collector |
| **Authority** | `⊚allows_commit` + `⊚CommitConstructionSiteEvent`; stock reader ⊰ industrial/concrete buffers — **no new place funnel** |
| **Risks** | Inventing a parallel wallet; hardcoding amounts without catalog field |
| **Witness** | `debug_runs/cod_wall_concrete_gate_live.json` |
| **Acceptance** | Commit blocked when concrete=0; succeed + debit when stock sufficient; trench/bunker unchanged |
| **Compat** | Forward-only; no rewrite of existing sites |
| **Rollback** | Feature-flag gate off if play scenarios starve |

### ▢COD-P2 — Recipes (`COD-DEPLOYABLE-RECIPE-001`)

| | |
|:---|:---|
| **Goal** | Thin `ManufacturingBlueprint` rows: `mfg_dragon_teeth_v1`, `mfg_mine_unit_v1` under `ManufacturingDomain::Custom` |
| **Owner** | `@coder` |
| **Files** | `manufacturing_plugin.rs` registry seed · optional catalog building id mapping later |
| **Authority** | `⊚ManufacturingBlueprintRegistry` / `⊚ManufacturingNode` |
| **Risks** | Spawning new production ECS modules (reject — Custom domain only) |
| **Witness** | fields inside `cod_deployable_pipeline_live.json` (`recipes_registered`) |
| **Acceptance** | Blueprints present; tick can credit tagged buffers in lib harness |
| **Defer** | Dedicated munitions plant buildings → **DR-MIL-MUNITIONS-INDUSTRY** |

### ▢COD-P3 — Logistics + staging (`COD-DEPLOYABLE-LOGISTICS-001`)

| | |
|:---|:---|
| **Goal** | `FreightLot.buffer_tag` for `dragon_teeth_unit` / `mine_unit`; arrivals credit `SiteStagingStock` (or depot staging resource) |
| **Owner** | `@coder` · `@sim-steward` reviews dual-writer risk |
| **Files** | `economy/logistics/propagation` consumers · thin staging resource near construction/strategic site |
| **Authority** | `⊚InTransitLedger` · `⊚ThroughputSolverState` · single writer for staging |
| **Risks** | Teleport credit; dual writers on staging; coupling async_district incorrectly |
| **Witness** | `debug_runs/cod_deployable_pipeline_live.json` (`in_transit`, `staged`) |
| **Acceptance** | Lib: manufacture → dispatch → ticks → staging credit; zero same-tick teleport |
| **Note** | `async_district` remains capacity/solve — do not invent a second freight path |

### ▢COD-P4 — Place + AI (`COD-DEPLOYABLE-PLACE-001`)

| | |
|:---|:---|
| **Goal** | `DefenseKind::{DragonTeeth,Minefield}` + `SiteArchetype` variants; two-click place; `allows_commit` gated on staging; AI uses same event |
| **Owner** | `@coder` |
| **Files** | `build_tool_authority.rs` · `SiteArchetype` · picker · `AI-CON` probe archetype optional |
| **Authority** | Same commit funnel as wall/trench/bunker |
| **Witness** | `debug_runs/cod_deployable_pipeline_live.json` (`place_ok`, `place_blocked_no_stock`, `ai_commit_gated`) |
| **Acceptance** | User + AI: blocked without stock; succeed with debit; no ConstructionPlanQueue for defense |
| **Rollback** | Hide catalog rows if staging incomplete |

---

## ECS schedule plan (sim causality)

Defense place does **not** invent view/render sets. Freight + manufacture stay on existing economy/sim schedules:

```text
Sim tick:
  tick_manufacturing_nodes
    ═▶ credit output buffers
  ThroughputSolver / dispatch_freight_from_solver
    ═▶ write InTransitLedger lots
  commit_freight_arrivals
    ═▶ credit SiteStagingStock (single writer)
  (Input) build two-click + allows_commit ◂⊳[snapshot] staging + concrete stock
    ═▶ CommitConstructionSiteEvent
  commit_construction_site_system
    ═▶ ConstructionSite + debit stock

View / UI:
  ghosts + HUD ◂⊳[snapshot] preview + staging counts
  ⛔ UI never writes staging / ManufacturingNode / ledger
```

Honor bevy-simulation-grade: one writer per staging resource; no UI→Sim invert.

---

## Diagnostics / witnesses

| Artifact | Owner | Proves |
|:---|:---|:---|
| `debug_runs/des_mil_deployable_defense_live.json` | designer | Catalog IA PASS |
| `debug_runs/cod_wall_concrete_gate_live.json` | coder | Wall concrete gate |
| `debug_runs/cod_deployable_pipeline_live.json` | coder | Recipe → transit → stage → place (+ AI) |
| `debug_runs/des_mil_defense_place_residual_packet.json` | planner | Residual rollup |
| `debug_runs/construction_stage_live.json` | steward | Opt-in cross-link only — do not fold Stage gate |
| `debug_runs/logistics_throughput_live.json` | existing | Ledger/solver still green |
| `debug_runs/industrial_activation_live.json` | existing | Concrete chain still green for wall gate |

Validation-first after COD slices: `validate-report cargo --cached --compress 4` · `cargo test -p proc_A_dine01 --lib construction::` · logistics/mfg filters as needed.

---

## Edge cases

- **Zero concrete / zero staging** — ghost may show, commit disabled (`allows_commit == false`); toast/copy from designer charter.
- **Partial freight** — place blocked until full footprint debit; no fractional site spawn.
- **Stale RouteHandle** — honor topology_revision; failed route does not credit staging.
- **Async district** — results apply main-thread only; staging credit after arrival commit, never from UI.
- **Trench / bunker** — unchanged this program (materialization deferred).
- **Saves** — new archetypes forward-only; old worlds without staging simply cannot place new deployables until stock flows.
- **AI probe** — if enabled for Minefield/DragonTeeth, must fail closed without stock (same gate).

---

## Deferrals (`unblock_when`)

| ID | Item | Class | unblock_when | Owner |
|:---|:---|:---|:---|:---|
| **DR-MIL-MUNITIONS-INDUSTRY** | Dedicated munitions / prefab plants beyond Custom blueprints | product_blocked | `cod_deployable_pipeline_live.json` green **AND** industrial L3 charter signed for military domain | this plan → industrial_activation |
| **DR-MIL-MINE-COMBAT** | Detection, clearing, detonation gameplay | product_blocked | Warfare/combat plan signed | future warfare plan |
| **DR-MIL-DEPLOYABLE-ART** | Module kits / iso stamps for teeth & mines | product_blocked | DES charter art brief **AND** MCP G4 stills path | designer-mcp |
| **DR-MIL-TRENCH-MATERIAL** | Trench/bunker material gates | incremental_ok | Wall concrete gate green | this plan residual |

Cite these in `plan_deferral_registry_v1.md` when `@plan-orchestrator` registers the program (footer).

---

## Open questions

| ID | Joint | Ask |
|:---|:---|:---|
| Q1 | `@designer` | Staging UX: show “in transit / staged / ready” on ghost — one line or sheet section? |
| Q2 | `@coder` | Staging resource: per-`ConstructionSite` component vs depot `FacilityPortal` buffer — prefer portal credit then site pull? |
| Q3 | `@coder` | Concrete source for wall gate: portland chain buffer tag vs abstract construction_cost currency — prefer real concrete tag for honesty |
| Q4 | `@sim-steward` | Confirm single writer name for `SiteStagingStock` before COD-P3 lands |

---

## Next ΔWF

```text
@designer DES-MIL-DEPLOYABLE-CATALOG
  → charter design_mil_deployable_defense_v1.md + des_mil_deployable_defense_live.json
@coder COD-WALL-CONCRETE-GATE-001 (can parallel after DES-0 IA locked for wall copy)
@coder COD-DEPLOYABLE-RECIPE-001 → COD-DEPLOYABLE-LOGISTICS-001 → COD-DEPLOYABLE-PLACE-001
@sim-steward WIT-HON on pipeline witness + dual-writer check at P3
```

---

## Plan integration footer

- Register with `@plan-orchestrator` if promoted to indexed program.
- Add **DR-MIL-*** rows to [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) on register.
- Do not pick while APS finish / BQ residual owns operator eyes unless HANDOFF leases construction QoP.
