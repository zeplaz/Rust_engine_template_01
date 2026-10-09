//! Site phase occupation is drawn by [`super::phase_visual`] in map space.
//!
//! Production does not publish those cells into [`TileDebugInstanceMap`]. That GPU
//! bucket stored map-Y in the second XY component at Z = 0, and
//! `tile_debug_instanced.wgsl` painted phase ≤ 3 gold and phase == 6 green — a
//! stacked quad at the map corner.

use bevy::prelude::*;

/// Retained so representation policy and the minimap can observe that the GPU
/// phase channel is off. Not a second tile-debug writer.
#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionPhaseGpuChannel {
    pub active: bool,
    pub instance_count: u32,
}

/// Clears the GPU phase channel. Does not append [`crate::gui::TileDebugInstanceMap`] rows.
pub fn push_site_phase_tile_instances(mut channel: ResMut<ConstructionPhaseGpuChannel>) {
    channel.active = false;
    channel.instance_count = 0;
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::gui::{tile_flags, TileDebugInstance, TileDebugInstanceMap, TileDebugViewId};
    use crate::strategic::{
        BuildSiteTile, ConstructionSite, FootprintTiles, LayerType, PlannedSite, SiteArchetype,
        SiteConstructionPhase, SiteFootprint, SiteId,
    };

    use super::{push_site_phase_tile_instances, ConstructionPhaseGpuChannel};
    use crate::construction::tile_visual::ConstructionTileVisualSettings;

    fn spawn_site(app: &mut App, phase: SiteConstructionPhase, tile: IVec2) {
        app.world_mut().spawn((
            ConstructionSite {
                site_id: 7,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::Factory,
                phase,
                operational_readiness: 0.0,
            },
            PlannedSite {
                site_id: SiteId::UNASSIGNED,
                origin: BuildSiteTile {
                    x: tile.x.max(0) as u32,
                    z: tile.y.max(0) as u32,
                },
                footprint: FootprintTiles { width: 1, depth: 1 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: None,
                placement: None,
            },
            SiteFootprint {
                tiles: vec![tile],
                layer: LayerType::Surface,
            },
        ));
    }

    #[test]
    fn production_does_not_push_phase_tiles_into_gpu_debug_map() {
        let mut app = App::new();
        let settings = ConstructionTileVisualSettings::default();
        assert!(settings.show_site_phase_tiles);
        app.insert_resource(settings);
        let mut map = TileDebugInstanceMap::default();
        map.per_view.insert(
            TileDebugViewId::WorldMain,
            vec![TileDebugInstance {
                world_pos: [0.5, 0.5],
                size: 0.46,
                lod: 0,
                flags: tile_flags::FOCUS,
            }],
        );
        app.insert_resource(map);
        app.init_resource::<ConstructionPhaseGpuChannel>();
        app.add_systems(Update, push_site_phase_tile_instances);
        spawn_site(&mut app, SiteConstructionPhase::Foundation, IVec2::ZERO);
        spawn_site(
            &mut app,
            SiteConstructionPhase::Operational,
            IVec2::new(12, 40),
        );
        app.update();

        let map = app.world().resource::<TileDebugInstanceMap>();
        let rows = map
            .per_view
            .get(&TileDebugViewId::WorldMain)
            .expect("world bucket");
        assert_eq!(rows.len(), 1, "phase sites must not append debug rows");
        assert!(rows
            .iter()
            .all(|row| row.flags & tile_flags::CONSTRUCTION_SITE == 0));
        let channel = app.world().resource::<ConstructionPhaseGpuChannel>();
        assert!(!channel.active);
        assert_eq!(channel.instance_count, 0);
    }
}
