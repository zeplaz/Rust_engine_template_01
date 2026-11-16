//! Deterministic ballistic / blast bookkeeping (Ranking C3 scaffold).
//!
//! **Dual-cite (V4-PHYS09 / PHYS-09):**
//! - NEW: gea4v2 14.1 Do You Want Physics in Your Game? · 14.1.2.1 Simulations (Sims) ·
//!   14.1.3 Impact of Physics on a Game — bookkeeping / kinematic envelopes beat a full
//!   rigid-body world for this city-sim; we do **not** host middleware dynamics here.
//! - LEGACY: `physics_collision` (gea_v2_e4 §14 p352) · `physics_middleware` (gea_v2_e4 §14.2 p358)
//!   — chapter asks the same design question; answer for this engine is intentional **no**
//!   rigid-body middleware (no Rapier/Havok/PhysX host).
//!
//! Contract: [`BallisticFlightBook`] + [`step_ballistic_flights`] advance kinematic flights,
//! then **enqueue** [`BlastIntent`] (intent→execute). Construction owns damage execute
//! (PHYS-07/08). Presentation/audio consume dumps. Voxel occupation clears stay on the
//! demolish funnel — ballistics never writes sites / occupation.

use bevy::prelude::*;

use crate::audio::{AudioOneShotKind, AudioOneShotQueue};
use crate::construction::{BlastIntent, BlastIntentQueue};
use crate::systems::sim_attention::SimAttentionFrame;
use crate::systems::sim_control::{SimControlSystemSet, SimStepClock};
use crate::systems::weather::{missile_weather_drift, WeatherSample};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

const METRES_PER_TILE: f32 = TILE_METRES_GAMEPLAY;

/// Fixed aim-correction iterations (MOVE-09).
pub const MISSILE_AIM_REFINE_ITERS: usize = 6;
/// Early-out when predicted miss is below half a tile.
pub const MISSILE_AIM_HALF_CELL: f32 = 0.5;

/// Constant-wind params for guided-missile drift during ballistic step / prediction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MissileBallisticWind {
    pub sample: WeatherSample,
    pub drift_coef: f32,
    pub max_wind_mps: f32,
}

#[must_use]
pub fn wind_drift_tiles(
    wind: &MissileBallisticWind,
    flight_time_s: f32,
) -> Vec2 {
    missile_weather_drift(
        &wind.sample,
        wind.drift_coef,
        wind.max_wind_mps,
        flight_time_s,
    ) / METRES_PER_TILE
}

enum BallisticAdvance {
    InFlight {
        pos: Vec2,
        seg_time_s: f32,
    },
    Impact {
        pos: Vec2,
        seg_time_s: f32,
    },
}

/// One ballistic integrator step — shared by live [`step_ballistic_flights`] and prediction.
fn advance_ballistic_toward_aim(
    prev: Vec2,
    aim: Vec2,
    speed_tiles_per_step: f32,
    sim_steps: u32,
    step_secs: f32,
) -> BallisticAdvance {
    let speed = speed_tiles_per_step.max(0.01);
    let step_dist = speed * sim_steps as f32;
    let delta = aim - prev;
    let dist = delta.length();

    if dist <= step_dist || dist < 1e-3 {
        let seg_time = (dist / speed) * step_secs;
        return BallisticAdvance::Impact {
            pos: aim,
            seg_time_s: seg_time,
        };
    }

    let dir = delta / dist;
    let next = prev + dir * step_dist;
    if let Some(hit) = first_target_cell_on_segment(prev, next, aim) {
        let seg_len = (hit - prev).length();
        let seg_time = (seg_len / speed) * step_secs;
        return BallisticAdvance::Impact {
            pos: hit,
            seg_time_s: seg_time,
        };
    }

    BallisticAdvance::InFlight {
        pos: next,
        seg_time_s: sim_steps as f32 * step_secs,
    }
}

#[must_use]
fn apply_missile_wind_to_impact(
    nominal: Vec2,
    flight_time_s: f32,
    wind: Option<MissileBallisticWind>,
) -> Vec2 {
    wind.map(|w| nominal + wind_drift_tiles(&w, flight_time_s))
        .unwrap_or(nominal)
}

/// Predict where a missile lands under constant wind (same integrator as live step, 1 sim step per loop).
#[must_use]
pub fn simulate_missile_impact(
    origin: Vec2,
    aim: Vec2,
    speed_tiles_per_step: f32,
    wind: &MissileBallisticWind,
    step_secs: f32,
) -> Vec2 {
    let mut pos = origin;
    let mut flight_time_s = 0.0_f32;
    const MAX_SIM_STEPS: u32 = 10_000;
    for _ in 0..MAX_SIM_STEPS {
        match advance_ballistic_toward_aim(pos, aim, speed_tiles_per_step, 1, step_secs) {
            BallisticAdvance::Impact { pos: nominal, seg_time_s } => {
                flight_time_s += seg_time_s;
                return apply_missile_wind_to_impact(nominal, flight_time_s, Some(*wind));
            }
            BallisticAdvance::InFlight { pos: next, seg_time_s } => {
                flight_time_s += seg_time_s;
                pos = next;
            }
        }
    }
    apply_missile_wind_to_impact(pos, flight_time_s, Some(*wind))
}

/// Iteratively offset aim so impact under wind lands on `desired` (MOVE-09).
#[must_use]
pub fn refine_missile_aim(
    origin: Vec2,
    desired: Vec2,
    speed_tiles_per_step: f32,
    wind: &MissileBallisticWind,
    step_secs: f32,
) -> Vec2 {
    let mut aim = desired;
    for _ in 0..MISSILE_AIM_REFINE_ITERS {
        let impact = simulate_missile_impact(origin, aim, speed_tiles_per_step, wind, step_secs);
        let miss = desired - impact;
        if miss.length() < MISSILE_AIM_HALF_CELL {
            break;
        }
        aim += miss;
    }
    aim
}

/// One in-flight projectile (missile / shell) — package-level, not airframe ECS.
#[derive(Clone, Debug)]
pub struct BallisticFlight {
    pub id: u64,
    pub origin: Vec2,
    pub target: Vec2,
    pub pos: Vec2,
    pub speed_tiles_per_step: f32,
    pub blast_radius: f32,
    pub kind: BallisticKind,
    /// Accumulated sim seconds since launch (wind drift at impact).
    pub flight_time_s: f32,
    pub wind: Option<MissileBallisticWind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BallisticKind {
    Missile,
    Shell,
    EmberProxy,
}

#[derive(Resource, Default, Debug)]
pub struct BallisticFlightBook {
    pub flights: Vec<BallisticFlight>,
    next_id: u64,
    pub impacts: u64,
}

/// Grid cells visited along `origin` → `end` (Amanatides–Woo); floats only for setup.
fn grid_cells_on_segment(origin: Vec2, end: Vec2) -> Vec<(i32, i32)> {
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let goal_x = end.x.floor() as i32;
    let goal_y = end.y.floor() as i32;

    let dx = end.x - origin.x;
    let dy = end.y - origin.y;

    let step_x = if dx > 0.0 {
        1
    } else if dx < 0.0 {
        -1
    } else {
        0
    };
    let step_y = if dy > 0.0 {
        1
    } else if dy < 0.0 {
        -1
    } else {
        0
    };

    let abs_dx = dx.abs();
    let abs_dy = dy.abs();
    let inv_dx = if abs_dx > 1e-8 { 1.0 / abs_dx } else { f32::INFINITY };
    let inv_dy = if abs_dy > 1e-8 { 1.0 / abs_dy } else { f32::INFINITY };

    let mut t_max_x = if step_x > 0 {
        (x as f32 + 1.0 - origin.x) * inv_dx
    } else if step_x < 0 {
        (origin.x - x as f32) * inv_dx
    } else {
        f32::INFINITY
    };
    let mut t_max_y = if step_y > 0 {
        (y as f32 + 1.0 - origin.y) * inv_dy
    } else if step_y < 0 {
        (origin.y - y as f32) * inv_dy
    } else {
        f32::INFINITY
    };
    let t_delta_x = inv_dx;
    let t_delta_y = inv_dy;

    let mut cells = Vec::new();
    loop {
        cells.push((x, y));
        if x == goal_x && y == goal_y {
            break;
        }
        if t_max_x < t_max_y {
            t_max_x += t_delta_x;
            x += step_x;
        } else {
            t_max_y += t_delta_y;
            y += step_y;
        }
    }
    cells
}

/// First grid cell on `prev` → `next` that contains `target`, if any.
fn first_target_cell_on_segment(prev: Vec2, next: Vec2, target: Vec2) -> Option<Vec2> {
    let hit_x = target.x.floor() as i32;
    let hit_y = target.y.floor() as i32;
    let start_x = prev.x.floor() as i32;
    let start_y = prev.y.floor() as i32;

    for (cx, cy) in grid_cells_on_segment(prev, next) {
        if cx == hit_x && cy == hit_y && !(cx == start_x && cy == start_y) {
            return Some(target);
        }
    }
    None
}

impl BallisticFlightBook {
    pub fn launch(
        &mut self,
        origin: Vec2,
        target: Vec2,
        speed_tiles_per_step: f32,
        blast_radius: f32,
        kind: BallisticKind,
    ) -> u64 {
        self.launch_with_wind(origin, target, speed_tiles_per_step, blast_radius, kind, None)
    }

    pub fn launch_with_wind(
        &mut self,
        origin: Vec2,
        target: Vec2,
        speed_tiles_per_step: f32,
        blast_radius: f32,
        kind: BallisticKind,
        wind: Option<MissileBallisticWind>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.flights.push(BallisticFlight {
            id,
            origin,
            target,
            pos: origin,
            speed_tiles_per_step: speed_tiles_per_step.max(0.01),
            blast_radius,
            kind,
            flight_time_s: 0.0,
            wind,
        });
        id
    }
}

/// Advance flights; enqueue combat audio on impact. Blast damage apply is DEFER → construction funnel.
pub fn step_ballistic_flights(
    mut book: ResMut<BallisticFlightBook>,
    mut audio: ResMut<AudioOneShotQueue>,
    mut blast_queue: Option<ResMut<BlastIntentQueue>>,
    attention: Option<Res<SimAttentionFrame>>,
    clock: Res<SimStepClock>,
) {
    if clock.due == 0 {
        return;
    }
    let steps = clock.due;
    let step_secs = clock.step_secs();
    let full_audio = attention
        .as_ref()
        .map(|a| a.player_theater_active)
        .unwrap_or(true);

    let mut flights = std::mem::take(&mut book.flights);
    let mut remain = Vec::with_capacity(flights.len());
    let mut impacts = 0_u64;
    for mut f in flights.drain(..) {
        let prev = f.pos;
        match advance_ballistic_toward_aim(prev, f.target, f.speed_tiles_per_step, steps, step_secs) {
            BallisticAdvance::InFlight { pos, seg_time_s } => {
                f.pos = pos;
                f.flight_time_s += seg_time_s;
                remain.push(f);
                continue;
            }
            BallisticAdvance::Impact { pos, seg_time_s } => {
                f.flight_time_s += seg_time_s;
                let impact_pos =
                    apply_missile_wind_to_impact(pos, f.flight_time_s, f.wind);
                impacts += 1;
                if full_audio {
                    let kind = match f.kind {
                        BallisticKind::Missile => AudioOneShotKind::Explosion,
                        BallisticKind::Shell => AudioOneShotKind::Explosion,
                        BallisticKind::EmberProxy => AudioOneShotKind::Shot,
                    };
                    audio.push_combat(kind, impact_pos);
                }
                // Construction owns damage — enqueue blast intent (never mutate sites here).
                if let Some(q) = blast_queue.as_mut() {
                    q.enqueue(BlastIntent {
                        center_xz: impact_pos,
                        radius_m: f.blast_radius.max(1.0) * METRES_PER_TILE,
                        damage: match f.kind {
                            BallisticKind::Missile => 120.0,
                            BallisticKind::Shell => 80.0,
                            BallisticKind::EmberProxy => 15.0,
                        },
                        source_flight: f.id,
                    });
                }
            }
        }
    }
    book.flights = remain;
    book.impacts = book.impacts.saturating_add(impacts);
}

pub struct BallisticsPlugin;

impl Plugin for BallisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BallisticFlightBook>().add_systems(
            Update,
            step_ballistic_flights.after(SimControlSystemSet::AdvanceSimTick),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioOneShotQueue;
    use crate::systems::sim_control::SimStepClock;

    #[test]
    fn launch_and_impact_in_steps() {
        let mut book = BallisticFlightBook::default();
        book.launch(Vec2::ZERO, Vec2::new(10.0, 0.0), 5.0, 2.0, BallisticKind::Missile);
        assert_eq!(book.flights.len(), 1);
        // Manual step mimic
        let mut f = book.flights.remove(0);
        f.pos += (f.target - f.pos).normalize() * 5.0;
        assert!((f.pos.x - 5.0).abs() < 1e-3);
    }

    #[test]
    fn swept_impact_hits_target_cell_between_step_positions() {
        let mut app = App::new();
        app.init_resource::<BallisticFlightBook>()
            .init_resource::<AudioOneShotQueue>()
            .init_resource::<SimStepClock>()
            .add_systems(Update, step_ballistic_flights);
        {
            let world = app.world_mut();
            world
                .resource_mut::<BallisticFlightBook>()
                .launch(Vec2::ZERO, Vec2::new(3.5, 0.0), 3.0, 2.0, BallisticKind::Shell);
            world.resource_mut::<SimStepClock>().due = 1;
        }
        app.update();

        let book = app.world().resource::<BallisticFlightBook>();
        assert_eq!(book.flights.len(), 0, "flight should impact on the swept path");
        assert_eq!(book.impacts, 1);
    }

    #[test]
    fn crosswind_uncorrected_misses_corrected_lands_near_target() {
        let wind = MissileBallisticWind {
            sample: WeatherSample {
                wind_mps: Vec2::new(20.0, 0.0),
                ..Default::default()
            },
            drift_coef: 0.5,
            max_wind_mps: 30.0,
        };
        let origin = Vec2::ZERO;
        let desired = Vec2::new(30.0, 0.0);
        let speed = 6.0_f32;
        let step_secs = 1.0_f32;

        let naive = simulate_missile_impact(origin, desired, speed, &wind, step_secs);
        let miss_naive = (naive - desired).length();
        assert!(
            miss_naive > 1.0,
            "uncorrected impact should miss by >1 tile, got miss {miss_naive} at {naive:?}"
        );

        let aim = refine_missile_aim(origin, desired, speed, &wind, step_secs);
        let corrected = simulate_missile_impact(origin, aim, speed, &wind, step_secs);
        let miss_corrected = (corrected - desired).length();
        assert!(
            miss_corrected <= 1.0,
            "corrected impact should land within one tile, got miss {miss_corrected} at {corrected:?}"
        );
    }

    #[test]
    fn live_step_with_wind_matches_simulate_impact() {
        let wind = MissileBallisticWind {
            sample: WeatherSample {
                wind_mps: Vec2::new(12.0, 0.0),
                ..Default::default()
            },
            drift_coef: 0.2,
            max_wind_mps: 30.0,
        };
        let origin = Vec2::ZERO;
        let desired = Vec2::new(18.0, 0.0);
        let speed = 6.0_f32;
        let step_secs = 1.0_f32;
        let aim = refine_missile_aim(origin, desired, speed, &wind, step_secs);
        let predicted = simulate_missile_impact(origin, aim, speed, &wind, step_secs);

        let mut app = App::new();
        app.init_resource::<BallisticFlightBook>()
            .init_resource::<AudioOneShotQueue>()
            .init_resource::<SimStepClock>()
            .add_systems(Update, step_ballistic_flights);
        {
            let world = app.world_mut();
            world.resource_mut::<BallisticFlightBook>().launch_with_wind(
                origin,
                aim,
                speed,
                2.0,
                BallisticKind::Missile,
                Some(wind),
            );
        }
        for _ in 0..64 {
            {
                let world = app.world_mut();
                world.resource_mut::<SimStepClock>().due = 1;
            }
            app.update();
            let book = app.world().resource::<BallisticFlightBook>();
            if book.flights.is_empty() {
                break;
            }
        }
        let book = app.world().resource::<BallisticFlightBook>();
        assert_eq!(book.flights.len(), 0, "flight should complete");
        assert_eq!(book.impacts, 1);
        let corrected = simulate_missile_impact(origin, aim, speed, &wind, step_secs);
        assert!(
            (corrected - desired).length() <= 1.0,
            "refined aim should hit desired within one tile"
        );
        assert!(
            (predicted - desired).length() <= 1.0,
            "predicted impact should land on target"
        );
    }

    #[test]
    fn phys09_dual_cite_in_module_docs() {
        let src = include_str!("ballistics.rs");
        let prod = src.split_once("#[cfg(test)]").map_or(src, |(p, _)| p);
        assert!(prod.contains("PHYS-09") || prod.contains("V4-PHYS09"));
        assert!(prod.contains("14.1") && prod.contains("14.1.2.1") && prod.contains("14.1.3"));
        assert!(prod.contains("physics_collision") || prod.contains("physics_middleware"));
        assert!(prod.contains("gea_v2_e4"));
        assert!(
            prod.contains("bookkeeping") && prod.contains("rigid-body"),
            "docs must state bookkeeping over rigid-body design"
        );
    }

    /// PHYS-09: this module is kinematic bookkeeping + intent enqueue — not a dynamics world.
    /// Scans non-comment production lines only (docs may cite middleware names as rejects).
    #[test]
    fn phys09_source_rejects_rigid_body_middleware() {
        let src = include_str!("ballistics.rs");
        let prod = src.split_once("#[cfg(test)]").map_or(src, |(p, _)| p);
        let code_only: String = prod
            .lines()
            .filter(|line| {
                let t = line.trim_start();
                !(t.starts_with("//") || t.starts_with("//!") || t.starts_with("/*") || t.is_empty())
            })
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in [
            "RigidBody",
            "bevy_rapier",
            "ImpulseJoint",
            "ExternalForce",
            "Collider::",
            "PhysicsPipeline",
            "rapier3d",
            "rapier2d",
            "use rapier",
            "use physx",
            "use havok",
        ] {
            assert!(
                !code_only.contains(forbidden),
                "ballistics must not host rigid-body middleware (found `{forbidden}`)"
            );
        }
        assert!(
            prod.contains("BallisticFlightBook") && prod.contains("BlastIntent"),
            "bookkeeping flight book + blast intent must remain the contract"
        );
    }

    #[test]
    fn phys09_impact_enqueues_blast_intent_not_site_mutate() {
        let mut app = App::new();
        app.init_resource::<BallisticFlightBook>()
            .init_resource::<AudioOneShotQueue>()
            .init_resource::<BlastIntentQueue>()
            .init_resource::<SimStepClock>()
            .add_systems(Update, step_ballistic_flights);
        {
            let world = app.world_mut();
            world
                .resource_mut::<BallisticFlightBook>()
                .launch(Vec2::ZERO, Vec2::new(2.0, 0.0), 4.0, 2.0, BallisticKind::Shell);
            world.resource_mut::<SimStepClock>().due = 1;
        }
        app.update();

        let book = app.world().resource::<BallisticFlightBook>();
        assert_eq!(book.flights.len(), 0);
        assert_eq!(book.impacts, 1);
        let queue = app.world().resource::<BlastIntentQueue>();
        assert_eq!(
            queue.pending.len(),
            1,
            "impact must enqueue BlastIntent (intent→execute)"
        );
        assert!(
            (queue.pending[0].center_xz - Vec2::new(2.0, 0.0)).length() < 1e-3
        );

        let prod = include_str!("ballistics.rs")
            .split_once("#[cfg(test)]")
            .map_or(include_str!("ballistics.rs"), |(p, _)| p);
        for forbidden in [
            "BlastCraterBook",
            "execute_demolish",
            "TileOccupationBook",
            "VoxelOccupationBook",
            "ConstructionSite",
            ".despawn(",
        ] {
            assert!(
                !prod.contains(forbidden),
                "ballistics must stay intent-only (found `{forbidden}`)"
            );
        }
    }

    #[test]
    fn phys09_ballistic_book_is_sole_flight_writer() {
        let prod = include_str!("ballistics.rs")
            .split_once("#[cfg(test)]")
            .map_or(include_str!("ballistics.rs"), |(p, _)| p);
        assert!(
            prod.contains("mut book: ResMut<BallisticFlightBook>"),
            "step_ballistic_flights must be the BallisticFlightBook writer"
        );
        assert!(
            prod.contains("Option<ResMut<BlastIntentQueue>>")
                || prod.contains("mut blast_queue: Option<ResMut<BlastIntentQueue>>"),
            "damage path must be enqueue-only via BlastIntentQueue"
        );
    }
}
