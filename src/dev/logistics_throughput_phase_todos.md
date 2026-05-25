# Logistics throughput — Phase LOG (post industrial activation)

> **Architecture:** [`Logistics throughput architecture.md`](Logistics%20throughput%20architecture.md)  
> **Live board:** [`logistics_throughput_todos.rs`](logistics_throughput_todos.rs) — `LOGISTICS_THROUGHPUT_GREEN` when all rows **Done**.  
> **Prerequisite:** `INDUSTRIAL_ACTIVATION_GREEN`. **Not** Stage 5 · **not** construction lane.

## North star

**Transport owns topology.** **Economy owns demand.** **ThroughputSolver owns movement.**  
`LogisticsGraph` is a **derived cache only** — never authoritative sim state.

Failure to fix: magical inventory transfer, `path_open: true` stubs, facility nodes orphaned on graph rebuild, overlays showing static capacity instead of solved load.

---

## Work order

1. **LOG-A** — Authority & wiring (implement first; unblocks everything).
2. **LOG-B** — Freight ledger (no teleport; compact paths).
3. **LOG-C** — ThroughputSolver + pressure + RouteProof (before async).
4. **LOG-D** — Scale, classification, streaming, tooling.

---

## LOG-A — Authority & wiring

| Id | Goal | Runtime check |
|----|------|----------------|
| `LOG-A-01` | `LogisticsGraph` derived-only + `revision`; stop appending facility nodes in `logistics_bridge` | Graph rebuild replaces nodes; solve does not `push` to graph |
| `LOG-A-02` | `FacilityPortal { anchor, transport_anchor }` + `PortalAttachmentMap` at GraphSync | No `LogisticsNodeId` on facility components |
| `LOG-A-03` | `LogisticsEdge.transport_edge: Option<TransportEdgeId>` | Every corridor edge pairs 1:1 with transport directory |
| `LOG-A-04` | `path_open` from `TransportNavExport` reachability | Kill `path_open: true` in `link_supply_chain_edges_system` |
| `LOG-A-05` | `RouteHandle { id, topology_revision }` + invalidate on `ConstructionWorldRevision` | Stale handle → route refresh |
| `LOG-A-06` | `debug_runs/logistics_throughput_live.json` | `routes_open`, `routes_blocked`, `topology_revision` |
| `LOG-A-07` | `InfrastructureGraph` pairs by `transport_edge` not sorted index | Unit test: permuted edge order still correct |

**Acceptance:** road connects mine→refinery → open; remove segment → blocked after refresh.

---

## LOG-B — Freight ledger

| Id | Goal | Runtime check |
|----|------|----------------|
| `LOG-B-01` | `RoutePathStore` + `RoutePath { first_edge, edge_count }` | No `Vec<TransportEdgeId>` on lots |
| `LOG-B-02` | `InTransitLedger` with `RouteHandle` + `progress_edge` + `remaining_ticks` | Transfer test uses ledger |
| `LOG-B-03` | `FreightMovementModel` Continuous / Batched | Rail/batch vs truck paths differ ETA |
| `LOG-B-04` | `propagate_resource_flow_system` commits arrivals only | Same-tick teleport removed |
| `LOG-B-05` | Partial fulfillment + shortage witness | Deficit recorded when route saturated |

---

## LOG-C — ThroughputSolver

| Id | Goal | Runtime check |
|----|------|----------------|
| `LOG-C-01` | SoA `ThroughputSolverState { load, capacity, reserved }` | Hot loop uses `Vec` index, not HashMap |
| `LOG-C-02` | `FreightReservationBook` + solve pass | Edge `reserved` ≤ `capacity` invariant test |
| `LOG-C-03` | `feedback_congestion_from_load_system` | Congestion rises when load > 0.8 cap |
| `LOG-C-04` | `propagate_corridor_pressure_system` | Neighbor edges gain pressure when saturated |
| `LOG-C-05` | `RouteProof` ring + JSON export | `blocked_edge`, `requested`, `delivered` per request |
| `LOG-C-06` | Overlay injects **solver load** not static capacity | `logistics_throughput` tracks load after solve |
| `LOG-C-07` | Integration: cut road → refinery starved → smelter | `cargo test` aluminum chain geography |

**Order:** LOG-C-05 before LOG-D-04 (async).

---

## LOG-D — Scale & futures

| Id | Goal | Runtime check |
|----|------|----------------|
| `LOG-D-01` | `CorridorClass` on `TransportEdgeMeta` / field state | Road vs rail route legality test |
| `LOG-D-02` | District-scoped solve via `IndustrialDistrictSnapshot` | Only active district edges in solve |
| `LOG-D-03` | Route cache invalidation on chunk/streaming bbox | Revision bump on partial hydrate |
| `LOG-D-04` | Async district solve scaffold (main-thread apply only) | Job posts + applies next frame; no off-thread field mutate |
| `LOG-D-05` | `diagnostics_ui` logistics panel | Top saturated edges + starved facilities |

---

## Proof commands

```powershell
cargo test -p proc_A_dine01 economy:: --lib
cargo test -p proc_A_dine01 strategic::transport_bridge --lib
cargo test -p proc_A_dine01 dev::logistics_throughput --lib
```

---

## Exit gate — `LOGISTICS_THROUGHPUT_GREEN`

All 24× `LOG-*` rows in [`logistics_throughput_todos.rs`](logistics_throughput_todos.rs) reconcile **Done** via witness predicates.

Minimum playable bar before full green:

- **LOG-A** complete (real `path_open`, portals, proof JSON)
- **LOG-B-04** (no teleport)
- **LOG-C-07** (one geographic cascade test)

Full green adds SoA solver, pressure, RouteProof, scale rows.

---

## Re-opened industrial rows (semantic)

`INDUSTRIAL-I4-03` marked Done on `path_open` stub — treat as **superseded** by LOG-A-04 / LOG-C-07 until LOG-A green. Do not revert industrial board; LOG board owns depth.
