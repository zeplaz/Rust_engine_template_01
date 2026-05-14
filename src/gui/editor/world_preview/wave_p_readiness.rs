//! Wave P readiness — entry gates after Wave S save spine.

use super::composite_preview_graph::CompositePreviewGraph;
use super::composite_preview_contract::{
    wave_p_consumer_contract_passes, WAVE_P_LAYER_BINDINGS, WAVE_P_OPEN_BACKLOG_ITEMS,
};
use super::layers::PreviewLayers;
use super::preview_render_contract::{PreviewAuthoritativeSurface, PreviewPathAuthority};
use crate::io::save::{SAVE_WORLD_MANIFEST_SCHEMA_VERSION, SAVED_CHUNK_BODY_SCHEMA_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WavePReadinessReport {
    pub save_manifest_schema_version: u32,
    pub save_chunk_schema_version: u32,
    pub composite_layer_bindings: u32,
    pub consumer_contract_ok: bool,
    pub composite_graph_sources: u32,
    pub gpu_authoritative_surface: bool,
    pub open_backlog_items: u32,
}

#[must_use]
pub fn gather_wave_p_readiness(
    layers: PreviewLayers,
    authority: &PreviewPathAuthority,
) -> WavePReadinessReport {
    let graph = CompositePreviewGraph::from_layers(layers);
    WavePReadinessReport {
        save_manifest_schema_version: SAVE_WORLD_MANIFEST_SCHEMA_VERSION,
        save_chunk_schema_version: SAVED_CHUNK_BODY_SCHEMA_VERSION,
        composite_layer_bindings: WAVE_P_LAYER_BINDINGS.len() as u32,
        consumer_contract_ok: wave_p_consumer_contract_passes(layers),
        composite_graph_sources: graph.canonical_sources().len() as u32,
        gpu_authoritative_surface: authority.authoritative_surface
            == PreviewAuthoritativeSurface::GpuRenderTarget,
        open_backlog_items: WAVE_P_OPEN_BACKLOG_ITEMS.len() as u32,
    }
}

#[must_use]
pub fn wave_p_readiness_passes(report: &WavePReadinessReport) -> bool {
    report.save_manifest_schema_version > 0
        && report.save_chunk_schema_version > 0
        && report.composite_layer_bindings > 0
        && report.consumer_contract_ok
        && report.composite_graph_sources > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_p_readiness_passes_with_default_layers() {
        let authority = PreviewPathAuthority::default();
        let report = gather_wave_p_readiness(PreviewLayers::BIOME, &authority);
        assert!(wave_p_readiness_passes(&report));
    }
}
