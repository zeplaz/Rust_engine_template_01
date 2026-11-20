//! General surface-water schedule: seed the map field → step on the sim clock → project → gauges
//! → substrate.
//!
//! WM-1: one field covers the whole map. It is seeded once the world exists (every chunk is
//! present), from the real chunk elevation and the world's static water; nobody requests a region.

use bevy::prelude::*;

use crate::systems::sim_control::{SimControlSystemSet, SimStepClock};
use crate::terrain::family::default_terrain_families;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

use super::dam_faces::DamOps;
use super::gauge::GaugeBook;
use super::rain::{sample_weather_rain_system, SoilClass};
use super::sources::BASELINE_WATER_MIN_M;
use super::surface_water::{SurfaceWaterField, SurfaceWaterMode, FAR_FIELD_MIN_DT_S};
use super::{BarrierControlBook, BarrierControlIntent, SlowDomainTimeSkip, WorldStaticWaterDepth};

/// Most solver steps one frame runs; a bigger backlog (high sim speed) is folded into longer
/// steps so the cost stays bounded.
pub const MAX_WATER_STEPS_PER_FRAME: u32 = 4;

/// Scene overrides of the river sources. Rivers get their water from the world (`sources.rs`: each
/// headwater cell injects its channel's baseflow); a scene that needs a fixed river pins it here.
#[derive(Resource, Debug, Clone, Default)]
pub struct SurfaceWaterBaseflow {
    /// Pinned total river inflow (m3/s), shared over the sources by cross-section. 0 = every
    /// source carries its own channel's discharge.
    pub inflow_m3s: f32,
    pub min_release_m3s: f32,
}

#[derive(Resource, Debug, Default)]
pub struct SurfaceWaterSeedLatch {
    pub seeded: bool,
    /// `WorldStaticWaterDepth::stamp` the field was seeded from (a changed stamp = new world).
    pub stamp: u64,
}

/// What the world gives the water field per map cell: the bed (metres), standing-water flag, and
/// the soil (class + resting moisture) the rain falls on.
struct WorldSample {
    bed: Vec<f32>,
    standing: Vec<bool>,
    soil: Vec<SoilClass>,
    moisture: Vec<f32>,
}

/// Sample the chunk matrices into a [`WorldSample`]. `None` until every chunk of the `width` x
/// `height` world is present: a partial bed would seed holes.
fn sample_world_bed(
    chunks: &Query<(&Chunk, &ChunkCellMatrix)>,
    width: u32,
    height: u32,
    elevation_range_m: f32,
) -> Option<WorldSample> {
    let edge = CHUNK_TILES_SIM;
    let (cnx, cnz) = (width.div_ceil(edge), height.div_ceil(edge));
    let mut covered = vec![false; (cnx * cnz) as usize];
    let mut have = 0_usize;
    for (chunk, matrix) in chunks.iter() {
        let (c, sx, sy) = (chunk.coord, matrix.size.x, matrix.size.y);
        if sx == 0 || sy == 0 || c.x < 0 || c.y < 0 || c.x as u32 >= cnx || c.y as u32 >= cnz {
            continue;
        }
        let slot = &mut covered[(c.y as u32 * cnx + c.x as u32) as usize];
        if !*slot {
            *slot = true;
            have += 1;
        }
    }
    if have < covered.len() {
        return None;
    }
    let n = (width as usize) * (height as usize);
    let family_soil: Vec<SoilClass> = default_terrain_families()
        .families
        .iter()
        .map(|f| SoilClass::from_family_name(&f.name))
        .collect();
    let mut sample = WorldSample {
        bed: vec![0.0_f32; n],
        standing: vec![false; n],
        soil: vec![SoilClass::default(); n],
        moisture: vec![0.45_f32; n],
    };
    for (chunk, matrix) in chunks.iter() {
        let origin = chunk.coord * edge as i32;
        for ly in 0..matrix.size.y {
            for lx in 0..matrix.size.x {
                let (wx, wz) = (origin.x + lx as i32, origin.y + ly as i32);
                if wx < 0 || wz < 0 || wx as u32 >= width || wz as u32 >= height {
                    continue;
                }
                let i = (wz as usize) * (width as usize) + wx as usize;
                let m = matrix.idx(lx, ly);
                // Normalized 0..1 -> metres via the world's height range (CLOSE-W3).
                sample.bed[i] = matrix.elevation[m] * elevation_range_m;
                sample.standing[i] = matrix.water.get(m).is_some_and(|c| c.is_standing());
                if let Some(c) = matrix.family.get(m).and_then(|f| family_soil.get(f.0 as usize)) {
                    sample.soil[i] = *c;
                }
                if let Some(mo) = matrix.moisture.get(m) {
                    sample.moisture[i] = *mo;
                }
            }
        }
    }
    Some(sample)
}

/// Give the field its sources from the world: headwater cells of the river channels and the soil
/// the rain falls on. The channel mask is the baseline water that is not standing (lake / sea).
fn install_world_sources(field: &mut SurfaceWaterField, sample: &WorldSample, baseflow: &SurfaceWaterBaseflow) {
    let river: Vec<bool> = (0..field.cell_count())
        .map(|i| !sample.standing[i] && field.ref_depth_m.get(i).is_some_and(|d| *d > BASELINE_WATER_MIN_M))
        .collect();
    field.derive_river_sources(&river);
    field.pin_river_inflow((baseflow.inflow_m3s > 0.0).then_some(baseflow.inflow_m3s));
    field.set_soil(&sample.soil, &sample.moisture);
}

/// Allocate the map field once the world exists (and again when the world changes).
pub fn seed_surface_water_system(
    mut latch: ResMut<SurfaceWaterSeedLatch>,
    mut field: ResMut<SurfaceWaterField>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    static_depth: Option<Res<WorldStaticWaterDepth>>,
    params: Option<Res<WorldGenParams>>,
    baseflow: Res<SurfaceWaterBaseflow>,
) {
    let st = static_depth.as_deref().filter(|s| s.width > 0 && s.height > 0);
    let stale = st.is_some_and(|s| latch.stamp != s.stamp);
    if field.wired && latch.seeded && !stale {
        // Config follows the baseflow resource; a loaded field regains its baseline and sources.
        if baseflow.is_changed() {
            let (q, rel) = (baseflow.inflow_m3s, baseflow.min_release_m3s);
            field.pin_river_inflow((q > 0.0).then_some(q));
            field.outlet_flow_m3s = q;
            field.hydraulics.min_release_m3s = rel;
        }
        if field.ref_depth_m.len() != field.cell_count() {
            if let Some(st) = st {
                field.ref_depth_m = vec![0.0; field.cell_count()];
                for lz in 0..field.height {
                    for lx in 0..field.width {
                        let d = st.depth_at_world(field.origin_x + lx as i32, field.origin_z + lz as i32);
                        if let Some(d) = d.filter(|d| *d > 0.05) {
                            let i = field.idx(lx, lz);
                            field.ref_depth_m[i] = d;
                        }
                    }
                }
            }
        }
        if field.river_sources_stale() && field.ref_depth_m.len() == field.cell_count() {
            let range_m = params
                .as_deref()
                .map_or(crate::terrain::world_scale_contract::DEFAULT_ELEVATION_RANGE_M, |p| p.elevation_range_m);
            if let Some(sample) = sample_world_bed(&chunks, field.width, field.height, range_m) {
                install_world_sources(&mut field, &sample, &baseflow);
            }
        }
        return;
    }

    let (width, height) = match (st, params.as_deref()) {
        (Some(s), _) => (s.width, s.height),
        (None, Some(p)) => (p.width, p.height),
        (None, None) => return,
    };
    if width == 0 || height == 0 {
        return;
    }
    let range_m = params
        .as_deref()
        .map_or(crate::terrain::world_scale_contract::DEFAULT_ELEVATION_RANGE_M, |p| p.elevation_range_m);
    // Not every chunk exists yet (world still generating): wait, never seed a partial bed.
    let Some(sample) = sample_world_bed(&chunks, width, height, range_m) else {
        return;
    };

    field.allocate(0, 0, width, height, sample.bed.clone(), 0.0);
    field.mode = SurfaceWaterMode::VirtualPipes;
    field.hydraulics.min_release_m3s = baseflow.min_release_m3s;
    field.outlet_flow_m3s = baseflow.inflow_m3s;

    // CLOSE-W3 one water: start from the world's static water (the river the world drew).
    let static_wet = st.map_or(0, |s| field.seed_from_static_depth(s));
    // Standing water starts level at the world water line over the real bed (at rest).
    let leveled = params.as_deref().map_or(0, |p| {
        field.seed_standing_water(&sample.standing, p.biome_tuning.shallow_water_height_max * range_m)
    });
    field.recompute_mass();
    install_world_sources(&mut field, &sample, &baseflow);
    latch.seeded = true;
    latch.stamp = st.map_or(0, |s| s.stamp);

    info!(
        target: "water::solver",
        "SurfaceWaterField map {width}x{height} static_wet={static_wet} leveled_standing={leveled} river_sources={} inflow={:.1} m3/s",
        field.river_sources().len(),
        field.inflow_m3s
    );
}

pub fn step_surface_water_system(
    mut field: ResMut<SurfaceWaterField>,
    control: Res<BarrierControlIntent>,
    book: Res<BarrierControlBook>,
    skip: Res<SlowDomainTimeSkip>,
    clock: Res<SimStepClock>,
) {
    // The solver runs on the sim clock: no sim step this frame (paused, or between steps) = no
    // water motion.
    if !field.wired || clock.due == 0 {
        return;
    }
    // Hitch attribution (measurement only; `HITCH_MS=<ms>` reports the top scopes per slow frame).
    let _perf = crate::render::PerfScope::new("upd_water_solver");
    let t0 = std::time::Instant::now();

    // Every dam runs on its own control: the owner's entry in the book, else the global intent
    // (scenario rows without a dam site).
    let fallback = DamOps::from(&*control);
    field.set_dam_ops(|owner| owner.and_then(|e| book.get(e)).unwrap_or(fallback));

    let sim_secs = clock.step_secs() * clock.due as f32;
    // Time skip rides the sim clock like `SlowDomainElapsedSecs` (ecology): each due sim step
    // advances the reservoir by `dt_slow_secs` (OP-12). Only then may a shut-in dam's reservoir
    // window be filled by the level pool; the rest of the map keeps stepping in 2D (capped dt).
    let skip_dt = skip.dt_slow_secs.max(1) as f32 * clock.due as f32;
    let far_field = skip.enabled && skip_dt >= FAR_FIELD_MIN_DT_S && field.far_field_eligible();
    field.mode = if far_field { SurfaceWaterMode::LevelPool } else { SurfaceWaterMode::VirtualPipes };
    if far_field {
        field.advance(skip_dt);
    } else {
        // Real-time water: one solver step per due sim step (folded into longer steps beyond
        // MAX_WATER_STEPS_PER_FRAME so a fast sim cannot stall the frame).
        let n = clock.due.min(MAX_WATER_STEPS_PER_FRAME).max(1);
        let dt = sim_secs / n as f32;
        for _ in 0..n {
            field.advance(dt);
        }
    }
    field.solver_ms = t0.elapsed().as_secs_f32() * 1000.0;
}

pub fn publish_gauges_from_solver_system(
    field: Option<Res<SurfaceWaterField>>,
    gauges: Option<ResMut<GaugeBook>>,
) {
    let (Some(field), Some(mut gauges)) = (field, gauges) else {
        return;
    };
    if !field.wired || gauges.sites.is_empty() {
        return;
    }
    let sites: Vec<(i32, i32, u32)> = gauges
        .sites
        .iter()
        .map(|s| (s.tile.x, s.tile.y, s.cross_section_half_tiles))
        .collect();
    for (i, (x, z, half)) in sites.into_iter().enumerate() {
        let (q, stage, vel) = field.gauge_discharge(x, z, half);
        if let Some(r) = gauges.readings.get_mut(i) {
            r.discharge_m3s = q;
            r.flux_m3s = field.gauge_flux_m3s(x, z, half);
            r.stage_m = stage;
            r.velocity_mps = vel;
        }
    }
}

pub struct SurfaceWaterSolverPlugin;

impl Plugin for SurfaceWaterSolverPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.init_resource::<SurfaceWaterField>()
            .init_resource::<SurfaceWaterSeedLatch>()
            .init_resource::<SurfaceWaterBaseflow>()
            .register_type::<SurfaceWaterField>()
            .add_systems(
                Update,
                (
                    seed_surface_water_system,
                    sample_weather_rain_system.after(seed_surface_water_system),
                    step_surface_water_system
                        .after(sample_weather_rain_system)
                        .after(SimControlSystemSet::AdvanceSimTick),
                    publish_gauges_from_solver_system.after(step_surface_water_system),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::UVec2;

    /// A chunk whose ground falls toward z = 0 by `fall_per_tile` (normalized height per tile).
    fn chunk(app: &mut App, x: i32, y: i32, elevation: f32) {
        let mut m = ChunkCellMatrix::new(UVec2::splat(CHUNK_TILES_SIM));
        let edge = CHUNK_TILES_SIM;
        for ly in 0..edge {
            for lx in 0..edge {
                let wz = y as u32 * edge + ly;
                let i = m.idx(lx, ly);
                m.elevation[i] = elevation + wz as f32 * 2.0e-4;
            }
        }
        app.world_mut().spawn((Chunk { coord: IVec2::new(x, y) }, m));
    }

    fn app_64() -> App {
        let mut app = App::new();
        let params = WorldGenParams { width: 64, height: 64, ..Default::default() };
        let mut st = WorldStaticWaterDepth { width: 64, height: 64, depth_m: vec![0.0; 64 * 64], stamp: 1 };
        for z in 0..64 {
            for x in 30..34 {
                st.depth_m[z * 64 + x] = 2.5;
            }
        }
        app.insert_resource(params)
            .insert_resource(st)
            .init_resource::<SurfaceWaterField>()
            .init_resource::<SurfaceWaterSeedLatch>()
            .insert_resource(SurfaceWaterBaseflow { inflow_m3s: 120.0, min_release_m3s: 12.0 })
            .add_systems(Update, seed_surface_water_system);
        app
    }

    /// The map field waits for the whole world, then takes its bed from the chunks and its water
    /// from the static grid; the river that enters at the upstream edge is sourced there.
    #[test]
    fn the_map_field_waits_for_every_chunk_then_seeds_from_the_world() {
        let mut app = app_64();
        for (x, y) in [(0, 0), (1, 0), (0, 1)] {
            chunk(&mut app, x, y, 0.5);
        }
        app.update();
        assert!(!app.world().resource::<SurfaceWaterField>().wired, "3 of 4 chunks: no partial bed");
        chunk(&mut app, 1, 1, 0.5);
        app.update();
        let f = app.world().resource::<SurfaceWaterField>();
        assert!(f.wired && (f.width, f.height) == (64, 64) && (f.origin_x, f.origin_z) == (0, 0));
        let range = WorldGenParams::default().elevation_range_m;
        assert!((f.bed_m[0] - 0.5 * range).abs() < 1e-3, "bed from the chunk elevation, no fallback");
        assert!((f.depth_m[f.idx(31, 10)] - 2.5).abs() < 1e-6, "static river depth seeded");
        assert_eq!(f.inflow_m3s, 120.0, "the scene's pin is shared over the river's headwater cells");
        assert_eq!(f.river_sources().len(), 4, "the 4-wide river that enters at the upstream edge");
        assert!(f.river_sources().iter().all(|s| s.cell / 64 == 63));
        // Changing world (new static stamp) seeds the field again.
        app.world_mut().resource_mut::<WorldStaticWaterDepth>().stamp = 2;
        app.world_mut().resource_mut::<SurfaceWaterField>().depth_m[0] = 9.0;
        app.update();
        assert_eq!(app.world().resource::<SurfaceWaterField>().depth_m[0], 0.0, "reseeded for the new world");
    }

    /// Each dam in the field runs on the entry its owner published; a dam with none follows the
    /// global intent.
    #[test]
    fn each_dam_takes_its_own_control_from_the_book() {
        use super::super::DamOps;
        let mut app = app_64();
        for y in 0..2 {
            for x in 0..2 {
                chunk(&mut app, x, y, 0.5);
            }
        }
        app.init_resource::<BarrierControlIntent>()
            .init_resource::<BarrierControlBook>()
            .init_resource::<SlowDomainTimeSkip>()
            .init_resource::<SimStepClock>()
            .add_systems(Update, step_surface_water_system.after(seed_surface_water_system));
        app.update(); // seed
        let (a, b, c) = (
            Entity::from_bits(1 << 32 | 7),
            Entity::from_bits(1 << 32 | 8),
            Entity::from_bits(1 << 32 | 9),
        );
        let row = |x0: i32, x1: i32, z: i32| -> Vec<IVec2> { (x0..x1).map(|x| IVec2::new(x, z)).collect() };
        {
            let mut f = app.world_mut().resource_mut::<SurfaceWaterField>();
            f.install_barrier_tiles_for(&row(5, 15, 20), 10.0, Some(a));
            f.install_barrier_tiles_for(&row(40, 50, 40), 10.0, Some(b));
            f.install_barrier_tiles_for(&row(5, 15, 50), 10.0, Some(c));
        }
        let ops_a = DamOps { gate_open01: 0.9, outlet_open01: 0.1, ..DamOps::default() };
        let ops_b = DamOps { outlet_open01: 0.6, turbine_m3s: 12.0, ..DamOps::default() };
        {
            let mut book = app.world_mut().resource_mut::<BarrierControlBook>();
            book.set(a, ops_a);
            book.set(b, ops_b);
        }
        app.world_mut().resource_mut::<BarrierControlIntent>().outlet_open01 = 0.25;
        app.world_mut().resource_mut::<SimStepClock>().due = 1;
        app.update();
        let f = app.world().resource::<SurfaceWaterField>();
        assert_eq!(f.dam_ops(0), Some(ops_a));
        assert_eq!(f.dam_ops(1), Some(ops_b));
        let fallback = f.dam_ops(2).unwrap();
        assert_eq!((fallback.outlet_open01, fallback.gate_open01), (0.25, 0.0), "no entry: the global intent");
    }

    /// The solver runs on the sim clock: no due step (pause / between steps) = no water motion.
    #[test]
    fn the_solver_steps_only_when_the_sim_clock_is_due() {
        let mut app = app_64();
        for y in 0..2 {
            for x in 0..2 {
                chunk(&mut app, x, y, 0.5);
            }
        }
        app.init_resource::<BarrierControlIntent>()
            .init_resource::<BarrierControlBook>()
            .init_resource::<SlowDomainTimeSkip>()
            .init_resource::<SimStepClock>()
            .add_systems(Update, step_surface_water_system.after(seed_surface_water_system));
        app.update(); // seed (clock.due == 0: nothing steps)
        let moved = |app: &App| app.world().resource::<SurfaceWaterField>().solver_ms;
        assert_eq!(moved(&app), 0.0);
        app.world_mut().resource_mut::<SimStepClock>().due = 1;
        app.update();
        assert!(moved(&app) > 0.0, "a due sim step runs the solver");
        let t = app.world().resource::<SurfaceWaterField>().tile_stats();
        assert!(t.1 > 0 && t.1 < t.0, "only the river's tiles ran: {t:?}");
    }
}
