# Planner status audit v8 (PLAN-LEDGER-REFRESH-006)

| Field | Value |
|:---|:---|
| **Audit ID** | **PLAN-LEDGER-REFRESH-006** |
| **Date** | 2026-05-26 |
| **Scope** | Fleet reconcile after wave 3 + wave 4/5 planner + qualified visual close |
| **Checklist** | [`plan_ledger_refresh_006_checklist_v1.md`](plan_ledger_refresh_006_checklist_v1.md) |
| **Authority** | [`orchestrator_signoff_snapshot_20260526_v1.md`](orchestrator_signoff_snapshot_20260526_v1.md) |
| **Prior** | [`planner_status_audit_v7.md`](planner_status_audit_v7.md) |
| **Delivery matrix** | [`planner_delivery_signoff_matrix_v1.md`](planner_delivery_signoff_matrix_v1.md) |
| **Status** | **SIGNED** |

**Witness JSON wins** over markdown checkboxes. This audit supersedes stale rows in human boards listed in § Stale markdown.

---

## Executive verdict

| Lane | Verdict |
|:---|:---|
| **Planner wave 4** | **12/12 CLOSED** — [`planner_wave4_todos_v1.md`](planner_wave4_todos_v1.md) |
| **Planner wave 5 specs** | **8/8 planner docs** — [`planner_wave5_todos_v1.md`](planner_wave5_todos_v1.md) |
| **R4-PLAN-001/002** | **SIGNED** — impl **blocked** until `product_board_open` |
| **Coder fleet** | **CLOSED** — `coder_active_queue.json` v3.0 `active: []` |
| **Operator** | **Tails only** — optional visual upgrade, OPS, captures |
| **Wave 6** | **Do not open** unless product names Round 4 open or P2 depth slice |

---

## Fleet table — closed vs qualified vs operator

| ID | Owner | Verdict | Witness / proof | Notes |
|:---|:---|:---|:---|:---|
| **Stage 5 spine** | maintain | **CLOSED** | `stage5_full_app_live.json` → `readiness.passes: true` | Regression only |
| **FIRE7-F7-A-EXIT-001** | @coder A | **CLOSED** | `fire7_f7_a_exit_001.green: true` | infra JSON |
| **FIRE7-F7-B/C-001** | @coder A | **CLOSED** | `fire_streaming_live.json` · F7-C lib | wave 3 |
| **VFX-VISUAL-SIGNOFF-001** | @coder A | **QUALIFIED CLOSED** | `vfx_visual_signoff_001.green: true` | `visual_run_pending: true` = optional upgrade |
| **UI-WP-VISUAL-001** | @coder A | **QUALIFIED CLOSED** | designer ACCEPT + `wave_p_live.json` | pixel audit optional |
| **LOG-E01-VISUAL-CONFIRM-001** | @coder B | **QUALIFIED CLOSED** | `logistics_active_rows: 1` · harness qualified | not STALE |
| **TRIAGE-PHASE-D-PARITY-001** | @coder B | **CLOSED** | `triage_phase_d_parity_001` + S1–S3 stress | lib |
| **INFRA-VM-DEEP-001** | @coder A | **CLOSED** | `infra_vm_deep_001` in infra JSON | sim trace |
| **STAGE6-OPS-WITNESS-001** | @coder A | **CLOSED** | `stage6_virtualization_live.json` | OPS-F03 optional refresh |
| **CONSTRUCTION-MV-SIM-001** | @coder B | **CLOSED** | `construction_mv_001.green: true` | |
| **IND-E02-DEFAULT-PLAY-001** | @coder B | **CLOSED** | `concrete_chain_e2e.ind_e02_green: true` | |
| **UI-P3-M3-UNITS/REPLAY** | @coder B | **CLOSED** | minimap witness greens | witness-only depth OK |
| **REPLAY-PARITY-001** | @coder B | **CLOSED** | `parity_green: true` | live ring = P2 |
| **S7B-M1–M3** | @coder | **CLOSED** | `s7b_steward_green: true` | behavioral JSON |
| **S7B-M4-SIM-001** | @coder A | **QUALIFIED CLOSED** | lib `refresh_s7b_m4` greens disk | live sim = P2 tail |
| **CONSTRUCTION-R4-PREP-001** | @coder B | **CLOSED** | `construction_r4_prep_001.green` | board **closed** |
| **R4-CORRIDOR-001** | @coder | **BLOCKED** | — | needs `product_board_open` |
| **R4-MV-GHOST-001** | @coder | **BLOCKED** | — | needs `product_board_open` |
| **OPS-F01** | operator | **OPEN** | `perf_attribution_60s.md` | tail |
| **OPS-F03** | operator | **OPEN** | stage6 sim refresh | tail |
| **VFX-CAPTURE-INSIM-001** | operator | **OPEN** | review_captures/ | tail |
| **OPERATOR-VISUAL-BUNDLE** | operator | **OPTIONAL** | `--test visual` clears `visual_run_pending` | not required for qualified close |
| **M3/F7/REPLAY depth** | @coder P2 | **DEFERRED** | specs on disk | product priority only |

---

## Witness spot-check (`debug_runs/*_live.json`)

| File | Key fields | Verdict |
|:---|:---|:---:|
| `stage5_full_app_live.json` | `readiness.passes: true`, `logistics_active_rows: 1`, `vfx_visual_signoff_001.green: true` | **CURRENT** |
| `infrastructure_view_isolation_live.json` | `fire7_f7_a_exit_001.green`, `triage_phase_d_parity_001`, `infra_vm_deep_001` | **CURRENT** |
| `fire_streaming_live.json` | `green: true`, `runtime_writer: true` | **CURRENT** |
| `construction_stage_live.json` | `operational_green`, `construction_mv_001`, `construction_r4_prep_001` | **CURRENT** |
| `industrial_activation_live.json` | `concrete_chain_e2e.ind_e02_green` | **CURRENT** |
| `minimap_compositor_live.json` | `ui_p3_m3/m4`, units/replay tails | **CURRENT** |
| `stage7_behavioral_live.json` | `s7b_steward_green: true`, `s7b_m4_play_green` | **CURRENT** |
| `stage6_virtualization_live.json` | ops witness block | **CURRENT** |
| `replay_editor_parity_live.json` | `parity_green: true` | **CURRENT** |
| `wave_p_live.json` | layout / pipeline greens | **CURRENT** |
| `ui_shell_migration_live.json` | phase2/4/5 — `icon_atlas` may lag lib | **STALE optional** |

---

## Machine queues (2026-05-26)

| File | State |
|:---|:---|
| [`planner_active_queue.json`](../../tools/orchestrator/queues/planner_active_queue.json) | Wave 4 done; wave 5 specs done; **006 signed** |
| [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) | v3.0 — **`coder_a.active: []`**, **`coder_b.active: []`** |
| [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) | Infra triage slices — not wave 3 P1 |

---

## Human boards reconciled

| Board | Action |
|:---|:---|
| [`stage_open_todos_v1.md`](stage_open_todos_v1.md) | → **fleet closed / tails only** v2.0.0 |
| [`active_coder_queue_v1.md`](active_coder_queue_v1.md) | → **fleet closed** v2.0.0 |
| [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) | → **fleet closed / tails only** v2.0.0 |

---

## Stale markdown (do not use as active queue)

| File | Why stale | Use instead |
|:---|:---|:---|
| `stage_open_todos_v1.md` § "Open todos wave 3" (pre-006) | Listed P1 F7/MV as open | This audit + `coder_active_queue.json` |
| `active_coder_queue_v1.md` v1.6 | "Wave 3 active" | `coder_active_queue.json` v3.0 |
| `stage_coder_workboard_v1.md` v1.4 § execution queue | P1 pick-one rows | **006** boards |
| `stage_tracks_signoff_ledger_v1.md` | Pre-wave-3 rows | `orchestrator_signoff_snapshot_20260526_v1.md` |
| `coder_dual_queue_todos_v2.md` § operator ☐ | Ops still valid tails | § Operator tails below |

**Not stale:** [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) v1.1 — rows marked ☑.

---

## Product board gate (P1 — when product opens)

Policy: [`construction_round4_product_gate_plan_v1.md`](construction_round4_product_gate_plan_v1.md) § **Product board open policy**.

| Step | Action |
|:---|:---|
| 1 | Product declares Round 4 open |
| 2 | Set `construction_r4_prep_001.product_board_open: true` in witness (coder) |
| 3 | Queue **R4-CORRIDOR-001** + **R4-MV-GHOST-001** |
| 4 | **Do not** re-plan F7 / M3 / MV wave 4 specs |

**Wave 6 planner:** only if product requests Round 4 open or names a P2 depth slice (`m3_minimap_product_depth`, `fire7_streaming_depth`, `replay_live_ring`).

---

## Operator tails (only active work)

```powershell
cargo test -p proc_A_dine01 --lib coder_a_wave3 coder_b_wave3
# Optional upgrade (not required for qualified close):
cargo run -p proc_A_dine01 --release -- --test visual
```

| ID | Owner |
|:---|:---|
| **OPS-F01** | operator |
| **OPS-F03** | operator |
| **VFX-CAPTURE-INSIM-001** | operator |
| **OPERATOR-VISUAL-BUNDLE** | operator (optional) |
| **S7B-M4-LIVE-001** | operator / @coder P2 |

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_wave3 coder_b_wave3
python tools/orchestrator/scripts/refresh_006_sync.py
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v8.0.0 | 2026-05-26 | **PLAN-LEDGER-REFRESH-006** **SIGNED** — fleet closed, tails only |
