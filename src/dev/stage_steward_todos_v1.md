# Steward todo board `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@sim-steward` |
| **Workboard** | [`stage_steward_workboard_v1.md`](stage_steward_workboard_v1.md) |
| **Queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |

**Rule:** One steward package per session. **Queued** rows run only after listed **When** / coder prereqs.

---

## Queued — run after Wave 1 coder work

| ID | When | Agent | Status | PASS requires |
|:---|:---|:---|:---:|:---|
| **STEWARD-W3-GATE-001** | After Wave 1 witness fields are present | `@sim-steward` | **DONE (PASS)** | See gate [`steward_w3_gate_v1.md`](steward_w3_gate_v1.md) |
| **UI-SHELL-REFRESH-001** | Same session as W3 gate (Shift C sub-check) | `@sim-steward` | **DONE (re-verified)** | `phase2_zones_live`, `phase2b_closed`, `egui_pass_count_in_sim: 0` |

### STEWARD-W3-GATE-001 — PASS matrix

| Check | Witness / spec |
|:---|:---|
| Stage 5 spine | `stage5_full_app_live.json` → `stage5_closure.passes`, `readiness.passes` |
| Shell vs spec | [`ui_shell_migration_live.json`](../../debug_runs/ui_shell_migration_live.json) matches [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) (2A/2B rollup) |
| Minimap M2 | [`minimap_compositor_live.json`](../../debug_runs/minimap_compositor_live.json) when M2 exists → `ui_w3_m2_001.green` / `ui_oh_m2_001.green` |

**Coder prereq (Wave 1):** `UI-W3-2A-001`, `UI-W3-2B-001`, `UI-W3-M2-001` (or lane bundles `coder_a_ui_five_lane_001`, `coder_b_ui_five_lane_001`) — see [`ui_overhaul_plan.md`](ui_overhaul_plan.md).

**Lib proof (steward):** `cargo test -p proc_A_dine01 --lib steward_w3_gate_001_lib_bundle`

**Executed:** `steward_w3_gate_001_lib_bundle` passed with `CARGO_TARGET_DIR=target/test-alt-steward` (workaround for locked default `target/debug` test exe).

**Same-session closure:** `simulation_shell_phase2 -- --test-threads=1` and `stage5` both passed in the same alternate target dir.

### UI-SHELL-REFRESH-001 — same-session sub-check

Historical steward pass: **DONE** (proof-only). **W3 gate session** re-verifies without reopening UI-P2B architecture:

| Field | Required |
|:---|:---:|
| `phase2_zones_live` | `true` |
| `phase2b_closed` | `true` |
| `egui_pass_count_in_sim` | `0` |

Included in **`steward_w3_gate_001_lib_bundle`** — do **not** schedule as a separate primary row unless bundle fails.

---

## Shift checklist (copy-paste)

### STEWARD-W3-GATE-001-A

```
Lane: STEWARD-W3-GATE-001
Agent: @sim-steward
When: After Wave 1 coder (2A/2B/M2) — NOT before
Read: steward_w3_gate_v1.md, ui_shell_witness_spec_v1.md, ui_overhaul_plan.md
Witness: stage5_full_app_live.json, ui_shell_migration_live.json, minimap_compositor_live.json
Do NOT: re-open UI-P2B-001; run before coder five-lane green
```

### STEWARD-W3-GATE-001-C (includes UI-SHELL-REFRESH re-verify)

```
Act: cargo test -p proc_A_dine01 --lib steward_w3_gate_001_lib_bundle
Optional operator: cargo run -p proc_A_dine01 --release -- --test visual
Deliver: PASS/BLOCK in steward_w3_gate_v1.md
```

---

## Done — do not re-run

See [`stage_steward_workboard_v1.md`](stage_steward_workboard_v1.md) § Done (2026-05-24–25).

| ID | Verdict | Gate doc |
|:---|:---|:---|
| **S7B-PREFLIGHT-001** | **GO (qualified)** | [`steward_s7b_preflight_gate_v1.md`](steward_s7b_preflight_gate_v1.md) |
| **FIRE7-PREFLIGHT-001** | **GO (qualified)** | [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) |

**Verify (regression):**

```powershell
$env:CARGO_TARGET_DIR = "target\test-alt-steward"
cargo test -p proc_A_dine01 --lib steward_s7b_preflight_001
cargo test -p proc_A_dine01 --lib fire_view_extract fire_visual_extract stage5
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.3 | 2026-05-26 | **S7B-PREFLIGHT-001** + **FIRE7-PREFLIGHT-001** recorded DONE |
| v1.0.2 | 2026-05-26 | STEWARD-W3-GATE-001 **PASS**; UI-SHELL-REFRESH re-verified in same session |
| v1.0.1 | 2026-05-25 | STEWARD-W3-GATE-001 → **HOLD**; Wave P witness-only note |
| v1.0.0 | 2026-05-25 | STEWARD-W3-GATE-001 + UI-SHELL-REFRESH same-session todos |
