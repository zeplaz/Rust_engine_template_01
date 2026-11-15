# VoxelMassing presentation contract `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **VOXEL-MASSING-PRES-001** |
| **Owner** | `@planner` (schema) → `@coder` (paint) → `@designer-mcp` (height bands) |
| **Status** | **SIGNED** |
| **Date** | 2026-10-17 |
| **Parent** | [`design_ebbtide_build_style_rubric_v1.md`](design_ebbtide_build_style_rubric_v1.md) |
| **Code** | `src/construction/voxel_massing.rs` |
| **Paint** | `ConstructionVisualRequests.massing_columns` → `world_map_frame` |
| **Deferral** | **DR-VOXEL-OCCUPATION** — sim voxel authority out of scope |

**Rule:** Schema + authority only. Preview ≠ gameplay. Columns are **disposable** — never enter commit payloads.

---

## 1. Inputs (read-only)

| Input | Source | Notes |
|:---|:---|:---|
| Occupied XY cells | `BuildGhostState` / weighted footprint snapshot / staged book | Same as footprint ghost |
| Height band | Catalog heuristic + `BuildingFamily` / occupied count · optional ARCH-DNA `βvert` later | Clamped **1..=4** floors for operational zoom |
| Validity | `BuildPlacementPreview` / staged validity | Same `FootprintTileColorKind` tokens |

## 2. Output (presentation-only)

```text
VoxelColumnRequest {
  tile: IVec2,           // world cell
  floors: u8,            // 1..=4 stacked visual layers
  color_kind: FootprintTileColorKind,
  weight: f32,           // partial-alpha from weighted raster
}
```

Aggregated into `ConstructionVisualRequests.massing_columns` each frame (cleared with other requests).

## 3. Derive rule (deterministic)

```text
floors = clamp(
  1 + floor(log2(max(1, occupied_cells)))   // size band
    + family_height_bonus,                  // 0..2
  1, 4
)
```

- One column per occupied ghost tile (weight > omit threshold).
- No column on empty / omitted cells.
- Adjust-lock ring still from footprint path — massing does not own lock state.

## 4. Paint rule

- Base footprint fill + rim remain the occupation read.
- Massing draws **stacked quads** offset in map-plane +Y with rising draw-Z — silhouette of height, not a second XY authority.
- Edge rim uses `ghost_visual` outline tokens (~1px discipline).
- Below ~24 px/tile edge: prefer footprint grid only (readability charter) — massing may collapse to floors=1.

## 5. Explicit non-goals

| Forbidden | Why |
|:---|:---|
| Commit voxels / dig / freeform place | DR-VOXEL-OCCUPATION |
| Parallel extractor / GPU path outside visual requests | Invariant §18 |
| Infer gameplay from stack height | Sim owns `FootprintMatrix` only |
| Replace two-click place | UX redesign PASS |

## 6. Conflict matrix

| Peer | Relation |
|:---|:---|
| Footprint ghost / weighted raster | SERIAL — massing derives after footprint sync |
| Site stub overlay | PARALLEL OK — different request fields |
| PG-2 / RepresentationResult bake | COORDINATE — bake may later replace column stacks for committed sites |
| Future `plan_voxel_occupation_v1` | SERIAL / BLOCK until DR-VOXEL-OCCUPATION predicates true |

## 7. Witness

- Design: `debug_runs/design_ebbtide_build_style_live.json` → `voxel_massing_presentation: true`
- Lib: `voxel_massing` unit tests + construction visual sync tests

**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-VOXEL-OCCUPATION**.
**Migration truth:** [`mig_v1_gate.json`](../debug_runs/mig_bevy_019/mig_v1_gate.json)

---

## 8. Book-aligned parallel lanes (maintainability)

Orient: `python src/dev/ref_books_index/lookup.py --brief src/construction/voxel_massing.rs` → **PCG-06**.

Presentation massing is **not** a second sim authority. Books split tool-side vs runtime (**ECS-15**) and asset conditioning (**IO-09**) from the render application stage (**RND-01** / instancing **RND-09**). Use that split so APS, construction derive, and paint can land in parallel on one JSON contract.

| Lane | Book row | Sole owner | Owns | Must not own |
|:---|:---|:---|:---|:---|
| **A — Artist / conditioning** | **IO-09** · **ECS-15** (tool-side) · **PCG-06** (grammar height intent) | `@designer-mcp` / `@coder-mcp` APS Massing tab | Seeded `massing_preview.json` (`massing_columns[]`); StudioTask `massing_export`; HTML dash for QC | `FootprintMatrix`, commit payloads, Bevy systems |
| **B — Engine derive** | **PCG-06** (shape / massing heuristic) | `@coder` [`voxel_massing.rs`](../construction/voxel_massing.rs) | `derive_massing_floors`, `VoxelColumnRequest`, sole writer into `ConstructionVisualRequests.massing_columns` from ghost/footprint | GPU meshes, APS UI, occupation dig/place |
| **C — View / render paint** | **RND-01** (application stage) · **RND-09** (instanced stacks later) · **RND-18** (viewport / dual-view) | `@coder` world_map_frame / pixel_world layers | Consume `massing_columns` RO; stacked quads / silhouette; zoom collapse | Height heuristics, grammar seeds, occupation |

**Shared wire (parallel-safe):**

```text
APS massing_preview.json  ≈  VoxelColumnRequest[]
  tile, floors∈[1,4], color_kind, weight, seed
→ engine may ingest for ghost override OR keep catalog derive
→ ConstructionVisualRequests.massing_columns (sole B writer)
→ C paints only
```

**Future (blocked until DR-VOXEL-OCCUPATION):** sim voxel occupation plan — serial after presentation lanes are green; do not grow Lane B/C into dig authority.

**Studio charter:** [`plan_artist_studio_expansion_v1.md`](plan_artist_studio_expansion_v1.md) § Book lanes.
