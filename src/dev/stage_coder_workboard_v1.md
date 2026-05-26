# Coder workboard `v1` (active)

| Field | Value |
|:---|:---|
| **Version** | `1.4.0` |
| **Date** | 2026-05-26 |
| **Coder todos** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) |
| **Prior closure** | [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md) |
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
| **Dual-queue A+B (28 IDs)** | 2026-05-26 | [`coder_a_dual_queue_closure_v1.rs`](coder_a_dual_queue_closure_v1.rs) + [`coder_b_queue_bundle_proof.rs`](coder_b_queue_bundle_proof.rs) |

---

## Execution queue — wave 3

**Authority:** [`coder_dual_queue_v3.md`](coder_dual_queue_v3.md) · **Checklist:** [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md) · **Machine:** [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) v3.0

**F7 split:** v2 **FIRE7-F7-A-001** = witness bundle ☑ · **FIRE7-F7-A-EXIT-001** = product gate ☐ (Coder A P1 #1)

### @coder A — P1 (one primary)

| ID | Task |
|:---|:---|
| **FIRE7-F7-A-EXIT-001** | F7-A product gate A1–A5 |
| **VFX-VISUAL-SIGNOFF-001** | Visual run P2 sparks/water |
| **TRIAGE-GPU-TILE-WGSL-001** | WGSL instanced tiles |
| **TRIAGE-VISUAL-TEARDOWN-001** | VR-02 GPU exit |
| **TRIAGE-PHASE-F-CULL-001** | Particle cull |
| **UI-WP-VISUAL-001** | World preview visual sign-off |
| **INFRA-GPU-TILE-GIZMO-001** | Drop gizmo fallback |
| **S7B-M4-SIM-001** | M4 play in sim |

### @coder B — P1 (one primary)

| ID | Task |
|:---|:---|
| **IND-E02-DEFAULT-PLAY-001** | `ind_e02_green` in play |
| **CONSTRUCTION-MV-SIM-001** | MV ghosts in sim |
| **S7P-GRID-UX-UI-001** | Grid toast UI |
| **LOG-E01-VISUAL-CONFIRM-001** | Logistics on visual |
| **UI-P3-M3-UNITS-001** / **REPLAY-001** | Minimap M3 tails |
| **REPLAY-PARITY-001** | Replay editor parity |
| **UX-E02-APPLY-POLISH-001** | BQ-128 apply polish |
| **WAVE-S-SHELL-POLISH-001** | Wave S shell edges |

**Blocked until F7-A-EXIT:** FIRE7-F7-B-001 · FIRE7-F7-C-001

### Operator / design (not coder queue)

| Owner | ID |
|:---|:---|
| @operator | OPS-F01 · OPS-F03 · VFX-CAPTURE-INSIM-001 |
| @designer | FIRE7-DESIGN-001 · S7P-DESIGN-002 |

---

## Global regression

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.4.0 | 2026-05-26 | Wave 3 queues — F7-A-EXIT split; 12 A + 15 B active rows |
| v1.3.0 | 2026-05-26 | Dual-queue A+B closed — machine queue empty; next_lane only |
| v1.2.6 | 2026-05-26 | **coder_dual_queue_todos_v1** — 14 @coder A execution rows |
| v1.2.5 | 2026-05-25 | PLAN-LEDGER-REFRESH-003 — `plan_doc` + machine queues |
| v1.2.4 | 2026-05-25 | PLAN-LEDGER-REFRESH-002 sync |
| v1.2.3 | 2026-05-25 | Fleet reconcile: M4 witness refresh; TRAY-OPT + WC-D04 done; P1 = BQ-128-APPLY |
| v1.2.2 | 2026-05-25 | PLAN-LEDGER-REFRESH-001: queue sync; P1 = UI-P3-M4-001 |
| v1.2.1 | 2026-05-25 | D-07 done; logistics/minimap witnesses current |
| v1.2.0 | 2026-05-25 | Audit sync: water/fire/UI-P4/shell done |
| v1.0.1 | 2026-05-24 | Queue mirror + WATER done rows |
| v1.0.0 | 2026-05-24 | Initial workboard |
