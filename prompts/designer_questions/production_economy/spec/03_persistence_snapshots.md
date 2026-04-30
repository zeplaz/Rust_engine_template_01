# Production — persistence & snapshots `03`

**Matrix:** `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md`.

## Snapshots

- Version chunk per manufacturing domain (`implementation_questions_v1.md` §9).
- **Repair jobs** shape keyed by `EntityId` + blueprint/spares 📎 §10.

## Migration

- When adding a new `ProductionConfig` field, bump nested schema version and document migrator path.

## Code truth (2026)

- `ConcreteSerializationPlugin` / `AluminumSerializationPlugin` / `PowerSerializationPlugin` in `src/systems/production/serialization.rs` are **empty** until TODOs are implemented — do not document as working persistence.
- **`ProductionSerializationPlugin`** is still registered in `EnginePlugin` so hooks stay in the schedule graph.

## Overlap with world save

- Global resources vs per-chunk production 📎 — align with terrain chunk policy if facilities are tied to tiles.
