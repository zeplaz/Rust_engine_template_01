# Landscape keyframe requirements — burn / scar / regrowth `v1` (DMCP-LG5-KEYFRAME-REQS-001)

| Field | Value |
|:---|:---|
| **Program** | **APS-E4** |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **Spine** | `bake_source: keyframe_pack` · [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) |
| **Matrix** | [`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) |

---

## 1. Export folder

| Field | Value |
|:---|:---|
| Folder | `assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1/` |
| Naming | `{variant_key}.png` |
| Size | **64×64** px (pilot parity) |
| Seed | **550005** (batch `render.seed`) |
| Rig | `utils/Tile_iso_rig_v1.blend` or procedural teach fallback until G4 manual |

---

## 2. Per-row visual requirements

| `variant_key` | Read at iso zoom | Color / structure |
|:---|:---|:---|
| `topology_patch` | Dense canopy diamond | Green `#228034` core, dark edge |
| `topology_patch_scar` | Char gap in canopy | Ash `#3a3a3a` center, black edge ring |
| `topology_patch_burn_00` | Ember onset | Orange rim `#e87830` on scar base |
| `topology_patch_burn_04` | Mid fire | Brighter core `#ff9930`, smoke fringe |
| `topology_patch_burn_07` | Late fire | Dim ember `#994422`, wide ash |
| `topology_patch_regrowth_grass` | Low green scatter | `#6a9a48` dots on brown soil |
| `topology_patch_regrowth_shrub` | Shrub clumps | `#4a8040` blobs |
| `topology_patch_regrowth_canopy` | Young canopy | `#2d7038` diamond, lighter than mature |
| `topology_corridor` | Transport spine stripe | Brown `#78582c` band on green field |
| `topology_corridor_scar` | Burnt corridor | Ash stripe, broken edges |
| `topology_corridor_burn_00` | Fire along spine | Ember line on scar |
| `topology_corridor_burn_04` | Mid corridor fire | Wider orange band |
| `topology_corridor_burn_07` | Late corridor fire | Dark ember line |
| `topology_corridor_regrowth_grass` | Grass in corridor | Pale green on brown spine |
| `topology_ring` | Gold enclosure ring | `#d2aa3c` ring on dark fringe |
| `topology_ring_burn_00` | Ring ignited | Orange segment on ring |
| `topology_cluster` | Regrowth cluster | `*` green scatter pattern |
| `topology_cluster_scar` | Cluster gap | Ash patches in cluster |
| `topology_cluster_regrowth_grass` | Pioneer grass | Light green noise |
| `topology_cluster_regrowth_shrub` | Shrub nuclei | Medium green blobs |
| `topology_fringe` | Edge scrub | `.` shrub dots `#5a7048` |
| `topology_fringe_regrowth_grass` | Fringe pioneers | Sparse yellow-green |

---

## 3. G4 minimum review set (operator)

Before `proceed_ship: yes`:

1. `topology_patch_burn_04`
2. `topology_patch_scar`
3. `topology_corridor_regrowth_grass`

---

## 4. Forbidden

- Headless ortho-only PNGs for `ship: true`
- Unseeded per-export randomness
- Single-tile ad-hoc exports outside batch folder

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |
