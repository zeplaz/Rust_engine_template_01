# PLAN-CONSTRUCTION-R4-MV-EXEC-001 — R4 MV ghost execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-R4-MV-EXEC-001** |
| **Prior / parent** | `R4-PLAN-002` — [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) |
| **Paired corridor spec** | `R4-PLAN-001` — [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Coder lane** | **R4-MV-GHOST-001** (product gate) |
| **Status** | **READY (planner finalized)** — coder lane `R4-MV-GHOST-001` |

**Planner sign-off:** PASS (2026-05-27). Queue: `tools/orchestrator/queues/planner_active_queue.json` → archived `PLAN-CONSTRUCTION-R4-MV-EXEC-001`.

---

## Coder handoff (acceptance)

| Field | Value |
|:---|:---|
| **Witness** | `debug_runs/construction_stage_live.json` → `construction_r4_mv_ghost_001` |
| **Unblocks** | `R4-MV-GHOST-001` |
| **Green rollup** | `construction_r4_mv_ghost_001.green` := `mv_001_still_green` AND `corridor_overlay_tokens_wired` AND `legend_wired` |
| **Verify** | `cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json construction_mv` |

---

## Scope

Implement the Round 4 **multiview ghost preset delta** so the SimulationMap and WorldMain surfaces render a corridor phase overlay + a tray legend keyed to the design token table.

The plan also wires a witness block:
`debug_runs/construction_stage_live.json` → `construction_r4_mv_ghost_001`

---

## Authority map (single writer per resource)

| Resource | Single writer | Allowed mutation | Must NOT be second-written by |
|:---|:---|:---|:---|
| Ghost token table | `src/construction/ghost_visual.rs` | define corridor phase RGBA / stroke/pattern tokens | UI overlays drawing different colors |
| Multiview overlay pass | the overlay render path used by construction MV ghosts | draw planned/in-progress/completed overlays on correct surfaces only | construction-stage sim writer; minimap overlay writer |
| Tray legend UI | construction tray legend writer (egui/overlay) | render legend swatches + labels aligned to token table | any debug-only overlay |
| `debug_runs/construction_stage_live.json` witness | sim writer extension of `write_construction_live_proof_system` | add `construction_r4_mv_ghost_001` block | manual JSON edits |

---

## Task list (≤3 files per PR)

### C1 — Extend ghost visuals/token language for corridor phases
1. Add token definitions corresponding to R4 corridor phase swatches:
   - `corridor_planned`
   - `corridor_in_progress`
   - reuse `road_committed_color()` for committed
2. Keep a single code source so MV presets and overlay use identical tokens.

Files (≤3):
- `src/construction/ghost_visual.rs`
- `src/construction/visual_authority.rs` (if token routing needs updates)

### C2 — Overlay pass: render corridor phase on MV surfaces
1. Draw corridor phase overlay **on top of terrain** and **under tool ghosts** per spec.
2. Ensure correct surface scope:
   - SimulationMap: overlay visible
   - WorldMain: overlay visible
   - World Preview: overlay NOT shown
   - Minimap: heat only (no polylines)
3. Ensure overlay respects invalid/overlap behavior:
   - corridor overlay wins on committed topology
   - invalid/valid road ghosts remain visible for uncommitted picks

Files (≤3):
- `src/construction/visual_authority.rs`
- `src/construction/mod.rs`
- `src/strategic/construction_book.rs`

### C3 — Tray legend: wire phase key shown in construction tool footer
1. Ensure legend renders when corridor rows are not completed OR road tool is active.
2. Legend labels must match design intent (`Planned`, `Building`, `Open` / phase key).

Files (≤3):
- `src/construction/mod.rs`
- `src/gui/hud/stage7_ui_shell.rs` (only if legend is owned there)

### C4 — Witness wiring: add `construction_r4_mv_ghost_001` block
1. Extend `src/construction/live_proof.rs` to write the witness block with:
   - `/gate == "DESIGN-R4-MV-001"`
   - rollup logic using `construction_mv_001` green + token wiring + legend wiring.
2. The witness must not force green when product gate is closed; it should stay blocked/false until the product gate lifts.

Files (≤3):
- `src/construction/live_proof.rs`
- `src/construction/construction_stage_witness.rs` (only if witness rollups are centralized)

---

## Witness JSON schema (must match `R4-PLAN-002`)

**File:** `debug_runs/construction_stage_live.json`  
**Witness:** `construction_r4_mv_ghost_001`

Fields:
- `/construction_r4_mv_ghost_001/gate: string` (`"DESIGN-R4-MV-001"`)
- `/construction_r4_mv_ghost_001/green: bool` rollup
- `/construction_r4_mv_ghost_001/corridor_overlay_tokens_wired: bool`
- `/construction_r4_mv_ghost_001/legend_wired: bool`
- `/construction_r4_mv_ghost_001/mv_001_still_green: bool` (must reflect `construction_mv_001.green: true`)

Rollup:
```text
construction_r4_mv_ghost_001.green :=
  mv_001_still_green
  AND corridor_overlay_tokens_wired
  AND legend_wired
```

---

## Verification (required test commands)

```powershell
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
cargo test -p proc_A_dine01 --lib construction_mv
```

Expected evidence:
- `construction_mv_001.green: true` remains true
- `construction_r4_mv_ghost_001.green` becomes true only when wiring is present (and product gate lifted for the overall lane)

---

## Anti-patterns / do-not-reopen list (R4 MV ghost)

Do NOT:
- hand-edit construction stage witness JSON
- add a second ghost token/color source (single code source only)
- introduce corridor overlays on World Preview or minimap polylines
- re-open `R4-PLAN-002` design token spec; this plan only executes the existing signed preset delta
- re-open any F7-A/B/C exit gates or dual-queue / steward preflights

