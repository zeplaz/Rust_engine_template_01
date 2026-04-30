// Core module — shared cross-cutting helpers.
//
// `id_generator.rs` retired (2026-04-26): replaced by `crate::idgen`
// (atomic `EntityId` with serde, single source of truth across the engine).
// New code MUST use `EntityId::new()` from `crate::idgen`; do not introduce a
// non-atomic counter struct.
