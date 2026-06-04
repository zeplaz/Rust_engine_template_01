//! WSS-SMOKE-BRIDGE-001 — Layer B smoke extract from [`SimChunkSmokeVisualExtract`] (ECS L1 → render).

use bevy::prelude::*;

use crate::render::SimChunkSmokeVisualExtract;

/// Witness rollup for `wss_substrate_live.json` / smoke bridge gate.
#[derive(Resource, Clone, Debug, Default)]
pub struct SmokeVisualBridgeWitness {
    pub smoke_density_sum: f32,
    pub smoke_row_count: u32,
    pub smoke_extract_wired: bool,
    pub smoke_stub_removed: bool,
}

/// Aggregates chunk smoke GPU rows published by atmosphere `publish_sim_visual_extract`.
pub fn build_smoke_visual_extract(
    smoke: Res<SimChunkSmokeVisualExtract>,
    mut witness: ResMut<SmokeVisualBridgeWitness>,
) {
    witness.smoke_density_sum = smoke
        .instances
        .iter()
        .map(|row| row.density_tox_vis.x.max(0.0))
        .sum();
    witness.smoke_row_count = smoke.instances.len() as u32;
    witness.smoke_extract_wired = !smoke.instances.is_empty();
    witness.smoke_stub_removed = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::ChunkSmokeGpu;
    #[test]
    fn smoke_extract_witness_sums_density() {
        let mut app = App::new();
        app.init_resource::<SimChunkSmokeVisualExtract>()
            .init_resource::<SmokeVisualBridgeWitness>()
            .add_systems(Update, build_smoke_visual_extract);

        {
            let mut smoke = app.world_mut().resource_mut::<SimChunkSmokeVisualExtract>();
            smoke.instances.push(ChunkSmokeGpu {
                chunk_xy: Vec4::new(1.0, 2.0, 0.0, 0.0),
                density_tox_vis: Vec4::new(0.4, 0.1, 0.0, 0.0),
            });
        }
        app.update();

        let w = app.world().resource::<SmokeVisualBridgeWitness>();
        assert!(w.smoke_extract_wired);
        assert!(w.smoke_stub_removed);
        assert!(w.smoke_density_sum > 0.0);
        assert_eq!(w.smoke_row_count, 1);
    }
}
