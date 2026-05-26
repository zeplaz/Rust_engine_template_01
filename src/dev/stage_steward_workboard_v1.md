# Steward workboard `v1` (active)

| Field | Value |
|:---|:---|
| **Version** | `1.2.0` |
| **Steward todos** | [`stage_steward_todos_v1.md`](stage_steward_todos_v1.md) |
| **Date** | 2026-05-25 |
| **Owner** | `@sim-steward` |
| **Agent** | [`.cursor/agents/sim-steward.md`](../../.cursor/agents/sim-steward.md) |
| **Queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |

**Rule:** One steward package per session — run **Shift A → B → C** in order for the active parent row. Do **not** re-run gates listed in § Ledger DONE.

---

## Done (2026-05-24–25)

| Package | Status | Witness |
|:---|:---|:---|
| **UI-SHELL-REFRESH-001** | **DONE** PASS proof-only | `phase2b_closed: true` |
| **S7P-STEWARD-001** | **DONE** | `stage7_play_live.json` → `production_green: true` |
| **STEWARD-VM-09-001** | **DONE** | [`steward_vm09_gate_v1.md`](steward_vm09_gate_v1.md) **GO**; witness refreshed |
| **STEWARD-WITNESS-SYNC-001** | **DONE** | [`steward_witness_sync_gate_v1.md`](steward_witness_sync_gate_v1.md) **PASS (qualified)** |
| **STEWARD-SPARK-VFX-001** | **DONE** | [`steward_spark_vfx_gate_v1.md`](steward_spark_vfx_gate_v1.md) **GO (qualified)** |
| **S7B-PREFLIGHT-001** | **DONE** | [`steward_s7b_preflight_gate_v1.md`](steward_s7b_preflight_gate_v1.md) **GO (qualified)** |
| **FIRE7-PREFLIGHT-001** | **DONE** | [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) **GO (qualified)** |
| **UI-OH-GATE-001** | **DONE** | [`steward_ui_oh_gate_v1.md`](steward_ui_oh_gate_v1.md) **PASS (qualified)** |

---

## Primary — **CLOSED** (2026-05-26)

| Package | When | Status | Doc |
|:---|:---|:---:|:---|
| **STEWARD-W3-GATE-001** | After 2A/2B/M2 witness fields | **DONE (PASS)** | [`steward_w3_gate_v1.md`](steward_w3_gate_v1.md) |
| **UI-SHELL-REFRESH-001** | Same session (re-verify) | **DONE** | Sub-check in W3 gate — see [`stage_steward_todos_v1.md`](stage_steward_todos_v1.md) |

**Todo board:** [`stage_steward_todos_v1.md`](stage_steward_todos_v1.md)

```
Verify: cargo test -p proc_A_dine01 --lib steward_w3_gate_001_lib_bundle
Prereq: coder_b_ui_w3_witness_001_lib_bundle OR five-lane bundles green
Do NOT: run before Wave 1 coder; do not reopen UI-P2B-001
```

---

## Secondary — product / infra (orchestrator routes elsewhere)

*(S7B M2/M3 → `@coder` — see [`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md))*

---

## Archive — prior primary *(closed)*

### STEWARD-SPARK-VFX-001 (closed)

```
Lane: STEWARD-SPARK-VFX-001
Agent: @sim-steward
Read: vfx_triage_v1.md, stage5_full_app_live.json fire_* + tactical_vfx_witness
Verify: cargo test -p proc_A_dine01 --lib fire_spark vx_p0_01 tactical_vfx_witness steward_spark_vfx_001_lib_bundle
Do NOT: disable strategic spark cull for witness green; conflate VfxSandbox with Simulation
```

### STEWARD-WITNESS-SYNC-001 (closed)

```
Lane: STEWARD-WITNESS-SYNC-001
Agent: @sim-steward
Read: debug_runs/agent_debug_index.json + bundle rows in steward_witness_sync_gate_v1.md
Act: lib witness refresh tests + steward_witness_sync_001_lib_bundle
Operator tail: cargo run -p proc_A_dine01 --release -- --test visual (stage5 timestamp only)
Do NOT: re-run STEWARD-WATER / S7P / VM-09 / UI-SHELL unless bundle test fails
```

---

## Archive — `UI-SHELL-REFRESH-001` (closed)

**Problem (historical):** `ui_shell_migration_live.json` had `phase2b_closed: false` while `egui_pass_count_in_sim: 0` — **stale proof frame**, not Phase 2B regression.

**Parent witness:** `debug_runs/ui_shell_migration_live.json`

### Todo checklist

| ID | Shift | Action | Status | Owner |
|:---|:---|:---|:---:|:---|
| **UI-SHELL-REFRESH-001-A** | A | Read `ui_shell_migration_live.json`, `simulation_shell_phase2.rs`, `hud_root_tick.rs`, PLAY-01 defaults (`simulation_session.rs`) | ☑ | sim-steward |
| **UI-SHELL-REFRESH-001-SIM** | A | Enter Simulation, tick HUD — collapsed rail/tray, F3 dev-only | — | skipped (lib refresh) |
| **UI-SHELL-REFRESH-001-TEST** | A | `cargo test -p proc_A_dine01 --lib stage5` + `simulation_shell_phase2` | ☑ | sim-steward |
| **UI-SHELL-REFRESH-001-VISUAL** | A | Optional: `cargo run -p proc_A_dine01 --release -- --test visual` if spine ambiguity | — | skipped |
| **UI-SHELL-REFRESH-001-B** | B | **PASS** = proof-only, close 2B claim; **BLOCK** = numbered `@coder` items only | ☑ PASS | sim-steward |
| **UI-SHELL-REFRESH-001-C** | C | Refresh witness; parent **done** | ☑ | sim-steward |

### Shift A — Observe (copy-paste)

```
Lane: UI-SHELL-REFRESH-001-A
Agent: @sim-steward
Read: debug_runs/ui_shell_migration_live.json
      src/gui/hud/simulation_shell_phase2.rs
      src/gui/hud/hud_root_tick.rs
      src/gui/hud/simulation_session.rs (PLAY-01 defaults)
Map: phase2b_closed vs egui_pass_count_in_sim vs witness.* flags
Do NOT: implement; reopen UI-P2B architecture
```

### Shift B — Decide

```yaml
shift: B
issue:
  id: UI-SHELL-REFRESH-001
  severity: MED
root_cause: [stale proof frame | code regression | interaction not replayed]
evidence:
  - ui_shell_migration_live.json phase2b_closed / witness fields
  - simulation_shell_phase2 lib tests pass/fail
route:
  pass: UI-SHELL-REFRESH-001-C refresh witness only
  block: "@coder numbered list — max 3 files per item"
```

**PASS criteria:** `egui_pass_count_in_sim: 0`, `phase2b_closed: true`, passive witness flags match replay (or documented optional tails).

### Shift C — Act

```powershell
# After operator replay (tray expand, ops hover, build rail click, ESC)
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5
cargo run -p proc_A_dine01 --release -- --test visual
# Confirm debug_runs/ui_shell_migration_live.json timestamp + fields
```

---

## Follow-up steward packages (after primary)

| Priority | ID | When | Action | Status |
|:---:|:---|:---|:---|:---:|
| 1 | **STEWARD-WATER-WITNESS-001** | After coders land ocean/foam | [`steward_water_witness_gate_v1.md`](steward_water_witness_gate_v1.md) **GO** | ☑ |
| 2 | **S7P-STEWARD-001** | After **S7P-DESIGN-001** signs scenario | [`steward_s7p_gate_v1.md`](steward_s7p_gate_v1.md) **GO qualified** | ☑ |
| 3 | **STEWARD-VM-09-001** | Infra slice 2+ | [`steward_vm09_gate_v1.md`](steward_vm09_gate_v1.md) **GO**; **INFRA-PROJ2-001** → `@coder` | ☑ |

### STEWARD-WATER-WITNESS-001

```
Lane: STEWARD-WATER-WITNESS-001
Agent: @sim-steward
Prereq: WATER-W1-OCEAN-001 + WATER-W2-FOAM-001 coder slices
Read: debug_runs/stage5_full_app_live.json water_* block
Verify: ocean_tiles > 0 OR documented fixture; coast_foam > 0; strategic rows == 0 at low zoom
Do NOT: disable D-W09 cull globally
```

### S7P-STEWARD-001

```
Lane: S7P-STEWARD-001
Agent: @sim-steward
Prereq: stage7_play_scenario_v1.md header SIGNED (S7P-DESIGN-001)
Witness: debug_runs/stage7_play_live.json, industrial_activation_live.json,
         construction_stage_live.json, logistics_throughput_live.json
Optional: $env:RUST_ENGINE_STAGE7_PLAY_SEED=1 for demo Portland chain on sim enter
```

### STEWARD-VM-09-001

```
Lane: STEWARD-VM-09-001
Agent: @sim-steward
Prereq: INFRA-PREFLIGHT-001 GO
Read: vm09_gate_v1.md, post_stage6_vm09_audit.md
Route: INFRA-PROJ2-001 or view_representation reader — delegate @coder if >3 files
```

---

## Ledger DONE — do not re-run

| Gate / triage | Status |
|:---|:---|
| UI-P2-GATE | DONE |
| UI-P3-PREFLIGHT | DONE |
| IND-E01-WITNESS / S7P-IND-001 | DONE |
| P2-VFX triage (S-VFX) | DONE (board only) |
| UI-P3-M1-GATE (S-M1) | DONE |
| S-VM-09 slice 1 | DONE |
| STEWARD-WITNESS-SYNC-001 | DONE |
| STEWARD-SPARK-VFX-001 | DONE |
| **S7B-PREFLIGHT-001** | DONE |
| **FIRE7-PREFLIGHT-001** | DONE |

---

## Global commands

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 minimap_compositor
cargo orchestrate --skip-cargo
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | UI-SHELL-REFRESH-001 shift todos + follow-up steward rows |
