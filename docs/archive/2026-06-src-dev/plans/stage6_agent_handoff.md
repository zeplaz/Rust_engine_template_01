# Stage 6 — agent handoff packages

**Board:** [`stage6_active_todos.md`](stage6_active_todos.md)  
**S6-0:** **DONE** — [`stage6_live_proof.rs`](../render/stage6_live_proof.rs)

Use this file to assign slices. Each package includes **agent type**, **todo IDs**, **entry prompt**, and **done proof**.

---

## Routing summary

| Agent | Owns | Do not |
|-------|------|--------|
| **coder** | S6-1 implementation, S6-2 atlas/async, fire/overlay cull | Invent BQ thresholds; second `RepresentationResult` |
| **sim-steward** | Residency authority, `ChunkResidencyTable` population, VM alignment | UI egui layout |
| **designer** | S6-P2, S6-25, BQ-128/130 UX | ECS mutation paths |
| **debug-intelligence** | S6-3 witness triage, FULL_APP regression analysis | Feature implementation |
| **planner** | DQ overrides, BQ rows in rulebook | Code |
| **main-thread-orchestrator** | Stuck subagents, `cargo orchestrate` cycles | — |
| **orchestrator** | Queue priority, slice pick | — |

---

## Package A — **coder** (S6-1 residency authoritative)

**Todos:** S6-10, S6-11, S6-13, S6-14, S6-15, S6-17, S6-18  
**Depends:** S6-0 done; coordinate **Package B** (S6-12) first for residency population.

### Entry prompt (copy to subagent)

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6.1 — residency authoritative (coder)

Read: docs/archive/2026-06-src-dev/plans/stage6_active_todos.md (S6-1 rows), src/render/stage6_virtualization.rs,
  src/render/stage6_live_proof.rs, src/gui/hud/stage6_telemetry.rs, src/gui/hud/dock_shell.rs,
  src/render/fire_view_extract.rs, src/gui/editor/world_preview/gpu_preview.rs

Tasks:
1. S6-10: Ensure refresh_stage6_hud_telemetry runs AFTER publish_stage6_virtualization_frame (schedule order).
2. S6-11: Remove mock_residency_overlay_consumer fallback in dock_shell.rs — use Stage6HudTelemetry only; empty state shows zeros not mock.
3. S6-13: In sync_visible_fire_chunks_from_views (fire_view_extract.rs), intersect visible chunks with Stage6VirtualizationFrame.consumer_window_coords when non-empty.
4. S6-14: Cap SharedOverlayFieldBuffers / overlay extract rows using residency window (no full-world scan).
5. S6-15: Add/extend test for gpu_preview residency cull; confirm witness field in live JSON.
6. S6-17: Verify ResidencyOverlayConsumerDto ghost/core in live JSON after sim.
7. S6-18: Lib test: stage6_readiness_passes with populated ChunkResidencyTable fixture.

Rules: attach to spine; no parallel RepresentationResult; cargo test -p proc_A_dine01 --lib must pass.
Mark completed rows [x] in stage6_active_todos.md.
```

### Done proof

- `cargo test -p proc_A_dine01 --lib stage6`
- `debug_runs/stage6_virtualization_live.json` → `residency_chunk_count > 0`, `stage6_readiness.violations` shrinking
- F3 panel shows live residency counts (not mock 12/4)

---

## Package B — **sim-steward** (S6-12 residency population)

**Todos:** S6-12, S6-16 (verify), S6-T1 prep  
**Blocks:** Package A S6-13/14/18

### Entry prompt

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6.1 — ChunkResidencyTable authoritative (sim-steward)

Read: src/io/streaming/residency.rs, src/render/stage6_virtualization.rs,
  docs/archive/2026-06-src-dev/plans/view_runtime_architecture_v1.md, AGENTS.md Stage 6 rules

Tasks:
1. S6-12: Ensure ChunkResidencyTable is populated during normal sim enter / worldgen (not empty in live witness).
   - Trace who inserts entries; add init/sync if missing after world load.
2. S6-16: Confirm gather_wave_c_readiness already in stage6_live_proof JSON (should be done — verify wave_c_ok).
3. Document residency single-writer in recovery_viewport.md if new path added.

Rules: residency owns membership; sim/render only consume. No second overlay truth.
cargo test -p proc_A_dine01 --lib
Update stage6_active_todos.md checkboxes.
```

### Done proof

- `stage6_virtualization_live.json` → `frame.residency_chunk_count > 0` after 90f sim
- `stage6_readiness.report.residency_populated: true`

---

## Package C — **coder** (S6-2 atlas + async)

**Todos:** S6-20, S6-21, S6-22, S6-23, S6-24, S6-26  
**Start after:** S6-1 green (readiness mostly true)

### Entry prompt

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6.2 — atlas, async apply, per-view windows (coder)

Read: src/render/stage6_virtualization.rs, src/render/view_runtime/per_view_policy.rs,
  src/gui/hud/frame_budget_diagnostics.rs, src/io/streaming/, prompts/guides/base_finsh_5.md §2

Tasks:
1. S6-20/21: Tie PagedAtlasResidency to real GPU upload bytes; change gather_stage6_readiness atlas_slots_active per DQ-S6-03 (upload > 0).
2. S6-22: Document + schedule-order test for AsyncDomainApplyQueue main-thread ECS apply.
3. S6-23/24: Per-ViewSurfaceId residency window on Stage6VirtualizationFrame; apply with PerViewRepresentationPolicy fire caps.
4. S6-26: FrameBudgetDiagnostics.stage6 churn anomalies + sample line in perf_attribution_60s.md

cargo test -p proc_A_dine01 --lib
```

---

## Package D — **debug-intelligence** (S6-3 exit gate)

**Todos:** S6-30, S6-31, S6-32, S6-33, S6-34  
**Start after:** S6-2 complete

### Entry prompt

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6.3 — operational exit (debug-intelligence)

Read: debug_runs/stage6_virtualization_live.json, stage5_full_app_live.json,
  docs/archive/2026-06-src-dev/plans/stage6_active_todos.md S6-3, src/render/stage6_live_proof.rs

Tasks:
1. Run sim 60s+; confirm stage6_readiness.passes true in live JSON (not fixture-only).
2. Verify F3 BQ-134 panel uses authoritative DTO (S6-31).
3. Run cargo run -p proc_A_dine01 --release -- --test visual; confirm stage5_full_app_live.json no regression.
4. Write docs/archive/2026-06-src-dev/plans/stage6_operational_signoff.md; update stage6_plan_open.md §11.

Report: violations list, authority drift, recommended triage rows only.
```

---

## Package E — **designer** (parallel, non-blocking)

**Todos:** S6-P2, S6-25, S6-S2 (optional)  
**Start:** anytime; does not block S6-1

### Entry prompt

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6 UX / Wave S (designer)

Read: docs/archive/2026-06-prompts-guides/runbooks/guides/experience_layer_ux_hud_designer_brief_v1.md §8,
  docs/archive/2026-06-prompts-guides/runbooks/guides/rulebook_backlog_designer_brief_v1.md §4,
  src/gui/hud/stage6_consumer.rs

Tasks:
1. S6-P2: Decide inspector/registry table surface (egui F8 vs external tool) — add BQ row if needed.
2. S6-25: Spec minimap ghost-band tint (defer implementation if not S6 exit).
3. S6-S2: If HUD layout Wave S schema locked, document slot beside ProductShellPersistenceBundleR8.

Deliverable: markdown decision in src/dev/ or BQ row — no Rust unless schema locked.
```

---

## Package F — **planner** (DQ lock / BQ promotion)

**Todos:** DQ-S6 overrides only if product disagrees; S6-S1/S6-S3 sequencing  
**When:** before Wave S code expansion

### Entry prompt

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6 planning (planner)

Read: docs/archive/2026-06-src-dev/plans/stage6_plan_open.md §6, stage6_active_todos.md DQ table,
  docs/archive/2026-06-prompts-guides/runbooks/guides/backlog_serialization_preview_streaming_runbook_v1.md

Output: confirm or amend DQ-S6-01…10 defaults; list BQ-128/130/133/134 promotion order for Wave S after S6-1.
No code changes unless user requests.
```

---

## Package G — **main-thread-orchestrator** (ops / perf)

**Todos:** S6-T3, S6-06 refresh, orchestrator slice  
**When:** after S6-1 or on CI failure

### Entry prompt

```
Repo: c:\dev\github\Rust_engine_template_01
Lane: Stage 6 ops (main-thread-orchestrator)

Run: cargo test -p proc_A_dine01 --lib
Optional: cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-clippy --skip-test
Update tools/orchestrator/queues/HANDOFF.md with witness dates.
S6-T3: one perf_attribution_60s.md sample if STALL=1 session available.
```

---

## Wave C / Stage 7 (queued — not S6 exit)

| Package | Agent | Todos | When |
|---------|-------|-------|------|
| Streaming depth | **coder** + **sim-steward** | S6-C1, S6-C2 | After S6-22 |
| VM-C shim | **sim-steward** | S6-T1 | After S6-23 |
| Fire streaming | **coder** | S6-T2 | Stage 7-prep |
| Wave S save | **coder** + **designer** | S6-S1, S6-S3 | After S6-1 |

---

## Launch order (recommended)

1. **sim-steward** — Package B (S6-12) — *unblocks readiness*  
2. **coder** — Package A (S6-1 rest) — *parallel once B started*  
3. **coder** — Package C (S6-2) — *after S6-1*  
4. **debug-intelligence** — Package D (S6-3)  
5. **designer** — Package E (parallel anytime)

---

## Sign-off

| Date | Action |
|------|--------|
| 2026-05-23 | Handoff packages created; agents ready to launch |
