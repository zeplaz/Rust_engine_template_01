pub mod aluminum;
pub mod concrete;
pub mod core;
pub mod power;

// **CLN-P0-R8-001 / SCH-W1-P1-001** — obsolete legacy modules removed 2026-07-03:
// prod_comps.rs, aluminum/production_sys.rs, concrete/sys.rs (see ProductionManifest notes).

// Intentionally no `pub use submodule::*` — each subsystem has its own `components` / `systems`;
// merging them here caused ambiguous glob re-exports at `production::`.

