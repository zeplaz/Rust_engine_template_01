# Procedural buildings + organic growth — program index `v1`

| Field | Value |
|:---|:---|
| **ID** | **CONSTRUCTION-PROC-GROWTH-INDEX-001** |
| **Date** | 2026-06-02 |
| **Architecture hub** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| **Alignment** | [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) |
| **Product roadmap** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) v1.1 |
| **Coder pull** | [`coder_unified_backlog_v1.md`](coder_unified_backlog_v1.md) § Horizon (after P2) |

**North star:** One **archetype + style pack + grammar** stack — not 500 JSON houses. Growth **never** instant-spawns operational sites; same execute funnel as player builds.

**Economy vision (state factories + private infill + market niches):** [`construction_economy_growth_vision_v1.md`](construction_economy_growth_vision_v1.md)

**MCP (consumers vs builders):** [`agent_mcp_consumer_guide_v1.md`](agent_mcp_consumer_guide_v1.md)

---

## Deliverables map

| Queue ID | Doc | Owner | Status |
|:---|:---|:---|:---:|
| **CONSTRUCTION-PROC-GROWTH-001** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) | @planner | **SIGNED** |
| **PLAN-PROC-BUILD-EXEC-001** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) | @planner → @coder | **SIGNED** |
| **PLAN-ORGANIC-GROWTH-EXEC-001** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) | @planner → @coder | **SIGNED** |
| **PLAN-SETTLEMENT-HIERARCHY-005** | [`plan_settlement_hierarchy_exec_005_v1.md`](plan_settlement_hierarchy_exec_005_v1.md) | @planner → @coder | **SIGNED** |
| **PLAN-ECON-GROWTH-ACTORS-001** | [`plan_econ_growth_actors_exec_001_v1.md`](plan_econ_growth_actors_exec_001_v1.md) | @planner → @coder B | **SIGNED** |
| **PLAN-CONSTRUCTION-SCALING-AUDIT-003** | [`plan_construction_scaling_audit_exec_003_v1.md`](plan_construction_scaling_audit_exec_003_v1.md) | @planner → @coder | **SIGNED** |
| **PLAN-ART-DESIGN-INBOUND-ALIGN-001** | [`plan_art_design_inbound_alignment_v1.md`](plan_art_design_inbound_alignment_v1.md) | @planner | **SIGNED** |
| **PLAN-CONSTRUCTION-STAGE-PIPELINE-002** | [`plan_construction_stage_pipeline_exec_002_v1.md`](plan_construction_stage_pipeline_exec_002_v1.md) | @planner → @coder | **SIGNED** |
| **PLAN-PROC-TILE-PROD-001** | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) | @planner → designer-mcp / coder-mcp / coder | **ACTIVE** (PT-0 **SIGNED**) |
| **PLAN-BUILDING-GRAMMAR-001** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) | @planner-mcp → @coder / @coder-mcp / @designer-mcp | **ACTIVE** (hierarchical grammar + APS authoring) |
| **DESIGN-PROC-MODULE-KIT-001** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) | @designer | **PASS** (2026-06-02) |
| **DESIGN-ORGANIC-GROWTH-UX-001** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) | @designer | **PASS** (2026-06-02) |
| **DESIGN-CONSTRUCTION-STAGE-READ-001** | [`design_construction_site_stage_read_v1.md`](design_construction_site_stage_read_v1.md) | @designer | **PASS** |
| **DESIGN-CONSTRUCTION-SCALING-READ-001** | [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) | @designer | **PASS** |
| **DESIGN-INFRA-NETWORK-OVERLAY-001** | [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) | @designer | **PASS** |
| **DESIGN-SETTLEMENT-HIERARCHY-READ-001** | [`design_settlement_hierarchy_read_v1.md`](design_settlement_hierarchy_read_v1.md) | @designer | **PASS** |

**Artist brief (one line):** 10 modules per category (wall/window/door/roof/prop), not 200 buildings; 7 style packs.

---

## Concept → repo today → plan

| Your concept | Repo today | Plan slice |
|:---|:---|:---|
| `BuildingArchetype` | `BuildingDefinition` + `BuildingFamily` | **PG-1** RON loaders + alias migration |
| `BuildingUsage` | `BuildingFamily` / `SiteArchetype` | **PG-1** map 1:1 |
| Module assembly | `_mock_shapes.ron` (footprint only) | **PG-2** grid + **lod0/production** mesh extract → Stage 5 |
| **Building iso (product)** | lod0 pilot atlases (4) | **PLAN-PROC-TILE-PROD-001** — production bakes + sim→`variant_key` + fire frames |
| Zone + infra → sim builds | `ZoneTool`; no growth sim | **OG-1/2** queue only |
| District hierarchy | not yet | **OG-1** books + **OG-4** Town rollup |
| Style packs | N/A | Designer RON + **PG-2** |

**Hard invariant (unchanged):** growth and procedural gen → validation → `ConstructionPlanQueue` → commit → `SiteConstructionPhase` — no `Operational` on commit.

---

## Recommended build order

| Step | Lane | Queue IDs |
|:---:|:---|:---|
| 1 | Placement validation | CON P1 (mostly closed) |
| 2 | Staged construction pipeline | **CON-P2-001** (A) · **CON-P2-002** (B) · **CON-P2-003** (A) |
| 3 | Scaling audit | **CON-P3-S1–S3** (A) · **S4–S6 done** (B) · **CON-P3-WIT** |
| **4a** | Designer module kit | **DESIGN-PROC-MODULE-KIT-001** (parallel with 4b) |
| **4b** | Coder archetypes | **PG-1** (`PROC-PG-1-001` A) |
| 5 | District / Town books | **SET-P5-001** (A) · **SET-P5-002** (B) · **SET-P5-003** (A) |
| **6a** | Coder pressure + proposals | **OG-1**, **OG-2** (B) |
| **6b** | Designer growth UX | **DESIGN-ORGANIC-GROWTH-UX-001** |
| 7 | Module assembly + approve UI | **PG-2**, **OG-3** |
| 8 | Grammar depth (optional) | **PG-4** |

**Implement next (coders):** See [`fleet_coder_workload_queue_20260602_v1.md`](fleet_coder_workload_queue_20260602_v1.md) · snapshot [`fleet_snapshot_20260602_v3.md`](fleet_snapshot_20260602_v3.md). **SET-P5 + ECON-OG + PROC-OG closed on disk (2026-06-02).** **A:** CON-P3-S1..WIT → infra column → PG-2 tail. **B:** partial-alpha + procedural test regressions → infra E4/E5 tails.

---

## Coder slices (machine IDs)

| Slice | ID | Owner default | Exec |
|:---|:---|:---|:---|
| PG-1 | **PROC-PG-1-001** | Coder A | Archetype + StylePack RON |
| PG-2 | **PROC-PG-2-001** | Coder A | Footprint grid + module extract (lod0/production only) |
| PG-3 | **PROC-PG-3-001** | Coder B | Commit bridge metadata |
| PG-4 | **PROC-PG-4-001** | Coder A | Shape grammar (later) |
| OG-1 | **PROC-OG-1-001** / **ECON-OG-1-*** | Coder B | `DistrictMetrics` + `MarketSaturation` + actors — [`plan_econ_growth_actors_exec_001_v1.md`](plan_econ_growth_actors_exec_001_v1.md) |
| OG-2 | **PROC-OG-2-001** | Coder B | Growth proposals → queue |
| OG-3 | **PROC-OG-3-001** | Coder B | Policy + approve UI |
| OG-4 | **PROC-OG-4-001** | Coder A | Town rollup |
| SET-P5 | **SET-P5-001** | Coder A | TownBook + DistrictBook |
| SET-P5 | **SET-P5-002** | Coder B | BlockBook + site linkage |
| SET-P5 | **SET-P5-003** | Coder A | Witness + save slice |

Witness targets (future): `construction_procedural_build_001`, `construction_organic_growth_001` in `construction_stage_live.json`.

---

## PG-2 mesh authority (hard rule — 2026-06-02)

**Policy:** [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md)

| Source | PG-2 may use? | Notes |
|:---|:---:|:---|
| `_module_index.ron` row with `development_tier: production` | **Yes** | Default StylePack path |
| `_module_index.ron` row with `development_tier: lod0` | **Yes** | Explicit opt-in per StylePack or assembly profile |
| `_module_index.ron` row with `development_tier: smoke` | **No** | Includes all `kit_greybox_*` legacy rows |
| MCP cube GLBs without tier migration | **No** | Treat as smoke until re-tiered |
| Engine primitive / footprint-only fallback | **Yes** | When no lod0+ row — **not** silent smoke GLB |

**PROC-PG-2-001 acceptance addendum:**

- Assembly code calls `ProceduralModuleRegistry::modules_for_stylepack()` (or equivalent) — **never** iterates all index rows.
- Witness records `mesh_tier_used: lod0 | production | fallback_primitive` per assembled site.
- **Unblocks PG-2** without waiting for 50 production modules — parallel **`kit_lod0_001`** batch (5 modules).

**Artist brief correction:** PG-2 needs **silhouette-correct lod0**, not MCP smoke cubes.

---

## Registry tier filter API (MCP-E0-001 / MCP-PLN-001)

**Implementer:** `@coder` · **Consumer:** PROC-PG-2-001, `BuildingDefinitionRegistry::procedural_glb_asset`, StylePack assembly.

### Policy

Promoted GLBs in `_module_index.ron` include **smoke** rows (`kit_greybox_*`). The engine must **not** treat every index row as StylePack-eligible mesh. PG-2 assembly uses **lod0 + production** silhouettes only; **smoke is witness inventory**.

### Index fields (parse into `ProceduralModuleEntry`)

| Field | Type | Default if missing |
|:---|:---|:---|
| `development_tier` | `smoke` \| `lod0` \| `production` | infer `smoke` if `batch_id` starts with `kit_greybox` else `lod0` |
| `stylepack_visible` | bool | `false` if tier==smoke else `true` |
| `replaced_by` | optional `module_id` | — |
| `pbr_status` | `none` \| `deferred` \| `shipped` | `none` |

### Public API (`ProceduralModuleRegistry`)

| Method | Purpose |
|:---|:---|
| `get(module_id)` | Raw lookup — **debug / admin only** |
| `modules_for_stylepack()` | Iterator over entries where `stylepack_visible == true` AND `development_tier != smoke` |
| `modules_for_assembly()` | Same as stylepack today; alias for PG-2 extract — **lod0 ∪ production** |
| `resolve_module_id(id)` | If `get(id)` is smoke and `replaced_by` set, follow to replacement row (lod0/production) |

### Resolution rules

1. **StylePack** and **PG-2 extract** call `modules_for_stylepack()` or `resolve_module_id` — never iterate `by_module_id` unfiltered.
2. **PG-2 may use `lod0`** meshes for silhouette assembly and sim readability.
3. **PG-2 must never use `smoke`** — even if it is the only row for a legacy alias id.
4. **Missing module** (no lod0+ row for canonical id): use **engine primitive footprint extrusion or hide slot** — **do not** fall back to smoke GLB.
5. **`procedural_glb_asset`:** resolve via `resolve_module_id` → `modules_for_stylepack()` entry only; return `None` if only smoke exists (caller applies primitive/hide).

### Tests (minimum)

- Smoke row excluded from `modules_for_stylepack()`.
- `wall_brick_1u` with smoke + lod0 rows: `resolve_module_id` returns **lod0** job after MCP-P0-001 `replaced_by`.
- `get("corner_brick_outer")` may still return smoke for diagnostics; assembly path must not use it.

### Witness

After E0-001: extend `debug_runs/art_pipeline/kit_lod0_001_live.json` `_agent_meta.engine_registry` or add `debug_runs/art_pipeline/mcp_engine_bridge_live.json` with `{ "modules_for_stylepack_count", "smoke_excluded_count" }`.

**Roadmap:** [`plan_kit_lod0_roadmap_v1.md`](plan_kit_lod0_roadmap_v1.md) · **Tier policy:** [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md)

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib construction
# future:
cargo test -p proc_A_dine01 --lib procedural_build organic_growth
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Index + queue wiring for planner proc/growth batch |
| v1.1.0 | 2026-06-02 | Designer six-phase long-run PASS — all design rows closed |
| v1.2.0 | 2026-06-02 | Economy growth vision + MCP consumer guide links |
| v1.2.0 | 2026-06-02 | PLAN-SETTLEMENT-HIERARCHY-005 signed; SET-P5-001..003 IDs |
| v1.3.0 | 2026-06-02 | PLAN-CONSTRUCTION-SCALING-AUDIT-003 signed; Phase 3 active |
| v1.4.0 | 2026-06-02 | PG-2 mesh authority — smoke GLBs not authoritative; lod0/production only |
| v1.5.0 | 2026-06-02 | MCP-PLN-001 — Registry tier filter API for MCP-E0-001 |
