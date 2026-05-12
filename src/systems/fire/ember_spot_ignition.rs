//! Long-range **ember** spot ignition: [`emit_ember_spot_ignition_events`] samples hot
//! [`ChunkFireOverlay`](super::types::ChunkFireOverlay) cells and emits [`EmberSpotIgnitionEvent`];
//! [`apply_ember_spot_ignitions`] applies heat to target cells (any chunk). External gameplay
//! (lightning, ordnance) may also write [`EmberSpotIgnitionEvent`].

use std::collections::HashMap;

use bevy::prelude::*;

use crate::systems::chunk_environment_persist::{ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks};
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::systems::weather::ChunkWeather;
use crate::terrain::ChunkCellKey;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

use super::chunk_surface_fire::ChunkSurfaceFire;
use super::fire_fuel::FireFuelField;
use super::types::ChunkFireOverlay;

/// Apply heat at a chunk cell. Consumed by [`apply_ember_spot_ignitions`]; safe for other systems to send.
#[derive(Message, Clone, Copy, Debug)]
pub struct EmberSpotIgnitionEvent {
    pub target: ChunkCellKey,
    pub spark: f32,
}

#[inline]
fn hash01(sim: u64, chunk: IVec2, cell: u32, salt: u32) -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    sim.hash(&mut h);
    chunk.hash(&mut h);
    cell.hash(&mut h);
    salt.hash(&mut h);
    let v = h.finish();
    (v as f64 / u64::MAX as f64) as f32
}

/// Map a local cell offset that may cross chunk boundaries into a [`ChunkCellKey`].
/// Assumes axis-aligned chunk grid with dimensions `sx`×`sy`.
pub fn resolve_spot_ignite_cell(
    origin_chunk: IVec2,
    sx: u32,
    sy: u32,
    local_x: i32,
    local_y: i32,
    ox: i32,
    oy: i32,
) -> Option<ChunkCellKey> {
    let sx = sx as i32;
    let sy = sy as i32;
    if sx <= 0 || sy <= 0 {
        return None;
    }
    let mut x = local_x + ox;
    let mut y = local_y + oy;
    let mut cx = origin_chunk.x;
    let mut cy = origin_chunk.y;
    while x < 0 {
        x += sx;
        cx -= 1;
    }
    while x >= sx {
        x -= sx;
        cx += 1;
    }
    while y < 0 {
        y += sy;
        cy -= 1;
    }
    while y >= sy {
        y -= sy;
        cy += 1;
    }
    if x < 0 || y < 0 || x >= sx || y >= sy {
        return None;
    }
    let idx = (y * sx + x) as u32;
    Some(ChunkCellKey::new(IVec2::new(cx, cy), idx))
}

/// Reads overlay + fuel + weather; emits [`EmberSpotIgnitionEvent`] for long jumps (non-local diffusion).
pub fn emit_ember_spot_ignition_events(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    tick: Res<SimTick>,
    mut writer: MessageWriter<EmberSpotIgnitionEvent>,
    q: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &ChunkWeather,
        Option<&FireFuelField>,
        &ChunkFireOverlay,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    const HEAT_MIN: f32 = 0.24;
    const MAX_EMBERS_PER_CHUNK: usize = 64;
    const SALT_PROB: u32 = 0xE0BE_5700;
    const SALT_D: u32 = 0xE0BE_5701;
    const SALT_ANG: u32 = 0xE0BE_5702;

    for (chunk, matrix, wx, fuel_opt, ovl) in &q {
        let sx_u = matrix.size.x as usize;
        let sy_u = matrix.size.y as usize;
        let n = sx_u.saturating_mul(sy_u);
        if n == 0 || ovl.heat.len() != n || ovl.fuel.len() != n {
            continue;
        }
        let ember = fuel_opt.map(|f| f.ember_spread_factor).unwrap_or(0.25);
        let rain = (1.0_f32 - wx.rain_intensity * 0.85).max(0.0);
        let wind = 1.0 + wx.wind_speed * 0.75;

        let mut sent = 0usize;
        for i in 0..n {
            if sent >= MAX_EMBERS_PER_CHUNK {
                break;
            }
            let h = ovl.heat[i];
            if h < HEAT_MIN {
                continue;
            }
            let f = ovl.fuel[i];
            let base_p = h * h * f * ember * rain * wind * dt * 0.55;
            if base_p <= 0.0 {
                continue;
            }
            let r = hash01(tick.0, chunk.coord, i as u32, SALT_PROB);
            if r > base_p {
                continue;
            }

            let ly = i / sx_u;
            let lx = i % sx_u;
            let d_raw = hash01(tick.0, chunk.coord, i as u32, SALT_D);
            let dist = 2 + (d_raw * 6.0).floor() as i32;
            let angle = hash01(tick.0, chunk.coord, i as u32, SALT_ANG) * std::f32::consts::TAU;
            let ox = (angle.cos() * dist as f32).round() as i32;
            let oy = (angle.sin() * dist as f32).round() as i32;

            let Some(target) = resolve_spot_ignite_cell(
                chunk.coord,
                matrix.size.x,
                matrix.size.y,
                lx as i32,
                ly as i32,
                ox,
                oy,
            ) else {
                continue;
            };

            let spark = (h * 0.07 * ember * (0.55 + wx.wind_speed * 0.45)).clamp(0.0, 0.3);
            if spark > 1e-4 {
                writer.write(EmberSpotIgnitionEvent { target, spark });
                sent += 1;
            }
        }
    }
}

pub fn apply_ember_spot_ignitions(
    mut reader: MessageReader<EmberSpotIgnitionEvent>,
    mut hooks: ResMut<ChunkEnvironmentPersistHooks>,
    chunks: Query<(Entity, &Chunk), With<ChunkFireOverlay>>,
    mut q: Query<(
        &ChunkCellMatrix,
        &mut ChunkFireOverlay,
        &mut ChunkSurfaceFire,
        &mut ChunkEnvironmentDirty,
    )>,
) {
    let mut accum: HashMap<IVec2, HashMap<u32, f32>> = HashMap::new();
    for ev in reader.read() {
        if ev.spark <= 1e-6 {
            continue;
        }
        accum
            .entry(ev.target.chunk)
            .or_default()
            .entry(ev.target.cell_index)
            .and_modify(|s| *s += ev.spark)
            .or_insert(ev.spark);
    }
    if accum.is_empty() {
        return;
    }

    let coord_to_entity: HashMap<IVec2, Entity> = chunks.iter().map(|(e, c)| (c.coord, e)).collect();

    for (coord, cells) in accum {
        let Some(&entity) = coord_to_entity.get(&coord) else {
            continue;
        };
        let Ok((matrix, mut ovl, mut surf, mut dirty)) = q.get_mut(entity) else {
            continue;
        };

        let n = (matrix.size.x * matrix.size.y) as usize;
        if ovl.heat.len() != n {
            continue;
        }

        let mut any = false;
        for (idx_u32, spark) in cells {
            let i = idx_u32 as usize;
            if i >= n {
                continue;
            }
            let m = matrix.moisture.get(i).copied().unwrap_or(0.5);
            let damp = (1.0 - m * 0.9).max(0.05);
            let spark = spark * damp;
            if spark <= 1e-6 {
                continue;
            }
            let nh = (ovl.heat[i] + spark).clamp(0.0, 1.0);
            if nh > ovl.heat[i] + 1e-6 {
                ovl.heat[i] = nh;
                any = true;
            }
        }

        if any {
            let mean_h: f32 = ovl.heat.iter().sum::<f32>() / n as f32;
            let mean_f: f32 = ovl.fuel.iter().sum::<f32>() / n.max(1) as f32;
            surf.heat = mean_h;
            surf.fuel = mean_f;

            dirty.fire_field = true;
            hooks.notify_fire_field_dirty(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_crosses_chunk_east() {
        let k = resolve_spot_ignite_cell(IVec2::ZERO, 4, 4, 3, 0, 2, 0).expect("key");
        assert_eq!(k.chunk, IVec2::new(1, 0));
        assert_eq!(k.cell_index, 1);
    }

    #[test]
    fn resolve_same_chunk() {
        let k = resolve_spot_ignite_cell(IVec2::new(2, -1), 8, 8, 4, 4, 1, -2).expect("key");
        assert_eq!(k.chunk, IVec2::new(2, -1));
        assert_eq!(k.cell_index, 4 + 1 + (4 - 2) * 8);
    }
}
