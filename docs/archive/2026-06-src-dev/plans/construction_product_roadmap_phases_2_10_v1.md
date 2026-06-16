# Construction & settlement product roadmap — Phases 2–10 `v1`

| Field | Value |
|:---|:---|
| **Doc ID** | **CONSTRUCTION-ROADMAP-2-10** |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **SIGNED — aligned** |
| **Alignment hub** | [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) |
| **Phase 2 exec** | [`plan_construction_stage_pipeline_exec_002_v1.md`](plan_construction_stage_pipeline_exec_002_v1.md) |
| **Infrastructure program** | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) (transport/graph — **not** site phases) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Spine** | [`recovery_construction.md`](recovery_construction.md) · `src/construction/` |
| **Immediate order** | User-recommended 1→11 (below) |

**Rule:** No instant gameplay spawn from preview. All commits through execute funnel. No parallel construction authority outside `src/construction/` + strategic commit handlers.

---

## Recommended order (accepted)

| P | Lane | Repo mapping | Prereq |
|:---:|:---|:---|:---|
| **1** | Placement validation | `build_validation.rs`, `allows_commit`, parametric ghost | — |
| **2** | Construction pipeline (staged build) | `SiteConstructionPhase` + sim tick | **1** |
| **3** | Building scaling audit | `placement_scaling.rs`, `FootprintMatrix`, `parametric_commit.rs` | **1** |
| **4** | Placeholder building assets | **→ module kit** [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) + PG exec | **3** |
| **5** | Town / district hierarchy | `strategic/settlement/` + [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) OG-4 | **2** |
| **6** | Organic settlement growth | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) + UX | **5** |
| **7** | Logistics & trade | `economy/logistics/`, transport R8 — **gate:** INFRA Epic 1 hydrate (see alignment hub §3) | **5** + INFRA E1–E3 |
| **8** | Real-world GIS import | `terrain/`, worldgen IO (RON/JSON) | worldgen stable |
| **9** | Military industry | `economy/activation/` on civilian spine | **7** |
| **10** | Command & control | comms overlay (new lane) | **9** |
| **11** | Frontline warfare | operational layer (not RTS micro) | **10** |

---

## Phase 2 — Construction pipeline (not instant spawn)

### Product intent

Forest (or obstructed) tiles advance through **visible stages** with `progress ∈ [0,1]` per stage. Enables later: equipment, workers, costs, delays — without implementing those in Phase 2.

### Already in repo (do not rename blindly)

**Authoritative phase enum** — [`SiteConstructionPhase`](../strategic/site/resources.rs):

```text
Planned → Surveying → Clearing → Foundation → UnderConstruction → Provisioning → Operational
         (+ Damaged / Offline / Abandoned)
```

**Site component** — [`ConstructionSite`](../strategic/site/components.rs): `phase`, `operational_readiness` (no per-stage `progress` yet).

**Execute funnel** — [`execute_construction_plans_system`](../construction/construction_pipeline.rs) → [`CommitConstructionSiteEvent`](../strategic/site/events.rs) → `commit_construction_site_system`.

**Gap vs your sketch:** `Groundworks`, `Building` as names; forest sub-steps (`Clear Trees`, `Remove Stumps`) as **Clearing** substeps, not top-level enum variants (keeps enum small).

### Proposed alignment (Phase 2 scope)

| Your `ConstructionStage` | Repo authority | Notes |
|:---|:---|:---|
| `Surveying` | `SiteConstructionPhase::Surveying` | exists |
| `Clearing` | `Clearing` | split internally (see below) |
| `Groundworks` | **new** or map to `Clearing` tail | planner choice: add variant **or** `ClearingSubstep` resource |
| `Foundation` | `Foundation` | exists |
| `Building` | `UnderConstruction` | rename display only |
| `Finished` | `Operational` | exists |

**Do not** add a second `ConstructionStage` on `ConstructionSite` without deprecating `SiteConstructionPhase` — one lifecycle writer.

### Forest tile pipeline (example)

```text
Commit (execute) → Planned
  → Surveying        (progress 0→1)
  → Clearing::Trees  (substep)
  → Clearing::Stumps (substep)
  → Foundation
  → UnderConstruction
  → Operational
```

| Task | Owner | Files (≤3 per PR) |
|:---|:---|:---|
| P2-C1 | `SiteStageProgress` component: `phase`, `progress`, optional `substep` | `strategic/site/components.rs` |
| P2-C2 | `advance_site_construction_tick_system` — sim-only, no render write | `construction/site_stage_tick.rs` (new) |
| P2-C3 | Witness: `construction_site_stage_pipeline_001` in `construction_stage_live.json` | `construction/live_proof.rs` |

**Anti-patterns:** spawn `Operational` on commit; bypass `ConstructionPlanQueue`; UI timer that skips validation.

---

## Phase 3 — Building scaling audit

### Product intent

Drag scaling shows **occupied / blocked / terrain-mod** tiles before confirm.

### Already in repo

| Piece | Location |
|:---|:---|
| Continuous scale | [`placement_scaling.rs`](../construction/placement_scaling.rs), [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) |
| Weighted footprint | `FootprintMatrix`, `SiteWeightedFootprint` |
| Parametric commit snapshot | [`parametric_commit.rs`](../construction/parametric_commit.rs) |
| Preview occupation | `ConstructionVisualRequests`, `FootprintTileWitness` |

### Audit checklist (Phase 3 exit)

| # | Verify | Pass when |
|:---:|:---|:---|
| S1 | Presets `1×1` … `12×12` (via scale_factor) | ghost + commit footprint cells match matrix |
| S2 | Occupied tiles | green/yellow/red footprint flags on map |
| S3 | Blocked tiles | `allows_commit == false`, invalid flag |
| S4 | Terrain mods required | witness or ghost legend token (mud/cut/fill) |
| S5 | Rotation + scale at commit | `BuildingScaleParams` on entity after commit |
| S6 | No widget resize as placement | invariant §15–16 |

**Witness:** extend `construction_parametric_placement_001` or add `construction_scaling_audit_001`.

**Designer:** legend + staged panel — [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md).

---

## Phase 4 — Placeholder art package

Designer-owned **generic kit** (roof / wall / interior / props families). Engineering attaches via catalog `BuildingDefinition` + Stage 5 `RepresentationResult` — not parallel mesh spawns.

| Deliverable | Owner |
|:---|:---|
| `construction_placeholder_art_kit_v1.md` | @designer |
| Catalog tags: `roof_*`, `wall_*`, `interior_*`, `prop_*` | JSON beside `assets/configs/buildings/` |
| LOD0 greybox mesh IDs | optional — after Phase 3 audit green |

---

## Phase 5 — Town / district hierarchy

```text
Building → Block → District → Town → Region → State → Nation
```

| Level | Suggested home | Persistence |
|:---|:---|:---|
| Building | `ConstructionSite` + footprint | ECS + save |
| Block | `BlockBook` (grid cluster id) | RON slice |
| District | `Zone` / district book | existing zone paint |
| Town | `Town` resource / book | new |
| Region+ | strategic books | worldgen + import |

```rust
// Target shape (strategic book — not ECS hot path)
pub struct Town {
    pub population: u32,
    pub jobs: u32,
    pub housing: u32,
    pub industries: Vec<SiteId>,
}
```

**Depends on:** Phase 2 operational sites (not instant Operational).

---

## Phase 6 — Organic settlement growth

**Drivers:** `LocalDemand { population, jobs, freight, wealth }` per district/block.

**Rule:** simulation **queues** `PlannedSite` / zoning proposals — player does not LMB each shop. Growth system calls same execute funnel when auto-build policy allows.

**Depends on:** Phase 5 hierarchy + economy activation spine.

---

## Phase 7 — Logistics & trade

Reuse [`economy/logistics/`](../economy/logistics/), transport snapshot, R8. `ResourceType` enum maps to existing flow nodes — extend, do not fork.

`LogisticsHub { throughput, storage }` → facility archetypes + depot components.

---

## Phase 8 — Real-world GIS (Natural Earth)

```text
Natural Earth → GIS import → World Generator → Bevy ECS
```

| Use | Crate / path |
|:---|:---|
| GeoJSON ingest | `geo`, `geojson`, serde |
| Heavy GDAL | optional feature — not default CI |
| Terrain | existing `terrain/generation/` |
| Admin boundaries | strategic books |

**Planner slice:** `plan_worldgen_gis_import_001` when Phase 5+ stable.

---

## Phase 9 — Military industrial

**Single production spine:** military factories = `BuildingDefinition` + recipes consuming `Steel`, `Electronics`, etc. No duplicate factory sim.

Command structure (Factory → Depot → Army Group → …) = **organizational book**, not second ECS economy.

---

## Phase 10 — Command & control

`HQ { command_range, bandwidth }` — overlay on logistics/comms graph. Orders propagate on graph edges; EW degrades bandwidth.

**Out of scope** until Phase 9 consumes civilian logistics.

---

## Phase 11 — Frontline warfare

`Front { sectors, supply_rating }` — operational map layer, not per-unit RTS selection. Shares supply chain with Phase 9 depots.

---

## Immediate planner slices (next 3 docs)

| Queue ID | Deliverable | Unblocks |
|:---|:---|:---|
| **PLAN-CONSTRUCTION-STAGE-PIPELINE-002** | [`plan_construction_stage_pipeline_exec_002_v1.md`](plan_construction_stage_pipeline_exec_002_v1.md) — CON-P2-001..003 | Coder Phase 2 |
| **PLAN-CONSTRUCTION-SCALING-AUDIT-003** | [`plan_construction_scaling_audit_exec_003_v1.md`](plan_construction_scaling_audit_exec_003_v1.md) | CON-P3-S1–S3, CON-P3-WIT | Phase 4, PG-2 |
| **PLAN-PROC-BUILD-EXEC-001** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) | PROC-BUILD-GEN-001 |
| **PLAN-ORGANIC-GROWTH-EXEC-001** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) | ORGANIC-GROWTH-001 |
| **PLAN-SETTLEMENT-HIERARCHY-005** | [`plan_settlement_hierarchy_exec_005_v1.md`](plan_settlement_hierarchy_exec_005_v1.md) | SET-P5-001..003 | Phase 6+, OG-4, INFRA-E5 |

**Architecture hub:** [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) · **Index:** [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md)

**Deferred product board:** optional Round 4 product policy — [`PLAN-CONSTRUCTION-R4-PRODUCT-OPEN-001`](fleet_wave4_assignments_20260527_v1.md) remains separate.

---

## Verification (construction lane)

```powershell
cargo test -p proc_A_dine01 --lib construction
# witness refresh when boards change:
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Align user Phases 2–10 to repo spine + execution order |
| v1.1.0 | 2026-06-02 | Signed; linked alignment hub + Phase 2 exec + INFRA program gates |
| v1.2.0 | 2026-06-02 | PLAN-SETTLEMENT-HIERARCHY-005 exec link in § Immediate slices |
| v1.3.0 | 2026-06-02 | PLAN-CONSTRUCTION-SCALING-AUDIT-003 exec link |
