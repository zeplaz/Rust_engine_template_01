//! The one ground-display material resolution shared by the CPU raster
//! (`render/tile_world_fallback.rs`) and the GPU atlas→world bake
//! (`render/pipelines/terrain_gpu_bake_spike.rs`).
//!
//! * A cell's display material is the registry default of its terrain family.
//! * Water families (`DeepWater` / `ShallowWater`) never texture the ground: the ground under water
//!   is bed material (the `Beach` family default, sand). Only the GPU water field draws water
//!   (`plan_final_gpu_presentation_v1`, `plan_water_depth_truth_v1`).

use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};
use crate::terrain::material::{family_default_material_def, MaterialRegistry};

/// Family whose default material is the riverbed / lakebed ground texture.
pub const BED_FAMILY_NAME: &str = "Beach";

/// True for the water look-labels (`DeepWater` / `ShallowWater`).
#[must_use]
pub fn is_water_family(families: &TerrainFamilyRegistry, family: TerrainFamilyId) -> bool {
    families
        .def(family)
        .is_some_and(|d| d.name == "DeepWater" || d.name == "ShallowWater")
}

/// Family used to pick the **ground** material: water families map to the bed family.
#[must_use]
pub fn ground_display_family(
    families: &TerrainFamilyRegistry,
    family: TerrainFamilyId,
) -> TerrainFamilyId {
    if is_water_family(families, family) {
        families.id(BED_FAMILY_NAME).unwrap_or(family)
    } else {
        family
    }
}

/// Registry row (= atlas layer) of the ground material for `family`; `None` when the registry has
/// no material for the (bed-mapped) family.
#[must_use]
pub fn ground_display_material_index(
    reg: &MaterialRegistry,
    families: &TerrainFamilyRegistry,
    family: TerrainFamilyId,
) -> Option<u32> {
    let fam = ground_display_family(families, family);
    let def = family_default_material_def(reg, fam)?;
    reg.materials
        .iter()
        .position(|m| std::ptr::eq(m, def))
        .map(|i| i as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load() -> (MaterialRegistry, &'static TerrainFamilyRegistry) {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let reg = MaterialRegistry::load_from_json(
            root.join("assets/config/terrain/material_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        (reg, crate::terrain::default_terrain_families())
    }

    #[test]
    fn water_family_resolves_to_non_water_bed_material() {
        let (reg, fams) = load();
        for name in ["DeepWater", "ShallowWater"] {
            let id = fams.id(name).unwrap();
            let idx = ground_display_material_index(&reg, fams, id).unwrap() as usize;
            let def = &reg.materials[idx];
            assert!(
                !is_water_family(fams, def.family),
                "{name} ground resolved to water material {}",
                def.name
            );
            assert_eq!(def.name, "sand_dry");
        }
    }

    #[test]
    fn land_family_resolves_to_family_default() {
        let (reg, fams) = load();
        let grass = fams.id("Grassland").unwrap();
        let idx = ground_display_material_index(&reg, fams, grass).unwrap() as usize;
        assert_eq!(reg.materials[idx].name, "loam_wet");
    }
}
