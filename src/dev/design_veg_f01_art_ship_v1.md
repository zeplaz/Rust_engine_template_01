# VEG-F01 art-ship gate — LG-5 expanded atlas `v1` (DMCP-VEG-F01-ART-SHIP-001)

| Field | Value |
|:---|:---|
| **Program** | **VEG-F** · **VEG-F01-DESIGN-ATLAS-001** |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **Parent** | [`design_landscape_lg5_atlas_v1.md`](design_landscape_lg5_atlas_v1.md) |

---

## Mission

Define **art-ship** vs **schema/bake/runtime** green for LG-5 expanded atlas (G4/G5). Three greens vocabulary per `PLAN-THREE-GREENS-VOCAB-001`.

---

## Gate table

| Gate | Owner | Pass when |
|:---|:---|:---|
| **G0** | designer-mcp | [`landscape_expanded_g0_rules.yaml`](../debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml) `proceed_production_bake: yes` |
| **G1** | designer-mcp | Matrix + keyframe reqs signed (this wave) |
| **G2** | coder-mcp | Keyframe PNGs ≥16 real pixels · `tile_batch_validate` pass |
| **G3** | coder-mcp | `tile_batch_run` G3 · atlas_meta on disk |
| **G4** | designer-mcp | Manual keyframe review · `proceed_ship: yes` on signoff YAML |
| **G5** | coder-mcp | `_landscape_atlas_index.ron` row · `landscape_grammar_lg5_live.json` `program_rollup_green: true` |

---

## Art-ship green (rollup)

All required:

| # | Criterion | Witness |
|:---:|:---|:---|
| A1 | Expanded batch **not** `frozen` when ship | tile_batch JSON |
| A2 | `development_tier: production` or explicit `pilot` teach with `ship: false` | batch JSON |
| A3 | `honest_gate` ≠ `dishonest_gate` | art_pipeline witness |
| A4 | G4 minimum 3 keys reviewed | keyframe reqs §3 |
| A5 | Registry stamp + chunk UV resolve | `landscape_grammar_lg5_live.json` |

**Pilot-only teach:** 3-tile pilot may stay `ship: false` while expanded v1 pursues art-ship.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |
