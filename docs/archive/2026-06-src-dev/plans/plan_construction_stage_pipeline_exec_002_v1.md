# PLAN-CONSTRUCTION-STAGE-PIPELINE-002 — Site stage progress exec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-STAGE-PIPELINE-002** |
| **Parent** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) § Phase 2 |
| **Alignment** | [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@coder A` + `@coder B` |
| **Status** | **SIGNED — READY** |
| **Horizon** | **2–3 weeks** (3 PRs) |

**Hard rules:** Preview never commits. Single lifecycle: [`SiteConstructionPhase`](../../src/strategic/site/resources.rs). Execute funnel only: `execute_construction_plans_system` → `CommitConstructionSiteEvent` → `commit_construction_site_system`.

---

## 1. Problem

Today many paths jump to `SiteConstructionPhase::Operational` on commit (instant spawn). Product needs **visible staged progress** (`progress ∈ [0,1]`) per phase, with forest/obstructed flows using **Clearing substeps**, not six new top-level enum variants.

---

## 2. Out of scope (this plan)

| Item | Where |
|:---|:---|
| New top-level `ConstructionStage` enum | **Forbidden** — use `SiteConstructionPhase` only |
| Workers, equipment, costs, delays | Phase 2+ product |
| Road/rail transport graph edges | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) INFRA-E2-003 (after CON P1) |
| Parametric scaling audit | PLAN-CONSTRUCTION-SCALING-AUDIT-003 / roadmap Phase 3 |

---

## 3. Target behavior

### 3.1 Commit sets Planned, not Operational

On successful `CommitConstructionSiteEvent` handling for a **new** site:

- `phase = SiteConstructionPhase::Planned`
- `operational_readiness = 0.0` (or existing field semantics)
- `SiteStageProgress { progress: 0.0, substep: None }` attached

**Exception:** explicit test fixtures may fast-forward only behind `#[cfg(test)]` or `RUST_ENGINE_CONSTRUCTION_INSTANT=1` (document in env registry).

### 3.2 Sim tick advances phases

`advance_site_construction_tick_system` (sim schedule, after `SimControlSystemSet::AdvanceSimTick`):

1. For each `ConstructionSite` with `SiteStageProgress`:
   - Increment `progress` by `dt * rate(phase, archetype)` (rate table stub: 1.0 phase-units/sec for v1).
2. When `progress >= 1.0`:
   - Advance to next `SiteConstructionPhase` per transition table.
   - Reset `progress = 0.0`.
3. On entering `Operational`:
   - Set `operational_readiness = 1.0`.
   - Emit existing activation hooks (economy) — **not** on commit.

### 3.3 Forest / obstructed example (Clearing substeps)

| Step | `SiteConstructionPhase` | `substep` (optional) |
|:---|:---|:---|
| 1 | `Surveying` | — |
| 2 | `Clearing` | `ClearingSubstep::Trees` |
| 3 | `Clearing` | `ClearingSubstep::Stumps` |
| 4 | `Foundation` | — |
| 5 | `UnderConstruction` | — |
| 6 | `Operational` | — |

`Groundworks` → map to tail of `Clearing` or first half of `Foundation` via product table (v1: **Clearing::Stumps` then `Foundation`**).

---

## 4. Types (add)

**File:** `src/strategic/site/components.rs` (or `src/construction/site_stage.rs` re-exported)

```rust
#[derive(Component, Debug, Clone)]
pub struct SiteStageProgress {
    pub progress: f32,
    pub substep: Option<ClearingSubstep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearingSubstep {
    Trees,
    Stumps,
}
```

**Transition table:** `src/construction/site_stage_transitions.rs` — pure fn `next_phase(current, substep) -> Option<(SiteConstructionPhase, Option<ClearingSubstep>)>`.

---

## 5. PR train

### CON-P2-001 — `SiteStageProgress` + commit sets Planned (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `strategic/site/components.rs`, `strategic/site/systems.rs` (commit handler), `economy/activation/concrete_chain_e2e.rs` (remove instant Operational in default path) |
| **Tests** | `commit_leaves_site_planned_not_operational` |
| **Exit** | Default commit → `Planned`; grep gate: no `Operational` assign in `commit_construction_site` except transition table |

### CON-P2-002 — `advance_site_construction_tick_system` (Coder B)

| Item | Detail |
|:---|:---|
| **Files** | `src/construction/site_stage_tick.rs` (new), register in `ConstructionPlugin` or strategic schedule |
| **Tests** | `forest_pipeline_reaches_operational_in_n_ticks` with fixed dt |
| **Exit** | Sim-only; no render mutation; phases advance in order |

### CON-P2-003 — Witness + live JSON (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `construction/live_proof.rs`, `debug_runs/construction_stage_live.json` |
| **Key** | `construction_site_stage_pipeline_001.green` |
| **Fields** | `phases_observed[]`, `instant_operational_on_commit: false`, `clearing_substeps_seen` |
| **Tests** | `simulation_writes_construction_stage_live_json` or dedicated witness test |

---

## 6. Regression

```powershell
cargo test -p proc_A_dine01 --lib construction
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

---

## 7. Relationship to infrastructure program

| Infrastructure slice | When |
|:---|:---|
| INFRA-E2-003 road → `TransportEdgeRecord` | **After** CON-P2-001 (placement/commit stable) |
| INFRA-E5-001 Town book | **After** CON Phase 5 schema — not this plan |

See [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) §3.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | P2-C1..C3 from construction roadmap; aligned with SiteConstructionPhase authority |
