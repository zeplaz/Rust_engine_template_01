//! Stage-6 virtualization contracts — residency-driven representation host.

use bevy::prelude::*;

use crate::gui::WorldRepresentationFrame;
use crate::io::streaming::{gather_wave_c_readiness, wave_c_readiness_passes, ChunkResidencyTable};
use crate::gui::editor::world_preview::PreviewPathAuthority;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuntimeAtlasSlot {
    TerrainChunk,
    OverlayField,
    UtilityMask,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct PagedAtlasResidency {
    pub active_slots: Vec<RuntimeAtlasSlot>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct AsyncDomainApplyQueue {
    pub pending_labels: Vec<&'static str>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct Stage6VirtualizationFrame {
    pub focus_chunk: IVec2,
    pub residency_chunk_count: usize,
    pub core_chunk_count: usize,
    pub ghost_chunk_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stage6ReadinessReport {
    pub wave_c_ok: bool,
    pub residency_populated: bool,
}

#[must_use]
pub fn gather_stage6_readiness(
    authority: &PreviewPathAuthority,
    frame: &Stage6VirtualizationFrame,
) -> Stage6ReadinessReport {
    let wave_c = gather_wave_c_readiness(authority);
    Stage6ReadinessReport {
        wave_c_ok: wave_c_readiness_passes(&wave_c),
        residency_populated: frame.residency_chunk_count > 0,
    }
}

#[must_use]
pub fn stage6_readiness_passes(report: &Stage6ReadinessReport) -> bool {
    report.wave_c_ok && report.residency_populated
}

pub fn publish_stage6_virtualization_frame(
    world: Res<WorldRepresentationFrame>,
    residency: Res<ChunkResidencyTable>,
    mut frame: ResMut<Stage6VirtualizationFrame>,
) {
    use crate::io::streaming::ChunkResidencyRole;
    frame.focus_chunk = world.focus_chunk;
    frame.residency_chunk_count = residency.entries.len();
    frame.core_chunk_count = residency
        .entries
        .values()
        .filter(|entry| entry.role == ChunkResidencyRole::Core)
        .count();
    frame.ghost_chunk_count = residency
        .entries
        .values()
        .filter(|entry| entry.role == ChunkResidencyRole::GhostBand)
        .count();
}

pub struct Stage6VirtualizationPlugin;

impl Plugin for Stage6VirtualizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PagedAtlasResidency>()
            .init_resource::<AsyncDomainApplyQueue>()
            .init_resource::<Stage6VirtualizationFrame>()
            .add_systems(
                Update,
                publish_stage6_virtualization_frame
                    .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage6_readiness_requires_wave_c_and_residency() {
        let report = gather_stage6_readiness(
            &PreviewPathAuthority::default(),
            &Stage6VirtualizationFrame {
                residency_chunk_count: 1,
                ..Default::default()
            },
        );
        assert!(stage6_readiness_passes(&report));
    }
}
