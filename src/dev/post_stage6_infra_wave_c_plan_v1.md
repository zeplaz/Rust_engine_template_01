# Post–Stage 6 — Infra (VM-09) + Wave C depth plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-INFRA-C-WC** |
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` · **execute:** `@sim-steward` → `@coder` |
| **Slice 2 hub** | [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md) v2 (**PLAN-INFRA-SLICE2-001**) |
| **VM-09 closure** | [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) — **CLOSED** |
| **Slice 3 launch** | [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](infra_slice3_wc_d04_ops_f01_plan_v1.md) — **OPS-F01** + **WC-D04** |
| **Status** | **ACTIVE** — VM-09 slice 2 **CLOSED** · **WC-D04** + **OPS-F01** open |
| **Board** | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) Phase C + D |
| **Infra track** | [`stages/infra_55_execution_plan_v1.md`](stages/infra_55_execution_plan_v1.md) |
| **Wave C track** | [`stages/wave_c_depth_plan_v1.md`](stages/wave_c_depth_plan_v1.md) |

---

## Executive summary

**Phase C audits complete.** **VM-09 slice 2 + PROJ2** landed — see [`post_stage6_infra_slice2_plan_v1.md`](post_stage6_infra_slice2_plan_v1.md). **Next:** **OPS-F01** → **WC-D04** (WC-DEPTH-003) → **OPS-F03** stage6 JSON refresh.

---

## Infra 5.5+ — signed vs open

| ID | Task | Board status | Execution slice |
|:---|:---|:---|:---|
| IN-C01…C07 | VM/PROJ audits | ☑ done | — |
| **INFRA-PREFLIGHT-001** | Steward gate | ☐ queued | Shift A/B YAML |
| **INFRA-VM09-001** | Slice 1 `gpu_particles` | ☑ **done** | [`vm09_gate_v1.md`](vm09_gate_v1.md) |
| **TRIAGE-VM-09-CODER-B** | `view_representation` zoom | ☑ **done** | `triage_vm09_coder_b_green: true` |
| **STEWARD-VM-09-001** | Slice 2 steward gate | ☑ **CLOSED** | [`steward_vm09_gate_v1.md`](steward_vm09_gate_v1.md) |
| **INFRA-PROJ2-001** | Hit-test sweep | ☑ **done** | `infra_proj2_001_green: true` |
| **TRIAGE-VM-09-v2** | Invert bridge (layer C) | ☐ open | planner-sized — not Stage 5 blocker |
| **OPS-F01** | 60s perf | ☐ operator | `perf_attribution_60s.md` |

**Does not reopen** Stage 5/6 operational sign-offs.

### Copy-paste — INFRA-VM09-001

```
Track: INFRA-55 — INFRA-VM09-001
Read: src/dev/post_stage6_infra_wave_c_plan_v1.md
      src/dev/stages/infra_55_execution_plan_v1.md
Prereq: INFRA-PREFLIGHT-001 GO
First: rg MapCameraDesired writers; fix one callsite to ViewManager bridge
Verify: cargo test -p proc_A_dine01 --lib stage5 view_authority
Witness: infrastructure_view_isolation_live.json clean
```

---

## Wave C — signed vs open

| ID | Task | Board status | Execution slice |
|:---|:---|:---|:---|
| WC-D01…D03 | Witness + tests | ☑ done | `wave_c_live.json` |
| **WC-D04** | Residency churn tune | ☑ coder (**WC-D04-CODER-B**) | **WC-DEPTH-003** · **OPS-F03** sim refresh |
| **WC-DEPTH-001** | Close backlog row | ☑ **DONE** | **BQ-101** — `wc_depth_001_green` |
| **OPS-F03** | Refresh stage6 JSON | ☐ operator | sim session |

### Copy-paste — WC-DEPTH-001

```
Track: WAVE-C — WC-DEPTH-001
Read: src/dev/stages/wave_c_depth_plan_v1.md
      prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md §6
First: pick one WAVE_C_OPEN_BACKLOG_ITEMS row; implement + test
Verify: cargo test -p proc_A_dine01 --lib stage6 wave_c
```

---

## Recommended cycle (infra + wave)

| Week | Primary | Secondary |
|:---|:---|:---|
| 1 | INFRA-PREFLIGHT + VM09-001 | OPS-F03 stage6 refresh |
| 2 | WC-DEPTH-001 | PROJ2 one callsite |
| 3 | WC-D04 + OPS-F01 | — |

---

## post_stage6_active_todos.md linkage

Add execution pointers (this doc) under Phase C § execution and Phase D § WC-D04 — audits remain ☑; slices above are **launch queue** for coders.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Link PLAN-INFRA-SLICE2-001; VM-09 slice 2 closed |
| v1.0.0 | 2026-05-24 | PLAN-INFRA-C-WC |
