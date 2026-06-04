# PG-MODULE-AUDIT-001 — Industrial West / warehouse module kit audit `v1`

| Field | Value |
|:---|:---|
| **Todo ID** | **PG-MODULE-AUDIT-001** |
| **Program** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |
| **Style pack** | [`assets/configs/buildings/style_packs/style_industrial_west.ron`](../assets/configs/buildings/style_packs/style_industrial_west.ron) |
| **Index** | [`assets/configs/buildings/_module_index.ron`](../assets/configs/buildings/_module_index.ron) |
| **Pilot assembly** | `industrial_west_4x2_s43_a879` · [`building_definition_warehouse_industrial_west_production_v1.json`](../../tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json) |
| **Owner** | `@designer` |
| **Date** | 2026-06-02 |
| **Verdict** | **PASS (audit complete)** — **ship blockers** listed for **PG-MODULE-AUDIT-002** |

---

## Scope

Audit `_module_index.ron` entries tagged `style_pack: "style_industrial_west"` against:

1. **Style pack slot map** (11 slots in `style_industrial_west.ron`)
2. **Kit categories** from [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md): walls, roofs, corners, windows, doors, stacks, vents, pipes, platforms, signs, lights, AC, cranes
3. **Warehouse pilot** module picks in the production building definition

**Out of scope:** ECS/render, atlas register, headless bake policy (see [`mcp_orchestrator_tile_fix_warehouse_slice_v2.md`](mcp_orchestrator_tile_fix_warehouse_slice_v2.md)).

---

## Style pack slot resolution

| Slot | Resolves to | Index tier (`style_industrial_west`) | Ship-ready? |
|:---|:---|:---|:---:|
| `wall_1u` | `wall_steel_1u` | **production** `wall_steel_1u_production_run001` | yes |
| `wall_2u` | `wall_concrete_2u` | lod0 `wall_concrete_2u_lod0_run001` | no |
| `door_default` | `door_shop` | **not in pack** — index row is `style_rural` lod0 | no |
| `door_wide` | `door_warehouse` | lod0 `door_warehouse_lod0_run001` | no |
| `window_1u` | `win_double_1u` | **not in pack** — index row is `style_rural` lod0 | no |
| `window_industrial` | `win_industrial_3u` | lod0 `win_industrial_3u_lod0_run001` | no |
| `roof_default` | `roof_sawtooth` | **production** `roof_sawtooth_production_run001` | yes |
| `roof_industrial` | `roof_shed` | lod0 `roof_shed_lod0_run001` | no |
| `roof_flat` | `roof_metal_low` | lod0 `roof_metal_low_lod0_run001` | no |
| `corner_outer` | `corner_L` | lod0 `corner_L_lod0_run001` only under this pack | no |
| `prop_clutter` | `prop_vent` | lod0 `prop_vent_lod0_run001` | no |

**Production-tier modules under `style_industrial_west`:** **2** — `wall_steel_1u`, `roof_sawtooth`.

---

## Category coverage (`style_industrial_west` index filter)

| Category | Unique IDs | Production tier | Gap |
|:---|---:|---:|:---|
| **walls** | 6 | 1 (`wall_steel_1u`) | `wall_concrete_*`, `wall_industrial_panel_2u` lod0/smoke only |
| **roofs** | 8 | 1 (`roof_sawtooth`) | shed / flat / canopy lod0 or smoke |
| **corners** | 4 | 0 | `corner_L`/`corner_T` lod0; `corner_steel_inner` smoke |
| **windows** | 3 | 0 | all lod0 or smoke |
| **doors** | 6 | 0 | `door_warehouse` lod0; others smoke/lod0 |
| **stacks** | 0 | 0 | **EMPTY** |
| **vents** | 2 | 0 | `prop_vent` lod0; `prop_vent_roof_1u` smoke |
| **pipes** | 0 | 0 | **EMPTY** |
| **platforms** | 0 | 0 | **EMPTY** |
| **signs** | 0 | 0 | **EMPTY** |
| **lights** | 1 | 0 | `prop_light` lod0 |
| **ac** | 1 | 0 | `prop_ac` lod0 |
| **cranes** | 0 | 0 | **EMPTY** |
| **other props** | 2 | 0 | `prop_tank`, `prop_transformer` lod0 |

Compared to kit minimum (**50 modules / 10 per category** in [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md)), industrial west is **shell-complete for Long Hall warehouse massing** (steel wall + sawtooth roof) but **detail grammar is thin** — no stacks, pipes, platforms, signs, or cranes in pack.

---

## Warehouse pilot — bdef vs index mismatches

Current production bdef modules:

| bdef `module_id` | bdef `job_id` | Index `style_pack` for that job | Issue |
|:---|:---|:---|:---|
| `corner_L` | `corner_L_production_run001` | **`style_victorian`** (brick/red) | Style drift — production corner is Victorian art on industrial west assembly |
| `wall_steel_1u` | `wall_steel_1u_production_run001` | `style_industrial_west` | OK |
| `door_shop` | `door_shop_lod0_run001` | **`style_rural`** (wood plank) | Cross-pack slot; lod0 tier |
| `roof_sawtooth` | `roof_sawtooth_production_run001` | `style_industrial_west` | OK |

**Tile-fix witness:** `tile_fix_09` reports **8 production + 16 lod0** placements — consistent with shell production + greybox fill.

**Designer sign-off:** Shell (`wall_steel` + `roof_sawtooth`) is sufficient for **manual keyframe G4** on warehouse silhouette; corner/door tier mismatches are **PG-MODULE-AUDIT-002** production jobs, not blockers for checklist prep.

---

## Recommended production jobs (handoff → PG-MODULE-AUDIT-002)

Priority for `IndustrialWarehouse` grammar + G4 stills:

| Priority | Category | Suggested `module_id` | Target job pattern | Rationale |
|:---:|:---|:---|:---|:---|
| P0 | corner | `corner_L` | `corner_L_production_run001` **re-tag or fork** `style_industrial_west` | bdef already references production job; mesh must match steel/concrete west |
| P0 | door | `door_warehouse` | `door_warehouse_production_run001` | `door_wide` slot; replace rural `door_shop` lod0 in bdef |
| P1 | window | `win_industrial_3u` | `win_industrial_3u_production_run001` | facade grammar for upper floors |
| P1 | wall | `wall_concrete_2u` | `wall_concrete_2u_production_run001` | `wall_2u` slot for deep bays |
| P2 | prop | `prop_vent` | `prop_vent_production_run001` | `prop_clutter` slot |
| P2 | roof alt | `roof_shed` | `roof_shed_production_run001` | `roof_industrial` slot diversity |
| P3 | stacks | `stack_chimney_1u` (new) | production run | grammar detail — category empty |
| P3 | platforms | `platform_dock_2u` (new) | production run | loading-bay grammar |
| P3 | signs / cranes | TBD | production run | optional facade/detail grammar |

---

## Exit criteria (this todo)

| Criterion | Status |
|:---|:---:|
| Slot map vs index documented | yes |
| 13 kit categories scored for `style_industrial_west` | yes |
| Warehouse bdef cross-check | yes |
| PG-MODULE-AUDIT-002 job list drafted | yes |
| Blocks PILOT-GRAMMAR-001 **prep** | no — checklist is doc-only |

**Unblocks:** [`DESIGN-PILOT-GRAMMAR-001-PREP`](../tools/orchestrator/queues/grammar_continuation_queue.json), **PG-MODULE-AUDIT-002** (@coder-mcp gap jobs).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Initial warehouse / industrial west audit |
