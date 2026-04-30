//! LEGACY SHIM — prefer explicit paths:
//! - `crate::entities::production::*`
//! - `crate::entities::vehicles::*`
//! - `crate::entities::types::*` / `crate::entities::types_of::*` (shim)
//!
//! Do not add new wildcard re-exports here.

pub use crate::entities::production::*;
pub use crate::entities::vehicles::*;
