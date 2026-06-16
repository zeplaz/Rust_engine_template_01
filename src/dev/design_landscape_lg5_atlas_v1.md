# Landscape LG-5 minimal iso atlas `v1` (VEG-DESIGN-ATLAS-001)

| Field | Value |
|:---|:---|
| **Program** | **VEG-DESIGN-ATLAS-001** · **VEG-MCP-ATLAS-001** · **VEG-LG5-WITNESS-001** |
| **Date** | 2026-06-14 |
| **Owner** | `@designer-mcp` (charter) · `@coder-mcp` (batch) |
| **Verdict** | **PASS** |
| **Parent** | [`plan_landscape_grammar_exec_001_v1.md`](plan_landscape_grammar_exec_001_v1.md) §7 |
| **Lexicon** | [`prompts/guides/landscape_grammar_lexicon_v1.md`](../prompts/guides/landscape_grammar_lexicon_v1.md) §2 |
| **Witness** | [`debug_runs/landscape_grammar_lg5_live.json`](../debug_runs/landscape_grammar_lg5_live.json) |

**No Rust in this doc.** Locked extract map + MCP batch contract.

---

## Mission

Ship a **minimal deterministic iso atlas** for three topology kinds visible in LG-4 preview — Patch, Corridor, Ring — as terminal extract only. Same seed → same PNGs → same UV grid.

**Acceptance:** MCP tile batch G3 green · row in `assets/configs/landscape/_landscape_atlas_index.ron` · `landscape_grammar_lg5_live.json` green.

---

## 1. Variant map (topology → atlas key)

| Topology kind | Extract glyph (§2) | `variant_key` | Visual intent |
|:---|:---:|:---|:---|
| **Patch** | `#` mature | `topology_patch` | Dense green canopy diamond |
| **Corridor** | `=` transport spine | `topology_corridor` | Horizontal brown/green stripe |
| **Ring** | `()` enclosure | `topology_ring` | Gold ring on dark fringe |

---

## 2. Batch contract

| Field | Value |
|:---|:---|
| `batch_id` | `tile_landscape_lg5_pilot_v1` |
| `atlas_id` | `landscape_lg5_pilot_v1` |
| `tile_id` | `landscape_lg5` |
| `atlas_domain` | `landscape` (register `_landscape_atlas_index.ron`, not buildings) |
| `bake_source` | `keyframe_pack` |
| `ship` | `false` |
| `development_tier` | `pilot` |
| `render.seed` | `550005` |
| `tile_px` | `64` |
| Grid | 3×1 |

Spec: [`tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json`](../tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json)

---

## 3. Registry + stamp

| Surface | Path |
|:---|:---|
| Atlas PNG | `assets/textures/landscape/staging/landscape_lg5_pilot_v1_atlas.png` |
| Meta | `assets/staging/tiles/tile_landscape_lg5_pilot_v1/atlas_meta.json` |
| Index | `assets/configs/landscape/_landscape_atlas_index.ron` |
| MCP witness | `debug_runs/art_pipeline/tile_tile_landscape_lg5_pilot_v1_live.json` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-14 |
| `@coder-mcp` | **batch G3 green · landscape index registered** | 2026-06-14 |
