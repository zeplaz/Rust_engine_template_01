//! Wave P — shared composite graph for CPU and GPU preview consumers.

use super::color_presets::{height_to_color, moisture_to_color, temperature_to_color};
use super::composite_preview_contract::{
    canonical_sources_for_layers, CompositePreviewCanonicalSource,
};
use super::ecology_preview::{blend_fire_overlay, ecology_preview_rgba, EcologyPreviewSample};
use super::layers::PreviewLayers;
use bevy::prelude::{Res, ResMut, Resource};
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::material::{MaterialId, MaterializedChunk, MaterialRegistry};

/// Active preview layer stack and its canonical read surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositePreviewGraph {
    pub layers: PreviewLayers,
}

impl CompositePreviewGraph {
    #[must_use]
    pub fn from_layers(layers: PreviewLayers) -> Self {
        Self { layers }
    }

    #[must_use]
    pub fn canonical_sources(&self) -> Vec<CompositePreviewCanonicalSource> {
        canonical_sources_for_layers(self.layers)
    }

    #[must_use]
    pub fn base_layers(&self) -> PreviewLayers {
        self.layers.base_bits()
    }
}

/// Cached composite graph for preview consumers (synced from [`WorldGenUiState`]).
#[derive(Resource, Clone, Copy, Debug)]
pub struct CompositePreviewGraphResource(pub CompositePreviewGraph);

impl Default for CompositePreviewGraphResource {
    fn default() -> Self {
        Self(CompositePreviewGraph::from_layers(PreviewLayers::default()))
    }
}

pub fn sync_composite_preview_graph_resource(
    map_views: Res<crate::gui::MapViewInstances>,
    mut graph: ResMut<CompositePreviewGraphResource>,
) {
    graph.0 = CompositePreviewGraph::from_layers(map_views.world_preview.layers);
}

#[must_use]
pub fn materialized_chunk_base_rgba(
    chunk: &MaterializedChunk,
    reg: &MaterialRegistry,
) -> [u8; 4] {
    let id = chunk
        .materials
        .first()
        .copied()
        .unwrap_or(MaterialId(0));
    reg.materials
        .get(id.0 as usize)
        .map(|material| material.preview_color)
        .unwrap_or([64, 64, 64, 255])
}

#[must_use]
fn chunk_scalar_mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

#[must_use]
pub fn chunk_base_rgba_for_graph(
    graph: &CompositePreviewGraph,
    mat_chunk: &MaterializedChunk,
    reg: &MaterialRegistry,
    ecology: Option<EcologyPreviewSample>,
    cell_matrix: Option<&ChunkCellMatrix>,
) -> [u8; 4] {
    let base = graph.base_layers();
    if base.is_empty() {
        return materialized_chunk_base_rgba(mat_chunk, reg);
    }
    if base.contains(PreviewLayers::ECOLOGY) {
        if let Some(sample) = ecology {
            return ecology_preview_rgba(&sample);
        }
    }
    if base.contains(PreviewLayers::BIOME) {
        return materialized_chunk_base_rgba(mat_chunk, reg);
    }
    if let Some(matrix) = cell_matrix {
        if base.contains(PreviewLayers::HEIGHT) {
            return height_to_color(chunk_scalar_mean(&matrix.elevation));
        }
        if base.contains(PreviewLayers::MOISTURE) {
            return moisture_to_color(chunk_scalar_mean(&matrix.moisture));
        }
        if base.contains(PreviewLayers::TEMPERATURE) {
            return temperature_to_color(chunk_scalar_mean(&matrix.temperature));
        }
    }
    materialized_chunk_base_rgba(mat_chunk, reg)
}

#[must_use]
pub fn composite_chunk_rgba(
    graph: &CompositePreviewGraph,
    mat_chunk: &MaterializedChunk,
    reg: &MaterialRegistry,
    ecology: Option<EcologyPreviewSample>,
    cell_matrix: Option<&ChunkCellMatrix>,
    heat: f32,
    smoke: f32,
) -> [u8; 4] {
    let base = chunk_base_rgba_for_graph(graph, mat_chunk, reg, ecology, cell_matrix);
    blend_fire_overlay(base, heat, smoke)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::{ChunkEcology, VegetationField};
    use crate::systems::fire::{ChunkSmokeField, FireFuelField};
    use crate::systems::weather::ChunkWeather;
    use bevy::math::UVec2;

    #[test]
    fn ecology_base_overrides_material_color() {
        let reg = MaterialRegistry {
            schema_version: 1,
            materials: Vec::new(),
            name_to_id: Default::default(),
        };
        let mat_chunk = MaterializedChunk {
            size: UVec2::ONE,
            materials: vec![MaterialId(0)],
        };
        let graph = CompositePreviewGraph::from_layers(PreviewLayers::ECOLOGY);
        let sample = EcologyPreviewSample::from_chunk_components(
            Some(&ChunkEcology::default()),
            Some(&VegetationField::default()),
            Some(&ChunkWeather::default()),
            Some(&FireFuelField::default()),
            0.2,
            Some(&ChunkSmokeField::default()),
        );
        let ecology = composite_chunk_rgba(&graph, &mat_chunk, &reg, Some(sample), None, 0.0, 0.0);
        let biome = composite_chunk_rgba(
            &CompositePreviewGraph::from_layers(PreviewLayers::BIOME),
            &mat_chunk,
            &reg,
            Some(sample),
            None,
            0.0,
            0.0,
        );
        assert_ne!(ecology, biome);
    }

    #[test]
    fn height_base_reads_chunk_cell_matrix_means() {
        let reg = MaterialRegistry {
            schema_version: 1,
            materials: Vec::new(),
            name_to_id: Default::default(),
        };
        let mat_chunk = MaterializedChunk {
            size: UVec2::new(2, 1),
            materials: vec![MaterialId(0), MaterialId(0)],
        };
        let matrix = ChunkCellMatrix::new(UVec2::new(2, 1));
        let mut matrix = matrix;
        matrix.elevation = vec![0.0, 1.0];
        let graph = CompositePreviewGraph::from_layers(PreviewLayers::HEIGHT);
        let rgba = chunk_base_rgba_for_graph(&graph, &mat_chunk, &reg, None, Some(&matrix));
        assert_eq!(rgba, height_to_color(0.5));
    }
}
