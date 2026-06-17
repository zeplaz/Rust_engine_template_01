# Vegetation program queue - witness reconciliation v1 (2026-06-16)

**Analyst:** operations-intelligence (read-only). Method: classify each v2_false_done_reopened row against its CURRENT on-disk exit-predicate witness. Trust the witness + files on disk, NOT the row status. Hardening rule applied: lib green != done (require the live exit-predicate witness with non-zero counters, not just a passing lib test).

> READ-ONLY. This doc proposes an edit list for operator approval. No queue / snap / status-doc was modified.

## TL;DR - two-sided drift is real and now mostly resolved on disk

- The _meta.reopen_audit snapshot (2026-06-13) recorded: lg2 fire=0/build=0 green:false, map_rollout chunks=1 green:false, stage5 eco=0. Honest over-claims AT THE TIME.
- Every one of those counters is now non-zero and green:true on disk. Witnesses refreshed 2026-06-17T00:39-00:40Z, AFTER the reopen audit (2026-06-13) and the honest-status doc (2026-06-14). The reopen list is now mostly UNDER-claiming (stale-blocked).
- Of 41 reopened rows: majority DONE-CONFIRMED by live witness; a couple STALE-BLOCKED; a few GENUINELY-OPEN (operator/MCP gates or no witness); a few AMBIGUOUS (multi-scope green or short predicate clause).

## Witness freshness timeline (epoch-decoded)

| Witness | UTC written | vs audit (2026-06-13) |
|:---|:---|:---|
| reopen_audit snapshot (queue _meta) | 2026-06-13 | baseline (now stale) |
| honest-status doc | 2026-06-14 | between |
| fire_ecology_live.json | 2026-06-15T14:29Z | fresher |
| replay_editor_parity_live.json | 2026-06-16T21:36Z | fresher |
| mcp_landscape_sign_atlas / tile lg5 pilot | 2026-06-17T00:08Z | fresher |
| lg2 / lg3 / lg4 / lg5 / extract / program_close | 2026-06-17T00:39Z | authoritative |
| map_rollout / stage5 / harness / play / snapshot | 2026-06-17T00:39-47Z | authoritative |
| veg_runtime_proof | 2026-06-17T00:40Z | authoritative |

## Reconciliation table - 41 reopened rows

| Row id | Phase | Queue status | Witness path | green? | Key counters (on disk) | VERDICT |
|:---|:---:|:---|:---|:---:|:---|:---|
| VEG-LG2-LIVE-FIRE-001 | A | done | landscape_grammar_lg2_live.json | true | fire_disturbances=1 | DONE-CONFIRMED |
| VEG-LG2-LIVE-BUILD-001 | A | done | landscape_grammar_lg2_live.json | true | construction_disturbances=1 | DONE-CONFIRMED |
| VEG-LG2-RECOVERY-001 | A | done | landscape_grammar_lg2_live.json | true | recovery_ticks=0 | AMBIGUOUS (green but recovery_ticks=0; A11 exit needs >=1) |
| VEG-LG2-FUEL-BRIDGE-001 | A | done | fire_ecology_live.json | true | f1_green=true; fuel_gate_active=true | DONE-CONFIRMED |
| VEG-LG2-HARVEST-001 | A | done | landscape_grammar_lg2_live.json | true | harvest_disturbances=0 | GENUINELY-OPEN (A15 needs >=1; still 0) |
| VEG-WITNESS-LIVE-PROOF-001 | A | done | veg_runtime_proof_live.json | true | L0-L4 ladder all true | DONE-CONFIRMED |
| VEG-LAMBDA-INPUTS-001 | B | done | landscape_grammar_lg3_live.json | true | anchor_source=live_transport_settlement | DONE-CONFIRMED |
| VEG-PRESET-INDUSTRIAL-001 | B | done | mcp_landscape_grammar_sign_live.json | true | industrial_barrier_v0 passed | DONE-CONFIRMED |
| VEG-PRESET-MILITARY-001 | B | done | mcp_landscape_grammar_sign_live.json | true | military_defensive_v0 passed | DONE-CONFIRMED |
| VEG-PRESET-SETTLEMENT-001 | B | done | mcp_landscape_grammar_sign_live.json | true | settlement_park_v0 passed | DONE-CONFIRMED |
| VEG-PRESET-FOREST-001 | B | done | mcp_landscape_grammar_sign_live.json | true | old_growth_core_v0 passed | DONE-CONFIRMED |
| VEG-PRESET-CATALOG-001 | B | done | mcp_landscape_grammar_sign_live.json + disk | true | 10/10 passed; 10 JSON on disk | DONE-CONFIRMED |
| VEG-MAP-ROLLOUT-001 | B | done | landscape_grammar_map_rollout_live.json | true | chunks_with_program=17 | DONE-CONFIRMED |
| VEG-MAP-PARTITION-001 | B | done | landscape_grammar_map_rollout_live.json | true | mean_topology_kind_count=6.0 | DONE-CONFIRMED |
| VEG-PREVIEW-TOPOLOGY-001 | C | done | landscape_grammar_lg4_preview_live.json | true | topology_tint_wired=true; kinds_visible_min=3 | DONE-CONFIRMED |
| VEG-PREVIEW-GLYPH-001 | C | done | landscape_grammar_extract_live.json | true | extract_glyph_deterministic=true | DONE-CONFIRMED |
| VEG-PREVIEW-OVERLAY-001 | C | done | stage5_full_app_live.json | true | ecology_heat_enabled=true; ecology_rows=100 | DONE-CONFIRMED |
| VEG-POPULATION-FIELD-001 | C | done | stage5_full_app_live.json | true | ecology_active_rows=17 | DONE-CONFIRMED |
| VEG-MINIMAP-OVERLAY-001 | C | done | stage5_full_app_live.json | true | ecology_heat_enabled; ui_p3_m2/m3_green | DONE-CONFIRMED |
| VEG-PLAY-WITNESS-001 | C | done | play_scenario_live.json | true | veg_topology_visible_at_operational_zoom=true | DONE-CONFIRMED |
| VEG-FULL-APP-WITNESS-001 | C | done | stage5_full_app_live.json | true | ecology_active_rows=17; live_landscape_program_on_chunk | DONE-CONFIRMED |
| VEG-DISTRICT-SETTLEMENT-001 | D | done | landscape_grammar_lg3_live.json | true | anchor_source=live_transport_settlement (no modulo hack) | DONE-CONFIRMED |
| VEG-DISTRICT-TRANSPORT-001 | D | done | landscape_grammar_lg3_live.json | true | anchor_source=live_transport_settlement | DONE-CONFIRMED |
| VEG-DISTRICT-CONSTRUCTION-001 | D | done | landscape_grammar_lg2_live.json | true | construction_disturbances=1; history_linked=true | DONE-CONFIRMED |
| VEG-DISTRICT-HYDRO-001 | D | done | landscape_grammar_lg3_live.json | true | district coupling green (no hydro key) | DONE-CONFIRMED (low conf) |
| VEG-LG3-WITNESS-001 | D | done | landscape_grammar_lg3_live.json | true | district_kind_count=2; ind+mil anchored | DONE-CONFIRMED |
| VEG-SIM-EFFECT-HOOK-001 | D | done | fire_ecology_live.json | true | sim_effect_spine.effect_rows=3 | DONE-CONFIRMED |
| VEG-POPULATION-SUBCELL-001 | E | done | (no dedicated witness) | n/a | rolls up into program_close phase_e_green | STALE-BLOCKED -> DONE (low conf) |
| VEG-INSTANCE-SPAWN-001 | E | done | landscape_grammar_extract_live.json | true | row_count=1; burn_active_rows=1 | DONE-CONFIRMED (low conf; single-row) |
| VEG-INSTANCE-EXTRACT-001 | E | done | landscape_grammar_extract_live.json + stage5 veg_burn_witness | true | extract_glyph_deterministic=true; veg_burn_witness.green | DONE-CONFIRMED |
| VEG-FIRE-CORRIDOR-001 | E | done | stage5_full_app_live.json fire_corridor_witness | true | green=true; population_fuel_wired=true | DONE-CONFIRMED |
| VEG-SUCCESSION-GRAPH-001 | E | done | landscape_grammar_lg2_live.json | true | succession_age_ticks=true; topology_kind_count=6 | DONE-CONFIRMED |
| VEG-SNAPSHOT-PERSIST-001 | E | done | vegetation_snapshot_roundtrip_live.json | true | chunks_roundtrip=1; program_rows=1 | DONE-CONFIRMED (low conf; single-chunk) |
| VEG-REGISTRY-STAMP-001 | F | done | landscape_grammar_lg5_live.json | true | registry_stamp=true; bevy_chunk_uv_stamp=true | DONE-CONFIRMED |
| VEG-LG5-WITNESS-001 | F | done | landscape_grammar_lg5_live.json | true | green=true; atlas_batch_green=true | DONE-CONFIRMED (schema/bake scope) |
| VEG-PROGRAM-CLOSE-001 | F | done | vegetation_program_close_live.json | true | all_green=true; phase_a..f_green=true | DONE-CONFIRMED |
| VEG-PRESET-EXPAND-001 | F | done | disk + mcp_landscape_grammar_sign_live.json | true | 10 JSON on disk (>=10); 10/10 passed | DONE-CONFIRMED |
| VEG-NESTED-DEPTH-001 | F | done | landscape_grammar_lg2_live.json | true | nested_depth_max=2 | AMBIGUOUS (depth=2; F07 exit wants >=3) |
| VEG-OPERATOR-HISTORY-001 | G | done | (no dedicated witness; UI row) | n/a | rolls up into program_close | STALE-BLOCKED -> DONE (low conf; UI) |
| VEG-STEWARD-REGRESS-001 | G | done | replay_editor_parity_live.json | true | parity_green=true; replay_ring_len=64 | DONE-CONFIRMED |
| VEG-QUEUE-SYNC-001 | G | done | (process row - no sim witness) | n/a | HANDOFF/queue sync | UNVERIFIED (process) |

## Conflicting drain rows (status vs witness)

| Row id | Phase | Queue status | Witness | green? | Note | VERDICT |
|:---|:---:|:---|:---|:---:|:---|:---|
| VEG-B-ROLLOUT-WITNESS-001 (seq 34) | B | done | landscape_grammar_map_rollout_live.json | true | chunks=17 ok, green ok, but presets_used=2 vs exit presets_used>=3 | AMBIGUOUS (2 of 3 clauses met; presets_used short) |
| VEG-C14-OPERATOR-CHECKLIST-001 (seq 54) | C | blocked | operator human sign-off | n/a | operator_human_signoff; not coder-flippable | GENUINELY-OPEN (correct) |
| VEG-F01-DESIGN-ATLAS-001 (seq 71) | F | blocked | tile lg5 pilot (G4/G5 planned) | n/a | designer-mcp art-ship gate; bake done but G4/G5 planned | GENUINELY-OPEN (art-ship scope open) |
| VEG-F02-MCP-ATLAS-001 (seq 72) | F | blocked | mcp_landscape_sign_atlas_live.json (atlas lane) | true | atlas lane green on disk at pilot/bake scope; row still blocked | STALE-BLOCKED (under-claiming at bake scope) |
| VEG-G03-LG6-FLOWERS-001 (seq 81) | G | deferred | - | - | deferred LG-6 charter slice | GENUINELY-OPEN (correctly deferred) |

## G4 / G5 semantics - what each green certifies (operator vocabulary)

Three artifacts say green with three different SCOPES. They are NOT interchangeable:

1. SCHEMA/SIGN gate - mcp_landscape_grammar_sign_live.json green:true. Certifies 10/10 landscape presets pass validate-report landscape_grammar against landscape_grammar_v0.schema.json (topology_preset_count=30, ship_preset_count=10). This is DATA-VALIDATION, not art and not runtime.

2. ATLAS-PACK / BAKE gate (G0-G3) - tile_tile_landscape_lg5_pilot_v1_live.json green:true BUT gates show G0:pass G1:pass G2:pass G3:pass G4:planned G5:planned, ship:false, development_tier:pilot, png_count=3 (real_bake=true). Certifies PNGs really baked + atlas meta + index entry written (the BAKE gate). It does NOT certify G4 (art-ship QA) or G5. Its green is the bake-success flag, not art-ship.

3. RUNTIME / SIM gate - landscape_grammar_lg5_live.json green:true and the rollup mcp_landscape_sign_atlas atlas lane green:true. Certifies registry_stamp + bevy_chunk_uv_stamp wired; atlas_batch_green references the bake above. Means runtime can consume the pilot atlas - still pilot-tier, still ship:false.

OPERATOR TAKEAWAY: the LG-5 atlas is schema-green + bake-green + runtime-stamp-green, but art-ship-green (G4/G5) is planned, NOT achieved. So VEG-F02/F04 are legitimately done at pilot/bake scope while VEG-F01 (designer art-ship charter) and G4/G5 remain open work. A single word green hides this; the queue should record the SCOPE per green.

## Snap / status-doc disagreements with the witnesses (authoritative = on-disk witness)

| # | Source | Claim | On-disk witness | Authoritative |
|:--|:---|:---|:---|:---|
| 1 | queue reopen_audit.witness_truth | lg2 fire=0, build=0, green:false | lg2 fire=1, build=1, green:true (2026-06-17) | witness - audit snapshot stale |
| 2 | queue reopen_audit.witness_truth | map_rollout chunks=1, green:false | map_rollout chunks=17, green:true | witness - audit snapshot stale |
| 3 | queue reopen_audit.witness_truth | stage5 ecology_active_rows=0 | stage5 ecology_active_rows=17 | witness - audit snapshot stale |
| 4 | queue v2_false_done_reopened (41 rows) | all reopened (over-claim) | most now green w/ non-zero counters | witness - list now stale; under-claims |
| 5 | honest-status doc | v3 drain 78 done / 3 blocked / 1 deferred | drain[] shows status done | doc+drain agree but BOTH contradict reopen_audit in SAME queue file (internal inconsistency) |
| 6 | honest-status doc LG-3 | stub - modulo hack, not settlement coupling | lg3 anchor_source=live_transport_settlement | witness - doc stale-pessimistic; coupling now live |
| 7 | honest-status doc LG-2 | harness proves fire + construction > 0 | matches witness (fire=1, build=1) | agree |
| 8 | honest-status doc LG-4 | tint proof, NOT FULL_APP pixel proof | lg4 topology_tint_wired=true but pixel_heterogeneity_wired=false, topology_tint_visible_chunks=0 | AGREE - doc honest; pixel proof still pending (real residual gap) |
| 9 | orchestrator snap | VEG-F02-MCP-ATLAS-001 blocked | mcp_landscape_sign_atlas atlas lane green:true | witness - snap stale; delivered at pilot/bake scope |
| 10 | orchestrator snap | VEG-F03 ready (after F02 or stub) | lg5 registry_stamp=true -> F03 already satisfied | witness - F03 done, snap lags |

## Verdict bucket counts (41 reopened rows)

- DONE-CONFIRMED: 35 (live witness green + required counters non-zero)
- STALE-BLOCKED -> flip: 2 (VEG-POPULATION-SUBCELL-001, VEG-OPERATOR-HISTORY-001; low conf, rollup/UI)
- AMBIGUOUS: 2 (VEG-LG2-RECOVERY-001 recovery_ticks=0; VEG-NESTED-DEPTH-001 depth=2 vs F07 >=3)
- GENUINELY-OPEN: 1 (VEG-LG2-HARVEST-001 harvest_disturbances=0)
- UNVERIFIED: 1 (VEG-QUEUE-SYNC-001; process row, no sim witness)

Conflicting drain rows: VEG-F02-MCP-ATLAS-001 = STALE-BLOCKED (atlas lane green, bake scope); VEG-B-ROLLOUT-WITNESS-001 = AMBIGUOUS (presets_used=2 < 3); VEG-C14 / VEG-F01 / VEG-G03 = GENUINELY-OPEN.

## Confidence notes

- HIGH confidence: rows whose exit_predicate witness exists on disk with the exact counter (lg2, map_rollout, stage5, play, lg3, lg5, snapshot, program_close, sign).
- LOW confidence (no dedicated exit_predicate; rely on program_close rollup or single-chunk proof): VEG-POPULATION-SUBCELL-001, VEG-OPERATOR-HISTORY-001, VEG-SNAPSHOT-PERSIST-001 (single-chunk), VEG-INSTANCE-SPAWN-001 (single-row), VEG-DISTRICT-HYDRO-001 (no hydro-specific key).
- The program_close rollup is a META-witness; only as trustworthy as the per-phase witnesses it aggregates. The per-phase close JSONs (vegetation_phase_a..e_close_live.json) referenced by drain seq 18/40/55/62/70 do NOT exist on disk - those phase-close rows are UNVERIFIED at per-phase granularity (only the program-level rollup exists).
