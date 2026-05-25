//! Standing **surface water** vs **atmospheric moisture** for fire.
//!
//! - [`ChunkCellMatrix::moisture`] — humidity / rainfall / dryness proxy on **burnable** land.
//! - Standing water (lake, river, flooded tile) — no combustion regardless of moisture; a dry
//!   climate does not burn open water.

use bevy::prelude::*;

use crate::terrain::biome::BiomeTuning;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::ChunkCellMatrix;
use crate::terrain::material::{TagId, TagRegistry};

/// Per-tile standing-water test shared by overlay, ember, and scalar chunk fire.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SurfaceWaterFireGate {
    /// Open water when `elevation <= water_line` (matches hydrology / shallow water band).
    pub water_line: f32,
    /// Optional `flooded` tag from P4 hydrology (rivers/lakes on non-depressed cells).
    pub flooded_tag: Option<TagId>,
}

impl Default for SurfaceWaterFireGate {
    fn default() -> Self {
        let tuning = BiomeTuning::default();
        Self {
            water_line: tuning.shallow_water_height_max,
            flooded_tag: None,
        }
    }
}

impl SurfaceWaterFireGate {
    #[inline]
    pub fn cell_has_standing_water(&self, matrix: &ChunkCellMatrix, i: usize) -> bool {
        if i >= matrix.elevation.len() {
            return false;
        }
        if matrix.elevation[i] <= self.water_line {
            return true;
        }
        if let Some(tag) = self.flooded_tag {
            if matrix.tags.get(i).is_some_and(|t| t.contains(tag)) {
                return true;
            }
        }
        false
    }

    /// Dryness from humidity proxy only (land cells); not used on standing water.
    #[inline]
    pub fn atmospheric_dryness(moisture: f32) -> f32 {
        (0.42 - moisture).max(0.0)
    }
}

pub fn init_surface_water_fire_gate(
    mut commands: Commands,
    params: Option<Res<WorldGenParams>>,
) {
    let water_line = params
        .as_ref()
        .map(|p| p.biome_tuning.shallow_water_height_max)
        .unwrap_or_else(|| BiomeTuning::default().shallow_water_height_max);
    let flooded_tag = load_flooded_tag_id();
    commands.insert_resource(SurfaceWaterFireGate {
        water_line,
        flooded_tag,
    });
}

fn load_flooded_tag_id() -> Option<TagId> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/config/terrain/tag_registry.example.json");
    let path = path.to_str()?;
    TagRegistry::load_from_json(path)
        .ok()
        .and_then(|reg| reg.tag_id("flooded"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;
    use crate::terrain::material::TagSet;

    #[test]
    fn standing_water_by_elevation_not_moisture() {
        let gate = SurfaceWaterFireGate {
            water_line: 0.35,
            flooded_tag: None,
        };
        let mut matrix = ChunkCellMatrix::new(UVec2::ONE);
        matrix.elevation[0] = 0.1;
        matrix.moisture[0] = 0.05;
        assert!(gate.cell_has_standing_water(&matrix, 0));
        matrix.elevation[0] = 0.9;
        matrix.moisture[0] = 0.05;
        assert!(!gate.cell_has_standing_water(&matrix, 0));
    }

    #[test]
    fn flooded_tag_marks_standing_water() {
        let flooded = TagId(7);
        let gate = SurfaceWaterFireGate {
            water_line: 0.35,
            flooded_tag: Some(flooded),
        };
        let mut matrix = ChunkCellMatrix::new(UVec2::ONE);
        matrix.elevation[0] = 0.9;
        matrix.moisture[0] = 0.9;
        matrix.tags[0] = TagSet::default();
        assert!(!gate.cell_has_standing_water(&matrix, 0));
        matrix.tags[0].insert(flooded);
        assert!(gate.cell_has_standing_water(&matrix, 0));
    }
}
