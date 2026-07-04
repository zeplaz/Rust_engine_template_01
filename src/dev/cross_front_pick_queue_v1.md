# Cross-front pick queue `v1`

**Date:** 2026-07-03 · **Branch:** `master` (Bevy 0.19)

## Migration status (read first)

**PLAN-BEVY-019-MIG-v1 = CLOSED on master.** Witness: `debug_runs/mig_bevy_019/mig_a_program_close.json` · `mig_v1_gate.json` → `mig_program_closed: true`.

| Class | Meaning |
|:---|:---|
| **Done** | MIG-V1 + all non-blocked MIG-A slices (A1–A14, A16–A18 except blockers) |
| **Blocked (not migration debt)** | A15 morph · `bevy_ecs_tilemap` 0.19 (DR-MIG-TILEMAP) |
| **Closed / handoff** | A8 Settings · A9 BSN → city grammar · A11 audit (deep prepass = POST-MIG perf) |
| **NOT migration** | A11/A13/A17 “deep” merges — moved to **PERF-GPU-TERRAIN** / fire perf lanes |

**Do not pick MIG-A11/A13/A17 as migration work.**

---

Single board for **parallel lanes** — pick one primary stream per session; cross-drain when blocked (display, designer sign-off, file mutex).

**Authority:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) · [`defer_registry.json`](../tools/orchestrator/queues/defer_registry.json) · [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md)

**Session ritual:**

```powershell
python tools/orchestrator/scripts/sync_dispatch_subqueues.py
python tools/orchestrator/scripts/scan_queues_hub.py
cargo test -p proc_A_dine01 --lib block_recipe building_quality city_g1 stage5 construction
```

---

**Remaining open work:** [`remaining_lane_todos_v1.md`](remaining_lane_todos_v1.md)

## Pick now (this board — see remaining_lane_todos for full list)

**Delegated lanes:** all closed except PERF display — see [`coder_todos_v1.md`](coder_todos_v1.md)

| Pri | Lane | ID | Owner | Exit witness / command |
|:---:|:---|:---|:---|:---|
| **P0** | RTT/VFX | **DR-RTT-VR16** | operator / @coder_b | `cargo run --release -- --test vfx` · refresh `stage5_full_app_live.json` |
| **P1** | APSR | **APSR-A4-Q1-001** | @coder-mcp | ☑ QC strip on Assembly tab · `apsr_a4_q1_001_live.json` |
| **P1** | Schedule | **SCH-W1-T1-001** | @coder | Pause dt_scale · next after E1 ☑ |
| **P1** | Cleanup | **CLN-P0-*** | @coder_a/b | Phase 0 hygiene · [`plan_cleanup_v1.md`](plan_cleanup_v1.md) |
| **P2** | GPU terrain | **P0-C′ instanced** | @coder | [`plan_gpu_terrain_production_exec_001_v1.md`](plan_gpu_terrain_production_exec_001_v1.md) — POST-MIG perf |
| **P2** | BQ kits | **BQ-K1-KITFILL-001** | @coder-mcp | **Charter SIGNED** — bake `kit_fill_bq_k1_001` + wire style packs |

---

## Done recently (do not re-pick)

| ID | Witness |
|:---|:---|
| MIG-V1 + MIG-A program | `mig_v1_gate.json` · `mig_a_program_close.json` · `mig_a_rollup.json` |
| MIG-A9 BSN handoff | `mig_a_a9_bsn_scene_handoff.json` → city grammar § BSN ASSEMBLY CHARTER |
| BQ-F1/F2/F3 | `building_quality_bq_f1_live.json` · `bq_f2_style_001_live.json` · `bq_f3_slot_001_live.json` |
| BQ-C1/C2/C3 | `bq_c1_contract_001_live.json` · `bq_c2_bounds_001_live.json` · `bq_c3_seam_001_live.json` |
| BQ-A1/A2 | `bq_a1_adjacency_001_live.json` · `building_quality_live.json` |
| CITY G0–G3 + P1/P2 | `city_*` lib tests · BSN pilot `city_c6_bsn_001_live.json` |
| APSR-A0 T1/T2 | pytest guardrails baseline |

---

## Blocked — wait for predicate (cite DR-*)

| DR-* | Item | Unblock when |
|:---|:---|:---|
| **DR-MIG-TILEMAP** | `bevy_tilemap_adapter` default | crates.io `bevy_ecs_tilemap` 0.19.x + compat witness |
| **DR-MIG-A15** | MorphWeights adoption | Product skinning plan or `grep MorphWeights src/` |
| **DR-CITY-C6-BSN** | BSN assembly expansion | C6 visual + designer-mcp BSN charter (§ BSN ASSEMBLY CHARTER) |
| **DR-CITY-P2** | Block LOD impostor | CITY-C8 planner sign-off |
| **DR-CLEANUP-P2** | Cleanup Phase 2+ | Phase 0 started or PERF baseline + steward sign-off |
| **DR-SCHED-W2** | Schedule Wave 2 fire authority | Wave 1 gate + no RTT file conflict |
| **DR-GPU-TERRAIN-P0C** | Tilemap default path | DR-MIG-TILEMAP or P0-C′ signed |

---

## Agent routing (one line)

| Agent | If idle, pick |
|:---|:---|
| **@coder** | SCH-W1 · GPU P0-C′ · BQ-Q1 | BQ-C4 · BQ-H → [`delegated_lane_todos_v1.md`](delegated_lane_todos_v1.md) |
| **@coder_b** | DR-RTT-VR16 · CLN-P0-* | PERF-INSTR-VFX-002 → delegated doc |
| **@coder-mcp** | APSR-Q1 · BQ-K bake batches | APSR-S1/S2 → delegated doc |
| **@coder_a** | APSR-A0 tail · CLN-P0-* · BQ-F parallel safe |
| **@designer-mcp** | DR-CITY-C6-BSN charter (BQ-K lane closed) |
| **@sim-steward** | DR-CLEANUP-P2 readiness · POST-MIG perf triage only |
| **operator** | `--test vfx` · G-PLAY-OPERATOR-01 |

---

## Stream 4 backlog (delegated — [`delegated_lane_todos_v1.md`](delegated_lane_todos_v1.md))

| Phase | Slices | Status |
|:---|:---|:---|
| **BQ-H** | H1 facade · H2 openings · H3 v0 grammar retirement | **H1/H2 in flight elsewhere** |
| **BQ-K** | K1 kit fill · K2 slot coverage audit · K3 grammar enrichment | **designer-mcp SIGNED** → @coder-mcp bake · @coder RON merge |
| **BQ-Q** | Q1 witness wire · Q2 screenshot QC · Q3 golden seeds | open on cross-front P2 |
| **APSR-P/D/Q** | Panel split · design lint · QC surfaces | after APSR-S1/S2 |

---

## Conflict matrix (do not parallelize same file)

| Files | Lanes |
|:---|:---|
| `building_grammar.rs` | CITY-G0 only (G0 done — avoid unless G1+) |
| `procedural_build_extract.rs` / `assembly_snapshot.rs` | BQ-A/H + APSR-S2 |
| `src/render/terrain_instanced_draw.rs` | GPU terrain P0-C′ · POST-MIG perf |
| Fire authority files | SCH-W2 only after Wave 1 |

---

## Verify bundle (session exit)

```powershell
cargo test -p proc_A_dine01 --lib building_quality edge_adjacency city_g1 stage5 construction procedural_build_extract -q
cd tools/mcp/python && python -m pytest tests/test_building_quality_bq_c1.py tests/test_aps_mutation_inventory.py tests/test_aps_panel_sync_characterization.py -q
cargo orchestrate --skip-cargo
```

Refresh hub: `python tools/orchestrator/scripts/scan_queues_hub.py`
