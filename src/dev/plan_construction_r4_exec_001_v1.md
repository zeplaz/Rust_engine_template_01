# PLAN-CONSTRUCTION-R4-EXEC-001 — R4 corridor execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-R4-EXEC-001** |
| **Prior / parent** | `R4-PLAN-001` — [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Coder lane** | **R4-CORRIDOR-001** (product gate) |
| **Status** | queued (exec plan, not witness close) |

---

## Scope

Implement the Round 4 **corridor (transport-edge)** construction phase lifecycle as codable, testable, and witness-backed behavior.

This plan expands the signed spec into a phased implementation sequence:
- Phase tags: `R4-C1..C5`
- Writer wiring: `construction_stage_live.json` → `construction_r4_corridor_001`
- Minimap construction heat cross-check respects `traffic_factor < 1`

---

## Authority map (single writer per resource)

| Resource | Single writer | Allowed mutation | Must NOT be second-written by |
|:---|:---|:---|:---|
| `CorridorConstructionBook` | `align_corridor_book_with_transport_directory` + `CorridorConstructionBook::plan_edge` + `advance_corridor_construction_book_on_sim_tick` | insert `Planned`, update existing rows (`progress`, `phase`) | render/minimap extraction; preview-only systems |
| R8 persistence slice (`TransportNetworkSnapshot.construction`) | `transport_construction_records_from_book` + `apply_corridor_book_from_transport_snapshot` | embed / hydrate `phase` strings + `progress` | map-editor Save code writing directly without book |
| `debug_runs/construction_stage_live.json` | `write_construction_live_proof_system` extension (sim writer) | add `construction_r4_corridor_001` witness block | manual JSON editing |
| Minimap construction heat | `fill_construction_heat_from_book` (render pipeline) | dim construction heat when `traffic_factor(edge) < 1` | any other heat writer |

---

## Phase/task list (R4-C1..C5)

Each numbered task must keep changes localized (≤3 files per PR).

### R4-C1 — Map editor save/load embeds corridor construction slice
1. Confirm R8 save path includes the corridor construction slice from the **live** book (not a seed stub).
2. Ensure `TransportNetworkSnapshot.construction` roundtrips `phase` string + `progress`.

Files (≤3):
- `src/gui/editor/map_editor/mod.rs`
- `src/systems/transport/persistence.rs`
- `src/strategic/transport_bridge.rs`

### R4-C2 — Sim writer wires `construction_r4_corridor_001` witness block
1. Extend `src/construction/live_proof.rs` to write a `construction_r4_corridor_001` block.
2. The block must include the signed witness pointers and rollup:
   - `/construction_r4_corridor_001/gate == "R4-CORRIDOR-001"`
   - `/construction_r4_corridor_001/green` rollup = product gate + sim tick writer + roundtrip ok + phase visual wired + book counters match.
3. Add a `product_board_open` witness field that flips to `true` only after the product gate is lifted.

Files (≤3):
- `src/construction/live_proof.rs`
- `src/strategic/construction_book.rs` (for book counters / phase mapping)
- `debug_runs/construction_stage_live.json` is output only; do not edit by hand

### R4-C3 — Corridor phase debug reads active edge phases (read-only)
1. Add read-only debug visibility of corridor phase on active edges (no book mutation).
2. If the witness includes an optional `corridor_phase_visual_wired` field, ensure it is driven by designer tokens/preset readiness (R4-PLAN-002).

Files (≤3):
- `src/construction/phase_visual.rs`
- `src/construction/construction_pipeline.rs`
- `src/strategic/construction_book.rs`

### R4-C4 — Construction tool plans corridor, no instant Completed cheat
1. Ensure construction tool “plan corridor” path creates/updates `CorridorConstructionBook` rows with:
   - `Planned` inserted on new edge commit
   - `InProgress` via sim tick writer only (no wall-clock progression)
   - `Completed` only after phase evolution in the book
2. Validate that “Completed” does not jump instantly on player road commit.

Files (≤3):
- `src/construction/construction_pipeline.rs`
- `src/strategic/construction_book.rs`
- `src/construction/build_commit.rs`

### R4-C5 — Minimap construction heat respects `traffic_factor < 1`
1. Cross-check that minimap construction heat dims when corridor `traffic_factor < 1`.
2. Do not add a second extraction/heat writer; use the existing `CorridorConstructionBook → construction_heat` binding.

Files (≤3):
- `src/render/minimap_compositor/composite.rs`
- `src/strategic/construction_book.rs`
- `src/gui/hud/diagnostics_ui.rs` (only if needed for a debug/readout)

---

## Witness JSON schema (must match `R4-PLAN-001`)

**File:** `debug_runs/construction_stage_live.json`  
**Writer:** extension of `write_construction_live_proof_system` (sim writer)

`construction_r4_corridor_001` pointers:
- `/construction_r4_corridor_001/gate: string` (`"R4-CORRIDOR-001"`)
- `/construction_r4_corridor_001/green: bool` (rollup below)
- `/construction_r4_corridor_001/product_board_open: bool` (true only after product gate)
- `/construction_r4_corridor_001/book_row_count: number` (>=0)
- `/construction_r4_corridor_001/planned_count: number`
- `/construction_r4_corridor_001/in_progress_count: number`
- `/construction_r4_corridor_001/completed_count: number`
- `/construction_r4_corridor_001/sim_tick_writer_wired: bool` (advance writer registered)
- `/construction_r4_corridor_001/r8_roundtrip_ok: bool` (lib snapshot ↔ book)
- `/construction_r4_corridor_001/corridor_phase_visual_wired: bool` (R4-PLAN-002 tokens landed)

Rollup (product gate open):
```text
construction_r4_corridor_001.green :=
  product_board_open
  AND sim_tick_writer_wired
  AND r8_roundtrip_ok
  AND (planned_count + in_progress_count + completed_count) >= book_row_count
  AND corridor_phase_visual_wired
```

---

## Verification (required test commands)

Run these after each ≤3-file slice:
```powershell
cargo test -p proc_A_dine01 --lib corridor_construction
cargo test -p proc_A_dine01 --lib transport::persistence
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

---

## Anti-patterns / do-not-reopen list (R4 corridor)

Do NOT:
- hand-edit `debug_runs/construction_stage_live.json` to flip `green`
- progress corridor phase by wall-clock time; progress must come from the sim tick writer
- introduce a second writer for corridor phase in render/minimap extraction
- re-open any F7-A/B/C exit gates or dual-queue / steward preflights (regression only)

