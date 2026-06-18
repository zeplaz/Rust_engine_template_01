# LG-5 expanded atlas bake charter `v1` — DMCP-LG5-EXPAND-BAKE-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-LG5-EXPAND-BAKE-001** |
| **Program** | APS-E4 · landscape production |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Authority** | [`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) · [`design_landscape_keyframe_burn_reqs_v1.md`](design_landscape_keyframe_burn_reqs_v1.md) · [`design_veg_burn_visual_language_v1.md`](design_veg_burn_visual_language_v1.md) |
| **Batch** | [`tile_batch_landscape_expanded_v1.json`](../assets/staging/specs/tile_batch_landscape_expanded_v1.json) |
| **Verdict** | **PASS WITH NOTES** — teach bake authorized · G4 manual still required for `proceed_ship: yes` |

```yaml
order_critique:
  request_summary: "Execute expanded 16-cell LG-5 bake without dishonest ship"
  rules_audit:
    no_ai_generated_images: pass
    deterministic_output: pass
    batch_processing: pass
    grid_alignment: pass
  blocked: false
  proceed: yes_with_notes
  note: "Procedural keyframes = schema/G3 lane only — not G4 art-ship"
```

---

## 0. Two-phase bake (honest)

| Phase | Owner | Keyframes | `ship` | Unlocks |
|:---|:---|:---|:---:|:---|
| **A — Teach bake** | @coder-mcp | Procedural PNGs in `keyframe_stills/tile_landscape_expanded_v1/` | `false` | `tile_batch_run` · registry dry-run · APS States parity |
| **B — Art ship** | @designer-mcp + operator | Blender rig `Tile_iso_rig_v1.blend` per reqs doc | `true` | `DMCP-VEG-ATLAS-SHIP-001` · engine LG-5 consumer |

**Do not** mark G4 green on Phase A alone.

---

## 1. Phase A execution checklist (@coder-mcp)

```powershell
cd tools/mcp/python
python -m rust_engine_mcp.landscape_lg5_expanded_batch
python -m rust_engine_mcp.cli validate-report tile_batch assets/staging/specs/tile_batch_landscape_expanded_v1.json
```

| Step | Pass when |
|:---|:---|
| Keyframes written | 16 files `{variant_key}.png` under `assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1/` |
| Pilot reuse | `topology_patch`, `topology_corridor`, `topology_ring` may symlink/copy pilot PNGs |
| Batch run | `debug_runs/art_pipeline/tile_tile_landscape_expanded_v1_live.json` green |
| Rollup | `debug_runs/art_pipeline/tile_landscape_expanded_v1_live.json` · `png_count > 3` |
| G0 | [`landscape_expanded_g0_rules.yaml`](../debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml) `proceed_production_bake: yes` |
| Honesty | `honest_gate` ≠ `dishonest_gate` on batch witness |

---

## 2. Phase B — G4 minimum review (@designer-mcp + operator)

Per [`design_landscape_keyframe_burn_reqs_v1.md`](design_landscape_keyframe_burn_reqs_v1.md) §3:

1. `topology_patch_burn_04`
2. `topology_patch_scar`
3. `topology_corridor_regrowth_grass`

Plus readability cross-check against [`design_veg_burn_visual_language_v1.md`](design_veg_burn_visual_language_v1.md) at **64px iso**.

| Criterion | Fail if |
|:---|:---|
| Burn vs scar | Same hue family at thumbnail |
| Corridor spine | Indistinguishable from patch fill |
| Regrowth | Reads as clean mature canopy |

**Sign-off artifact:** `debug_runs/art_pipeline/landscape_expanded_g4_signoff.yaml` with `proceed_ship: yes|no`.

---

## 3. Registry flip (`ship: true`)

When Phase B passes:

| Field | Before | After |
|:---|:---|:---|
| `tile_batch_landscape_expanded_v1.json` → `ship` | `false` | `true` |
| `development_tier` | `pilot` | `production` |
| `_landscape_atlas_index.ron` | teach row | production row with `atlas_id: landscape_lg5_expanded_v1` |

Blocks: **DMCP-VEG-ATLAS-SHIP-001** rollup A1–A5.

---

## 4. State row coverage (16 cells)

All cells in matrix §3 must pack into **4×4** grid — no orphan keys. Catalog authority: [`_vegetation_variant_catalog.ron`](../assets/configs/landscape/_vegetation_variant_catalog.ron) (35 rows post E3).

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS WITH NOTES** | 2026-06-02 |

```text
DMCP-LG5-EXPAND-BAKE-001 — Phase A authorized · Phase B G4 manual required
```
