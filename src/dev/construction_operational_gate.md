# CONSTRUCTION_OPERATIONAL_GREEN

Separate from Stage 5 **FULL_APP**. Proves the construction **spine** works in the running app after Phase 2 cleanup.

**Board:** [`construction_operational_todos.rs`](construction_operational_todos.rs)  
**Proof:** extend `debug_runs/construction_stage_live.json` with `operational` + `invariants` sections.

## Requirements

| # | Requirement | Witness row |
|---|-------------|-------------|
| 1 | Toolbox opens; hierarchical categories; tool stays active across commits | `CONSTRUCTION-OP-01` |
| 2 | Road path: LMB → Shift+LMB commit → transport/executed network updates | `CONSTRUCTION-OP-02` |
| 3 | Zone paint: drag → auto-queue or Shift batch → strategic `Zone` on confirm | `CONSTRUCTION-OP-03` |
| 4 | Building: catalog pick (e.g. Duplex) → ghost → confirm → site entity | `CONSTRUCTION-OP-04` |
| 5 | Demolish: pick → pending → confirm → despawn | `CONSTRUCTION-OP-05` |
| 6 | Ctrl+Z undoes last road/site/zone commit | `CONSTRUCTION-OP-06` |
| 7 | `construction_stage_live.json` written in sim with board snapshots | `CONSTRUCTION-OP-07` |
| 8 | No legacy paths: `rg` clean for `gui::build`, tile-road intent, fake demolish queue | `CONSTRUCTION-OP-08` |

## When to run

After **Phase 2 P6–P8** are green; before treating **Round 3** catalog/topology as production-ready.

## Plain English

Playable construction loop in sim: pick tool, preview, validate, commit, undo — measurable in JSON, not fixture-only.
