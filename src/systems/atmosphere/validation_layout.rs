//! Named regions for the **atmosphere validation** map (`atm-scene-1a`, `base_fire2_smoke.md` part 2).
//!
//! Tile space matches the CLI test harness default (`WorldGenParams` 256×256). Coordinates are **inclusive**
//! `min..=max` in tile indices `(x, y)` with `x` east and `y` south if your map uses screen-down Y — adjust
//! producers if your world origin differs.

use bevy::prelude::IVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtmosphereValidationRegion {
    pub id: &'static str,
    pub min_tile: IVec2,
    pub max_tile: IVec2,
    /// Human intent for scenario tooling / future material paint hints.
    pub intent: &'static str,
}

/// Layout v1 — coarse rectangles for scripted weather/fire/atmosphere tests.
pub static ATMOSPHERE_VALIDATION_LAYOUT_V1: &[AtmosphereValidationRegion] = &[
    AtmosphereValidationRegion {
        id: "mountain_ridge",
        min_tile: IVec2::new(0, 0),
        max_tile: IVec2::new(255, 31),
        intent: "ridge wind + smoke column visualization",
    },
    AtmosphereValidationRegion {
        id: "dense_forest",
        min_tile: IVec2::new(0, 32),
        max_tile: IVec2::new(255, 95),
        intent: "crown fire + ember transport",
    },
    AtmosphereValidationRegion {
        id: "grassland",
        min_tile: IVec2::new(0, 96),
        max_tile: IVec2::new(255, 143),
        intent: "fast low-intensity surface fire",
    },
    AtmosphereValidationRegion {
        id: "fuel_depot",
        min_tile: IVec2::new(0, 144),
        max_tile: IVec2::new(31, 175),
        intent: "explosion + toxic smoke plume",
    },
    AtmosphereValidationRegion {
        id: "ammo_dump",
        min_tile: IVec2::new(32, 144),
        max_tile: IVec2::new(63, 175),
        intent: "cookoff chain",
    },
    AtmosphereValidationRegion {
        id: "urban_block",
        min_tile: IVec2::new(64, 144),
        max_tile: IVec2::new(255, 191),
        intent: "structure fire propagation",
    },
    AtmosphereValidationRegion {
        id: "fog_basin",
        min_tile: IVec2::new(0, 192),
        max_tile: IVec2::new(127, 223),
        intent: "visibility + fog layering",
    },
    AtmosphereValidationRegion {
        id: "wetland",
        min_tile: IVec2::new(128, 192),
        max_tile: IVec2::new(255, 255),
        intent: "fire suppression + moisture barrier",
    },
];

#[inline]
pub fn tile_in_any_validation_region(tile: IVec2) -> Option<&'static AtmosphereValidationRegion> {
    ATMOSPHERE_VALIDATION_LAYOUT_V1.iter().find(|r| {
        tile.x >= r.min_tile.x
            && tile.y >= r.min_tile.y
            && tile.x <= r.max_tile.x
            && tile.y <= r.max_tile.y
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_defines_multiple_regions() {
        assert!(ATMOSPHERE_VALIDATION_LAYOUT_V1.len() >= 4);
    }

    #[test]
    fn center_tile_maps_to_grassland_or_adjacent() {
        let t = IVec2::new(128, 120);
        let hit = tile_in_any_validation_region(t);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().id, "grassland");
    }
}
