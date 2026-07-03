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
use super::render_target_barrier::{
    committed_render_target_handle, try_commit_world_preview_render_target,
    WorldPreviewRenderTargetBindBarrier, WorldPreviewRenderTargetRegistry,
    WorldPreviewViewportEvent,
};
use super::preview_readiness::world_generation_complete;
use crate::engine::WorldGenFlowState;
use crate::render::{
    trace_camera_sync, DebugRenderTraceConfig, ResolvedViewports, Stage6VirtualizationFrame,
};
use crate::gui::WorldRepresentationFrame;
use crate::io::streaming::ghost_band_neighbor_coords_for_preview;
use crate::render::SharedOverlayFieldBuffers;
use crate::systems::ecology::{ChunkEcology, LandscapeProgramOnChunk, VegetationField};
use crate::systems::fire::{ChunkSmokeField, FireFuelField};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::world_generator_enhanced::WorldGenJobSlot;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{MaterializedChunk, MaterialRegistry};

use bevy::camera::{Camera, Camera2d, ClearColorConfig, RenderTarget};
use bevy::diagnostic::FrameCount;
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
    /// VM-C5: both swap handles allocated (no ad-hoc single-image GPU preview).
    pub pooled_swap_ready: bool,
}

impl Default for WorldPreviewGpuRuntime {
    fn default() -> Self {
        Self {
            offscreen_renderer_ready: false,
            last_overlay_revision: 0,
            pooled_swap_ready: false,
        }
    }
}

/// VM-C5: GPU preview must use [`crate::gui::SwapImageBuffers`] front/back, not orphan handles.
pub(crate) fn enforce_gpu_preview_pooled_swap(
    preview_cam: Res<PreviewCameraState>,
    mut gpu_rt: ResMut<WorldPreviewGpuRuntime>,
    swap: Res<crate::gui::SwapImageBuffers>,
    registry: Res<WorldPreviewRenderTargetRegistry>,
) {
    if !gpu_rt.offscreen_renderer_ready || preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        gpu_rt.pooled_swap_ready = false;
        return;
    }
    gpu_rt.pooled_swap_ready =
        swap.front != Handle::default() && swap.back != Handle::default();
    let _ = registry;
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

pub(crate) fn seed_world_preview_render_target_registry(
    swap: Res<crate::gui::SwapImageBuffers>,
    images: Res<Assets<Image>>,
    resolved: Res<ResolvedViewports>,
    preview_cam: Res<PreviewCameraState>,
    gpu_rt: Res<WorldPreviewGpuRuntime>,
    mut registry: ResMut<WorldPreviewRenderTargetRegistry>,
    mut barrier: ResMut<WorldPreviewRenderTargetBindBarrier>,
) {
    if !gpu_rt.pooled_swap_ready || registry.revision > 0 || swap.back == Handle::default() {
        return;
    }
    if preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        return;
    }
    let Some(image) = images.get(&swap.back) else {
        return;
    };
    let size = image.texture_descriptor.size;
    let swap_extent = UVec2::new(size.width, size.height);
    if resolved.world_preview.valid
        && resolved.world_preview.physical_extent != UVec2::ZERO
        && swap_extent != resolved.world_preview.physical_extent
    {
        return;
    }
    registry.committed_image = swap.back.clone();
    registry.committed_size = swap_extent;
    registry.revision = 1;
    barrier.bound = swap.back.clone();
}

pub(crate) fn commit_world_preview_render_target(
    frame: Res<FrameCount>,
    images: Res<Assets<Image>>,
    mut barrier: ResMut<WorldPreviewRenderTargetBindBarrier>,
    mut registry: ResMut<WorldPreviewRenderTargetRegistry>,
    mut events: MessageWriter<WorldPreviewViewportEvent>,
) {
    if let Some(event) =
        try_commit_world_preview_render_target(&mut barrier, &mut registry, &frame, &images)
    {
        events.write(event);
    }
}

pub(crate) fn sync_world_preview_offscreen_camera(
    mut commands: Commands,
    flow: Res<State<WorldGenFlowState>>,
    job_slot: Res<WorldGenJobSlot>,
    preview_cam: Res<PreviewCameraState>,
    gpu_rt: Res<WorldPreviewGpuRuntime>,
    images: Res<Assets<Image>>,
    palette: Res<crate::gui::UiPalette>,
    contract: Res<super::render_target_barrier::WorldPreviewRenderViewportContract>,
    mut bind_barrier: ResMut<WorldPreviewRenderTargetBindBarrier>,
    mut registry: ResMut<WorldPreviewRenderTargetRegistry>,
    mut q_gpu: Query<(Entity, &mut Camera), With<WorldPreviewGpuCamera>>,
) {
    if !gpu_rt.offscreen_renderer_ready || preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        bind_barrier.clear();
        *registry = WorldPreviewRenderTargetRegistry::default();
        for (e, _) in q_gpu.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    if !world_generation_complete(*flow.get(), job_slot.is_busy()) {
        bind_barrier.clear();
        for (e, _) in q_gpu.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    if bind_barrier.pending.is_some() || !contract.camera_ready {
        for (e, _) in q_gpu.iter() {
            commands.entity(e).despawn();
        }
        return;
    }

    let Some(target) = committed_render_target_handle(&registry, &images) else {
        for (e, _) in q_gpu.iter() {
            commands.entity(e).despawn();
        }
        return;
    };
    let rt = RenderTarget::from(target);
    let extent = registry.committed_size.max(UVec2::ONE);
    let viewport = Some(bevy::camera::Viewport {
        physical_position: UVec2::ZERO,
        physical_size: extent,
        depth: 0.0..1.0,
    });

    if q_gpu.is_empty() {
        let mut camera = Camera {
            order: 2,
            clear_color: ClearColorConfig::Custom(palette.bevy_sim_map_field_clear()),
            ..default()
        };
        camera.viewport = viewport;
        let e = commands
            .spawn((
                WorldPreviewGpuCamera,
                Camera2d,
                rt.clone(),
                camera,
            ))
            .id();
        let _ = e;
    } else {
        for (e, mut camera) in q_gpu.iter_mut() {
            commands.entity(e).insert(rt.clone());
            camera.viewport = viewport.clone();
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct WorldPreviewGpuChunkSync<'w, 's> {
    preview_ready: Res<'w, super::preview_readiness::WorldPreviewReady>,
    preview_cam: Res<'w, PreviewCameraState>,
    preview_target: Res<'w, PreviewRenderTarget>,
    resolved: Res<'w, ResolvedViewports>,
    gpu_rt: ResMut<'w, WorldPreviewGpuRuntime>,
    shared_overlay: Res<'w, SharedOverlayFieldBuffers>,
    preview_graph: Res<'w, CompositePreviewGraphResource>,
    world_frame: Res<'w, WorldRepresentationFrame>,
    terrain: WorldPreviewGpuTerrainAccess<'w>,
    chunk_mats: Query<'w, 's, (&'static Chunk, &'static MaterializedChunk)>,
    chunk_cells: Query<'w, 's, (&'static Chunk, &'static ChunkCellMatrix)>,
    chunk_ecology: Query<
        'w,
        's,
        (
            &'static Chunk,
            Option<&'static ChunkEcology>,
            Option<&'static VegetationField>,
            Option<&'static ChunkWeather>,
            Option<&'static FireFuelField>,
            Option<&'static ChunkSmokeField>,
            Option<&'static LandscapeProgramOnChunk>,
        ),
    >,
    q_chunks: Query<'w, 's, (Entity, &'static WorldPreviewGpuChunkQuad)>,
    meshes: ResMut<'w, Assets<Mesh>>,
    color_materials: ResMut<'w, Assets<ColorMaterial>>,
    swap: ResMut<'w, crate::gui::SwapImageBuffers>,
}

pub(crate) fn sync_world_preview_gpu_chunk_quads(
    mut commands: Commands,
    mut sync: WorldPreviewGpuChunkSync,
    stage6: Option<Res<Stage6VirtualizationFrame>>,
    mut last_visible: Local<usize>,
) {
    if !sync.preview_ready.0
        || !sync.gpu_rt.offscreen_renderer_ready
        || sync.preview_cam.mode != PreviewRenderMode::GpuRenderTarget
    {
        for (e, _) in sync.q_chunks.iter() {
            commands.entity(e).despawn();
        }
        return;
    }
    if sync.preview_target.size.x == 0 || sync.preview_target.size.y == 0 {
        return;
    }

    let reg = match sync.terrain.materials.get(&sync.terrain.handles.material_registry) {
        Some(r) => r,
        None => return,
    };
    let graph = sync.preview_graph.0;

    let chunk_geom: Vec<(IVec2, UVec2)> = sync
        .chunk_mats
        .iter()
        .map(|(c, m)| (c.coord, m.size))
        .collect();
    let viewport_size = if sync.resolved.world_preview.valid {
        sync.resolved.world_preview.logical_size
    } else {
        Vec2::new(
            sync.preview_target.size.x as f32,
            sync.preview_target.size.y as f32,
        )
    };
    let mut visible = visible_chunk_coords_for_preview(
        sync.preview_cam.center,
        sync.preview_cam.zoom,
        viewport_size,
        &chunk_geom,
    );
    for coord in ghost_band_neighbor_coords_for_preview(
        sync.world_frame.focus_chunk,
        sync.world_frame.interest_radius_chunks.max(1),
    ) {
        visible.insert(coord);
    }
    if let Some(stage6) = stage6.as_deref() {
        visible = crate::render::intersect_visible_chunks_with_residency_window(visible, stage6);
    }

    let mut live: HashMap<IVec2, Entity> = HashMap::new();
    for (e, quad) in sync.q_chunks.iter() {
        if visible.contains(&quad.coord) {
            live.insert(quad.coord, e);
        } else {
            commands.entity(e).despawn();
        }
    }

    let overlay_revision = sync.shared_overlay.revision;
    let recolor = overlay_revision != sync.gpu_rt.last_overlay_revision;
    sync.gpu_rt.last_overlay_revision = overlay_revision;

    let mut ecology_by_chunk: HashMap<IVec2, EcologyPreviewSample> = HashMap::new();
    for (chunk, eco, veg, wx, fuel, smoke, program) in sync.chunk_ecology.iter() {
        let heat = sync.shared_overlay.fire_surface_heat_at(chunk.coord);
        let mut sample = EcologyPreviewSample::from_chunk_components(eco, veg, wx, fuel, heat, smoke);
        if let Some(prog) = program {
            sample = sample.with_topology_kinds(&prog.evaluation.topology_kinds);
        }
        ecology_by_chunk.insert(chunk.coord, sample);
    }

    for (chunk, mat_chunk) in sync.chunk_mats.iter() {
        if !visible.contains(&chunk.coord) {
            continue;
        }
        let heat = sync.shared_overlay.fire_surface_heat_at(chunk.coord);
        let smoke = ecology_by_chunk
            .get(&chunk.coord)
            .map(|sample| sample.smoke.density)
            .unwrap_or(0.0);
        let cell_matrix = sync
            .chunk_cells
            .iter()
            .find(|(c, _)| c.coord == chunk.coord)
            .map(|(_, matrix)| matrix);
        let rgba = composite_chunk_rgba(
            &graph,
            mat_chunk,
            reg,
            ecology_by_chunk.get(&chunk.coord).copied(),
            cell_matrix,
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
                if let Ok((_, quad)) = sync.q_chunks.get(entity) {
                    if let Some(mut mat) = sync.color_materials.get_mut(&quad.material) {
                        mat.color = color;
                    }
                }
            }
            continue;
        }

        let mesh = sync.meshes.add(Rectangle::new(sx, sy));
        let material = sync.color_materials.add(ColorMaterial::from(color));
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
    let visible_count = visible.len();
    if recolor || visible_count != *last_visible {
        sync.swap.dirty = true;
        *last_visible = visible_count;
    }
}

pub(crate) fn sync_world_preview_offscreen_camera_transform(
    cfg: Res<DebugRenderTraceConfig>,
    preview_cam: Res<PreviewCameraState>,
    snapshot: Res<crate::gui::ViewRepresentationSnapshot>,
    mut q_tf: Query<&mut Transform, With<WorldPreviewGpuCamera>>,
    mut q_proj: Query<&mut Projection, With<WorldPreviewGpuCamera>>,
) {
    if preview_cam.mode != PreviewRenderMode::GpuRenderTarget {
        return;
    }
    let center = Vec3::new(
        snapshot.camera.translation.x,
        snapshot.camera.translation.y,
        999.0,
    );
    for mut tf in &mut q_tf {
        *tf = Transform::from_translation(center);
    }
    let zoom = snapshot.camera.zoom.max(1e-4);
    let view_w = snapshot.viewport.width().max(1.0);
    let view_h = snapshot.viewport.height().max(1.0);
    for mut proj in &mut q_proj {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            ortho.scale = 1.0 / zoom;
            ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
                width: view_w / zoom,
                height: view_h / zoom,
            };
        }
    }
    if cfg.camera_sync_trace {
        trace_camera_sync(
            &cfg,
            &format!(
                "preview_gpu_camera center=({:.1},{:.1}) zoom={:.3} ortho=({:.1},{:.1}) snapshot=({:.1},{:.1})",
                snapshot.camera.translation.x,
                snapshot.camera.translation.y,
                snapshot.camera.zoom,
                view_w / zoom,
                view_h / zoom,
                snapshot.viewport.width(),
                snapshot.viewport.height(),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::composite_preview_graph::CompositePreviewGraph;
    use crate::terrain::material::MaterialId;
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

    #[test]
    fn gpu_scalar_preview_uses_chunk_cell_matrix_means() {
        use super::super::composite_preview_graph::CompositePreviewGraph;
        use super::super::layers::PreviewLayers;
        use crate::terrain::generation::cell_matrix::ChunkCellMatrix;

        let reg = MaterialRegistry {
            schema_version: 1,
            materials: Vec::new(),
            name_to_id: HashMap::new(),
        };
        let mat_chunk = MaterializedChunk {
            size: UVec2::new(2, 1),
            materials: vec![MaterialId(0), MaterialId(0)],
        };
        let mut matrix = ChunkCellMatrix::new(UVec2::new(2, 1));
        matrix.moisture = vec![0.0, 1.0];
        let graph = CompositePreviewGraph::from_layers(PreviewLayers::MOISTURE);
        let with_matrix = composite_chunk_rgba(&graph, &mat_chunk, &reg, None, Some(&matrix), 0.0, 0.0);
        let without_matrix =
            composite_chunk_rgba(&graph, &mat_chunk, &reg, None, None, 0.0, 0.0);
        assert_ne!(with_matrix, without_matrix);
    }
}
