//! Adaptive **sim LOD** per chunk — scales local dt for weather / fire / ecology (runbook direction).
//!
//! Full chunk scheduler lives in [`prompts/guides/chunk_scheduler_runbook_v1.md`](../../../prompts/guides/chunk_scheduler_runbook_v1.md);
//! this is a lightweight scalar tier until that lands.

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Simulation attention tier for chunk-local environment systems.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkSimLod {
    /// Low fuel / calm weather — cheaper ticks.
    Dormant,
    Normal,
    /// Active burn, heavy precip, high wind, or lightning — more frequent effective integration.
    Active,
}

impl ChunkSimLod {
    #[inline]
    pub fn dt_scale(self) -> f32 {
        match self {
            ChunkSimLod::Dormant => 0.4,
            ChunkSimLod::Normal => 1.0,
            ChunkSimLod::Active => 2.0,
        }
    }
}

impl Default for ChunkSimLod {
    fn default() -> Self {
        ChunkSimLod::Normal
    }
}

pub(crate) fn spawn_chunk_sim_lod_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkSimLod>)>,
) {
    for e in &q {
        commands.entity(e).insert(ChunkSimLod::default());
    }
}

/// Uses **current** [`ChunkWeather`] + [`ChunkSurfaceFire`] (end of last tick) to pick tier for *this* tick’s scaled dt.
pub fn chunk_sim_lod_refresh(
    mut q: Query<(
        Option<&ChunkCellMatrix>,
        Option<&ChunkSurfaceFire>,
        Option<&ChunkWeather>,
        &mut ChunkSimLod,
    )>,
) {
    for (matrix_opt, fire_opt, wx_opt, mut lod) in &mut q {
        let fire = fire_opt.copied().unwrap_or_default();
        let wx = wx_opt.cloned().unwrap_or_default();

        let mut mean_m = None::<f32>;
        if let Some(matrix) = matrix_opt {
            let n = (matrix.size.x * matrix.size.y) as usize;
            if matrix.moisture.len() == n && n > 0 {
                let sum: f32 = matrix.moisture.iter().sum();
                mean_m = Some(sum / n as f32);
            }
        }

        let intense_weather = wx.rain_intensity > 0.45
            || wx.lightning_risk > 0.35
            || wx.wind_speed > 0.78;
        let active_fire = fire.heat > 0.12;
        let calm_surface = mean_m.map_or(true, |m| m > 0.28);

        *lod = if active_fire || intense_weather {
            ChunkSimLod::Active
        } else if fire.heat < 0.015
            && wx.rain_intensity < 0.08
            && wx.wind_speed < 0.22
            && calm_surface
        {
            ChunkSimLod::Dormant
        } else {
            ChunkSimLod::Normal
        };
    }
}

pub struct ChunkSimLodPlugin;

impl Plugin for ChunkSimLodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_chunk_sim_lod_on_new_chunk.in_set(ChunkEnvironmentSet::Lod),
                chunk_sim_lod_refresh.in_set(ChunkEnvironmentSet::Lod),
            )
                .chain(),
        );
    }
}
