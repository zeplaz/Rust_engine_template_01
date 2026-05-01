//! ECS runtime types for materialized terrain chunks (U5).

use bevy::prelude::{Component, UVec2};

use super::MaterialId;
use crate::terrain::generation::MaterializedChunkData;

/// Resolved material id per cell for one chunk (pass 6 output, ECS form).
#[derive(Component, Clone, Debug)]
pub struct MaterializedChunk {
    pub size: UVec2,
    pub materials: Vec<MaterialId>,
}

/// Duplicate of [`MaterializedChunk`] cell indices for the resource tile layer (rules deferred).
#[derive(Component, Clone, Debug)]
pub struct MaterializedResources {
    pub ids: Vec<MaterialId>,
}

impl From<MaterializedChunkData> for MaterializedChunk {
    fn from(data: MaterializedChunkData) -> Self {
        Self {
            size: data.size,
            materials: data.materials,
        }
    }
}

#[cfg(feature = "dev_tools")]
/// Per-cell winning `rule_index` from [`super::rules::MaterialRule::rule_index`], or `u32::MAX` for family default.
#[derive(Component, Clone, Debug)]
pub struct RuleTrace {
    pub winners: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn materialized_chunk_from_data() {
        let data = MaterializedChunkData {
            size: UVec2::new(3, 4),
            materials: vec![MaterialId(0), MaterialId(1)],
        };
        let m: MaterializedChunk = data.into();
        assert_eq!(m.size, UVec2::new(3, 4));
        assert_eq!(m.materials.len(), 2);
        assert_eq!(m.materials[0], MaterialId(0));
    }
}
