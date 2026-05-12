//! World preview raster ↔ U7 coupling: coarse epoch + chunk dirty queue (`invalidate_world` entry).

use bevy::prelude::{IVec2, Resource};
use std::collections::HashSet;

/// Monotonic generation stamp for preview consumers (texture epoch, cache busting).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct PreviewGenerationEpoch(pub u64);

/// Central preview invalidation reasons — maps to logging / future fine-grained paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidationReason {
    Registry,
    Rules,
    Tags,
    Tuning,
    Noise,
    /// Ecology / strategic site / corridor footprint touched chunk raster.
    StrategicInfrastructure,
    /// Meso-scale vegetation, fuel, or burn fields changed — ecology preview layer.
    EcologyFields,
}

/// Shared preview invalidation state (terrain + editor; no egui types).
#[derive(Resource, Default)]
pub struct WorldPreviewState {
    pub epoch: PreviewGenerationEpoch,
    pub dirty_queue: Vec<IVec2>,
}

/// Single entry: bump epoch, replace dirty queue with `all_chunk_coords` (deduped).
pub fn invalidate_world(
    _reason: InvalidationReason,
    state: &mut WorldPreviewState,
    all_chunk_coords: impl Iterator<Item = IVec2>,
) {
    state.epoch.0 = state.epoch.0.wrapping_add(1);
    let mut seen = HashSet::new();
    state.dirty_queue.clear();
    for c in all_chunk_coords {
        if seen.insert(c) {
            state.dirty_queue.push(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec2;

    #[test]
    fn invalidate_world_bumps_epoch_and_replaces_queue() {
        let mut state = WorldPreviewState::default();
        assert_eq!(state.epoch.0, 0);
        invalidate_world(
            InvalidationReason::Tuning,
            &mut state,
            [IVec2::new(1, 2), IVec2::new(3, 4)].into_iter(),
        );
        assert_eq!(state.epoch.0, 1);
        assert_eq!(state.dirty_queue, vec![IVec2::new(1, 2), IVec2::new(3, 4)]);
        invalidate_world(
            InvalidationReason::Noise,
            &mut state,
            [IVec2::ZERO].into_iter(),
        );
        assert_eq!(state.epoch.0, 2);
        assert_eq!(state.dirty_queue, vec![IVec2::ZERO]);
    }

    #[test]
    fn invalidate_world_dedupes_chunk_coords() {
        let mut state = WorldPreviewState::default();
        let c = IVec2::new(7, -1);
        invalidate_world(
            InvalidationReason::Registry,
            &mut state,
            [c, c, c, IVec2::ONE].into_iter(),
        );
        assert_eq!(state.dirty_queue.len(), 2);
        assert!(state.dirty_queue.contains(&c));
        assert!(state.dirty_queue.contains(&IVec2::ONE));
    }
}
