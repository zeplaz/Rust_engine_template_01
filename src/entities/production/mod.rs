pub mod aluminum;
pub mod concrete;
pub mod core;
pub mod power;

// Legacy modules intentionally left on disk and excluded from active wiring:
// - prod_comps.rs
// These are kept as migration reference for subsequent incremental passes.

pub use aluminum::*;
pub use concrete::*;
pub use core::*;
pub use power::*;

