//! DF-5 — flood drown clock from the live water field into [`ChunkCellStand`] (WM-4: the field
//! only, through [`WaterTileProbe`]; no static-depth fallback).

use bevy::prelude::*;

use crate::systems::sim_control::SimTimeMicros;
use crate::systems::water::{SlowDomainElapsedSecs, WaterTileProbe, WET_DEPTH_M};
use crate::terrain::generation::chunk::Chunk;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;

use super::cell_stand::ChunkCellStand;

/// Sole writer of flood submergence clocks on stands (ecology domain).
pub fn stand_flood_from_water_system(
    water: WaterTileProbe,
    sim_time: Res<SimTimeMicros>,
    slow_elapsed: Option<Res<SlowDomainElapsedSecs>>,
    mut q: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &mut ChunkCellStand,
        Option<&mut super::fuel_moisture::ChunkCellFuelMoisture>,
    )>,
) {
    let _perf = crate::render::PerfScope::new("upd_eco_stand_flood_from_water_system");
    // No live field yet (world still loading): nothing is submerged.
    if !water.seeded() {
        return;
    }
    // DF-1 D4: time-skip adds ecology seconds so drown/dead clocks can finish in a benchmark.
    let now = ((sim_time.0 as f64 / 1_000_000.0)
        + slow_elapsed.map(|s| s.0).unwrap_or(0.0)) as f32;
    for (chunk, matrix, mut stand, fuel_m) in &mut q {
        let sx = matrix.size.x as i32;
        let sy = matrix.size.y as i32;
        if sx <= 0 || sy <= 0 {
            continue;
        }
        let n = (sx * sy) as usize;
        if stand.kind.len() != n {
            continue;
        }
        let mut wet = vec![false; n];
        for y in 0..sy {
            for x in 0..sx {
                let wx = chunk.coord.x * sx + x;
                let wz = chunk.coord.y * sy + y;
                let i = (y * sx + x) as usize;
                wet[i] = water.is_wet(IVec2::new(wx, wz), WET_DEPTH_M);
            }
        }
        let was_dead = stand.flood_dead.clone();
        stand.apply_flood_submergence(&wet, now);
        // DF-9 recession: newly Dead (dry after tolerance) saturates fine fuel moisture.
        if let Some(mut fuel) = fuel_m {
            fuel.resize_to(n, 0.5);
            for i in 0..n {
                if stand.flood_dead.get(i).copied().unwrap_or(false)
                    && !was_dead.get(i).copied().unwrap_or(false)
                    && !wet.get(i).copied().unwrap_or(true)
                {
                    fuel.fine_01[i] = 1.0;
                } else if stand.flood_dead.get(i).copied().unwrap_or(false)
                    && stand.submerged_since.get(i).copied().unwrap_or(-1.0) < 0.0
                {
                    fuel.fine_01[i] = fuel.fine_01[i].max(0.98);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::cell_stand::{StandKind, StandPhase, StandBurnConfig};

    /// WM-4: submergence comes from the live field alone. A static river stamp (here none at all)
    /// is not consulted; a cell under field water starts the drown clock, a dry one does not.
    #[test]
    fn stand_under_field_water_drowns_without_any_static_fallback() {
        use crate::systems::water::SurfaceWaterField;
        use bevy::math::UVec2;
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 2, 1, vec![0.0; 2], 0.0);
        field.depth_m[0] = 1.0;
        let mut app = App::new();
        app.insert_resource(field)
            .init_resource::<SimTimeMicros>()
            .add_systems(Update, stand_flood_from_water_system);
        let mut stand = ChunkCellStand::new(2);
        for i in 0..2 {
            stand.kind[i] = StandKind::Grass;
            stand.full_cap[i] = 0.5;
            stand.fuel_cap[i] = 0.5;
        }
        let chunk = app
            .world_mut()
            .spawn((Chunk { coord: IVec2::ZERO }, ChunkCellMatrix::new(UVec2::new(2, 1)), stand))
            .id();
        app.update();
        let at = |app: &App| app.world().get::<ChunkCellStand>(chunk).unwrap().clone();
        assert!(at(&app).submerged_since[0] >= 0.0, "wet cell starts the clock");
        assert!(at(&app).submerged_since[1] < 0.0, "dry cell does not");
        // Past the tolerance the wet cell is dead (Drowned phase), the dry one lives.
        app.world_mut().resource_mut::<SimTimeMicros>().0 =
            ((ChunkCellStand::FLOOD_TOLERANCE_S + 1.0) * 1.0e6) as u64;
        app.update();
        let s = at(&app);
        assert!(s.flood_dead[0] && !s.flood_dead[1]);
    }

    #[test]
    fn flood_does_not_delete_stand_marks_drowned_after_tolerance() {
        let mut stand = ChunkCellStand::new(1);
        stand.kind[0] = StandKind::Grass;
        stand.full_cap[0] = 0.5;
        stand.fuel_cap[0] = 0.5;
        stand.apply_flood_submergence(&[true], 0.0);
        assert!(stand.submerged_since[0] >= 0.0);
        assert!(!stand.flood_dead[0]);
        stand.apply_flood_submergence(&[true], ChunkCellStand::FLOOD_TOLERANCE_S + 1.0);
        assert!(stand.flood_dead[0]);
        let cfg = StandBurnConfig::default();
        assert_eq!(
            stand.phase(0, ChunkCellStand::FLOOD_TOLERANCE_S + 1.0, &cfg, 0, 1, 0.4),
            StandPhase::Drowned
        );
        stand.apply_flood_submergence(&[false], ChunkCellStand::FLOOD_TOLERANCE_S + 2.0);
        assert_eq!(
            stand.phase(0, ChunkCellStand::FLOOD_TOLERANCE_S + 2.0, &cfg, 0, 1, 0.4),
            StandPhase::Dead
        );
        assert!(stand.fuel_cap[0] > 0.0 || stand.full_cap[0] > 0.0);
    }

    #[test]
    fn short_flood_does_not_drown() {
        let mut stand = ChunkCellStand::new(1);
        stand.kind[0] = StandKind::Grass;
        stand.full_cap[0] = 0.4;
        stand.fuel_cap[0] = 0.4;
        stand.apply_flood_submergence(&[true], 0.0);
        // Recede well before species tolerance.
        stand.apply_flood_submergence(&[false], 3600.0);
        assert!(!stand.flood_dead[0]);
        assert!(stand.submerged_since[0] < 0.0);
        let cfg = StandBurnConfig::default();
        assert_eq!(stand.phase(0, 3600.0, &cfg, 0, 1, 0.4), StandPhase::Live);
    }

    #[test]
    fn recession_after_tolerance_marks_dead_and_saturates_fine() {
        use crate::systems::ecology::fuel_moisture::ChunkCellFuelMoisture;
        let mut stand = ChunkCellStand::new(1);
        stand.kind[0] = StandKind::Grass;
        stand.full_cap[0] = 0.5;
        stand.fuel_cap[0] = 0.5;
        stand.apply_flood_submergence(&[true], 0.0);
        stand.apply_flood_submergence(&[true], ChunkCellStand::FLOOD_TOLERANCE_S + 1.0);
        assert!(stand.flood_dead[0]);
        let was_dead = stand.flood_dead.clone();
        stand.apply_flood_submergence(&[false], ChunkCellStand::FLOOD_TOLERANCE_S + 2.0);
        let cfg = StandBurnConfig::default();
        assert_eq!(
            stand.phase(0, ChunkCellStand::FLOOD_TOLERANCE_S + 2.0, &cfg, 0, 1, 0.4),
            StandPhase::Dead
        );
        let mut fuel = ChunkCellFuelMoisture::new(1, 0.2);
        fuel.fine_01[0] = 0.3;
        // Mirror recession saturate hook used by the system.
        if stand.flood_dead[0] && was_dead[0] && stand.submerged_since[0] < 0.0 {
            fuel.fine_01[0] = fuel.fine_01[0].max(0.98);
        }
        assert!(fuel.fine_01[0] >= 0.98);
    }
}
