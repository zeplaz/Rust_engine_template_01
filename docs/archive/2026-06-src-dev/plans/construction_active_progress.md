# Construction recovery — active progress

**Last updated:** 2026-05-22 (PLAY closed · follow-up board added)

## Status

| Milestone | State |
|-----------|--------|
| Phase 2 P6–P9 | Done (boards + witnesses) |
| **CONSTRUCTION_OPERATIONAL_GREEN** | **true** in `debug_runs/construction_stage_live.json` |
| Round 2 / Round 3 (27× R3) | Done (static + runtime boards) |
| BUILD-P0…P5 recovery | Done (19/19 via module witnesses) |

## Resume session changes

1. **Proof pipeline** — write JSON → `sync_construction_proof_witness_flags` → sync boards (OP-07 / PHASE2-BUILD-16 align with `operational_green`).
2. **Catalog menus** — commercial, industrial, utilities use `BuildingDefinitionRegistry` + `intent_from_archetype`.
3. **Authority audit** — `no_legacy_gui_build_placement_in_src` integration test (no `crate::gui::build` imports).
4. **Recovery registry** — `CONSTRUCTION_TODOS` paths → `src/construction/`.

## Proof commands

```powershell
cargo test -p proc_A_dine01 construction:: --lib
cargo test -p proc_A_dine01 construction::live_proof::live_proof_sim_tests::simulation_writes_construction_stage_live_json_operational_green --lib
```

## Phase 4 — industrial activation + supply chains

- Guide: [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md)
- Chain index: `assets/configs/industrial_supply_chains.json`
- **10 placeable steps** (not monolith-only): aggregate mine, cement kiln, mixer (Portland + Geopolymer), bauxite, alumina refinery, smelter, fabrication + legacy integrated plants
- Activation: `src/economy/supply_chain.rs` — role → runtime component + `ElectricalComponent` from JSON `power_consumption`
- Industrial menu grouped by `supply_chain` with power labels
- Tests: `cargo test -p proc_A_dine01 economy:: --lib` (5 passing)
- **Todo board:** 31× `INDUSTRIAL-*` — spec [`industrial_activation_phase_todos.md`](industrial_activation_phase_todos.md)
- **Sprint done:** I1-05 proof JSON, I2-01..07 resource flow + starvation, I3-01..05 grid overload/thermal, I3-03..04 utilities, I4-01..03 logistics/batch/path gate, GOV-01
- **Tests:** `cargo test -p proc_A_dine01 economy:: --lib` (15 passing)
- **I4-04 done:** `spatial_district.rs` — chunk anchors, `IndustrialDistrictSnapshot`, clustered vs spread load test
- **INDUSTRIAL_ACTIVATION_GREEN:** all 31× `INDUSTRIAL-*` rows reconcile Done (witness-driven)

## Infrastructure + logistics (witness-closed 2026-05-22)

| Lane | Status | Proof |
|------|--------|-------|
| VM multiview / isolation (VM-06…11) | **Green** (live witness) | `infrastructure_view_isolation_live.json`, `isolation` lib tests |
| Logistics (LOG-A…D) | **Green** | `LOGISTICS_THROUGHPUT_GREEN`, `logistics_throughput_live.json` |
| Industrial activation | **Green** | `INDUSTRIAL_ACTIVATION_GREEN`, `industrial_activation_live.json` |

**Stage 5 operational:** closed — [`stage5_operational_signoff.md`](stage5_operational_signoff.md). **Next lane:** [`stage5_5_open.md`](stage5_5_open.md).

### Phase LOG — logistics throughput (active)

- **Architecture:** [`Logistics throughput architecture.md`](Logistics%20throughput%20architecture.md) (review corrections merged §3–§7)
- **Spec:** [`logistics_throughput_phase_todos.md`](logistics_throughput_phase_todos.md)
- **24× `LOG-*` rows** → exit `LOGISTICS_THROUGHPUT_GREEN`
- **LOG-A slice (2026-05-21):** live proof harness + witness `path_open` fix + infra `transport_edge` pairing test
- **LOG-B (2026-05-21):** `InTransitLedger`, no same-tick teleport, partial proofs, staggered supply-chain link fix
  - `cargo test -p proc_A_dine01 log_b --lib`
- **LOG-C (2026-05-21):** SoA reservation invariant, congestion/pressure tests, geographic cascade sim, overlay uses solver `load`, route proofs in live JSON
  - `cargo test -p proc_A_dine01 log_c --lib`
- **LOG-D (2026-05-21):** async district queue, corridor class, route invalidation, diagnostics panel section, full-board live proof
  - `cargo test -p proc_A_dine01 economy::logistics::live_proof::live_proof_sim_tests::simulation_writes_logistics_throughput_live_json_all_logistics_green --lib`
- **LOGISTICS_THROUGHPUT_GREEN:** 24× rows reconcile in proof harness (lib tests)
- `INDUSTRIAL-I4-03` stub superseded by LOG-A-04 / LOG-C-07 (industrial board stays green)

Stage 5 operational readiness and construction operational green can stay green while these run in parallel.

## Session playback issues (2026-05-22)

User-reported sim UX: panels, perf, build/road **tile occupation** (Syx-style), fire. Board: [`session_playback_issues_todos.md`](session_playback_issues_todos.md) — **all PLAY rows closed** (2026-05-22): sim HUD defaults, tile occupation, fire stability witness, mock shapes in registry.

**Follow-up (closed):** [`post_play_followup_todos.md`](post_play_followup_todos.md). **Active list:** [`next_action_todos.md`](next_action_todos.md).

## Still deferred (construction lane — lower than P0 above)

- Full **Ctrl+Y redo replay**
- **Demolish undo** entity restore
- Hands-on GUI sim for every OP runtime row (`cargo run -p proc_A_dine01 -- --test visual`)
