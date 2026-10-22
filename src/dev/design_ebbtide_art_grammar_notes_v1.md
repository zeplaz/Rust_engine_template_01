# Ebbtide art grammar notes (Slice 4) `v1`

| Field | Value |
|:---|:---|
| **Owner** | `@designer-mcp` → `@coder-mcp` (bake when ship) |
| **Date** | 2026-10-17 |
| **Status** | **SIGNED** (notes only — no AI final art) |
| **Parent** | [`design_ebbtide_build_style_rubric_v1.md`](design_ebbtide_build_style_rubric_v1.md) |
| **Grammar** | [`arch_build_grammar_v0_baseline_v1.md`](arch_build_grammar_v0_baseline_v1.md) §10 |
| **Massing** | [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) |

## Production rules

1. Spec → MCP/CLI → validate → promote → Bevy registry. No AI-generated final art.
2. Deterministic seed on every bake job.
3. When `ship: true`, `bake_source: keyframe_pack` (iso tile bake spine).
4. Committed visuals land in **`RepresentationResult`** — not a parallel ghost extractor.
5. VoxelMassing column stacks are **runtime presentation**; bake may later replace them for operational sites with lod0+ massing.

## Height / silhouette checklist for bake packs

| Check | Pass when |
|:---|:---|
| Silhouette | Building reads at operational zoom with hard edge |
| LOD | Max-zoom pack carries ~2× readable detail vs operational stamp |
| Site void | Primary massing ≤ 40% of site stub footprint |
| Grid | Modules snap to footprint cells — no free-rotated props |

## Next bake pick (not this slice)

Warehouse / logistics pilot already has ARCH-DNA preset — attach height-band metadata to assembly snapshot when G4 stills open. Residual operator G4 stays on MCP APS tooling finish queue.
