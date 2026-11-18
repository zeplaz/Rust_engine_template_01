# PHASE-NEXT-2026-05-28 — Fleet phase plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-FLEET-PHASE-NEXT-001** |
| **Working title** | **PHASE-NEXT-2026-05-28** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **SUPERSEDED for open work** by [`plan_fleet_stability_integrity_001_v1.md`](plan_fleet_stability_integrity_001_v1.md) (PHASE-STABLE-2026-06). Closed rows in §3 remain authoritative. |
| **Horizon** | 2–4 engineering weeks |
| **Prior audit** | [`planner_status_audit_v15.md`](planner_status_audit_v15.md) (superseded for open tails by **v16**) |
| **Returns reconcile** | [`fleet_snapshot_20260528_v2.md`](fleet_snapshot_20260528_v2.md) |
| **Coder slices** | [`plan_fleet_phase_next_exec_001_v1.md`](plan_fleet_phase_next_exec_001_v1.md) |
| **Ledger** | [`planner_status_audit_v16.md`](planner_status_audit_v16.md) |

**Rule:** Witness JSON wins. Do **not** reopen archived wave 6 exec plans (parametric, R4 impl, WSS PR-3/4 exec) unless disk regression.

---

## 1. Executive summary

Wave 6 **product closure is achieved on disk**: Stage 5 FULL_APP (`readiness.passes: true`), WSS substrate rollup green (PR-2→PR-5, atmos, hydro, post-spine), construction operational, Stage 7 M1–M4 + steward, minimap GPU compositor, and industrial/logistics witnesses are green. The next phase **does not add features** — it **ships measurable simulation quality**: release `--test visual` for 60 s **without `RASTER_*`**, documented p95 baselines, witness containment completion, and scoped WSS/infra depth **without** collapsing infrastructure into the Stage 5 gate. **North star:** operators can run one clean script, get green witnesses + p95 under budget, and coders finish containment migration so release builds never depend on witness I/O.

---

## 2. Phase decision

### Recommended primary mega-theme: **SHIP-QUALITY**

Rationale: Product spine witnesses are green; the largest **credibility gap** is **OPS-F01** (no release 60 s p95 table in [`debug_runs/perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md)) and **perf witness refresh** (`visual_witness` / `perf_attribution_60s` blocks exist in code but are **absent** from current `stage5_full_app_live.json`). Finishing PERF-VIS P1-B → P4 closes the “emergency env var” era and gives CI a measurable bar.

### Recommended secondary mega-theme: **ENGINEERING-HYGIENE**

Rationale: `runtime_witness/` B–C + minimap Slice 1 landed; **8 domain shims** remain. Completing containment (Slices 2–7) prevents new `live_proof.rs` sprawl and enables `-HardFail` CI without blocking sim when `RUNTIME_WITNESS_WRITES_FORCE_OFF=1`.

### Rejected / deferred as primary

| Theme | Verdict | Rationale |
|:---|:---|:---|
| **WSS-DEPTH** | **Secondary (P2 tail)** | Substrate spine green on disk; F2 extract and deformation are **depth**, not ship blockers |
| **INFRA-UX** | **Deferred P3** | `ui_shell_migration_live.json` has many reds but FULL_APP + PLAY-01 sim HUD green; VM-09 v2 is triage, not gate |
| **PRODUCT-R4** | **Deferred optional** | [`plan_construction_r4_product_open_001_v1.md`](plan_construction_r4_product_open_001_v1.md) policy SIGNED — no impl charter until product names scope |

---

## 3. Closed — do not re-plan

Summarized from disk + code (2026-05-28 spot-check). **Do not reopen** unless witness regression.

| Domain | Evidence |
|:---|:---|
| Stage 5 / 6 operational gates | `stage5_full_app_live.json` → `readiness.passes: true`, `stage6_virtualization_live.json` green |
| Wave S / P / C save + preview + streaming | `wave_s_hydrate_live.json`, `wave_p_live.json`, `wave_c_live.json` |
| WSS PR-2→PR-5 + atmos + hydro + smoke + post-spine | `wss_substrate_live.json` → top `green: true`, `ecs_retire_fixture_green: true`, `wss_post_spine_001.green: true`, `hybrid_ecs_smoke_authoritative: false` |
| Construction operational + parametric + R4 + BQ-128 | `construction_stage_live.json` → `operational_green: true` |
| Industrial + LOG-E01 visual confirm | `industrial_activation_live.json`; `log_e01_fullapp_upgrade_001.full_visual_confirm: true` |
| Stage 7 M1–M4 + steward | `stage7_behavioral_live.json` → `s7b_m3_green`, `s7b_steward_green`, `s7b_m4_play_green`, `play_enqueue_wired: true` |
| Fire F7 A/B/C streaming spine | `fire_streaming_live.json`, `fire_ecology_live.json` |
| Infra slice 3 / WC-D04 | `stage6_virtualization_live.json`, `infrastructure_view_isolation_live.json` |
| Minimap M3 + replay | `minimap_compositor_live.json` → `composite_ok`, `presentation_source: SharedRenderTargetImage` |
| Planner horizon exec | perf, containment, S7 M4 play — **SIGNED** |
| Designer perf degrade UX | [`visual_perf_spike_degrade_ux_v1.md`](visual_perf_spike_degrade_ux_v1.md) **PASS** |
| **Coder A landed (partial DoD)** | P1-A, P2-A, P2-C cadence, P1-C CI, DEV-CONTAIN B–C + minimap Slice 1 — see §Inherited slice status |

**Coder B:** **drained** — no blocking product rows.

---

## 4. Three horizons (2–6 weeks)

| Horizon | Weeks | Goal | Exit |
|:---|:---|:---|:---|
| **Operational readiness** | 1–2 | Ship-quality sim perf + witness truth | 60 s release visual, no `RASTER_*`; p95 frame &lt; 33 ms documented; `stage5` perf block on disk |
| **Infrastructure hardening** | 2–4 | Containment + viewport/shell tails | `check_live_proof_containment.ps1 -HardFail` green; optional UI-OH / VM-09 v2 scoped rows |
| **WSS depth** | 3–6 (parallel tail) | F2 extract + substrate maturation | `f2_extract_witness.green: true`; Hanabi **feature-only**; deformation/vector_shapes **plan-only** |

---

## 5. Authority map

| Layer | Owns | Must NOT dual-write |
|:---|:---|:---|
| **L1 WSS substrate** | `src/substrate/*` slabs, hydrology, atmos sim fields | GPU particles, Hanabi, witness JSON |
| **L2 extraction** | `RepresentationResult`, `RenderProjectionGraph`, `FireVisualFramesByView`, `resolve_minimap_texture_source` | File I/O, env throttles in release |
| **L3 GPU / Hanabi** | Compositor, particles, optional `HanabiEmbellishmentPlugin` (**`hanabi_l3` feature only**) | L1 slab mutation |
| **Witness I/O** | `src/dev/runtime_witness/*` + gate | Domain `std::fs` in `render/`, `construction/`, etc. |
| **Perf policy** | `TileRasterBudget`, `FireExtractCadence`, `UxFrameSpikeGuard` | Silent drop of projection graph / readiness contracts |
| **Viewport** | `SimulationMapViewport` single commit, `ResolvedViewports` | Parallel ortho/scissor from ad-hoc camera paths |

---

## 6. Inherited slice status (PERF-VIS + CONTAIN)

Evidence from code + disk (audit v15 **stale** on S7/WSS — refreshed in v16).

### PERF-VIS ([`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md))

| Slice | Status | Evidence |
|:---|:---|:---|
| P1-A duplicate CPU minimap skip | **DONE** | `tile_fallback_cpu_minimap_raster_needed`, `TileFallbackRasterPolicy` in `tile_world_fallback.rs` |
| P1-B GPU minimap default in Simulation | **PARTIAL** | `apply_simulation_map_presentation_defaults` sets GPU when compositor **env** on — ship path should follow `composite_ok` without env |
| P1-C runbook + CI no `RASTER_*` | **DONE** | `check_visual_runbook_no_raster_env.ps1`; `.github/workflows/ci.yml` + `tools/orchestrator/ci/run.ps1` |
| P2-A `TileRasterBudget` | **DONE** | `src/render/visual_perf_budget.rs`; release ignores `RASTER_*` in tests |
| P2-B spike → budget feedback | **OPEN** | No EMA feedback from `FrameBudgetDiagnostics` into `TileRasterBudget` yet |
| **P2-C** `FireExtractCadence` | **PARTIAL** | Wired; `residency_scoped: true` in release cadence; p95 gate = live attribution |
| **P2-D** residency-scoped extract | **PARTIAL** | Query bounded when residency table populated; p95 **not** lib-gated |
| P3 viewport stability | **PARTIAL** | `VisualReadinessWitness` + `RenderHoleLatch.steady_flip_count` in code; **not** in live `stage5_full_app_live.json` |
| P4 60 s acceptance + witness p95 | **PARTIAL** | `PerfAttributionWitness` in `perf_attribution_witness.rs`; `perf_attribution_60s.md` **no release measured p95 table**; OPS-F01 open |

### DEV-CONTAIN ([`plan_dev_artifact_containment_exec_001_v1.md`](plan_dev_artifact_containment_exec_001_v1.md))

| Slice | Status | Evidence |
|:---|:---|:---|
| Phase 0 + gate/io | **DONE** | `runtime_witness/gate.rs`, `io.rs`, `containment.rs` |
| wave_c, wave_s, stage6, view_runtime | **DONE** | `runtime_witness/{wave_c,wave_s,stage6,view_runtime}.rs` |
| Slice 1 minimap | **DONE** | `runtime_witness/minimap.rs`; shim in `minimap_compositor/live_proof.rs` |
| Slices 2–7 (construction → wss, CI hard-fail retire) | **OPEN** | No `runtime_witness/construction.rs`; shims in `exceptions_manifest.json` |
| CI `-HardFail` | **PARTIAL** | `tools/orchestrator/ci/run.ps1` invokes `-HardFail`; shims still allowed via manifest |

---

## 7. Phased execution plan (P0→P3)

### P0 — Ship acceptance gate (week 1, ~5 days)

**Goal:** Operator + coder prove release visual perf without env throttles; refresh stage5 witness with perf block.

| ID | Slice | Owner | Files | Tests | Witness keys |
|:---|:---|:---|:---|:---|:---|
| P0-1 | **OPS-F01** 60 s release capture | @operator | `debug_runs/perf_attribution_60s.md` | `run_visual_test_clean.ps1 -Release` | Dated p95: frame, `raster_b`, `view_fire` |
| P0-2 | Refresh stage5 with `visual_witness` | @coder A | `visual_readiness_witness.rs`, `stage5_full_app_harness.rs` | `cargo test -p proc_A_dine01 --lib stage5` | `readiness.visual_witness.*`, `perf_attribution_60s.p95_*` |
| P0-3 | Verify CI guards | @operator | `ci.yml`, orchestrator scripts | containment + runbook scripts | N/A |
| P0-4 | Queue hygiene `done_2026_05_28` reconcile | @planner | `coder_active_queue.json` | — | Queue matches disk |

**Exit criteria:** `perf_attribution_60s.md` § **2026-05-28 release** with HW baseline; p95 frame &lt; 33 ms **or** documented hardware-bound exception; no `RASTER_*` env.

**Parallel:** P0-1 ∥ P0-2 after first clean run recipe verified.

---

### P1 — PERF-VIS completion (weeks 1–2, ~8 days)

**Goal:** Close remaining perf exec slices; attach policy to contracts not env.

| ID | Slice | Owner | Files | Tests | Witness |
|:---|:---|:---|:---|:---|:---|
| P1-1 | P1-B compositor-driven GPU default | @coder A | `simulation_session.rs`, `minimap_compositor/pass.rs` | `minimap_compositor`, `stage5` | `minimap_compositor_live.json` → `presentation_source` |
| P1-2 | P2-B spike EMA → chunk cap | @coder A | `visual_perf_budget.rs`, `frame_budget_diagnostics.rs`, `tile_world_fallback.rs` | `chunk_grid_tests` | PERF `raster_b` p95 &lt; 12 ms |
| P1-3 | P2-D residency-scoped fire extract | @coder A | `fire_visual_extract.rs`, `fire_chunk_runtime.rs` | `fire`, `stage5` lib (scope wiring only) | **Runtime** p95 `view_fire` &lt; 8 ms via OPS-F01 — not lib gate |
| P1-4 | P3 viewport witness on disk | @coder A | `map_camera.rs`, `authoritative_viewport.rs`, `visual_readiness_witness.rs` | visual + `stage5` | `render_hole_steady_flip_count: 0` |
| P1-5 | P4 perf acceptance rollup | @coder A | `perf_attribution_witness.rs`, `plan_visual_perf_production_exec_001_v1.md` §Baseline | lib + operator re-run | `stage5_full_app_live.json` perf block green |

**Exit criteria:** PERF-VIS exec §9 DoD checkboxes **true** on disk; runbook clean path is only operator recipe.

**Serial:** P1-1 before P1-2 (minimap authority); P1-3 ∥ P1-2 after P1-1.

---

### P2 — Witness containment (weeks 2–3, ~10 days, parallel with P1 tail)

**Goal:** All witness writers in `runtime_witness/`; release safe when writes off.

| ID | Slice | Owner | Files | Tests | Witness |
|:---|:---|:---|:---|:---|:---|
| P2-1 | Construction writer | @coder A | `runtime_witness/construction.rs`, shim | `construction` lib | `construction_stage_live.json` |
| P2-2 | Industrial + logistics | @coder A | `runtime_witness/industrial.rs`, `logistics.rs` | economy lib tests | `industrial_activation_live.json`, `logistics_throughput_live.json` |
| P2-3 | Fire + wave_p | @coder A | `runtime_witness/fire.rs`, `wave_p.rs` | fire, wave_p | `fire_ecology_live.json`, `wave_p_live.json` |
| P2-4 | Stage7 behavioral + play | @coder A | `runtime_witness/stage7_*.rs` | `stage7_behavioral`, `stage7_play` | behavioral + play JSON |
| P2-5 | WSS substrate writer | @coder A | `runtime_witness/wss_substrate.rs` | `wss_substrate` | `wss_substrate_live.json` |
| P2-6 | Retire shims + manifest trim | @coder A | `exceptions_manifest.json`, domain shims | `check_live_proof_containment.ps1 -HardFail` | parity diff all `*_live.json` |

**Exit criteria:** DEV-CONTAIN exec §9 DoD; `-HardFail` green.

**Parallel:** P2-1 ∥ P2-2 ∥ P2-3 (separate PRs); P2-6 serial last.

---

### P3 — Depth + infra optional (weeks 3–4, ~10 days)

**Goal:** Scoped WSS/infra without reopening wave 6 product exec.

| ID | Slice | Owner | Files | Tests | Witness |
|:---|:---|:---|:---|:---|:---|
| P3-1 | **FIRE-F2-EXTRACT-001** | @coder A | `fire_view_extract.rs`, projection graph, harness | `stage5`, F2 lib | `f2_extract_witness.green: true` |
| P3-1b | **VFX-VECTOR-SHAPES-001** | @coder A | `tactical_vector_overlay.rs`, `Cargo.toml` | unit witness JSON | `tactical_vector_overlay.backend: bevy_vector_shapes`, `drawn_shapes > 0` |
| P3-2 | Hanabi **policy hold** | @planner | charter only | `hanabi_validation` | `hanabi_l3_plugin_wired: false` in default binary |
| P3-3 | **STAGE5-VT-FLICKER / VR-04** | @coder A | [`visual_run_blockers.md`](visual_run_blockers.md), `vt_spatial_invariants.rs` | **`cargo run -- --test visual`** + `vt_ci_matrix` lib | Visual log: no sustained VT-5 fail; lib CI ≠ live-only fix |
| P3-3 | UI shell **UI-W3-2B** tail (optional) | @coder A + @designer on-call | `simulation_shell_phase2.rs`, `ui_shell_migration_live.json` writers | `ui_p2b`, `simulation_shell_phase2` | `ui_w3_2b_001.green` |
| P3-4 | VM-09 v2 invert bridge (optional) | @coder A | [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md) | `infrastructure_view_isolation` | `vm_09` fields |
| P3-5 | WSS depth **plan-only** | @planner | deformation / `bevy_vector_shapes` charter update | — | — |
| P3-6 | R4 **product** board (optional) | @planner + @designer | existing policy doc | — | `product_board_open` |

**Exit criteria:** F2 green **or** explicit defer with planner sign-off; infra slices **optional** — pick ≤2 per cycle.

---

## 8. Parallelization matrix

| Lane | Parallel with | Must be serial after |
|:---|:---|:---|
| P0 operator 60 s | P1 code prep | — |
| P1-1 minimap default | P2-1 construction contain | P1-2 raster feedback |
| P1-2 ∥ P1-3 perf | P2-* containment | P1-1 |
| P2-1..3 containment | each other | P2-6 shim retire |
| P3-1 F2 extract | P2 tail | Stage 5 spine tests green |
| P3-3 UI shell | P1-4 viewport | designer sign-off if UX unclear |

---

## 9. Role routing

| Role | P0 | P1 | P2 | P3 |
|:---|:---|:---|:---|:---|
| **@planner** | Ledger v16, queue rows | Track PERF DoD | Track CONTAIN DoD | F2 / R4 charter only |
| **@designer** | — | On-call if degrade UX drift | — | UI-W3-2B if P3-3 picked |
| **@coder A** | P0-2 | P1-1..P1-5 | P2-1..P2-6 | P3-1, optional P3-3/4 |
| **@coder B** | — | **Stand down** | — | Optional infra-only if A blocked |
| **@operator** | **P0-1 primary** | Re-run 60 s after P1 | Witness index refresh | OPS-F03 stage6 optional |

---

## 10. Witness & acceptance matrix

### p95 targets (release, GPU minimap on, no `RASTER_*`)

| Metric | Target | Read |
|:---|:---|:---|
| Frame p95 | **&lt; 33 ms** | `perf_attribution_60s.md` or `stage5` → `perf_attribution_60s.p95_frame_ms` |
| `raster_b` p95 | **&lt; 12 ms** | PERF line / witness |
| `view_fire` p95 | **&lt; 8 ms** | **OPS-F01 / live** PERF attribution (`perf_attribution_60s.md` or stage5 perf block) — residency scope is code exit; p95 is **operator-measured**, not `cargo test` alone |

### Files touched this phase

| File | Keys to hold green / new |
|:---|:---|
| `stage5_full_app_live.json` | `readiness.passes`, **new** `visual_witness`, `f2_extract_witness.green` |
| `minimap_compositor_live.json` | `composite_ok`, `presentation_source`, `dual_minimap_present: false` |
| `wss_substrate_live.json` | top `green`, post-spine, ecs retire (maintain) |
| `stage7_behavioral_live.json` | M3/M4/steward (maintain) |
| `construction_stage_live.json` | `operational_green` (maintain) |
| `ui_shell_migration_live.json` | optional: `ui_w3_2b_001.green` |
| `infrastructure_view_isolation_live.json` | optional: VM-09 v2 |
| All migrated `*_live.json` | schema-compatible after containment |

### Verification bundle

```powershell
.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 chunk_grid_tests
.\tools\orchestrator\scripts\check_visual_runbook_no_raster_env.ps1
.\tools\orchestrator\scripts\check_live_proof_containment.ps1 -HardFail
```

---

## 11. Risk register

| Risk | Phase | Mitigation | Rollback |
|:---|:---|:---|:---|
| p95 fails on reference HW | P0 | Document baseline; tune `TileRasterBudget` not env | Revert P1-2 cap only |
| Containment parity drift | P2 | Shim + JSON diff per slice | Keep shim; revert writer move |
| F2 extract breaks sparks | P3 | Keep overlay bootstrap fallback | Revert graph path; F2 stays open |
| UI shell scope creep | P3 | Max one UI-W3 row per cycle | Defer P3-3 |
| Hanabi default wire | P3 | Charter: feature flag only | N/A — policy |
| Stage 5 regression | All | `cargo test --lib stage5` each PR | Revert slice; spine first |

---

## 12. Definition of done (mega-phase)

- [ ] **OPS-F01** release 60 s section in `perf_attribution_60s.md` with HW baseline + p95 table
- [ ] **PERF-VIS** exec DoD: no release `RASTER_*`; p95 targets met or documented exception
- [ ] **DEV-CONTAIN** exec DoD: `-HardFail` green; no domain `std::fs` witness writes
- [ ] `stage5_full_app_live.json` includes refreshed `visual_witness` / perf attribution block
- [ ] `planner_status_audit_v16.md` **SIGNED**; elemental index **v1.2**
- [ ] Machine queues `next_phase` → **PHASE-NEXT-2026-05-28**
- [ ] Optional P3 rows (F2, UI, VM-09) **explicitly** CLOSED or DEFERRED in v16

---

## 13. Machine queue updates

Apply to [`tools/orchestrator/queues/coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json), [`planner_active_queue.json`](../../tools/orchestrator/queues/planner_active_queue.json), [`designer_active_queue.json`](../../tools/orchestrator/queues/designer_active_queue.json):

### `next_phase` block (all roles)

```json
{
  "phase_id": "PHASE-NEXT-2026-05-28",
  "plan_doc": "src/dev/plan_fleet_phase_next_001_v1.md",
  "exec_doc": "src/dev/plan_fleet_phase_next_exec_001_v1.md",
  "audit": "src/dev/planner_status_audit_v16.md",
  "primary_theme": "SHIP-QUALITY",
  "secondary_theme": "ENGINEERING-HYGIENE",
  "rule": "Witness JSON wins; do not reopen wave 6 archived exec"
}
```

### `coder_active_queue.json` → `coder_a.active[]`

| id | priority | plan_doc | witness |
|:---|:---:|:---|:---|
| `PHASE-NEXT-P0-2` | 1 | `plan_fleet_phase_next_exec_001_v1.md` | `stage5_full_app_live.json` |
| `PHASE-NEXT-P1-1` | 2 | `plan_visual_perf_production_exec_001_v1.md` Slice 1 | `minimap_compositor_live.json` |
| `PHASE-NEXT-P1-2` | 3 | perf exec Slice 4 | PERF / stage5 perf block |
| `PHASE-NEXT-P1-3` | 4 | perf exec Slice 5 | fire extract p95 |
| `PHASE-NEXT-P2-1` | 5 | `plan_dev_artifact_containment_exec_001_v1.md` Slice 2 | `construction_stage_live.json` |

### `planner_active_queue.json` → `active[]`

| id | deliverable |
|:---|:---|
| `PLAN-LEDGER-REFRESH-016` | `planner_status_audit_v16.md` after P0 operator return |
| `PLAN-FLEET-PHASE-NEXT-001` | this doc — **SIGNED** |

### `designer_active_queue.json` → on-call only

| id | when |
|:---|:---|
| `DESIGN-UI-W3-2B-001` | If P3-3 picked |
| `DESIGN-CONSTRUCTION-R4-PRODUCT-001` | If product board scopes features |

---

## 14. Start Here (48 h)

### @operator (first)

1. `.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release`
2. Enter Simulation; settle 10 s; record **60 s** PERF lines
3. Append **§2026-05-28 release** to `debug_runs/perf_attribution_60s.md` (HW spec + p95 frame / raster_b / view_fire)
4. Refresh `debug_runs/agent_debug_index.json` if witnesses re-written

### @coder A (after or parallel with operator)

1. **P0-2:** ensure lib test writes `visual_witness` + `perf_attribution_60s` into stage5 JSON (`cargo test -p proc_A_dine01 --lib stage5`)
2. **P1-1:** compositor-committed → GPU `presentation_source` without env gate
3. **P2-1:** next containment PR — `runtime_witness/construction.rs`
4. Do **not** touch archived wave 6 construction parametric/R4 exec plans

---

## 15. Recommended default (one thing)

If the team can only do **one thing**: run **OPS-F01** (P0-1) — a **release 60 s clean visual** with documented p95 and **no `RASTER_*`**. Everything else (budget tuning, containment, F2) prioritizes against that measured baseline. Without it, perf work lacks a ship bar and queue rows stay ambiguous.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-28 | PLAN-FLEET-PHASE-NEXT-001 — post wave 6 returns |
