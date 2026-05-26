# Infra 5.5+ — VM-09 / projection authority / perf `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `INFRA-55` |
| **Version** | `1.0.0` |
| **Status** | **ACTIVE** |
| **Exit milestone** | **Infra 5.5 milestone** — VM-09 slice 1 + PROJ-2 sweep + perf baseline |
| **Slice 1 gate** | [`../vm09_gate_v1.md`](../vm09_gate_v1.md) — **S-VM-09 GO** 2026-05-24 |
| **Backlog** | [`../stage5_triage_backlog.md`](../stage5_triage_backlog.md) T1/T2/T4 |
| **Recovery** | [`../recovery_viewport.md`](../recovery_viewport.md) |

**Does not** replace Stage 5/6 operational gates.

---

## North star

**Single authority** per view for pose, projection, and overlay isolation — measured in infra JSON, not only FULL_APP green.

---

## Witness bundle

| File | Use |
|:---|:---|
| `debug_runs/infrastructure_view_isolation_live.json` | VM-06…11, minimap isolation |
| `debug_runs/viewport_drift.json` | extent / DPR drift |
| `debug_runs/viewport_authority_migration_witness.json` | migration debt |
| `debug_runs/stage5_full_app_live.json` | `view_isolation` block |
| `src/dev/perf_attribution_60s.md` | shell perf baseline (operator) |

---

## @designer instructions

**None required** for infra track. Consult on **preview vs main semantic** only for **INFRA-VM11-001** (audit checklist, no mocks mandatory).

Optional: annotate one screenshot where minimap click must **not** pan world main — attach to handoff.

---

## @sim-steward instructions

### INFRA-PREFLIGHT-001 (start every infra cycle)

**Shift A — Observe**

1. Read latest `infrastructure_view_isolation_live.json` + `viewport_drift.json`
2. Map writers: `view_authority.rs`, `viewport_pipeline.rs`, `MapCameraDesired`, `apply_minimap_camera_intent`
3. List dual-writer suspects

**Shift B — Decide**

```yaml
issue:
  id: INFRA-PREFLIGHT-001
  route: INFRA-VM09-001 | INFRA-PROJ2-001 | INFRA-PERF-001
  blockers: []
```

**Shift C — Act**

- Bounded fix ≤3 files **or** handoff YAML to `@coder`

**Playbooks:** [`../../tools/orchestrator/agents/viewport_cleanup_agent.md`](../../tools/orchestrator/agents/viewport_cleanup_agent.md), [`../../tools/orchestrator/agents/render_pipeline_agent.md`](../../tools/orchestrator/agents/render_pipeline_agent.md)

---

## @coder instructions

### Slice map (one row per cycle)

| ID | Goal | Source | Files (indicative) |
|:---|:---|:---|:---|
| **INFRA-VM09-001** | Document + fix one stray `MapCameraDesired` reader | TRIAGE-VM-09 | `gui/view_authority.rs`, call sites from grep |
| **PLAN-INFRA-PROJ2-001** | PROJ-2 sole writer + hit-test rollup | **DONE** (planner) | [`infra_proj2_sole_writer_plan_v1.md`](../infra_proj2_sole_writer_plan_v1.md) |
| **INFRA-PROJ2-001** | Per-view hit-test (`ViewId::Minimap` / `WorldPreview`) | **DONE** | § PROJ2-A in plan |
| **INFRA-PROJ2-CODER-B** | `ViewManager` sole `ResMut` writer (VM-06) | **DONE** | § PROJ2-B in plan |
| **INFRA-VM10-001** | Minimap lockstep diagnostics hardening | TRIAGE-VM-10 | `infrastructure_view_isolation` writer |
| **INFRA-VM11-001** | Preview semantic audit vs FULL_APP | TRIAGE-VM-11 | docs + 1–2 fixes |
| **INFRA-GPU-TILE-001** | Instanced tile authoritative | TRIAGE-GPU-TILE | `gpu_tile_debug`, WGSL |
| **INFRA-PERF-001** | 60s capture attribution | OPS-F01 | `frame_budget_diagnostics.rs` + md |

### Copy-paste — INFRA-VM09-001

```
Track: INFRA-55 — INFRA-VM09-001
Read: src/dev/stages/infra_55_execution_plan_v1.md
      src/dev/recovery_viewport.md
      src/dev/post_stage6_vm09_audit.md (if present)
Prereq: @sim-steward INFRA-PREFLIGHT-001 GO
First: rg MapCameraDesired writers; fix one callsite to ViewManager bridge
Do NOT: change Stage 5 readiness predicates; add minimap-only extract
Verify: cargo test -p proc_A_dine01 --lib stage5 view_authority
Witness: refresh infrastructure_view_isolation_live.json
```

### Copy-paste — INFRA-PROJ2-001

```
Track: INFRA-55 — INFRA-PROJ2-001
Read: render/extraction/render_projection_graph.rs (read-only policy)
First: find world_to_screen bypass; route through ViewProjectionAuthority
Max files: 3
Verify: cargo test -p proc_A_dine01 --lib stage5
```

### Acceptance — Infra 5.5 milestone

| # | Criterion |
|:---:|:---|
| I1 | VM-09: audit doc updated + ≥1 callsite fixed with test |
| I2 | PROJ-2: ≥5 call sites migrated or inventory in `post_stage6_vm09_audit.md` |
| I3 | `infrastructure_view_isolation_live.json` regression clean |
| I4 | `perf_attribution_60s.md` has one 2026 sample (operator) |
| I5 | FULL_APP still green after each merge |

---

## @operator — INFRA-PERF-001

1. Run sim 60s with `PERF=1` if supported
2. Paste top buckets into `src/dev/perf_attribution_60s.md`
3. No code required unless steward routes hotspot to coder

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Infra track plan |
