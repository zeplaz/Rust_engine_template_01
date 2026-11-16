//! **MOVE-10** — minimal seed-driven wildlife flock (ecology domain).
//!
//! aig3 15.2 flocking / 15.2.2 interactive tuning / 15.2.3 steering stability.
//! Soft path costs are sim-affecting; ⛔ never writes combat `Formation.tile`.

use bevy::math::{IVec2, Vec2};
use bevy::prelude::*;

use crate::ai::faction::rng::derive_keyed_seed;

/// World units: separation neighborhood (wider than depot MOVE-03 — herd scale).
pub const WILDLIFE_SEP_RADIUS: f32 = 2.5;
pub const WILDLIFE_SEP_STRENGTH: f32 = 0.9;
/// Cohesion / alignment gains (kept small for 15.2.3 stability).
pub const WILDLIFE_COHESION: f32 = 0.12;
pub const WILDLIFE_ALIGNMENT: f32 = 0.18;
/// Max speed (world units / step) — hard clamp for steering stability.
pub const WILDLIFE_MAX_SPEED: f32 = 0.55;
/// Soft path penalty per nearby agent (clamped).
pub const WILDLIFE_SOFT_COST_PER: f32 = 0.08;
pub const WILDLIFE_SOFT_COST_MAX: f32 = 0.45;
/// Default herd size for a seeded spawn.
pub const WILDLIFE_DEFAULT_COUNT: u32 = 6;

const STREAM_SPAWN: u64 = 0x4d31_3057; // "M10W"
const STREAM_POS: u64 = 0x504f_5358; // "POSX"
const STREAM_VEL: u64 = 0x5645_4c59; // "VELY"

/// One ecology wildlife agent — not a combat formation member.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WildlifeAgent {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
}

/// Chunk soft cost sample for nav RO (sim-affecting).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WildlifeSoftCostSample {
    pub chunk: IVec2,
    /// 0..[`WILDLIFE_SOFT_COST_MAX`] path penalty.
    pub soft_cost: f32,
}

/// Sole ecology writer for minimal wildlife / crowd agents (**MOVE-10**).
#[derive(Resource, Clone, Debug, Default)]
pub struct WildlifeHerdBook {
    pub seed: u64,
    pub agents: Vec<WildlifeAgent>,
    pub soft_cost_samples: Vec<WildlifeSoftCostSample>,
    pub step: u64,
}

impl WildlifeHerdBook {
    #[must_use]
    pub fn from_seed(seed: u64, count: u32, origin: Vec2) -> Self {
        let mut book = Self {
            seed,
            agents: spawn_wildlife_agents(seed, count, origin),
            soft_cost_samples: Vec::new(),
            step: 0,
        };
        book.rebuild_soft_costs();
        book
    }

    pub fn rebuild_soft_costs(&mut self) {
        self.soft_cost_samples = soft_costs_from_agents(&self.agents);
    }
}

/// Deterministic spawn ring around `origin` (seeded).
#[must_use]
pub fn spawn_wildlife_agents(seed: u64, count: u32, origin: Vec2) -> Vec<WildlifeAgent> {
    let n = count.max(1).min(32);
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let s = derive_keyed_seed(seed, STREAM_SPAWN ^ u64::from(i));
        let px = derive_keyed_seed(s, STREAM_POS);
        let py = derive_keyed_seed(s, STREAM_POS ^ 1);
        let vx = derive_keyed_seed(s, STREAM_VEL);
        let vy = derive_keyed_seed(s, STREAM_VEL ^ 1);
        let angle = (i as f32) * (std::f32::consts::TAU / n as f32);
        let radius = 1.0 + unit01(px) * 1.5;
        let position = origin + Vec2::new(angle.cos(), angle.sin()) * radius;
        let velocity = Vec2::new(unit01(vx) - 0.5, unit01(vy) - 0.5) * WILDLIFE_MAX_SPEED;
        out.push(WildlifeAgent {
            id: i,
            position,
            velocity: clamp_speed(velocity, WILDLIFE_MAX_SPEED),
        });
    }
    out
}

#[inline]
fn unit01(bits: u64) -> f32 {
    ((bits >> 40) as u32 as f32) * (1.0 / 16_777_216.0)
}

#[inline]
#[must_use]
pub fn clamp_speed(v: Vec2, max_speed: f32) -> Vec2 {
    let speed = v.length();
    if speed <= max_speed || speed < f32::EPSILON {
        return v;
    }
    v * (max_speed / speed)
}

/// One flock step: separation + cohesion + alignment, then speed clamp (aig3 15.2.3).
#[must_use]
pub fn flock_step(agents: &[WildlifeAgent]) -> Vec<WildlifeAgent> {
    let positions: Vec<Vec2> = agents.iter().map(|a| a.position).collect();
    let mut next = Vec::with_capacity(agents.len());
    for (i, agent) in agents.iter().enumerate() {
        let neighbors: Vec<Vec2> = positions
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, p)| *p)
            .collect();
        let sep = wildlife_separation(
            agent.position,
            &neighbors,
            WILDLIFE_SEP_RADIUS,
            WILDLIFE_SEP_STRENGTH,
        );
        let (cohesion, alignment) = cohesion_alignment(agent, agents, i);
        let steer = sep + cohesion * WILDLIFE_COHESION + alignment * WILDLIFE_ALIGNMENT;
        let velocity = clamp_speed(agent.velocity + steer, WILDLIFE_MAX_SPEED);
        next.push(WildlifeAgent {
            id: agent.id,
            position: agent.position + velocity,
            velocity,
        });
    }
    next
}

/// Same formula as MOVE-03 separation (kept local — avoid ecology↔nav cycle).
fn wildlife_separation(position: Vec2, neighbors: &[Vec2], radius: f32, strength: f32) -> Vec2 {
    let r2 = radius * radius;
    let mut push = Vec2::ZERO;
    for &other in neighbors {
        let delta = position - other;
        let d2 = delta.length_squared();
        if d2 < f32::EPSILON || d2 > r2 {
            continue;
        }
        let d = d2.sqrt();
        push += (delta / d) * (strength * (radius - d) / radius);
    }
    push
}

fn cohesion_alignment(
    agent: &WildlifeAgent,
    agents: &[WildlifeAgent],
    self_idx: usize,
) -> (Vec2, Vec2) {
    let mut center = Vec2::ZERO;
    let mut mean_vel = Vec2::ZERO;
    let mut n = 0u32;
    for (j, other) in agents.iter().enumerate() {
        if j == self_idx {
            continue;
        }
        let d2 = (other.position - agent.position).length_squared();
        if d2 > WILDLIFE_SEP_RADIUS * WILDLIFE_SEP_RADIUS * 4.0 {
            continue;
        }
        center += other.position;
        mean_vel += other.velocity;
        n += 1;
    }
    if n == 0 {
        return (Vec2::ZERO, Vec2::ZERO);
    }
    let inv = 1.0 / n as f32;
    let cohesion = (center * inv) - agent.position;
    let alignment = (mean_vel * inv) - agent.velocity;
    (cohesion, alignment)
}

/// Aggregate soft path costs by chunk (sim-affecting occupancy signal).
#[must_use]
pub fn soft_costs_from_agents(agents: &[WildlifeAgent]) -> Vec<WildlifeSoftCostSample> {
    let mut samples: Vec<WildlifeSoftCostSample> = Vec::new();
    for agent in agents {
        let chunk = IVec2::new(agent.position.x.floor() as i32, agent.position.y.floor() as i32);
        if let Some(row) = samples.iter_mut().find(|s| s.chunk == chunk) {
            row.soft_cost = (row.soft_cost + WILDLIFE_SOFT_COST_PER).min(WILDLIFE_SOFT_COST_MAX);
        } else {
            samples.push(WildlifeSoftCostSample {
                chunk,
                soft_cost: WILDLIFE_SOFT_COST_PER.min(WILDLIFE_SOFT_COST_MAX),
            });
        }
    }
    samples.sort_by_key(|s| (s.chunk.y, s.chunk.x));
    samples
}

/// Advance herd one step and refresh soft costs.
pub fn tick_wildlife_herd(book: &mut WildlifeHerdBook) {
    if book.agents.is_empty() {
        return;
    }
    book.agents = flock_step(&book.agents);
    book.rebuild_soft_costs();
    book.step = book.step.saturating_add(1);
}

/// RO helper: soft cost at chunk (0 if none).
#[must_use]
pub fn soft_cost_at(book: &WildlifeHerdBook, chunk: IVec2) -> f32 {
    book.soft_cost_samples
        .iter()
        .find(|s| s.chunk == chunk)
        .map(|s| s.soft_cost)
        .unwrap_or(0.0)
}

/// Sync wildlife soft samples into nav snapshot (RO projection; nav does not write wildlife).
pub fn sync_nav_wildlife_soft(
    nav: &mut crate::compute::NavFieldFrame,
    book: &WildlifeHerdBook,
) {
    nav.wildlife_soft_samples.clear();
    for s in &book.soft_cost_samples {
        nav.wildlife_soft_samples.push(crate::compute::NavWildlifeSoftSample {
            chunk: s.chunk,
            soft_cost: s.soft_cost,
        });
    }
}

/// Sync wildlife count into agent frame (ecology → compute RO).
pub fn sync_agent_frame_from_wildlife(
    agents: &mut crate::compute::AgentFrame,
    book: &WildlifeHerdBook,
) {
    agents.agent_count = book.agents.len() as u32;
}

pub fn tick_wildlife_herd_system(mut book: ResMut<WildlifeHerdBook>) {
    tick_wildlife_herd(&mut book);
}

pub fn register_wildlife_flock(app: &mut App) {
    app.init_resource::<WildlifeHerdBook>().add_systems(
        Update,
        tick_wildlife_herd_system.in_set(crate::systems::chunk_environment_set::ChunkEnvironmentSet::Ecology),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move10_seed_determinism() {
        let a = spawn_wildlife_agents(0xC0FFEE, WILDLIFE_DEFAULT_COUNT, Vec2::new(10.0, 20.0));
        let b = spawn_wildlife_agents(0xC0FFEE, WILDLIFE_DEFAULT_COUNT, Vec2::new(10.0, 20.0));
        assert_eq!(a, b);
        let c = spawn_wildlife_agents(0xC0FFEE ^ 1, WILDLIFE_DEFAULT_COUNT, Vec2::new(10.0, 20.0));
        assert_ne!(a[0].position, c[0].position);
    }

    #[test]
    fn move10_flock_stability_and_soft_cost() {
        let mut book = WildlifeHerdBook::from_seed(42, WILDLIFE_DEFAULT_COUNT, Vec2::ZERO);
        assert_eq!(book.agents.len(), WILDLIFE_DEFAULT_COUNT as usize);
        assert!(!book.soft_cost_samples.is_empty());
        let cost0 = book.soft_cost_samples.iter().map(|s| s.soft_cost).sum::<f32>();
        assert!(cost0 > 0.0);

        for _ in 0..8 {
            tick_wildlife_herd(&mut book);
            for a in &book.agents {
                assert!(a.velocity.length() <= WILDLIFE_MAX_SPEED + 1e-4);
            }
        }
        assert!(book.step == 8);
        assert!(soft_cost_at(&book, book.soft_cost_samples[0].chunk) > 0.0);

        // Separation: stacked pair should push apart after a step.
        let stacked = vec![
            WildlifeAgent {
                id: 0,
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
            },
            WildlifeAgent {
                id: 1,
                position: Vec2::new(0.1, 0.0),
                velocity: Vec2::ZERO,
            },
        ];
        let next = flock_step(&stacked);
        let dist0 = (stacked[1].position - stacked[0].position).length();
        let dist1 = (next[1].position - next[0].position).length();
        assert!(dist1 > dist0);
    }

    #[test]
    fn move10_nav_and_agent_sync() {
        let book = WildlifeHerdBook::from_seed(7, 4, Vec2::new(3.2, -1.1));
        let mut nav = crate::compute::NavFieldFrame::default();
        let mut agents = crate::compute::AgentFrame::default();
        sync_nav_wildlife_soft(&mut nav, &book);
        sync_agent_frame_from_wildlife(&mut agents, &book);
        assert_eq!(agents.agent_count, 4);
        assert_eq!(nav.wildlife_soft_samples.len(), book.soft_cost_samples.len());
        assert!(nav.wildlife_soft_samples.iter().all(|s| s.soft_cost > 0.0));
    }
}
