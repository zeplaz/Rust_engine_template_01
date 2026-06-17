# Landscape LG-5 expansion matrix charter `v1` (DMCP-E4-MATRIX-CHARTER-001)

| Field | Value |
|:---|:---|
| **Program** | **APS-E4** · **DMCP-E4-MATRIX-CHARTER-001** |
| **Date** | 2026-06-17 |
| **Owner** | `@designer-mcp` |
| **Verdict** | **PASS** (re-verdict 2026-06-02) |
| **Blocks** | `APS-EVO-E4-ATLAS-EXPAND-001` |
| **Parent** | [`plan_aps_evolution_veg_capability_20260616_v1.md`](plan_aps_evolution_veg_capability_20260616_v1.md) |
| **Pilot** | [`design_landscape_lg5_atlas_v1.md`](design_landscape_lg5_atlas_v1.md) (3× clean) |

**No bpy in this doc.** Authoritative **topology × state** budget for expanded atlas v1.

---

## Mission

Size the **full state×topology matrix** before bake so burn rows do not force sheet re-layout. Pilot teaches 3 clean cells; **expanded v1** adds scar + burn + regrowth rows per charter below.

---

## 1. Topology kinds in atlas v1

| Kind | Extract glyph | `variant_key` prefix | In pilot? | In expanded v1? |
|:---|:---:|:---|:---:|:---:|
| **Patch** | `#` | `topology_patch` | yes (clean) | yes |
| **Corridor** | `=` | `topology_corridor` | yes (clean) | yes |
| **Ring** | `()` | `topology_ring` | yes (clean) | yes |
| **Cluster** | `*` regrowth | `topology_cluster` | no | yes |
| **Fringe** | `.` shrub | `topology_fringe` | no | yes |
| Network | `◊` | — | no | **out** (graph bone, not iso sprite) |

**Rule:** Six LG-1 eval kinds exist; atlas v1 ships **five** extract glyphs (Network excluded — tint-only in LG-4).

---

## 2. State rows (tile-generation spine)

| State row | `variant_key` suffix | Sim / succession | Required in v1? |
|:---|:---|:---|:---:|
| **clean** | _(none — base key)_ | operational canopy | yes |
| **scar** | `_scar` | `BurnScar` / disturbance `x` | yes |
| **burn_00** | `_burn_00` | fire frame 0 | yes |
| **burn_04** | `_burn_04` | fire mid-sequence | yes |
| **burn_07** | `_burn_07` | fire late frame | Patch + Corridor only |
| **regrowth_grass** | `_regrowth_grass` | `SuccessionTopologyStage::Grass` | yes |
| **regrowth_shrub** | `_regrowth_shrub` | `Shrub` | Patch + Cluster |
| **regrowth_canopy** | `_regrowth_canopy` | `YoungForest` / `#` | Patch only |

---

## 3. Matrix (atlas v1 — 16 cells, 4×4 grid)

| Topology ↓ / State → | clean | scar | burn_00 | burn_04 | regrowth_grass | regrowth_shrub |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|
| **Patch** | ● | ● | ● | ● | ● | ● |
| **Corridor** | ● | ● | ● | ● | — | — |
| **Ring** | ● | — | ● | — | — | — |
| **Cluster** | ● | — | — | — | ● | — |
| **Fringe** | ● | — | — | — | ● | — |

**Cell count:** 16 (`atlas.columns: 4`, `atlas.rows: 4`, `tile_px: 64`).

**Pilot reuse:** first 3 cells (`topology_patch`, `topology_corridor`, `topology_ring`) keep pilot PNGs; expanded bake fills remaining 13.

---

## 4. Production rules (every row)

```yaml
rules_applied:
  - no_ai_generated_images
  - deterministic_output
  - batch_processing
  - grid_alignment
bake_source: keyframe_pack   # ship path — not smoke_ortho_headless
render.seed: 550005          # same family as pilot
atlas_domain: landscape
```

---

## 5. Coder handoff

| Slice | Do |
|:---|:---|
| **DMCP-TILE-BATCH-EXPAND-SPEC-001** | Signed JSON — this matrix |
| **DMCP-LG5-KEYFRAME-REQS-001** | Per-row visual reqs |
| **APS-EVO-E4-ATLAS-EXPAND-001** | Pack + witness after G0 + keyframes |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 · **re-verdict 2026-06-02** (`dmcp_e4_matrix_charter_live.json`) |
