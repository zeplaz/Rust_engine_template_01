//! Bundled map-pick SystemParam — keeps draw/build input systems under Bevy's 16-param limit.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::{MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::WorldGenDenseTerrainCache;

use super::roads::cursor_world_on_map;

#[derive(SystemParam)]
pub struct ConstructionMapPick<'w, 's> {
    pub win: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    pub authority: Option<Res<'w, ViewProjectionAuthority>>,
    pub desired: Res<'w, MapCameraDesiredRes>,
    pub map_vp: Res<'w, SimulationMapViewport>,
    pub params: Res<'w, WorldGenParams>,
    pub height_grid: Option<Res<'w, WorldGenDenseTerrainCache>>,
    pub ortho: Res<'w, MainWorldCameraOrthoTrace>,
}

impl ConstructionMapPick<'_, '_> {
    #[must_use]
    pub fn cursor_world(&self) -> Option<Vec3> {
        let window = self.win.single().ok()?;
        cursor_world_on_map(
            window,
            self.authority.as_deref(),
            self.desired.as_ref(),
            self.map_vp.as_ref(),
            self.params.as_ref(),
        )
    }

    #[must_use]
    pub fn height_grid(&self) -> Option<&WorldGenDenseTerrainCache> {
        self.height_grid.as_deref()
    }
}
