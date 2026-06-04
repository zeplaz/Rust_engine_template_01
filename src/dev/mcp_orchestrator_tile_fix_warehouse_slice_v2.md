# Orchestrator slice — TILE-FIX warehouse v2 `v2` (revoked headless ship)

| Field | Value |
|:---|:---|
| **Slice ID** | **TILE-FIX-WAREHOUSE-MIN-G4-001** |
| **Supersedes** | Phase B/C in [`mcp_orchestrator_tile_fix_warehouse_slice_v1.md`](mcp_orchestrator_tile_fix_warehouse_slice_v1.md) § headless bake as ship |
| **Date** | 2026-06-03 |
| **Owner** | `@orchestrator-mcp` |
| **Status** | **ACTIVE** — sequencing only, **no** automated headless bake |

---

## Bottom line (user / orchestrator)

The v2 minimum-G4 atlas under `assets/staging/tiles/tile_warehouse_industrial_v2_minimum_g4/` is a **plumbing proof** (schema v2 + 24 facings + promotion JSON). It is **not** the warehouse tile set for ship — same class of problem as greybox v1, more cells and a fancier witness.

| Decision | State |
|:---|:---|
| `tile_compile_minimum_bake` / headless `tile_keyframe_bake` as **ship art** | **STOP** |
| Warehouse v2 on active index | **DE-INDEXED** (`_tile_atlas_index.ron` entries: `[]`) |
| `tile_fix_10_warehouse_industrial_live.json` `green` | **REVOKED** (`art_quality: rejected_headless_procedural`) |
| `proceed_ship` on designer signoffs tied to TILE-FIX-10 schema | **REVOKE** until manual keyframe G4 |

**Plumbing may stay green** (`promotion_validation.status: passed`) — that does **not** authorize register or `buildings_iso/production/`.

---

## Correct sequence (only path to ship)

```text
Production shell GLBs (wall_steel + roof_sawtooth)     [done if shell_production_ready]
        ↓
Manual keyframe_render on assembled.blend
  → 24 minimum cells (3 states × 8 facings) as real 128px PNGs
  → export folder (NOT headless procedural grid)
        ↓
tile-atlas-pack (or pack v2 from folder) on those PNGs
        ↓
@designer-mcp  G4 on the actual stills (truck / Light_keysshotsetup spine)
        ↓
@coder-mcp     --register ONLY if designer proceed_ship: yes + witness art_quality: keyframe_manual
        ↓
@coder         map stamp smoke
```

**Blend:** `assets/staging/assemblies/industrial_west_4x2_s43_a879.blend` (per `building_definition_warehouse_industrial_west_production_v1.json`)

**Spine:** [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) — **keyframe_render** = ship; headless ortho/procedural = debug only.

---

## Role owners

| Role | Owns |
|:---|:---|
| **You** | Approve slice; run plan-only / read witnesses; **do not** treat staging grid PNG as ship |
| **@orchestrator** | This doc — sequence only |
| **@planner-mcp** | Job IDs + matrix + visual_config contract (readonly) |
| **@coder-mcp** | Shell GLBs, bdef/snapshot, **pack/register tooling** — **not** headless ship bake |
| **@designer-mcp** | G4 on **manual** keyframe stills |
| **@coder** | Index + map stamp after G4 |

---

## Forbidden

- `python tools/mcp/scripts/tile_compile_minimum_bake.py` **without** `--plan-only` as ship path
- `--register` on headless v2 minimum_g4 atlas
- `mcp_export_pilot_keyframes_g4` or greybox v1 re-promote
- Marking `proceed_ship: yes` because `validate_tile_promotion` passed on procedural PNGs
- Another automated headless bake “to fix” art

**Allowed (debug):** `--plan-only`, pytest promotion gates, staging folder for schema diff.

---

## Exit witness (revised)

| Witness | Pass when |
|:---|:---|
| `tile_fix_10_warehouse_industrial_live.json` | `green: true` **and** `art_quality: keyframe_manual` (not schema-only) |
| Designer signoff | `proceed_ship: yes` on **manual** still review |
| `_tile_atlas_index.ron` | One v2 row after above |

---

## Paste — @orchestrator (internal)

> TILE-FIX warehouse: headless v2 revoked. Index empty. Next = manual keyframe_render 24 cells on assembled.blend → designer G4 → register. No automated minimum bake for ship.

---

## Paste — @coder-mcp

> TILE-FIX warehouse v2 — **do not** run tile_compile_minimum_bake for ship art.
> Read mcp_orchestrator_tile_fix_warehouse_slice_v2.md.
> Shell GLBs + bdef already OK if shell_production_ready.
> Support: pack/register **after** designer exports manual keyframe PNGs to a folder.
> pytest promotion gates OK; staging atlas is debug-only.

---

## Paste — @designer-mcp

> TILE-FIX warehouse — G4 on **manual** keyframe_render stills (3 states × 8 facings), NOT headless v2 grid in tile_warehouse_industrial_v2_minimum_g4/.
> tile_fix_10 schema green is **revoked** for ship — set proceed_ship: no until real stills reviewed.
> Rubric: design_procedural_tile_production_bar_v1.md + design_tile_bake_spine_convergence_v1.md.

---

## Paste — @coder

> Blocked until designer proceed_ship on **manual** keyframe atlas.
> Then: register one _tile_atlas_index.ron row + map_tile_atlas_stamp smoke with rotation_quarter_turns.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v2.0.0 | 2026-06-03 | Revoke headless ship; manual keyframe path only |
