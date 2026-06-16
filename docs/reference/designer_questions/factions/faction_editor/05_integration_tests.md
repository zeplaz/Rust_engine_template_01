# Faction editor — integration tests `05`

## Headless / CI

1. **Round-trip:** load minimal scenario with 2–3 blueprints + graph → save → reload → assert equality of Serializable slice (ids, stances, tags).
2. **Import/export:** export one `.ron` blueprint, import into empty scenario, assert match modulo timestamps.
3. **Migration:** old `schema_version` fixture → migrator → new schema loads.

## Manual / playtest hooks

- Snapshot diplomacy graph before/after scripted event (war declaration) 📎.

## Repo placement 📎

- Prefer `tests/` integration test with minimal `App` + serialization stubs, or feature-gated harness mirroring production save tests.
