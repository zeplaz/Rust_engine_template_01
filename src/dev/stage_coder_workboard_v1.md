# Coder workboard `v1` (active)

| Field | Value |
|:---|:---|
| **Version** | `1.2.5` |
| **Date** | 2026-05-25 |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) v1.2.5 |
| **Open todos** | [`stage_open_todos_v1.md`](stage_open_todos_v1.md) v1.5.0 |
| **Audit** | [`planner_status_audit_v5.md`](planner_status_audit_v5.md) |
| **Coder queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Copy-paste detail** | Per-track plans under [`stages/`](stages/) |

**Rule:** One primary slice per session (≤3 files). Check **DONE** — do not re-implement.

---

## Done — do not redo (2026-05-25)

| ID | Track |
|:---|:---|
| S7P-IND-001, S7P-STEWARD-001 (witness writer) | S7-PLAY |
| FX-WATER-SHADER/PARTICLE, WATER-W1-OCEAN, W1-RIVER, W2-FOAM, STRATEGIC | FX-WATER |
| FX-FIRE-SPARK-001…011, P2-VFX-VISUAL/WITNESS, P2-FIRE-SPARK-010/011 | VFX-P2 · [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) **CLOSED** |
| UI-WP-LAYOUT-001, **UI-WP-LAYOUT-002**, **UI-WP-LAYOUT-D07** | UI-P4 |
| UI-P3-M1, M2, **UI-P3-M2-CODER-A**, **UI-P3-M2-TRAY-OPT**, **UI-P3-M3-001**, **UI-P3-M4-001** (FoW+EW witness), UI-P3-001 | UI-P3 |
| **INFRA-PROJ2-001** / **INFRA-PROJ2-CODER-B** | INFRA-55 · [`infra_proj2_sole_writer_plan_v1.md`](infra_proj2_sole_writer_plan_v1.md) **DONE** |
| **WC-D04-CODER-B** | WAVE-C |
| **WC-DEPTH-001** | WAVE-C |
| UI-SHELL-REFRESH-001, UI-P2A-CODER-B, **UI-P2B-CODER-B** (done — `ui_p2b_coder_b_green`) | UI-P2 |
| **TRIAGE-VM-09-CODER-B**, **STEWARD-VM-09-001** (slice 2) | INFRA-55 |
| IND-E03-CODER-A | INDUSTRIAL · [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) **DONE** |
| **BQ-128-APPLY-001** | WAVE-S / UX-E02 | Preset **Apply ghost** ([`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md)) |
| **UI-P5-PAUSE-001** | UI-P5 | Bevy pause — **CLOSED** — [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) |

---

## Execution queue (pick one)

**Mirrors:** [`active_coder_queue_v1.md`](active_coder_queue_v1.md) · [`coder_triage_list_v1.md`](coder_triage_list_v1.md) · [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json)

| Queue ID | When | Done when | Status |
|:---|:---|:---|:---:|
| ~~**S7B-M1-001**~~ | — | `s7b_m1_green: true` | ☑ |
| **S7B-M2-001** | — | `dispatch_delay_ticks: 8`, `s7b_m2_green` | ☑ |
| **S7B-M3-001** | — | overlays → `s7b_m3_green` | ☑ |
| **OPS-F01** / **OPS-F03** | operator | 60s perf + optional sim stage6 refresh | ☐ |
| **UI-P3-M3-UNITS-001** / **REPLAY-001** | Optional | M3 units / replay — witness `unit_marker_rows`, `replay_scrub_enabled` | ☑ |
| **UI-WP-LAYOUT-D02-OPT** | Optional | D-02 map ≥65% — `d02_sidebar_max_width_px`, witness helper | ☑ |
| **UI-P2A-F03 / P4-AUTH** | Witness tails | `ui_p2a_tail.*` green via replay + lib test | ☑ |

---

## Global regression

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.5 | 2026-05-25 | PLAN-LEDGER-REFRESH-003 — `plan_doc` + machine queues |
| v1.2.4 | 2026-05-25 | PLAN-LEDGER-REFRESH-002 sync |
| v1.2.3 | 2026-05-25 | Fleet reconcile: M4 witness refresh; TRAY-OPT + WC-D04 done; P1 = BQ-128-APPLY |
| v1.2.2 | 2026-05-25 | PLAN-LEDGER-REFRESH-001: queue sync; P1 = UI-P3-M4-001 |
| v1.2.1 | 2026-05-25 | D-07 done; logistics/minimap witnesses current |
| v1.2.0 | 2026-05-25 | Audit sync: water/fire/UI-P4/shell done |
| v1.0.1 | 2026-05-24 | Queue mirror + WATER done rows |
| v1.0.0 | 2026-05-24 | Initial workboard |
