//! DW-4 — standing structures are walls to the surface-water solver.
//!
//! Construction publishes each standing site's footprint and structure height (metres above the
//! ground: floors x floor height, the same massing the voxel view draws); water applies it
//! ([`crate::systems::water::WaterObstacleRequest`] -> `SurfaceWaterField::set_obstacles`). A
//! site that stops standing (destroyed, despawned, demolished) clears its cells. Dams are not
//! here: their per-column crest has its own request (DW-3).

use bevy::prelude::*;
use std::collections::HashMap;

use crate::strategic::{ConstructionSite, SiteArchetype, SiteConstructionPhase, SiteFootprint};

/// Structure height in metres above the ground for a standing site; `None` = water passes
/// (open works, a dam handled by its own crest, or a site with no standing shell yet).
#[must_use]
pub fn water_obstacle_height_m(
    archetype: SiteArchetype,
    phase: SiteConstructionPhase,
    tiles: usize,
) -> Option<f32> {
    match archetype {
        SiteArchetype::Dam
        | SiteArchetype::Minefield
        | SiteArchetype::TrenchLine
        | SiteArchetype::DragonTeeth => return None,
        _ => {}
    }
    let standing = match phase {
        SiteConstructionPhase::Provisioning
        | SiteConstructionPhase::Operational
        | SiteConstructionPhase::Offline
        | SiteConstructionPhase::Damaged => 1.0,
        SiteConstructionPhase::UnderConstruction => 0.55,
        _ => return None,
    };
    let floors = f32::from(super::voxel_massing::derive_massing_floors(tiles as u32, None).max(1));
    Some(floors * super::procedural::standard_wall_height_m() * standing)
}

/// Publish wall cells for every standing site; clear cells of sites that stopped standing.
pub fn publish_water_obstacles_system(
    sites: Query<(Entity, &ConstructionSite, &SiteFootprint)>,
    mut request: ResMut<crate::systems::water::WaterObstacleRequest>,
    mut published: Local<HashMap<Entity, (Vec<IVec2>, f32)>>,
) {
    let mut seen = std::collections::HashSet::new();
    for (e, site, fp) in &sites {
        seen.insert(e);
        let h = water_obstacle_height_m(site.archetype, site.phase, fp.tiles.len()).unwrap_or(0.0);
        let prev = published.get(&e);
        if prev.is_some_and(|(t, ph)| *t == fp.tiles && (*ph - h).abs() < 0.1) {
            continue;
        }
        if let Some((old, _)) = prev {
            for t in old.iter().filter(|t| !fp.tiles.contains(t)) {
                request.push(*t, 0.0);
            }
        }
        if h <= 0.0 && prev.is_none() {
            continue;
        }
        for t in &fp.tiles {
            request.push(*t, h);
        }
        published.insert(e, (fp.tiles.clone(), h));
    }
    published.retain(|e, (tiles, _)| {
        if seen.contains(e) {
            return true;
        }
        for t in tiles.iter() {
            request.push(*t, 0.0);
        }
        false
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dw4_standing_buildings_wall_water_open_works_do_not() {
        let op = SiteConstructionPhase::Operational;
        assert!(water_obstacle_height_m(SiteArchetype::Factory, op, 9).unwrap() > 2.0);
        assert!(water_obstacle_height_m(SiteArchetype::Dam, op, 12).is_none());
        assert!(water_obstacle_height_m(SiteArchetype::TrenchLine, op, 6).is_none());
        assert!(water_obstacle_height_m(SiteArchetype::Factory, SiteConstructionPhase::Planned, 9).is_none());
    }

    #[test]
    fn dw4_despawned_site_clears_its_walls() {
        let mut app = App::new();
        app.init_resource::<crate::systems::water::WaterObstacleRequest>()
            .add_systems(Update, publish_water_obstacles_system);
        let e = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    site_id: 1,
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase: SiteConstructionPhase::Operational,
                    operational_readiness: 1.0,
                },
                SiteFootprint {
                    tiles: vec![IVec2::new(3, 4)],
                    layer: crate::strategic::LayerType::Surface,
                },
            ))
            .id();
        app.update();
        let first = std::mem::take(
            &mut app.world_mut().resource_mut::<crate::systems::water::WaterObstacleRequest>().pending,
        );
        assert_eq!(first.len(), 1);
        assert!(first[0].1 > 0.0);
        app.world_mut().despawn(e);
        app.update();
        let after = std::mem::take(
            &mut app.world_mut().resource_mut::<crate::systems::water::WaterObstacleRequest>().pending,
        );
        assert_eq!(after, vec![(IVec2::new(3, 4), 0.0)]);
    }
}
