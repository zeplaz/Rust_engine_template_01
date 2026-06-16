# Coder dual queue `v3` — post-closure wave

| Field | Value |
|:---|:---|
| **Version** | `3.0.0` |
| **Date** | 2026-05-26 |
| **Repo** | `C:\dev\github\Rust_engine_template_01` · **`master` only** |
| **Prior wave** | [`coder_dual_queue_v2.md`](coder_dual_queue_v2.md) — 28 IDs closed via lib bundles |
| **Checklist** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) · compact [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |

**Rule:** One **P1 primary** per session (≤3 files). Witness JSON wins. Do **not** re-queue v2 § Done unless regression fails.

> **Archive status (2026-05-27):** This document is historical wave-3 execution context.  
> Use `tools/orchestrator/queues/coder_active_queue.json` + `docs/archive/2026-06-src-dev/plans/stage_coder_workboard_v1.md` for live assignments.

**Regression (every slice):**

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle
```

---

## F7 gate split (important)

| ID | Meaning | Status |
|:---|:---|:---:|
| **FIRE7-F7-A-001** | Dual-queue **witness bundle** (`fire7_f7_a_001/green` in infra JSON) | ☑ v2 closed |
| **FIRE7-F7-A-EXIT-001** | **Product gate** A1–A5 in [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) | ☐ **P1 @coder A** |
| **FIRE7-F7-B-001** | Real streaming (sleep/wake, neighbor wake) | ☐ blocked until F7-A-EXIT |
| **FIRE7-F7-C-001** | Real LOD caps in extract / `FireChunkLodState` | ☐ blocked until F7-A-EXIT |

```text
PLAN ☑ → PREFLIGHT ☑ → F7-A-EXIT ☐ → F7-B ☐ → F7-C ☐
         v2 witness ☑ (does NOT close F7-A-EXIT)
```

---

## @coder A — wave 3 (render / GPU / fire / VFX spine)

### P1 — pick one primary

| # | ID | Task | Plan / entry | Exit |
|:---:|:---|:---|:---|:---|
| A1 | **FIRE7-F7-A-EXIT-001** | Close F7-A product gate (A1–A5) | [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) § F7-A exit | lib tests + explicit `fire7_f7_a_001_green` + stage5 green |
| A2 | **VFX-VISUAL-SIGNOFF-001** | `--test visual` sign-off for P2 sparks + water (not lib-only) | [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) | tactical rows in `stage5_full_app_live.json` from visual run |
| A3 | **TRIAGE-GPU-TILE-WGSL-001** | WGSL storage instances + view-aware tile colors | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) TRIAGE-GPU-TILE-WGSL | instanced path without naga panic on visual |
| A4 | **TRIAGE-VISUAL-TEARDOWN-001** | Graceful GPU surface exit (VR-02) | [`visual_run_blockers.md`](visual_run_blockers.md) | visual test exits without surface panic |
| A5 | **TRIAGE-PHASE-F-CULL-001** | View-aware particle culling refinement | triage T4 | lib test + stage5 readiness field |
| A6 | **UI-WP-VISUAL-001** | World preview `--test visual` sign-off (lib → visual) | [`ui_world_preview_coder_queue_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_world_preview_coder_queue_v1.md) | `wave_p_live.json` from visual run + layout greens |
| A7 | **INFRA-GPU-TILE-GIZMO-001** | Remove CPU gizmo fallback when instanced authoritative | triage TRIAGE-GPU-TILE | sim never hits gizmo path when instanced on |
| A8 | **S7B-M4-SIM-001** | M4 playtest hooks exercised in sim (beyond JSON writer) | [`s7b_closure_plan_v1.md`](s7b_closure_plan_v1.md) | `pending_dispatch_count` moves without seed-only path |

### P2 — optional / parallel (disjoint from B P1)

| # | ID | Task | Notes |
|:---:|:---|:---|:---|
| A9 | **VFX-CAPTURE-HOOK-001** | In-sim PNG capture hooks for operator lane | supports **VFX-CAPTURE-INSIM-001** |
| A10 | **TRIAGE-COMPILE-HYGIENE-001** | Reconcile CW board vs live warnings | [`COMPILE_WARNINGS_TODOS.md`](COMPILE_WARNINGS_TODOS.md) |
| A11 | **FIRE7-DESIGN-LOD-WIRE-001** | Wire designer LOD table when [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) lands | blocked on **FIRE7-DESIGN-001** |
| A12 | **STAGE5-VT-DEEP-001** | VT-4/5 camera isolation proof matrix | triage TRIAGE-VT-DEEP · debug-intelligence assist |

### Blocked until **FIRE7-F7-A-EXIT-001**

| ID | Task |
|:---|:---|
| **FIRE7-F7-B-001** | Sleep/wake + neighbor wake → `fire_streaming_live.json` **runtime** writer |
| **FIRE7-F7-C-001** | `FireChunkLodState` band caps in extract path |

### Copy-paste — Coder A primary

```
@coder A — FIRE7-F7-A-EXIT-001
Read: docs/archive/2026-06-src-dev/plans/fire_sim_phase7_architecture_v1.md § F7-A exit (A1–A5)
      src/render/fire_view_extract.rs · fire_visual_extract.rs
First: extend per_view_fire_extract_bounded + fire_visual_producer_count == 1 proof
Do NOT: second global extract · F7-B/C stub JSON · minimap ECS fire query
Verify: cargo test -p proc_A_dine01 --lib fire_view_extract fire_visual_extract stage5
Exit: architecture gate chain marks F7-A CLOSED (not witness-only)
```

---

## @coder B — wave 3 (product / infra / UI / witnesses in sim)

### P1 — pick one primary

| # | ID | Task | Plan / entry | Exit |
|:---:|:---|:---|:---|:---|
| B1 | **IND-E02-DEFAULT-PLAY-001** | Default industrial writer sets `ind_e02_green` in play | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) | `industrial_activation_live.json` without seed-only env |
| B2 | **CONSTRUCTION-MV-SIM-001** | Multiview construction ghost fields from **sim** live writer | [`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md) | `construction_stage_live.json` MV fields in sim run |
| B3 | **S7P-GRID-UX-UI-001** | In-game grid overload toast (witness → UI) | **S7P-DESIGN-002** **SIGNED** — [`s7p_grid_overload_ux_note_v1.md`](s7p_grid_overload_ux_note_v1.md) · `grid_overload_ux.rs` | toast visible in sim; `s7p_grid_ux_toast_ui_wired` |
| B4 | **LOG-E01-VISUAL-CONFIRM-001** | Confirm logistics rows on **visual** run (not lib fixture) | [`logistics_visual_lane_spec_v1.md`](logistics_visual_lane_spec_v1.md) | `logistics_active_rows > 0` from `--test visual` |
| B5 | **UI-P3-M3-UNITS-001** | Unit aggregation markers on minimap | [`ui_p3_m4_minimap_coder_queue_v1.md`](ui_p3_m4_minimap_coder_queue_v1.md) | `unit_marker_rows` in compositor JSON |
| B6 | **UI-P3-M3-REPLAY-001** | Replay scrub ticks on minimap | [`ui_oh_m3_001_plan_v1.md`](ui_oh_m3_001_plan_v1.md) | `replay_scrub_enabled` + scrub witness |
| B7 | **REPLAY-PARITY-001** | Deterministic replay + editor parity | triage TRIAGE-REPLAY | `replay_editor_parity_live.json` green |
| B8 | **TRIAGE-PHASE-D-PARITY-001** | Overlay parity stress / edge cases | triage T4 | infra or stage5 witness extension |
| B9 | **UX-E02-APPLY-POLISH-001** | BQ-128 preset apply ghost polish in editor | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) | roundtrip witness + UX smooth apply |
| B10 | **WAVE-S-SHELL-POLISH-001** | Wave S dock/shell edge cases after hydrate | [`wave_s_open.md`](wave_s_open.md) | `wave_s_hydrate_live.json` + manual sim pass |

### P2 — optional

| # | ID | Task | Notes |
|:---:|:---|:---|:---|
| B11 | **IND-E03-SIM-UX-001** | Grid overload ops strip polish beyond witness | pairs with B3 |
| B12 | **CONSTRUCTION-R4-PREP-001** | Round 4 catalog reconcile (when product board opens) | construction recovery docs |
| B13 | **INFRA-VM-DEEP-001** | Extended VM-08/10/11 sim-time traces | beyond lib refresh |
| B14 | **STAGE6-OPS-WITNESS-001** | Sim-time stage6 refresh helper (supports OPS-F03) | operator can still run sim |
| B15 | **S7B-M3-SIM-001** | M3 overlay exercise in sim session | beyond `s7b_m3_green` writer |

### Blocked until **FIRE7-F7-A-EXIT-001**

| ID | Task |
|:---|:---|
| **FIRE7-F7-B-001** | Streaming systems + `fire_streaming_live.json` runtime writer |
| **FIRE7-F7-C-001** | LOD tier enforcement tied to live state |

### Copy-paste — Coder B primary

```
@coder B — S7P-GRID-UX-UI-001
Read: src/economy/activation/grid_overload_ux.rs · industrial_grid_overload_impl_plan_v1.md
First: wire toast to ops strip / simulation HUD (not witness-only)
Do NOT: new industrial sim writer · duplicate grid overload detect
Verify: cargo test -p proc_A_dine01 --lib industrial activation
Exit: in-sim toast + industrial_activation_live.json s7p fields
```

---

## Suggested parallel pairs (disjoint files)

| Cycle | Coder A | Coder B |
|:---:|:---|:---|
| 1 | **FIRE7-F7-A-EXIT-001** | **S7P-GRID-UX-UI-001** |
| 2 | **TRIAGE-GPU-TILE-WGSL-001** | **IND-E02-DEFAULT-PLAY-001** |
| 3 | **VFX-VISUAL-SIGNOFF-001** | **CONSTRUCTION-MV-SIM-001** |
| 4 | **UI-WP-VISUAL-001** | **UI-P3-M3-UNITS-001** |
| 5 | **TRIAGE-PHASE-F-CULL-001** | **REPLAY-PARITY-001** |

---

## Operator / planner (not @coder queue)

| Owner | ID | Action |
|:---|:---|:---|
| @operator | **OPS-F01** | 60s perf → `debug_runs/perf_attribution_60s.md` |
| @operator | **OPS-F03** | Optional sim refresh → `stage6_virtualization_live.json` |
| @operator | **VFX-CAPTURE-INSIM-001** | PNG captures after **VFX-CAPTURE-HOOK-001** |
| @planner | **FIRE7-DESIGN-001** | `fire_lod_player_read_v1.md` LOD table |
| @designer | **S7P-DESIGN-002** | Grid overload toast copy/layout (supports B3) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v3.0.0 | 2026-05-26 | Post dual-queue closure; F7-A witness vs EXIT split; 20+ rows per coder |
