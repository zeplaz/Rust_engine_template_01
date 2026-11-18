//! Single SystemParam for gameplay water queries (PLAN-WATER-DEPTH-TRUTH-v1 W1).
//! Order: live field depth → runtime → world static depth → catalog last resort.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::render::fx_spine::water_surface_visual::{WaterSurfaceKind, WaterSurfaceVisualCatalog};

use super::static_depth::WorldStaticWaterDepth;
use super::surface_water::SurfaceWaterField;
use super::water_runtime::{tile_is_wet, WaterClassRuntime};

#[derive(SystemParam)]
pub struct WaterTileProbe<'w> {
    pub field: Option<Res<'w, SurfaceWaterField>>,
    pub runtime: Option<Res<'w, WaterClassRuntime>>,
    pub static_depth: Option<Res<'w, WorldStaticWaterDepth>>,
    pub catalog: Option<Res<'w, WaterSurfaceVisualCatalog>>,
}

impl WaterTileProbe<'_> {
    #[must_use]
    pub fn available(&self) -> bool {
        self.field.is_some()
            || self.runtime.is_some()
            || self.static_depth.is_some()
            || self.catalog.is_some()
    }

    #[must_use]
    pub fn is_water(&self, x: i32, z: i32) -> bool {
        tile_is_wet(
            self.field.as_deref(),
            self.runtime.as_deref(),
            self.static_depth.as_deref(),
            x,
            z,
            |x, z| {
                let Some(catalog) = self.catalog.as_deref() else {
                    return false;
                };
                if x < 0 || z < 0 {
                    return false;
                }
                catalog.tile_kind(x as u32, z as u32) != WaterSurfaceKind::None
            },
        )
    }
}
