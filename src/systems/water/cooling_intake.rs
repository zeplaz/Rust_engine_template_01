//! General industrial cooling intake — any site can hook a reservoir or river tile.
//!
//! PLAN-DAM-FLOOD-BENCHMARK-v1 §0 / DF-6: dam makes a reservoir; coolers attach here.
//! Destruction drains stage → intakes derate/trip without DamFlood special-cases.

use bevy::prelude::*;

use super::surface_water::{SurfaceWaterField, SurfaceWaterMode};

/// Where the intake samples water truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum CoolingIntakeSource {
    /// Use level-pool stage while the barrier is intact / LevelPool mode.
    ReservoirPool,
    /// Sample depth (or bed+depth surface) at a world tile.
    WorldTile { x: i32, z: i32 },
}

/// Cooling availability derived from reservoir/river stage vs intake thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect, Default)]
pub enum CoolingWaterState {
    /// Stage/depth at or above `min_stage_m` — full cooling.
    #[default]
    Available,
    /// Between trip and min — partial cooling (derate factor applied by consumers).
    Derated,
    /// Below `trip_stage_m` — cooling lost.
    Tripped,
    /// Field not wired / sample out of ROI.
    Unavailable,
}

/// Declares an industrial cooling demand against general water truth.
///
/// Attach to any cooler (nuclear, coal, chemical, …). Nuclear is one consumer.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct CoolingIntake {
    pub source: CoolingIntakeSource,
    /// Minimum stage (or local surface elevation) for full cooling [m].
    pub min_stage_m: f32,
    /// Stage at/below which cooling trips [m]. Must be ≤ `min_stage_m`.
    pub trip_stage_m: f32,
    /// Design withdrawal demand [m³/s] (informational for DF-6 mass coupling later).
    pub demand_m3s: f32,
    /// Last evaluated absolute water level used for thresholds [m].
    pub sample_stage_m: f32,
    pub state: CoolingWaterState,
    /// 1.0 Available, (0,1) Derated, 0 Tripped/Unavailable.
    pub cooling_factor: f32,
}

impl CoolingIntake {
    #[must_use]
    pub fn reservoir(min_stage_m: f32, trip_stage_m: f32, demand_m3s: f32) -> Self {
        Self {
            source: CoolingIntakeSource::ReservoirPool,
            min_stage_m,
            trip_stage_m: trip_stage_m.min(min_stage_m),
            demand_m3s: demand_m3s.max(0.0),
            sample_stage_m: 0.0,
            state: CoolingWaterState::Unavailable,
            cooling_factor: 0.0,
        }
    }

    #[must_use]
    pub fn world_tile(x: i32, z: i32, min_stage_m: f32, trip_stage_m: f32, demand_m3s: f32) -> Self {
        Self {
            source: CoolingIntakeSource::WorldTile { x, z },
            min_stage_m,
            trip_stage_m: trip_stage_m.min(min_stage_m),
            demand_m3s: demand_m3s.max(0.0),
            sample_stage_m: 0.0,
            state: CoolingWaterState::Unavailable,
            cooling_factor: 0.0,
        }
    }

    #[must_use]
    pub fn water_ok_for_full_cooling(&self) -> bool {
        matches!(self.state, CoolingWaterState::Available)
    }

    /// True when any cooling flow is possible (full or derated).
    #[must_use]
    pub fn water_available(&self) -> bool {
        matches!(
            self.state,
            CoolingWaterState::Available | CoolingWaterState::Derated
        )
    }
}

/// Pure evaluator — unit-tested; system only samples the field then calls this.
#[must_use]
pub fn evaluate_cooling_from_stage(
    sample_stage_m: Option<f32>,
    min_stage_m: f32,
    trip_stage_m: f32,
) -> (CoolingWaterState, f32) {
    let Some(stage) = sample_stage_m else {
        return (CoolingWaterState::Unavailable, 0.0);
    };
    let trip = trip_stage_m.min(min_stage_m);
    if stage <= trip {
        return (CoolingWaterState::Tripped, 0.0);
    }
    if stage < min_stage_m {
        let span = (min_stage_m - trip).max(1e-3);
        let factor = ((stage - trip) / span).clamp(0.05, 0.99);
        return (CoolingWaterState::Derated, factor);
    }
    (CoolingWaterState::Available, 1.0)
}

#[must_use]
pub fn sample_intake_stage_m(field: &SurfaceWaterField, source: CoolingIntakeSource) -> Option<f32> {
    if !field.wired {
        return None;
    }
    match source {
        CoolingIntakeSource::ReservoirPool => {
            if field.mode == SurfaceWaterMode::LevelPool || field.pool.storage_m3 > 0.0 {
                Some(field.pool.stage_m)
            } else {
                // Virtual-pipes with an empty pool: no coherent reservoir stage yet.
                Some(field.pool.stage_m)
            }
        }
        CoolingIntakeSource::WorldTile { x, z } => {
            let depth = field.depth_at_world(x, z)?;
            let lx = (x - field.origin_x) as u32;
            let lz = (z - field.origin_z) as u32;
            let bed = field.bed_m.get(field.idx(lx, lz)).copied().unwrap_or(0.0);
            Some(bed + depth)
        }
    }
}

/// Sole writer of [`CoolingIntake`] derived state. Reads [`SurfaceWaterField`] only.
pub fn update_cooling_intake_system(
    field: Option<Res<SurfaceWaterField>>,
    mut q: Query<&mut CoolingIntake>,
) {
    let Some(field) = field else {
        for mut intake in &mut q {
            intake.state = CoolingWaterState::Unavailable;
            intake.cooling_factor = 0.0;
        }
        return;
    };
    for mut intake in &mut q {
        let sample = sample_intake_stage_m(&field, intake.source);
        intake.sample_stage_m = sample.unwrap_or(0.0);
        let (state, factor) =
            evaluate_cooling_from_stage(sample, intake.min_stage_m, intake.trip_stage_m);
        intake.state = state;
        intake.cooling_factor = factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooling_intake_reads_reservoir_available_above_min() {
        let (state, factor) = evaluate_cooling_from_stage(Some(120.0), 100.0, 90.0);
        assert_eq!(state, CoolingWaterState::Available);
        assert!((factor - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cooling_derates_on_low_stage() {
        let (state, factor) = evaluate_cooling_from_stage(Some(95.0), 100.0, 90.0);
        assert_eq!(state, CoolingWaterState::Derated);
        assert!(factor > 0.0 && factor < 1.0);
    }

    #[test]
    fn cooling_trips_below_trip_stage() {
        let (state, factor) = evaluate_cooling_from_stage(Some(88.0), 100.0, 90.0);
        assert_eq!(state, CoolingWaterState::Tripped);
        assert_eq!(factor, 0.0);
    }

    #[test]
    fn second_industry_can_share_reservoir() {
        let mut field = SurfaceWaterField::default();
        field.wired = true;
        field.mode = SurfaceWaterMode::LevelPool;
        field.pool.stage_m = 110.0;
        field.pool.storage_m3 = 1.0e6;

        let mut nuclear = CoolingIntake::reservoir(100.0, 90.0, 48.0);
        let mut coal = CoolingIntake::reservoir(100.0, 90.0, 20.0);
        for intake in [&mut nuclear, &mut coal] {
            let sample = sample_intake_stage_m(&field, intake.source);
            let (state, factor) =
                evaluate_cooling_from_stage(sample, intake.min_stage_m, intake.trip_stage_m);
            intake.state = state;
            intake.cooling_factor = factor;
        }
        assert!(nuclear.water_available());
        assert!(coal.water_available());
        assert_eq!(nuclear.state, coal.state);

        field.pool.stage_m = 85.0;
        for intake in [&mut nuclear, &mut coal] {
            let sample = sample_intake_stage_m(&field, intake.source);
            let (state, factor) =
                evaluate_cooling_from_stage(sample, intake.min_stage_m, intake.trip_stage_m);
            intake.state = state;
            intake.cooling_factor = factor;
        }
        assert_eq!(nuclear.state, CoolingWaterState::Tripped);
        assert_eq!(coal.state, CoolingWaterState::Tripped);
    }

    #[test]
    fn update_system_writes_intake_from_field() {
        let mut app = App::new();
        app.init_resource::<SurfaceWaterField>()
            .add_systems(Update, update_cooling_intake_system);

        {
            let mut field = app.world_mut().resource_mut::<SurfaceWaterField>();
            field.wired = true;
            field.mode = SurfaceWaterMode::LevelPool;
            field.pool.stage_m = 105.0;
            field.pool.storage_m3 = 5.0e5;
        }

        let e = app
            .world_mut()
            .spawn(CoolingIntake::reservoir(100.0, 90.0, 10.0))
            .id();
        app.update();
        let intake = app.world().get::<CoolingIntake>(e).unwrap();
        assert_eq!(intake.state, CoolingWaterState::Available);
        assert!((intake.cooling_factor - 1.0).abs() < 1e-6);
    }

    #[test]
    fn breach_stage_collapse_trips_intakes() {
        let w = 16_u32;
        let h = 16_u32;
        let n = (w * h) as usize;
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, vec![0.0; n], 0.0);
        field.install_barrier_row(8, 20.0, 6, None);
        field.mode = SurfaceWaterMode::LevelPool;
        field.pool.area_m2 = 10_000.0;
        field.pool.storage_m3 = 100_000.0;
        field.pool.stage_m = 10.0;

        let mut nuclear = CoolingIntake::reservoir(8.0, 3.0, 48.0);
        let sample = sample_intake_stage_m(&field, nuclear.source);
        let (state, _) = evaluate_cooling_from_stage(sample, nuclear.min_stage_m, nuclear.trip_stage_m);
        nuclear.state = state;
        assert_eq!(nuclear.state, CoolingWaterState::Available);

        field.apply_barrier_breach(true, 500.0);
        let sample = sample_intake_stage_m(&field, nuclear.source);
        let (state, factor) =
            evaluate_cooling_from_stage(sample, nuclear.min_stage_m, nuclear.trip_stage_m);
        assert_eq!(state, CoolingWaterState::Tripped);
        assert_eq!(factor, 0.0);
    }
}
