# Active coder queue `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.3.0` |
| **Date** | 2026-05-25 |
| **Machine queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Triage** | [`coder_triage_list_v1.md`](coder_triage_list_v1.md) |
| **S7B plan** | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) **SIGNED** |

---

## Status snapshot

| Queue ID | plan_doc | Status |
|:---|:---|:---:|
| ~~**S7B-PREFLIGHT-001**~~ | [`steward_s7b_preflight_gate_v1.md`](steward_s7b_preflight_gate_v1.md) | ☑ **GO** |
| **S7B-M1-001** | impl plan + witness spec | ☑ **DONE** |
| **UI-P5-PAUSE-001** | [`ui_oh_p5_001_plan_v1.md`](ui_oh_p5_001_plan_v1.md) | ☑ **CLOSED** |
| **UI-P5-DESIGN-001** | `ui_phase5_pause_menu_plan_v1.md` | ☑ designer PASS — [`ui_p5_design_signoff_v1.md`](ui_p5_design_signoff_v1.md) |
| **LOG-E01** | `logistics_projection_impl_plan_v1.md` | STALE_TIMESTAMP (content green) |
| **TRIAGE-VM-09-v2** | [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md) | ☑ **CLOSED** — `triage_vm09_v2_001_lib_bundle` |

---

## S7B-M1-001 (next coder slice)

```powershell
cargo test -p proc_A_dine01 --lib comms_contract stage7_play
```

**Read:** `src/dev/stage7_behavioral_implementation_plan_v1.md` · `src/strategic/comms_contract.rs`

**Exit:** `debug_runs/stage7_behavioral_live.json` → `behavioral_contract_ok: true`, `s7b_m1_green: true`

---

## Operator

| ID | Action |
|:---|:---|
| **OPS-F01** | Dated 60s in `perf_attribution_60s.md` |
| **OPS-F03** | Optional stage6 sim refresh |
