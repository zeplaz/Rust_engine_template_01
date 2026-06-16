# Derived metric pipeline (v1)

**Status:** simulation contract — **computed** scalars and vectors from **physical fields** + **fact tags**.  
**Rule:** derived metrics are **not** authored per cell in the tag registry; they are **materialized** by systems (worldgen post-passes or runtime ticks) and cached where needed.

---

## 1. Why this layer exists

Facts are discrete (`TagSet`). Many interpretations need **continuous** quantities:

- grade / slope
- drainage accumulation
- traction-derived index
- concealment from vegetation + relief
- construction bearing / support heuristic
- **extraction difficulty** (geology + access + facts)

Those values should be **reproducible** from the same world truth so AI, preview, and sim agree.

---

## 2. Chunk component shape (target API — stabilize early)

**Design:** [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) should carry **dense per-cell `Vec<f32>`** aligned with `ChunkCellMatrix` (same `size`, same linear index). **Early stubs:** every field listed below may exist; **only fields with real passes compute non-zero values**; the rest `vec![0.0; w*h]`.

| Field (example) | Role |
|:---|:---|
| `slope_grade` | Cardinal grade / seam-stitched at chunk borders |
| `drainage` | Hydrology / accumulation (basin continuity) |
| `erosion_risk` | Longer-timescale substrate / flow stress |
| `support_capacity` | Construction interpretation input (may mirror `build.*` hints + geology) |
| `concealment` | Recon / combat interpretation input (may combine relief + canopy) |

Additional persist-friendly scalars (when specified by design): `soil_depth`, `rock_hardness`, `flood_basin_id` — type as `Vec<f32>` or integer id vectors consistently.

**Why stub now:** stable ECS queries, save schema, tests, AI tooling — **before** every pass is implemented.

---

## 2b. Dynamic terrain overlay (transient sim — separate from facts)

**Not** part of designer tag ontology. Holds **weather mud, snow, battle damage, congestion, temporary flooding**, etc.

- **Must not** mutate fact `TagSet` or static `MaterialDef` rows for temporary state.
- **Persistence:** optional, partial, or regenerate — gameplay decision per mode.
- **Implementation note:** prompt examples using `HashMap<CellId, f32>` are **conceptual**; production may use **chunk-partitioned** buffers for locality and determinism. Define **one** cell coordinate story (world id vs chunk+local).

Static terrain facts + derived metrics + dynamic overlay → **systems interpret** (mobility, build, warfare, economy).

---

## 3. Conceptual per-cell struct (legacy sketch — single-tile view)

The following remains a **mental model** for one cell; on-disk / ECS may use SoA (`Vec` per field on the chunk):

```rust
/// Populated by DerivedTerrainSystems — not serialized as designer-authored tags.
pub struct TerrainDerivedMetrics {
    pub slope_grade: f32,
    pub drainage: f32,
    pub erosion_risk: f32,
    pub support_capacity: f32,
    pub concealment: f32,
    // ...
}
```

Storage options (engineering choice — document when picked):

- Dense grid parallel to `ChunkCellMatrix`
- Sparse overlay for LOD1/2
- Recompute on demand in pocket from height + tags

---

## 4. Pipeline stages (worldgen + runtime)

| Stage | Produces | Notes |
|:---|:---|:---|
| **Physical generation** | DEM, moisture, temp, optional erosion/sediment | Existing passes / enhanced |
| **Material assignment** | `MaterialId` + **fact** `TagSet` | Rules RON + registry |
| **Fact extraction** | Normalize tags from materials + pass-2 thresholds | No gameplay tags |
| **Derived compute** | `ChunkDerivedMetrics` (SoA per chunk) + global fields | Deterministic from seeds + chunk coords; **stub vecs OK** until passes land |
| **Interpretation** | Mobility, build, combat, economy | Reads facts + derived; never writes “traversable” tags |

**Runtime deltas**

- Weather, war damage, engineering — mutate **sim state** or **temporary overlays**, not core fact ontology unless the world truly changed (e.g. permanent flood scar → new hydrology facts via event).

---

## 5. Consumers (non-exhaustive)

| Consumer | Typical inputs |
|:---|:---|
| Mobility | facts + `slope_grade`, `traction` proxy, water depth |
| Construction | `support_capacity`, `waterlogged`, `unstable_subsurface` |
| Resource extraction | `mineral_rich`, `extraction_difficulty`, logistics reach |
| Combat / recon | `concealment`, exposure (often derived from relief + canopy) |
| Hydrology sim | `drainage`, `erosion_risk` — flood propagation |
| Preview / debug | Heatmaps per metric and per mobility profile |

---

## 6. Preview evolution

- **Fact preview:** toggle overlays per **category** (topology, hydrology, …).
- **Derived preview:** slope, drainage, concealment, construction support.
- **Interpretation preview:** select **`MobilityProfileId`** → heatmap from matrix evaluation.

This replaces weak single-mode `PreviewMode::Tag` with explicit **fact vs derived vs interpreted** modes (implementation tracked in preview matrix / `world_preview.rs`).

---

## 7. Serialization contract — persist vs recompute

**Policy (v1):**

| Persist (expensive, stable) | Typical fields |
|:---|:---|
| Save / long-lived cache | `slope_grade`, `drainage`, `erosion_index`, `soil_depth`, `rock_hardness`, `flood_basin_id` (as implemented per phase) |

| Recompute (dynamic) | Typical fields |
|:---|:---|
| Runtime or high-frequency tick | mud, snow, damage, danger, traffic pressure, congestion, water accumulation |

**Why:** smaller saves, stable versioning, faster load, deterministic regeneration from shared inputs, better mod compatibility.

**Terrain truth:** store **facts + slow derived**; **do not** persist mobility / build / warfare **verdicts** as tile tags.

**Code today:** [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) (`slope_grade` only); cross-chunk **stitching** at borders ([`stitch_chunk_slope_grades`](../../../src/terrain/generation/derived.rs)). Additional fields arrive incrementally per this contract.

Bump save format when derived structs change.

---

## 8. Checklist before implementation

- [x] Choose storage layout for `TerrainDerivedMetrics` vs chunk matrix. **→ v1:** [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) Bevy component (`slope_grade` dense `Vec` per chunk).
- [x] Define which passes compute which fields (worldgen vs first sim tick). **→ v1:** after chunk materialize in [`material_plugin.rs`](../../../src/systems/terrain/material_plugin.rs).
- [ ] Align with transport graph (`systems::transport`) for on-road vs off-road cost composition.
- [x] Update [`../material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md) cross-links to this folder.
