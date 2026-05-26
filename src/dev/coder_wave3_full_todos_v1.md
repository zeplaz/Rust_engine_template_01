# Coder wave 3 — full todo lists

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Repo** | `master` only |
| **Assign detail** | [`coder_dual_queue_v3.md`](coder_dual_queue_v3.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) |

**Rule:** One **P1** primary per session (≤3 files). If a row is **blocked**, use **Start instead** for that session.

**Regression (every slice):**

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle
```

---

# @coder A — all todos (14 rows)

**Default start (if nothing else picked):** **#1 FIRE7-F7-A-EXIT-001**

| # | ☐ | ID | Task | Exit / verify | Blocked? | If blocked → start |
|:---:|:---:|:---|:---|:---|:---|:---|
| 1 | ☑ | **FIRE7-F7-A-EXIT-001** | Close F7-A product gate A1–A5 (not v2 witness) | `fire7_f7_a_exit_001` in infra JSON + lib bundle | — | — |
| 2 | ☑ | **VFX-VISUAL-SIGNOFF-001** | `--test visual` sign-off: P2 sparks + water | lib tactical VFX witness (`--test visual` = operator) | — | — |
| 3 | ☑ | **TRIAGE-GPU-TILE-WGSL-001** | WGSL instanced tile colors (view-aware) | `triage_gpu_tile_wgsl_001_green` + storage WGSL | — | — |
| 4 | ☑ | **TRIAGE-VISUAL-TEARDOWN-001** | Graceful GPU surface exit (VR-02) | `visual_teardown_vr02_wired` + graceful exit gate | — | — |
| 5 | ☑ | **TRIAGE-PHASE-F-CULL-001** | View-aware particle culling | `view_aware_particle_cull_wired` + spark tests | — | — |
| 6 | ☑ | **UI-WP-VISUAL-001** | World preview visual sign-off (lib → visual) | `ui_wp_visual_001` in `wave_p_live.json` (lib qualified) | — | — |
| 7 | ☑ | **INFRA-GPU-TILE-GIZMO-001** | Drop CPU gizmo when instanced authoritative | sim skips gizmo when instanced on | — | — |
| 8 | ☑ | **S7B-M4-SIM-001** | M4 playtest in sim (beyond JSON writer) | M4 enqueue on sim enter + `pending_dispatch_count` | — | — |
| 9 | ☑ | **VFX-CAPTURE-HOOK-001** | In-sim PNG capture hooks (operator lane) | `VfxCaptureHookPlugin` + enqueue API | — | — |
| 10 | ☑ | **TRIAGE-COMPILE-HYGIENE-001** | Reconcile CW board vs live warnings | `compile_hygiene_live.json` | — | — |
| 11 | ☑ | **FIRE7-DESIGN-LOD-WIRE-001** | Wire designer LOD table into extract | designer caps 32/128/512 in extract | design doc signed | — |
| 12 | ☑ | **STAGE5-VT-DEEP-001** | VT-4/5 camera isolation proof matrix | `stage5_vt_deep_001_green` | — | — |
| 13 | ☑ | **FIRE7-F7-B-001** | Sleep/wake + neighbor wake streaming | `fire_streaming_live.json` runtime writer | was **F7-A-EXIT** | — |
| 14 | ☑ | **FIRE7-F7-C-001** | `FireChunkLodState` band caps in extract | `fire7_f7_c_001_green` + lib test | was **F7-A-EXIT** | — |

### Coder A — read first (by #)

| # | Read |
|:---:|:---|
| 1 | `fire_sim_phase7_architecture_v1.md` § F7-A exit · `fire_view_extract.rs` · `fire_visual_extract.rs` |
| 2 | `vfx_coder_phase2_queue_v1.md` · `gpu_fire_particle_raster.rs` · `gpu_weather_fire_field` |
| 3 | `stage5_triage_backlog.md` TRIAGE-GPU-TILE-WGSL · `tile_debug_instanced.wgsl` |
| 4 | `visual_run_blockers.md` VR-02 · `gpu_surface_teardown.rs` |
| 5 | `stage5_triage_backlog.md` TRIAGE-PHASE-F-CULL |
| 6 | `ui_world_preview_coder_queue_v1.md` · `wave_p_live_proof.rs` |
| 7 | `stage5_triage_backlog.md` TRIAGE-GPU-TILE |
| 8 | `s7b_closure_plan_v1.md` · `stage7_behavioral_live_proof.rs` |
| 9 | VFX capture paths under `assets/vfx/reference/` |
| 10 | `COMPILE_WARNINGS_TODOS.md` |
| 11 | `fire_lod_player_read_v1.md` (when exists) · `fire_view_extract.rs` |
| 12 | `stage5_triage_backlog.md` TRIAGE-VT-DEEP |
| 13–14 | `fire_sim_phase7_architecture_v1.md` § F7-B / F7-C — **only after #1 done** |

### Coder A — suggested session order

1. **#1** FIRE7-F7-A-EXIT-001  
2. **#2** VFX-VISUAL-SIGNOFF-001 *(parallel-safe with Coder B #3)*  
3. **#3** TRIAGE-GPU-TILE-WGSL-001  
4. **#6** UI-WP-VISUAL-001  
5. **#7** INFRA-GPU-TILE-GIZMO-001  
6. **#5** TRIAGE-PHASE-F-CULL-001  
7. **#8** S7B-M4-SIM-001  
8. **#4** TRIAGE-VISUAL-TEARDOWN-001  
9. **#9–12** P2 polish  
10. **#13–14** only after **#1** ☑  

---

# @coder B — all todos (17 rows)

**Default start (if nothing else picked):** **#3 S7P-GRID-UX-UI-001**

| # | ☐ | ID | Task | Exit / verify | Blocked? | If blocked → start |
|:---:|:---:|:---|:---|:---|:---|:---|
| 1 | ☐ | **IND-E02-DEFAULT-PLAY-001** | Default writer sets `ind_e02_green` in play | `industrial_activation_live.json` without seed env | — | — |
| 2 | ☐ | **CONSTRUCTION-MV-SIM-001** | MV construction ghosts from **sim** live writer | `construction_stage_live.json` MV fields in sim | — | — |
| 3 | ☐ | **S7P-GRID-UX-UI-001** | Grid overload **in-game toast** (witness → UI) | toast in sim · `s7p_grid_ux_toast_ui_wired` | soft: **S7P-DESIGN-002** copy | **#1 IND-E02-DEFAULT-PLAY-001** (use placeholder copy) |
| 4 | ☐ | **LOG-E01-VISUAL-CONFIRM-001** | Logistics rows on `--test visual` (not lib fixture) | `logistics_active_rows > 0` from visual | needs visual run green | **#1** or **#5** while visual blocked |
| 5 | ☐ | **UI-P3-M3-UNITS-001** | Minimap unit aggregation markers | `unit_marker_rows` in compositor JSON | — | — |
| 6 | ☐ | **UI-P3-M3-REPLAY-001** | Minimap replay scrub ticks | `replay_scrub_enabled` + scrub witness | — | — |
| 7 | ☐ | **REPLAY-PARITY-001** | Deterministic replay + editor parity | `replay_editor_parity_live.json` green | — | — |
| 8 | ☐ | **TRIAGE-PHASE-D-PARITY-001** | Overlay parity stress / edge cases | infra or stage5 witness extension | — | — |
| 9 | ☐ | **UX-E02-APPLY-POLISH-001** | BQ-128 preset apply ghost polish | `wave_s_blueprint_roundtrip.json` + UX | — | — |
| 10 | ☐ | **WAVE-S-SHELL-POLISH-001** | Wave S dock/shell edge cases after hydrate | `wave_s_hydrate_live.json` + sim pass | — | — |
| 11 | ☐ | **IND-E03-SIM-UX-001** | Grid overload ops strip polish | ops strip UX beyond witness | **#3** toast wired first | **#3 S7P-GRID-UX-UI-001** |
| 12 | ☐ | **CONSTRUCTION-R4-PREP-001** | Round 4 catalog reconcile prep | construction docs + index aligned | product board not open | **#2 CONSTRUCTION-MV-SIM-001** |
| 13 | ☐ | **INFRA-VM-DEEP-001** | Extended VM-08/10/11 sim-time traces | infra JSON sim-written fields | — | — |
| 14 | ☐ | **STAGE6-OPS-WITNESS-001** | Sim-time stage6 refresh helper | supports OPS-F03 operator | — | — |
| 15 | ☐ | **S7B-M3-SIM-001** | M3 overlays exercised in sim session | overlays visible in play | — | — |
| 16 | ☐ | **FIRE7-F7-B-001** | Streaming systems + runtime `fire_streaming_live.json` | sleep/wake mutates residency | **Coder A #1 F7-A-EXIT** | **#3 S7P-GRID-UX-UI-001** or **#1 IND-E02-DEFAULT-PLAY-001** |
| 17 | ☐ | **FIRE7-F7-C-001** | LOD tier enforcement in extract | caps in extract, not JSON stub | **Coder A #1 F7-A-EXIT** | **#2 CONSTRUCTION-MV-SIM-001** or **#5 UI-P3-M3-UNITS-001** |

### Coder B — read first (by #)

| # | Read |
|:---:|:---|
| 1 | `ind_board_reconcile_plan_v1.md` · `economy/activation/live_proof.rs` |
| 2 | `ui_phase6_shell_perf_multiview_plan_v1.md` · `construction/` live writer |
| 3 | `grid_overload_ux.rs` · `industrial_grid_overload_impl_plan_v1.md` |
| 4 | `logistics_visual_lane_spec_v1.md` |
| 5–6 | `ui_p3_m4_minimap_coder_queue_v1.md` · `ui_oh_m3_001_plan_v1.md` · `minimap_compositor/` |
| 7 | `replay_editor_parity_live.json` · triage TRIAGE-REPLAY |
| 8 | `stage5_triage_backlog.md` TRIAGE-PHASE-D-PARITY |
| 9 | `bq128_editor_path_plan_v1.md` |
| 10 | `wave_s_open.md` · `dock_shell.rs` |
| 11 | pairs with #3 |
| 12 | `construction_recovery_todos.md` |
| 13 | `infrastructure_view_isolation_live.json` writer · `view_runtime/` |
| 14 | `stage6_virtualization_live.json` · `wave_c_live_proof.rs` |
| 15 | `s7b_closure_plan_v1.md` M3 § |
| 16–17 | `fire_sim_phase7_architecture_v1.md` § F7-B/C — **only after Coder A #1 ☑** |

### Coder B — suggested session order

1. **#3** S7P-GRID-UX-UI-001  
2. **#1** IND-E02-DEFAULT-PLAY-001  
3. **#2** CONSTRUCTION-MV-SIM-001  
4. **#5** UI-P3-M3-UNITS-001  
5. **#6** UI-P3-M3-REPLAY-001  
6. **#7** REPLAY-PARITY-001  
7. **#9** UX-E02-APPLY-POLISH-001  
8. **#10** WAVE-S-SHELL-POLISH-001  
9. **#4** LOG-E01-VISUAL-CONFIRM-001 *(after Coder A visual fixes if needed)*  
10. **#8, #11–15** P2 / polish  
11. **#16–17** only after **Coder A #1** ☑  

---

# Parallel pairs (disjoint files)

| Cycle | Coder A # | Coder B # |
|:---:|:---|:---|
| 1 | **1** F7-A-EXIT | **3** S7P grid toast |
| 2 | **3** GPU tile WGSL | **1** IND-E02 default |
| 3 | **2** VFX visual | **2** CONSTRUCTION-MV sim |
| 4 | **6** UI-WP visual | **5** M3 units |
| 5 | **5** particle cull | **7** replay parity |

---

# Blocked summary

| Blocked ID | Waits on | Both coders → start |
|:---|:---|:---|
| **FIRE7-F7-B-001** | **FIRE7-F7-A-EXIT-001** (Coder A #1) | A: **#1** · B: **#3** or **#1** |
| **FIRE7-F7-C-001** | **FIRE7-F7-A-EXIT-001** (Coder A #1) | A: **#1** · B: **#2** or **#5** |
| **FIRE7-DESIGN-LOD-WIRE-001** | **FIRE7-DESIGN-001** + F7-A-EXIT | A: **#2** or **#3** |
| **IND-E03-SIM-UX-001** | **S7P-GRID-UX-UI-001** (#3) | B: **#3** first |
| **CONSTRUCTION-R4-PREP-001** | product Round 4 board | B: **#2** |
| **LOG-E01-VISUAL-CONFIRM-001** | visual run healthy | B: **#1** / **#5** until A fixes visual |

---

# Operator / design (not coder todos)

| ☐ | Owner | ID | Action |
|:---:|:---|:---|:---|
| ☐ | @operator | **OPS-F01** | 60s perf → `perf_attribution_60s.md` |
| ☐ | @operator | **OPS-F03** | Sim refresh → `stage6_virtualization_live.json` |
| ☐ | @operator | **VFX-CAPTURE-INSIM-001** | PNG captures after Coder A **#9** |
| ☐ | @designer | **FIRE7-DESIGN-001** | `fire_lod_player_read_v1.md` — unblocks A **#11** |
| ☐ | @designer | **S7P-DESIGN-002** | Toast copy/layout — polishes B **#3** |

---

# v2 closed — do not re-queue (28 IDs)

See [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md). **FIRE7-F7-A-001** there = witness bundle only, not A **#1**.
