//! Material / tag / rule unification — see `prompts/guides/terrain_unification_runbook_v1.md`.

mod registry;
mod resolver;
pub mod dependency;
pub mod profile;
pub mod runtime;
mod rules;
mod tags;

pub use registry::{
    family_default_material_def, MaterialDef, MaterialId, MaterialRegistry, MaterialRegistryLoader,
};
pub use resolver::{resolve_material, resolve_material_with_rule_index};
pub use rules::{MaterialRule, RuleSet, RuleSetLoader};
pub use runtime::{MaterializedChunk, MaterializedResources};
pub use tags::{TagDef, TagId, TagRegistry, TagRegistryLoader, TagSet};

pub use dependency::{
    compute_chunk_dependency, hash_asset, hash_material_registry, hash_pass1_bucket, hash_rule_set,
    hash_tag_registry,
    hash_tuning_bucket, lowest_dirty_pass, source_noise_id_for_chunk, ChunkDependency, ChunkDirty,
    DIRTY_ALL, DIRTY_PASS1, DIRTY_PASS2, DIRTY_PASS3, DIRTY_PASS4, DIRTY_PASS5, DIRTY_PASS6,
    DIRTY_PASSES_2_THROUGH_6,
};
pub use profile::{apply_profile, ProfileHandles, WorldProfile, WorldProfileLoader, WorldProfileSelector};

#[cfg(feature = "dev_tools")]
pub use runtime::RuleTrace;
