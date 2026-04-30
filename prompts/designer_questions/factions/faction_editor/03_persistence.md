# Faction editor — persistence `03`

**Pair:** `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md`.

## Three reservoirs

1. **Scenario / worldgen seed:** authored blueprints + initial stance graph embedded in scenario blob (hash-stable for MP).
2. **Save game:** full runtime faction roster + graph + any mid-game deltas (create/retire, stance history 📎).
3. **Loose assets:** `assets/...` RON blueprints for mods and AI archetypes (FE-06); referenced by id from scenario/save.

## Dynamic roster (matrix § v1)

- **Create/retire** in tools or sim must produce **atomic save deltas** or explicit commands (MP: server authoritative — `implementation_questions_v1.md` §9).

## Tech / research overlays

- Prefer **runtime overlay** serialized inside save (or delta log) rather than mutating authored `FactionBlueprint` files on disk — keeps mods reproducible 📎.

## Migration

- Bump `schema_version` when adding fields to `FactionBlueprint` or edge types; document one-hop migrators in serialization matrix style.
