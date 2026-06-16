# PLAN-SETTLEMENT-HIERARCHY-005 — Town / District / Block book exec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-SETTLEMENT-HIERARCHY-005** |
| **Parent** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) § Phase 5 |
| **Alignment** | [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) — **G-TOWN-ONE** |
| **Designer input** | [`design_settlement_hierarchy_read_v1.md`](design_settlement_hierarchy_read_v1.md) **PASS** |
| **Growth UX** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) |
| **Organic exec** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) — OG-1..4 consume these books |
| **Infrastructure gate** | **INFRA-E5-001** attaches `SettlementNode` by id only — no duplicate `Town` resource |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` → `@coder A` / `@coder B` |
| **Status** | **SIGNED — READY** |
| **Horizon** | **2–3 weeks** (3 PRs) |

**Hard rules:** One **`TownBook`** authority in `strategic/settlement/`. No second `Town` type in `infrastructure/`. No `ConstructionStage` enum. Zoning paint (`ZoneTool`) writes **mask only** — district picker reads books, does not mutate zone paint.

---

## 1. Problem

Product needs a stable hierarchy for organic growth, logistics, and player navigation:

```text
Building (ConstructionSite) → Block → District → Town → Region+
```

Today:

| Piece | Location | Gap |
|:---|:---|:---|
| Sites | `ConstructionSite` + footprint | No block/district id on commit |
| Zone paint | `ZoneTool` → `PendingEntryKind::ZonePaint` | Mask only — no named district book |
| Influence zones | `strategic::Zone` (Supply/Fire/…) | **Not** urban districts — do not overload |
| Runbook stub | `SettlementSite` in `strategic/sim.rs` | Coarse pop entity — **not** book authority |
| Tier label | `SettlementTier::Town` in runbook | Display tier — **not** `TownId` |

Without books, OG-1 pressure and INFRA-E5-001 settlement nodes cannot share ids.

---

## 2. Out of scope (this plan)

| Item | Where |
|:---|:---|
| Region / State / Nation persistence | GIS Phase 8 |
| Instant building spawn from district | OG-2 exec |
| Duplicate `Town` in `infrastructure/settlement/` | **Forbidden** — G-TOWN-ONE |
| Replacing `strategic::Zone` influence fields | Unrelated lane |
| New top-level `ConstructionStage` | **Forbidden** — G-PHASE-ONE |
| Full district boundary editor | Designer picker only v1 |

---

## 3. Target hierarchy (v1 minimal)

```text
TownBook
  └── TownRecord { id, name, center_tile, population, jobs, housing }
        └── DistrictRecord { id, name, town_id, bounds, zoning_default }
              └── BlockRecord { id, district_id, tile_set, site_ids[] }
                    └── ConstructionSite (entity) — optional BlockId component
```

**Region+:** schema stub `RegionId` in RON only; no sim writer until GIS import.

---

## 4. Types (sim authority)

**Home:** `src/strategic/settlement/` (new module — OG exec already reserves this path).

```rust
// ids.rs
pub struct TownId(pub String);
pub struct DistrictId(pub String);
pub struct BlockId(pub String);
pub struct RegionId(pub String); // persist only v1

// town.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TownRecord {
    pub id: TownId,
    pub name: String,
    pub center_tile: IVec2,
    pub population: u32,
    pub jobs: u32,
    pub housing: u32,
}

#[derive(Resource, Default)]
pub struct TownBook {
    pub towns: HashMap<TownId, TownRecord>,
    pub default_town: Option<TownId>,
}

// district.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistrictRecord {
    pub id: DistrictId,
    pub town_id: TownId,
    pub name: String,
    /// Inclusive tile rect or explicit tile list — v1: `tile_rect: IRect`
    pub tile_rect: IRect,
    pub zoning_default: ZoningClass,
    /// Procedural rules — allowed archetypes/roofs/style (not mesh generation).
    pub style_rules: DistrictStyleRules,
}

#[derive(Resource, Default)]
pub struct DistrictBook {
    pub districts: HashMap<DistrictId, DistrictRecord>,
}

// block.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRecord {
    pub id: BlockId,
    pub district_id: DistrictId,
    pub tiles: HashSet<IVec2>,
    pub site_ids: Vec<SiteId>,
}

#[derive(Resource, Default)]
pub struct BlockBook {
    pub blocks: HashMap<BlockId, BlockRecord>,
    pub tile_to_block: HashMap<IVec2, BlockId>,
}

// zoning.rs — separate from strategic::Zone influence fields
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ZoningClass {
    Residential,
    Commercial,
    Industrial,
    MixedUse,
    Civic,
    Rural,
}
```

**Component (optional v1):** `DistrictMembership { district_id, block_id }` on `ConstructionSite` entity after commit.

**Zoning mask (v1):** `ZoningMaskBook` or tile map `HashMap<IVec2, ZoningClass>` written by zone paint execute — **does not** create district rows.

---

## 5. RON layout (default assets)

| File | Content |
|:---|:---|
| `assets/config/settlement/default_town.ron` | One town + 1–3 districts for DefaultIndustrial / Portland |
| `assets/config/settlement/blocks_{district_id}.ron` | Optional split if block count large |

**Loader:** extension dispatch RON first (see AGENTS.md serialization policy).

**Default play scenario:** seed loads `default_town.ron` — no harness-only town spawn.

**District rules RON:** optional sidecar per district — schema [`tools/mcp/schemas/district_style_rules_v1.schema.json`](../../tools/mcp/schemas/district_style_rules_v1.schema.json). Example industrial district:

```json
{
  "schema_version": 1,
  "district_id": "north_industrial",
  "zoning_default": "industrial",
  "allowed_archetypes": ["warehouse", "factory", "storage_yard"],
  "allowed_roof_modules": ["roof_flat", "roof_sawtooth"],
  "preferred_style_pack": "style_industrial_west",
  "style_tags": ["railway_district"]
}
```

OG-2 filters `GrowthProposal` against `style_rules` before queue insert.

---

## 6. Relationship to existing types

| Existing | Action |
|:---|:---|
| `SettlementSite` (`strategic/sim.rs`) | **Do not delete** v1 — add `town_id: Option<TownId>` or document as legacy runbook; rollup reads `TownBook` for inspector header |
| `strategic::Zone` | Unchanged — influence fields |
| `ZoneTool` / zone paint | Maps to `ZoningClass` on tile mask; **does not** open district inspector (designer rule) |
| `SettlementTier::Town` | Display only — map from `TownRecord.population` when needed |
| OG-1 `DistrictMetrics` | Keyed by `DistrictId` from `DistrictBook` |

---

## 7. INFRA-E5-001 attachment (Coder B — later)

When this exec is green, infrastructure implements **only**:

```rust
// infrastructure/settlement/mod.rs — NOT a second TownBook
pub struct SettlementNode {
    pub id: SettlementId,
    pub town_id: TownId,           // from strategic/settlement
    pub kind: SettlementNodeKind,  // Town | Port | Depot
    pub world_tile: IVec2,
    pub attached_transport_nodes: Vec<TransportNodeId>,
}
```

**Gate G-TOWN-ONE:** `attach_settlement_to_nearest_transport_node(town_id, …)` reads `TownBook` — never inserts parallel population/jobs fields on `SettlementNode`.

---

## 8. PR train

### SET-P5-001 — `TownBook` + `DistrictBook` (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `src/strategic/settlement/mod.rs`, `town.rs`, `district.rs`, `assets/config/settlement/default_town.ron` |
| **Plugin** | `SettlementBookPlugin` — load on `BaseState::Simulation` enter |
| **Tests** | `default_town_ron_loads_one_district` |
| **Blocked by** | — (schema may land before CON-P2-003) |
| **Exit** | `TownBook.default_town` Some; `DistrictBook` len ≥ 1 |

### SET-P5-002 — `BlockBook` + site linkage (Coder B)

| Item | Detail |
|:---|:---|
| **Files** | `src/strategic/settlement/block.rs`, `src/strategic/settlement/assign.rs`, `strategic/site/components.rs` (optional `BlockId`) |
| **Logic** | `assign_block_for_tile(tile) -> BlockId` — grid cluster v1: `block_id = district + (tx/8, ty/8)` |
| **On commit** | `commit_construction_site_system` registers `SiteId` in block's `site_ids` |
| **Tests** | `three_sites_same_block_after_portland_chain` |
| **Blocked by** | SET-P5-001 |
| **Exit** | `tile_to_block` populated for committed site tiles |

### SET-P5-003 — Witness + save slice + play seed (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `construction/live_proof.rs`, `src/io/snapshot/` or transport book pattern, `engine/play_scenario.rs` |
| **Witness key** | `construction_settlement_hierarchy_001` in `debug_runs/construction_stage_live.json` |
| **Save** | RON slice `settlement_books` in hybrid snapshot header (round-trip test) |
| **Tests** | `simulation_writes_construction_settlement_hierarchy_witness` |
| **Blocked by** | **CON-P2-003** green on disk (`construction_site_stage_pipeline_001.green`) |
| **Exit** | Witness green; `g_town_one: true`; play scenario loads town without env seed |

---

## 9. Witness schema

| Pointer | Type | Pass when |
|:---|:---|:---|
| `/construction_settlement_hierarchy_001/gate` | string | `SET-P5-003` |
| `/construction_settlement_hierarchy_001/green` | bool | `true` |
| `/construction_settlement_hierarchy_001/town_book_loaded` | bool | `true` |
| `/construction_settlement_hierarchy_001/district_count` | number | `≥ 1` |
| `/construction_settlement_hierarchy_001/block_assignment_wired` | bool | `true` |
| `/construction_settlement_hierarchy_001/site_to_block_wired` | bool | `true` |
| `/construction_settlement_hierarchy_001/g_town_one` | bool | `true` — grep: no `struct Town` in `infrastructure/` |
| `/construction_settlement_hierarchy_001/save_roundtrip_ok` | bool | `true` |

**Disk truth:** witness keys absent or `green: false` → slice **open** regardless of markdown.

---

## 10. Designer wiring (no Rust in designer doc)

| Surface | Book field |
|:---|:---|
| District picker list | `DistrictRecord.name` |
| Inspector header | `TownRecord.name`, pop/jobs rollup |
| Proposal card “Block 12” | `BlockId` display alias |
| Zone paint | `ZoningClass` on mask only |

See [`design_settlement_hierarchy_read_v1.md`](design_settlement_hierarchy_read_v1.md).

---

## 11. Unblocks

| Downstream | Slice |
|:---|:---|
| **PROC-OG-1-001** | `DistrictMetrics` keyed by `DistrictId` |
| **PROC-OG-4-001** | Town rollup from `TownBook` |
| **INFRA-E5-001** | `SettlementNode.town_id` attachment |
| **INFRA-E5-002** | Town → port routes reference graph + town id |
| CON P6 organic growth | Phase 6 after P5 witness |

---

## 12. Anti-patterns

- Second `Town` resource in `infrastructure/`
- Zone paint opening district inspector or mutating `DistrictBook`
- Overloading `strategic::Zone` as urban district
- Instant `Operational` sites to “seed” town population
- Hand-editing witness JSON without lib refresh test

---

## 13. Regression

```powershell
cargo test -p proc_A_dine01 --lib settlement construction
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
# after SET-P5-003:
cargo test -p proc_A_dine01 --lib construction_settlement_hierarchy
```

---

## 14. Coder handoff

| Field | Value |
|:---|:---|
| **Machine IDs** | **SET-P5-001** (A) · **SET-P5-002** (B) · **SET-P5-003** (A) |
| **Pull after** | CON-P2-001 for site linkage; **CON-P2-003** for SET-P5-003 witness |
| **Parallel OK** | SET-P5-001 with INFRA-E0-001 (disjoint files) |
| **Do not** | Reopen SIGNED P2/PROC/ORGANIC exec rows |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | G-TOWN-ONE; 3 PRs; designer PASS input; INFRA-E5-001 gate |
