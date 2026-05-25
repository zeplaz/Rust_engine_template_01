//! Wave P — composite preview reads canonical terrain state; it does not author simulation.

use super::layers::PreviewLayers;

/// Canonical ECS / asset sources the preview stack may read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompositePreviewCanonicalSource {
    MaterialRegistry,
    TagRegistry,
    TerrainFamilyRegistry,
    MobilityProfileRegistry,
    MaterializedChunk,
    ChunkCellMatrix,
    TileMarker,
    ChunkDerivedMetrics,
    SharedOverlayFieldBuffers,
    EcologyField,
    FireOverlayField,
}

/// One preview layer bit mapped to its authoritative read surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositePreviewLayerBinding {
    pub layer: &'static str,
    pub source: CompositePreviewCanonicalSource,
}

/// Matrix §6 / §15 bindings for the shipped preview layer stack.
pub const WAVE_P_LAYER_BINDINGS: &[CompositePreviewLayerBinding] = &[
    CompositePreviewLayerBinding {
        layer: "height",
        source: CompositePreviewCanonicalSource::TileMarker,
    },
    CompositePreviewLayerBinding {
        layer: "moisture",
        source: CompositePreviewCanonicalSource::TileMarker,
    },
    CompositePreviewLayerBinding {
        layer: "temperature",
        source: CompositePreviewCanonicalSource::TileMarker,
    },
    CompositePreviewLayerBinding {
        layer: "biome",
        source: CompositePreviewCanonicalSource::MaterializedChunk,
    },
    CompositePreviewLayerBinding {
        layer: "regions",
        source: CompositePreviewCanonicalSource::TileMarker,
    },
    CompositePreviewLayerBinding {
        layer: "tag_overlay",
        source: CompositePreviewCanonicalSource::ChunkCellMatrix,
    },
    CompositePreviewLayerBinding {
        layer: "derived_slope",
        source: CompositePreviewCanonicalSource::ChunkDerivedMetrics,
    },
    CompositePreviewLayerBinding {
        layer: "mobility_overlay",
        source: CompositePreviewCanonicalSource::ChunkCellMatrix,
    },
    CompositePreviewLayerBinding {
        layer: "ecology",
        source: CompositePreviewCanonicalSource::EcologyField,
    },
    CompositePreviewLayerBinding {
        layer: "fire_overlay",
        source: CompositePreviewCanonicalSource::SharedOverlayFieldBuffers,
    },
];

/// Preview modules that must stay read-only against gameplay chunk truth.
pub const WAVE_P_CONSUMER_ROOTS: &[&str] = &[
    "src/gui/editor/world_preview",
];

/// Planned Wave P surfaces still gated on backlog choice (inspector host, direct-sample path).
/// `wave_p_live.json` writer ships in `wave_p_live_proof.rs` (post–Stage 6).
pub const WAVE_P_OPEN_BACKLOG_ITEMS: &[&str] = &[];

#[must_use]
pub fn canonical_sources_for_layers(layers: PreviewLayers) -> Vec<CompositePreviewCanonicalSource> {
    let mut out = Vec::new();
    let mut push = |source: CompositePreviewCanonicalSource| {
        if !out.contains(&source) {
            out.push(source);
        }
    };
    if layers.contains(PreviewLayers::HEIGHT)
        || layers.contains(PreviewLayers::MOISTURE)
        || layers.contains(PreviewLayers::TEMPERATURE)
        || layers.contains(PreviewLayers::REGIONS)
    {
        push(CompositePreviewCanonicalSource::TileMarker);
    }
    if layers.contains(PreviewLayers::BIOME) {
        push(CompositePreviewCanonicalSource::MaterializedChunk);
        push(CompositePreviewCanonicalSource::MaterialRegistry);
        push(CompositePreviewCanonicalSource::TerrainFamilyRegistry);
    }
    if layers.contains(PreviewLayers::TAG_OVERLAY) {
        push(CompositePreviewCanonicalSource::ChunkCellMatrix);
        push(CompositePreviewCanonicalSource::TagRegistry);
    }
    if layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
        push(CompositePreviewCanonicalSource::ChunkDerivedMetrics);
    }
    if layers.contains(PreviewLayers::MOBILITY_OVERLAY) {
        push(CompositePreviewCanonicalSource::ChunkCellMatrix);
        push(CompositePreviewCanonicalSource::MobilityProfileRegistry);
        push(CompositePreviewCanonicalSource::TagRegistry);
    }
    if layers.contains(PreviewLayers::ECOLOGY) {
        push(CompositePreviewCanonicalSource::EcologyField);
    }
    push(CompositePreviewCanonicalSource::SharedOverlayFieldBuffers);
    out
}

#[must_use]
pub fn wave_p_consumer_contract_passes(layers: PreviewLayers) -> bool {
    !canonical_sources_for_layers(layers).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
    }

    #[test]
    fn wave_p_layer_bindings_cover_preview_modes() {
        assert!(WAVE_P_LAYER_BINDINGS.len() >= 8);
    }

    #[test]
    fn canonical_sources_for_biome_and_tag_layers() {
        let layers = PreviewLayers::BIOME | PreviewLayers::TAG_OVERLAY;
        let sources = canonical_sources_for_layers(layers);
        assert!(sources.contains(&CompositePreviewCanonicalSource::MaterializedChunk));
        assert!(sources.contains(&CompositePreviewCanonicalSource::ChunkCellMatrix));
        assert!(sources.contains(&CompositePreviewCanonicalSource::TagRegistry));
    }

    #[test]
    fn world_preview_modules_avoid_chunk_sim_mutation_queries() {
        let root = repo_root().join("src/gui/editor/world_preview");
        let forbidden = [
            "Query<(&mut ChunkCellMatrix",
            "Query<(&mut MaterializedChunk",
            "Query<(&mut ChunkDirty",
        ];
        for entry in std::fs::read_dir(root).expect("world_preview dir") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("composite_preview_contract.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read preview module");
            for needle in forbidden {
                assert!(
                    !text.contains(needle),
                    "{} must not mutate gameplay chunk truth ({needle})",
                    path.display()
                );
            }
        }
    }
}
