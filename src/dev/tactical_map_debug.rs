//! Always-on tactical map / minimap diagnostic for CLI `--test` runs.
//!
//! Writes `debug_runs/tactical_map_debug_live.json` at sim frames 30, 60, 90, 120, 180
//! and every 60 frames thereafter. Run `cargo run --release -- --test vfx`, reproduce,
//! then open the JSON — no `--deep-debug` required.
//!
//! **`rtt_render_trace`** block explains void tactical map: RTT bind barrier, camera
//! render target vs UI ImageNode, terrain sprite visibility/layers, ortho + view_proj.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ImageRenderTarget, RenderTarget};
use bevy::ecs::system::SystemParam;
use bevy::math::{Mat4, Vec2};
use bevy::prelude::*;
use serde_json::{json, Value};

pub const TACTICAL_MAP_DEBUG_JSON: &str = "debug_runs/tactical_map_debug_live.json";

const RTT_LAYER: usize = crate::gui::SIMULATION_MAP_RTT_RENDER_LAYER;
#[derive(Resource, Default)]
struct TacticalMapDebugState {
    last_written_frame: u32,
}

pub struct TacticalMapDebugPlugin;

impl Plugin for TacticalMapDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TacticalMapDebugState>().add_systems(
            PostUpdate,
            write_tactical_map_debug_witness
                .run_if(in_state(crate::engine::states::BaseState::Simulation)),
        );
    }
}

#[derive(SystemParam)]
struct TacticalMapDebugInputs<'w, 's> {
    launch: Option<Res<'w, crate::engine::EngineLaunchArgs>>,
    frame: Res<'w, bevy::diagnostic::FrameCount>,
    sim_time: Option<Res<'w, crate::systems::sim_control::SimTimeMicros>>,
    fill: Res<'w, crate::gui::TacticalMapFillRect>,
    sim_tex: Res<'w, crate::gui::SimulationMapTexture>,
    authority: Res<'w, crate::render::TerrainRenderAuthority>,
    raster_dirty: Res<'w, crate::render::TileWorldFallbackRasterDirty>,
    raster_ctrl: Res<'w, crate::render::TileWorldFallbackRasterCtrl>,
    raster_policy: Res<'w, crate::render::TileFallbackRasterPolicy>,
    fallback: Res<'w, crate::render::TileWorldFallbackState>,
    overlay: Res<'w, crate::render::SharedOverlayFieldBuffers>,
    compositor: Res<'w, crate::render::minimap_compositor::MinimapCompositorState>,
    atlas: Option<Res<'w, crate::render::TerrainMaterialAtlasGpu>>,
    vt4: Option<Res<'w, crate::render::VtCiMatrixLiveReport>>,
    tile_debug: Res<'w, crate::gui::TileGpuDebugSettings>,
    fire_override: Res<'w, crate::gui::FireDebugOverride>,
    construction: Option<Res<'w, crate::construction::ConstructionVisualRequests>>,
    params: Res<'w, crate::terrain::generation::WorldGenParams>,
    dense: Option<Res<'w, crate::terrain::generation::WorldGenDenseTerrainCache>>,
    images: Res<'w, Assets<Image>>,
    rtt_barrier: Option<Res<'w, crate::gui::sim_map_rtt::SimulationMapRttBindBarrier>>,
    ortho: Option<Res<'w, crate::gui::MainWorldCameraOrthoTrace>>,
    cam_metrics: Option<Res<'w, crate::render::ExtractedCameraMetrics>>,
    fire_globals: Option<Res<'w, crate::render::FireParticleDrawGlobals>>,
    fire_frame: Option<Res<'w, crate::render::WorldFireParticleFrame>>,
    map_desired: Option<Res<'w, crate::gui::MapCameraDesiredRes>>,
    flow: Option<Res<'w, bevy::prelude::State<crate::engine::states::WorldGenFlowState>>>,
    wg_progress: Option<Res<'w, crate::terrain::generation::WorldGenProgress>>,
    base_state: Option<Res<'w, bevy::prelude::State<crate::engine::states::BaseState>>>,
    rtt_diag: Option<Res<'w, crate::gui::RttDiagCameraConfig>>,
    image_node: Query<'w, 's, &'static bevy::ui::widget::ImageNode, With<crate::gui::SimulationMapViewportFill>>,
    terrain_display: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static Sprite>,
            Option<&'static Mesh2d>,
            &'static Transform,
            &'static Visibility,
            Option<&'static RenderLayers>,
        ),
        With<crate::render::TileWorldFallbackSprite>,
    >,
    tile_markers: Query<'w, 's, (), With<crate::terrain::generation::TileMarker>>,
    chunks: Query<'w, 's, (), With<crate::terrain::generation::ChunkCellMatrix>>,
    test_chunks: Query<'w, 's, (), With<crate::engine::test_harness::TestSceneSimChunk>>,
}

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn rgba_stats(data: &[u8], w: u32, h: u32) -> Value {
    let expected = (w as usize).saturating_mul(h as usize).saturating_mul(4);
    if data.len() < expected || w == 0 || h == 0 {
        return json!({
            "valid": false,
            "byte_len": data.len(),
            "expected_bytes": expected,
        });
    }
    let mut nonzero = 0u64;
    let mut sum: u64 = 0;
    for px in data.chunks_exact(4) {
        let lum = px[0] as u32 + px[1] as u32 + px[2] as u32;
        if lum > 24 {
            nonzero += 1;
        }
        sum += lum as u64;
    }
    let sample = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        if i + 3 < data.len() {
            [data[i], data[i + 1], data[i + 2], data[i + 3]]
        } else {
            [0, 0, 0, 0]
        }
    };
    let cx = w / 2;
    let cy = h / 2;
    json!({
        "valid": true,
        "nonzero_pixels": nonzero,
        "total_pixels": w as u64 * h as u64,
        "coverage_ratio": nonzero as f64 / (w as f64 * h as f64).max(1.0),
        "luma_sum": sum,
        "corner_tl": sample(0, 0),
        "corner_br": sample(w.saturating_sub(1), h.saturating_sub(1)),
        "center": sample(cx, cy),
    })
}

fn image_stats(images: &Assets<Image>, handle: &Handle<Image>) -> Value {
    let Some(img) = images.get(handle) else {
        return json!({ "loaded": false, "handle": handle_label(handle) });
    };
    let mut out = json!({
        "loaded": true,
        "handle": handle_label(handle),
        "width": img.width(),
        "height": img.height(),
        "has_cpu_data": img.data.is_some(),
        "asset_usage": format!("{:?}", img.asset_usage),
        "render_world": img.asset_usage.contains(RenderAssetUsages::RENDER_WORLD),
        "texture_format": format!("{:?}", img.texture_descriptor.format),
        "texture_usages": format!("{:?}", img.texture_descriptor.usage),
    });
    if let Some(data) = img.data.as_ref() {
        out["pixels"] = rgba_stats(data, img.width(), img.height());
    }
    out
}

fn handle_label(handle: &Handle<Image>) -> String {
    format!("{handle:?}")
}

fn render_target_trace(
    target: &RenderTarget,
    sim_tex: &Handle<Image>,
    bound: &Handle<Image>,
    flag_void_on_window: bool,
) -> Value {
    match target {
        RenderTarget::Image(ImageRenderTarget { handle, scale_factor }) => json!({
            "kind": "image",
            "handle": handle_label(handle),
            "scale_factor": scale_factor,
            "matches_sim_tex": *handle == *sim_tex,
            "matches_barrier_bound": *handle == *bound,
        }),
        RenderTarget::Window(_) if flag_void_on_window => {
            json!({ "kind": "window", "void_suspect": "CAMERA_RTT_TARGET_IS_WINDOW" })
        }
        RenderTarget::Window(_) => json!({ "kind": "window" }),
        RenderTarget::TextureView(_) => json!({ "kind": "texture_view" }),
        RenderTarget::None { .. } => json!({ "kind": "none", "void_suspect": "CAMERA_RTT_TARGET_NONE" }),
        _ => json!({ "kind": "other", "debug": format!("{target:?}") }),
    }
}

fn layers_intersect_camera(sprite_layers: Option<&RenderLayers>, cam_layers: Option<&RenderLayers>) -> bool {
    match (sprite_layers, cam_layers) {
        (None, None) => true,
        (None, Some(_)) | (Some(_), None) => true,
        (Some(a), Some(b)) => a == b,
    }
}

fn view_proj_degenerate(m: Mat4) -> bool {
    !m.is_finite() || m == Mat4::IDENTITY || m.determinant().abs() < 1e-12
}

fn build_rtt_render_trace(
    barrier: Option<&crate::gui::sim_map_rtt::SimulationMapRttBindBarrier>,
    sim_tex: &Handle<Image>,
    image_node: Option<&bevy::ui::widget::ImageNode>,
    cam: Option<(
        &Camera,
        &Projection,
        &GlobalTransform,
        &Visibility,
        &RenderTarget,
        Option<&RenderLayers>,
    )>,
    hud_cam: Option<(&Camera, &RenderTarget)>,
    terrain: Option<(
        Entity,
        Option<&Sprite>,
        Option<&Mesh2d>,
        &Transform,
        &Visibility,
        Option<&RenderLayers>,
    )>,
    fallback: &crate::render::TileWorldFallbackState,
    ortho: Option<&crate::gui::MainWorldCameraOrthoTrace>,
    metrics: Option<&crate::render::ExtractedCameraMetrics>,
    fire_globals: Option<&crate::render::FireParticleDrawGlobals>,
    fire_frame: Option<&crate::render::WorldFireParticleFrame>,
    map_desired: Option<&crate::gui::MapCameraDesired>,
    fill_valid: bool,
    images: &Assets<Image>,
) -> Value {
    let bound = barrier.map(|b| b.bound.clone()).unwrap_or_else(|| sim_tex.clone());
    let pending = barrier.and_then(|b| b.pending.as_ref()).map(|p| {
        let size = images.get(&p.target).map(|img| (img.width(), img.height()));
        json!({
            "frame_requested": p.frame_requested,
            "handle": handle_label(&p.target),
            "scale_factor": p.scale_factor,
            "allocated_size": size.map(|(w, h)| json!([w, h])),
        })
    });

    let cam_json = cam
        .map(|(camera, proj, gt, vis, rt, cam_layers)| {
            json!({
                "is_active": camera.is_active,
                "order": camera.order,
                "viewport": camera.viewport.as_ref().map(|v| json!([v.physical_position.x, v.physical_position.y, v.physical_size.x, v.physical_size.y])),
                "visibility": format!("{vis:?}"),
                "render_layers": cam_layers.map(|l| format!("{l:?}")),
                "world_position": [gt.translation().x, gt.translation().y, gt.translation().z],
                "projection": format!("{proj:?}"),
                "render_target": render_target_trace(rt, sim_tex, &bound, true),
            })
        })
        .unwrap_or(json!({ "missing": true, "void_suspect": "NO_MAIN_WORLD_CAMERA" }));

    let hud_json = hud_cam.map(|(c, rt)| {
        json!({
            "is_active": c.is_active,
            "order": c.order,
            "render_target": render_target_trace(rt, sim_tex, &bound, false),
        })
    });

    let terrain_json = terrain
        .map(|(entity, spr, mesh, tf, vis, layers)| {
            let display = if spr.is_some() {
                "Sprite"
            } else if mesh.is_some() {
                "Mesh2d"
            } else {
                "unknown"
            };
            json!({
                "entity": format!("{entity:?}"),
                "display": display,
                "visibility": format!("{vis:?}"),
                "render_layers": layers.map(|l| format!("{l:?}")).unwrap_or_else(|| "default(0)".into()),
                "image_handle": spr.map(|s| handle_label(&s.image)),
                "image_matches_fallback_state": spr.is_some_and(|s| s.image == fallback.image),
                "custom_size": spr.and_then(|s| s.custom_size.map(|sz| [sz.x, sz.y])),
                "transform_xy": [tf.translation.x, tf.translation.y],
                "transform_z": tf.translation.z,
            })
        })
        .unwrap_or(json!({ "missing": true, "void_suspect": "NO_TERRAIN_DISPLAY_ENTITY" }));

    let layer_ok = match (cam, terrain) {
        (Some((_, _, _, _, _, cam_layers)), Some((_, _, _, _, _, terr_layers))) => {
            layers_intersect_camera(terr_layers, cam_layers)
        }
        _ => false,
    };

    let sim_cpu_coverage = images
        .get(&fallback.image)
        .and_then(|img| img.data.as_ref())
        .map(|data| {
            rgba_stats(data, fallback.last_w, fallback.last_h)
                .get("coverage_ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0);

    let rtt_cpu_coverage = images
        .get(sim_tex)
        .map(|img| {
            img.data.as_ref().map_or(0.0, |data| {
                rgba_stats(data, img.width(), img.height())
                    .get("coverage_ratio")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            })
        })
        .unwrap_or(0.0);

    let mut void_suspects: Vec<&str> = Vec::new();
    let mut mechanical_invariant_failed = false;
    if pending.is_some() {
        void_suspects.push("RTT_BIND_PENDING: camera may still target previous image while UI shows committed tex");
    }
    if let Some(node) = image_node {
        if node.image != *sim_tex {
            void_suspects.push("UI_IMAGE_NODE_HANDLE != SimulationMapTexture");
            mechanical_invariant_failed = true;
        }
        if node.image != bound {
            void_suspects.push("UI_IMAGE_NODE_HANDLE != barrier.bound");
        }
    } else {
        mechanical_invariant_failed = true;
    }
    if let Some((cam, _, _, vis, rt, _)) = cam {
        if !cam.is_active {
            void_suspects.push("MAIN_CAMERA_INACTIVE");
            mechanical_invariant_failed = true;
        }
        if !matches!(rt, RenderTarget::Image(_)) {
            void_suspects.push("MAIN_CAMERA_NOT_IMAGE_TARGET");
            mechanical_invariant_failed = true;
        }
        if matches!(vis, Visibility::Hidden) {
            void_suspects.push("MAIN_CAMERA_VISIBILITY_HIDDEN");
        }
    }
    if fallback.sprite_entity.is_none() {
        void_suspects.push("NO_TERRAIN_SPRITE");
    }
    if let Some((_, _, _, _, vis, _)) = terrain {
        if matches!(vis, Visibility::Hidden) {
            void_suspects.push("TERRAIN_DISPLAY_HIDDEN");
            mechanical_invariant_failed = true;
        }
    }
    if !layer_ok && terrain.is_some() && cam.is_some() {
        void_suspects.push("RENDER_LAYER_MISMATCH: terrain may not extract into MainWorldCamera RTT pass");
        mechanical_invariant_failed = true;
    }
    // GPU_RTT_VOID is raised only from the mechanical invariants already computed above
    // (camera inactive, render-target mismatch, layer intersect false, sprite not visible,
    // ImageNode missing) combined with CPU evidence that terrain content actually exists.
    // `rtt_cpu_readback` alone must NOT raise it: it is a CPU-side readback of a GPU-only
    // render target that is architecturally never written from the CPU, so it is always
    // ~0 by design — see `coverage` block below for the raw numbers.
    if sim_cpu_coverage > 0.9 && mechanical_invariant_failed {
        void_suspects.push("GPU_RTT_VOID: CPU fallback terrain painted but a mechanical RTT invariant failed — Core2d→Image extract/draw failure");
    }
    if let Some(m) = metrics {
        if view_proj_degenerate(m.view_proj) {
            void_suspects.push("VIEW_PROJ_DEGENERATE: fire/particle overlay raster gated off");
        }
        // FIRE-VIS-001: gate re-keyed to px-per-tile (camera zoom_level) — see FIRE_SPARK_MIN_PX_PER_TILE.
        if m.zoom_level < crate::render::gpu_particles::FIRE_SPARK_MIN_PX_PER_TILE {
            void_suspects.push("FIRE_ZOOM_CULLED: px-per-tile below FIRE_SPARK_MIN_PX_PER_TILE (1.5) — sparks suppressed");
        }
    }
    if !fill_valid {
        void_suspects.push("FILL_RECT_INVALID");
    }

    let primary = void_suspects.first().copied().unwrap_or(if sim_cpu_coverage > 0.9 {
        "UNKNOWN_GPU_VOID"
    } else {
        "UNKNOWN_CPU_RASTER_VOID"
    });

    json!({
        "primary_void_suspect": primary,
        "void_suspects": void_suspects,
        "bind_barrier": {
            "revision": barrier.map(|b| b.revision),
            "bound_handle": handle_label(&bound),
            "sim_tex_handle": handle_label(sim_tex),
            "bound_matches_sim_tex": bound == *sim_tex,
            "pending": pending,
        },
        "main_world_camera": cam_json,
        "hud_ui_camera": hud_json,
        "terrain_display": terrain_json,
        "render_layer_intersect_ok": layer_ok,
        "ortho_trace": ortho.map(|o| json!({
            "view_pixels": [o.view_pixels.x, o.view_pixels.y],
            "fixed_world_span": [o.fixed_width, o.fixed_height],
            "authority_zoom": o.authority_zoom,
            "desired_zoom": o.desired_zoom,
            "camera_center": [o.camera_center.x, o.camera_center.y],
        })),
        "map_camera_desired": map_desired.map(|d| json!({
            "translation": [d.translation.x, d.translation.y],
            "scale": d.scale.x,
        })),
        "extracted_camera_metrics": metrics.map(|m| json!({
            "view_pixels": [m.view_pixels.x, m.view_pixels.y],
            "zoom_level": m.zoom_level,
            "zoom_alpha": m.zoom_alpha,
            "view_proj_degenerate": view_proj_degenerate(m.view_proj),
        })),
        "fire_particle_raster": json!({
            "world_frame_instances": fire_frame.map(|f| f.instances.len()),
            "draw_globals_vertex_count": fire_globals.map(|g| g.vertex_count),
            "spark_view_culled": fire_frame.map(|f| f.spark_witness.view_culled),
            "spark_rows": fire_frame.map(|f| f.spark_witness.rows),
        }),
        "coverage": {
            "fallback_sim_cpu": sim_cpu_coverage,
            "rtt_cpu_readback": rtt_cpu_coverage,
            "note": "rtt_cpu_readback=0 is normal/expected for a GPU-only RTT (CPU never writes it) and does not by itself imply GPU_RTT_VOID; GPU_RTT_VOID is raised only when sim CPU terrain is painted AND a mechanical RTT invariant above has failed",
        },
    })
}

fn should_flush(frame: u32, last: u32) -> bool {
    if matches!(frame, 30 | 60 | 90 | 120 | 180) {
        return true;
    }
    frame >= 240 && frame.saturating_sub(last) >= 60
}

pub fn write_tactical_map_debug_witness(
    mut state: ResMut<TacticalMapDebugState>,
    q: TacticalMapDebugInputs,
    main_cam: Query<
        (
            &Camera,
            &Projection,
            &GlobalTransform,
            &Visibility,
            &RenderTarget,
            Option<&RenderLayers>,
        ),
        With<crate::gui::MainWorldCamera>,
    >,
    hud_cam: Query<(&Camera, &RenderTarget), With<crate::gui::SimulationHudUiCamera>>,
) {
    if !q.launch.as_deref().is_some_and(|l| l.test_mode()) {
        return;
    }
    let f = q.frame.0 as u32;
    if !should_flush(f, state.last_written_frame) {
        return;
    }
    state.last_written_frame = f;

    let cam_tuple = main_cam.iter().next();
    let cam_json = cam_tuple
        .map(|(cam, proj, gt, vis, _rt, _layers)| {
            json!({
                "is_active": cam.is_active,
                "order": cam.order,
                "visibility": format!("{:?}", *vis),
                "clear_color": format!("{:?}", cam.clear_color),
                "output_mode": format!("{:?}", cam.output_mode),
                "world_position": [gt.translation().x, gt.translation().y, gt.translation().z],
                "projection": format!("{:?}", proj),
            })
        })
        .unwrap_or(json!({"missing": true}));

    let image_node_ref = q.image_node.iter().next();
    let terrain_tuple = q.terrain_display.iter().next();
    let rtt_trace = build_rtt_render_trace(
        q.rtt_barrier.as_deref(),
        &q.sim_tex.0,
        image_node_ref,
        cam_tuple.map(|(a, b, c, d, e, f)| (a, b, c, d, e, f)),
        hud_cam.iter().next(),
        terrain_tuple.map(|(a, b, c, d, e, f)| (a, b, c, d, e, f)),
        q.fallback.as_ref(),
        q.ortho.as_deref(),
        q.cam_metrics.as_deref(),
        q.fire_globals.as_deref(),
        q.fire_frame.as_deref(),
        q.map_desired.as_deref().map(|r| &r.0),
        q.fill.valid,
        &q.images,
    );

    let dense_json = q.dense.as_deref().map(|c| {
        json!({
            "width": c.width,
            "height": c.height,
            "tile_count": c.tile_count(),
        })
    });

    let image_node_bind = q
        .image_node
        .iter()
        .next()
        .map(|node| {
            json!({
                "present": true,
                "handle_matches_rtt": node.image == q.sim_tex.0,
                "stretch_mode": format!("{:?}", node.image_mode),
            })
        })
        .unwrap_or(json!({ "present": false }));

    let body = json!({
        "schema": "tactical_map_debug_v2",
        "frame": f,
        "sim_time_micros": q.sim_time.as_deref().map(|t| t.0),
        "test_scene": q.launch.as_ref().map(|l| format!("{:?}", l.test_scene)),
        "world_params": {
            "width": q.params.width,
            "height": q.params.height,
            "field_storage": format!("{:?}", q.params.field_storage),
        },
        "terrain_authority": format!("{:?}", *q.authority),
        "uses_gpu_sprite_display": q.authority.uses_gpu_sprite_display(),
        "uses_cpu_fallback_raster": q.authority.uses_cpu_fallback_raster(),
        "terrain_display_path": "Sprite (P0) or Mesh2d+ColorMaterial (P1 Camera3d diag)",
        "rtt_diag_mode": q.rtt_diag.as_deref().map(|c| format!("{:?}", c.mode)),
        "world_gen": {
            "flow": q.flow.as_deref().map(|s| format!("{:?}", s.get())),
            "progress_running": q.wg_progress.as_deref().map(|p| p.running),
            "base_state": q.base_state.as_deref().map(|s| format!("{:?}", s.get())),
            "chunk_entities": q.chunks.iter().len(),
            "terrain_baked": q.fallback.sprite_entity.is_some(),
        },
        "terrain_bake_note": "GpuInstancedAtlas: CPU dirty-gated raster → GPU texture; minimap reads texture directly; tactical map = Camera2d RenderTarget::Image + same RenderLayers as sprite",
        "rtt_render_trace": rtt_trace,
        "tactical_rtt": {
            "fill_valid": q.fill.valid,
            "fill_logical_px": [q.fill.logical_size().x, q.fill.logical_size().y],
            "fill_steady_invalid_flip_count": q.fill.steady_invalid_flip_count,
            "rtt_image": image_stats(&q.images, &q.sim_tex.0),
            "image_node_bind": image_node_bind,
            "main_camera": cam_json,
        },
        "fallback_raster": {
            "sprite_entity": q.fallback.sprite_entity.is_some(),
            "sim_image": image_stats(&q.images, &q.fallback.image),
            "minimap_image": image_stats(&q.images, &q.fallback.minimap_image),
            "last_w": q.fallback.last_w,
            "last_h": q.fallback.last_h,
            "raster_revision": q.raster_dirty.revision(),
            "raster_applied_revision": q.raster_ctrl.last_applied_raster_revision(),
            "chunk_grid_dirty": q.raster_ctrl.chunk_grid.has_dirty(),
            "test_harness_boost": q.raster_policy.test_harness_boost,
            "cpu_minimap_pass": q.raster_policy.cpu_minimap_pass,
        },
        "terrain_sources": {
            "tile_marker_entities": q.tile_markers.iter().len(),
            "chunk_cell_matrix_entities": q.chunks.iter().len(),
            "test_scene_chunk_entities": q.test_chunks.iter().len(),
            "dense_cache": dense_json,
        },
        "minimap_compositor": {
            "terrain_source_label": q.compositor.terrain_source_label,
            "composite_path": format!("{:?}", q.compositor.composite_path),
            "dual_minimap_present": q.compositor.dual_minimap_present,
            "atlas_image_loaded": q.atlas.as_deref().map(|a| a.image != Handle::default()).unwrap_or(false),
        },
        "overlays_clutter": {
            "shared_overlay_fire_cells": q.overlay.chunk_fire_heat.len(),
            "overlay_revision": q.overlay.revision,
            "tile_debug_batched_overlay": q.tile_debug.use_batched_mesh_overlay,
            "fire_debug_force_visible": q.fire_override.force_visible,
            "construction_footprint_tiles": q.construction.as_deref().map(|r| r.footprint_tiles.len()).unwrap_or(0),
            "construction_paths": q.construction.as_deref().map(|r| r.paths.len()).unwrap_or(0),
        },
        "vt4": q.vt4.as_deref().map(|r| json!({
            "mismatch_count": r.vt4.mismatch_count,
            "failing_surface_mask": format!("{:#x}", r.vt4.failing_surface_mask),
            "stamp": r.vt4.stamp.tick,
        })),
        "diagnosis_hints": diagnosis_hints(
            q.authority.as_ref(),
            &q.fallback,
            &q.images,
            q.compositor.terrain_source_label,
            q.tile_debug.use_batched_mesh_overlay,
            q.fill.valid,
            q.images.get(&q.sim_tex.0).is_some(),
            q.image_node
                .iter()
                .next()
                .is_some_and(|n| n.image == q.sim_tex.0),
        ),
    });

    let wrapped = json!({
        "written_at_epoch_secs": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "path": TACTICAL_MAP_DEBUG_JSON,
        "body": body,
    });

    let path = repo_path(TACTICAL_MAP_DEBUG_JSON);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(&wrapped) {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, &text).is_ok() {
            let _ = std::fs::rename(tmp, &path);
        }
        info!(
            target: "tactical_map_debug",
            path = TACTICAL_MAP_DEBUG_JSON,
            frame = f,
            "tactical map debug witness written"
        );
    }
}

fn diagnosis_hints(
    _authority: &crate::render::TerrainRenderAuthority,
    fallback: &crate::render::TileWorldFallbackState,
    images: &Assets<Image>,
    minimap_source: &str,
    tile_debug_on: bool,
    fill_valid: bool,
    rtt_loaded: bool,
    image_node_matches_rtt: bool,
) -> Value {
    let mut hints: Vec<&str> = Vec::new();
    if !fill_valid {
        hints.push("RTT_FILL_INVALID: tactical map ImageNode fill rect invalid — check HUD layout / SimulationMapViewportFill node");
    }
    if !rtt_loaded {
        hints.push("RTT_IMAGE_NOT_LOADED: SimulationMapTexture handle missing from Assets<Image> — RTT resize or bootstrap not run");
    }
    if rtt_loaded && !image_node_matches_rtt {
        hints.push("RTT_IMAGE_NODE_MISMATCH: SimulationMapViewportFill ImageNode handle != SimulationMapTexture — UI void likely compositor bind (RTT-C-005)");
    }
    if fallback.sprite_entity.is_none() {
        hints.push("NO_TERRAIN_SPRITE: TileWorldFallbackSprite not spawned — tactical RTT will show void clear color");
    } else if let Some(img) = images.get(&fallback.image) {
        if let Some(data) = img.data.as_ref() {
            let stats = rgba_stats(data, img.width(), img.height());
            if stats
                .get("nonzero_pixels")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                == 0
            {
                hints.push("FALLBACK_IMAGE_EMPTY: CPU raster never painted sim terrain texture");
            } else {
                hints.push("FALLBACK_IMAGE_CPU_OK: sim terrain painted — if tactical map void, check RTT RenderLayers + ImageNode bind + ExtractedCameraMetrics view_proj (particles)");
            }
        } else {
            hints.push("FALLBACK_IMAGE_NO_CPU_DATA: sim terrain Image has no CPU bytes (GPU-only ok if RTT shows terrain)");
        }
    } else if fallback.image != Handle::default() {
        hints.push("FALLBACK_IMAGE_NOT_LOADED: sim terrain handle not in Assets<Image>");
    }
    // NOTE: the minimap compositor terrain input is the CPU-rastered world image
    // (`TileWorldFallbackState`) regardless of `TerrainRenderAuthority` — `minimap_source`
    // is `"world_raster"` in the normal case and `"none"` only when no world texture is
    // bound yet. `TerrainMaterialAtlasGpu` (a material-swatch palette) is never a valid
    // minimap input; see `run_minimap_compositor_pass` in
    // src/render/minimap_compositor/pass.rs.
    if minimap_source == "none" && fallback.image != Handle::default() {
        hints.push("MINIMAP_NO_TERRAIN_BOUND: minimap compositor reports no terrain source despite a world raster image existing — check resolve_minimap_texture_source routing");
    }
    if tile_debug_on {
        hints.push("TILE_DEBUG_OVERLAY_ON: red/green GPU chunk squares drawn on tactical RTT (fire + LOD debug)");
    }
    json!(hints)
}
