//! Fire **chunk runtime** + view/extraction scaffolding (CPU-only).
//!
//! Pipeline (intended): **Simulation → [`FireChunkRuntime`] → view extraction ([`VisibleFireChunkSet`])
//! → LOD ([`FireChunkLodState`]) → [`FireVisualFrame`] (render-facing)**.
//! GPU propagation stays out of this module.
//!
//! [`FireSimulationSnapshot`] is the single ECS-backed **sim** snapshot per frame; [`FireVisualFrame`]
//! is filled only from simulation + extraction policy (today: pass-through copy for parity).

use std::collections::{HashMap, HashSet};

use bevy::math::IVec2;
use bevy::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::gui::ViewId;
use crate::render::sim_visual_extract::{
    ChunkFireHeat, FireVisualGpuInstance, FIRE_VISUAL_ACTIVE_HEAT_EPS,
};
use crate::systems::sim_control::SimStepStamp;

/// Chunk index in the same space as [`ChunkFireHeat::chunk`] / fire visual rows.
pub type ChunkCoord = IVec2;

/// Any sim heat above this marks the chunk **active** for lifecycle (distinct from visual threshold).
pub const FIRE_SIM_CHUNK_ACTIVE_EPS: f32 = 1e-5;

/// Per-chunk CPU runtime (sim domain). Not a GPU struct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FireChunk {
    pub coord: ChunkCoord,
    pub active: bool,
    pub visual_active: bool,
    pub heat_sum: f32,
    pub max_heat: f32,
    pub last_active_tick: u32,
    pub dirty: bool,
}

impl Default for FireChunk {
    fn default() -> Self {
        Self {
            coord: ChunkCoord::ZERO,
            active: false,
            visual_active: false,
            heat_sum: 0.0,
            max_heat: 0.0,
            last_active_tick: 0,
            dirty: false,
        }
    }
}

/// Authoritative per-chunk sim rollup for the current frame (cleared + rebuilt from ECS).
#[derive(Resource, Default, Debug, Clone)]
pub struct FireChunkRuntime {
    pub chunks: HashMap<ChunkCoord, FireChunk>,
}

/// Full **simulation** fire snapshot (one ECS scan). Render must not read ECS fire components directly.
#[derive(Resource, Debug, Clone)]
pub struct FireSimulationSnapshot {
    pub stamp: SimStepStamp,
    pub instances: Vec<FireVisualGpuInstance>,
    pub chunk_heat: Vec<ChunkFireHeat>,
}

impl Default for FireSimulationSnapshot {
    fn default() -> Self {
        Self {
            stamp: SimStepStamp::default(),
            instances: Vec::new(),
            chunk_heat: Vec::new(),
        }
    }
}

impl FireSimulationSnapshot {
    #[must_use]
    pub fn chunk_coords_with_active_heat(&self) -> HashSet<ChunkCoord> {
        chunk_coords_above_visual_eps(&self.instances, &self.chunk_heat)
    }
}

/// Chunks with [`FireChunk::visual_active`] after the latest sim rollup.
#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveFireChunkSet {
    pub chunks: HashSet<ChunkCoord>,
}

/// View-scoped chunk visibility for fire: intersection of sim-active chunks with each view's
/// [`crate::gui::ViewInstance::visible_world_rect`], keyed by [`ViewId`]. Empty set means **no**
/// fire chunks for that view (does not fall back to full sim when the key is present).
#[derive(Resource, Default, Debug, Clone)]
pub struct VisibleFireChunkSet {
    pub per_view: FxHashMap<ViewId, FxHashSet<ChunkCoord>>,
}

/// Per-chunk fire **render** LOD band (visual domain; independent of world representation LOD).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FireLodBand {
    #[default]
    None,
    SmokeOnly,
    LowFlame,
    FullFlame,
}

/// Latest [`FireLodBand`] per chunk, derived from simulation heat (CPU stub policy).
#[derive(Resource, Default, Debug, Clone)]
pub struct FireChunkLodState {
    pub bands: HashMap<ChunkCoord, FireLodBand>,
}

#[must_use]
pub fn chunk_coords_above_visual_eps(
    instances: &[FireVisualGpuInstance],
    chunk_heat: &[ChunkFireHeat],
) -> HashSet<ChunkCoord> {
    let mut out = HashSet::default();
    for row in instances {
        if row.heat() <= FIRE_VISUAL_ACTIVE_HEAT_EPS {
            continue;
        }
        let xy = row.chunk_grid_xy();
        out.insert(ChunkCoord::new(xy.x as i32, xy.y as i32));
    }
    for h in chunk_heat {
        if h.heat > FIRE_VISUAL_ACTIVE_HEAT_EPS {
            out.insert(h.chunk);
        }
    }
    out
}

#[must_use]
pub fn fire_lod_band_for_visual_heat(heat: f32) -> FireLodBand {
    if heat <= FIRE_VISUAL_ACTIVE_HEAT_EPS {
        FireLodBand::None
    } else if heat < 0.08 {
        FireLodBand::SmokeOnly
    } else if heat < 0.35 {
        FireLodBand::LowFlame
    } else {
        FireLodBand::FullFlame
    }
}

pub fn sync_active_fire_chunk_set(runtime: Res<FireChunkRuntime>, mut set: ResMut<ActiveFireChunkSet>) {
    set.chunks = runtime
        .chunks
        .iter()
        .filter(|(_, c)| c.visual_active)
        .map(|(k, _)| *k)
        .collect();
}

pub fn sync_fire_chunk_lod_from_snapshot(sim: Res<FireSimulationSnapshot>, mut lod: ResMut<FireChunkLodState>) {
    *lod = fire_chunk_lod_state_from_simulation(sim.as_ref());
}

/// Rebuild per-chunk fire render LOD from a full sim snapshot (same policy as [`sync_fire_chunk_lod_from_snapshot`]).
#[must_use]
pub fn fire_chunk_lod_state_from_simulation(sim: &FireSimulationSnapshot) -> FireChunkLodState {
    let mut lod = FireChunkLodState::default();
    for h in &sim.chunk_heat {
        lod.bands
            .insert(h.chunk, fire_lod_band_for_visual_heat(h.heat));
    }
    for row in &sim.instances {
        let xy = row.chunk_grid_xy();
        let c = ChunkCoord::new(xy.x as i32, xy.y as i32);
        let band = fire_lod_band_for_visual_heat(row.heat());
        lod.bands
            .entry(c)
            .and_modify(|b| *b = merge_fire_lod_band_for_heat(*b, row.heat()))
            .or_insert(band);
    }
    lod
}

impl FireLodBand {
    #[inline]
    fn rank(self) -> u8 {
        match self {
            FireLodBand::None => 0,
            FireLodBand::SmokeOnly => 1,
            FireLodBand::LowFlame => 2,
            FireLodBand::FullFlame => 3,
        }
    }
}

#[inline]
fn merge_fire_lod_band_for_heat(current: FireLodBand, heat: f32) -> FireLodBand {
    let next = fire_lod_band_for_visual_heat(heat);
    if next.rank() >= current.rank() {
        next
    } else {
        current
    }
}
