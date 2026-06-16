# @coder — Vegetation full chain (drain to completion) `v1`

**Copy this entire section to @coder.** Do not stop until seq **DONE** or explicit operator block.

```text
⟦VEGETATION-FULL-CHAIN⟧  PROGRAM: VEGETATION-PROGRAM-001
Authority: tools/orchestrator/queues/coder_vegetation_drain_queue.json
Runbook:   src/dev/coder_vegetation_full_chain_prompt_v1.md (this file)
Charter:   src/dev/guide_landscape_grammar_v1.md
Exec:      src/dev/plan_landscape_grammar_exec_001_v1.md
Lexicon:   prompts/guides/landscape_grammar_lexicon_v1.md
Honest:    src/dev/vegetation_system_honest_status_v1.md

RULE: Lib witness green ≠ done. Drain seq 1→N top-down. Q✓ each row → next row.
      **Before marking done: read witness JSON — all exit_predicate.must fields must pass.**
      Do NOT re-plan. Do NOT stop after unit tests. Operator-visible = phase C gate.
      Keep going until seq 82 DONE or blocked on designer/MCP (then implement coder consumer stubs only).
```

---

## Mission

Ship the **full landscape vegetation chain** — not a pilot stub:

```text
LAND-DNA → λ PRESSURE → LANDSCAPE-PROGRAM → TOPOLOGY-GRAPH
    → SUCCESSION + DISTURBANCE-HISTORY
    → POPULATION-FIELDS → DETERMINISTIC-INSTANCES
    → PREVIEW TINTS (operator visible) → SPRITE EXTRACT (LG-5)
```

**Rejected forever:** Biome → density → green blob · one chunk pilot · JSON-only preview · global Tree ECS ∝ map area.

---

## What exists (extend, do not rewrite)

| Module | State |
|:---|:---|
| `src/systems/ecology/landscape_grammar.rs` | LG-1 evaluator · 1 preset · pilot chunk `(12,0)` |
| `src/systems/ecology/landscape_grammar_lg2.rs` | LG-2 components · lib witness · **0 live disturbances** |
| `src/systems/ecology/vegetation_field.rs` | Biome scalars still drive preview + fire |
| `assets/configs/landscape/presets/agri_riparian_v0.json` | Only preset on disk |

---

## Regression (every seq)

```powershell
cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology
BLANG:CARGO --cached --compress 4
```

**Done gate:** Open witness path from row `exit_predicate.witness` — verify every `must` field. If witness missing field, row is not done.

After phase C+: refresh witnesses from sim/FULL_APP where noted — not test-only JSON.

---

## Hardening (mandatory)

Read: `src/dev/coder_queue_hardening_rules_v1.md`

| Forbidden | Why |
|:---|:---|
| `cargo test` pass only | Witness counters may still be 0 |
| `lib_green: true` | Product `green` may still be false |
| Eval JSON topology counts | Preview raster may still be uniform green |
| Single chunk `(12,0)` | Map rollout requires ≥16 chunks |

---

## Full drain v3 — 82 rows (summary)

**Phase A (1–18):** Harness → fire event → fire witness green → build footprint → build witness → recovery → fuel → harvest → live proof wire → phase close

**Phase B (19–38):** 5 presets × (load + eval) → catalog → rollout Q1–Q4 → rollout witness ≥16 → λ hydro/transport/construction → partition → phase close

**Phase C (39–56):** Preview raster → shader → heterogeneity → preview witness operator_visible → population authority → play key → stage5 ecology_active_rows → phase close

**Phase D–G (57–82):** Districts → population/instances → snapshot → art close → steward

Full machine queue: `tools/orchestrator/queues/coder_vegetation_drain_queue.json`

---

## Full drain — do all rows in order (v2 archive below)

### Phase A — LG-2 sim truth (seq 1–6)

| Seq | ⟨ID⟩ | Do | Exit | Witness |
|:---:|:---|:---|:---|:---|
| 1 | **VEG-LG2-LIVE-FIRE-001** | Prove fire writes `DisturbanceHistory` + OldGrowth→BurnScar in **running sim** (scenario or harness) | `fire_disturbances >= 1` | `landscape_grammar_lg2_live.json` |
| 2 | **VEG-LG2-LIVE-BUILD-001** | Construction commit → disturbance on **footprint chunk entity** — delete first-entity stub | `construction_disturbances >= 1` | same |
| 3 | **VEG-LG2-RECOVERY-001** | BurnScar→Grass→Shrub ticks when heat clears | stage advances without new fire | lib test |
| 4 | **VEG-LG2-FUEL-BRIDGE-001** | `SuccessionTopologyStage` modulates `chunk_fuel_profile` + `VegetationField.old_growth` | fuel differs BurnScar vs OldGrowth | `fire_ecology_live.json` |
| 5 | **VEG-LG2-HARVEST-001** | Add `DisturbanceKind::Harvest` path (stub tick or scenario) | harvest event in history | lg2 witness |
| 6 | **VEG-WITNESS-LIVE-PROOF-001** | `src/dev/landscape_grammar_live_proof.rs` — refresh lg1/lg2 from FULL_APP path | not `#[cfg(test)]` only | `dev/mod.rs` wired |

### Phase B — Map authority (seq 7–14)

| Seq | ⟨ID⟩ | Do | Exit | Witness |
|:---:|:---|:---|:---|:---|
| 7 | **VEG-λ-INPUTS-001** | λ blend reads hydrology bands + transport proximity + construction sites (read-only) | `lambda_inputs_wired` | lg1 witness keys |
| 8 | **VEG-PRESET-INDUSTRIAL-001** | `industrial_barrier_v0.json` — BARRIER · POCKET · FRINGE · MOSAIC | schema valid | catalog load |
| 9 | **VEG-PRESET-MILITARY-001** | `military_defensive_v0.json` — RING_FORTIFIED · BARRIER · POCKET | schema valid | catalog load |
| 10 | **VEG-PRESET-SETTLEMENT-001** | `settlement_park_v0.json` — PATCH · FRINGE · CORRIDOR street trees | schema valid | catalog load |
| 11 | **VEG-PRESET-FOREST-001** | `old_growth_core_v0.json` — PATCH · NETWORK · NESTED depth ≥2 | schema valid | catalog load |
| 12 | **VEG-PRESET-CATALOG-001** | `assets/configs/landscape/_preset_index.ron` — index ≥5 presets | `presets_on_disk >= 5` | — |
| 13 | **VEG-MAP-ROLLOUT-001** | Attach `LandscapeProgramOnChunk` on **all** chunks — preset pick heuristic (biome/land-use/transport) | `chunks_with_program >= 16` | `landscape_grammar_map_rollout_live.json` |
| 14 | **VEG-MAP-PARTITION-001** | 3–8 topology nodes per chunk partition; nested depth ≥2 map-wide mean | `mean_topology_kind_count >= 3` | map rollout witness |

### Phase C — Operator visible (seq 15–22) ⚡ product gate

| Seq | ⟨ID⟩ | Do | Exit | Witness |
|:---:|:---|:---|:---|:---|
| 15 | **VEG-PREVIEW-TOPOLOGY-001** | World preview raster tints by `topology_kind` — **not** uniform `VegetationField` green | patches visible at zoom | `landscape_grammar_lg4_preview_live.json` |
| 16 | **VEG-PREVIEW-GLYPH-001** | Planning→extract glyph map (lexicon §3); debug overlay | same seed → same field | — |
| 17 | **VEG-PREVIEW-OVERLAY-001** | Diagnostics: fire corridor, wind, suppression toggles (LG-4-004) | overlay keys in diagnostics | — |
| 18 | **VEG-POPULATION-FIELD-001** | `VegetationPopulation` **authority** feeds `vegetation_field_tick` — demote biome-only path | canopy tracks graph density | lib test |
| 19 | **VEG-MINIMAP-OVERLAY-001** | Minimap ecology layer shows topology boundaries | not flat fill | minimap witness |
| 20 | **VEG-PLAY-WITNESS-001** | `play_scenario_live.json` — `veg_topology_visible_at_operational_zoom` | operator can see | `play_scenario_live.json` |
| 21 | **VEG-FULL-APP-WITNESS-001** | `stage5_full_app_live.json` ecology rows reference grammar topology count | `ecology_topology_wired` | stage5 witness |
| 22 | **VEG-OPERATOR-CHECKLIST-001** | Handoff operator — G-PLAY veg row | human sign-off | checklist doc |

### Phase D — District coupling (seq 23–28)

| Seq | ⟨ID⟩ | Do | Exit | Witness |
|:---:|:---|:---|:---|:---|
| 23 | **VEG-DISTRICT-SETTLEMENT-001** | `LandUseInfluence` from settlement district/block — **remove** `x%16` hack | preset matches usage | — |
| 24 | **VEG-DISTRICT-TRANSPORT-001** | Rail/road edge → CORRIDOR/FRINGE topology bias | adjacent chunks typed | — |
| 25 | **VEG-DISTRICT-CONSTRUCTION-001** | Site footprint → Fringe/Pocket clear + ⊖ on graph | footprint tiles disturbed | lg2 witness |
| 26 | **VEG-DISTRICT-HYDRO-001** | Riparian preset bias from hydrology graph (WSS read-only) | moisture λ correlates | `wss_substrate_live.json` cross |
| 27 | **VEG-LG3-WITNESS-001** | Ag + industrial + military presets resolve on test world | lg3 green | `landscape_grammar_lg3_live.json` |
| 28 | **VEG-SIM-EFFECT-HOOK-001** | `SimEffect` HydroDirty / IgniteCells → landscape disturbance adapter (waist pattern) | effect row in lg2 history | `sim_effect_spine_live.json` |

### Phase E — Population + instances (seq 29–34)

| Seq | ⟨ID⟩ | Do | Exit | Witness |
|:---:|:---|:---|:---|:---|
| 29 | **VEG-POPULATION-SUBCELL-001** | Subcell partition grid (coarse 4×4 per chunk) population scalars from graph | bounded array per chunk | — |
| 30 | **VEG-INSTANCE-SPAWN-001** | Deterministic instance seeds from budget — **no** Tree entity per cell | `instance_count <= budget` | — |
| 31 | **VEG-INSTANCE-EXTRACT-001** | Extract glyph per instance from lexicon §2 mapping | deterministic | — |
| 32 | **VEG-FIRE-CORRIDOR-001** | FIRE-CORRIDOR topology → ember spread bias (read-only) | spread counters wired | `fire_ecology_live.json` |
| 33 | **VEG-SUCCESSION-GRAPH-001** | Graph node stage transitions on disturbance (not scalar only) | node stage in witness | lg2 witness |
| 34 | **VEG-SNAPSHOT-PERSIST-001** | Save/load `SuccessionState` + `DisturbanceHistory` in world snapshot slice | round-trip test | construction/io pattern |

### Phase F — Art + terminal extract (seq 35–40)

| Seq | ⟨ID⟩ | Do | Exit | Owner |
|:---:|:---|:---|:---|:---|
| 35 | **VEG-DESIGN-ATLAS-001** | Request designer-mcp charter: corridor + patch + ring iso extracts | sign-off | designer-mcp **blocked** |
| 36 | **VEG-MCP-ATLAS-001** | Keyframe tile batch spec JSON for 3 topology sprites | batch spec on disk | coder-mcp **blocked** |
| 37 | **VEG-REGISTRY-STAMP-001** | Bevy registry load + chunk UV stamp from atlas meta | stamp visible in sim | coder_a |
| 38 | **VEG-LG5-WITNESS-001** | `landscape_grammar_lg5_live.json` — atlas + stamp green | lg5 green | — |
| 39 | **VEG-LG6-FLOWERS-001** | Flowers aesthetic layer hook (data only until designer) | deferred | designer-mcp |
| 40 | **VEG-PROGRAM-CLOSE-001** | Roll all witnesses; write `vegetation_program_close_live.json` | all phases A–E green | — |

### Phase G — Scale + vNext hooks (seq 41–45) — continue after F or parallel

| Seq | ⟨ID⟩ | Do | Exit |
|:---:|:---|:---|:---|
| 41 | **VEG-PRESET-EXPAND-001** | Add 5 more presets toward 30-id catalog (lexicon §5 table) | `presets_on_disk >= 10` |
| 42 | **VEG-NESTED-DEPTH-001** | Enforce nested topology depth ≥3 on pilot + industrial chunks | witness nested_depth_max≥3 |
| 43 | **VEG-OPERATOR-HISTORY-001** | DisturbanceHistory replay in diagnostics (read-only timeline) | diagnostics panel |
| 44 | **VEG-STEWARD-REGRESS-001** | sim-steward: stage5 + fire_ecology + landscape after close | replay parity green |
| 45 | **VEG-QUEUE-SYNC-001** | Mark all rows done in `coder_vegetation_drain_queue.json` + HANDOFF | queue drained |

---

## Territory (do not collide)

| Path | Owner |
|:---|:---|
| `src/systems/ecology/` | coder_a — grammar spine |
| `src/terrain/material/` · world preview | coder_a — preview tints |
| `src/systems/fire/chunk_fuel_profile.rs` | coder_a — fuel bridge |
| `src/construction/` (disturbance hook only) | coder_a — read commit events |
| `src/strategic/settlement/` | coder_a — district read |
| `src/gui/minimap_shell.rs` | coder_b — seq 19 only |
| `tools/mcp/` · tile bake | coder-mcp — seq 36 only |

---

## Forbidden

- `if preset_id == "agri_riparian_v0"` hardcode outside test fixtures
- Mark row done on lib test when exit says `operator_visible` or `fire_disturbances >= 1` in **live** witness
- Global `Tree` entity per map cell
- Biome-only `estimate_ecological_suitability` as final authority (grammar must win)
- Skip preview wiring and jump to MCP atlas
- Dev Postgres in `proc_A_dine01`

---

## Keep-going ritual

```text
BLANG:PRE → pick seq N → implement → witness refresh → Q✓ → seq N+1
```

On Task quota error: **continue in same chat** — do not stop.

On designer/MCP block (seq 35–36): implement seq 37 consumer stubs + spec JSON; note blocked in queue `note`; **continue** seq 29–34 and 41–45.

---

## Done definition (program close)

All must be true:

1. `play_scenario_live.json` — veg topology visible at operational zoom  
2. `landscape_grammar_lg2_live.json` — fire **and** construction disturbances > 0 from sim  
3. `landscape_grammar_map_rollout_live.json` — chunks_with_program ≥ 16  
4. `landscape_grammar_lg4_preview_live.json` — render wired, not eval-only  
5. `landscape_grammar_lg3_live.json` — district presets resolve  
6. Operator checklist row signed (seq 22)  
7. `cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology` green  

LG-5/6 (sprites/flowers) = **bonus** after 1–7; seq 37–38 if MCP unblocks.

---

## Single-line handoff

> Drain `coder_vegetation_drain_queue.json` seq 1→45 without stopping. Full chain: sim disturbances → map rollout → operator-visible preview → districts → population/instances → snapshot persist → atlas consumer. Lib green is not done. Witness JSON wins. Regression each seq.

```text
⟦/VEGETATION-FULL-CHAIN⟧  ΔWF→@coder A  START seq 1  VEG-LG2-LIVE-FIRE-001
```
