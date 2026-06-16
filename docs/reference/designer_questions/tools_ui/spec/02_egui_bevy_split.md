# Tools UI — egui vs Bevy split `02`

**Canonical:** `prompts/guides/ui_boundary_guide_v1.md`.

## Rules of thumb

- **Simulation truth** lives in ECS/resources — both UIs **read** the same sources; **writes** go through commands/events.
- **egui** for dense tables, manifest viewers, permissions matrices.
- **Bevy UI** for diegetic overlays when polish matters.

## Overlap rows

- **Perf HUD** sparklines: texture ring vs egui embed — `implementation_questions_v1.md` §7.
