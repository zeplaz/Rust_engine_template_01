# BUILDING-TILE-SPINE-001 — Authoritative building tile program `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **BUILDING-TILE-SPINE-001** |
| **Source** | [`docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md`](../../docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md) (Phases 1–10 + post–line 400) |
| **Parent index** | [`plan_tile_pipeline_fix_auto_build_v1.md`](plan_tile_pipeline_fix_auto_build_v1.md) (TILE-FIX foundation) |
| **Grammar program** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) (planner §824+ — hierarchical grammar, APS authoring) |
| **Status** | **ACTIVE** |
| **Pilot** | `warehouse_industrial` / `industrial_west_4x2_s43_a879` |

---

## North star (planner)

**Variants exist before render.** Procedural system owns geometry + materials + variant graph; rendering is the **final compile** (headless Blender worker), not an artist-facing editor.

```text
Asset Definition
    → Assembly Snapshot (graph: module + material + tags per node)
    → Variant Graph (layers: material / visibility / emission / decal overrides)
    → Build Graph (witnessed nodes)
    → Headless Blender (apply materials → variant → facing → frame → render)
    → Atlas (variant × facing × frame) + visual_config.ron
    → Registry
    → Runtime VisualState(variant, facing, frame)
```

**Wrong spine (retire):** PG assembly → greybox GLB → single ortho snapshot → pack → registry.

**Wrong primary UX (retire):** Designer → open Blender → fix → render → export.  
See [`warehouse_tile_ship_workflow_v1.md`](warehouse_tile_ship_workflow_v1.md) — **debug/recovery only**.

---

## Authoritative operator path

| Who | Surface |
|:---|:---|
| Designer / artist | **Art Pipeline Suite** — Catalog → Assembly → Variants → Atlas |
| Designer agent | Same steps via **MCP/CLI** (`assembly_snapshot_generate`, `variant_set_patch`, `variant_bake`, `tile_batch_run`, `tile-atlas-pack`) |
| Build backend | **Headless Blender** (`blender-worker`) — never greybox ortho as production |

```text
Designer → Art Pipeline Suite → MCP commands → Headless Blender → output assets
```

Blender = asset compiler (like `cargo`), not the primary UI.

---

## Single source of truth (authority chain)

| Layer | Artifact | Role |
|:---|:---|:---|
| 1 | **Asset Definition** | `BuildingDefinition` + module specs + `material_profile` in `_module_index` |
| 2 | **Assembly Snapshot** | `assets/staging/assemblies/<id>.json` — placements **+ per-node material/tags** (ARCH-003) |
| 3 | **Variant Graph** | `variant_set_v1` / generated `VariantRecipe[]` — overrides **before** bake |
| 4 | **Build Graph** | Job DAG: assembly → variants → blend → frames → pack → register |
| 5 | **Atlas** | `atlas.png`, `atlas_meta.json`, `visual_config.ron`, derived index row |

Derived only (not competing truth): stylepack picks, batch JSON, witness JSON, staging PNG folders.

---

## Tile output contract (Phase 9)

Per building folder:

```text
warehouse_industrial/
    atlas.png
    atlas_meta.json
    visual_config.ron
    assembly.blend          # cache from build_blend
    manifest.ron
```

`visual_config.ron` example (planner):

```ron
BuildingVisual(
    facings: 8,
    states: [
        "clean_day",
        "clean_night",
        "damage_light",
        "damage_heavy",
        "fire",
    ],
)
```

Warehouse pilot matrix: **State × Facing × Frame** (8 facings × 6+ states; fire adds animation frames).

---

## Build graph nodes (BUILD-001)

Each node emits: **artifact**, **manifest**, **witness**, **validation**.

| Node | MCP / CLI | Output |
|:---|:---|:---|
| `build_assembly` | `assembly_snapshot_generate` | `assembly_snapshot.json` |
| `build_variants` | `variant_set_validate` / generator | `variant_set` + `VariantRecipe[]` |
| `build_blend` | `assembly_build_job` | `assembly.blend` |
| `render_frames` | `variant_bake` / `tile_batch_run` | PNG per (variant, facing, frame) |
| `pack_atlas` | `tile-atlas-pack` | `atlas.png` + `atlas_meta.json` |
| `register_atlas` | index upsert (post G4) | `_tile_atlas_index.ron` row |

Headless worker job shape (RENDER-001):

```json
{
  "job": "render_variant",
  "asset": "warehouse_industrial",
  "variant": "damage_heavy",
  "facing": 3,
  "frame": 0
}
```

---

## Backlog (planner order — post TILE-FIX)

TILE-FIX-01…10 laid schema, resolver, validators, and froze greybox ship. **Spine work starts here.**

| ID | Owner | Deliverable | Status |
|:---|:---|:---|:---|
| **ARCH-001** | @planner-mcp | Formal **Assembly Graph** schema (`assembly_graph_node_v1.schema.json`) | **done** |
| **ARCH-002** | @planner-mcp | Formal **Variant Graph** schema (`VariantNode`: material/visibility/emission/decal overrides) | **pending** |
| **ARCH-003** | @coder-mcp | `material_profile` (+ tags) on each `module_placement`; enrich on generate/load; ship validator | **done** |
| **APS-UI-003b** | @coder-mcp | **Assembly Editor** in Art Pipeline Suite — placements list, material/tags/LOD, save/validate | **done** |
| **BUILD-001** | @coder-mcp | Explicit build dependency graph + per-node witness | **pending** |
| **RENDER-001** | @coder-mcp | Headless **blender-worker** contract; MCP never ships greybox ortho as production | **pending** |
| **ATLAS-001** | @planner-mcp | State × Facing × Frame atlas + lookup validation (extends TILE-FIX-02) | **partial** |
| **RUNTIME-001** | @coder | `VisualState` resolver wired for all building stamps (extends TILE-FIX-03) | **partial** |
| **PILOT-001** | @designer-mcp + @coder-mcp | Warehouse through full spine; G4 on real stills | **pending** |

**Gate:** `warehouse_tile_ship_workflow_v1` may become **shipping** workflow only after **PILOT-001** passes via Suite → MCP → headless build.

---

## Assembly Editor (APS-UI-003b scope)

Planner redefines tab **Assembly** as full editor:

| Panel | Controls |
|:---|:---|
| Footprint grid | Slot selection |
| Selected slot | Module combobox, **Material** profile, **Tags** checkboxes, **Variants** checkboxes, **LOD** tier |
| Actions | Generate snapshot, validate materials, Send to Variants |

Materials are **first-class** on each node — not “hope Blender figures it out later.”

---

## Promotion gates (Phase 10)

`ship = true` only when **all** pass (not PNG exists):

| Gate | Rule |
|:---|:---|
| Geometry | Module count > minimum; real production GLBs |
| Materials | 100% profile resolution (albedo+normal+roughness or explicit fallback) |
| Variants | Required states generated from graph |
| Facings | All contract facings rendered |
| Atlas | Lookup validation (variant × facing × frame) |
| Runtime | Spawn / stamp test passes |
| Designer | G4 on stills (readable art, not schema green) |

---

## MCP compile loop (Phase 8)

Never render directly from greybox export.

```text
for variant in variants:
    apply_variant(variant)
    for facing in facings:
        set_rotation(facing)
        for frame in animation_frames:
            render()
```

Implemented target: `tile_compile_loop` / `variant_bake` calling bpy apply_materials → apply_variant → set_facing → render.

---

## Agents

| Lane | Agent |
|:---|:---|
| ARCH schemas, authority chain | @planner-mcp |
| APS Assembly Editor, build graph, blender-worker | @coder-mcp |
| Runtime resolver / stamp | @coder |
| G4 pilot, variant_set authoring | @designer-mcp |
| Freeze / program order | @planner |

**Orchestrator paste:**

```text
Execute BUILDING-TILE-SPINE-001 in planner order (ARCH-001→PILOT-001).
Primary path: Art Pipeline Suite → MCP → headless Blender.
warehouse_tile_ship_workflow_v1 = debug/recovery only until PILOT-001.
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Indexed planner_fix_auto_build.md post–line 400; supersedes SHIP-WH-001 as primary |
