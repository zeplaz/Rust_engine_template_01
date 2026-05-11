//! Zone emitters → [`crate::strategic::ChunkStrategicOverlay`] (P2-E). Stamps operational sites onto overlay SOA (slot 0), after logistics / zone baselines.

use bevy::prelude::*;

use super::components::{ConstructionSite, SiteArchetype, SiteFootprint, ZoneEmitter};
use super::resources::SiteConstructionPhase;
use crate::strategic::{ChunkStrategicOverlay, StrategicRasterConfig, MAX_STRATEGIC_FACTION_SLOTS};

#[inline]
pub fn zone_emitter_for_archetype(archetype: SiteArchetype) -> ZoneEmitter {
    match archetype {
        SiteArchetype::RadarSite | SiteArchetype::SensorPost => ZoneEmitter {
            sensor_strength: 0.35,
            ..Default::default()
        },
        SiteArchetype::RailDepot | SiteArchetype::FuelDepot => ZoneEmitter {
            supply_strength: 0.4,
            ..Default::default()
        },
        SiteArchetype::Factory | SiteArchetype::PowerPlant | SiteArchetype::WaterPlant => ZoneEmitter {
            supply_strength: 0.22,
            civil_authority_strength: 0.12,
            ..Default::default()
        },
        SiteArchetype::MilitaryBase | SiteArchetype::BunkerComplex | SiteArchetype::TrenchLine => ZoneEmitter {
            fire_control_strength: 0.28,
            supply_strength: 0.12,
            ..Default::default()
        },
        SiteArchetype::CivilHousing => ZoneEmitter {
            civil_authority_strength: 0.2,
            supply_strength: 0.08,
            ..Default::default()
        },
    }
}

/// Refresh emitter scalars when lifecycle / archetype-relevant state changes.
pub fn sync_zone_emitter_from_archetype_system(
    mut q: Query<(&ConstructionSite, &mut ZoneEmitter), Changed<ConstructionSite>>,
) {
    for (site, mut z) in &mut q {
        *z = zone_emitter_for_archetype(site.archetype);
    }
}

/// Additive stamp of [`ZoneEmitter`] onto cells covered by [`SiteFootprint`] (operational sites only).
pub fn apply_site_zone_emitters_to_overlays_system(
    sites: Query<(&ConstructionSite, &SiteFootprint, &ZoneEmitter), With<SiteFootprint>>,
    mut overlays: Query<&mut ChunkStrategicOverlay>,
    cfg: Res<StrategicRasterConfig>,
) {
    let cw = cfg.cells_per_chunk.x.max(1) as i32;
    let ch = cfg.cells_per_chunk.y.max(1) as i32;
    let slot = 0usize.min(MAX_STRATEGIC_FACTION_SLOTS.saturating_sub(1));

    for (site, footprint, emitter) in &sites {
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }
        for tile in &footprint.tiles {
            let tx = tile.x;
            let tz = tile.y;
            let chunk_coord = IVec2::new(tx.div_euclid(cw), tz.div_euclid(ch));
            let lx = tx.rem_euclid(cw);
            let lz = tz.rem_euclid(ch);
            if lx < 0 || lz < 0 {
                continue;
            }
            let lx = lx as u32;
            let lz = lz as u32;
            for mut overlay in overlays.iter_mut() {
                if overlay.chunk_coord != chunk_coord {
                    continue;
                }
                if lx >= overlay.size.x || lz >= overlay.size.y {
                    continue;
                }
                let ci = (lz as usize) * (overlay.size.x as usize) + (lx as usize);
                if ci >= overlay.len_cells() {
                    continue;
                }
                let sup = emitter.supply_strength;
                if sup > 0.0 {
                    overlay.logistics_strength[ci][slot] = (overlay.logistics_strength[ci][slot] + sup).min(1.0);
                    overlay.logistics_throughput[ci] = (overlay.logistics_throughput[ci] + sup * 0.35).min(1.0);
                }
                let fire = emitter.fire_control_strength;
                if fire > 0.0 {
                    overlay.artillery_danger[ci][slot] = (overlay.artillery_danger[ci][slot] + fire).min(1.0);
                    overlay.threat[ci][slot] = (overlay.threat[ci][slot] + fire * 0.65).min(1.0);
                }
                let sense = emitter.sensor_strength;
                if sense > 0.0 {
                    overlay.recon_confidence[ci][slot] = (overlay.recon_confidence[ci][slot] + sense).min(1.0);
                    overlay.visibility[ci] = (overlay.visibility[ci] + sense * 0.4).min(1.0);
                }
                let civil = emitter.civil_authority_strength;
                if civil > 0.0 {
                    overlay.civilian_stability[ci] = (overlay.civilian_stability[ci] + civil).min(1.0);
                    overlay.faction_control[ci][slot] = (overlay.faction_control[ci][slot] + civil * 0.5).min(1.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::plugin::StrategicFieldsPlugin;
    use crate::strategic::site::{
        ConstructionSite, SiteArchetype, SiteConstructionPhase, SiteFootprint, ZoneEmitter,
    };
    use crate::strategic::spatial_network::LayerType;
    use crate::systems::terrain::MaterialUnificationPlugin;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use bevy::asset::AssetPlugin;
    use bevy::prelude::{App, IVec2, MinimalPlugins, UVec2};

    #[test]
    fn operational_site_stamps_overlay_cell() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(StrategicFieldsPlugin);

        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            ChunkCellMatrix::new(UVec2::new(4, 4)),
        ));

        app.update();

        let owner = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            ConstructionSite {
                site_id: 1,
                owner,
                archetype: SiteArchetype::FuelDepot,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            SiteFootprint {
                tiles: vec![IVec2::ZERO],
                layer: LayerType::Surface,
            },
            ZoneEmitter {
                supply_strength: 0.6,
                ..Default::default()
            },
        ));

        app.update();

        {
            let world = app.world_mut();
            let mut q = world.query::<&crate::strategic::ChunkStrategicOverlay>();
            let overlay = q.iter(world).next().expect("overlay");
            assert!(
                overlay.logistics_strength[0][0] > 0.01,
                "emitter should add logistics_strength at footprint cell"
            );
        }
    }
}
