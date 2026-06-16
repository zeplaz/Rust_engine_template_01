# PLANNER-BACKLOG-SWEEP-001 — stubs · unfinished plans · planner todos `v1`

```text
⟦SYMLANG⟧⟐v1  ◈BACKLOG
⟨ID⟩ PLANNER-BACKLOG-SWEEP-001
Date: 2026-06-13
Sweep: src/dev/*.md · docs/archive/**/plans/*.md · tools/orchestrator/queues/*.json
Rule: doc/schema deliverables only — coder impl rows referenced as **blocks** not duplicated
```

| Metric | Count |
|:---|:---:|
| **@planner todos** | 38 |
| **@planner-mcp todos** | 34 |
| **Defer / frozen (policy)** | 12 |
| **P0 combined (do first)** | 8 |

**Use:** `@planner` / `@planner-mcp` pick from numbered rows · mark done via queue `BLANG:Q✓` + plan sign-off row.

---

## P0 — do first (unblocks coders)

| # | Owner | ⟨ID⟩ | Deliverable | Status | Blocks |
|:---:|:---|:---|:---|:---|:---|
| P0-1 | **@planner** | PLAN-MAP-ZOOM-SMOOTH-001 | Sign [`plan_map_zoom_smooth_exec_001_v1.md`](plan_map_zoom_smooth_exec_001_v1.md) (Option A vs B) | **READY — sign pending** | TRIAGE-MAP-ZOOM-SMOOTH-001 |
| P0-2 | **@planner-mcp** | MCP-P2-SIM-VALIDATORS-PLAN-001 | Sign [`plan_mcp_sim_product_validators_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_mcp_sim_product_validators_v1.md) | **DONE — SIGNED 2026-06-13** | MCP-P2-QUEUE/VALID ready · OPS-BRIEF after QUEUE |
| P0-3 | **@planner-mcp** | MCP-P2-KIT002-PLAN | Write **missing** [`mcp_kit_production_002_unfreeze_v1.md`](../docs/archive/2026-06-src-dev/plans/mcp_kit_production_002_unfreeze_v1.md) | **ready — file absent** | `kit_production_002+` frozen lane |
| P0-4 | **@planner** | PLAN-QUEUE-SYNC-BUILD-READ | Add BUILD-READ-* rows to [`post_drain_phase4_queue.json`](../tools/orchestrator/queues/post_drain_phase4_queue.json) + `coder_active_queue.json` | **gap** | Orchestrator picks |
| P0-5 | **@planner** | PLAN-QUEUE-SYNC-SIM-EFFECT | Confirm L_SIM rows current in phase4 queue (SIM-EFFECT-* done/ready) | **partial** | Sim spine picks |
| P0-6 | **@planner-mcp** | ARCH-002 | Formal **Variant Graph** JSON schema (`variant_graph_v1.schema.json`) | **pending** | ATLAS compile · variant bakes |
| P0-7 | **@planner** | BUILD-READ-QUEUE-HOOK | Add PLAN-BUILD-READABILITY-001 to [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) deliverables table | **stub** | Index truth |
| P0-8 | **@planner-mcp** | BUILD-READ-vNext-STUB | `arch_build_grammar_vnext_defer_v1.md` — deferred β keys + ProgramGraph scope (1-page) | **not written** | Prevents scope creep |

---

## @planner — thin exec · sign-offs · policy (38)

### A — Sign / close (exec docs exist)

| # | ⟨ID⟩ | Doc | Action |
|:---:|:---|:---|:---|
| PL-01 | PLAN-MAP-ZOOM-SMOOTH-001 | [`plan_map_zoom_smooth_exec_001_v1.md`](plan_map_zoom_smooth_exec_001_v1.md) | **Sign** — only open P0 planner gate |
| PL-02 | PLAN-AUDIT-020 | `planner_status_audit_v20.md` (not on disk) | **Defer** until G-PLAY-01 operator EXECUTED |
| PL-03 | G-PLAY-01 | [`plan_g_play_close_001_checklist_v1.md`](plan_g_play_close_001_checklist_v1.md) | Operator row open — planner maintain |
| PL-04 | PLAN-CONSTRUCTION-R4-001 | [`plan_construction_r4_exec_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_construction_r4_exec_001_v1.md) | **Product board** narrative tail — `product_board_open` witness semantics |
| PL-05 | PLAN-REPLAY-RING-001 | [`plan_replay_ring_exec_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_replay_ring_exec_001_v1.md) | Finalized — confirm coder B pick or defer |
| PL-06 | PLAN-M3-DEPTH-001 | [`plan_m3_depth_exec_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_m3_depth_exec_001_v1.md) | Finalized — minimap optional depth gate |
| PL-07 | PLAN-IND-E02-PLAY | [`plan_ind_e02_play_exec_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_ind_e02_play_exec_001_v1.md) | Reconcile default JSON vs commit-only green |
| PL-08 | PLAN-WSS-HYBRID-RETIRE | [`plan_wss_hybrid_retire_pr4_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_wss_hybrid_retire_pr4_001_v1.md) | Closure witness criteria doc refresh |

### B — Active programs (maintain / phase docs)

| # | ⟨ID⟩ | Doc | Gap / stub |
|:---:|:---|:---|:---|
| PL-09 | PLAN-BUILD-READABILITY-001 | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) | **ACTIVE** — v0 signed; track designer/coder todo board |
| PL-10 | PLAN-SIM-EFFECT-SPINE-001 | [`plan_sim_effect_spine_exec_001_v1.md`](plan_sim_effect_spine_exec_001_v1.md) | **SIGNED** — P5 FACTION-REACT thin slice if reopened |
| PL-11 | PLAN-PRODUCT-POLISH-001 | [`plan_product_polish_exec_001_v1.md`](plan_product_polish_exec_001_v1.md) | **SIGNED** — MAP-ZOOM child still open |
| PL-12 | PLAN-THREE-TRACK | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) | Track B **PAUSED** — maintain unpause criteria |
| PL-13 | PLAN-PROC-TILE-PROD-001 | [`plan_procedural_building_tiles_production_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_procedural_building_tiles_production_v1.md) | PT-1 variant matrix coverage gap (7×4 vs 5 YAML) |
| PL-14 | PLAN-INFRA-WORLD-LAYERS | [`plan_infrastructure_world_layers_exec_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_infrastructure_world_layers_exec_001_v1.md) | **INFRA-E0** profile RON catalog incomplete |
| PL-15 | POST-STAGE6 Phase E/F | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) | IND-E02 default JSON · UX-E02 BQ-128 apply · OPS-F01/F03 operator |
| PL-16 | PLAN-OPS-INTELLIGENCE-001 | [`plan_agent_operations_intelligence_v1.md`](plan_agent_operations_intelligence_v1.md) | **PLANNED** — Phase 1–3 JSON telemetry slices not started |
| PL-17 | MASTER-CHAIN-4D | [`master_chain_board_4d_v1.md`](master_chain_board_4d_v1.md) | Manual sync with queues after each drain |
| PL-18 | AGENT-LANG-001 | [`agent_lang_v1.md`](agent_lang_v1.md) | **ACTIVE** — ritual maintenance |

### C — Infrastructure / transport matrix (doc debt)

| # | ⟨ID⟩ | Doc | Gap |
|:---:|:---|:---|:---|
| PL-19 | TRANSPORT-R1-R7 | [`road_rail_migration_matrix_v1.md`](../docs/archive/2026-06-prompts-guides/matrix/matrix/transport/road_rail_migration_matrix_v1.md) | R1–R7 rows **pending** |
| PL-20 | TRANSPORT-R8 | same | Hybrid wave S body · M5 slice ownership **open** |
| PL-21 | TRANSPORT-R9 | same | Spline tool **halt** until R8 ≥ Partial |
| PL-22 | POST-STAGE6-INFRA-C | [`post_stage6_infra_wave_c_plan_v1.md`](../docs/archive/2026-06-src-dev/plans/post_stage6_infra_wave_c_plan_v1.md) | OPS-F01 perf capture template |
| PL-23 | INFRA-WC-D04 | [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](../docs/archive/2026-06-src-dev/plans/infra_slice3_wc_d04_ops_f01_plan_v1.md) | OPS-F01/F03 operator doc refresh |
| PL-24 | VISUAL-RUN-BLOCKERS | [`visual_run_blockers.md`](visual_run_blockers.md) | VM-06…11 triage — not closed |

### D — UI / experience runbooks

| # | ⟨ID⟩ | Doc | Gap |
|:---:|:---|:---|:---|
| PL-25 | EXP-VA1-VA6 | `docs/archive/2026-06-prompts-guides/matrix/matrix/experience/runbook/` | Step packs v1–v6 **pending** |
| PL-26 | UI-OH-P4-TAILS | [`ui_oh_p4_001_plan_v1.md`](../docs/archive/2026-06-src-dev/plans/ui_oh_p4_001_plan_v1.md) | Numbered tail tasks **OPEN** |
| PL-27 | UI-OH-P5-TAILS | [`ui_oh_p5_001_plan_v1.md`](../docs/archive/2026-06-src-dev/plans/ui_oh_p5_001_plan_v1.md) | Numbered tail tasks **OPEN** |
| PL-28 | BQ-128-EXT | [`bq128_editor_path_plan_v1.md`](../docs/archive/2026-06-src-dev/plans/bq128_editor_path_plan_v1.md) | **BQ-128-APPLY-001** apply slice open |
| PL-29 | DESIGN-MAP-ZOOM | [`design_map_zoom_read_v1.md`](design_map_zoom_read_v1.md) | Charter done — ties to PL-01 sign |
| PL-30 | DESIGN-EVENT-LOG | [`design_event_log_ui_v1.md`](design_event_log_ui_v1.md) | P3 stub — SimEffect P6 narrative defer |

### E — Sim / weather / fire (policy docs)

| # | ⟨ID⟩ | Doc | Gap |
|:---:|:---|:---|:---|
| PL-31 | PLAN-WEATHER-WITNESS-002 | [`plan_weather_witness_002_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_weather_witness_002_v1.md) | `validate-report weather` profile not in validation-first registry |
| PL-32 | WEATHER-PARALLEL | [`plan_weather_parallel_lane_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_weather_parallel_lane_v1.md) | `weather_sim_live.json` writer spec vs disk |
| PL-33 | FIRE-ECOLOGY-F1 | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) | F2+ board — not Stage 5 gate |
| PL-34 | WSS-DESIGN-GATE | [`wss_design_gate_001_v1.md`](../docs/archive/2026-06-src-dev/plans/wss_design_gate_001_v1.md) | G3–G4 steward sign-off row |

### F — Queue hygiene · defer registry

| # | ⟨ID⟩ | Action |
|:---:|:---|:---|
| PL-35 | DEFER-REGISTRY | Maintain [`defer_registry.json`](../tools/orchestrator/queues/defer_registry.json) vs [`plan_defer_registry_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_defer_registry_v1.md) |
| PL-36 | PLANNER-ACTIVE-Q | Sync [`planner_active_queue.json`](../tools/orchestrator/queues/planner_active_queue.json) closed rows (PRODUCT-POLISH done) |
| PL-37 | HANDOFF-SYNC | [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md) — phase4 active lanes vs this backlog |
| PL-38 | DEV-PLAN-INDEX | Trim [`development_plan_index.md`](development_plan_index.md) — archive stale ACTIVE links |

---

## @planner-mcp — schemas · MCP specs · grammar (34)

### A — P0 gates (blocks coder-mcp)

| # | ⟨ID⟩ | Deliverable | Status | Blocks |
|:---:|:---|:---|:---|:---|
| PM-01 | MCP-P2-SIM-VALIDATORS-PLAN-001 | Sign [`plan_mcp_sim_product_validators_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_mcp_sim_product_validators_v1.md) | **DONE** | Phase 1+2 coder-mcp ready |
| PM-02 | MCP-P2-KIT002-PLAN | [`mcp_kit_production_002_unfreeze_v1.md`](../docs/archive/2026-06-src-dev/plans/mcp_kit_production_002_unfreeze_v1.md) | **MISSING** | kit002+ frozen |
| PM-03 | ARCH-002 | `tools/mcp/schemas/variant_graph_v1.schema.json` | **pending** | Variant-aware bakes |
| PM-04 | ATLAS-001 | Atlas lookup validation spec (state×facing×frame) | **partial** | Production atlas G4 |
| PM-05 | BUILD-001 | Build dependency graph witness **doc** (BUILDING-TILE-SPINE) | **pending** | WRK→ATL clarity |

### B — BUILD-READ / grammar v0 → vNext

| # | ⟨ID⟩ | Deliverable | Status |
|:---:|:---|:---|:---|
| PM-06 | BUILD-READ-GRAMMAR-v0-001 | [`arch_build_grammar_v0_baseline_v1.md`](arch_build_grammar_v0_baseline_v1.md) | **🟢 SIGNED** |
| PM-07 | BUILD-READ-vNext-DEFER | `arch_build_grammar_vnext_defer_v1.md` — ProgramGraph · operators · extended β | **not written** |
| PM-08 | BUILD-READ-SHAPE-001 | FootprintMatrix spec for Industrial Rail Warehouse (designer-mcp) | **🟢 PASS** — [`design_shape_rail_warehouse_pilot_v1.md`](design_shape_rail_warehouse_pilot_v1.md) + RON on disk |
| PM-09 | BUILD-READ-β-EXTENDED | Schema appendix: βorn, βdef, βctl, βentropy, βinertia, βdepth (defer only) | **stub in v0 doc** |
| PM-10 | BUILD-READ-TOPOLOGY | Topology class doc LINEAR/RADIAL/NETWORK (classify only) | **not written** |

### C — BUILDING-TILE-SPINE / grammar evolution

| # | ⟨ID⟩ | Doc | Gap |
|:---:|:---|:---|:---|
| PM-11 | BUILDING-TILE-SPINE | [`plan_building_tile_spine_001_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_building_tile_spine_001_v1.md) | ARCH-002/ATLAS/BUILD/RENDER/PILOT rows open |
| PM-12 | RENDER-001 | [`arch_blender_worker_contract_v1.md`](../docs/archive/2026-06-src-dev/plans/arch_blender_worker_contract_v1.md) | Headless worker contract completion |
| PM-13 | PILOT-001 | tile spine | Full spine G4 on real stills — planning gate |
| PM-14 | PLAN-BUILDING-GRAMMAR-001 | [`plan_building_grammar_evolution_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_building_grammar_evolution_v1.md) | Orders 3–6 APS UI specs |
| PM-15 | APS-UI-003b-EXPANDED | grammar evolution § | Building Authoring Tool **UX spec** (partial impl) |
| PM-16 | APS-GRAMMAR-INSPECTOR-001 | same | Rule chain inspector spec |
| PM-17 | APS-PREVIEW-002/003 | same | Assembly + variant preview specs |
| PM-18 | APS-UX-AUTHORING-001 | three-track plan | Pipeline status row spec (Grammar/Assembly/…/Validation) |
| PM-19 | GRAMMAR-002-SLICE | [`grammar_002_slice_001_v1.md`](../docs/archive/2026-06-src-dev/plans/grammar_002_slice_001_v1.md) | G2S-1 doc done — facade/roof examples |
| PM-20 | GRAMMAR-ITER-001 | [`grammar_iter_001_spec_v1.md`](../docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md) | API phases 2+ in spec still open |

### D — Material studio · tiles · MCP productivity

| # | ⟨ID⟩ | Doc | Gap |
|:---:|:---|:---|:---|
| PM-21 | PLAN-MATERIAL-STUDIO-A2 | [`plan_material_studio_phase_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_material_studio_phase_v1.md) | Category tree extension spec |
| PM-22 | APS-MAT-005 | same § Phase D | Reference image extraction — **defer** |
| PM-23 | PT-1-002 | procedural tiles plan | Variant matrices 7×4 — **5 YAML on disk** |
| PM-24 | PT-1-003 | same | G4 sign-off template for production atlases |
| PM-25 | TILE-BATCH-PLANNED | [`plan_tile_batch_v1_planner_mcp_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_tile_batch_v1_planner_mcp_v1.md) | `tile.generate` honesty — frozen |
| PM-26 | MCP-AGENT-LANG-GAPS | [`plan_mcp_agent_lang_program_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md) | MCP-SNAPSHOT-DIFF · SPINE-CHAIN · GRAMMAR-ITER tool wrappers |
| PM-27 | MCP-OPS-REPORT-001 | ops plan | `ops_intelligence_scan` MCP wrapper — **deferred** |
| PM-28 | OPS-MCP-FN-LAYER | [`ops_mcp_function_layer_v1.md`](ops_mcp_function_layer_v1.md) | Cycle 2+ fn_* catalog — gated |

### E — Architecture contracts (maintain)

| # | ⟨ID⟩ | Doc | Status |
|:---:|:---|:---|:---|
| PM-29 | ARCH-MAT-001 | [`arch_mat_001_material_authority_v1.md`](../docs/archive/2026-06-src-dev/plans/arch_mat_001_material_authority_v1.md) | **ACTIVE** — maintain |
| PM-30 | ARCH-PBG-MASSING-002 | [`arch_pbg_massing_placement_v1.md`](../docs/archive/2026-06-src-dev/plans/arch_pbg_massing_placement_v1.md) | Gate defer — mesh-face massing |
| PM-31 | DESIGN-PROC-ART | [`design_procedural_art_acceptance_v1.md`](../docs/archive/2026-06-src-dev/plans/design_procedural_art_acceptance_v1.md) | **HOLD** — 50-module sign-off |
| PM-32 | MODULE-KIT-TIER | [`plan_module_kit_production_tier_v1.md`](../docs/archive/2026-06-src-dev/plans/plan_module_kit_production_tier_v1.md) | PBR deferral policy |
| PM-33 | PROC-ASSEMBLY-READ | [`design_procedural_assembly_read_v1.md`](../docs/archive/2026-06-src-dev/plans/design_procedural_assembly_read_v1.md) | lod0 `pbr_status: deferred` rows |
| PM-34 | PLANNER-MCP-IDLE | [`planner_mcp_maintenance_idle_v1.md`](../docs/archive/2026-06-src-dev/plans/planner_mcp_maintenance_idle_v1.md) | **Stale** — mcp_active_queue has 2 ready P2 rows; update idle block |

---

## Defer / frozen registry (policy — @planner owns unpause)

| ⟨ID⟩ | Track | Reason | Unblock when |
|:---|:---|:---|:---|
| DEF-01 | MCP-PILOT-GRAMMAR-001 | Manual keyframe ship | tile_promotion_honest_check + G4 checklist |
| DEF-02 | TRACK-B-G4-SHIP | Warehouse integration test paused | DEF-01 unpause |
| DEF-03 | APS-PHASE-9-PRODUCT-GATE | Product gate not spine blocker | ATL★ operator review |
| DEF-04 | TILE-SPINE-RUN-WAREHOUSE | tile_spine_run integration test only | Track B unpause |
| DEF-05 | PLAN-AUDIT-020 | Fleet audit v20 | G-PLAY-01 EXECUTED + INFRA-E5-002 |
| DEF-06 | kit_production_002+ | Frozen in mcp queue | PM-02 unfreeze plan |
| DEF-07 | tile_batch_* (3) | Missing variant matrices + honest bake | PT-1-002 matrices |
| DEF-08 | tile.generate | Registry PLANNED | PM-25 honesty + coder-mcp |
| DEF-09 | Postgres ops fn_* | JSON-first until gate | >500 events / 30 days |
| DEF-10 | WH-TRACK-B-PAUSE | Orchestrator pause | Artist-ready keyframe |
| DEF-11 | 50-module art HOLD | Full kit sign-off | Production tier policy |
| DEF-12 | BQ-128-EXT offline editor | UX-E02 tail | PL-28 apply slice |

---

## Code / runtime stubs (not planner — route to @coder)

| Stub | Location | Planner note |
|:---|:---|:---|
| `src/sim/effects/` | absent | SIM-EFFECT-QUEUE-001 — plan signed |
| Scenario `EmitSimEffect` | `scenario_steps.rs` | SCENARIO-TRIGGER-001 — defer in sim plan |
| `ProgramGraph` / operators | — | vNext only — PM-07 defer doc |
| `validate-report weather` | validators | PL-31 registry row |
| `weather_sim_live.json` | debug_runs | PL-32 witness writer |
| `construction_placement_live.json` | MCP validators plan | @coder per PM-01 |
| INFRA profile RON set | `assets/config/infrastructure/profiles/` | PL-14 catalog |
| Variant graph schema file | `tools/mcp/schemas/` | PM-03 |

---

## Recommended pick order (next 2 sessions)

### Session 1 — @planner (≤4h)

```text
1. PL-01  Sign PLAN-MAP-ZOOM-SMOOTH-001
2. PL-04  Queue-sync BUILD-READ + confirm SIM-EFFECT rows
3. PL-07  BUILD-READ index hook
4. PL-35  defer_registry ↔ HANDOFF reconcile
```

### Session 1 — @planner-mcp (≤4h)

```text
1. PM-01  Sign MCP-P2-SIM-VALIDATORS-PLAN-001  → unblocks 3 coder-mcp
2. PM-02  Write mcp_kit_production_002_unfreeze_v1.md
3. PM-03  variant_graph_v1.schema.json (minimal VariantNode)
4. PM-34  Refresh planner_mcp_maintenance_idle_v1.md queue truth
```

### Session 2 — @planner-mcp

```text
PM-07  vNext defer doc (build_grammer2_exman depth boundary)
PM-08  FootprintMatrix spec doc for BUILD-READ-SHAPE-001
PM-15–17  APS UI preview specs (thin — bullet + wireframe refs)
PM-23  PT-1 variant matrix gap table (7×4 vs on-disk YAML)
```

### Session 2 — @planner

```text
PL-09  BUILD-READABILITY board hygiene
PL-19–21  Transport matrix R1–R9 one-page status rollup
PL-16  OPS intelligence Phase 1 slice outline (JSON-only)
PL-38  development_plan_index trim pass
```

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-13 | Full sweep — 72 planner-facing todos + defer registry |

```text
⟦/PLANNER-BACKLOG-SWEEP-001⟧  ΔWF→@planner PL-01 · @planner-mcp PM-01 PM-02 PM-03
```
