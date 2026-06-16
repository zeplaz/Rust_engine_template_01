# PLAN-CON-P7-LOGISTICS-001 — graph-only freight + construction P7 hook `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **PLAN-CON-P7-LOGISTICS-001** |
| **Lang** | $ref:src/dev/agent_lang_v1.md · dispatch $ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md |
| **Unblocks** | **INFRA-E5-002** · **CON-P7-LOGISTICS-001** (coder A) |
| **Parent** | $ref:docs/archive/2026-06-src-dev/plans/plan_infrastructure_world_layers_exec_001_v1.md§9 · $ref:docs/archive/2026-06-src-dev/plans/construction_product_roadmap_phases_2_10_v1.md |
| **Tail slices** | $ref:docs/archive/2026-06-src-dev/plans/plan_infra_tail_exec_001_v1.md |
| **Gate** | **G-INFRA-07** — INFRA-E1-004 hydrate **green on disk** |
| **Owner** | `@coder` A |
| **Planner** | **SIGNED** |
| **Date** | 2026-06-03 |
| **Est** | 2–3 days |

---

## Problem

`logistics_throughput_live.json` shows `routes_open: 1`, `routes_blocked: 1`, `path_open_from_nav: true` — but **INFRA-E5-002** requires **all** play-scenario freight paths to resolve from `TransportNavExport` after road/rail build, with **no** non-test `patch_s7p_*` shortcuts.

Construction **Phase 7** (logistics & trade) reuses `economy/logistics/` — must not fork a parallel path system.

---

## Authority map ($sym)

| Domain | Authority | Consumers |
|:---|:---|:---|
| Transport topology | $sym:TransportGraph@src/infrastructure/transport/graph.rs → $sym:TransportTopology@src/systems/transport/types.rs | Nav export, construction corridor execute |
| Route existence | $sym:TransportNavExport@src/systems/transport/types.rs | $sym:routes@src/economy/logistics/routes.rs |
| Freight solve | $sym:ThroughputSolverState@src/economy/logistics/mod.rs | $ref:debug_runs/logistics_throughput_live.json |
| Construction P7 hook | $sym:FacilityPortal@src/economy/logistics/types.rs | Phase 7 `LogisticsHub` stub (thin) |

**Single writer:** transport graph hydrate (`INFRA-E1-004` done). Logistics **reads** nav — does not paint tiles.

---

## Coder slice — INFRA-E5-002 (pick first)

### Deliverables

| # | Task | Files | Exit |
|:---:|:---|:---|:---|
| 1 | **Graph-only route open** | `src/economy/logistics/routes.rs`, `portals.rs` | `path_open` from nav path only; no tile-boolean fallback in release |
| 2 | **Remove release shortcuts** | grep `patch_s7p_`, `apply_s7p_` outside `#[cfg(test)]` | `dehack_log_001` boundary holds |
| 3 | **Town → port proof** | integration test or play_scenario fixture | `routes_open >= 1` after seeded graph + road build |
| 4 | **Witness keys** | `src/dev/logistics_throughput_live_proof.rs` | See table below |

### Witness keys (`logistics_throughput_live.json`)

| Key | v19 | Target after E5-002 |
|:---|:---:|:---:|
| `path_open_from_nav` | true | **true** (maintain) |
| `routes_open` | 1 | **≥ 2** (town + port chain) |
| `routes_blocked` | 1 | **0** in default industrial fixture |
| `route_proofs_sample[].blocked_at` | mixed | **null** for primary chain |
| `graph_only_paths` | missing | **true** (new key) |
| `no_harness_tile_paint` | missing | **true** (new key) |

### Tests

```powershell
cargo test -p proc_A_dine01 --lib logistics play_scenario infrastructure::transport
```

---

## Construction P7 hook (same PR or follow-up)

**Do not** add new logistics solver — wire Phase 7 **facility** intent only.

| # | Task | Files | Exit |
|:---:|:---|:---|:---|
| 5 | **LogisticsHub stub** | `src/construction/` or `src/economy/logistics/types.rs` | `LogisticsHub { throughput, storage }` component + archetype id in book |
| 6 | **Depot portal attach** | reuse `FacilityPortal` | Operational depot registers portal → graph node |
| 7 | **Witness cross-link** | `construction_stage_live.json` | `construction_p7_logistics_hook_001.green: true` (lib fixture) |

**Territory:** one PR for E5-002 core; P7 hook may be second PR if >400 LOC.

---

## Acceptance (planner sign-off)

| # | Criterion |
|:---:|:---|
| 1 | Default industrial scenario: freight route opens **only** when nav path exists |
| 2 | No `patch_s7p_*` in non-test production modules |
| 3 | `logistics_throughput_live.json` keys `graph_only_paths` + `routes_blocked: 0` on fixture |
| 4 | Construction Phase 7 hook witness present (may be partial — hook only, not full trade sim) |
| 5 | Does not edit `src/construction/` execute funnel invariants ([`construction_invariants.md`](construction_invariants.md)) |

---

## Pick order (coder A)

```text
1. INFRA-E4-002 (utility flow) — may parallel if files disjoint
2. INFRA-E5-002 — THIS PLAN (after read)
3. INFRA-E6-001/002/004 — after E5-002 or parallel if territory clean
4. CON-P7-LOGISTICS-001 hook — witness row only
```

**Blocked:** none — E1-004 green. **Do not** wait on G-PLAY-01 operator.

---

## Orchestrator paste

```text
@coder A — INFRA-E5-002 + CON-P7 hook

READ: docs/archive/2026-06-src-dev/plans/plan_construction_p7_logistics_exec_001_v1.md
Territory: src/economy/logistics/, src/strategic/logistics_graph.rs (read-only attach)

Witness: logistics_throughput_live.json — add graph_only_paths; routes_blocked → 0
Optional: construction_stage_live.json construction_p7_logistics_hook_001

Regression: cargo test -p proc_A_dine01 --lib logistics play_scenario
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Thin exec; unblocks INFRA-E5-002 |
