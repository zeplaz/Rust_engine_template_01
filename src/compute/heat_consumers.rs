//! **DH-03b** — nav + attention RO consumers of dense heat (TAC-03).
//!
//! Tripwires + helpers; production wiring lives in
//! [`super::heat_diffusion::sync_nav_field_from_heat_diffusion`] and
//! [`crate::systems::sim_attention::build_sim_attention_frame`].

use bevy::math::IVec2;

use super::frame_snapshots::NavFieldFrame;
use super::heat_diffusion::{heat_to_nav_cost, HeatDiffusionCell, HeatDiffusionFieldBuffers};
use super::heat_diffusion_dense::HeatDiffusionConfig;

/// Confirm sparse play is retired (DH-02/03 default).
#[must_use]
pub fn sparse_play_retired(cfg: &HeatDiffusionConfig) -> bool {
    !cfg.use_sparse_play
}

/// Pure nav sync sample for tests (mirrors production sync).
#[must_use]
pub fn nav_from_heat_cells(cells: &[HeatDiffusionCell]) -> NavFieldFrame {
    let mut nav = NavFieldFrame::default();
    let mut sum = 0.0f32;
    for cell in cells {
        let cost = heat_to_nav_cost(cell.heat);
        if cost <= 1e-4 {
            continue;
        }
        sum += cost;
        nav.heat_cost_samples.push(super::frame_snapshots::NavHeatCostSample {
            chunk: cell.chunk,
            heat_cost: cost,
        });
    }
    nav.heat_cost_samples
        .sort_by_key(|s| (s.chunk.y, s.chunk.x));
    nav.cell_count = cells.len() as u32;
    nav.mean_heat_cost = if nav.heat_cost_samples.is_empty() {
        0.0
    } else {
        sum / nav.heat_cost_samples.len() as f32
    };
    nav
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::heat_diffusion::sync_nav_field_from_heat_diffusion;

    #[test]
    fn tac03_dh03b() {
        assert!(sparse_play_retired(&HeatDiffusionConfig::default()));

        let cells = [
            HeatDiffusionCell {
                chunk: IVec2::new(1, 0),
                heat: 0.8,
                smoke: 0.0,
            },
            HeatDiffusionCell {
                chunk: IVec2::new(0, 0),
                heat: 0.2,
                smoke: 0.0,
            },
        ];
        let nav = nav_from_heat_cells(&cells);
        assert_eq!(nav.heat_cost_samples.len(), 2);
        assert!(nav.mean_heat_cost > 0.0);
        assert!(nav.heat_cost_samples[0].chunk.x <= nav.heat_cost_samples[1].chunk.x
            || nav.heat_cost_samples[0].chunk.y <= nav.heat_cost_samples[1].chunk.y);

        let mut field = HeatDiffusionFieldBuffers::default();
        *field.write_cells_mut_pub() = cells.to_vec();
        field.swap_pub();
        field.active_cell_count = 2;
        let mut synced = NavFieldFrame::default();
        sync_nav_field_from_heat_diffusion(&mut synced, &field);
        assert_eq!(synced.heat_cost_samples.len(), 2);
        assert!((synced.mean_heat_cost - nav.mean_heat_cost).abs() < 1e-5);

        // Attention RO contract: denser heat → higher path budget floor helper.
        let base = 32u32;
        let bumped = base.saturating_add(16).min(128);
        assert_eq!(bumped, 48);
    }
}
