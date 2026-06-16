# R4-PLAN-001 — Round 4 corridor phase spec + witness schema `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **R4-PLAN-001** |
| **Parent** | **PLAN-CONSTRUCTION-R4-001** — [`construction_round4_product_gate_plan_v1.md`](construction_round4_product_gate_plan_v1.md) |
| **Coder lane** | **R4-CORRIDOR-001** (opens when `product_board_open: true`) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Designer companion** | **R4-PLAN-002** — [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Runbook** | [`infrastructure_construction_runbook_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/infrastructure_construction_runbook_v1.md) §10 |
| **Witness** | `debug_runs/construction_stage_live.json` |

**No Rust in this deliverable.** Spec + witness contract for **corridor** (transport-edge) construction phases — distinct from **site** `SiteConstructionPhase`.

---

## Executive summary

| Domain | Authority | Phases |
|:---|:---|:---|
| **Corridor spans** (roads/rail edges) | [`CorridorConstructionBook`](../strategic/construction_book.rs) | `Planned` → `InProgress` → `Completed` |
| **Operational sites** (buildings) | [`SiteConstructionBook`](../strategic/site/) | Full site enum (unchanged) |
| **Persistence** | R8 `TransportNetworkSnapshot.construction` | Wire strings + `progress` |
| **Sim tick** | `advance_corridor_construction_book_on_sim_tick` | Sole progress writer |

Round 4 **product** work = operator-visible corridor lifecycle on the **sim map** + proof JSON — not new execute funnels outside `src/construction/`.

---

## Phase model (corridor)

| Phase | `traffic_factor` | Logistics | Visual (designer) |
|:---|:---:|:---|:---|
| **Planned** | `0.0` | Edge closed to throughput | Amber / dashed edge overlay |
| **InProgress** | `progress` ∈ [0,1] | Partial | Progress fill along edge |
| **Completed** | `1.0` | Full | Committed road palette |

**Default for missing book row:** **Completed** (`traffic_factor` 1.0) — legacy baked edges.

**Wire encoding** (R8 slice):

| Phase | JSON `phase` string |
|:---|:---|
| Planned | `"Planned"` |
| InProgress | `"InProgress"` |
| Completed | `"Completed"` |

Maps via `corridor_phase_from_wire` / `corridor_phase_to_wire` in [`construction_book.rs`](../strategic/construction_book.rs).

---

## Authority map

```text
Construction tool (roads/rail commit)
  → ConstructionPlanQueue → execute (topology mutation)
  → plan_edge / align book (strategic)
CorridorConstructionBook
  → advance_corridor_construction_book_on_sim_tick (sim only)
  → GraphSync → CorridorConstructionStatus (ECS mirror)
  → logistics traffic_factor(edge)
Transport save/load
  → transport_construction_records_from_book
  → G4 hydrate → apply_corridor_book_from_transport_snapshot
```

| Writer | Allowed mutation |
|:---|:---|
| `advance_corridor_construction_book_on_sim_tick` | `progress`, `phase` on **existing** rows |
| `CorridorConstructionBook::plan_edge` | Insert **Planned** row |
| `align_corridor_book_with_transport_directory` | Reconcile after bake/G4 |
| Construction execute paths | Topology + **plan** corridor rows — **not** skip queue |
| Preview / ghosts | **No** book mutation |

---

## R4 product scope (coder — when board opens)

| # | Deliverable | Exit |
|:---:|:---|:---|
| R4-C1 | Map editor **Save** embeds `construction` slice from live book | Roundtrip in snapshot tests |
| R4-C2 | Simulation **live proof** writes `construction_r4_corridor_001` block | See § Witness |
| R4-C3 | Tile / edge debug shows corridor phase on active edges (read-only) | Optional field in witness |
| R4-C4 | Construction tool **plan corridor** on new edge commit (no instant Completed cheat) | Lib test: planned row after road execute |
| R4-C5 | Minimap construction heat respects `traffic_factor < 1` | Cross-check only — not Stage 5 gate |

**Budget:** ≤3 files per PR; all logic under `src/construction/` + strategic book (existing).

---

## Witness schema — `construction_r4_corridor_001`

**File:** `debug_runs/construction_stage_live.json`  
**Writer:** `write_construction_live_proof_system` extension (sim session) — **not** hand-edited.

| JSON pointer | Type | Green when |
|:---|:---|:---|
| `/construction_r4_corridor_001/gate` | string | `"R4-CORRIDOR-001"` |
| `/construction_r4_corridor_001/green` | bool | rollup below |
| `/construction_r4_corridor_001/product_board_open` | bool | `true` only after product gate lifted |
| `/construction_r4_corridor_001/book_row_count` | number | `≥ 0` |
| `/construction_r4_corridor_001/planned_count` | number | exists |
| `/construction_r4_corridor_001/in_progress_count` | number | exists |
| `/construction_r4_corridor_001/completed_count` | number | exists |
| `/construction_r4_corridor_001/sim_tick_writer_wired` | bool | `true` — `advance_corridor_construction_book_on_sim_tick` registered |
| `/construction_r4_corridor_001/r8_roundtrip_ok` | bool | lib test: snapshot ↔ book |
| `/construction_r4_corridor_001/corridor_phase_visual_wired` | bool | `true` when **R4-PLAN-002** designer tokens landed |

**Rollup (product gate open):**

```text
construction_r4_corridor_001.green :=
  product_board_open
  AND sim_tick_writer_wired
  AND r8_roundtrip_ok
  AND (planned_count + in_progress_count + completed_count) >= book_row_count
  AND corridor_phase_visual_wired
```

**Prep / board closed:** block omitted or `green: false`, `product_board_open: false` — do **not** fail `construction_r4_prep_001`.

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib corridor_construction
cargo test -p proc_A_dine01 --lib transport::persistence
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

**Existing tests to extend (coder):** `construction_book.rs` unit tests · transport G4 hydrate.

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| Hand `green: true` without sim writer | Not R4 |
| Site `SiteConstructionPhase` on transport edges | Wrong book |
| Wall-clock progression | Determinism / sim-tick only |
| Second book writer in render/minimap | Authority drift |
| Instant **Completed** on player road commit without plan | Breaks phase UX |

---

## Gate chain

```text
CONSTRUCTION-R4-PREP-001          ☑ prep docs
PLAN-CONSTRUCTION-R4-001          ☑ product gate
R4-PLAN-001 (this doc)            ☑ SIGNED
R4-PLAN-002 (designer MV presets) ☑ SIGNED
        │
        ▼
product_board_open = true         ◐ product decision
R4-CORRIDOR-001 @coder            ◐ blocked until board
```

---

## Copy-paste — @coder

```
Lane: R4-CORRIDOR-001
Read: docs/archive/2026-06-src-dev/plans/construction_round4_corridor_phase_spec_v1.md
      src/strategic/construction_book.rs
      src/systems/transport/snapshot.rs
Prereq: product_board_open + R4-PLAN-002 visual tokens
Budget: ≤3 files per PR; src/construction/ + book only
Verify: cargo test -p proc_A_dine01 --lib corridor_construction transport::persistence
Do NOT: hand-edit construction_stage_live.json; preview mutates book
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **R4-PLAN-001** signed — corridor phase + witness schema |
