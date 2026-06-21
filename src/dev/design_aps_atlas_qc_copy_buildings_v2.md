# APS atlas QC — plain-language copy (buildings domain) `v2` — DMCP-ATLAS-QC-PLAIN-002

| Field | Value |
|:---|:---|
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Supersedes** | landscape-only [`design_aps_atlas_qc_copy_v1.md`](design_aps_atlas_qc_copy_v1.md) (DMCP-ATLAS-QC-PLAIN-001) |
| **Wires to** | `aps_atlas_qc.py` · `atlas_meta_brief.py` |
| **Batches** | warehouse · shopfront · bunker (production + pilot) |

---

## Building domain messages

| Code | Artist-facing sentence | Fix hint |
|:---|:---|:---|
| `atlas_meta_missing` | “Atlas metadata file is missing.” | Run **Pack atlas** after tile-batch-run |
| `atlas_meta_v2_parse` | “Could not read atlas_meta.json — check JSON syntax.” | Compare with a known-good production meta |
| `atlas_meta_v2_version` | “Atlas meta must be schema version **2** — v1 greybox is frozen.” | Re-pack from production batch spec |
| `atlas_meta_v2_facings` | “render_contract.facings must be **4 or 8** for iso lookup.” | Match Tile_iso_rig_v1 facing count |
| `atlas_meta_v2_lookup_incomplete` | “Some variant/facing/frame cells are missing from lookups.” | Re-run tile-atlas-pack or fix variant_set |
| `variant_png_missing` | “Missing PNG for variant **{key}**.” | Export keyframe still to staging folder |
| `uv_grid_gap` | “UV grid has a hole at column {col}, row {row}.” | Re-pack atlas or fix variant list |
| `ship_false_pilot` | “Pilot atlas — **not** a ship target (`ship: false`).” | Open production batch JSON before register |
| `warehouse_footprint` | “Warehouse batch expects **6×3** footprint — meta grid mismatch.” | Check `tile_warehouse_industrial_west_production_v1` |
| `shopfront_footprint` | “Shopfront batch expects **4×3** footprint — meta grid mismatch.” | Check `tile_shopfront_colonial_production_v1` |
| `bunker_footprint` | “Bunker batch expects **6×3** military footprint — meta grid mismatch.” | Check `tile_bunker_military_production_v1` |
| `burn_frame_gap` | “Fire sequence missing frame **{n}**.” | Add `burning_{nn}` keyframe per variant matrix |
| `damage_frame_gap` | “Damage state missing for **{key}**.” | Add `damaged_*` row per production matrix |
| `style_pack_mismatch` | “Variant style_pack_id does not match batch charter.” | Align with `style_industrial_west` / `style_colonial` / `style_military` |
| `buildings_index_ok` | “Building atlas looks complete — safe toward tile-atlas-register.” | Register via `_tile_atlas_index.ron` |
| `honest_gate_fail` | “Bake witness is dishonest — do not register.” | Re-run batch without dry-run stub PNGs |

---

## Batch-specific QC notes

| Batch | `batch_id` | Footprint | Min variant lanes |
|:---|:---|:---:|:---|
| **Warehouse** | `tile_warehouse_industrial_west_production_v1` | 6×3 | clean · night · damaged · burning sequence |
| **Shopfront** | `tile_shopfront_colonial_production_v1` | 4×3 | clean · night · damaged · burning sequence |
| **Bunker** | `tile_bunker_military_production_v1` | 6×3 | clean · night · damaged · burning sequence |

Pilot batches (`*_pilot_v1`) use **`ship: false`** — messages must say “pilot teach” not “production ship”.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-02 |

```text
DMCP-ATLAS-QC-PLAIN-002 Q✓ — warehouse · shopfront · bunker copy locked
```
