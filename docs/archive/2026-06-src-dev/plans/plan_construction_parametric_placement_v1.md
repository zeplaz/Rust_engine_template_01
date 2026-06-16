# PLAN-CONSTRUCTION-PARAM-001 — Parametric placement implementation plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-PARAM-001** |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) (**CONSTRUCTION-PARAM-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Signed** | 2026-05-26 |
| **Coder lanes** | **CONSTRUCTION-PARAM-CODER-001…006** (below) |
| **Designer lane** | **CONSTRUCTION-PARAM-DESIGN-001** (tray + staged panel mocks) |
| **Witness** | `debug_runs/construction_stage_live.json` |

**Decisions locked (user 2026-05-26):**

1. **Stage placements OFF** → **Enter commits current ghost** (not LMB instant build).
2. **Weighted footprint from day one** — no integer-only MVP; fractional occupation is authoritative.

---

## Authority map

```text
ActiveBuildTool + catalog intent
  → BuildGhostState { origin, scale_factor, rotation, mirror }
  → weighted_footprint_raster(base FootprintMatrix, params) → WeightedFootprint
  → evaluate_weighted_site_placement → BuildPlacementPreview
  → [optional] StagedGhostBook.push(snapshot)
  → CommitConstructionSiteEvent { catalog_id, placement_params, weighted_footprint }
  → commit_construction_site_system → SiteWeightedFootprint + PlannedSite
  → activate_industrial_facilities_system → scaled rates via BuildingScaleParams
```

| Domain | Authority | Must not |
|:---|:---|:---|
| Ghost / preview | `src/construction/*` | Mutate sites, zones, transport |
| Weight raster | `src/construction/weighted_footprint.rs` (new) | Duplicate in egui vs sim |
| Tile occupation book | `TileOccupationBook` resource (committed weights) + `SiteWeightedFootprint` per site | Infer from egui widget size; **do not** write `ChunkStrategicOverlay` for collision |
| Commit | `CommitConstructionSiteEvent` → `commit_construction_site_system` | Side-door spawns |
| Economy rates | `economy/activation` + `BuildingScaleParams` | Parallel catalog rows per size |
| Staging UI | `src/construction/staged_ghost_panel.rs` (new) + tray | Hidden Shift shortcuts |

---

## Data model

### `PlacementParams` (construction + strategic)

```rust
/// Authoritative parametric placement — committed with site.
pub struct PlacementParams {
    pub scale_factor: f32,           // player drag, clamped
    pub effective_scale: f32,          // derived s_eff from weights
    pub rotation_quarter_turns: u8,  // 0..3
    pub mirror_x: bool,
}
```

### `WeightedFootprint`

```rust
pub struct WeightedFootprint {
    pub origin: BuildSiteTile,
    pub weights: Vec<(IVec2, f32)>,  // sparse, w in [0,1]
    pub bounds: FootprintTiles,      // axis-aligned envelope for broadphase
}
```

### Extend events / components

| Type | Additions |
|:---|:---|
| `CommitConstructionSiteEvent` | `placement: PlacementParams`, `weighted: WeightedFootprint` |
| `PendingBuildBlueprint` | Same fields; rename UX to `StagedGhostEntry` (alias OK) |
| `PlannedSite` | `placement`, `weighted` |
| `SiteFootprint` | Keep `tiles: Vec<IVec2>` for envelope OR migrate to weighted component |
| `BuildingDefinitionRef` | unchanged `catalog_id` |
| **New** `BuildingScaleParams` | `{ scale_factor, effective_scale, k_prod, ... cached or ref catalog }` |
| **New** `SiteWeightedFootprint` | authoritative weights post-commit |

### Catalog schema extension (optional RON/JSON per building)

```json
"placement_scaling": {
  "scale_min": 0.35,
  "scale_max": 2.75,
  "k_prod": 0.90,
  "k_exp": 1.00,
  "k_capex": 1.10,
  "k_risk": 1.35,
  "k_detect": 0.70,
  "fixed_overhead": 12.0,
  "min_occupied_mass": 0.55
}
```

Fallback: family defaults in `src/construction/placement_scaling.rs`.

---

## Weighted footprint raster (deterministic)

**File:** `src/construction/weighted_footprint.rs`

**Algorithm:** `rasterize_weighted_footprint(base: &FootprintMatrix, params: &PlacementParams, origin: BuildSiteTile) -> WeightedFootprint`

1. Build transform: origin + rotation + mirror + uniform scale about base centroid.
2. Compute world-space AABB in tile indices (ceil bounds).
3. For each tile in AABB, sample **4×4 subcell grid** (16 samples) — count fraction inside occupied base cells.
4. `weight = inside_count / 16.0`; omit tiles with `weight < 0.01`.
5. `effective_scale = sum(weights) / base_occupied_cell_count`.

**Properties:**

- Deterministic (fixed sample grid).
- Continuous in `scale_factor` (no integer snap).
- Same function used by preview validation and commit.

**Overlap query:**

```text
for (tile, w_new) in new.weights:
  w_exist = occupation_book.weight_at(tile)  // sum of committed + staged-if-preview
  if w_exist + w_new > 1.0 + EPS → error overlap
```

**Occupation book:** [`TileOccupationBook`](../strategic/site/tile_occupation.rs) **Resource** (see **Planner decisions** below) — single writer on commit/demolish; staged ghosts use **preview scratch** only (preview never writes book).

---

## Input systems (replace blueprint queue bindings)

| System | Change |
|:---|:---|
| `build_pick_ghost_tile_system` | LMB anchor; if `StagingMode::On` && valid → push staged snapshot |
| **NEW** `build_scale_ghost_drag_system` | Shift held + mouse delta Y → Δscale |
| **NEW** `build_rotate_ghost_ctrl_scroll_system` | Ctrl+scroll → rotate; keep R/X |
| `build_queue_blueprint_on_shift_click_system` | **Remove** or no-op building branch |
| `build_confirm_site_system` | Enter: immediate commit OR drain approved staged |
| `build_tool_authority` | Remove `shift_lmb_queues_building_blueprint` for buildings |

**Resource:**

```rust
pub enum ConstructionStagingMode { Off, On }
#[derive(Resource)]
pub struct ConstructionStagingSettings { pub mode: ConstructionStagingMode }
```

---

## Economy integration

**File:** `src/economy/activation/scale.rs` (new)

```rust
pub fn scaled_production(base: f32, s: f32, k_prod: f32) -> f32 {
    base * s.powf(k_prod)
}
pub fn scaled_expense(base: f32, s: f32, k_exp: f32, fixed: f32) -> f32 {
    base * s.powf(k_exp) + fixed
}
```

**Hook:** `activate_industrial_facilities_system` — when inserting supply chain runtime, pass `effective_scale` and exponents into `insert_supply_chain_runtime_for_catalog` (extend signature or wrap rates after insert).

**Construction manifest:** `site_advance_planned_to_under_construction_system` seeds `SiteResourceManifest` with `capex_mult(s)`.

**Detection / sound:** apply `detect_mult(s)` to `sound_emission` / `detection_multiplier` from JSON at activation (presentation + sim stub).

---

## UI implementation split

| ID | Owner | Deliverable |
|:---|:---|:---|
| **CONSTRUCTION-PARAM-DESIGN-001** | `@designer` | Tray toggle mock, staged list columns, hint strings, partial-alpha ghost spec |
| **CONSTRUCTION-PARAM-CODER-001** | `@coder` | `weighted_footprint.rs` + tests (raster, overlap, s_eff) |
| **CONSTRUCTION-PARAM-CODER-002** | `@coder` | Ghost state + scale/rotate input; deprecate Shift queue |
| **CONSTRUCTION-PARAM-CODER-003** | `@coder` | Commit path + `SiteWeightedFootprint` + event extensions |
| **CONSTRUCTION-PARAM-CODER-004** | `@coder` | Staged panel egui + Build approved/all |
| **CONSTRUCTION-PARAM-CODER-005** | `@coder` | Visual authority partial-alpha tiles |
| **CONSTRUCTION-PARAM-CODER-006** | `@coder` | Economy scale activation + catalog scaling defaults |

---

## Phased delivery

### Phase 1 — Weighted footprint spine (coder 001 + 003 partial)

- Rasterizer + validation
- `CommitConstructionSiteEvent` carries weighted footprint
- `SiteWeightedFootprint` on commit
- Witness: raster golden tests + commit roundtrip

**Exit:** `cargo test -p proc_A_dine01 --lib construction::weighted_footprint`

### Phase 2 — Input + ghost UX (coder 002 + 005)

- Scale drag, Ctrl rotate
- Remove Shift+LMB building queue
- Enter commits single ghost (staging OFF)
- Partial-alpha overlay

**Exit:** manual sim playtest + construction live proof green

### Phase 3 — Staging panel (coder 004 + design 001)

- Toggle + list + Build approved/all
- Migrate `PendingConstructionQueue` UX strings

### Phase 4 — Economy + tradeoffs (coder 006)

- Scaled production/expense at activation
- HUD preview readout
- Risk index (display-only v1; sim hook v1.1)

---

## Witness schema (`construction_parametric_placement_001`)

Add to `debug_runs/construction_stage_live.json`:

```json
"construction_parametric_placement_001": {
  "gate": "CONSTRUCTION-PARAM-001",
  "weighted_raster_tests_green": true,
  "shift_queue_building_removed": true,
  "enter_commits_single_ghost": true,
  "staging_toggle_wired": true,
  "build_approved_drains_staged": true,
  "overlap_blocks_commit": true,
  "commit_carries_scale_and_weights": true,
  "economy_scales_at_activation": true,
  "green": false
}
```

Rollup `green` when all booleans true in sim proof writer.

---

## Files touched (expected)

| Path | Action |
|:---|:---|
| `src/construction/weighted_footprint.rs` | **NEW** |
| `src/construction/placement_scaling.rs` | **NEW** — curve defaults |
| `src/construction/build_state.rs` | Add `scale_factor`, staging settings |
| `src/construction/build_interaction.rs` | Input rewrite |
| `src/construction/pending_construction.rs` | Extend entry + rename in UI |
| `src/construction/staged_ghost_panel.rs` | **NEW** |
| `src/construction/visual_authority.rs` | Weighted alpha tiles |
| `src/construction/tool_hints.rs` | New hint strings |
| `src/construction/build_tool_authority.rs` | Conflict matrix |
| `src/construction/mod.rs` | Register systems |
| `src/strategic/site/events.rs` | Event fields |
| `src/strategic/site/tile_occupation.rs` | **NEW** — `TileOccupationBook` resource |
| `src/strategic/site/components.rs` | `SiteWeightedFootprint`, `BuildingScaleParams` |
| `src/strategic/site/systems.rs` | Commit spawn |
| `src/strategic/site/validation.rs` | Weighted overlap |
| `src/economy/activation/bridge.rs` | Scaled activation |
| `src/economy/activation/scale.rs` | **NEW** |
| `src/construction/live_proof.rs` | Witness block |
| `src/dev/construction_invariants.md` | §15 weighted occupation |
| `assets/configs/buildings/*.json` | Optional `placement_scaling` blocks |

---

## Tests (required)

| Test | Proves |
|:---|:---|
| `weighted_footprint_scale_monotonic` | Larger scale → ≥ occupied mass |
| `weighted_footprint_rotation_preserves_mass` | s_eff invariant ±ε |
| `weighted_footprint_overlap_rejects` | Σw > 1 fails validation |
| `staged_drain_only_approved` | Existing pending test pattern |
| `commit_roundtrip_carries_scale` | Event → component |
| `economy_scale_non_unity` | s=1.5 → production ≠ base |

---

## Risks / mitigations

| Risk | Mitigation |
|:---|:---|
| Performance (many weighted tiles) | Sparse storage; broadphase via bounds |
| Legacy `FootprintTiles` callers | Envelope field + adapter `to_legacy_footprint()` for road/minimap until migrated |
| Shift key clash with map pan fast modifier | Scale drag only when build tool active + ghost origin Some |
| Rotation not in commit today | Phase 1 fixes with weighted raster (rotation in params) |

---

## Planner decisions (signed 2026-05-26)

| Decision | Resolution | Rationale |
|:---|:---|:---|
| **`TileOccupationBook` owner** | **New `TileOccupationBook` `Resource`** in `src/strategic/site/tile_occupation.rs` | **Single writer:** `commit_construction_site_system` (+ demolish undo). **Readers:** `src/construction/weighted_footprint.rs` overlap + validation. **Do not** extend [`ChunkStrategicOverlay`](../strategic/mod.rs) — overlay SOA is logistics/recon/zone flow with multiple writers; weighted collision is gameplay occupation, not a flow field. |
| **Per-site storage** | `SiteWeightedFootprint` **component** on committed sites (audit + demolish) | Book holds aggregated Σw; component holds authoritative sparse weights for the site entity. |
| **Preview layer** | `PreviewOccupationScratch` in `src/construction/` (ephemeral) | Staged ghosts + active ghost overlap checks; **never** mutates `TileOccupationBook`. |
| **Raster algorithm** | **Approved: 4×4 subcell grid (16 samples/tile)** | Deterministic, continuous in `scale_factor`, shared preview/commit path. Alternatives (analytic area) deferred — do not mix algorithms. |
| **Enter vs Shift (buildings only)** | **Staging OFF:** **Enter** commits current valid ghost (no LMB instant build). **Remove** Shift+LMB blueprint queue + **Shift+Enter** batch-approve for `BuildTool::Building`. **Staging ON:** LMB adds staged snapshot; **Build approved / Build all valid** drain queue. **Shift+drag** = scale (buildings only, ghost origin `Some`). Roads / rail / zone / demolish: **unchanged** Shift semantics (`build_tool_authority` per tool). | Matches product spec + frees Shift for scale; [`build_confirm_site_system`](../construction/build_interaction.rs) today commits on Enter **and** uses Shift+Enter approve-all — building branch must drop the latter. |

**Exec PR slices:** [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md)

---

## Planner checklist

- [x] Sign authority map (no dual occupation writers)
- [x] Confirm `TileOccupationBook` owner → **new resource, commit/demolish writer only**
- [x] Approve **4×4 subcell** raster (deterministic)
- [x] Queue **CONSTRUCTION-PARAM-DESIGN-001** then **CODER-001…006** (coders blocked until design PASS)
- [x] Update [`construction_invariants.md`](construction_invariants.md) §15
- [x] Indexed in [`development_plan_index.md`](development_plan_index.md) · wave board [`planner_wave6_parametric_todos_v1.md`](planner_wave6_parametric_todos_v1.md)

---

## Sign-off

| Role | Status | Date |
|:---|:---|:---|
| `@planner` | **PASS** | 2026-05-26 |
| `@designer` | **PASS (qualified)** (CONSTRUCTION-PARAM-DESIGN-001) | 2026-05-26 |
| `@coder` | **Unblocked** — CONSTRUCTION-PARAM-CODER-001…006 | 2026-05-26 |
