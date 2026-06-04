# Models — procedural module kit (lane C)

## Current layout (stable — do not mass-rename without reindex)

```text
models/modules/<job_id>/
  model.glb
  manifest.json
  <job_id>.module.json   # optional AssetSpec sidecar
```

Index: [`configs/buildings/_module_index.ron`](../configs/buildings/_module_index.ron) — **paths are authoritative**.

---

## Naming

| Part | Meaning |
|:---|:---|
| `module_id` | Catalog id (`wall_brick_1u`, `door_shop_1u`) |
| `job_id` | Promoted run folder (`wall_brick_1u_run001`, `*_lod0_run001`) |
| `batch_id` | MCP batch (`kit_greybox_001`, `kit_lod0_001`, `kit_production_*`) |

---

## Tiers (same module_id, different folders)

| Tier | Batch prefix | Map / StylePack | Tile bake |
|:---|:---|:---:|:---:|
| **smoke** | `kit_greybox_*` | Hidden | Forbidden |
| **lod0** | `kit_lod0_*` | Staging / PG-2 witness | Smoke only |
| **production** | `kit_production_*` | Ship | Source for keyframe building iso |

**One GLB per folder** — the unit artists and MCP promote. Category (`wall`, `door`, `roof`) lives in **index metadata**, not necessarily in the path (future optional: `modules/by_category/wall/` when we have a migration script).

---

## Relationship to buildings on the map

```text
StylePack + footprint  →  many module_placements (3D assembly)
                      →  optional iso atlas (2D stamp over footprint)
```

Modules are **not** stamped on the overworld grid one cell at a time. See [`../README.md`](../README.md).

---

## Tools

| Tool | Role |
|:---|:---|
| Module Kit Viewer | `tools/mcp/module_viewer/` — browse GLB + metadata |
| `library_register` | Updates `_module_index.ron` after promote |
