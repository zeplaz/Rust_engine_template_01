//! Material / tag / rule unification — see `prompts/guides/terrain_unification_runbook_v1.md`.

mod registry;
mod resolver;
mod rules;
mod tags;

pub use registry::{MaterialDef, MaterialId, MaterialRegistry, MaterialRegistryLoader};
pub use resolver::resolve_material;
pub use rules::{MaterialRule, RuleSet};
pub use tags::{TagDef, TagId, TagRegistry, TagSet};
