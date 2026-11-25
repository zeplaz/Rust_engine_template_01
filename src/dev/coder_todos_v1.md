# Open work board

**Updated:** 2026-11-24 · **Branch:** `master` · **Bevy 0.20** · **CI:** green on `d11695fe` (all Rust and Python steps).

This is the one list of open work. Each item links the plan that owns it; the plan has the detail.
How to check a run: [`AGENTS.md`](../../AGENTS.md) "Review path" (`sim_run_dump.json` → `tools/sim_review/score_dump.py`).
The old board (Bevy 0.19, witness JSONs, `cargo orchestrate`) is archived at
[`archive/closed_programs/coder_todos_v1_2026-07.md`](archive/closed_programs/coder_todos_v1_2026-07.md); its open rows were
either closed below or retired with the witness system.

---

## Open

### 1. Roads and rail — [`plan_roads_v1.md`](plan_roads_v1.md) (OPEN)

In code: road tiers, congestion (BPR), bridges (type pick, flood scour/collapse, fire, blast), works bills, grading, line tool, ghosts, merged line panel, panel drag.

| Item | Who | Note |
|:---|:---|:---|
| Hands-on check of the line tool | operator | RMB click vs pan, curves, G assist, T bridge type, ghosts along the line, cursor alignment, merged type list, panel drag, 2D bridge marks, materials row |
| Capture congestion and grading | coder | `road_flow.peak_vc` and grading volume in a dam-scene dump; a narrow road with many trucks |
| Trucks haul works materials | coder | today the bill is drawn from stock; send order-sim runs to the works site instead |
| RD-2 → RD-5 → RD-7 | coder | off-road speed field, worn tracks, rail freight |
| B-3 bridge repair (with D-7), overpass deck model | coder | after the phase order above |

### 2. Ref-books residual — [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md)

Ledger [`ref_books_index/adoption.tsv`](ref_books_index/adoption.tsv): **125 ADOPTED · 3 PARTIAL · 9 DEFER** (137 rows, 2026-11-24).

| Row | State | Gap |
|:---|:---|:---|
| RND-02 | PARTIAL | veg GPU instance buffer unwired; PVS / portals deferred |
| RND-12 | PARTIAL | fire particle expand still counts on CPU (not `dispatch_indirect`) |
| EXEC-05 | PARTIAL (locked) | attention scaffold is enough; more consumers are their own slices |
| DEFER ×9 | waiting on triggers | LEARN-02, TAC-10, RND-06, RND-07, TAC-14, ANIM-03, PHYS-11, DEC-09, DES-03 |

### 3. Voxel occupation — [`plan_voxel_occupation_v1.md`](plan_voxel_occupation_v1.md)

Review fixes done (save/load, deterministic site pick, witness code removed: CQ-07/20/30). Flags stay **off** by default
until OCC-P4/P5 are re-scored for turning it on in play.

### 4. Dam-flood benchmark — [`plan_dam_flood_benchmark_v1.md`](plan_dam_flood_benchmark_v1.md)

DF-1…9 in code. **DF-10** (lock baseline v1) blocked on a baseline capture; DF-3 capture Accept open.

### 5. Smaller open items

| Item | Plan | Note |
|:---|:---|:---|
| Snapshots record `placement_rule_id` per module (APS-GRAM-P3-002) | [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) | grammar set already at tier G4 |
| Split `tile_world_fallback.rs` (RPC-3-001); rename `RttDiagCameraMode::Production` → `Standard` (RPC-3-003) | [`plan_render_production_cleanup_v1.md`](plan_render_production_cleanup_v1.md) | rest of that plan done or overtaken |
| Warehouse v2 burn frames 5–8; PT-6 playtest | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) | engine side done |

### 6. Parked (needs a decision or an outside trigger)

| Item | Waiting on |
|:---|:---|
| Voxel occupation on in play | operator decision (OCC-P4/P5) |
| Who sets faction alliances in play (`FactionStanceBook`) — live pairs stay Neutral | operator decision; town growth plan leaves it open |
| Shopfront / bunker production tile art (only archived greybox exists) | operator: build or drop from scope |
| Fire `smoke_volume` | ES-4 (GPU stubs plan deferred it honestly) |
| RGR-V2-001 raster migration, Material2d VFX port | operator call on view-projection raster migration |
| BQ-Q2 screenshot QC | operator |
| DR-CITY-C6-BSN | BSN charter after C6 visual |

---

## Closed recently (do not re-pick)

| What | Where |
|:---|:---|
| CI green: `-D warnings` clean, lib tests 18 → 0 failed, Python steps fixed (jsonschema, `repo_root()` fallback) | [`plan_ci_test_triage_v1.md`](plan_ci_test_triage_v1.md) · `476ac36d`, `5d0263f2`, `cad07f75`, `d11695fe` |
| Cursor-wave cleanup 70/70 (CQ-78/79/84 on 11-24) | [`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md) |
| Bevy 0.20 migration incl. the M0 lib-test defer log | [`plan_bevy_020_migration_v1.md`](plan_bevy_020_migration_v1.md) · [`mig_bevy_020_m0_test_defer.md`](mig_bevy_020_m0_test_defer.md) |
| GPU stubs kill A–D, mips, array, cull | [`plan_gpu_stubs_kill_v1.md`](plan_gpu_stubs_kill_v1.md) |
| VSS-001, Artist Studio expansion, town growth — checked against code, closed | the three plans' status lines |
| Nov 16 review re-grades: TAC-03 dense heat GPU, MOVE-05 formation members, MOVE-10 wildlife, TAC-05 fort, PCG-07 settlement roads, NAV-13 route cost | cleanup CQ-50/51/52/53/54/57g/58 · `adoption.tsv` |

## Retired with the witness system

RTT-C-001..003, DR-RTT-VR16 and PERF-INSTR-VFX-002 were "refresh a live JSON" tasks. There are no witness gates now; a
display check is a normal playcheck (`python tools/sim_review/playcheck.py`).
