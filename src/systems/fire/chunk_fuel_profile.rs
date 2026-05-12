//! Chunk-level **fuel profile** (grass / brush / canopy layers) — bridges meso [`VegetationField`](crate::systems::ecology::VegetationField) to [`terrain::fire`](crate::terrain::fire) taxonomy (`base_fire_sim.md`).

use bevy::prelude::*;

use crate::systems::ecology::VegetationField;
use crate::terrain::fire::{fuel_material_def, FuelLayer, FuelMaterialKind, VegetationFuelLayer};
use crate::terrain::generation::Chunk;

/// Layered wildland fuels for a chunk (uniform until subcell fuel lands).
///
/// Optional [`FuelLayer::merge_structure_overlay`] via `structure_overlay` lets scenarios or editors
/// stamp industrial / urban fuel without replacing vegetation strata (`sim-fuel-1`).
#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkFuelProfile {
    pub grass: VegetationFuelLayer,
    pub brush: VegetationFuelLayer,
    pub canopy: VegetationFuelLayer,
    pub peat_depth: f32,
    pub suppression_difficulty: f32,
    /// When set, merged into [`Self::to_fuel_layer`] for emitters / smoke tint / logistics samples.
    pub structure_overlay: Option<FuelLayer>,
}

impl ChunkFuelProfile {
    /// Unified scalar row for VFX / atmosphere hints (see [`FuelLayer`]).
    #[inline]
    pub fn to_fuel_layer(&self) -> FuelLayer {
        let wild = FuelLayer::from_vegetation_strata(&self.grass, &self.brush, &self.canopy);
        self.structure_overlay
            .map(|s| FuelLayer::merge_structure_overlay(wild, s))
            .unwrap_or(wild)
    }
}

impl Default for ChunkFuelProfile {
    fn default() -> Self {
        Self {
            grass: VegetationFuelLayer {
                live_biomass: 0.2,
                dead_biomass: 0.12,
                moisture: 0.42,
                ignition_bias: 0.55,
                fuel_kind: FuelMaterialKind::Grass,
            },
            brush: VegetationFuelLayer {
                live_biomass: 0.15,
                dead_biomass: 0.1,
                moisture: 0.4,
                ignition_bias: 0.5,
                fuel_kind: FuelMaterialKind::Brush,
            },
            canopy: VegetationFuelLayer {
                live_biomass: 0.18,
                dead_biomass: 0.14,
                moisture: 0.38,
                ignition_bias: 0.45,
                fuel_kind: FuelMaterialKind::Timber,
            },
            peat_depth: 0.0,
            suppression_difficulty: 0.35,
            structure_overlay: None,
        }
    }
}

/// Map continuous vegetation structure into discrete fuel layers (CPU, cheap).
pub fn chunk_fuel_profile_from_vegetation(veg: &VegetationField) -> ChunkFuelProfile {
    let mut p = ChunkFuelProfile::default();
    let cd = veg.canopy_density.clamp(0.0, 1.0);
    let us = veg.understory_density.clamp(0.0, 1.0);
    let gf = veg.ground_fuel.clamp(0.0, 1.0);
    let dry = veg.dryness.clamp(0.0, 1.0);

    p.grass.live_biomass = (gf * (1.0 - us * 0.65)).max(0.05);
    p.grass.dead_biomass = (gf * dry * 0.85).max(0.02);
    p.grass.moisture = (1.0 - dry) * 0.45 + 0.12;
    p.grass.ignition_bias = 0.45 + dry * 0.4;
    p.grass.fuel_kind = FuelMaterialKind::Grass;

    p.brush.live_biomass = (us * (1.0 - cd * 0.5)).max(0.02);
    p.brush.dead_biomass = (us * dry * 0.7).max(0.02);
    p.brush.moisture = (1.0 - dry) * 0.42 + 0.1;
    p.brush.ignition_bias = 0.42 + dry * 0.38;
    p.brush.fuel_kind = FuelMaterialKind::Brush;

    p.canopy.live_biomass = (cd * 0.95).max(0.02);
    p.canopy.dead_biomass = (cd * dry * 0.55).max(0.02);
    p.canopy.moisture = (1.0 - dry) * 0.38 + 0.08;
    p.canopy.ignition_bias = 0.38 + dry * 0.42;
    p.canopy.fuel_kind = if cd > 0.72 {
        FuelMaterialKind::Timber
    } else {
        FuelMaterialKind::Brush
    };

    p.peat_depth = (veg.fragmentation * 0.15 + dry * 0.08).min(1.0);
    p.suppression_difficulty = (veg.fragmentation * 0.55 + dry * 0.35 + veg.fuel_load * 0.25).clamp(0.0, 1.0);

    if p.peat_depth > 0.22 {
        p.grass.fuel_kind = FuelMaterialKind::Peat;
        let d = fuel_material_def(FuelMaterialKind::Peat);
        p.grass.ignition_bias = (p.grass.ignition_bias * 0.65 + d.burn_energy * 0.2).min(1.0);
    }

    p
}

pub(crate) fn spawn_chunk_fuel_profile_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkFuelProfile>)>,
) {
    for e in &q {
        commands.entity(e).insert(ChunkFuelProfile::default());
    }
}

pub(crate) fn chunk_fuel_profile_tick(mut q: Query<(&VegetationField, &mut ChunkFuelProfile)>) {
    for (veg, mut profile) in &mut q {
        let overlay = profile.structure_overlay;
        *profile = chunk_fuel_profile_from_vegetation(veg);
        profile.structure_overlay = overlay;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_fuel_layer_is_sane() {
        let p = ChunkFuelProfile::default();
        let row = p.to_fuel_layer();
        assert!(row.surface_fuel > 0.2 && row.surface_fuel <= 1.0);
        assert!(row.moisture > 0.2 && row.moisture < 0.95);
    }

    #[test]
    fn structure_overlay_raises_toxic_vs_wildland_only() {
        let mut p = ChunkFuelProfile::default();
        let base = p.to_fuel_layer();
        p.structure_overlay = Some(FuelLayer::fuel_dump());
        let merged = p.to_fuel_layer();
        assert!(merged.toxic_smoke >= base.toxic_smoke);
        assert!(merged.toxic_smoke > 0.5);
    }
}
