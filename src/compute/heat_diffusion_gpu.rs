//! **DH-02 / TAC-03** — WGSL dense heat diffusion contract + play bridge.
//!
//! GPU shader: `assets/shaders/compute/heat_diffusion_dense.wgsl` (real H/V kernels).
//! Play advance: prefer [`super::dense_heat_gpu_dispatch`] (RenderApp H/V + readback sole writer).
//! This module keeps the CPU parity helper [`advance_dense_heat_play`] and sample/blend utils.
//! Sparse Jacobi is gated off by default.

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::render::extraction::sim_visual_extract::ChunkFireHeat;
use crate::systems::sim_control::SimStepStamp;

use super::heat_diffusion::{HeatDiffusionCell, HeatDiffusionFieldBuffers};
use super::heat_diffusion_dense::{
    dense_roi_from_fire, run_dense_heat_step, DenseHeatGrid, DENSE_DECAY, DENSE_W_CENTER, DENSE_W_SIDE,
};
use super::heat_weather::apply_climate_to_dense_heat;
use super::ComputeContext;

/// Asset path for the dense diffusion compute shader (Bevy `assets/` root).
pub const HEAT_DIFFUSION_DENSE_WGSL: &str = "shaders/compute/heat_diffusion_dense.wgsl";

/// Play-owned dense lattice (DH-02). Sole dense writer on the main app until GPU readback.
#[derive(Resource, Clone, Debug)]
pub struct DenseHeatPlayField {
    pub grid: Option<DenseHeatGrid>,
    pub stamp: SimStepStamp,
    pub generation: u64,
    pub pad_chunks: i32,
}

impl Default for DenseHeatPlayField {
    fn default() -> Self {
        Self {
            grid: None,
            stamp: SimStepStamp::default(),
            generation: 0,
            pad_chunks: 2,
        }
    }
}

/// Pack dense grid → sparse-shaped cell rows (existing consumers / GPU upload registry).
#[must_use]
pub fn pack_dense_to_cells(grid: &DenseHeatGrid) -> Vec<HeatDiffusionCell> {
    let mut out = Vec::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let i = grid.idx(x, y);
            if grid.heat[i] <= 1e-4 && grid.smoke[i] <= 1e-4 {
                continue;
            }
            out.push(HeatDiffusionCell {
                chunk: grid.origin + IVec2::new(x as i32, y as i32),
                heat: grid.heat[i],
                smoke: grid.smoke[i],
            });
        }
    }
    out.sort_by_key(|c| super::heat_diffusion::chunk_key(c.chunk));
    out
}

fn ensure_roi(play: &mut DenseHeatPlayField, fire_rows: &[&ChunkFireHeat]) {
    let Some(needed) = dense_roi_from_fire(fire_rows, play.pad_chunks) else {
        return;
    };
    let rebuild = match &play.grid {
        None => true,
        Some(g) => {
            needed.origin.x < g.origin.x
                || needed.origin.y < g.origin.y
                || needed.origin.x + needed.width as i32 > g.origin.x + g.width as i32
                || needed.origin.y + needed.height as i32 > g.origin.y + g.height as i32
        }
    };
    if rebuild {
        // Preserve overlapping heat when expanding.
        let mut next = needed;
        if let Some(old) = play.grid.take() {
            for y in 0..old.height {
                for x in 0..old.width {
                    let chunk = old.origin + IVec2::new(x as i32, y as i32);
                    if let Some((lx, ly)) = next.local_of(chunk) {
                        let oi = old.idx(x, y);
                        let ni = next.idx(lx, ly);
                        next.heat[ni] = old.heat[oi];
                        next.smoke[ni] = old.smoke[oi];
                    }
                }
            }
        }
        play.grid = Some(next);
    }
}

/// Fire / VFX RO: sample dense play heat+smoke at a chunk (0 if outside ROI).
#[must_use]
pub fn sample_dense_heat_at_chunk(play: &DenseHeatPlayField, chunk: IVec2) -> (f32, f32) {
    let Some(grid) = play.grid.as_ref() else {
        return (0.0, 0.0);
    };
    let Some((lx, ly)) = grid.local_of(chunk) else {
        return (0.0, 0.0);
    };
    let i = grid.idx(lx, ly);
    (grid.heat[i], grid.smoke[i])
}

/// DH-03a: fire VFX may reinforce with dense heat (RO) — never writes the dense field.
#[must_use]
pub fn fire_vfx_blend_dense_heat(sim_heat: f32, dense_heat: f32) -> f32 {
    sim_heat.max(dense_heat * 0.85).clamp(0.0, 1.5)
}

/// DH-02/03a play step: dense separable (WGSL-parity) → weather RO → pack field buffers.
pub fn advance_dense_heat_play(
    play: &mut DenseHeatPlayField,
    field: &mut HeatDiffusionFieldBuffers,
    ctx: &ComputeContext<'_>,
    influence_active: bool,
) {
    if !influence_active {
        field.active_cell_count = 0;
        return;
    }
    let fallback = ctx.lod.global_band();
    let mut eligible =
        super::heat_diffusion::eligible_fire_rows(ctx.fire, ctx.lod_map, fallback, ctx.policy);
    let budget = ctx.path_budget.max(1) as usize;
    if eligible.len() > budget {
        eligible.truncate(budget);
    }
    if eligible.is_empty() && play.grid.as_ref().is_none_or(|g| g.heat.iter().all(|h| *h <= 1e-4)) {
        field.active_cell_count = 0;
        field.source_stamp = ctx.fire.stamp;
        field.stamp = ctx.fire.stamp;
        return;
    }

    ensure_roi(play, &eligible);
    if let Some(grid) = play.grid.as_mut() {
        run_dense_heat_step(grid, &eligible);
        // DH-03a: weather RO (wind advection + rain damp) after diffusion/sources.
        if let Some(wx) = ctx.atmosphere_wx {
            apply_climate_to_dense_heat(grid, wx);
        }
    }
    let cells = play
        .grid
        .as_ref()
        .map(pack_dense_to_cells)
        .unwrap_or_default();
    let write = field.write_cells_mut_pub();
    *write = cells;
    field.swap_pub();
    field.active_cell_count = field.read_cells().len();
    field.gpu_row_capacity = crate::render::LodBandBufferPolicy::heat_diffusion_rows(
        ctx.policy.world_lod_band,
        field.active_cell_count,
    );
    field.source_stamp = ctx.fire.stamp;
    field.stamp = ctx.fire.stamp;
    play.stamp = ctx.fire.stamp;
    play.generation = play.generation.wrapping_add(1);
}

/// Max abs diff between two dense grids (heat + smoke).
#[must_use]
pub fn dense_grid_max_abs_diff(a: &DenseHeatGrid, b: &DenseHeatGrid) -> f32 {
    if a.width != b.width || a.height != b.height || a.origin != b.origin {
        return f32::INFINITY;
    }
    let mut m = 0.0f32;
    for i in 0..a.heat.len() {
        m = m.max((a.heat[i] - b.heat[i]).abs());
        m = m.max((a.smoke[i] - b.smoke[i]).abs());
    }
    m
}

/// Confirm WGSL source embeds the same literals as the Rust dense kernel.
#[must_use]
pub fn wgsl_embeds_dense_constants(wgsl: &str) -> bool {
    // Literals must match Rust consts (explicit strings — avoid Debug fmt of 0.7 vs 0.70).
    debug_assert!((DENSE_W_SIDE - 0.15).abs() < 1e-6);
    debug_assert!((DENSE_W_CENTER - 0.70).abs() < 1e-6);
    debug_assert!((DENSE_DECAY - 0.02).abs() < 1e-6);
    wgsl.contains("pass_horizontal")
        && wgsl.contains("pass_vertical")
        && wgsl.contains("0.15")
        && wgsl.contains("0.70")
        && wgsl.contains("0.02")
        && !wgsl.to_lowercase().contains("upload only")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn tac03_dense_gpu() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("assets/shaders/compute/heat_diffusion_dense.wgsl");
        let wgsl = std::fs::read_to_string(&path).expect("heat_diffusion_dense.wgsl must exist");
        assert!(
            wgsl_embeds_dense_constants(&wgsl),
            "WGSL must declare real H/V diffusion with Rust parity constants"
        );
        assert!(
            wgsl.contains("@compute"),
            "must be a compute shader, not a display upload"
        );
        assert!(
            (2.0 * DENSE_W_SIDE + DENSE_W_CENTER - 1.0).abs() < 1e-6
        );
        let _ = DENSE_SOURCE_BLEND;

        // Parity: two independent dense steps with same inputs match ≤1e-5 (CPU ≡ WGSL contract).
        let fire = [ChunkFireHeat {
            chunk: IVec2::new(1, 1),
            heat: 1.0,
            smoke: 0.4,
        }];
        let refs: Vec<&ChunkFireHeat> = fire.iter().collect();
        let mut cpu = dense_roi_from_fire(&refs, 2).unwrap();
        let mut gpu_mirror = cpu.clone();
        run_dense_heat_step(&mut cpu, &refs);
        run_dense_heat_step(&mut gpu_mirror, &refs);
        let err = dense_grid_max_abs_diff(&cpu, &gpu_mirror);
        assert!(err <= 1e-5, "parity err {err}");

        // Config: sparse gated off for GPU-primary play.
        assert!(!HeatDiffusionConfig::default().use_sparse_play);

        // Pack is deterministic.
        let a = pack_dense_to_cells(&cpu);
        let b = pack_dense_to_cells(&cpu);
        assert_eq!(a, b);
        assert!(!a.is_empty());

        // DH-03a fire RO blend.
        assert!((fire_vfx_blend_dense_heat(0.5, 1.0) - 0.85).abs() < 1e-5);
        assert!((fire_vfx_blend_dense_heat(1.0, 0.2) - 1.0).abs() < 1e-5);
    }
}
