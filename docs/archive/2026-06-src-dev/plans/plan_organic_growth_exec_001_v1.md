# PLAN-ORGANIC-GROWTH-EXEC-001 — Organic settlement growth exec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-ORGANIC-GROWTH-EXEC-001** |
| **Slice** | **ORGANIC-GROWTH-001** |
| **Parent** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| **UX** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) |
| **Prereq** | Phase 2 construction pipeline; district book (Phase 5 minimal); PG-1 archetypes |
| **Rich OG-1** | [`plan_econ_growth_actors_exec_001_v1.md`](plan_econ_growth_actors_exec_001_v1.md) — actors, saturation, employment |
| **Version** | `1.1.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Rule:** Growth **queues** [`ConstructionPlanQueue`](../../src/construction/construction_pipeline.rs) entries — same execute funnel as player builds. No `Operational` spawn shortcut.

---

## Summary

```text
DistrictMetrics + DevelopmentPressure
  → GrowthProposal queue
  → (optional) player approve
  → ConstructionPlan / PlannedSite
  → existing commit + stage pipeline
  → procedural visual (PG-2+)
```

---

## Authority map

| Resource | Writer | Must NOT |
|:---|:---|:---|
| `DistrictBook` / `BlockBook` | strategic loader + sim tick | construction preview |
| `DevelopmentPressure` | `compute_district_pressure_system` | render |
| `GrowthProposalQueue` | growth tick | direct entity spawn |
| `ConstructionPlanQueue` | approve / auto policy | duplicate player queue writer |
| Witness JSON | `construction/live_proof.rs` | hand edit |

---

## OG-1 — Metrics + pressure (≤3 files)

**Rich fields (actors, saturation, employment):** [`plan_econ_growth_actors_exec_001_v1.md`](plan_econ_growth_actors_exec_001_v1.md) — **PROC-OG-1-001** implements **ECON-OG-1-A/B/C** after this stub lands.

| File | Change |
|:---|:---|
| `src/strategic/settlement/district.rs` | **new** — `DistrictMetrics`, `DevelopmentPressure`, `MarketSaturation` |
| `src/strategic/settlement/pressure.rs` | compute from population, jobs, transport reach stub |
| `src/strategic/settlement/market.rs` | **new** — saturation caps + suppression |
| `src/strategic/settlement/actors.rs` | **new** — `GrowthActorLayer`, `BuildingUsage` |
| `src/strategic/settlement/mod.rs` | plugin + resources |

**Inputs v1:** zoning mask, road/rail graph distance, power/water service flags, employment/housing rollup, archetype caps (see econ actors plan).

**Exit:** lib test — high transport → commercial pressure; **4th shop suppressed** when `market_saturation_active`.

---

## OG-2 — Growth proposals (≤3 files)

| File | Change |
|:---|:---|
| `src/strategic/settlement/growth.rs` | `GrowthProposal`, tick → queue |
| `src/construction/construction_pipeline.rs` | accept `PendingEntryKind::GrowthProposal` |
| `src/construction/live_proof.rs` | `construction_organic_growth_001` witness |

**Proposal fields:** `district_id`, `archetype_id`, `usage`, `anchor_tile`, `priority`, `seed`.

**District filter:** reject proposals where `archetype_id` ∉ `DistrictRecord.style_rules.allowed_archetypes` or roof incompatible.

**Exit:** sim tick enqueues proposal; **no** world mutation until approve/execute.

---

## OG-3 — Auto-build policy + approve UI hook (≤3 files)

| File | Change |
|:---|:---|
| `src/strategic/settlement/policy.rs` | `AutoBuildPolicy` per district |
| `src/gui/construction/growth_inspector.rs` | **new** — designer wireframe |
| `src/construction/visual_authority.rs` | dashed proposal ghosts |

**Depends on:** [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md).

---

## OG-4 — Town rollup + GIS hooks (later)

| File | Change |
|:---|:---|
| `src/strategic/settlement/town.rs` | `Town { population, jobs, housing, industries }` |
| save/load RON slice | persist books |
| optional GIS import tags | Phase 8 |

**Do not start** until OG-2 witness green.

---

## Example scenario (acceptance narrative)

Player: road + power + water + rail station in zoned mixed-use district.

After N ticks:

| Proposal | Trigger |
|:---|:---|
| corner shop | commercial pressure + transport |
| grocery | employment + population threshold |
| warehouse | freight + industrial pressure |
| apartments | residential pressure |
| school | population + civic policy |

All appear as **proposals** first; auto-build policy determines instant queue vs approve.

---

## Witness schema

| Pointer | Meaning |
|:---|:---|
| `/construction_organic_growth_001/pressure_wired` | bool |
| `/construction_organic_growth_001/employment_demand_wired` | bool |
| `/construction_organic_growth_001/market_saturation_active` | bool |
| `/construction_organic_growth_001/proposals_queued` | number |
| `/construction_organic_growth_001/execute_via_pipeline` | bool — no bypass |
| `/construction_organic_growth_001/green` | rollup |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib organic_growth settlement construction
```

---

## Anti-patterns

- Zone paint → instant buildings
- Separate economy for auto-build vs player build
- Growth ignoring transport graph
- 500 static catalog entries instead of archetypes

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | ORGANIC-GROWTH-001 · **ECON-OG-1-A/B/C** |
| **Parallel with** | PG-1 (archetypes), Phase 2 pipeline |
| **Designer** | DESIGN-ORGANIC-GROWTH-UX-001 before OG-3 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Initial OG-1..4 exec |
| v1.1.0 | 2026-06-02 | Linked PLAN-ECON-GROWTH-ACTORS-001; extended OG-1 witness keys |
