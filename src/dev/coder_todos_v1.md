# Coder todos `v1` — authoritative pick board



**Date:** 2026-07-04 (session 3) · **Branch:** `master` · **Bevy 0.19** · **Migration:** CLOSED (`mig_a_program_close.json`)



**Queue sync (2026-07-04):** `sync_dispatch_subqueues` 0 stale · hub **pick_now:** `RGR-M1-001` (coder) · `PERF-INSTR-VFX-002` (coder_b, in_progress) · operator×3 · **@coder_a idle** (CHAIN-A lib closed)



**Rule:** Witness JSON `green: true` wins over stale queue rows. **Do not pick MIG-*** (program closed).



**Related boards:** [`remaining_lane_todos_v1.md`](remaining_lane_todos_v1.md) · [`cross_front_pick_queue_v1.md`](cross_front_pick_queue_v1.md) · [`coder_non_migration_todos_v1.md`](coder_non_migration_todos_v1.md) · [`delegated_lane_todos_v1.md`](delegated_lane_todos_v1.md)



```powershell

python tools/orchestrator/scripts/sync_dispatch_subqueues.py

python tools/orchestrator/scripts/scan_queues_hub.py

cargo test -p proc_A_dine01 --lib sch_w1 cln_p0 bq_q3 building_quality facade_propagation bq_h3 -q

cd tools/mcp/python && python -m pytest tests/test_building_quality_qc.py tests/test_aps_mutation_inventory.py tests/test_aps_panel_sync_characterization.py -q

```



---



## Witness truth (disk — green)



| ID | Witness | `green` |

|:---|:---|:---:|

| MIG program | `mig_bevy_019/mig_a_program_close.json` | ✓ |

| BQ-A1/A2 · C4 · H1/H2/H3 | `bq_*` · `building_quality_live.json` | ✓ |

| APSR-S1/S2 · Q1 | `apsr_a1_s1/s2_001_live.json` · `apsr_a4_q1_001_live.json` | ✓ |

| SCH-W1-E1 | `sch_w1_e1_001_live.json` | ✓ |

| **SCH-W1-T1** | `sch_w1_t1_001_live.json` | ✓ |

| **SCH-W1-E3** | `sch_w1_e3_001_live.json` | ✓ |

| **SCH-W1-E4** | `sch_w1_e4_001_live.json` | ✓ |

| **CLN-P0-S8** | `cln_p0_s8_001_live.json` | ✓ |

| **CLN-P0-P10** | `cln_p0_p10_001_live.json` | ✓ |

| **CLN-P0-R4/R8/T4/T7/T6** | `cln_p0_r4/r8/t4/t7/t6_001_live.json` | ✓ |

| **SCH-W1-P1** | `sch_w1_p1_001_live.json` | ✓ |

| **BQ-Q3** | `bq_q3_golden_001_live.json` | ✓ (12 seeds) |
| **BQ-K3** | `bq_k3_grammar_001_live.json` | ✓ |
| **GPU-P3-B/C/A/D** | `gpu_p3b_*` · `gpu_p3c_*` · `gpu_p3a_tracy_001_live.json` · `gpu_p3d_runbook_001_live.json` | ✓ |
| **RTT CHAIN-A lib** | `rtt_lane_witness_live.json` | ✓ (B5+A1+C-004/005 lib) |
| **RGR M1 chain** | `rgr_m1_witness_live.json` | ✓ (api.rs + latches) |
| **RGR Phase 0 (CHAIN-B)** | `chain_b_witness_live.json` | ✓ (CB-MIG/CB-RGR) |

| PERF lib | `triage_perf_vfx_fix_2026-06-11_live.json` | ✓ (`display_acceptance_pending: true`) |



---



## Pick now — by owner



### @coder_a — CHAIN-A RTT lane (PLAN-TACTICAL-MAP-RTT-v1) — **lib closed · idle**



| ☑ | ID | Exit |

|:---:|:---|:---|

| ☑ | **RTT-B5-001..003** | `ParticleViewGlobals` + fire/water raster from `ExtractedCameraMetrics` |

| ☑ | **RTT-B5-004 lib** | PostUpdate sync after `ExtractedCameraMetricsSet::Sync` · `rtt_lane` tests |

| ☑ | **RTT-A1-001..004** | Latch deleted · `TacticalMapFillRect` · fill streak witness |

| ☑ | **RTT-C-004/005 lib** | `diagnosis_hints` + `image_node_bind` in `tactical_map_debug.rs` |

| ☐ | **RTT-C-001..003** | **operator/steward** — release build · `--test vfx` · refresh `tactical_map_debug_live.json` |

| ☐ | **DR-RTT-VR16** | **operator** — refresh `stage5_full_app_live.json` |



### @operator / @sim-steward (P0 — display required)



| ☐ | ID | Exit |

|:---:|:---|:---|

| ☐ | **RTT-C-002/003** | `cargo run --release -- --test vfx` · refresh `tactical_map_debug_live.json` (frame 120+) |

| ☐ | **DR-RTT-VR16** | Same run · refresh `stage5_full_app_live.json` |

| ☐ | **PERF-INSTR-VFX-002 accept** | Display · clear `display_acceptance_pending` in `triage_perf_vfx_fix_2026-06-11_live.json` |



### @coder (P1 — RGR Phase 1 — **hub pick_now**)



| ☐ | ID | Scope |

|:---:|:---|:---|

| ☑ | **RGR-M1-001..004** | `render/api.rs` · mod.rs thin · gui shims · plugin latches |

| ☐ | **RGR-V2-001..** | blocked on operator P0 — ViewProjectionAuthority raster migration |

| ☑ | **GPU-P0C-PRIME** | PRIME-001..004 + P0-D/E partial — [`gpu_todos_v1.md`](gpu_todos_v1.md) |
| ☑ | **BQ-K3 RON merge** | `bq_k3_grammar_001_live.json` · golden seeds refreshed |
| ☑ | **GPU-P3-D** | Runbook perf truth · `run_demo_perf_truth.ps1` |
| ☑ | **GPU-P3-A** | Tracy optional feature + `tracy_integration.md` |



### @coder-mcp (P2 — APS tail)



| ☐ | ID | Scope |

|:---:|:---|:---|

| ☑ | **APSR-P1/P2/P3** | Panel split · preview_state_display · material browser |

| ☑ | **APSR-D1–D4** | Token lint · tooltip coverage · inline feedback · density polish |



### @coder + @designer-mcp (P2 — blocked on charters)



| ☐ | ID | Owner | Note |

|:---:|:---|:---|:---|

| ☑ | **BQ-K1 bake** | @coder-mcp | ✓ `debug_runs/bq_k1_bake_001_live.json` · 11 GLBs · style_pack wire |
| ☑ | **BQ-K1/K2/K3 charters** | designer-mcp | kit fill · coverage · grammar tables — **SIGNED** |

| ☐ | **BQ-Q2** | operator | screenshot QC · preview worker |



### @coder — closed streams (GPU / SCH / CLN — do not re-pick)



| ☑ | ID | Exit |

|:---:|:---|:---|

| ☑ | **SCH-W1-T1/E3/E4** | witness green |
| ☑ | **CLN-P0-S8/P10** | witness green |
| ☑ | **BQ-Q3** | golden-seed regression |



### @sim-steward (parallel / steward-owned)



| ☐ | ID | Note |

|:---:|:---|:---|

| ☑ | **SCH-W1-P1-001** | Dormant plugins deleted · `sch_w1_p1_001_live.json` |

| ☑ | **CLN-P0-R4/R8/T4/T7/T6** | Steward Phase 0 · `cln_p0_*_001_live.json` |

| ☐ | **DR-MIG-TILEMAP** | Monitor `bevy_ecs_tilemap` 0.19 |

| ☐ | **DR-CITY-C6-BSN** | BSN charter after C6 visual |



---



## Closed this session — do not re-pick



**RTT CHAIN-A (@coder_a):** RTT-B5-001..004 lib · RTT-A1-001..004 · RTT-C-004/005 lib · witness `rtt_lane_witness_live.json`



**Construction lib hygiene (2026-07-04):** tile atlas production-v1 tests · `con_p2_002` witness harness split (commit before tick plugin)



**Schedule Wave 1 (coder):** SCH-W1-T1 · SCH-W1-E3 · SCH-W1-E4 (+ prior E1)



**Cleanup Phase 0 (coder):** CLN-P0-S8 · CLN-P0-P10 · SCH-W1-P2 doc comment



**BQ:** BQ-Q3 golden-seed regression (12 seeds in `golden_seed_set.rs`)



**Migration / BQ streams:** MIG program · BQ F/C/A/H · APSR S1/S2/Q1



---



## Session pick order (next)



1. **Operator P0** — RTT-C-002/003 · DR-RTT-VR16 · PERF display accept  
2. **@coder RGR-V2-001** — GPU raster ViewProjectionAuthority (after operator P0)  
3. **@coder_b** — on-call only  
4. **@coder-mcp** — BQ-Q2 tail blocked on operator · city C8 when G1 gate  
5. **SCH-W2** — steward gate (`DR-SCHED-W2`) after operator P0 green



---



## Regression exit



```powershell

cargo test -p proc_A_dine01 --lib sch_w1 cln_p0 bq_q3 building_quality stage5 rtt_lane rgr_m1 -q

cargo orchestrate --skip-cargo

```



