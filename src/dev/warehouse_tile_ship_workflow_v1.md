# Warehouse tile workflow — debug / recovery `v1`

| Field | Value |
|:---|:---|
| **Status** | **DEBUG / RECOVERY ONLY** — not primary ship path |
| **Authoritative program** | [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) (**BUILDING-TILE-SPINE-001**) |
| **Primary path** | Art Pipeline Suite → MCP → headless Blender |
| **Pilot** | `industrial_west_4x2_s43_a879` / `warehouse_industrial` |

Per [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md): this document bypasses the intended **Suite → MCP → Headless Build** architecture. Use only when APS/MCP bake is blocked and you need to repair blends or export stills by hand.

**Becomes a shipping workflow only after PILOT-001** (warehouse through full spine + G4).

---

## When to use this doc

| Situation | Use |
|:---|:---|
| Normal warehouse pilot | [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) + `python tools/mcp/art_pipeline_suite/run.py` |
| Polluted assembly `.blend`, rig/truck embedded | Step 0 cleanup below |
| Emergency PNG export while RENDER-001 blocked | Steps 1–5 below |
| Product ship / registry promote | **Do not** — wait for PILOT-001 |

---

## Do not use for ship

| Path | Why |
|:---|:---|
| `tile_compile_minimum_bake.py` alone | Schema/CI — procedural slabs |
| `assets/textures/buildings_iso/production/*` | Frozen greybox |
| `Light_keysshotsetup.blend` daily | Legacy extract-only |
| `mcp_export_pilot_keyframes_g4.py` | Deprecated headless export |
| This doc as “authoritative ship” | Superseded by BUILDING-TILE-SPINE-001 |

---

## Recovery workflow (manual Blender)

### 0. Clean assembly blends

```powershell
cd C:\dev\github\Rust_engine_template_01
python tools/mcp/scripts/cleanup_assembly_blends.py
```

Optional: `--rebuild-only` · `--skip-rig`.

### 1. Open assembly — modules only

```text
assets/staging/assemblies/industrial_west_4x2_s43_a879.blend
```

Collection **`ASSEMBLY`** only. If polluted → step 0.

### 2. Assign materials (recovery)

Prefer fixing **`material_profile`** in module index + variant graph, then re-run MCP `assembly_build_job`.  
Manual Blender assign only when index/graph cannot be updated yet.

### 3. Append iso rig (recovery)

```text
utils/Tile_iso_rig_v1.blend → TILE_ISO_RIG
```

MCP default: `RUST_ENGINE_TILE_LIGHT_BLEND` → `Tile_iso_rig_v1.blend`.

### 4. keyframe_render.py → PNGs

Matrix: [`warehouse_state_facing_matrix_v1.yaml`](../../debug_runs/art_pipeline/warehouse_state_facing_matrix_v1.yaml).  
Minimum G4 reference: 24 cells (6 states × 4 facings) in `visual_config_warehouse_industrial_west_v2.json`.

### 5. Pack (after G4)

```powershell
python -m rust_engine_mcp.cli tile-atlas-pack path\to\png\folder -pk
```

Staging only until designer G4 + full promotion gates.

---

## Related

| Doc | Role |
|:---|:---|
| [`plan_building_tile_spine_001_v1.md`](plan_building_tile_spine_001_v1.md) | **Primary** program |
| [`design_art_pipeline_suite_v1.md`](design_art_pipeline_suite_v1.md) | Suite UX + MCP parity |
| [`tile_greybox_production_frozen_v1.md`](tile_greybox_production_frozen_v1.md) | Frozen atlases |
