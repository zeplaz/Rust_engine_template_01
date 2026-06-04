//! Urban zoning classes — separate from strategic influence [`Zone`](crate::strategic::Zone).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ZoningClass {
    #[default]
    Residential,
    Commercial,
    Industrial,
    MixedUse,
    Civic,
    Rural,
}
