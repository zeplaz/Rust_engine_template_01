//! **Phase D** — Offscreen [`Camera2d`] → `RenderTarget::Image` for world preview when
//! [`super::PreviewRenderMode::GpuRenderTarget`].
//!
//! Terrain/material chunk quads + shared fire overlay heat tint the render target (not clear-color only).
//! Runs only when [`WorldPreviewGpuRuntime::offscreen_renderer_ready`] is true.

use std::collections::{HashMap, HashSet};

use super::composite_preview_graph::composite_chunk_rgba;
use super::CompositePreviewGraphResource;
use super::ecology_preview::EcologyPreviewSample;
use super::preview_render_contract::{PreviewCameraState, PreviewRenderMode, PreviewRenderTarget};
use super::viewport::EditorViewport;
use crate::gui::WorldRepresentationFrame;
use crate::io::streaming::ghost_band_neighbor_coords_for_preview;
use crate::render::SharedOverlayFieldBuffers;
use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::systems::fire::{ChunkSmokeField, FireFuelField};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;
use crate::terrain::material::{MaterialId, MaterializedChunk, MaterialRegistry};

use bevy::camera::{Camera, Camera2d, ClearColorConfig, RenderTarget};
use bevy::ecs::system::SystemParam;
use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;

/// Marker on the preview-only offscreen camera entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct WorldPreviewGpuCamera;

/// One chunk quad drawn into the preview render target.
#[derive(Component, Debug, Clone)]
pub struct WorldPreviewGpuChunkQuad {
    pub coord: IVec2,
    pub material: Handle<ColorMaterial>,
}

/// When true, [`DefaultPlugins`] render is present — safe to spawn an image-target [`Camera2d`].
#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldPreviewGpuRuntime {
    pub offscreen_renderer_ready: bool,
    pub last_overlay_revision: u64,
}

impl Default for WorldPreviewGpuRuntime {
    fn default() -> Self {
        Self {
            offscreen_renderer_ready: false,
            last_overlay_revision: 0,
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct WorldPreviewGpuTerrainAccess<'w> {
    handles: Res<'w, TerrainRegistriesHandles>,
    materials: Res<'w, Assets<MaterialRegistry>>,
}

#[must_use]
pub(crate) fn visible_chunk_coords_for_preview(
    center: Vec2,
    zoom: f32,
    viewport_size: Vec2,
    chunks: &[(IVec2, UVec2)],
) -> HashSet<IVec2> {
    if zoom <= 0.0 || viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return chunks.iter().map(|(coord, _)| *coord).collect();
    }
    let half_w = viewport_size.x / (2.0 * zoom);
    let half_h = viewport_size.y / (2.0 * zoom);
    let min_x = center.x - half_w;
    let max_x = center.x + half_w;
    let min_y = center.y - half_h;
    let max_y = center.y + half_h;
    let mut out = HashSet::new();
    for (coord, size) in chunks {
        let sx = size.x as f32;
        let sy = size.y as f32;
        let x0 = coord.x as f32 * sx;
        let y0 = coord.y as f32 * sy;
        let x1 = x0 + sx;
        let y1 = y0 + sy;
        if x1 >= min_x && x0 <= max_x && y1 >= min_y && y0 <= max_y {
            out.insert(*coord);
        }
    }
    out
}

fn rgba_to_color(rgba: [u8; 4]) -> Color {
    Color::srgba_u8(rgba[0], rgba[1], rgba[2], rgba[3])
}

pub(crate) fn prefer_gpu_preview_mode_when_renderer_ready(
    mut cam: ResMut<PreviewCameraState>,
    rt: Res<WorldPreviewGpuRuntime>,
) {
    if rt.offscreen_renderer_ready {
        cam.mode = PreviewRenderMode::GpuRenderTarget;
    }
}

pub(crate) fn sync_world_preview_offscreen_camera(
    mut commands: Commands,
    preview_cam: Res<PreviewCameraState>,
    swap: Res<crate::gui::SwapImageBuffers>,
    gpu_rt: Res<WorldPreviewGpuRuntime>,
    q_gpu: Query<Entity, With<WorldPreviewGpuCamera>>,
) {
    if !gpu_rt.offscreen_renderer_ready || preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        for e in q_gpu.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    if swap.back == Handle::default() {
        for e in q_gpu.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    let rt = RenderTarget::from(swap.back.clone());

    if q_gpu.is_empty() {
        let e = commands
            .spawn((
                WorldPreviewGpuCamera,
                Camera2d,
                rt.clone(),
            ))
            .id();
        commands.entity(e).insert(Camera {
            order: 2,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.06, 0.09, 0.14)),
            ..default()
        });
    } else {
        for e in q_gpu.iter() {
            commands.entity(e).insert(rt.clone());
        }
    }
}

pub(crate) fn sync_world_preview_gpu_chunk_quads(
    mut commands: Commands,
    preview_cam: Res<PreviewCameraState>,
    preview_target: Res<PreviewRenderTarget>,
    viewport: Res<EditorViewport>,
    mut gpu_rt: ResMut<WorldPreviewGpuRuntime>,
    shared_overlay: Res<SharedOverlayFieldBuffers>,
    preview_graph: Res<CompositePreviewGraphResource>,
    world_frame: Res<WorldRepresentationFrame>,
    terrain: WorldPreviewGpuTerrainAccess,
    chunk_mats: Query<(&Chunk, &MaterializedChunk)>,
    chunk_ecology: Query<(
        &Chunk,
        Option<&ChunkEcology>,
        Option<&VegetationField>,
        Option<&ChunkWeather>,
        Option<&FireFuelField>,
        Option<&ChunkSmokeField>,
    )>,
    q_chunks: Query<(Entity, &WorldPreviewGpuChunkQuad)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
) {
    if !gpu_rt.offscreen_renderer_ready || preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        for (e, _) in q_chunks.iter() {
            commands.entity(e).despawn();
        }
        return;
    }
    if preview_target.size.x == 0 || preview_target.size.y == 0 {
        return;
    }

    let reg = match terrain.materials.get(&terrain.handles.material_registry) {
        Some(r) => r,
        None => return,
    };
    let graph = preview_graph.0;

    let chunk_geom: Vec<(IVec2, UVec2)> = chunk_mats
        .iter()
        .map(|(c, m)| (c.coord, m.size))
        .collect();
    let viewport_size = if viewport.viewport_size.x > 0.0 && viewport.viewport_size.y > 0.0 {
        viewport.viewport_size
    } else {
        Vec2::new(
            preview_target.size.x as f32,
            preview_target.size.y as f32,
        )
    };
    let mut visible = visible_chunk_coords_for_preview(
        preview_cam.center,
        preview_cam.zoom,
        viewport_size,
        &chunk_geom,
    );
    for coord in ghost_band_neighbor_coords_for_preview(
        world_frame.focus_chunk,
        world_frame.interest_radius_chunks.max(1),
    ) {
        visible.insert(coord);
    }

    let mut live: HashMap<IVec2, Entity> = HashMap::new();
    for (e, quad) in q_chunks.iter() {
        if visible.contains(&quad.coord) {
            live.insert(quad.coord, e);
        } else {
            commands.entity(e).despawn();
        }
    }

    let overlay_revision = shared_overlay.revision;
    let recolor = overlay_revision != gpu_rt.last_overlay_revision;
    gpu_rt.last_overlay_revision = overlay_revision;

    let mut ecology_by_chunk: HashMap<IVec2, EcologyPreviewSample> = HashMap::new();
    for (chunk, eco, veg, wx, fuel, smoke) in chunk_ecology.iter() {
        let heat = shared_overlay.fire_surface_heat_at(chunk.coord);
        ecology_by_chunk.insert(
            chunk.coord,
            EcologyPreviewSample::from_chunk_components(eco, veg, wx, fuel, heat, smoke),
        );
    }

    for (chunk, mat_chunk) in chunk_mats.iter() {
        if !visible.contains(&chunk.coord) {
            continue;
        }
        let heat = shared_overlay.fire_surface_heat_at(chunk.coord);
        let smoke = ecology_by_chunk
            .get(&chunk.coord)
            .map(|sample| sample.smoke.density)
            .unwrap_or(0.0);
        let rgba = composite_chunk_rgba(
            &graph,
            mat_chunk,
            reg,
            ecology_by_chunk.get(&chunk.coord).copied(),
            None,
            heat,
            smoke,
        );
        let color = rgba_to_color(rgba);
        let sx = mat_chunk.size.x as f32;
        let sy = mat_chunk.size.y as f32;
        let x0 = chunk.coord.x as f32 * sx;
        let y0 = chunk.coord.y as f32 * sy;
        let center = Vec3::new(x0 + sx * 0.5, y0 + sy * 0.5, 0.0);

        if let Some(entity) = live.get(&chunk.coord).copied() {
            commands
                .entity(entity)
                .insert(Transform::from_translation(center));
            if recolor {
                if let Ok((_, quad)) = q_chunks.get(entity) {
                    if let Some(mat) = color_materials.get_mut(&quad.material) {
                        mat.color = color;
                    }
                }
            }
            continue;
        }

        let mesh = meshes.add(Rectangle::new(sx, sy));
        let material = color_materials.add(ColorMaterial::from(color));
        commands.spawn((
            WorldPreviewGpuChunkQuad {
                coord: chunk.coord,
                material: material.clone(),
            },
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(center),
        ));
    }
    swap.dirty = true;
}

pub(crate) fn sync_world_preview_offscreen_camera_transform(
    preview_cam: Res<PreviewCameraState>,
    mut q_tf: Query<&mut Transform, With<WorldPreviewGpuCamera>>,
    mut q_proj: Query<&mut Projection, With<WorldPreviewGpuCamera>>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut last_center: Local<Vec2>,
    mut last_zoom: Local<f32>,
) {
    if preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        return;
    }
    if preview_cam.center != *last_center || preview_cam.zoom != *last_zoom {
        swap.dirty = true;
        *last_center = preview_cam.center;
        *last_zoom = preview_cam.zoom;
    }
    let center = Vec3::new(preview_cam.center.x, preview_cam.center.y, 999.0);
    for mut tf in &mut q_tf {
        *tf = Transform::from_translation(center);
    }
    for mut proj in &mut q_proj {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale = 1.0 / preview_cam.zoom.max(1e-4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::composite_preview_graph::CompositePreviewGraph;
    use super::super::layers::PreviewLayers;

    #[test]
    fn preview_chunk_rgba_prefers_ecology_sample_when_present() {
        let reg = MaterialRegistry {
            schema_version: 1,
            materials: Vec::new(),
            name_to_id: HashMap::new(),
        };
        let mat_chunk = MaterializedChunk {
            size: UVec2::ONE,
            materials: vec![MaterialId(0)],
        };
        let sample = EcologyPreviewSample::from_chunk_components(None, None, None, None, 0.4, None);
        let ecology_graph = CompositePreviewGraph::from_layers(PreviewLayers::ECOLOGY);
        let biome_graph = CompositePreviewGraph::from_layers(PreviewLayers::BIOME);
        let with_eco = composite_chunk_rgba(&ecology_graph, &mat_chunk, &reg, Some(sample), None, 0.4, 0.0);
        let material_only = composite_chunk_rgba(&biome_graph, &mat_chunk, &reg, None, None, 0.4, 0.0);
        assert_ne!(with_eco, material_only);
    }

    #[test]
    fn visible_chunks_follow_viewport_intersection() {
        let chunks = vec![(IVec2::new(0, 0), UVec2::new(4, 4)), (IVec2::new(1, 0), UVec2::new(4, 4))];
        let visible = visible_chunk_coords_for_preview(
            Vec2::new(1.0, 1.0),
            1.0,
            Vec2::new(2.0, 2.0),
            &chunks,
        );
        assert!(visible.contains(&IVec2::new(0, 0)));
        assert!(!visible.contains(&IVec2::new(1, 0)));

        let far = visible_chunk_coords_for_preview(
            Vec2::new(100.0, 100.0),
            1.0,
            Vec2::new(4.0, 4.0),
            &chunks,
        );
        assert!(far.is_empty());
    }
}
