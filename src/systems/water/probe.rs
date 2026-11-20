//! The one read API for gameplay water (WM-4): fire gate, flood route blocks, buildings
//! (DISASTER-PROXY), stands, placement, cursor readout. Every consumer asks this param; none reads
//! `SurfaceWaterField` buffers, static depth, class runtime or the render catalog on its own.
//!
//! Truth is the map-sized [`SurfaceWaterField`]. Before the field is seeded (world still loading)
//! the probe answers from the world's static depth stamp it will be seeded from, then from the
//! render catalog; once the field is wired those fallbacks are never consulted again.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::render::fx_spine::water_surface_visual::{WaterSurfaceKind, WaterSurfaceVisualCatalog};

use super::static_depth::WorldStaticWaterDepth;
use super::surface_water::SurfaceWaterField;
use super::water_runtime::WET_DEPTH_M;

#[derive(SystemParam)]
pub struct WaterTileProbe<'w> {
    field: Option<Res<'w, SurfaceWaterField>>,
    static_depth: Option<Res<'w, WorldStaticWaterDepth>>,
    catalog: Option<Res<'w, WaterSurfaceVisualCatalog>>,
}

impl WaterTileProbe<'_> {
    /// Any water source exists (field, static stamp or catalog).
    #[must_use]
    pub fn available(&self) -> bool {
        self.field.is_some() || self.static_depth.is_some() || self.catalog.is_some()
    }

    /// Some source can already answer: the seeded field, a non-empty static stamp, or a catalog
    /// with a grid. Callers that must not act on "no data yet" wait for this.
    #[must_use]
    pub fn ready(&self) -> bool {
        self.seeded()
            || self.static_depth.as_deref().is_some_and(|s| s.width > 0 && s.height > 0)
            || self
                .catalog
                .as_deref()
                .is_some_and(|c| c.grid_width > 0 && c.grid_height > 0)
    }

    /// The live field is allocated: answers come from it only.
    #[must_use]
    pub fn seeded(&self) -> bool {
        self.field.as_deref().is_some_and(|f| f.wired)
    }

    /// Live depth (m) under `tile`: `Some` only while the seeded field covers it, so a caller can
    /// tell "dry" from "no live water yet".
    #[must_use]
    pub fn live_depth_m(&self, tile: IVec2) -> Option<f32> {
        self.field.as_deref()?.depth_at_world(tile.x, tile.y)
    }

    /// Water depth (m) under `tile`. Seeded field: live depth (0 outside the map). Before the
    /// seed: the static stamp, else 0.
    #[must_use]
    pub fn depth_m(&self, tile: IVec2) -> f32 {
        if self.seeded() {
            return self.live_depth_m(tile).unwrap_or(0.0);
        }
        self.static_depth
            .as_deref()
            .and_then(|s| s.depth_at_world(tile.x, tile.y))
            .unwrap_or(0.0)
    }

    /// Water surface height (world m) over `tile`; `None` unless the seeded field covers it.
    #[must_use]
    pub fn surface_m(&self, tile: IVec2) -> Option<f32> {
        self.field.as_deref()?.surface_at_world(tile.x, tile.y)
    }

    /// Depth-averaged flow velocity (m/s) over `tile`; zero when still, dry or not covered.
    #[must_use]
    pub fn flow(&self, tile: IVec2) -> Vec2 {
        self.field
            .as_deref()
            .and_then(|f| f.velocity_at_world(tile.x, tile.y))
            .unwrap_or(Vec2::ZERO)
    }

    /// `tile` is under at least `min_depth_m` of water. Before the field is seeded the static
    /// stamp (then the catalog) decides, ignoring `min_depth_m`.
    #[must_use]
    pub fn is_wet(&self, tile: IVec2, min_depth_m: f32) -> bool {
        if self.seeded() {
            return self.live_depth_m(tile).is_some_and(|d| d >= min_depth_m);
        }
        if let Some(wet) = self
            .static_depth
            .as_deref()
            .and_then(|s| s.is_wet_world(tile.x, tile.y))
        {
            return wet;
        }
        self.catalog_wet(tile.x, tile.y)
    }

    /// Gameplay "is water" ([`WET_DEPTH_M`]): placement, settlement siting, dam banks.
    #[must_use]
    pub fn is_water(&self, x: i32, z: i32) -> bool {
        self.is_wet(IVec2::new(x, z), WET_DEPTH_M)
    }

    fn catalog_wet(&self, x: i32, z: i32) -> bool {
        let Some(catalog) = self.catalog.as_deref() else {
            return false;
        };
        if x < 0 || z < 0 {
            return false;
        }
        catalog.tile_kind(x as u32, z as u32) != WaterSurfaceKind::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn probe_app(field: Option<SurfaceWaterField>) -> App {
        let mut app = App::new();
        if let Some(f) = field {
            app.insert_resource(f);
        }
        app
    }

    fn seeded_field() -> SurfaceWaterField {
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, 4, 4, vec![10.0; 16], 0.0);
        let i = f.idx(1, 1);
        f.depth_m[i] = 0.4;
        f
    }

    #[test]
    fn probe_reads_live_field_depth_surface_and_threshold() {
        let mut app = probe_app(Some(seeded_field()));
        app.world_mut()
            .run_system_once(|p: WaterTileProbe| {
                let t = IVec2::new(1, 1);
                assert!((p.depth_m(t) - 0.4).abs() < 1e-6);
                assert_eq!(p.surface_m(t), Some(10.4));
                assert!(p.is_wet(t, 0.1) && p.is_wet(t, WET_DEPTH_M) && !p.is_wet(t, 0.5));
                assert!(p.is_water(1, 1));
                assert!(!p.is_wet(IVec2::new(0, 0), 0.01), "dry cell");
                assert!(!p.is_wet(IVec2::new(40, 40), 0.01), "off the map is dry");
                assert_eq!(p.flow(t), Vec2::ZERO);
            })
            .unwrap();
    }

    #[test]
    fn seeded_field_overrides_static_stamp() {
        // The static stamp says wet; the live field (dried) wins.
        let mut app = probe_app(Some(seeded_field()));
        app.insert_resource(WorldStaticWaterDepth {
            width: 4,
            height: 4,
            depth_m: vec![3.0; 16],
            stamp: 0,
        });
        app.world_mut()
            .run_system_once(|p: WaterTileProbe| {
                assert!(p.seeded());
                assert!(!p.is_water(0, 0));
            })
            .unwrap();
    }

    #[test]
    fn unseeded_probe_is_dry_without_any_source() {
        let mut app = probe_app(None);
        app.world_mut()
            .run_system_once(|p: WaterTileProbe| {
                assert!(!p.available() && !p.seeded());
                assert!(!p.is_water(3, 3));
                assert_eq!(p.depth_m(IVec2::new(3, 3)), 0.0);
            })
            .unwrap();
    }
}
