# Political territory on the world map `03`

**Pair:** `factions/faction_editor_tooling_matrix_v1.md`, `factions/faction_editor/README.md` (stances, treaties), diplomacy runtime.

## Representations 📎

- **Tile owner** vs **macro polygon** vs **chunk-owned bitmask** — pick one primary + migration path.
- **Strategic resources** / **exclusion zones** as overlay layers.

## Nav & production coupling

- **Closed border** → `navigation/spec/03_client_server_pathing.md` rejects edges.
- **Facility ownership** tint → `production_economy/spec/01_data_model_manifest.md`.

## Simulation LOD

- **Coarse political tick** for AI factions (`implementation_questions_v1.md` §10). Border redraw events rare vs per-frame.

## Display

- Map tint, border lines — Bevy HUD vs egui editor per `tools_ui` + UI boundary guide.
