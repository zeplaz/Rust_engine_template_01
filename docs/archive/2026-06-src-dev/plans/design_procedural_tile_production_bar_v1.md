# Design — Procedural tile production bar `v1` (Object-Plus)

| Field | Value |
|:---|:---|
| **ID** | **DESIGN-PROC-TILE-PROD-BAR-001** / **MCP-PT-1-001** |
| **Program** | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-03 |
| **Status** | **SIGNED** |
| **Blocks** | PT-2 production bakes, G4 atlas promote |

---

## Purpose

Define the **production tier** bar for assembled building iso tiles — above lod0 PG-2 proof, below final art direction polish. Bakes from **production** assembly snapshots only; lod0 atlases are APS automation proofs and **must not ship** (`lod0_atlas_ship_allowed: false`).

**Coupling:** [`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md) — step (5) forbidden until G4 pass.

---

## Tier ladder (recap)

| Tier | Tile bake | Module source | Ship in FULL_APP |
|:---|:---|:---|:---:|
| `lod0` | APS pilot / PG-2 witness | `kit_lod0_*` | No |
| `production` | PT-2 re-bake | `kit_production_*` | Yes (after TILE-PROD gates) |
| `smoke` | **Forbidden** | `kit_greybox_*` | Never |

---

## Production module bar (assembly input)

Per slot in assembly snapshot — all must pass before tile G4:

| Gate | Pass when |
|:---|:---|
| **M1 Silhouette** | Archetype profile readable at tactical zoom (wall family, roof profile, door width) — same cues as PG-2 but **full mesh**, not low-poly cheat |
| **M2 Tris** | ≤2k tris per slot mesh (production kit contract) |
| **M3 PBR** | `pbr_status: shipped`; tileable set id cited in job JSON |
| **M4 Proportion** | Real-world door height ~2.0–2.4m; floor height 3m grid respected |
| **M5 Canonical ID** | `module_id` in 50-set inventory; no smoke / greybox job ids |
| **M6 Reference** | Assembly snapshot `reference_tags` non-empty (PT-1-004) |

---

## Production tile bake bar (iso output)

**Authoritative spine:** [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) — same lane as `civ_truck_01` (keyframe stills → tilemapgen).

| Step | Tool / file |
|:---|:---|
| Assembly in Blender | `Light_keysshotsetup.blend` rig |
| Variant stills | `utils/keyframe_render.py` |
| Pack | `tile-atlas-pack` → `utils/tilemapgen` (`-pk` optional) |

**Forbidden for ship:** headless `tile_ortho_bake` only (`bake_source: smoke_ortho_headless`). CI may use ortho stub; production batches require `bake_source: keyframe_pack`.

| Gate | Pass when |
|:---|:---|
| **T1 Resolution** | Variant PNG ≥128×128; atlas `tile_px` ≥128 |
| **T2 Light rig** | `blender_orthographic_iso`; key fill + readable roof/wall separation |
| **T3 Entourage** | StylePack `prop_clutter` visible when slot resolved (or explicit `hide_slot` gap — not wrong mesh) |
| **T4 Night read** | `clean_night_on` / `damaged_night_on`: window emissive readable at iso scale without zoom |
| **T5 Damage read** | `damaged_*` visibly distinct from `clean_*` at 128px (wear, not recolor only) |
| **T6 Fire row** | `burning_00`…`burning_07` — 8 distinct frames; flame read on facade, not full-tile orange wash |
| **T7 Determinism** | Same assembly_id + variant_key + seed → pixel-identical re-bake |
| **T8 Style coupling** | Same footprint, different StylePack → different atlas cell (proven on pilot pair) |

---

## G4 sign-off rubric (atlas-level)

Use per-archetype YAML: `debug_runs/art_pipeline/*_production_signoff.yaml`. Template: [`tile_production_signoff_template.yaml`](../debug_runs/art_pipeline/tile_production_signoff_template.yaml).

| Gate | Pass when |
|:---|:---|
| **G4-1** | `source_tier: production` on assembly snapshot + index row |
| **G4-2** | `reference_tags` present and cited in sign-off notes |
| **G4-3** | Assembly silhouette matches reference one-liner (designer attestation + optional capture path) |
| **G4-4** | All **required** keys in archetype `variant_matrix_*_v1.yaml` baked and listed in `atlas_meta.json` |
| **G4-5** | Night + damaged variants pass T4/T5 at 128px iso |
| **G4-6** | Fire frames pass T6 (or documented waiver → TILE-PROD-003 blocked until baked) |
| **G4-7** | No smoke/greybox modules in assembly `module_placements` |
| **G4-8** | `proceed_ship: yes` — unblocks index `development_tier: production` ship path |

**Fail policy:** gap preferred over wrong mesh; missing required variant key → **fail** (do not ship atlas).

---

## Reference tags (PT-1-004)

Production assembly snapshots **must** include non-empty `reference_tags`:

```json
"reference_tags": ["ref:osm:way/…", "ref:survey:2026-rowhouse-portland-001"]
```

| Rule | Enforcement |
|:---|:---|
| Min 1 tag | MCP G0 rules YAML `reference_tags_required: true` |
| Tag format | `ref:<source>:<id>` — no free-text-only |
| Validator | `@coder-mcp` PT-2 — reject production bake if empty |

Designer supplies tag **intent** in variant matrix `style_packs.*.reference_tag_hints` for art team.

---

## Variant matrix authority

Per archetype: `debug_runs/art_pipeline/variant_matrix_{archetype}_v1.yaml`.

- **Required keys** → G4-4 + TILE-PROD-002 UV resolve
- **Optional keys** → may ship in later wave
- **7 style packs** → applicability matrix (primary / alternate / N/A)

Catalog defaults: `assets/configs/buildings/_variant_catalog.ron`.

---

## Archetype pilots (lod0 → production uplift)

| Archetype | Primary pack | Footprint | lod0 batch |
|:---|:---|:---|:---|
| rowhouse | `style_victorian` | 4×3×2 | `tile_rowhouse_victorian_pilot_v1` |
| warehouse | `style_industrial_west` | 4×2×2 | `tile_warehouse_industrial_west_pilot_v1` |
| shopfront | `style_colonial` | 3×3×2 | `tile_shopfront_colonial_pilot_v1` |
| bunker | `style_military` | 6×3×1 | `tile_bunker_military_pilot_v1` |

PT-2 re-bakes same footprints at `source_tier: production` with full matrix keys.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** (PT-1-001) | 2026-06-03 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | MCP-PT-1-001 production bar + G4 rubric |
