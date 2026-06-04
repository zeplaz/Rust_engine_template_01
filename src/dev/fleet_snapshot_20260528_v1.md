# Fleet snapshot — major review `v2.0`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-28 |
| **Worktree** | `C:\dev\github\Rust_engine_template_01` · **`master`** |
| **Truth** | `debug_runs/*.json` over markdown and machine queues |
| **Prior** | [`fleet_snapshot_20260527_v1.md`](fleet_snapshot_20260527_v1.md) v1.2 |
| **Routing** | [`fleet_maturity_signoff_routing_20260527_v1.md`](fleet_maturity_signoff_routing_20260527_v1.md) |
| **HANDOFF** | [`tools/orchestrator/queues/HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) |

---

## Executive summary

**Wave 6 product spine is largely landed** (Stage 5 FULL_APP, construction, industrial, infra slice 3, minimap/replay, Stage 7 M1–M3/steward on disk). **Machine queues are stale** on coder B P1 (M3/steward is green on disk).

**Two new fleet-wide priorities** (not optional hygiene):

1. **Visual perf production** — stop treating `RASTER_*` as normal workflow; ship budget resources ([`plan_visual_perf_production_v1.md`](plan_visual_perf_production_v1.md)). **P1-A landed** (skip duplicate CPU minimap when GPU RT committed).
2. **Dev artifact containment** — stop scattering `*live_proof*.rs` under `render/`, `construction/`, etc. ([`dev_artifact_containment_policy_v1.md`](dev_artifact_containment_policy_v1.md)).

**Witness drift on disk** (queue/docs still say OPEN):

| Witness | Issue |
|:---|:---|
| `wss_substrate_live.json` | `ecs_retire_fixture_green: false`, `wss_post_spine_001.green: false` while top-level `green: true` |
| `stage7_behavioral_live.json` | `s7b_m3_green` / `s7b_steward_green` **true** — close **S7B-M3-STEWARD** in queue |
| `stage7_behavioral_live.json` | `s7b_m4_play_green: false` — **new P1** for coder B |

---

## Role board (authoritative orders)

| Role | Queue | Verdict | Orders doc section |
|:---|:---|:---|:---|
| **@planner** | `active: []` | **REACTIVATED** (horizon exec) | [Planner orders](#planner-orders) |
| **@designer** | `active: []` drained | **ON-CALL** (2 signoffs) | [Designer orders](#designer-orders) |
| **@coder A** | `active: []` | **REASSIGNED** → perf + WSS witness | [Coder A orders](#coder-a-orders) |
| **@coder B** | 2 in `active[]` (stale) | **ACTIVE** → M4 play P1 | [Coder B orders](#coder-b-orders) |
| **@operator** | — | **ACTIVE** perf + witnesses | [Operator orders](#operator-orders) |

---

## Witness board (2026-05-28 spot-check)

| Domain | File | Status | Notes |
|:---|:---|:---|:---|
| Stage 5 FULL_APP | `stage5_full_app_live.json` | **green** | `readiness.passes: true` |
| Construction | `construction_stage_live.json` | **green** | `operational_green: true` |
| WSS substrate | `wss_substrate_live.json` | **mixed** | top `green: true`; **fixture/post-spine sub-keys red** |
| Stage 6 / infra | `stage6_virtualization_live.json` | **green** | slice3 / wc_d04 (prior) |
| Industrial | `industrial_activation_live.json` | **green** | IND-E02 (prior) |
| Minimap M3 | `minimap_compositor_live.json` | **green** | (prior) |
| Replay | `replay_editor_parity_live.json` | **green** | (prior) |
| Stage 7 behavioral | `stage7_behavioral_live.json` | **partial** | M3/steward **green**; **M4 play red** |
| Visual perf | code + runbook | **in progress** | P1-A in `tile_world_fallback.rs` |

---

## Planner orders

**Status:** Wave 6 exec **done**. **New work:** two exec plans + ledger reconcile.

| P | ID | Deliverable | Unblocks |
|:---:|:---|:---|:---|
| **1** | **PLAN-VISUAL-PERF-EXEC-001** | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) — **SIGNED** | @coder A perf lane |
| **2** | **PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001** | [`plan_dev_artifact_containment_exec_001_v1.md`](plan_dev_artifact_containment_exec_001_v1.md) — **SIGNED** | @coder A/B containment |
| **3** | **PLAN-LEDGER-REFRESH-015** | [`planner_status_audit_v15.md`](planner_status_audit_v15.md) — **SIGNED** | Fleet truth |
| 4 | **PLAN-STAGE7-M4-PLAY-001** | [`plan_stage7_m4_play_001_v1.md`](plan_stage7_m4_play_001_v1.md) — **SIGNED** | S7B-M4-PLAY-REMEDY |

**Do not:** Reopen closed wave 4/6 exec plans unless witness regression proves rollback.

**Sign-off:** Mark **SIGNED** when exec markdown exists; audit v15 after coder returns.

---

## Designer orders

**Status:** Wave 4–6 **drained**. **On-call** for new perf + Stage 7 play UX.

| P | ID | Deliverable | When |
|:---:|:---|:---|:---|
| **1** | **DESIGN-VISUAL-PERF-DEGRADE-001** | [`visual_perf_spike_degrade_ux_v1.md`](visual_perf_spike_degrade_ux_v1.md) **PASS** | Before PERF-P2 ships |
| **2** | **DESIGN-S7B-M4-PLAY-READ-001** | Player read for M4 play enqueue / pending dispatch (if UX unclear) | Only if coder B asks |
| 3 | DESIGN-HANABI-H-A2-PROD-001 | Deferred — default binary still must not wire L3 plugin | After explicit charter |
| 4 | DESIGN-CONSTRUCTION-R4-PRODUCT-001 | Product board UX | Planner horizon |

**Do not:** Rust. Sign **PASS** in `designer_signoff_registry.json` + design doc.

---

## Coder A orders

**Status:** Wave 6 WSS/Hanabi/infra **done on prior witness**; **reassigned** to perf + witness repair (not idle).

| P | ID | Scope | Exit |
|:---:|:---|:---|:---|
| **1** | **PERF-P2-TILE-RASTER-BUDGET-001** | `TileRasterBudget` resource; replace `RASTER_CHUNKS_PER_FRAME` env in release; wire spike clamp | 60s visual, no `RASTER_*`, p95 raster budget in PERF |
| **2** | **PERF-P2-FIRE-EXTRACT-CADENCE-001** | Cadence-scoped fire extract (replace spike-only skip as sole policy) | p95 `view_fire` &lt; 8 ms typical |
| **3** | **WSS-WITNESS-POST-SPINE-001** | Fix `wss_post_spine_001.green` + logistics on slab in `wss_substrate_live.json` | sub-block green |
| **4** | **WSS-WITNESS-ECS-RETIRE-001** | Align `ecs_retire_fixture_green` / smoke prod keys with hybrid authority false | consistent rollup |
| **5** | **DEV-CONTAIN-PHASE0-001** | Scaffold `src/dev/runtime_witness/` + CI deny new `**/live_proof.rs` outside root | policy Phase 0 |

**Depends on:** PLAN-VISUAL-PERF-EXEC-001 (P1) before P2 slices; containment exec before mass moves.

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib wss_substrate substrate stage5
cargo test -p proc_A_dine01 --lib chunk_grid_tests
```

---

## Coder B orders

**Status:** Product lanes closed; **queue stale** on M3/steward.

| P | ID | Status on disk | Action |
|:---:|:---|:---|:---|
| **0** | **S7B-M3-STEWARD-REMEDY-001** | `s7b_m3_green` + `s7b_steward_green` **true** | **Close queue row**; move to `done_2026_05_28`; no code unless regression |
| **1** | **S7B-M4-PLAY-REMEDY-001** | `s7b_m4_play_green: false`, `play_enqueue_wired: false` | **New P1** — wire sim writer + refresh `stage7_behavioral_live.json` |
| **2** | **LOG-E01-FULLAPP-UPGRADE-001** | optional | With @operator `--test visual` |
| 3 | Queue hygiene | — | Update `coder_active_queue.json` + backlog to match disk |

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib stage7_behavioral stage7_play
```

---

## Operator orders

| P | Task | Unblocks |
|:---:|:---|:---|
| **1** | Run `.\tools\orchestrator\scripts\run_visual_test_clean.ps1` then 60s `--test visual --stay-open` **release**, no `RASTER_*` | PERF acceptance baseline |
| **2** | `PERF=1` / `STALL=1` only during profiling sessions | Attribution |
| **3** | Refresh `debug_runs/agent_debug_index.json` after coder witness writes | Planner audit v15 |
| **4** | Capture p95 + top `upd_span` buckets to `debug_runs/perf_attribution_60s.md` | PERF-P4 |

---

## Cross-cutting rules

1. **Witness JSON wins** over queue markdown.
2. **No `RASTER_*` in clean run** — see [`visual_test_runbook_v1.md`](visual_test_runbook_v1.md).
3. **No new `live_proof.rs`** outside `src/dev/runtime_witness/` (after Phase 0 scaffold).
4. **Stage 5 gate** unchanged — perf work must not fork extraction spine.

---

## Regression (fleet gate)

```powershell
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7
cargo test -p proc_A_dine01 --lib chunk_grid_tests infra_slice3
```

---

## Version history

| Version | Date | Notes |
|:---|:---|:---|
| v2.0.0 | 2026-05-28 | Major review; perf + containment priorities; M3 green / M4 play open |
| v1.2.0 | 2026-05-27 | Planner/designer/coder A drained |
