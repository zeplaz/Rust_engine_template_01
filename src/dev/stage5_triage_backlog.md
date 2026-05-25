# Stage 5 triage backlog (deferred — not closure gate)

Items here **proved sticky** or **scope-heavy** during Stage 5 convergence. They are **removed from the Stage 5 operational gate** so closure stays measurable.

**Re-apply when:** dedicated worker, Stage 5.5 / 6+, or product explicitly expands the gate (see [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md)).

**Do not** add these rows to `STAGE5_TODOS` or block `stage5_readiness_passes` on them.

---

## How to use

| Column | Meaning |
|--------|---------|
| **ID** | Backlog slug (not a STAGE5 todo id) |
| **Stage** | Suggested owning milestone |
| **Worker** | Suggested agent / lane |
| **Source** | Doc or code anchor |

**Pickup:** Move one row to an active runbook → implement → prove in **non–FULL_APP** witness or infra JSON → mark **Done** here.

---

## T1 — View / projection hardening (was blocking narrative in `base_finsh_5.md`)

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-VM-06 | Sole writer per `ViewId`; audit all pose paths | 5.5-infra | view-runtime / debug-intelligence | `base_finsh_5.md` §1 — **Done** (witness `vm_06`) |
| TRIAGE-VM-07 | Input routing only through view authority | 5.5-infra | coder + designer | `base_finsh_5.md` §1 — **Done** (`input_routing.rs`, witness `vm_07`) |
| TRIAGE-VM-08 | Filter / overlay isolation per view | 5.5-infra | coder | `base_finsh_5.md` §1 |
| TRIAGE-VM-09 | Eliminate global `MapCameraDesired` as authority (invert bridge) | 5.5-infra | sim-steward | `base_finsh_5.md` vm-09b v2 — **slice 1 GO** [`vm09_gate_v1.md`](vm09_gate_v1.md); track still **open** |
| TRIAGE-VM-10 | Minimap vs main lockstep diagnostics hardening | 5.5-infra | debug-intelligence | `infrastructure_view_isolation_live.json` |
| TRIAGE-VM-11 | Preview vs main semantic audit (beyond readiness flags) | 5.5-infra | designer + coder | `stage5_full_app_live.json` `view_isolation` |
| TRIAGE-PROJ-2 | Sweep `world_to_screen` not via `ViewProjectionAuthority` | 5.5-infra | coder | `base_finsh_5.md` proj-2 |

**Witness (non-gating):** `debug_runs/infrastructure_view_isolation_live.json`.

---

## T0 — Visual run (`--test visual`) — terminal blockers

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-VISUAL-SHADER-INST | `tile_debug_instanced.wgsl` naga `inst` scope panic | 5-render | coder | **VR-01** fixed (`tile_instance_color(flags, phase_lod)`) — verify on next visual run |
| TRIAGE-VISUAL-TEARDOWN | Graceful GPU surface exit on visual test (`arm_visual_test_graceful_exit`) | 5-render | coder | `gpu_surface_teardown.rs`, **VR-02** |
| TRIAGE-COMPILE-HYGIENE | Reconcile CW board vs live `cargo build` warnings | ops | main-thread-orchestrator | **VR-03**, `COMPILE_WARNINGS_TODOS.md` **CW-50** |

**Active board:** [`visual_run_blockers.md`](visual_run_blockers.md).

---

## T2 — GPU / debug draw (sticky implementation depth)

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-GPU-TILE | Instanced tile debug only; remove CPU gizmo fallback | 6-render | coder | `base_finsh_5.md` §2, `gpu-tile` row |
| TRIAGE-GPU-TILE-WGSL | WGSL storage instances + view-aware colors | 6-render | coder | `base_finsh_5.md` §2, **VR-01** |

**Note:** FULL_APP does not require zero gizmo path if instanced path is authoritative.

---

## T3 — Fire sim / streaming (sticky — not visual spine)

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-FIRE-F1 | Fuel + old-growth ignition gate (CPU overlay) | 7-fire-sim | coder | **Done** — [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md), `fire_ecology_live.json` |
| TRIAGE-FIRE-EXTRACT | `fire-view-extract-final` — per-view visible set hardening | 7-fire-sim | coder + sim-steward | `base_finsh_5.md` §3 |
| TRIAGE-FIRE-STREAM | Active/sleep chunk streaming, neighbor wake, budgets | 7-fire-sim | sim-steward | `base_finsh_5.md` §5 |
| TRIAGE-FIRE-LOD-TIERS | Strategic/operational/tactical/cinematic tier policy | 7-fire-sim | planner + coder | `base_finsh_5.md` §4 |
| TRIAGE-FIRE-OVERLAY-DBG | Fire overlay debug tooling | 7-fire-sim | designer | `base_finsh_5.md` |

**Note:** `TODO-10` / `TODO-11` prove **spine** (single extract + projection); full streaming is **not** Stage 5 gate.

---

## T4 — Phase D / F polish beyond readiness

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-PHASE-D-PARITY | Overlay parity stress / edge cases | 5.5-infra | debug-intelligence | `base_finsh_5.md` §6 |
| TRIAGE-PHASE-F-CULL | View-aware particle culling refinement | 6-render | coder | `base_finsh_5.md` §7 |
| TRIAGE-VT-DEEP | VT-4/5 extended proofs (camera isolation, LOD proof matrix) | 5.5-infra | debug-intelligence | `base_finsh_5.md` §8 |

---

## T5 — Parallel product lanes (never Stage 5 gate)

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-CONSTRUCTION | Build toolbox, roads, rail, queue UX | construction | coder + designer | `construction_stage_live.json` |
| TRIAGE-LOGISTICS-VIS | `log_rows=0` — logistics visual snapshot population | logistics | coder | [`logistics_visual_todos.md`](logistics_visual_todos.md) — **partial** (VIS-01..04 code; confirm on visual run) |
| TRIAGE-PERF-SHELL | Frame wall time / logging / egui cost | perf | main-thread-orchestrator | `operational_readiness_vs_infrastructure_perf_v1.md` §2 |
| TRIAGE-REPLAY | Deterministic replay + editor parity | infra | sim-steward | `replay_editor_parity_live.json` |
| TRIAGE-STAGE6-VIRT | Virtualization / multiview scale | 6 | orchestrator | directive §10 |

---

## T6 — Docs / process debt

| ID | Work | Stage | Worker | Source |
|----|------|-------|--------|--------|
| TRIAGE-BASE-FINISH-5-SPLIT | Keep `base_finsh_5.md` as **reference only**; gate = this triage + close checklist | docs | planner | User directive 2026-05-22 |
| TRIAGE-ORCH-N21 | `map_view/mod.rs` STABLE after live visual witness | ops | cleanup-intelligence | `tools/orchestrator/NEXT.md` N-21 |

---

## Promotion rule (sticky → active)

Promote a triage row back into engineering **only if**:

1. FULL_APP regresses and root-cause maps to that row, **or**
2. Product expands Stage 5 gate in writing (directive + AGENTS.md), **or**
3. A dedicated stage owner accepts it with its own witness file.

Otherwise keep in triage and assign a **future stage** worker.
