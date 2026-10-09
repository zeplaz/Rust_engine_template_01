# PLAN-LOGISTICS-ORDERS-v1 — standing lines, one-off runs, draw demands

| Field | Value |
|:---|:---|
| **ID** | PLAN-LOGISTICS-ORDERS-v1 |
| **Fleet** | `logistics` (charter `fleet_charter_20261008_v1.md`) |
| **Date** | 2026-10-08 |
| **Code home** | `src/economy/logistics/orders/` (sim core + Bevy adapter) · `src/gui/logistics_orders_panel.rs` (UI) |
| **Depends on** | existing throughput spine (`LogisticsThroughputPlugin`, LOG-A…D) — read-only |
| **Consumers** | `ai` (issues orders via the same command API), `front` (fronts publish draw demands), `interaction` (menus / panel entry) |

---

## 0. Research note — Mars Tactics

Searched the Mars Tactics (Hooded Horse) Steam news feed through the 2026-09-22 "Capital Campaign"
post, the Sept 2025 "Playtest Insights — RTWP, Logistics and World Map" post, the 2025 Q4 dev
update and "Physical Inventory". What is public there, in our words:

- Their first logistics model was **too abstract and too punishing**; playtesters felt boxed in.
  They moved to **physical logistics**: a fleet of literal vehicles that carry cargo (and squads)
  across the map, and inventories that are fully visible.
- Demand on the economy is expressed as **quotas per period** (ship N Flux by the end of the
  earnings period), with escalating targets; transport assets (dropships) are **assigned to
  roles** (military vs civilian logistics).

The specific write-up the user referenced — the UI split between **continuous supply lines** and
**one-off supply runs** that loop vehicles until a quantity is delivered — was **not found** in the
indexed posts (likely Discord / social). This plan is built from the user's description:

> a one-off "deliver N of X to Y" uses the available vehicle pool, repeats trips until the
> quantity is fulfilled, then completes and releases its vehicles; standing lines keep stock
> flowing based on queues and **draw demands**.

---

## 1. What exists (survey)

| Layer | File | Models | Gap for orders |
|:---|:---|:---|:---|
| Throughput solver | `economy/logistics/solver.rs` | greedy per-edge capacity reservation for `ResourceFlowEdge`s | flow edges are facility→facility "pipes"; no vehicles, no orders |
| Freight ledger | `economy/logistics/propagation.rs`, `types.rs` | `InTransitLedger` of `FreightLot` (amount, ETA ticks, path) | lots are anonymous; no order id, no trip count, no return leg |
| Resource flow | `economy/resource_flow.rs` | `ResourceFlowNode` buffers, produce/consume, starvation | consumers don't *publish* demand — starvation is detected, not requested |
| Graph | `strategic/logistics_net.rs`, `transport_bridge.rs` | `LogisticsGraph` capacity/disruption → overlays | travel-time source for the adapter |
| Vehicles | `entities/vehicles/runtime.rs` | `Truck` / `RoadVehicle` structs (capacity, whitelist) | not ECS-scheduled, no dispatch |
| UI | `gui/logistics_targets_panel.rs`, `logistics_focus.rs`, `intent_draw.rs` | focus picker; haul polylines for ledger lots | no order creation or monitoring |

**Decision:** add an **order layer** beside the throughput spine, not inside it. The throughput solver
keeps modelling continuous industrial pipes (conveyors, pipelines, rail flows). The order layer
models **discrete vehicle trips** between logistics sites. They share nothing mutable.

---

## 2. Isolation contract (sim → snapshot → UI)

```text
 UI panel / AI / fronts            sim core (pure Rust, no Bevy)            presentation
 ─────────────────────            ─────────────────────────────            ────────────
 LogisticsOrderCommand ──queue──▶ LogisticsOrderSim::apply(cmd)
                                  LogisticsOrderSim::tick(&travel)  ──▶ LogisticsOrdersSnapshot ──▶ panel, map lines
```

- **Core** `orders/core.rs` etc.: plain structs, `BTreeMap` keyed state, integer ticks, no `Entity`,
  no `Res`. Fully deterministic: same commands + same travel model ⇒ identical state. Tested headless.
- **Commands** are the only write path. Player UI, AI, and front systems all emit the same
  `LogisticsOrderCommand` message; the adapter drains them once per sim tick, in arrival order.
- **Snapshot** `LogisticsOrdersSnapshot` (resource, rebuilt each tick by one system) is the only
  thing UI reads. UI never touches `LogisticsOrderSim`.
- **Adapter** `orders/plugin.rs`: owns the `LogisticsOrderSimRes`, the `SiteId ↔ Entity` registry,
  and the `TravelModel` impl (world distance + vehicle speed; graph-aware later).
- Single writer per resource: `step_logistics_orders_system` is the sole `ResMut<LogisticsOrderSimRes>`.

---

## 3. Domain model

### 3.1 Identifiers and cargo

- `SiteId(u32)` — a logistics endpoint (depot, front supply point, construction site, plant, unit).
- `OrderId(u32)`, `VehicleId(u32)`, `DemandId(u32)` — monotonically allocated by the sim.
- `CargoKind(String)` — buffer tag (`"Cement"`, `"ammo"`, `"fuel"`, `"personnel"`, …). Personnel are
  cargo like any other; vehicles declare which cargo **classes** they accept.
- `CargoClass { Bulk, Liquid, Munitions, Personnel, Deployable }` — vehicle compatibility filter.
  Quantities are `u32` **units** (1 unit = 1 t for bulk/munitions, 1 m³ for liquid, 1 person for
  personnel). Integers keep the sim exactly deterministic and trip counts exact.

### 3.2 Stock

`StockLedger`: `(SiteId, CargoKind) → { on_hand, reserved_out, inbound }`.
- `available = on_hand − reserved_out` (what a new pickup may claim).
- `projected = on_hand + inbound` (what draw-demand evaluation uses — prevents double-ordering).

### 3.3 Vehicle pool

```text
Vehicle { id, class_mask, capacity_units, speed_mps, home: SiteId, at: SiteId, phase, assignment }
phase: Idle | ToPickup{eta} | Loading{eta} | Hauling{eta, load} | Unloading{eta, load}
```
- Pools are per **owner** (`Faction`/player). AI and player draw from their own pools.
- A vehicle is **free** iff `phase == Idle`. Only free vehicles are allocated.
- After unloading, the vehicle becomes `Idle` at the destination. The allocator prefers vehicles
  with the smallest deadhead (`travel(at → source)`), so multi-trip orders naturally cycle the same
  trucks back to the source without a hard binding.
- Optional `vehicle_cap` on an order limits how many trucks it may hold at once.

### 3.4 Two order kinds

| | **StandingLine** (continuous) | **OneOffRun** (supply run) |
|:---|:---|:---|
| Meaning | "Keep site Y supplied with X from site S" | "Deliver N of X from S to Y, then stop" |
| Driver | the sink's **draw demand** (deficit vs buffer target) | fixed `quantity` |
| Ends | never by itself (Pause / Cancel only) | `Completed` when `delivered == quantity` |
| Vehicles | re-acquired each dispatch; released after each trip | held while trips remain; all released at completion |
| UI glyph | `⟳` loop, solid route line | `➜` arrow, dashed route line, trip `n/m` |
| Progress | fill ratio of sink buffer, throughput t/min | `delivered / quantity`, `trips done / planned` |

**Lifecycles**

```text
OneOffRun:    Pending ─▶ Active ─▶ Completed
                 │         │  ▲
                 │         ▼  │ (source refilled)
                 │      Waiting{reason}
                 └────────┴──▶ Cancelling ─▶ Cancelled      (in-flight trips finish and deliver)
                              Failed{reason}               (source/destination removed)

StandingLine: Active ⇄ Paused ;  Active ⇄ Idle (demand satisfied) ; Active ─▶ Starved{reason} ;  * ─▶ Cancelled
```

`Waiting` reasons: `NoVehicles`, `SourceEmpty`, `NoCompatibleVehicle`, `Unreachable`.

### 3.5 Trip planning (multi-trip batching)

For a one-off run with `remaining = quantity − delivered − in_flight`:
- each allocation pass assigns free vehicles while `remaining > 0` and `vehicle_cap` permits;
- each trip claims `load = min(vehicle.capacity, remaining, source.available)` and **reserves**
  it at the source (`reserved_out += load`) and at the destination (`inbound += load`);
- `trips_planned = trips_done + trips_in_flight + ceil(remaining / typical_capacity)` where
  typical capacity = the largest compatible free-or-assigned vehicle capacity — displayed as
  "trip 3 / 5"; it is re-estimated every tick so it stays honest when the pool changes;
- partial loads are allowed when the source is short (the run continues as stock arrives).

Trip timeline (ticks): `deadhead(at→src) → load_ticks → travel(src→dst) → unload_ticks`.
Loading debits `on_hand` and clears `reserved_out`; unloading credits `on_hand` at the sink and
clears `inbound`. Nothing is ever teleported.

---

## 4. Draw demands

A **draw demand** is a consumer saying "I need X here, this much, this urgently". Fronts, depots,
construction sites and units all publish the same struct:

```text
DrawDemand {
  id, site, cargo,
  consumer: Front | Depot | Construction | Unit | Plant,
  buffer_target: u32,        // desired on-hand stock (high watermark)
  reorder_point: u32,        // trigger when projected < reorder_point (low watermark)
  draw_rate_per_min: u32,    // modelled consumption at the site (sim drains on_hand)
  priority: Priority,        // Critical | High | Normal | Low
  min_priority_floor ...     // (v2) escalation caps
}
```

- **Hysteresis**: a line serving a demand dispatches only when `projected < reorder_point`, and then
  orders up to `buffer_target − projected`. Stops truck spam on tiny deficits.
- **Consumption**: the sim drains `on_hand` at `draw_rate_per_min` (fronts burn ammo, sites burn
  cement). Shortfall accumulates `starved_ticks` and sets `DemandHealth::{Satisfied, Low, Starving}`.
- **Who satisfies a demand**: a StandingLine binds `(source, sink, cargo)`; it reads the sink's
  demand for that cargo. A demand with no line is **unserved** and shows in the UI as a call to action
  ("Front Alpha needs ammo — no supply line"). AI auto-creates lines for unserved demands.
- **Multiple lines** may feed one demand (two depots → one front); the deficit is shared — each line
  sees `deficit − inbound_from_other_lines`, because `inbound` is per sink, not per line.

### 4.1 Allocation order, starvation and fairness

Each tick the allocator builds a request list and serves it with free vehicles:

1. `effective_priority = base_priority_weight + aging` where aging grows by 1 per N ticks a
   request waits unserved and resets on service. Critical (weight 300) > High (200) > Normal (100) >
   Low (0); aging is capped at +150, so Low can overtake Normal after a long wait but never Critical.
2. Starving demands (`on_hand == 0` with draw > 0) add an urgency bonus.
3. Sort by `(effective_priority desc, order id asc)` — deterministic.
4. Round-robin within one pass: each request gets **one** vehicle per round; rounds repeat until no
   free vehicles or no request can use one. A big one-off can't monopolise the pool while a front
   starves; a lone request still gets the whole pool.

---

## 5. Commands (shared API for player, AI, fronts)

```text
LogisticsOrderCommand
  CreateStandingLine { owner, source, sink, cargo, priority, vehicle_cap }
  CreateOneOffRun    { owner, source, sink, cargo, quantity, priority, vehicle_cap }
  Cancel { order }  Pause { order }  Resume { order }  SetPriority { order, priority }
  PublishDemand { site, cargo, consumer, buffer_target, reorder_point, draw_rate_per_min, priority }
  RetractDemand { site, cargo }
  AddVehicle { owner, home, class_mask, capacity_units, speed_mps }   // pool growth (production / editor)
  SetStock { site, cargo, on_hand }                                   // seeding / bridge import
```
Commands are validated in the sim (unknown site → rejected, reason logged in snapshot
`rejected_commands`). `PublishDemand` is an upsert keyed by `(site, cargo)` so fronts can republish
every tick cheaply.

---

## 6. UI (presentation only, reads the snapshot)

Panel `Logistics Orders` (`src/gui/logistics_orders_panel.rs`), toggled from the existing logistics
targets panel and its own hotkey once `interaction` adds a binding:

- **Tabs**: `Supply lines ⟳` · `Supply runs ➜` · `Draw demands` · `Fleet`.
- **New order bar**: kind toggle (`Line` / `Run`), source ▾, destination ▾, cargo ▾, quantity (runs
  only), priority ▾, vehicle cap. The kind toggle swaps the quantity field for "keep supplied"
  wording so the two kinds can't be confused at creation time.
- **Run rows**: `➜ 40 t Cement  Depot A → Site 3   [████░░] 24/40  trip 3/5  ⛟×2  ETA 1:20  [Cancel]`.
  Completed runs fade into a "recent" list for a few minutes then drop.
- **Line rows**: `⟳ Ammo  Depot B → Front Alpha  fill 62%  ▲ 12 t/min  ⛟×1  [Pause][Cancel]`.
- **Demand rows**: health dot (green/amber/red), site, cargo, on-hand/target bar, draw rate,
  served-by count; unserved demands show `[+ Create line]` that pre-fills the order bar.
- **Fleet**: free / busy counts per owner; per-vehicle phase and assignment.
- **Map**: order routes drawn in the existing intent overlay pass — solid for lines, dashed for runs,
  vehicle dots at interpolated trip progress (from snapshot ETA fractions).

---

## 7. Phases (each = one `[logistics]` checkpoint commit)

| Phase | Slice | Exit |
|:---|:---|:---|
| **LO-0** | This plan | committed |
| **LO-1** | Sim core: ids, stock ledger, vehicle pool, one-off runs with multi-trip, completion/cancel, travel trait | unit tests: single trip, multi-trip 5×, completion releases vehicles, cancel lets in-flight finish, partial load / waiting on source, determinism |
| **LO-2** | Draw demands + standing lines: hysteresis dispatch, consumption, health, priority + aging + round-robin fairness | tests: line refills to target and idles, starvation fairness, multi-line shared deficit |
| **LO-3** | Snapshot + command API + Bevy adapter plugin (sim tick under `economy_sim_running`, site registry, distance travel model) | headless App test: commands → snapshot rows |
| **LO-4** | UI panel (tabs, creation bar, progress rows, cancel/pause) | compiles; panel reads snapshot only |
| **LO-5** | Map route overlay + vehicle dots; site discovery from `LogisticsSiteRoot` / `ResourceFlowNode` entities; stock bridge | live smoke |
| **LO-6** | Cross-fleet: AI issuing helper (`ai`), front demand publisher hook (`front`), hotkey (`interaction`) | board asks answered |

## 8. Risks

- **Dual stock truth** with `ResourceFlowNode.buffer_by_tag`: until LO-5 the order sim owns its own
  ledger; the bridge must be one-way per tag (import on register, explicit export), never two writers.
- **Tick rate**: the sim counts sim ticks, not frames; the adapter converts `speed_mps` and distances
  to ticks with the sim's seconds-per-tick so pauses/time-scale stay correct.
- **Pool exhaustion UX**: always show *why* an order waits (`NoVehicles`, `SourceEmpty`) — opaque
  waiting is exactly the "too harsh" failure Mars Tactics reported.
