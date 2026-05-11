# Agent / authoring notes

## Serialization: RON vs JSON

- **Default for engine-owned serde assets** (terrain registries, transport R8, world-gen tuning, hybrid snapshot bodies, world subengine export): prefer **RON** on disk. Loaders use extension dispatch (`.ron` / `.json`; unknown extension often tries RON then JSON). Examples: `*.example.ron` beside legacy JSON where applicable.
- **JSON** is retained where **external tooling** or **human interchange** expects it (Python asset editor pages, some fixtures, explicit `.json` paths, HTTP APIs).
- **Documented in code** near loaders: `src/terrain/registry_serde_path.rs`, `src/terrain/generation/tuning_io.rs`, `src/systems/transport/persistence.rs`, `src/io/snapshot/mod.rs` (hybrid header may be JSON or **RON** line).

## Transport R8 + construction

- `TransportNetworkSnapshot` may include a **`construction`** slice (corridor phases). **G4** load hydrates [`CorridorConstructionBook`](src/strategic/construction_book.rs) when that resource is present. Map editor **Save** embeds book rows from the live graph.
