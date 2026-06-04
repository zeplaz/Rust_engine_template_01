# Design — Art Pipeline Suite (APS) `v1`

| Field | Value |
|:---|:---|
| **ID** | **DESIGN-APS-001** |
| **Replaces (conceptually)** | “Module Kit Viewer” only — becomes **stage 1** of the suite |
| **Owner** | `@planner-mcp` + `@designer-mcp` · build **`@coder-mcp`** |
| **Date** | 2026-06-02 |
| **Status** | **SIGNED** |
| **Parents** | [`design_assembly_to_tile_coupling_v1.md`](design_assembly_to_tile_coupling_v1.md) · [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) |
| **Spine program** | [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) |

---

## Authority chain (planner)

```text
Asset Definition → Assembly Snapshot → Variant Graph → Build Graph → Atlas
```

Suite + MCP edit layers 1–3; headless Blender compiles layers 4–5. See [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) (post–line 400). Manual Blender = debug only ([`warehouse_tile_ship_workflow_v1.md`](warehouse_tile_ship_workflow_v1.md)).

---

## Vision

One desktop + MCP surface for the full art path:

```text
Catalog (modules) → Assembly (procedural building) → Variants (USD-like) → Atlas (tile maps)
```

**Not** four disconnected tools. Same app, same variant IDs, same witnesses — agents and humans use identical contracts.

**Launch (transitional):** `python tools/mcp/module_viewer/run.py`  
**Target package:** `tools/mcp/art_pipeline_suite/` (rename when APS-UI-001 lands; keep `run.py` shim).

---

## Four workspaces

| # | Workspace | Input | Output | Primary user |
|:---:|:---|:---|:---|:---|
| **1** | **Catalog** | `_module_index`, GLBs | validate, edit sidecar, preview mesh | artist / agent |
| **2** | **Assembly** | StylePack + footprint + module slots | `assembly_snapshot.json`, `.blend` (headless) | artist / agent |
| **3** | **Variants** | assembly + **variant_set** | per-variant PNG layers / bake jobs | artist / agent |
| **4** | **Atlas** | variant PNG folder | `tile_map_*.png`, `atlas_meta.json`, index row | artist / agent |

**Flow buttons (suite-level):**

- **Send to Assembly** — selected modules + style pack → open Assembly workspace with slots prefilled  
- **Bake variants** — Assembly → Variants workspace, expand `variant_set` to job list  
- **Pack atlas** — Variants → Atlas (`tile_atlas_pack` / full `tile_batch_run`)

Every button calls the **same** `rust_engine_mcp` functions as MCP agents.

---

## Variant model (USD-like, controlled)

We do **not** fork Blender files per variant. We use a **`variant_set_v1`** document: declarative axes + optional overrides + tags.

### Layers (stack order)

| Layer | What it controls | Example keys |
|:---|:---|:---|
| `base` | Mesh + materials from assembly | `style_pack_id`, `assembly_id` |
| `material` | PBR swaps | `wall_material: brick_red_01` |
| `lighting` | Scene lights + emissive | `power: on`, `night_lights: true`, `emissive_strength: 0.8` |
| `damage` | Overlays / geo decimation | `damage: 0.45`, `state: damaged` |
| `fill` | Interior occupancy visual | `fill: half` |
| `sim` | Read-only mirror of sim state (optional) | `SitePhase: UnderConstruction` |

**Variant = named composition of layer values** (like USD variant sets), not a duplicate scene file.

### Example

```ron
(
    schema_version: 1,
    variant_set_id: "rowhouse_victorian_night_damage",
    assembly_id: "rowhouse_victorian_4x3_s42",
    axes: (
        state: ["clean", "dirty", "damaged", "ruined"],
        power: ["off", "partial", "on"],
        fill: ["empty", "half", "full"],
        lighting: ["day", "night_off", "night_on"],
    ),
    variants: [
        (
            variant_key: "clean_day",
            tags: ["default", "stylepack_victorian"],
            layers: (lighting: (lighting: "day", power: "off"), damage: (state: "clean", damage: 0.0)),
        ),
        (
            variant_key: "damaged_night_on",
            tags: ["sim_night", "power_grid_on"],
            layers: (lighting: (lighting: "night_on", power: "on", night_lights: true), damage: (state: "damaged", damage: 0.45)),
        ),
    ],
)
```

**Deterministic bake key** (same as tile_batch):

```text
{assembly_id}_{variant_key}
```

### Tags (manual + agent)

| Tag kind | Use |
|:---|:---|
| `stylepack_*` | Filter variants per StylePack |
| `sim_*` | Map sim → visual without hardcoding in engine yet |
| `agent_request_*` | “needs artist review” / “auto_bake_ok” |
| `user_*` | Freeform bookmarks |

**UI:** tag editor on variant row; filter catalog/assemblies by tag.

**Agent:** MCP `variant_set_patch` — JSON Patch on layers/tags; never raw Blender instructions.

---

## Manual vs agent control

| Action | Manual (suite UI) | Agent (MCP) |
|:---|:---|:---|
| Preview module GLB | Catalog → browser / trimesh | `validate_asset_report` |
| Build assembly | Assembly → footprint grid + slot picks | `assembly_snapshot_generate` |
| Toggle lights on | Variants → layer `lighting` dropdown | `variant_set_patch` path `/variants/…/layers/lighting` |
| Change brick color | Variants → `material` layer | `variant_set_patch` + re-bake job |
| “Fix night lights” | **Request agent** button → copies context JSON to clipboard / opens Cursor task | `variant_agent_request` returns suggested patch |
| Bake all variants | Variants → **Run bake** | `tile_batch_run` |
| Pack atlas | Atlas → **Pack** | `tile_atlas_pack_tool` |

**Agent callback contract (`variant_agent_request`):**

```json
{
  "assembly_id": "…",
  "variant_key": "damaged_night_on",
  "intent": "add_warm_window_lights",
  "current_layers": { "lighting": { "power": "on" } },
  "constraints": ["lod0_tier", "deterministic_seed_42"],
  "reference_tags": ["US_rowhouse_survey_12"]
}
```

Response: **`variant_set_patch`** proposal (agent applies via MCP after human optional approve in UI).

---

## Suite UI map (coder-mcp APS-UI phases)

| Phase | UI |
|:---|:---|
| **APS-UI-001** | Rename window → “Art Pipeline Suite”; tab bar: Catalog \| Assembly \| Variants \| Atlas |
| **APS-UI-002** | Catalog = current module viewer body |
| **APS-UI-003** | Assembly: load StylePack RON, W×D grid, slot → module_id, **Generate snapshot** |
| **APS-UI-003b** | **Assembly Editor** — module + material + tags + variant + LOD per slot; validation (extends 003) |
| **APS-UI-004** | Variants: load/edit `variant_set_v1`, layer inspectors, tag chips, live preview thumb (last bake) |
| **APS-UI-005** | Atlas: batch picker, progress log, atlas preview |
| **APS-UI-006** | **Request agent** + **Apply patch** strip (validation-first) |

**Debug only:** `RUST_ENGINE_ART_DEBUG_GUI=1` → Blender GUI buttons hidden by default.

---

## MCP / CLI tools (suite backend)

Extends [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md):

| Tool | Stage |
|:---|:---|
| `assembly_snapshot_generate` | Assembly |
| `assembly_build_job` | Assembly |
| `variant_set_validate` | Variants |
| `variant_set_patch` | Variants |
| `variant_bake` | Variants (single variant_key) |
| `variant_agent_request` | Variants (stub → returns patch template; optional LLM external) |
| `tile_batch_run` | Atlas (batch of variants) |
| `tile_atlas_pack_tool` | Atlas |
| `lod0_batch_run_tool` | Catalog (module factory) |

**Rule:** Suite UI never imports bpy; only calls CLI/MCP Python API.

---

## Data on disk

| Artifact | Path |
|:---|:---|
| Module GLB | `assets/models/modules/<job_id>/model.glb` |
| Assembly snapshot | `assets/staging/assemblies/<assembly_id>.json` |
| Assembly blend (cache) | `assets/staging/assemblies/<assembly_id>.blend` |
| Variant set | `assets/staging/variants/<variant_set_id>.ron` |
| Variant PNG | `assets/staging/tiles/<assembly_id>/<variant_key>.png` |
| Atlas | `assets/textures/tiles/<batch_id>_atlas.png` |
| Witness | `debug_runs/art_pipeline/<batch_or_set>_live.json` |

---

## Relationship to engine

| Engine | Suite feeds |
|:---|:---|
| PG-2 extract | Same `assembly_snapshot.json` |
| Sim state | Maps to `variant_key` via tags + axes (later: `sim_variant_resolver`) |
| Bevy atlas | `_tile_atlas_index.ron` — **@coder** after Atlas witness green |

---

## Implementation order (orchestrator)

```text
1. AUTO-001…011 (headless bake pipeline)     ← coder-mcp active
2. APS-UI-001…002 (suite shell + catalog)    ← coder-mcp
3. variant_set_v1 schema + validate          ← planner-mcp DONE draft
4. APS-UI-003…006                            ← coder-mcp
5. variant_agent_request stub                ← coder-mcp
6. PG-2 + sim resolver                       ← coder
```

**Do not** build Variants UI before `variant_set_v1` schema + `variant_bake` exists.

---

## Acceptance (suite v1)

1. User opens suite → Catalog → validates module → **Send to Assembly** → snapshot JSON exists.  
2. User edits variant “night_on” lights in UI → **Bake** → PNG exists without opening Blender.  
3. Agent runs `variant_set_patch` + `variant_bake` → same PNG hash as UI.  
4. **Pack atlas** → atlas + meta; witness green.  
5. No production path requires Blender GUI.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | APS design — module viewer → four-workspace suite + variant_set |
