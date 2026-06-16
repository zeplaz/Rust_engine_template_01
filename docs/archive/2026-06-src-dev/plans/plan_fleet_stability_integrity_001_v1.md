# PHASE-STABLE-2026-06 — Stability, integrity, and playability plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-FLEET-STABILITY-INTEGRITY-001** |
| **Working title** | **PHASE-STABLE-2026-06** |
| **Version** | `1.2.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` |
| **Status** | **SIGNED — ACTIVE (P1 sweep)** |
| **Exec slices (open)** | [`plan_fleet_stability_integrity_exec_002_v1.md`](plan_fleet_stability_integrity_exec_002_v1.md) |
| **Exec slices (closed)** | [`plan_fleet_stability_integrity_exec_001_v1.md`](plan_fleet_stability_integrity_exec_001_v1.md) |
| **Jank sweep** | [`production_jank_sweep_20260602_v1.md`](production_jank_sweep_20260602_v1.md) |
| **Audit** | [`planner_status_audit_v17.md`](planner_status_audit_v17.md) |
| **Dispatch** | [`fleet_stability_coder_dispatch_v1.md`](fleet_stability_coder_dispatch_v1.md) |
| **Prior phase** | PHASE-NEXT cycle 2 — **coders drained** ([`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) v4.14.0, `wave7_open_slices: 0`) |
| **Machine queue** | Repopulate after sign-off → `next_phase` block in `coder_active_queue.json` |
| **Audit baseline** | [`planner_status_audit_v16.md`](planner_status_audit_v16.md) (witness); this plan adds **playability** truth |

---

## 1. Executive summary (honest)

**What we achieved:** The repo can prove a **converged simulation spine** in CI and lib tests: Stage 5 readiness, WSS substrate rollup, construction operational, Stage 7 behavioral/play witnesses, witness containment for most lanes, GPU minimap default without `RASTER_*`, and logistics lib suite green after headless guards.

**What we did not achieve:** A **player-trustworthy game loop** where the same code paths that pass witnesses are the paths a human uses for 30+ minutes without jank, flicker, dual-authority drift, or “green JSON that lies.”

**North star for this phase:**  
*One authoritative sim → one render extraction → one HUD session → measurable stability.*  
Witness JSON remains **evidence**, not the product.

---

## 2. Current situation (2026-05-28)

### 2.1 Fleet posture

| Role | Posture |
|:---|:---|
| **@coder A / B** | **Drained** on wave 7 / PHASE-NEXT cycle 2 — `active: []` |
| **@planner** | Must **replan** — prior PHASE-NEXT assumed open PERF/CONTAIN tails; most are **done** |
| **@designer** | Drained on perf-degrade UX; call for **playability** readability passes only |
| **Operator** | Still owns **live** 60s visual + VR-04 log ([`perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) is a template, not a measured baseline) |

### 2.2 Witness vs playability gap

| Signal | Witness / lib | Human / visual session |
|:---|:---|:---|
| Stage 5 FULL_APP | `readiness.passes: true`, `full_visual_confirm: true` | VR-04 VT-5 still **intermittent** under `--test visual` ([`visual_run_blockers.md`](visual_run_blockers.md)) |
| Logistics | 48/48 lib tests | Tests use **shortcuts** (`patch_s7p_logistics_throughput_witness_for_play_proof`, play seeds) |
| LOG-E01 | Fixture + visual keys split | `full_visual_confirm` is **visual_run only**; fixture uses `log_e01_fixture_green` |
| Fire F2 | `fire_instance_buffer_rows > 0` on disk | Overlay **bootstrap** still exists (`fire_degraded_overlay_bootstrap`) |
| WSS | `green: true` | **Dual-write shim** ECS→slab still authoritative for weather/fire ([`substrate/shim.rs`](../substrate/shim.rs)) |
| UI shell | P3/P4/P5 greens in refreshed JSON | Compositor vs shell frame can **diverge** until sim refresh; editor vs sim chrome split remains cognitively heavy |
| Containment | 12 writers in `runtime_witness/`; **4 shims** remain | Domain trees still carry `witness_collectors`, `*_witness_green()` fns, test refresh bundles |

**Rule for this phase:** Every slice must name **(a)** user-visible behavior, **(b)** hack removed or bounded, **(c)** witness that cannot be satisfied by patch/shortcut alone.

### 2.3 Closed — do not re-open without regression

Same as [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) §3, plus cycle-2 coder landings:

- DEV-CONTAIN 002–007 + HardFail CI  
- PERF-VIS P1BC, P2A/B/D, P3/4, P1B GPU default, witness disk refresh  
- LOG-S7 headless guards, UI 2B/P3/P4/P5, WSS L1+L2 deformation tick  
- Wave 6 product: S7B M3/M4, BQ-128, parametric construction, R4 prep  

---

## 3. Problem taxonomy — “janky injected code”

Use this table when triaging PRs. **Goal:** shrink each category every sprint; no new rows without a `ScaffoldContract` or time-boxed witness.

| Class | Symptom | Examples in tree | Remediation pattern |
|:---|:---|:---|:---|
| **W1 — Witness theater** | Green JSON without runtime truth | `qualified_close`, `patch_*_witness_for_play_proof`, `apply_*_witness_shortcut`, lib-only `refresh_*_live_witness` | Split **ProofGrade**: `LibFixture` / `HeadlessSim` / `VisualCapture`; forbid shortcut in `VisualCapture` |
| **W2 — Dual authority** | Two writers, sync bridges, drift | `MapCameraDesired` mirror, `sync_view_manager_bridge`, substrate `DualWriteShimState`, ECS+slab | Pick **one writer** per concern; bridges become read-only or deleted |
| **W3 — Env / throttle hacks** | Behavior changes via env vars | Historical `RASTER_*`, `MINIMAP_GPU_COMPOSITOR`, `STAGE5_VERBOSE`, witness force flags | **Release profile** table in code; env only in `dev`/`ci` profiles |
| **W4 — Bootstrap injection** | Test harness seeds world state | `test_harness` menu/bootstrap, `seed_ind_e02_*`, `seed_stage7_*`, Portland chain commits in proof apps | **`PlayScenario` resource** — one scenario ID used by tests *and* default New Game |
| **W5 — Transitional scaffolds** | `#[allow(dead_code)]`, empty plugins, duplicate LOD | Stage 5 scaffolds, VM shims, `in_game_ui` feature gating | `ScaffoldContract` + expiry date; orchestrator `CONTINUE` → must wire or delete |
| **W6 — Schedule / ordering debt** | `.before`/`.after` chains, 16-param splits | `tile_world_fallback` policy split, fire extract cadence, logistics witness chain | Document **CoreSystemSet** ownership; reduce ad-hoc chains per domain |

---

## 4. Strategic goals (reordered)

| Priority | Goal | Success looks like |
|:---:|:---|:---|
| **P0** | **Playable session** | Enter Simulation → build → logistics tick → fire/visual read **without** debug menu bootstrap |
| **P0** | **Honest proof** | No new `patch_*_witness*`; visual capture lane cannot use shortcuts |
| **P1** | **Authority cleanup** | Viewport + fire extract + minimap: one commit path documented and enforced |
| **P1** | **Finish containment Slice D** | 4 remaining shims deleted; collectors only in domain |
| **P2** | **WSS single truth** | Slab authoritative; ECS components read-only mirrors or retired per [`ecs_retire`](../substrate/ecs_retire.rs) plan |
| **P2** | **Feature depth** | Construction→activation→logistics in **one** default scenario (not separate proof apps) |
| **P3** | **Infra / multiview** | VM-09 v2, isolation audits — **after** P0–P1, not instead of |

---

## 5. Workstreams and phases

### Horizon A — “Game loop truth” (weeks 1–2)

| ID | Slice | Owner | Do | Exit |
|:---|:---|:---|:---|:---|
| **PLAY-TRUTH-001** | Default play scenario | @coder + @designer | Single `PlayScenarioId::DefaultIndustrial` wires: terrain min, Portland chain **via construction UI**, one logistics route, sim HUD defaults | Manual script: 10 min sim without `test_harness` bootstrap |
| **PLAY-TRUTH-002** | Proof grade separation | @coder | `ProofGrade` enum on witness writers; remove shortcut calls from visual harness path | Grep gate: no `patch_*_witness` in `stage5_full_app_harness` visual lane |
| **PLAY-TRUTH-003** | LOG-E01 integrity | @coder | `full_visual_confirm` only from `--test visual` capture; lib fixture uses separate `log_e01_fixture_green` key | `stage5_full_app_live.json` distinguishes lanes |
| **OPS-PLAY-001** | Operator playbook | operator | Document “clean play session” checklist in [`perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) after 60s run | Measured p95 table filled |

### Horizon B — “De-hack pass” (weeks 2–4)

| ID | Slice | Owner | Do | Exit |
|:---|:---|:---|:---|:---|
| **DEHACK-VIEW-001** | Viewport single commit | @coder | Audit [`recovery_viewport.md`](recovery_viewport.md) table; delete or gate `mirror_map_camera_desired_*` write paths in sim | `viewport_drift.json` steady-state flip count bounded in 60s visual |
| **DEHACK-FIRE-001** | Fire extract truth | @coder | Residency + projection graph only; demote overlay bootstrap to explicit `DegradedMode` UI flag | `fire_degraded_overlay_bootstrap` false in default scenario |
| **DEHACK-LOG-001** | Logistics witness | @coder | Replace `apply_s7p_logistics_throughput_witness_shortcut` with scenario-driven routes | `routes_open > 0` from sim graph only in `VisualCapture` grade |
| **DEHACK-WSS-001** | Substrate shim | @coder | Plan cutover: slab writes authoritative; shim read-only compare mode | `dual_write_drift_max` under epsilon in `wss_substrate_live.json` |
| **CONTAIN-D-001** | Retire 4 shims | @coder | Delete `wave_c_live_proof.rs`, `wave_s_live_proof.rs`, `stage6_live_proof.rs`, `view_runtime/live_proof.rs` re-exports; plugins import `runtime_witness` only | `exceptions_manifest.json` `allowed_shim_paths: []` |

### Horizon C — “Stability measurement” (weeks 3–5, parallel)

| ID | Slice | Owner | Do | Exit |
|:---|:---|:---|:---|:---|
| **STAB-PERF-001** | Release perf budget | @coder | Document caps in [`plan_visual_perf_production_v1.md`](plan_visual_perf_production_v1.md); wire spike feedback to **player-visible** degrade only | OPS-F01 p95: frame &lt; 33 ms, raster_b &lt; 8 ms (release build) |
| **STAB-VT-001** | VT-5 live gate | operator + @coder | OPS-VT5 log; promote VR-04 to triage gate only if reproduced | [`visual_run_blockers.md`](visual_run_blockers.md) VR-04 **closed** or **won't fix** with rationale |
| **STAB-CI-001** | `-D warnings` in CI | @coder | `cargo rustc -- -D warnings` on main path (scoped crates first) | CI green; [`compile_warnings_registry.md`](compile_warnings_registry.md) updated |

### Horizon D — “Feature integration” (weeks 4–8, after B starts)

| ID | Slice | Owner | Do | Exit |
|:---|:---|:---|:---|:---|
| **FEAT-CONSTRUCTION-001** | Round 4 product (scoped) | @coder | Only items named in [`plan_construction_r4_product_open_001_v1.md`](plan_construction_r4_product_open_001_v1.md) | One vertical slice playable (e.g. corridor phase visual + commit) |
| **FEAT-WSS-002** | Hydrology player read | @coder + @designer | Player-visible moisture/flood hint from slab (no new parallel extract) | Designer sign-off + substrate witness keys |
| **FEAT-VFX-001** | Hanabi policy | @coder | Remains **feature flag**; embellishment never required for readiness | Default build: no Hanabi dep required to run sim |

---

## 6. Authority map (unchanged core, enforced)

| Layer | Owns | Stop |
|:---|:---|:---|
| **Simulation** | ECS systems in domain plugins, `SimControlState`, construction commit funnel | Witness JSON writes |
| **Substrate L1** | `WorldSubstrateRegistry`, slabs, hydrology/atmos ticks | GPU particles |
| **Representation** | `RepresentationResult`, `WorldLodMap`, projection graph | File I/O |
| **Render extract** | `FireVisualFramesByView`, minimap compositor, tile fallback **policy** | Sim mutation |
| **Viewport** | `commit_authority_from_semantic` → `publish_simulation_map_viewport` | `MapCameraDesired` writes from minimap/preview |
| **Witness** | `src/dev/runtime_witness/*` only | New `*live_proof*.rs` in domain trees |
| **HUD** | Bevy UI sim shell; egui **editor-only** | Product egui in `BaseState::Simulation` |

---

## 7. Gate model (new)

Do **not** fold these into Stage 5 closure.

| Gate | Type | Criteria |
|:---|:---|:---|
| **G-PLAY-01** | Playability | 10 min default scenario without harness bootstrap; no panic; pause menu works |
| **G-PROOF-01** | Proof honesty | Visual capture witnesses have zero `shortcut`/`patch`/`qualified_close` fields |
| **G-STAB-01** | Stability | OPS 60s release table + VR-04 disposition documented |
| **G-CONTAIN-01** | Hygiene | `check_live_proof_containment.ps1 -HardFail` with **empty** shim manifest |
| **G-SHIP-01** | Regression | `cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 logistics` + visual smoke optional |

Stage 5 operational gate stays **CLOSED** — fixes go through triage unless they break `readiness.passes`.

---

## 8. What we stop doing

1. **Chasing witness keys** without a play scenario that exercises the same code.  
2. **New `*live_proof*.rs`** outside `runtime_witness/` (HardFail CI already active).  
3. **Parallel perf env vars** in release — use `VisualBudgetSettings` + compositor policy only.  
4. **Large feature drops** (R4 product board, WSS depth, VM multiview) before **PLAY-TRUTH-001** lands.  
5. **Merging** perf edits + witness migration + viewport refactors in one PR.  
6. Treating **lib fixture green** as operator sign-off.

---

## 9. Suggested queue seed (after sign-off)

Populate `coder_active_queue.json` `next_phase` → `active[]`:

**Coder A (primary: play truth + de-hack viewport/fire)**  
1. PLAY-TRUTH-002  
2. DEHACK-VIEW-001  
3. DEHACK-FIRE-001  
4. STAB-VT-001 (with operator)  

**Coder B (primary: play scenario + containment + logistics)**  
1. PLAY-TRUTH-001  
2. DEHACK-LOG-001  
3. CONTAIN-D-001  
4. DEHACK-WSS-001 (plan + compare mode only first PR)  

**Operator**  
1. OPS-PLAY-001  
2. OPS-VT5-001  

**Designer (on-call)**  
1. PLAY-TRUTH-001 UX readability  
2. FEAT-WSS-002 hydrology read language  

---

## 10. Planner prompt (copy-paste)

```text
@planner Sign off PLAN-FLEET-STABILITY-INTEGRITY-001 (plan_fleet_stability_integrity_001_v1.md).

Context: Both coders drained on PHASE-NEXT cycle 2. Witness spine is green; playability and hack debt are not.

Deliver:
1. planner_status_audit_v17.md — witness + playability columns
2. plan_fleet_stability_integrity_exec_001_v1.md — 2-week sprint slices with file paths
3. Repopulate coder_active_queue.json next_phase.active[] from §9
4. Mark plan_fleet_phase_next_001_v1.md SUPERSEDED for open work only

Constraints:
- P0 = PLAY-TRUTH + DEHACK before new features
- No patch_*_witness in visual capture path
- Stage 5 gate stays closed; G-PLAY-01 is the new player bar
- One PR per workstream row; no tile_world_fallback + witness migration mix
```

---

## 11. Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-28 | **SIGNED** — audit v17, exec doc, queue repopulated from §9 |
| v1.0.0 | 2026-05-28 | Initial draft: post-dual-coder-return; playability + de-hack focus |
