# PLAN-UI-PHASE6-001 — shell perf + multiview UI isolation `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-PHASE6-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — infrastructure **partial**; UI shell **maintain regression** |
| **Post-Stage 6** | [`post_stage6_design_plan.md`](post_stage6_design_plan.md) · [`post_stage6_active_todos.md`](post_stage6_active_todos.md) |
| **Stage 6 gate** | [`stage6_operational_signoff.md`](stage6_operational_signoff.md) **CLOSED** |
| **Witness** | `debug_runs/stage6_virtualization_live.json` · `debug_runs/infrastructure_view_isolation_live.json` · `debug_runs/ui_shell_migration_live.json` |

**No Rust in this deliverable.** UI-facing slice of post-Stage-6 **infrastructure hardening** — not Stage 5 spine.

---

## Executive summary

| Track | Verdict |
|:---|:---|
| **Shell perf (egui retirement)** | **PASS** — `egui_pass_count_in_sim: 0` in sim |
| **Multiview isolation (VM-08/10/11)** | **PARTIAL** — live proofs green; VM-09-v2 deferred |
| **Stage 6 residency window** | **CLOSED** — virtualization witness |
| **UI-OH regression** | **maintain only** — do not reopen Phase 2/3 |

**Distinction:** [`operational_readiness_vs_infrastructure_perf_v1.md`](../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) — FULL_APP green ≠ VM backlog closed.

---

## Gate chain

```text
Stage 5 spine (readiness.passes)           ☑
Stage 6 operational gate                   ☑ CLOSED
UI-P2B (egui_pass_count_in_sim: 0)         ☑
        │
        ▼
VM-08 per-view overlay isolation           ☑ witness vm08
VM-10/11 diagnostics                       ☑ infrastructure_view_isolation_live.json
        │
        ├─► TRIAGE-VM-09-v2                 ☐ deferred — S7B M2+ comm authority
        ├─► OPS-F01 perf attribution         ☐ operator
        └─► Construction multiview ghosts    ☐ DQ-POST-04
```

---

## PASS gate — shell perf

| # | Criterion | Witness | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| P6-1 | Sim egui passes | `ui_shell_migration_live.json` → `egui_pass_count_in_sim` | `0` | ☑ |
| P6-2 | 2B closed | `phase2b_closed` | `true` | ☑ |
| P6-3 | Bevy product path | `pause_menu_bevy`, dock shell | no egui pause in sim | ☑ |
| P6-4 | Minimap GPU default | `minimap_compositor_live.json` → `composite_path` | `GpuCompute` | ☑ |

**Lib anchor:**

```powershell
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
```

**Operator perf:** `debug_runs/perf_attribution_60s.md` — refresh under **OPS-F01**, not UI-OH blocker.

---

## PASS gate — multiview UI isolation

| # | Criterion | Witness file | Field | 2026-05-25 |
|:---:|:---|:---|:---|:---:|
| MV-1 | View fire isolation | `infrastructure_view_isolation_live.json` | `vm08` green | ☑ lib |
| MV-2 | VM-10/11 rollup | same | `vm_10` / `vm_11` | ☑ lib |
| MV-3 | Stage 6 window | `stage6_virtualization_live.json` | residency fields | ☑ |
| MV-4 | Viewport authority | `viewport_authority_migration_witness.json` | no drift | maintain |
| MV-5 | PROJ-2 sole writer | `infra_proj2_sole_writer_plan_v1.md` | extract contract | ☑ plan |

**Deferred (explicit):** **TRIAGE-VM-09-v2** — required before Stage 7 **M2+** comm authority in sim; does **not** block UI Phase 6 shell perf slice.

---

## Authority map (UI touch)

```text
Input / UI chrome
  → ViewportAuthority resolve
  → ViewManager (active view id)
  → MapViewInstance per slot
  → RenderProjectionContext (per-view masks)
  → minimap_compositor (single presentation RT)
```

**Forbidden:** Per-view duplicate minimap extract; egui overlay authority in **Simulation**; collapsing VM audits into Stage 5 green.

---

## Coder routing (forward)

| ID | Agent | When |
|:---|:---|:---|
| **TRIAGE-VM-09-v2** | @coder | Before S7B M2+ |
| **WC-D04-CODER-B** | @coder | Wave C / ops strip |
| **OPS-F01** | operator | 60s perf capture |
| **UI shell** | maintain | `steward_ui_oh_gate_001_lib_bundle` only |

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib stage5 stage6 infrastructure_view_isolation steward_ui_oh_gate_001_lib_bundle
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **PLAN-UI-PHASE6-001** signed |
