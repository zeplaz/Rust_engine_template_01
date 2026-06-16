# UI-OH-GATE-001 gate `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `UI-OH-GATE-001` |
| **Date** | 2026-05-25 |
| **Owner** | `@sim-steward` |
| **Scope** | Triage **Phase 2A + 2B** shell witnesses vs **Stage 5** spine |
| **Spec** | [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) |
| **Master lane** | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| **Witness** | [`debug_runs/ui_shell_migration_live.json`](../../debug_runs/ui_shell_migration_live.json) |

## Verdict: **PASS (qualified)**

**2A + 2B spine green** after lib refresh. Stage 5 FULL_APP **passes**. No `@coder` blockers for UI overhaul exit.

---

## Two columns (do not collapse)

| Column | What it proves | Verdict |
|:---|:---|:---:|
| **A — Phase 2B (egui retirement)** | `phase2b_closed`, `egui_pass_count_in_sim: 0`, `ui_oh_2b_001.green` | ✅ **PASS** |
| **B — Phase 2A (zones + interaction)** | `phase2a_closed`, `ui_oh_2a_001.green`, `ui_p2a_tail.*` | ✅ **PASS** (lib replay) |
| **C — Stage 5 cross-check** | `stage5_closure.passes`, `readiness.passes` | ✅ **PASS** |

**Qualified:** On-disk JSON can be **STALE** or **corrupt** after parallel `simulation_shell_phase2` tests (shared writer). Use **`steward_ui_oh_gate_001_lib_bundle`** or `--test-threads=1` for module runs.

---

## Shift A — Observe

### Before refresh (stale frame example)

| Path | Stale value | Spine impact |
|:---|:---|:---|
| `ui_oh_2a_001.green` | `false` while `phase2_zones_live: true` | **STALE** — not 2B regression |
| `ui_p2a_coder_b.green` / `ui_p2a_tail.*` | `false` | **STALE** until full witness commit |
| `witness.*` interaction flags | `false` | Partial writer frame |

### After `steward_ui_oh_gate_001_lib_bundle`

| Gate | Observed |
|:---|:---|
| `phase2a_closed` | ✅ `true` |
| `phase2b_closed` | ✅ `true` |
| `ui_oh_2a_001.green` | ✅ `true` |
| `ui_oh_2b_001.green` | ✅ `true` |
| `ui_p2b_coder_b_green` | ✅ `true` |
| `egui_pass_count_in_sim` | ✅ **0** |
| `ui_p2a_coder_b.green` | ✅ `true` |
| `ui_p2a_tail.f03_green` / `p4_auth_green` | ✅ `true` |
| `stage5_closure.passes` | ✅ `true` |
| `readiness.passes` | ✅ `true` |

### Non-blocking (qualified)

| Path | Note |
|:---|:---|
| `ui_p3_001.closed` | `false` at CPU-path proof frame — compositor witness authoritative |
| `phase2.minimap_gpu_path` | `false` in shell JSON — timing; see minimap compositor live JSON |
| `ui_p5_pause_001_green` | Phase 5 P2 — not UI-OH gate |

---

## Shift B — Route

```yaml
shift: B
issue:
  id: UI-OH-GATE-001
  severity: LOW
route:
  pass: close UI overhaul 2A/2B steward gate; maintain regression only
  monitor:
    - run simulation_shell_phase2 with --test-threads=1 if witness parse errors
    - optional --test visual for operator replay tails
  block: none for spine
```

**Do not** reopen **UI-P2B-001** / **UI-P2A-001** — code lanes **CLOSED**. Re-run **UI-SHELL-REFRESH-001** only if bundle test fails.

---

## Shift C — Act

```powershell
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 -- --test-threads=1
cargo test -p proc_A_dine01 --lib stage5
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial PASS — 2A/2B + stage5 triage |
