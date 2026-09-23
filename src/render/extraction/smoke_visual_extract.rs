//! WSS-SMOKE-BRIDGE-001 — Layer B smoke extract from [`SimChunkSmokeVisualExtract`] (ECS L1 → render).

use bevy::prelude::*;

use crate::render::extraction::SmokeProjectionNode;
use crate::render::SimChunkSmokeVisualExtract;

/// Witness rollup for `wss_substrate_live.json` / smoke bridge gate / stage5.
#[derive(Resource, Clone, Debug, Default)]
pub struct SmokeVisualBridgeWitness {
    pub smoke_density_sum: f32,
    pub smoke_row_count: u32,
    pub smoke_extract_wired: bool,
    pub smoke_stub_removed: bool,
}

impl SmokeVisualBridgeWitness {
    /// Merge Layer-B smoke keys into stage5 / effects witness JSON.
    pub fn merge_into_stage5_json(&self, root: &mut serde_json::Value, smoke_proj: Option<&SmokeProjectionNode>) {
        let wired = self.smoke_extract_wired
            || smoke_proj.map(|s| s.extract_wired).unwrap_or(false);
        let stub_removed = self.smoke_stub_removed;
        let density = if self.smoke_density_sum > 0.0 {
            self.smoke_density_sum
        } else {
            smoke_proj.map(|s| s.density_sum).unwrap_or(0.0)
        };
        let rows = if self.smoke_row_count > 0 {
            self.smoke_row_count
        } else {
            smoke_proj.map(|s| s.row_count).unwrap_or(0)
        };
        if let Some(obj) = root.as_object_mut() {
            obj.insert("smoke_extract_wired".into(), serde_json::json!(wired));
            obj.insert("smoke_stub_removed".into(), serde_json::json!(stub_removed));
            obj.insert(
                "smoke_bridge".into(),
                serde_json::json!({
                    "density_sum": density,
                    "row_count": rows,
                    "projection_wired": smoke_proj.map(|s| s.extract_wired).unwrap_or(false),
                    "stub_removed": stub_removed,
                }),
            );
        }
    }
}

/// Aggregates chunk smoke GPU rows published by atmosphere `publish_sim_visual_extract`.
pub fn build_smoke_visual_extract(
    coherence: Option<Res<crate::render::FireExtractDiagnostics>>,
    smoke: Res<SimChunkSmokeVisualExtract>,
    mut witness: ResMut<SmokeVisualBridgeWitness>,
) {
    if coherence.as_deref().is_some_and(|d| d.snapshot_unchanged) {
        return;
    }
    witness.smoke_density_sum = smoke
        .instances
        .iter()
        .map(|row| row.density_tox_vis.x.max(0.0))
        .sum();
    witness.smoke_row_count = smoke.instances.len() as u32;
    // Path is wired when extract resource is live (stub retired); non-empty proves active smoke.
    witness.smoke_extract_wired = !smoke.instances.is_empty();
    witness.smoke_stub_removed = true;
}

/// Headless fixture for stage5 / ES-3 witness merge (non-empty smoke → projection node).
#[must_use]
pub fn smoke_bridge_projection_fixture() -> (SmokeVisualBridgeWitness, SmokeProjectionNode) {
    use crate::render::extraction::sim_visual_extract::ChunkSmokeGpu;

    let mut extract = SimChunkSmokeVisualExtract::default();
    extract.instances.push(ChunkSmokeGpu {
        chunk_xy: Vec4::new(2.0, 3.0, 0.0, 0.0),
        density_tox_vis: Vec4::new(0.55, 0.12, 0.0, 0.0),
    });
    let mut node = SmokeProjectionNode::default();
    node.project_from_extract(&extract, 9);
    let witness = SmokeVisualBridgeWitness {
        smoke_density_sum: node.density_sum,
        smoke_row_count: node.row_count,
        smoke_extract_wired: node.extract_wired,
        smoke_stub_removed: true,
    };
    (witness, node)
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

    #[test]
    fn smoke_extract_wired_projection_fixture_feeds_stage5_keys() {
        let (witness, node) = smoke_bridge_projection_fixture();
        assert!(witness.smoke_extract_wired);
        assert!(witness.smoke_stub_removed);
        assert!(node.extract_wired);
        assert!(node.density_sum > 0.0);

        let mut root = serde_json::json!({ "profile": "FULL_APP" });
        witness.merge_into_stage5_json(&mut root, Some(&node));
        assert_eq!(root["smoke_extract_wired"], true);
        assert_eq!(root["smoke_stub_removed"], true);
        assert_eq!(root["smoke_bridge"]["projection_wired"], true);
    }
}
