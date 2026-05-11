//! Emergent **frontline** metrics from control-field competition (not a placed object).

use bevy::prelude::*;

use crate::strategic::ChunkStrategicOverlay;

/// Derived tactical contour: contested chunks where two faction slots both hold meaningful control.
#[derive(Resource, Clone, Debug, Default)]
pub struct FrontlineState {
    pub stability: f32,
    pub flux: f32,
    pub contested_chunks: Vec<IVec2>,
}

/// Marks chunks where `faction_control[slot0]` and `faction_control[slot1]` are both significant and close.
pub fn derive_frontline_from_control_system(
    overlays: Query<&ChunkStrategicOverlay>,
    mut front: ResMut<FrontlineState>,
) {
    let mut contested = Vec::new();
    const TIE_LOW: f32 = 0.25;
    const TIE_BAND: f32 = 0.3;

    for ov in overlays.iter() {
        let mut chunk_contested = false;
        let n = ov.len_cells();
        for ci in 0..n {
            let c0 = ov.faction_control[ci][0];
            let c1 = ov.faction_control[ci][1];
            if c0 > TIE_LOW && c1 > TIE_LOW && (c0 - c1).abs() < TIE_BAND {
                chunk_contested = true;
                break;
            }
        }
        if chunk_contested {
            contested.push(ov.chunk_coord);
        }
    }

    contested.sort_by_key(|c| (c.y, c.x));
    contested.dedup();
    let k = contested.len() as f32;
    front.contested_chunks = contested;
    front.flux = (k * 0.04).min(1.0);
    front.stability = (1.0 - k * 0.02).max(0.0);
}
