# Vegetation system — honest status `v1` (2026-06-14)

**Charter:** [`guide_landscape_grammar_v1.md`](guide_landscape_grammar_v1.md)  
**Exec:** [`plan_landscape_grammar_exec_001_v1.md`](plan_landscape_grammar_exec_001_v1.md)  
**Drain:** [`coder_vegetation_drain_queue.json`](../tools/orchestrator/queues/coder_vegetation_drain_queue.json)

---

## Executive verdict

| Layer | Lib witness | Normal play / map | Plain English |
|:---|:---:|:---:|:---|
| **LG-0** schema + lexicon | 🟢 | — | Docs/schemas done |
| **LG-1** evaluator | 🟢 | 🟡 | Map rollout ≥16 chunks in sim harness; pilot preset still primary QA anchor |
| **LG-2** succession + disturbance | 🟢 JSON | 🟡 | Harness proves fire + construction disturbances > 0 |
| **LG-3** district programs | 🟡 stub | 🔴 | `x % 16 == 7` hack, not settlement coupling |
| **LG-4** population + preview | 🟡 tint proof | 🟡 | Topology tint bias on ≥2 program chunks in sim harness — **not** FULL_APP pixel proof |
| **LG-5/6** sprites / flowers | — | 🔴 | MCP-blocked (correctly deferred) |

**Honest gap:** lib witnesses now require **live `LandscapeProgramOnChunk` query** + **topology tint bias** on multiple chunks — but **FULL_APP / `--test visual` pixel confirmation** is still a separate operator gate (VEG-C14).

---

## Runtime proof upgrades (2026-06-14)

| Before | After |
|:---|:---|
| `stage5` ecology rows from harness struct injection | `live_landscape_program_chunk_count_after_harness()` + `ecology_rows_source: live_landscape_program_on_chunk` |
| `ClimateVisualAggregate.ecology_chunk_count` = `ChunkEcology` only | Prefers `LandscapeProgramOnChunk` count when programs exist (atmosphere visual extract) |
| Play veg key = pilot eval + `topology_kind_count >= 3` | Requires `topology_tint_bias > 0` on **≥2** program chunks in sim harness |
| LG-4 `operator_visible` = eval math | `lg4_preview_operator_visible(tint_chunks, eval)` when tint count supplied |

---

## Witness vs truth (on disk)

| JSON | Key fields | Read as |
|:---|:---|:---|
| `landscape_grammar_sim_harness_live.json` | `chunks_with_program >= 16`, `topology_tint_visible_chunks >= 2` | Sim harness green |
| `landscape_grammar_lg4_preview_live.json` | `topology_tint_visible_chunks` | Tint proof, not pixel heterogeneity |
| `play_scenario_live.json` | `veg_topology_visible_at_operational_zoom` | Sim harness tint + zoom coherence |
| `stage5_full_app_live.json` | `ecology_active_rows`, `ecology_rows_source` | Live program count (harness-fed until FULL_APP extract wired end-to-end) |
| `vegetation_program_close_live.json` | `phase_f_green` | Presets on disk ≥10 + phases A–E |

---

## Still blocked (not coder fault)

| ID | Blocker |
|:---|:---|
| VEG-F01 / F02 | designer-mcp → coder-mcp atlas |
| VEG-C14 | operator `--test visual` sign-off |
| MCP P2 | planner SIGN + coder-mcp chain |

---

## Remaining hardening (next coder slices)

1. **FULL_APP extract** — `EcologyVisualSnapshot` populated from running sim `publish_climate_visual_aggregate` (not harness patch-only refresh).
2. **λ / districts** — replace coord heuristics with hydrology/transport/construction reads.
3. **SIM-STEWARD-FIRE-REGRESS-001** — fire after veg visible in combined lib regression.

**Queue:** v3 drain **78 done · 3 blocked · 1 deferred · 0 ready** (coder-implementable clear).
