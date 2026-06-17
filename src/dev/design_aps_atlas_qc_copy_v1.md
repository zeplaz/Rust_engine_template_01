# APS atlas QC — plain-language copy (landscape domain) `v1` (DMCP-ATLAS-QC-PLAIN-001)

| Field | Value |
|:---|:---|
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **Wires to** | `aps_atlas_qc.py` · `atlas_meta_brief.py` |

---

## Landscape domain messages

| Code | Artist-facing sentence | Fix hint |
|:---|:---|:---|
| `atlas_meta_missing` | “Atlas metadata file is missing.” | Run tile batch pack first |
| `variant_png_missing` | “Missing PNG for variant **{key}**.” | Export keyframe still to staging folder |
| `uv_grid_gap` | “UV grid has a hole at column {col}, row {row}.” | Re-pack atlas or fix variant list |
| `frozen_batch` | “This batch is frozen for greybox — use expanded v1 spec.” | Open `tile_batch_landscape_expanded_v1.json` |
| `honest_gate_fail` | “Bake witness is dishonest — do not register.” | Re-run batch without dry-run stub PNGs |
| `landscape_index_ok` | “Landscape atlas looks complete — safe to register.” | Run `tile-atlas-register` for landscape domain |
| `pilot_teach` | “Pilot teach atlas — not a ship target.” | Set `ship: false` in batch JSON |
| `burn_frame_gap` | “Fire sequence missing frame **{n}**.” | Add `burn_{nn}` keyframe per matrix |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |
