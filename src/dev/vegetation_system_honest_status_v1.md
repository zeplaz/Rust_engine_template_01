# Vegetation system — honest status `v1` (2026-06-17 refresh)

**Charter:** [`guide_landscape_grammar_v1.md`](guide_landscape_grammar_v1.md)  
**Exec:** [`plan_landscape_grammar_exec_001_v1.md`](plan_landscape_grammar_exec_001_v1.md)  
**Drain:** [`coder_vegetation_drain_queue.json`](../tools/orchestrator/queues/coder_vegetation_drain_queue.json)

---

## Executive verdict

| Layer | Lib witness | Normal play / map | Plain English |
|:---|:---:|:---:|:---|
| **LG-0** schema + lexicon | 🟢 | — | Docs/schemas done |
| **LG-1** evaluator | 🟢 | 🟡 | Map rollout ≥16 chunks in sim harness |
| **LG-2** succession + disturbance | 🟢 JSON | 🟡 | Harness: fire=1, construction=1, harvest=1 (SimEffect wire) |
| **LG-3** district programs | 🟢 | 🟡 | `presets_used>=3`; λ-driven preset pick (no coord hash) |
| **LG-4** population + preview | 🟢 | 🟡 | `pixel_heterogeneity_wired` + tint≥1; `proof_grade=headless_sim` |
| **LG-5/6** sprites / flowers | 🟡 pilot | 🔴 | Pilot stamp green; expanded atlas = @coder-mcp E4 |

**Honest gap:** lib harness + headless_sim witnesses are green; **`--test visual` ecology raster** remains `CDR-A-VISUAL-SMOKE-ECO-001` lib smoke + operator visual capture.

---

## Runtime proof upgrades (2026-06-17)

| Before | After |
|:---|:---|
| `patch_stage5_ecology_*` shortcut in harness refresh | **Removed** — `stage5_live_ecology_already_verified()` only (`CDR-A-ECOLOGY-HARNESS-CLEAN-001`) |
| Harvest via direct queue poke | **SimEffect** `LandscapeDisturbance.harvest` wire (`CDR-A-FIRE-HARVEST-WIRE-001`) |
| `vegetation_program_close` rollup dishonest | Child sub-rules + WIT-HON `proof_grade=headless_sim` |
| Coord-hash preset fallback | **λ-influence** scoring (`CDR-A-PRESET-PICK-LAMBDA-001`) |
| G-PLAY single `green` | **Split** `lib_contract_green` vs `operator_session_green` (`CDR-A-PLAY-OPS-SPLIT-001`) |

---

## Key witnesses (disk truth)

| Witness | Exit |
|:---|:---|
| `landscape_grammar_lg4_preview_live.json` | green, pixel_heterogeneity_wired, tint≥1 |
| `vegetation_program_close_live.json` | all_green + child_rollup true |
| `landscape_grammar_lg2_live.json` | harvest≥1, recovery≥1, nested_depth≥3 |
| `landscape_grammar_fire_harvest_wire_live.json` | fire + harvest via SimEffect |
| `landscape_grammar_visual_smoke_live.json` | lib_smoke_green (visual capture pending) |
| `play_scenario_live.json` | lib_contract_green + operator_session fields |
| `g_play_product_close_live.json` | `g_play_coder_rollup_green` (operator rollup separate) |

```text
[/vegetation_system_honest_status_v1]  ΔWF→ CDR-A-VISUAL-SMOKE visual capture · G-PLAY-OPERATOR-01
```
