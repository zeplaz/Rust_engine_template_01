# Post–Stage 6 active todos

**Design plan:** [`post_stage6_design_plan.md`](post_stage6_design_plan.md)  
**Decisions:** [`post_stage6_design_decisions.md`](post_stage6_design_decisions.md)  
**Closed gates:** Stage 5/6 + Wave S write — see sign-off docs.

**Lib tests:** `632 passed` (2026-05-23) — `cargo test -p proc_A_dine01 --lib`

**Current phase:** **E — Product** (witness-driven). Phases A–D code lanes closed; operator refresh + Phase F ops run in parallel.

**Stage tracks:** [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) · planner [`stage_planner_workboard_v1.md`](stage_planner_workboard_v1.md) · designer [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) · coder [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md)

**Operator refresh:** run sim → `wave_s_hydrate_live.json`, `wave_p_live.json`, `wave_c_live.json`, `stage6_virtualization_live.json`; `cargo run -p proc_A_dine01 --release -- --test visual` for Stage 5 regression.

---

## Phase A — Wave S completion ✅ CODE

| ID | Task | Status | Proof |
|----|------|--------|-------|
| WS-A01 | Load `product_shell.ron` (flag `WAVE_S_AUTOLOAD_SHELL=1`) | [x] | `hydrate_wave_s_artifacts_from_bundle`, `try_autoload_wave_s_on_bundle_dir` |
| WS-A02 | Restore layout from save bundle button | [x] | `dock_shell.rs` + `WaveSShellRestorePending` |
| WS-A03 | Import Wave S presets in construction panel | [x] | `pending_construction_panel.rs` |
| WS-A04 | Live witness writer | [x] | `wave_s_live_proof.rs` → `wave_s_hydrate_live.json` |
| WS-A05 | BQ-133 RON-only decision | [x] | `wave_s_open.md` § BQ-133 |

---

## Phase B — Wave P ✅ CODE (visual refresh ops)

| ID | Task | Status | Proof |
|----|------|--------|-------|
| WP-B01 | `wave_p_live.json` writer | [x] | `wave_p_live_proof.rs` |
| WP-B02 | Readiness in live payload | [x] | `build_wave_p_live_proof_payload` + lib test |
| WP-B03 | Consumer audit | [x] | [`post_stage6_wave_p_audit.md`](post_stage6_wave_p_audit.md) |
| WP-B04 | Close `WAVE_P_OPEN_BACKLOG_ITEMS` | [x] | empty + live writer |
| WP-B05 | `--test visual` regression | [x] | **2026-05-24** — `stage5_full_app_live.json` streak 149 + `minimap_compositor_live.json` |

---

## Phase C — Infrastructure ◐ AUDIT + PARTIAL

| ID | Task | Status | Proof |
|----|------|--------|-------|
| IN-C01 | VM-09 audit | [x] | [`post_stage6_vm09_audit.md`](post_stage6_vm09_audit.md) |
| IN-C02 | PROJ-2 audit | [x] | same doc § PROJ-2 |
| IN-C03 | VM-08 overlay isolation | [x] | `ViewFireIsolationWitness` + `vm08` in `infrastructure_view_isolation_live.json` |
| IN-C04 | VM-10/11 audit run | [x] | `vm_10` / `vm_11` in live proof + witness green gates |
| IN-C05 | Per-view fire in projection graph | [x] | `view_fire_projection.rs` → `run_render_projection_graph` |
| IN-C06 | GPU tile authoritative | [x] | `OnEnter(Simulation)` enables instanced path; gizmo when `use_batched_mesh_overlay == false` |
| IN-C07 | Viewport event-bus gap doc | [x] | vm09 audit + `recovery_viewport.md` |

**Execution (PLAN-INFRA-SLICE2-001):** VM-09 [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) **CLOSED** · next [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](infra_slice3_wc_d04_ops_f01_plan_v1.md) **OPS-F01** → **WC-D04** (Coder B)

---

## Phase D — Wave C ✅ CODE (sim refresh ops)

| ID | Task | Status | Proof |
|----|------|--------|-------|
| WC-D01 | Backlog item closed (live witness) | [x] | `wave_c_live_proof.rs`, empty `WAVE_C_OPEN_BACKLOG_ITEMS` |
| WC-D02 | TileStorage in live JSON | [x] | `wave_c_live.json` `tile_storage_apply` |
| WC-D03 | Missing manifest error test | [x] | `load.rs` `hydrate_error_tests` |
| WC-D04 | Residency churn tune | [x] coder | **WC-D04-CODER-B** hysteresis + witness `wc_d04` · operator **OPS-F03** sim refresh |

**Execution (PLAN-INFRA-C-WC):** **WC-DEPTH-001** **done** (BQ-101) · **OPS-F03** stage6 JSON refresh

---

## Phase E — Product ◐ ACTIVE

| ID | Task | Status | Proof / notes |
|----|------|--------|---------------|
| CON-E01 | Phase 2 P6→P8 + R3 catalog | [x] | **P9 closed** — `p9_build` + `con_e01_p9_green` in `construction_stage_live.json` |
| CON-E02 | Operational green | [x] | `CONSTRUCTION-OP-*` Done; refresh `construction_stage_live.json` in sim |
| CON-E03 | Round 3 topology/visual | [x] | `CONSTRUCTION-R3-*` static Done — runtime reconcile via live JSON |
| IND-E01 | Supply chain E2E | [x] | **DONE** — `activation_green: true` + `production_green` ([`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md)) |
| IND-E02 | Concrete chain in play | [x] commit / [ ] default JSON | **Commit path** only: `ind_e02_green` via `simulation_writes_industrial_activation_live_json_ind_e02_in_play`. Default writer keeps `ind_e02_green: false` — see [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) |
| IND-E03 | Grid/substation stress | [x] | [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) — `ind_e03_green`; seed `RUST_ENGINE_IND_E03_SEED` |
| LOG-E01 | `log_rows` in FULL_APP | [x] | [`logistics_visual_lane_spec_v1.md`](logistics_visual_lane_spec_v1.md) — startup transport seed + `log_rows≥1` in visual run |
| UX-E01 | GPU minimap M1+M2+M3 FoW/EW | [x] | **UI-P3-M4-001** FoW+EW — `ui_p3_m4_green` in `minimap_compositor_live.json`; units/replay optional |
| UX-E02 | BQ-128 editor path | [x] design / [ ] apply | [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) (**PLAN-UX-BQ128-001**) · **BQ-128-APPLY-001** open |
| UX-E03 | Transmission stub note | [x] | [`ux_e03_transmission_shell_note_v1.md`](ux_e03_transmission_shell_note_v1.md) · stub [`transmission_media.rs`](../gui/hud/transmission_media.rs) |

---

## Phase F — Ops ◐

| ID | Task | Status | Proof |
|----|------|--------|-------|
| OPS-F01 | 60s perf capture | [ ] | **Operator** — `perf_attribution_60s.md` |
| OPS-F02 | Refresh stage5 FULL_APP JSON | [x] | **2026-05-24** — capture run |
| OPS-F03 | Refresh stage6 JSON | [ ] | **Operator** — sim |
| OPS-F04 | `cargo orchestrate` | [x] | **2026-05-24** — 0 issues |
| OPS-F05 | Compile hygiene | [x] | 0 rustc warnings (2026-05-23 batch) |

---

## Launch queue — Phase E (pick **one** primary per cycle)

| Priority | IDs | Owner | Deliverable |
|----------|-----|-------|-------------|
| 1 | **S7B-PREFLIGHT-001** | sim-steward | M1 preflight GO |
| 2 | **S7B-M1-001** | coder | [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md) |
| 4 | **OPS-F01**, **OPS-F03** | operator | `perf_attribution_60s.md` + optional sim stage6 refresh |
| — | ~~UX-E01 / UI-P3-M4~~ | — | **DONE** — minimap M1/M2/M3 FoW+EW |
| — | ~~WC-D04~~ | — | **DONE** — `wc_d04.green` |
| — | ~~IND-E01~~ | — | **DONE** |
