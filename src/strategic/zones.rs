//! Spatial **influence fields** — zones modify chunk overlays and (later) [`super::world_field_layers::ChunkFieldCell`] buffers.

use bevy::prelude::*;

use crate::strategic::{ChunkStrategicOverlay, StrategicRasterConfig, MAX_STRATEGIC_FACTION_SLOTS};

/// Tactical zone discriminator (supply, sensor, control, …).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    Supply,
    Fire,
    Sensor,
    Command,
    Control,
    Denial,
}

/// Radial field emitter in **world tile** space (`center_tile.x` = column, `.y` = row / Z).
#[derive(Component, Clone, Debug)]
pub struct Zone {
    pub kind: ZoneKind,
    pub faction_slot: u8,
    pub center_tile: IVec2,
    /// Radius in **tiles** (Euclidean falloff inside disk).
    pub radius_tiles: f32,
    pub intensity: f32,
}

#[inline]
fn zone_falloff(dist: f32, radius: f32) -> f32 {
    if radius <= 0.0 || dist >= radius {
        return 0.0;
    }
    1.0 - dist / radius
}

/// Project zones into [`ChunkStrategicOverlay`] (additive, clamped). Runs after transport/logistics injection.
pub fn apply_zones_to_strategic_overlays_system(
    zones: Query<&Zone>,
    mut overlays: Query<&mut ChunkStrategicOverlay>,
    cfg: Res<StrategicRasterConfig>,
) {
    let cw = cfg.cells_per_chunk.x.max(1);
    let ch = cfg.cells_per_chunk.y.max(1);

    for z in zones.iter() {
        let r = z.radius_tiles.max(0.0);
        if r <= 0.0 || z.intensity <= 0.0 {
            continue;
        }
        let slot = (z.faction_slot as usize).min(MAX_STRATEGIC_FACTION_SLOTS.saturating_sub(1));
        let min_tx = z.center_tile.x as f32 - r;
        let max_tx = z.center_tile.x as f32 + r;
        let min_tz = z.center_tile.y as f32 - r;
        let max_tz = z.center_tile.y as f32 + r;
        let min_cx = (min_tx / cw as f32).floor() as i32;
        let max_cx = (max_tx / cw as f32).floor() as i32;
        let min_cz = (min_tz / ch as f32).floor() as i32;
        let max_cz = (max_tz / ch as f32).floor() as i32;

        for cz in min_cz..=max_cz {
            for cx in min_cx..=max_cx {
                let chunk_coord = IVec2::new(cx, cz);
                for mut overlay in overlays.iter_mut() {
                    if overlay.chunk_coord != chunk_coord {
                        continue;
                    }
                    let n = overlay.len_cells();
                    for ci in 0..n {
                        let lx = (ci % overlay.size.x as usize) as u32;
                        let lz = (ci / overlay.size.x as usize) as u32;
                        let wx = (cx as f32 * cw as f32) + lx as f32 + 0.5;
                        let wz = (cz as f32 * ch as f32) + lz as f32 + 0.5;
                        let dx = wx - z.center_tile.x as f32;
                        let dz = wz - z.center_tile.y as f32;
                        let dist = (dx * dx + dz * dz).sqrt();
                        let w = z.intensity * zone_falloff(dist, r);
                        if w <= 0.0 {
                            continue;
                        }
                        match z.kind {
                            ZoneKind::Supply => {
                                overlay.logistics_strength[ci][slot] =
                                    (overlay.logistics_strength[ci][slot] + w).min(1.0);
                                overlay.logistics_throughput[ci] =
                                    (overlay.logistics_throughput[ci] + w * 0.5).min(1.0);
                            }
                            ZoneKind::Fire => {
                                overlay.artillery_danger[ci][slot] =
                                    (overlay.artillery_danger[ci][slot] + w).min(1.0);
                                overlay.threat[ci][slot] = (overlay.threat[ci][slot] + w * 0.7).min(1.0);
                            }
                            ZoneKind::Sensor => {
                                overlay.recon_confidence[ci][slot] =
                                    (overlay.recon_confidence[ci][slot] + w).min(1.0);
                            }
                            ZoneKind::Command => {
                                overlay.civilian_stability[ci] =
                                    (overlay.civilian_stability[ci] + w * 0.25).min(1.0);
                                overlay.ew_denial[ci] = (overlay.ew_denial[ci] - w * 0.15).max(0.0);
                            }
                            ZoneKind::Control => {
                                overlay.faction_control[ci][slot] =
                                    (overlay.faction_control[ci][slot] + w).min(1.0);
                            }
                            ZoneKind::Denial => {
                                overlay.mobility_cost[ci] = (overlay.mobility_cost[ci] + w).min(1.0);
                                overlay.ew_denial[ci] = (overlay.ew_denial[ci] + w * 0.5).min(1.0);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_boosts_control_slice() {
        let mut app = App::new();
        app.init_resource::<StrategicRasterConfig>()
            .add_systems(Update, apply_zones_to_strategic_overlays_system);
        app.world_mut().resource_mut::<StrategicRasterConfig>().cells_per_chunk = UVec2::new(4, 4);
        let e = app
            .world_mut()
            .spawn((
                Zone {
                    kind: ZoneKind::Control,
                    faction_slot: 0,
                    center_tile: IVec2::new(2, 2),
                    radius_tiles: 3.0,
                    intensity: 1.0,
                },
                ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(4, 4)),
            ))
            .id();
        app.update();
        let overlay = app.world().entity(e).get::<ChunkStrategicOverlay>().unwrap();
        assert!(overlay.faction_control[0][0] > 0.0);
    }
}
