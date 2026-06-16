# Production — tools UI (egui) `02`

**Pattern:** `src/systems/production/tools_ui.rs` — `ProductionToolsUiPlugin`, `ProductionToolsState`, domain enum.

## Panels (target)

- Domain picker (power / concrete / aluminum / manufacturing).
- Facility inspector: bind to ECS selection or entity id field 📎.
- Manifest **read-only** viewer for debugging registry drift.

## Split

- **egui** = devtools / operator deep inspection.
- **Bevy UI** = player-facing gauges — rules in `tools_ui/spec/` + `guides/ui_boundary_guide_v1.md`.

## Overlap

- Production **serialization debug** tab may live under `tools_ui/tooling_cross_domain_v1.md` row.
