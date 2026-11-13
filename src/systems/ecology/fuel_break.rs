//! Fuel-break / firebreak cell clears — ecology sole writer of stand fuel caps (W2).
//!
//! Construction execute emits [`FuelBreakCommitEvent`]; this module applies Bare + zero fuel.
//! Spotting may still jump a break at high wind.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::systems::fire::ChunkFireOverlay;
use crate::terrain::generation::Chunk;

use super::cell_stand::{ChunkCellStand, StandKind};

/// One committed fuel-clear strip (road, rail, or player firebreak).
#[derive(Clone, Debug)]
pub struct FuelBreakCommitEvent {
    pub tiles: Vec<BuildSiteTile>,
    pub cells_per_chunk: UVec2,
}

/// Construction execute pushes here; ecology drains (avoids growing execute's SystemParam count).
#[derive(Resource, Default, Debug)]
pub struct PendingFuelBreaks(pub Vec<FuelBreakCommitEvent>);

impl PendingFuelBreaks {
    pub fn push(&mut self, ev: FuelBreakCommitEvent) {
        if !ev.tiles.is_empty() {
            self.0.push(ev);
        }
    }
}

/// Clear standing fuel along committed tiles. Single writer for this mutation path.
pub fn apply_fuel_break_commits(
    mut pending: ResMut<PendingFuelBreaks>,
    mut q: Query<(
        &Chunk,
        &mut ChunkCellStand,
        Option<&mut ChunkFireOverlay>,
        Option<&mut super::vegetation_field::CanopyDrivers>,
    )>,
) {
    let events = std::mem::take(&mut pending.0);
    for ev in events {
        if ev.tiles.is_empty() {
            continue;
        }
        let cw = ev.cells_per_chunk.x.max(1) as i32;
        let ch = ev.cells_per_chunk.y.max(1) as i32;
        let mut by_chunk: HashMap<IVec2, Vec<usize>> = HashMap::new();
        for tile in &ev.tiles {
            let tx = tile.x as i32;
            let tz = tile.z as i32;
            let cc = IVec2::new(tx.div_euclid(cw), tz.div_euclid(ch));
            let local_x = tx.rem_euclid(cw) as usize;
            let local_z = tz.rem_euclid(ch) as usize;
            let idx = local_z * cw as usize + local_x;
            by_chunk.entry(cc).or_default().push(idx);
        }
        for (chunk, mut stand, mut ovl, mut drivers) in &mut q {
            let Some(indices) = by_chunk.get(&chunk.coord) else {
                continue;
            };
            let n = stand.kind.len();
            if n == 0 {
                continue;
            }
            for &i in indices {
                if i >= n {
                    continue;
                }
                stand.kind[i] = StandKind::Bare;
                stand.full_cap[i] = 0.0;
                stand.fuel_cap[i] = 0.0;
                if stand.burnt_at[i] >= 0.0 {
                    stand.burnt_at[i] = -1.0;
                    stand.burnt_count = stand.burnt_count.saturating_sub(1);
                }
                if let Some(ref mut ovl) = ovl {
                    if i < ovl.fuel.len() {
                        ovl.fuel[i] = 0.0;
                        ovl.heat[i] = 0.0;
                    }
                }
            }
            if let Some(ref mut drivers) = drivers {
                drivers.clear_to = Some(super::vegetation_field::CANOPY_CLEARED_DENSITY);
            }
            stand.render_rev = stand.render_rev.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::StandKind;
    use crate::systems::fire::ChunkFireOverlay;

    #[test]
    fn fuel_break_zeros_stand_and_overlay() {
        let mut app = App::new();
        app.init_resource::<PendingFuelBreaks>()
            .add_systems(Update, apply_fuel_break_commits);
        let mut stand = ChunkCellStand::new(4);
        stand.kind = vec![StandKind::Timber; 4];
        stand.full_cap = vec![0.6; 4];
        stand.fuel_cap = vec![0.6; 4];
        stand.seeded = true;
        let e = app
            .world_mut()
            .spawn((
                Chunk {
                    coord: IVec2::ZERO,
                },
                stand,
                ChunkFireOverlay {
                    heat: vec![0.2; 4],
                    fuel: vec![0.5; 4],
                    smoke: vec![0.0; 4],
                    toxic: vec![0.0; 4],
                    burn_hold: vec![0; 4],
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<PendingFuelBreaks>()
            .push(FuelBreakCommitEvent {
                tiles: vec![BuildSiteTile { x: 1, z: 0 }],
                cells_per_chunk: UVec2::new(2, 2),
            });
        app.update();
        let stand = app.world().get::<ChunkCellStand>(e).unwrap();
        let ovl = app.world().get::<ChunkFireOverlay>(e).unwrap();
        assert_eq!(stand.kind[1], StandKind::Bare);
        assert_eq!(stand.fuel_cap[1], 0.0);
        assert_eq!(ovl.fuel[1], 0.0);
        assert_eq!(stand.kind[0], StandKind::Timber);
    }
}
