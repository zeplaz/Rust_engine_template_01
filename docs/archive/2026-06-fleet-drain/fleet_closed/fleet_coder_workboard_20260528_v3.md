# Fleet coder workboard — PHASE-NEXT cycle 2 `v3`

| Field | Value |
|:---|:---|
| **Queue ID** | **FLEET-CODER-WORKBOARD-20260528-V3** |
| **Date** | 2026-05-28 (post-session reconcile) |
| **Owner** | `@orchestrator` |
| **Parent** | [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) |
| **Exec slices** | [`plan_fleet_phase_next_exec_001_v1.md`](plan_fleet_phase_next_exec_001_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Status** | **ACTIVE** — both coders have parallel lanes |

**Rule:** Witness JSON wins. One primary lane per coder per PR. **Never** mix `tile_world_fallback.rs` perf edits with witness writer migration in the same PR.

---

## 1. Project status snapshot (2026-05-28)

### Green on disk / in lib tests

| Lane | Witness / test | State |
|:---|:---|:---|
| Stage 5 spine | `stage5_full_app_live.json` | `readiness.passes: true`; F2 extract rows > 0 |
| WSS substrate | `wss_substrate_live.json` | `green: true` |
| Construction | `construction_stage_live.json` | `operational_green: true` |
| Stage 7 play | `stage7_behavioral_live.json` | M3/M4 steward + play wired |
| Perf wave 7 (A) | `perf_vis_002_*` tests | P2B zoom/spike + P2D residency — pass |
| Containment scan | `check_live_proof_containment.ps1` | OK |
| Witness parity | `wss_witness_parity_001` | pass |
| UI shell 2B | `ui_oh_2b_001_live_witness_refresh` | pass — `phase2b_closed: true`, `floating_egui_shells_gated: true` |

**UI gate correction:** Real PLAY-01 gate is `ui_oh_2b_001.green: true` with **`product_egui_shell_in_simulation: false`** (no product egui in sim). Do **not** treat `true` as exit — spec and tests expect **false**.

### Partial / open

| Item | Notes |
|:---|:---|
| Wave 7 B containment tail | **DEV-CONTAIN-004…007** (fire, stage7, wss writers + shim retire) |
| UI shell tail | `ui_p3_001.closed` still `false` in shell JSON; compositor witness authoritative; P4/P5 partial |
| Logistics lib tests | ~13 failures — Stage 7 `OnEnter(Simulation)` hooks need `SimTick` / `MapViewInstances` in headless apps |
| OPS-F01 | 60s clean perf baseline table still empty (operator lane) |
| VT-5 flicker | Needs `--test visual` confirmation — lib `vt_ci_matrix` ≠ live VR-04 |
| Stage5 perf block on disk | Code writes `visual_witness` / `perf_attribution_60s`; **absent** from current `stage5_full_app_live.json` refresh |

### Landed this session (do not re-pick)

| ID | Owner | Evidence |
|:---|:---|:---|
| **DEV-CONTAIN-002-CONSTRUCTION** | B | `runtime_witness/construction.rs`; shim in `construction/live_proof.rs` |
| **DEV-CONTAIN-003-ECONOMY** | B | `runtime_witness/economy.rs`; industrial + logistics shims; 5 industrial lib tests pass |
| **VFX-VECTOR-SHAPES-001** | A | `bevy_vector_shapes` 0.12 + wire draw; witness `drawn_shapes > 0` |
| **PERF-VIS-002-P2B / P2D** | A | lib-green; P2D p95 acceptance = OPS-F01 only |

---

## 2. North star (this cycle)

1. **Ship bar:** operator fills `perf_attribution_60s.md`; coder refreshes stage5 perf witness on disk.
2. **Hygiene bar:** coder B finishes containment 004→007; enable `-HardFail` when shims retired.
3. **Harness bar:** either coder fixes Stage 7 headless guards → logistics lib suite green.

---

## 3. Coder B — containment + UI tail (primary)

**Start today:** **DEV-CONTAIN-004-FIRE-WAVEP**

| P | ID | Files (pattern = economy slice) | Tests | Witness exit |
|:---:|:---|:---|:---|:---|
| 1 | **DEV-CONTAIN-004-FIRE-WAVEP** | `runtime_witness/fire.rs`, `runtime_witness/wave_p.rs`; shims `systems/fire/live_proof.rs`, `gui/editor/world_preview/wave_p_live_proof.rs` | `fire`, wave_p lib | `fire_ecology_live.json`, `wave_p_live.json` keys unchanged |
| 2 | **DEV-CONTAIN-005-STAGE7** | `runtime_witness/stage7_behavioral.rs`, `runtime_witness/stage7_play.rs`; shims in `dev/stage7_*_live_proof.rs` | `stage7_behavioral`, `stage7_play` | Preserve `s7b_m3_green`, `s7b_steward_green`, `s7b_m4_play_green` |
| 3 | **DEV-CONTAIN-006-WSS** | `runtime_witness/wss_substrate.rs`; substrate collectors stay | `wss_substrate` | `wss_substrate_live.json` single writer path |
| 4 | **DEV-CONTAIN-007-SHIM-RETIRE** | `exceptions_manifest.json`, remove retired shims, `tools/orchestrator/ci/run.ps1` | `check_live_proof_containment.ps1 -HardFail` | all lanes parity; zero out-of-root writers |
| 5 | **UI-P3-SHELL-ROLLUP-001** | `simulation_shell_phase2.rs`, `minimap_compositor/live_proof.rs` | `ui_oh` steward proof | Shell JSON documents compositor authority OR `ui_p3_001.closed: true` via rollup |
| 6 | **UI-OH-P4-001** | icon atlas + petroleum panel (plan signed) | `ui_oh_p4` proof | `phase4.icon_atlas_loaded` + petroleum tab witness |
| 7 | **UI-OH-P5-001** | pause menu (`plan_ui_p5_pause_menu_index_v1.md`) | hud pause proof | pause overlay witness in sim |

**Optional (P8+):** `INFRA-VM09-V2-001`, `CONSTRUCTION-R4-PRODUCT-001` — only if containment 007 lands early.

### B — migration recipe (copy from economy)

```text
1. Add runtime_witness/<lane>.rs with commit_* + write_* systems + LiveProofState
2. Domain live_proof.rs: build_*_payload only; re-export writer from runtime_witness
3. exceptions_manifest.json: add migrated writer path
4. cargo test <lane>; check_live_proof_containment.ps1
5. Parity: refresh JSON; diff keys vs pre-migrate
```

---

## 4. Coder A — ship-quality + harness (primary)

**Start today:** **PERF-WITNESS-DISK-REFRESH-001** (unblocks PERF acceptance truth)

| P | ID | Focus | Exit |
|:---:|:---|:---|:---|
| 1 | **PERF-WITNESS-DISK-REFRESH-001** | Wire `visual_readiness_witness` + `perf_attribution_witness` into live stage5 harness refresh | `stage5_full_app_live.json` contains `visual_witness` + `perf_attribution_60s` blocks |
| 2 | **PERF-VIS-P1B-GPU-DEFAULT-001** | Minimap compositor GPU default without `RASTER_*` env | `presentation_source: SharedRenderTargetImage` without env in sim default path |
| 3 | **LOG-S7-HEADLESS-GUARDS-001** | Stage 7 `OnEnter(Simulation)` systems use `Option<Res<SimTick>>`, `Option<Res<MapViewInstances>>` | `cargo test -p proc_A_dine01 --lib logistics` — 0 failures |
| 4 | **STAGE5-VT-FLICKER-VISUAL-001** | Operator/coder visual run + log capture | VR-04 absent in 60s `--test visual` log; document in `visual_run_blockers.md` |
| 5 | **WSS-DEFORMATION-SLAB-L2-001** | L2 deformation tick hook (L1 scaffold done) | `wss_substrate_live.json` deformation keys exercised |
| 6 | **DEV-CONTAIN-HARDFAIL-CI-001** | After B slice 007 — wire `-HardFail` in CI if not already | `tools/orchestrator/ci/run.ps1` exit 0 with HardFail |

**Deferred until OPS-F01 baseline exists:** PERF p95 sign-off (`view_fire` < 8 ms) — code done; measurement is operator.

---

## 5. Parallel picks (same week — disjoint files)

| Week | Coder A | Coder B | Conflict? |
|:---|:---|:---|:---:|
| W1 D1–D2 | PERF-WITNESS-DISK-REFRESH-001 | DEV-CONTAIN-004-FIRE-WAVEP | No |
| W1 D3–D4 | LOG-S7-HEADLESS-GUARDS-001 | DEV-CONTAIN-005-STAGE7 | No |
| W1 D5 | PERF-VIS-P1B-GPU-DEFAULT-001 | DEV-CONTAIN-006-WSS | No |
| W2 D1–D2 | STAGE5-VT-FLICKER-VISUAL-001 | DEV-CONTAIN-007-SHIM-RETIRE | No |
| W2 D3+ | WSS-DEFORMATION-SLAB-L2-001 | UI-P3-SHELL-ROLLUP-001 | No |
| W2 tail | DEV-CONTAIN-HARDFAIL-CI-001 | UI-OH-P4-001 → P5 | No |

**Either coder may take LOG-S7-HEADLESS-GUARDS-001** — prefer **A** if B is mid-containment PR; prefer **B** if A is blocked on visual run.

---

## 6. Operator (unblocks acceptance)

| ID | Task | Exit |
|:---|:---|:---|
| **OPS-F01** | `run_visual_test_clean.ps1 -Release`, 60s | §2026-05-28 p95 table in `perf_attribution_60s.md` |
| **OPS-VT5-001** | Same run — capture VT-5 / VR-04 log | Confirms or clears `STAGE5-VT-FLICKER-VISUAL-001` |
| **OPS-F03** | Optional stage6 sim refresh | `stage6_virtualization_live.json` timestamp |

---

## 7. Regression (every slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7
cargo test -p proc_A_dine01 --lib perf_vis_002 chunk_grid_tests
cargo test -p proc_A_dine01 --lib logistics
.\tools\orchestrator\scripts\check_live_proof_containment.ps1
```

After slice 007:

```powershell
.\tools\orchestrator\scripts\check_live_proof_containment.ps1 -HardFail
```

---

## 8. Do not re-open

Wave 6 product closure, parametric exec, WSS PR-3/4 exec, Hanabi default binary wire, archived wave 6 boards — see [`plan_fleet_phase_next_001_v1.md`](plan_fleet_phase_next_001_v1.md) §3.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v3.0.0 | 2026-05-28 | Post-session reconcile; B containment 002/003 done; dual-coder parallel board |
