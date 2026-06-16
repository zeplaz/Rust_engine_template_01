# Coder dual queue `v2` — long assign list

| Field | Value |
|:---|:---|
| **Version** | `2.0.0` |
| **Date** | 2026-05-26 |
| **Repo** | `C:\dev\github\Rust_engine_template_01` · **`master` only** |
| **Boards** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) · [`stage_open_todos_v1.md`](stage_open_todos_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Open todos** | [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md) |

**Status (2026-05-26):** Wave **v3** active — [`coder_dual_queue_v3.md`](coder_dual_queue_v3.md) · checklist [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md). Prior v2 closure (28 IDs): [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md).

**Verify both bundles:**

```powershell
cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle
```

**Rule:** Do **not** re-queue § Done unless regression fails. Pick work from machine queue § `next_lane` or planner FIRE7 / operator OPS.

**Regression (every slice):**

```powershell
cargo test -p proc_A_dine01 --lib stage5
```

---

## Done — do not assign again

Prior fleet: UI shell 2A/2B/2C · Wave 3 · P4/P5/P6 · minimap M1–M4 · S7B M1/M2/M3 · VM-09 v2 · PROJ2 · WC-D04 · STEWARD-W3-GATE-001

**Dual-queue closure 2026-05-26:**

| Bundle | IDs | Proof |
|:---|:---|:---|
| **Coder A** (14) | FIRE7-F7-A-001 … S7B-TUNE-DELAY-001 | [`coder_a_dual_queue_closure_v1.rs`](coder_a_dual_queue_closure_v1.rs) |
| **Coder B** (14) | LOG-E01 … S7P-GRID-UX-001 | [`coder_b_queue_bundle_proof.rs`](coder_b_queue_bundle_proof.rs) |

**Qualified (witness-only, not full product UI):** S7P-GRID-UX toast · CONSTRUCTION-MV multiview ghosts in sim · UI-WP P2 via lib constants (not full `--test visual` sign-off) · INFRA-PERF via wc_d04 (OPS-F01 60s still operator).

---

## @coder A — queued (render / GPU / sim shell / fire)

### P1 — pick one primary

| # | ID | Task | Plan / entry | Exit witness |
|:---:|:---|:---|:---|:---|
| A1 | **FIRE7-F7-A-001** | Harden per-view `FireVisualFramesByView` | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) · prereq **FIRE7-PLAN-001** on disk | lib `fire_view_extract` + stage5 |
| A2 | **P2-FIRE-SPARK-010** | Sparks **above** smoke / weather field pass order | [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) | tactical visual: sparks visible |
| A3 | **P2-FIRE-SPARK-011** | Spark compute tune @ tactical zoom | same · `fire_spark_compute.wgsl` | compare `fire_spark_target_v1.png` |
| A4 | **P2-WATER-POLISH-001** | River ribbon + ocean tile coverage | same · D-W01/D-W04 | player-visible rivers; `water_ocean_tiles` |
| A5 | **UX-E03-CODER-A** | Transmission media provider wiring (no new queue writer) | [`ux_e03_transmission_shell_note_v1.md`](ux_e03_transmission_shell_note_v1.md) | sim: narrative lane read-only |
| A6 | **S7B-M4-PLAY-001** | Mission enqueue UI hook (Move/Secure corridor) — **tune/play** | [`s7b_closure_plan_v1.md`](s7b_closure_plan_v1.md) playtest § | `pending_dispatch_count` moves in sim |
| A7 | **INFRA-GPU-TILE-001** | Instanced tile authoritative; drop CPU gizmo fallback | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) TRIAGE-GPU-TILE | sim uses instanced path |

### P2 — optional polish

| # | ID | Task | Notes |
|:---:|:---|:---|:---|
| A8 | **UI-WP-PIPELINE** | World preview raster/GPU/viewport bugs only | **Not** `window.rs` layout |
| A9 | **UI-WP-L4-001** | Raster look from signed ref captures | After **UI4-DESIGN-003** |
| A10 | **UI-WP-MOTION-001** | WP motion table §6 | After LAYOUT-003 |
| A11 | **UI-WP-LAYOUT-003** | Paper frames + D-09 offsets | `window.rs` + assets |
| A12 | **P4-VEH-01** | Vehicle icon row consumers (logistics/convoy UI) | [`stages/ui_phase4_execution_plan_v1.md`](stages/ui_phase4_execution_plan_v1.md) |
| A13 | **INFRA-PERF-001** | Shell/frame budget hotspots (if OPS-F01 finds issue) | `frame_budget_diagnostics.rs` |
| A14 | **S7B-TUNE-DELAY-001** | Tune `dispatch_delay_ticks` (default 8) | amend plan only; lib test |

### Copy-paste — Coder A starters

```
@coder A — FIRE7-F7-A-001
Read: docs/archive/2026-06-src-dev/trees/stages/fire_sim_phase7_plan_v1.md · fire_sim_phase7_architecture_v1.md (when landed)
First: fire_view_extract.rs isolation test
Do NOT: minimap fire extract
Verify: cargo test -p proc_A_dine01 --lib fire_view_extract stage5
```

```
@coder A — P2-FIRE-SPARK-010
Read: docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md § P2-FIRE-SPARK-010
First: gpu_fire_particle_raster.rs transparent order vs gpu_weather_fire_field
Verify: cargo run -p proc_A_dine01 --release -- --test visual
```

```
@coder A — P2-WATER-POLISH-001
Read: vfx_coder_phase2_queue_v1.md § P2-WATER-POLISH-001
First: river half_width / ocean tile_kind in water_surface_visual.rs
Verify: tactical zoom river read; stage5 water witness fields
```

```
@coder A — UX-E03-CODER-A
Read: docs/archive/2026-06-src-dev/plans/ux_e03_transmission_shell_note_v1.md
First: transmission.rs ingest only — StrategicCommandQueue stays strategic/
Do NOT: enqueue orders from transmission shell
Verify: cargo test -p proc_A_dine01 --lib stage7_behavioral comms_contract
```

---

## @coder B — queued (witness / infra / product JSON / WP chrome)

### P1 — pick one primary

| # | ID | Task | Plan / entry | Exit witness |
|:---:|:---|:---|:---|:---|
| B1 | **LOG-E01-WITNESS** | Refresh FULL_APP logistics rollup | [`logistics_projection_impl_plan_v1.md`](logistics_projection_impl_plan_v1.md) | `stage5_full_app_live.json` `logistics_active_rows > 0` |
| B2 | **IND-E02-DEFAULT** | Default industrial writer sets `ind_e02_green` in play | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) | `industrial_activation_live.json` without seed-only path |
| B3 | **P2-VFX-WITNESS-001** | Tactical zoom lib gates fire+water rows | [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) | harness tests @ zoom 0.8 |
| B4 | **P2-WATER-WITNESS-002** | Water particle proof fields in visual JSON | same | `water_particle_*` at tactical |
| B5 | **INFRA-VM10-001** | Minimap lockstep diagnostics hardening | [`stages/infra_55_execution_plan_v1.md`](stages/infra_55_execution_plan_v1.md) | `infrastructure_view_isolation_live.json` |
| B6 | **INFRA-VM11-001** | Preview vs main semantic audit + 1–2 fixes | same | vm11 + wave_p alignment |
| B7 | **INFRA-VM09-STRAY-001** | Grep/fix remaining stray `MapCameraDesired` writers | [`recovery_viewport.md`](recovery_viewport.md) | view_authority tests green |
| B8 | **S7P-GRID-UX-001** | Grid overload toast/tray (coder impl of design note) | **S7P-DESIGN-002** when designer delivers | industrial UX in sim |
| B9 | **WITNESS-SHELL-P4** | Emit `phase4.icon_atlas_loaded: true` in shell JSON | lib `icon_atlas` already green | `ui_shell_migration_live.json` |

### P2 — optional / parallel

| # | ID | Task | Notes |
|:---:|:---|:---|:---|
| B10 | **UI-WP-LAYOUT-D02-OPT** | Map ≥65% dominance | witness helper in lib |
| B11 | **UI-WP-LAYOUT-001** | D-01 unified workspace shell | **May be DONE** — verify `wave_p_live.json` before assign |
| B12 | **UI-P2A-WITNESS-TAIL** | Hover ops zone + build rail auth refresh | deferred — 15 min sim pass |
| B13 | **UI-P3-M2-TRAY-OPT** | Overlay tray → `MinimapOverlayMask` | check workboard — may be done |
| B14 | **CONSTRUCTION-MV-001** | Multiview construction ghosts (DQ-POST-04) | [`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md) |
| B15 | **WAVE-P-WITNESS** | Refresh `wave_p_live.json` in sim (~120 frames) | witness-only |
| B16 | **WAVE-C-WITNESS** | Refresh `wave_c_live.json` | witness-only |
| B17 | **STAGE6-WITNESS** | OPS-F03 refresh | operator-led optional |

### Copy-paste — Coder B starters

```
@coder B — LOG-E01-WITNESS
Read: docs/archive/2026-06-src-dev/plans/logistics_visual_lane_spec_v1.md
Run: cargo run -p proc_A_dine01 --release -- --test visual
Exit: stage5_full_app_live.json projection_graph.logistics_active_rows > 0
Do NOT: new logistics extract
```

```
@coder B — P2-VFX-WITNESS-001
Read: docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md § P2-VFX-WITNESS-001
First: gpu_particles.rs + gpu_water_particles.rs tests @ zoom_alpha 0.8
Verify: cargo test -p proc_A_dine01 --lib gpu_particles gpu_water_particles stage5
```

```
@coder B — INFRA-VM10-001
Read: docs/archive/2026-06-src-dev/trees/stages/infra_55_execution_plan_v1.md
First: infrastructure_view_isolation writer + minimap lockstep fields
Verify: cargo test -p proc_A_dine01 --lib stage5 view_runtime
```

```
@coder B — IND-E02-DEFAULT
Read: docs/archive/2026-06-src-dev/plans/ind_board_reconcile_plan_v1.md
First: economy/activation/live_proof.rs default writer path
Exit: industrial_activation_live.json ind_e02_green without RUST_ENGINE_IND_E02_SEED only
```

---

## Suggested parallel pairs (disjoint files)

| Cycle | Coder A | Coder B |
|:---:|:---|:---|
| 1 | **P2-FIRE-SPARK-010** | **LOG-E01-WITNESS** |
| 2 | **P2-WATER-POLISH-001** | **P2-VFX-WITNESS-001** |
| 3 | **FIRE7-F7-A-001** (after planner doc) | **INFRA-VM10-001** |
| 4 | **UX-E03-CODER-A** | **S7P-GRID-UX-001** |
| 5 | **UI-WP-PIPELINE** | **UI-WP-LAYOUT-D02-OPT** |

---

## Operator (not coder — do not queue as @coder)

| ID | Action |
|:---|:---|
| **OPS-F01** | 60s perf → `debug_runs/perf_attribution_60s.md` |
| **OPS-F03** | Optional `stage6_virtualization_live.json` refresh |
| **VFX-CAPTURE-INSIM-001** | PNG captures under `assets/vfx/reference/review_captures/` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v2.0.0 | 2026-05-26 | Long dual-coder queue after planner/designer idle + UI/S7B closure |
