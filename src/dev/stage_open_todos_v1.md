# Stage tracks — open todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.6.0` |
| **Date** | 2026-05-25 |
| **Fleet snapshot** | [`stage_tracks_fleet_snapshot_signoff_v1.md`](stage_tracks_fleet_snapshot_signoff_v1.md) (**PLAN-LEDGER-REFRESH-004**) |
| **Fleet audit** | [`stage_tracks_audit_signoff_20260525.md`](stage_tracks_audit_signoff_20260525.md) v5 |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |
| **Queue** | [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Planner batch** | [`planner_queue_todos_v1.md`](planner_queue_todos_v1.md) |
| **Coder board** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) |
| **Designer board** | [`stage_designer_todos_v1.md`](stage_designer_todos_v1.md) |

---

## Done — no rework (2026-05-25)

### Designer (all gates **SIGNED**)

UI4 · S7P · VFX capture/post · WATER · UI-P2 · MINIMAP-M2/M3 spec · **DESIGN-D-WP-REVIEW** · **UX-E02-BQ128-001** · **S7B-DESIGN-001** — see [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md)

### Coder / infra (witness-backed)

| Track | IDs closed | Proof |
|:---|:---|:---|
| **Stage 5/6 spine** | FULL_APP path | `stage5_full_app_live.json` |
| **UI-P3 minimap** | M1, M2, **UI-P3-M3-001** (M2 ecology/construction), **UI-P3-M4-001** (FoW+EW), **UI-P3-M2-TRAY-OPT**, UI-P3-001 | `minimap_compositor_live.json` — `ui_p3_m4_green` refreshed |
| **UI-P4 World Preview** | UI-WP-LAYOUT-001/002, D-07 | `wave_p_live.json` |
| **VFX-P2 / FX-WATER** | Fire spark 001–011, water shader/particle | lib + `debug_runs/` |
| **INFRA-55** | VM-09 slice 2, **INFRA-PROJ2-001**, **WC-D04-CODER-B** | `stage6_virtualization_live.json` (`wc_d04.green`) |
| **S7-PLAY** | S7P-IND, logistics witness | `stage7_play_live.json` |
| **Wave S / UX-E02** | **UX-E02-BQ128-001** + **BQ-128-APPLY-001** apply ghost | `wave_s_blueprint_roundtrip.json` |
| **Construction** | Operational phase | `construction_stage_live.json` |

---

## Open todos (fleet truth)

### P1 — primary coder

| ID | Owner | Done when |
|:---|:---|:---|
| **UI-P5-PAUSE-001** | @coder (P2) | **CLOSED** — [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) |
| **UI-P5-DESIGN-001** | @designer (P2) | **DONE** — [`ui_p5_design_signoff_v1.md`](ui_p5_design_signoff_v1.md) |
| ~~**S7B-PREFLIGHT-001**~~ | — | **DONE** — [`steward_s7b_preflight_gate_v1.md`](steward_s7b_preflight_gate_v1.md) **GO** |
| ~~**S7B-M1-001**~~ | — | **DONE** — `behavioral_contract_ok` + `s7b_m1_green` in [`stage7_behavioral_live.json`](../../debug_runs/stage7_behavioral_live.json) |
| ~~**S7B-M2-001**~~ | — | **DONE** — `dispatch_delay_ticks: 8`, `s7b_m2_green` |
| ~~**S7B-M3-001**~~ | — | **DONE** — recon + logistics overlay → `s7b_m3_green` |

### P2 — optional M3 / UI polish

| ID | Owner | Done when |
|:---|:---|:---|
| **UI-P3-M3-UNITS-001** | @coder | Unit aggregation markers on minimap |
| **UI-P3-M3-REPLAY-001** | @coder | Replay scrub ticks (replay parity) |
| **UI-WP-LAYOUT-D02-OPT** | @coder | Map dominance ratio polish |
| **UI-WP-LAYOUT-003** / **UI-WP-MOTION-001** | @coder | WP-L1 paper / D-12 motion (deferred) |

### P3 — operator / infra

| ID | Owner | Done when |
|:---|:---|:---|
| **OPS-F01** | operator | Dated section in [`perf_attribution_60s.md`](../debug_runs/perf_attribution_60s.md) |
| **OPS-F03** | operator | Optional sim refresh — lib witness **green** (`wc_d04.green`, `gpu_upload_bytes_frame > 0`) |
| ~~**TRIAGE-VM-09-v2**~~ | — | **DONE** — invert bridge; `vm_09.triage_vm09_v2_green` in [`infrastructure_view_isolation_live.json`](../../debug_runs/infrastructure_view_isolation_live.json) |

### P4 — optional tails

| ID | Owner | Done when |
|:---|:---|:---|
| **UI-P2A-F03** / **UI-P2A-P4-AUTH** | @coder (deferred) | Witness tail only — `continuation_queue.json` status **deferred** |
| **VFX-CAPTURE-INSIM-001** | operator | In-sim PNG captures |

---

## Repeat each cycle

| ID | Action |
|:---|:---|
| **PLAN-LEDGER-REFRESH** | [`stage_tracks_ledger_refresh_runbook_v1.md`](stage_tracks_ledger_refresh_runbook_v1.md) |

```powershell
cargo test -p proc_A_dine01 --lib stage5 minimap_compositor wave_s wc_d04
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.6.0 | 2026-05-25 | **S7B-PLAN-001** SIGNED — preflight + M1 queued |
| v1.5.0 | 2026-05-25 | **PLAN-LEDGER-REFRESH-003** — planner batch closed; audit v5 |
| v1.4.0 | 2026-05-25 | **PLAN-LEDGER-REFRESH-002** fleet truth; queue restored; stage6 CURRENT |
| v1.3.0 | 2026-05-25 | Reconcile: UI-P3-M4 + TRAY-OPT + WC-D04 done; designer batch closed; P1 = BQ-128-APPLY + S7B-PLAN |
| v1.2.0 | 2026-05-25 | PLAN-LEDGER-REFRESH-001 |
| v1.2.2 | 2026-05-23 | WC-D04-CODER-B lib witness |
| v1.2.1 | 2026-05-23 | UI-P2B-CODER-B |
