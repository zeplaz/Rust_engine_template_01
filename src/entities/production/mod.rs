pub mod aluminum;
pub mod concrete;
pub mod core;
pub mod power;

// Legacy modules intentionally left on disk and excluded from active wiring:
// - prod_comps.rs
// These are kept as migration reference for subsequent incremental passes.

// Intentionally no `pub use submodule::*` — each subsystem has its own `components` / `systems`;
// merging them here caused ambiguous glob re-exports at `production::`.

