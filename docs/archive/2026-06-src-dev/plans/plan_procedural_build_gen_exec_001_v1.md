# PLAN-PROC-BUILD-EXEC-001 — Procedural building generation exec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-PROC-BUILD-EXEC-001** |
| **Slice** | **PROC-BUILD-GEN-001** |
| **Parent** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| **Designer** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) |
| **Prereq** | Construction scaling audit green; Stage 5 representation contract |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **SIGNED — Wave 2 MCP-PLN-PG2-001** |
| **Fleet** | [`mcp_fleet_wave2_orders_v1.md`](mcp_fleet_wave2_orders_v1.md) Stream ENGINE |

**Rule:** Procedural output is **render/presentation** + committed **metadata** on site — sim authority remains construction commit + `SiteConstructionPhase`.

**Downstream:** PG-2 assembly manifest is the **sole input** to building iso tile bakes ([`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md)). lod0 modules wire PG-2; **production** modules + references gate tile art.

---

## Summary

Phased delivery: **data model → RON loaders → greybox assembly → grammar**. No Phase skips to full CityEngine.

---

## Authority map

| Resource | Writer | Readers |
|:---|:---|:---|
| `BuildingArchetype` registry | loader `src/construction/procedural/` | ghost preview, growth proposals |
| `StylePack` registry | loader | assembly only |
| `ProceduralBuildingRequest` | commit hook / growth proposal | extract |
| Assembled instances | render extract | GPU / minimap — **read-only** |
| `BuildingDefinition` legacy rows | unchanged | industrial chains until migrated |

---

## PG-1 — Archetype + StylePack data (≤3 files) — **MCP-PG-1-001**

| File | Change |
|:---|:---|
| `src/construction/procedural/types.rs` | **new** — `BuildingArchetype`, `BuildingUsage`, `StylePack`, `StylePackRegistry`, `ProceduralBuildingRequest` |
| `src/construction/procedural/load.rs` | load `assets/configs/buildings/style_packs/style_*.ron` per [`plan_style_pack_ron_v1.md`](plan_style_pack_ron_v1.md) |
| `src/construction/procedural/mod.rs` | register resources + `init_style_pack_registry` |

**Exit:** lib test loads ≥1 style pack + archetype stub; witness `style_packs_loaded: true`.

**Tests (names):**

- `style_pack_ron_loads_victorian_slots`
- `style_pack_rejects_duplicate_ids`
- `style_pack_slot_module_id_non_empty`

**Registry rule:** resolve mesh handles via `ProceduralModuleRegistry::resolve_module_id()` — never raw smoke rows ([`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) § Registry tier filter).

---

## PG-2 — Footprint grid + lod0 assembly (≤3 files) — **MCP-PG-2-001 / MCP-PG-2-002**

| File | Change |
|:---|:---|
| `src/construction/procedural/footprint_grid.rs` | W/D/C token grid from width×depth + door edge |
| `src/render/extraction/procedural_build_extract.rs` | **new** — instances from grid + `StylePack` slot → `resolve_module_id` → GLB |
| `src/render/extraction/procedural_module_extract.rs` | wire `RepresentationResult.procedural_module_meshes` (MCP-PG-2-002) |

**Depends on:** ≥10 lod0 modules promoted (Wave 1) + **MCP-D0-SP-001** style pack RON (7 packs).

**Exit:** tactical view shows **different module GLBs** per `StylePackId` for same footprint — **lod0 only**, never smoke.

**Tests (names):**

- `footprint_grid_door_on_floor_zero`
- `footprint_grid_corner_token_consumes_c`
- `procedural_build_extract_resolves_lod0_glb`
- `procedural_build_extract_skips_smoke_row`
- `procedural_build_extract_hide_slot_when_module_missing`
- `style_pack_victorian_vs_industrial_different_wall_ids`

**Attach:** `RepresentationResult` / projection graph — **no** second extract path (Stage 5 convergence).

**Witness — MCP-PG-2-WIT:**

**File:** `debug_runs/procedural_assembly_live.json`

| Key | Type | Pass when |
|:---|:---|:---|
| `pg2_wired` | bool | extract registered |
| `style_pack_id` | string | e.g. `style_victorian` |
| `module_ids_used` | string[] | all lod0+, no `kit_greybox` job ids |
| `smoke_fallback_used` | bool | **must be false** |
| `footprint_cells` | u32 | W+D+C count |
| `green` | bool | rollup |

---

## PG-3 — Bridge from commit + parametric scale (≤3 files)

| File | Change |
|:---|:---|
| `src/construction/parametric_commit.rs` | emit `ProceduralBuildingRequest` on commit |
| `src/strategic/site/components.rs` | optional `ProceduralBuildingSpec` component |
| `src/construction/live_proof.rs` | `construction_procedural_build_001` block |

**Exit:** commit Portland test site carries spec; witness green.

---

## PG-4 — Shape grammar (optional / later)

| File | Change |
|:---|:---|
| `src/construction/procedural/grammar.rs` | rule parser (Residential → Apartment → …) |
| `assets/configs/buildings/grammars/residential.ron` | designer-authored rules |
| tests | same seed + grammar → stable layout |

**Do not start** until PG-2 green.

---

## Witness schema

**File:** `debug_runs/construction_stage_live.json` (rollup)

| Pointer | Meaning |
|:---|:---|
| `/construction_procedural_build_001/archetypes_loaded` | bool |
| `/construction_procedural_build_001/style_packs_loaded` | bool |
| `/construction_procedural_build_001/pg2_assembly_wired` | bool — was `greybox_assembly_wired` |
| `/construction_procedural_build_001/green` | rollup |

**File:** `debug_runs/procedural_assembly_live.json` (PG-2 detail — MCP-PG-2-WIT)

| Pointer | Meaning |
|:---|:---|
| `/pg2_wired` | extract path live |
| `/smoke_fallback_used` | must be **false** |
| `/module_ids_used` | lod0 canonical ids |
| `/green` | PG-2 slice pass |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib procedural::module_index procedural
cargo test -p proc_A_dine01 --lib procedural_build_extract -- --nocapture
```

---

## Anti-patterns

- One JSON file per visual variant
- Generator mutating `FootprintMatrix` after commit without validation
- Hanabi / particle VFX as building substitute
- Military-only procedural fork

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | PROC-BUILD-GEN-001 |
| **Mutex** | `src/construction/procedural/*`, render extract — avoid `src/substrate/` |
| **Parallel** | OG-1 can start after PG-1 (uses archetypes, not meshes) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Initial PG-1..4 phases |
| v1.1.0 | 2026-06-02 | MCP-PLN-PG2-001 — witness keys, test names, Wave 2 fleet alignment |
