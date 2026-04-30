# Production — data model & manifest `01`

## Manifest

- **`ProductionManifest`** keys and `tools_plugin` strings — source of truth `src/systems/production/manifest.rs`; do not drift in docs.
- New domain: add row + serde types + schedule ownership in one PR-style batch.

## serde boundaries

- Which fields on **ElectricalNode** / grid components are snapshot vs runtime-only — answer in `implementation_questions_v1.md` §1–3.

## Faction tint / ownership overlays

- Use same faction id source as `factions/spec/` when wiring power traces or facility ownership (`implementation_questions_v1.md` §11).
