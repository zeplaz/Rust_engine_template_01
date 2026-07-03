//! One-shot FULL_APP render / projection snapshot (single frame, structured log).

use std::collections::HashMap;

use bevy::camera::{Camera, RenderTarget};
use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;
use serde::Serialize;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::{
    PreviewCameraState, PreviewPathAuthority, PreviewPresentationDebug,
    PreviewRenderMode, PreviewRenderTarget, WorldPreviewGpuCamera,
    WorldPreviewGpuRuntime, WorldPreviewTexture, WorldPreviewRenderTargetBindBarrier,
};
use crate::render::viewport_pipeline::{ResolvedViewports, ViewportPresentationMismatch};
use crate::gui::hud::layout_store::HudLayoutStore;
use crate::gui::hud::shell_framework::HudWidgetId;
use crate::gui::{map_display_rect, map_texture_uv_rect};
use crate::gui::MapViewInstances;
use crate::gui::{MapCameraDesiredRes, MinimapShellState, SimulationMapViewport};
use crate::gui::MainWorldCamera;
use crate::render::gpu_particles::WorldFireParticleFrame;
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::{tactical_fire_visual, FireVisualFramesByView};
use crate::render::sim_visual_extract::SimFireEmitterVisualExtract;
use crate::render::tile_world_fallback::{TileWorldFallbackRasterDirty, TileWorldFallbackState};
use crate::render::Stage5ReadinessProfile;
use crate::systems::sim_control::{SimStepStamp, SimTick, SimTimeMicros};
use crate::systems::weather::WeatherVfxCameraChild;

const LOG_TARGET: &str = "proc_A_dine01::render::full_render_diagnostic";

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct FullRenderDiagnosticSummary {
    pub viewport_zero_size_detected: bool,
    pub camera_count_gt_one_world_camera: bool,
    pub particle_screen_space_detected: bool,
    pub render_target_mismatch: bool,
    pub egui_rect_ne_viewport_rect: bool,
    pub stale_texture_usage: bool,
}

#[must_use]
pub fn full_render_diagnostic_has_critical_anomaly(summary: &FullRenderDiagnosticSummary) -> bool {
    summary.viewport_zero_size_detected
        || summary.camera_count_gt_one_world_camera
        || summary.particle_screen_space_detected
        || summary.render_target_mismatch
        || summary.egui_rect_ne_viewport_rect
        || summary.stale_texture_usage
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct FullRenderDiagnosticGate {
    pub armed: bool,
    pub captured: bool,
    pub min_sim_ticks: u64,
    pub warmup_frames_remaining: u32,
}

impl Default for FullRenderDiagnosticGate {
    fn default() -> Self {
        Self {
            armed: false,
            captured: false,
            min_sim_ticks: 24,
            warmup_frames_remaining: 48,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FullRenderUiSlot {
    WorldPreviewWindow,
    WorldPreviewViewport,
    MinimapWindow,
    MinimapImage,
    HudShellRoot,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct FullRenderUiLayoutProbe {
    pub rects: HashMap<FullRenderUiSlot, egui::Rect>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct FullRenderDiagnosticFeedback {
    pub ui_wrote_map_camera: bool,
    pub camera_drove_ui_follow: bool,
}

#[derive(SystemParam)]
pub struct FullRenderDiagnosticUiHook<'w> {
    gate: Res<'w, FullRenderDiagnosticGate>,
    probe: ResMut<'w, FullRenderUiLayoutProbe>,
    feedback: ResMut<'w, FullRenderDiagnosticFeedback>,
}

impl<'w> FullRenderDiagnosticUiHook<'w> {
    pub fn record_rect(&mut self, slot: FullRenderUiSlot, rect: egui::Rect) {
        record_full_render_diagnostic_ui_rect(Some(&self.gate), Some(&mut self.probe), slot, rect);
    }

    pub fn note_ui_wrote_map_camera(&mut self) {
        note_full_render_ui_wrote_map_camera(Some(&mut self.feedback));
    }

    pub fn note_camera_drove_ui_follow(&mut self) {
        note_full_render_camera_drove_ui_follow(Some(&mut self.feedback));
    }
}

pub fn record_full_render_diagnostic_ui_rect(
    gate: Option<&FullRenderDiagnosticGate>,
    probe: Option<&mut FullRenderUiLayoutProbe>,
    slot: FullRenderUiSlot,
    rect: egui::Rect,
) {
    let Some(gate) = gate else {
        return;
    };
    if !gate.armed || gate.captured {
        return;
    }
    let Some(probe) = probe else {
        return;
    };
    probe.rects.insert(slot, rect);
}

pub fn note_full_render_ui_wrote_map_camera(mut feedback: Option<&mut FullRenderDiagnosticFeedback>) {
    if let Some(feedback) = feedback.as_mut() {
        feedback.ui_wrote_map_camera = true;
    }
}

pub fn note_full_render_camera_drove_ui_follow(mut feedback: Option<&mut FullRenderDiagnosticFeedback>) {
    if let Some(feedback) = feedback.as_mut() {
        feedback.camera_drove_ui_follow = true;
    }
}

#[derive(Debug, Clone, Serialize)]
struct Rect2 {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    width: f32,
    height: f32,
}

impl From<egui::Rect> for Rect2 {
    fn from(rect: egui::Rect) -> Self {
        Self {
            min_x: rect.min.x,
            min_y: rect.min.y,
            max_x: rect.max.x,
            max_y: rect.max.y,
            width: rect.width(),
            height: rect.height(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Vec2Ser {
    x: f32,
    y: f32,
}

impl From<Vec2> for Vec2Ser {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

#[derive(Debug, Serialize)]
struct CameraTruthRow {
    entity: u64,
    camera_type: &'static str,
    priority: isize,
    active: bool,
    projection: &'static str,
    viewport_logical: Option<Rect2>,
    viewport_physical: Option<Rect2>,
    render_target: String,
}

#[derive(Debug, Serialize)]
struct ViewportUiLayoutTruth {
    egui: HashMap<String, Option<Rect2>>,
    computed_viewport_per_camera: HashMap<u64, Option<Rect2>>,
    egui_vs_gpu_viewport_mismatch: HashMap<u64, bool>,
}

#[derive(Debug, Serialize)]
struct WorldScreenProjectionState {
    camera_translation: Vec2Ser,
    camera_rotation_deg_z: f32,
    camera_zoom_scale: f32,
    visible_world_bounds: Option<Rect2>,
    map_display_rect: Option<Rect2>,
    map_texture_uv_rect: Rect2,
    world_origin_screen: Option<Vec2Ser>,
    world_origin_screen_sanity_ok: bool,
}

#[derive(Debug, Serialize)]
struct ParticleEmitterAuditRow {
    source: &'static str,
    active_count: usize,
    coordinate_space: &'static str,
    camera_binding: Option<u64>,
    inside_main_frustum: Option<usize>,
    outside_main_frustum: Option<usize>,
    bounds_world: Option<Rect2>,
    fire_particle_view_culled: Option<bool>,
    fire_spark_rows: Option<usize>,
}

#[derive(Debug, Serialize)]
struct RenderPipelinePathRow {
    pass: &'static str,
    active: bool,
    gpu_path: bool,
    cpu_path: bool,
    fallback_active: bool,
    notes: String,
}

#[derive(Debug, Serialize)]
struct TextureGpuStateRow {
    name: &'static str,
    width: u32,
    height: u32,
    revision: u64,
    last_update_frame: u64,
    stale: bool,
}

#[derive(Debug, Serialize)]
struct InputCameraFeedbackLoop {
    map_camera_desired: MapCameraDesiredSnapshot,
    map_camera_desired_changed_this_frame: bool,
    main_camera_transform_changed_this_frame: bool,
    ui_updates_camera_this_frame: bool,
    camera_updates_ui_this_frame: bool,
    follow_mode: String,
}

#[derive(Debug, Serialize)]
struct MapCameraDesiredSnapshot {
    translation: Vec2Ser,
    scale: f32,
    rotation_deg_z: f32,
}

#[derive(Debug, Serialize)]
struct AnomalyFlags {
    viewport_zero_size_detected: bool,
    camera_count_gt_one_world_camera: bool,
    particle_screen_space_detected: bool,
    render_target_mismatch: bool,
    egui_rect_ne_viewport_rect: bool,
    stale_texture_usage: bool,
}

#[derive(Debug, Serialize)]
struct FullRenderDiagnosticSnapshot {
    timestamp_unix_ms: u128,
    frame_index: u32,
    sim_step_stamp: Option<SimStepStamp>,
    camera_truth: Vec<CameraTruthRow>,
    viewport_ui_layout_truth: ViewportUiLayoutTruth,
    world_screen_projection_state: WorldScreenProjectionState,
    particle_system_audit: Vec<ParticleEmitterAuditRow>,
    render_pipeline_paths: Vec<RenderPipelinePathRow>,
    texture_gpu_state: Vec<TextureGpuStateRow>,
    input_camera_feedback_loop: InputCameraFeedbackLoop,
    anomaly_flags: AnomalyFlags,
}

fn rect2_from_aabb(min: Vec2, max: Vec2) -> Rect2 {
    Rect2 {
        min_x: min.x,
        min_y: min.y,
        max_x: max.x,
        max_y: max.y,
        width: (max.x - min.x).max(0.0),
        height: (max.y - min.y).max(0.0),
    }
}

fn projection_label(projection: Option<&Projection>) -> &'static str {
    match projection {
        Some(Projection::Orthographic(_)) => "orthographic",
        Some(Projection::Perspective(_)) => "perspective",
        Some(Projection::Custom(_)) => "custom",
        None => "unknown",
    }
}

fn render_target_label(target: &RenderTarget) -> String {
    if let Some(handle) = target.as_image() {
        return format!("image:{:?}", handle.id());
    }
    match target {
        RenderTarget::Window(_) => "screen".to_string(),
        RenderTarget::TextureView(_) => "texture_view".to_string(),
        RenderTarget::None { .. } => "none".to_string(),
        RenderTarget::Image(_) => "image".to_string(),
    }
}

fn camera_type_label(
    main_world: bool,
    preview_gpu: bool,
    weather_child: bool,
) -> &'static str {
    if main_world {
        "main_world"
    } else if preview_gpu {
        "world_preview_gpu"
    } else if weather_child {
        "weather_vfx_child"
    } else {
        "other"
    }
}

fn visible_world_bounds_ortho(
    transform: &GlobalTransform,
    projection: Option<&Projection>,
    viewport_size: Vec2,
) -> Option<Rect2> {
    let Projection::Orthographic(ortho) = projection? else {
        return None;
    };
    let scale = ortho.scale;
    if scale <= 0.0 || viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return None;
    }
    let half_w = viewport_size.x * 0.5 / scale;
    let half_h = viewport_size.y * 0.5 / scale;
    let center = transform.translation().truncate();
    Some(rect2_from_aabb(
        center - Vec2::new(half_w, half_h),
        center + Vec2::new(half_w, half_h),
    ))
}

fn particle_bounds(instances: &[crate::render::gpu_particles::GpuParticleInstance]) -> Option<Rect2> {
    if instances.is_empty() {
        return None;
    }
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for row in instances {
        let p = row.world_xyz_heat.xy();
        min = min.min(p);
        max = max.max(p);
    }
    Some(rect2_from_aabb(min, max))
}

fn count_particles_in_main_frustum(
    instances: &[crate::render::gpu_particles::GpuParticleInstance],
    bounds: Option<Rect2>,
) -> (usize, usize) {
    let Some(bounds) = bounds else {
        return (0, instances.len());
    };
    let mut inside = 0usize;
    for row in instances {
        let p = row.world_xyz_heat.xy();
        if p.x >= bounds.min_x
            && p.x <= bounds.max_x
            && p.y >= bounds.min_y
            && p.y <= bounds.max_y
        {
            inside += 1;
        }
    }
    (inside, instances.len().saturating_sub(inside))
}

#[derive(SystemParam)]
pub struct FullRenderDiagnosticReads<'w> {
    base: Res<'w, State<BaseState>>,
    sim_tick: Res<'w, SimTick>,
    sim_time: Res<'w, SimTimeMicros>,
    frame_count: Res<'w, FrameCount>,
    desired: Res<'w, MapCameraDesiredRes>,
    map_views: Res<'w, MapViewInstances>,
    feedback: Res<'w, FullRenderDiagnosticFeedback>,
    ui_probe: Res<'w, FullRenderUiLayoutProbe>,
}

#[derive(SystemParam)]
pub struct FullRenderDiagnosticSurfaces<'w> {
    sim_map_vp: Res<'w, SimulationMapViewport>,
    hud_layout: Res<'w, HudLayoutStore>,
    map_views: Res<'w, MapViewInstances>,
    preview_cam: Res<'w, PreviewCameraState>,
    preview_target: Res<'w, PreviewRenderTarget>,
    preview_authority: Res<'w, PreviewPathAuthority>,
    preview_debug: Res<'w, PreviewPresentationDebug>,
    preview_tex: Res<'w, WorldPreviewTexture>,
    preview_ui: Res<'w, crate::gui::editor::world_preview::WorldPreviewUiState>,
    resolved: Res<'w, ResolvedViewports>,
    viewport_mismatch: Res<'w, ViewportPresentationMismatch>,
    bind_barrier: Res<'w, WorldPreviewRenderTargetBindBarrier>,
    minimap_shell: Res<'w, MinimapShellState>,
    minimap_registry: Option<Res<'w, crate::render::MinimapRenderTargetRegistry>>,
    minimap_compositor: Option<Res<'w, crate::render::MinimapCompositorState>>,
    fallback: Res<'w, TileWorldFallbackState>,
    raster_dirty: Res<'w, TileWorldFallbackRasterDirty>,
    overlay: Option<Res<'w, SharedOverlayFieldBuffers>>,
    fire_by_view: Option<Res<'w, FireVisualFramesByView>>,
    fire_emitters: Option<Res<'w, SimFireEmitterVisualExtract>>,
    particle_frame: Option<Res<'w, WorldFireParticleFrame>>,
}

#[derive(SystemParam)]
pub struct FullRenderDiagnosticRuntime<'w> {
    gpu_preview_rt: Res<'w, WorldPreviewGpuRuntime>,
    weather_settings: Option<Res<'w, crate::systems::weather::WeatherVisualSettings>>,
}

pub fn arm_full_render_diagnostic_for_full_app(
    profile: Res<Stage5ReadinessProfile>,
    base: Res<State<BaseState>>,
    mut gate: ResMut<FullRenderDiagnosticGate>,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP || gate.captured {
        return;
    }
    // Defer capture until Simulation — Editor/world-gen capture left sim_frames at 0 forever.
    gate.armed = *base.get() == BaseState::Simulation;
}

pub(crate) fn emit_full_render_diagnostic_snapshot(
    profile: Res<Stage5ReadinessProfile>,
    mut gate: ResMut<FullRenderDiagnosticGate>,
    mut summary: ResMut<FullRenderDiagnosticSummary>,
    reads: FullRenderDiagnosticReads,
    surfaces: FullRenderDiagnosticSurfaces,
    runtime: FullRenderDiagnosticRuntime,
    window_q: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(
        Entity,
        &Camera,
        &RenderTarget,
        &GlobalTransform,
        Option<&Projection>,
        Has<MainWorldCamera>,
        Has<WorldPreviewGpuCamera>,
        Has<WeatherVfxCameraChild>,
    )>,
    main_cam_tf: Query<&Transform, With<MainWorldCamera>>,
    main_cam_changed: Query<(), (With<MainWorldCamera>, Changed<Transform>)>,
    weather_vfx: Query<Entity, With<WeatherVfxCameraChild>>,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    if !gate.armed || gate.captured {
        return;
    }
    if !matches!(reads.base.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }
    if reads.sim_tick.0 < gate.min_sim_ticks {
        return;
    }
    if gate.warmup_frames_remaining > 0 {
        gate.warmup_frames_remaining -= 1;
        return;
    }

    let window = window_q.iter().next();
    let scale_factor = window.map(|w| w.scale_factor()).unwrap_or(1.0);
    let window_logical = window.map(|w| Vec2::new(w.width(), w.height()));

    let mut camera_truth = Vec::new();
    let mut computed_viewport_per_camera = HashMap::new();
    let mut egui_vs_gpu_viewport_mismatch = HashMap::new();
    let mut main_world_camera_count = 0u32;
    let mut viewport_zero_size_detected = false;
    let mut particle_screen_space_detected = false;

    for (entity, camera, render_target, _transform, projection, main_world, preview_gpu, weather_child) in
        &cameras
    {
        if main_world {
            main_world_camera_count += 1;
        }
        if weather_child
            && runtime
                .weather_settings
                .as_ref()
                .is_some_and(|s| s.enabled && s.particles)
        {
            particle_screen_space_detected = true;
        }

        let logical = camera
            .logical_viewport_size()
            .or(window_logical)
            .filter(|s| s.x > 0.0 && s.y > 0.0)
            .map(|s| Rect2 {
                min_x: 0.0,
                min_y: 0.0,
                max_x: s.x,
                max_y: s.y,
                width: s.x,
                height: s.y,
            });
        let physical = logical.as_ref().map(|r| Rect2 {
            min_x: r.min_x * scale_factor,
            min_y: r.min_y * scale_factor,
            max_x: r.max_x * scale_factor,
            max_y: r.max_y * scale_factor,
            width: r.width * scale_factor,
            height: r.height * scale_factor,
        });
        if logical.as_ref().is_some_and(|r| r.width <= 0.0 || r.height <= 0.0) {
            viewport_zero_size_detected = true;
        }

        let entity_bits = entity.to_bits();
        computed_viewport_per_camera.insert(entity_bits, logical.clone());

        camera_truth.push(CameraTruthRow {
            entity: entity_bits,
            camera_type: camera_type_label(main_world, preview_gpu, weather_child),
            priority: camera.order,
            active: camera.is_active,
            projection: projection_label(projection),
            viewport_logical: logical,
            viewport_physical: physical,
            render_target: render_target_label(render_target),
        });
    }

    let mut egui_rects = HashMap::new();
    for (slot, rect) in &reads.ui_probe.rects {
        egui_rects.insert(format!("ui_probe_{slot:?}"), Some(Rect2::from(*rect)));
    }
    egui_rects.insert(
        "world_preview_window".into(),
        surfaces.preview_ui.last_window_rect.map(Rect2::from),
    );
    egui_rects.insert(
        "world_preview_viewport".to_string(),
        surfaces.preview_ui.last_viewport_rect.map(Rect2::from),
    );
    egui_rects.insert(
        "minimap_window".to_string(),
        surfaces.minimap_shell.last_window_rect.map(Rect2::from),
    );
    egui_rects.insert(
        "minimap_image".to_string(),
        surfaces.minimap_shell.last_image_rect.map(Rect2::from),
    );

    let hud_shell = surfaces.hud_layout.frame(HudWidgetId::CommandShell);
    let hud_shell_rect = if hud_shell.initialized {
        Some(Rect2 {
            min_x: hud_shell.pos.x,
            min_y: hud_shell.pos.y,
            max_x: hud_shell.pos.x + hud_shell.size.x,
            max_y: hud_shell.pos.y + hud_shell.size.y,
            width: hud_shell.size.x,
            height: hud_shell.size.y,
        })
    } else if surfaces.sim_map_vp.valid {
        Some(rect2_from_aabb(surfaces.sim_map_vp.min, surfaces.sim_map_vp.max))
    } else {
        None
    };
    egui_rects.insert("hud_shell_root".to_string(), hud_shell_rect.clone());

    let mut egui_rect_ne_viewport_rect = false;
    if !surfaces.bind_barrier.pending.is_some() {
        if let Some(preview_rect) = egui_rects
            .get("world_preview_viewport")
            .and_then(|rect| rect.as_ref())
        {
            if surfaces.resolved.world_preview.valid {
                let resolved = surfaces.resolved.world_preview.logical_size;
                let dw = (preview_rect.width - resolved.x).abs();
                let dh = (preview_rect.height - resolved.y).abs();
                if dw > 4.0 || dh > 4.0 {
                    egui_rect_ne_viewport_rect = true;
                }
            }
            if let Some((preview_entity, camera)) = cameras
                .iter()
                .find(|(_, _, _, _, _, _, preview_gpu, _)| *preview_gpu)
                .map(|(entity, camera, _, _, _, _, _, _)| (entity, camera))
            {
                if let Some(gpu_size) = camera.logical_viewport_size() {
                    let dw = (preview_rect.width - gpu_size.x).abs();
                    let dh = (preview_rect.height - gpu_size.y).abs();
                    if dw > 4.0 || dh > 4.0 {
                        egui_rect_ne_viewport_rect = true;
                    }
                }
                egui_vs_gpu_viewport_mismatch
                    .insert(preview_entity.to_bits(), egui_rect_ne_viewport_rect);
            }
        }
    }

    let main_visible_bounds = cameras
        .iter()
        .find(|(_, _, _, _, _, main, _, _)| *main)
        .and_then(|(_, _, _, transform, projection, _, _, _)| {
            visible_world_bounds_ortho(
                transform,
                projection,
                window_logical.unwrap_or(Vec2::ONE),
            )
        });

    let preview_map_rect = surfaces
        .preview_ui
        .last_viewport_rect
        .or(surfaces.preview_ui.last_window_rect)
        .map(|rect| {
            map_display_rect(
                rect.center(),
                surfaces.map_views.world_preview.camera_center,
                surfaces.map_views.world_preview.zoom,
                surfaces.preview_tex.width as f32,
                surfaces.preview_tex.height as f32,
            )
        })
        .map(Rect2::from);

    let world_origin_screen = cameras
        .iter()
        .find(|(_, _, _, _, _, main, _, _)| *main)
        .and_then(|(_, camera, _, transform, _, _, _, _)| {
            camera
                .world_to_viewport(transform, Vec3::ZERO)
                .ok()
                .map(Vec2Ser::from)
        });

    let world_origin_screen_sanity_ok = world_origin_screen.is_some();

    let world_screen_projection_state = WorldScreenProjectionState {
        camera_translation: main_cam_tf
            .single()
            .map(|tf| Vec2Ser::from(tf.translation.truncate()))
            .unwrap_or(Vec2Ser { x: 0.0, y: 0.0 }),
        camera_rotation_deg_z: main_cam_tf
            .single()
            .map(|tf| tf.rotation.to_euler(EulerRot::ZYX).0.to_degrees())
            .unwrap_or(0.0),
        camera_zoom_scale: reads.desired.scale.x,
        visible_world_bounds: main_visible_bounds.clone(),
        map_display_rect: preview_map_rect,
        map_texture_uv_rect: Rect2::from(map_texture_uv_rect()),
        world_origin_screen,
        world_origin_screen_sanity_ok,
    };

    let mut particle_system_audit = Vec::new();
    if let Some(by_view) = surfaces.fire_by_view.as_ref() {
        let frame = tactical_fire_visual(by_view);
        particle_system_audit.push(ParticleEmitterAuditRow {
            source: "fire_visual_frame_tactical",
            active_count: frame.instances.len(),
            coordinate_space: "world",
            camera_binding: cameras
                .iter()
                .find(|(_, _, _, _, _, main, _, _)| *main)
                .map(|(e, _, _, _, _, _, _, _)| e.to_bits()),
            inside_main_frustum: None,
            outside_main_frustum: None,
            bounds_world: None,
            fire_particle_view_culled: None,
            fire_spark_rows: None,
        });
    }
    if let Some(emitters) = surfaces.fire_emitters.as_ref() {
        particle_system_audit.push(ParticleEmitterAuditRow {
            source: "sim_fire_emitter_extract",
            active_count: emitters.instances.len(),
            coordinate_space: "world",
            camera_binding: cameras
                .iter()
                .find(|(_, _, _, _, _, main, _, _)| *main)
                .map(|(e, _, _, _, _, _, _, _)| e.to_bits()),
            inside_main_frustum: None,
            outside_main_frustum: None,
            bounds_world: None,
            fire_particle_view_culled: None,
            fire_spark_rows: None,
        });
    }
    if let Some(frame) = surfaces.particle_frame.as_ref() {
        let bounds = particle_bounds(&frame.instances);
        let (inside, outside) =
            count_particles_in_main_frustum(&frame.instances, main_visible_bounds.clone());
        particle_system_audit.push(ParticleEmitterAuditRow {
            source: "world_fire_particle_frame",
            active_count: frame.instances.len(),
            coordinate_space: "world",
            camera_binding: cameras
                .iter()
                .find(|(_, _, _, _, _, main, _, _)| *main)
                .map(|(e, _, _, _, _, _, _, _)| e.to_bits()),
            inside_main_frustum: Some(inside),
            outside_main_frustum: Some(outside),
            bounds_world: bounds,
            fire_particle_view_culled: Some(frame.spark_witness.view_culled),
            fire_spark_rows: Some(frame.spark_witness.rows),
        });
    }
    let precip_count = if weather_vfx.is_empty() {
        0
    } else {
        runtime
            .weather_settings
            .as_ref()
            .filter(|s| s.enabled && s.particles)
            .map(|s| s.max_precip_particles)
            .unwrap_or(0)
    };
    if precip_count > 0 {
        particle_system_audit.push(ParticleEmitterAuditRow {
            source: "weather_precip_particles",
            active_count: precip_count,
            coordinate_space: "screen_hybrid",
            camera_binding: cameras
                .iter()
                .find(|(_, _, _, _, _, main, _, _)| *main)
                .map(|(e, _, _, _, _, _, _, _)| e.to_bits()),
            inside_main_frustum: None,
            outside_main_frustum: None,
            bounds_world: None,
            fire_particle_view_culled: None,
            fire_spark_rows: None,
        });
    }

    let preview_gpu_active = surfaces.preview_cam.mode == PreviewRenderMode::GpuRenderTarget
        && runtime.gpu_preview_rt.offscreen_renderer_ready
        && surfaces.preview_authority.authoritative_surface
            == crate::gui::editor::world_preview::PreviewAuthoritativeSurface::GpuRenderTarget;
    let preview_cpu_active = !preview_gpu_active
        || surfaces.preview_authority.cpu_raster_fallback_active;
    let minimap_gpu_active = crate::render::minimap_gpu_compositor_env_enabled()
        && surfaces.minimap_shell.presentation_source
            == crate::gui::MinimapPresentationSource::SharedRenderTargetImage
        && surfaces
            .minimap_registry
            .as_ref()
            .is_some_and(|r| r.committed_image != Handle::default());
    let minimap_active = (surfaces.fallback.sprite_entity.is_some() || minimap_gpu_active)
        && surfaces.minimap_shell.visible;
    let minimap_source = match surfaces.minimap_shell.presentation_source {
        crate::gui::MinimapPresentationSource::SharedCpuRaster => "CpuRaster",
        crate::gui::MinimapPresentationSource::SharedRenderTargetImage => "GpuRenderTarget",
    };
    let minimap_composite_revision = surfaces
        .minimap_compositor
        .as_ref()
        .map(|c| c.compositor_revision)
        .unwrap_or(0);

    let render_pipeline_paths = vec![
        RenderPipelinePathRow {
            pass: "world_main_pass",
            active: main_world_camera_count > 0,
            gpu_path: true,
            cpu_path: surfaces.fallback.sprite_entity.is_some(),
            fallback_active: surfaces.fallback.sprite_entity.is_some(),
            notes: "MainWorldCamera + tile fallback sprite".to_string(),
        },
        RenderPipelinePathRow {
            pass: "preview_pass",
            active: surfaces.preview_tex.width > 0 && surfaces.preview_tex.height > 0,
            gpu_path: preview_gpu_active,
            cpu_path: preview_cpu_active,
            fallback_active: surfaces.preview_authority.cpu_raster_fallback_active,
            notes: format!(
                "authority={:?} mode={:?}",
                surfaces.preview_authority.authoritative_surface, surfaces.preview_cam.mode
            ),
        },
        RenderPipelinePathRow {
            pass: "minimap_pass",
            active: minimap_active,
            gpu_path: minimap_gpu_active,
            cpu_path: !minimap_gpu_active,
            fallback_active: !minimap_gpu_active,
            notes: format!(
                "source={minimap_source} presentation={:?} rev={} composite_rev={} rt_bound={}",
                surfaces.minimap_shell.presentation_source,
                surfaces.raster_dirty.revision(),
                minimap_composite_revision,
                surfaces
                    .minimap_registry
                    .as_ref()
                    .is_some_and(|r| r.committed_image != Handle::default()),
            ),
        },
    ];

    let minimap_stale = surfaces.minimap_shell.visible
        && surfaces.minimap_shell.cached_texture_revision < surfaces.raster_dirty.revision();
    let preview_stale = surfaces.viewport_mismatch.stale_texture_binding;
    let render_target_mismatch = preview_stale;

    let overlay_revision = surfaces.overlay.as_ref().map(|o| o.revision).unwrap_or(0);
    let texture_gpu_state = vec![
        TextureGpuStateRow {
            name: "world_preview_front",
            width: surfaces.preview_tex.width,
            height: surfaces.preview_tex.height,
            revision: surfaces.preview_debug.swap_count as u64,
            last_update_frame: reads.frame_count.0 as u64,
            stale: preview_stale,
        },
        TextureGpuStateRow {
            name: "world_preview_gpu_target",
            width: surfaces.preview_target.size.x,
            height: surfaces.preview_target.size.y,
            revision: surfaces.preview_debug.last_contract_sync_tick,
            last_update_frame: reads.frame_count.0 as u64,
            stale: preview_stale,
        },
        TextureGpuStateRow {
            name: "minimap_fallback_raster",
            width: surfaces.fallback.last_w,
            height: surfaces.fallback.last_h,
            revision: surfaces.raster_dirty.revision(),
            last_update_frame: reads.frame_count.0 as u64,
            stale: minimap_stale,
        },
        TextureGpuStateRow {
            name: "minimap_gpu_target",
            width: surfaces
                .minimap_registry
                .as_ref()
                .map(|r| r.committed_size.x)
                .unwrap_or(0),
            height: surfaces
                .minimap_registry
                .as_ref()
                .map(|r| r.committed_size.y)
                .unwrap_or(0),
            revision: minimap_composite_revision,
            last_update_frame: reads.frame_count.0 as u64,
            stale: minimap_gpu_active
                && surfaces
                    .minimap_compositor
                    .as_ref()
                    .is_some_and(|c| c.stamp == 0),
        },
        TextureGpuStateRow {
            name: "shared_overlay_fields",
            width: 0,
            height: 0,
            revision: overlay_revision,
            last_update_frame: reads.frame_count.0 as u64,
            stale: false,
        },
    ];

    let input_camera_feedback_loop = InputCameraFeedbackLoop {
        map_camera_desired: MapCameraDesiredSnapshot {
            translation: Vec2Ser::from(reads.desired.translation.truncate()),
            scale: reads.desired.scale.x,
            rotation_deg_z: reads.desired.rotation.to_euler(EulerRot::ZYX).0.to_degrees(),
        },
        map_camera_desired_changed_this_frame: reads.desired.is_changed(),
        main_camera_transform_changed_this_frame: !main_cam_changed.is_empty(),
        ui_updates_camera_this_frame: {
            let ui_wrote = surfaces.minimap_shell.diagnostic_ui_wrote_camera || reads.feedback.ui_wrote_map_camera;
            ui_wrote || reads.desired.is_changed()
        },
        camera_updates_ui_this_frame: {
            surfaces.minimap_shell.diagnostic_camera_drove_ui || reads.feedback.camera_drove_ui_follow
        },
        follow_mode: format!(
            "{:?}",
            reads.map_views.world_preview.follow_mode
        ),
    };

    let stale_texture_usage = preview_stale;
    let anomaly_flags = AnomalyFlags {
        viewport_zero_size_detected,
        camera_count_gt_one_world_camera: main_world_camera_count > 1,
        particle_screen_space_detected,
        render_target_mismatch,
        egui_rect_ne_viewport_rect,
        stale_texture_usage,
    };
    *summary = FullRenderDiagnosticSummary {
        viewport_zero_size_detected: anomaly_flags.viewport_zero_size_detected,
        camera_count_gt_one_world_camera: anomaly_flags.camera_count_gt_one_world_camera,
        particle_screen_space_detected: anomaly_flags.particle_screen_space_detected,
        render_target_mismatch: anomaly_flags.render_target_mismatch,
        egui_rect_ne_viewport_rect: anomaly_flags.egui_rect_ne_viewport_rect,
        stale_texture_usage: anomaly_flags.stale_texture_usage,
    };

    let snapshot = FullRenderDiagnosticSnapshot {
        timestamp_unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        frame_index: reads.frame_count.0,
        sim_step_stamp: Some(SimStepStamp::new(reads.sim_tick.0, reads.sim_time.0)),
        camera_truth,
        viewport_ui_layout_truth: ViewportUiLayoutTruth {
            egui: egui_rects,
            computed_viewport_per_camera,
            egui_vs_gpu_viewport_mismatch,
        },
        world_screen_projection_state,
        particle_system_audit,
        render_pipeline_paths,
        texture_gpu_state,
        input_camera_feedback_loop,
        anomaly_flags,
    };

    if let Ok(payload) = serde_json::to_string_pretty(&snapshot) {
        info!(target: LOG_TARGET, "{}", payload);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("debug_runs")
            .join(format!(
                "full_render_diagnostic_{}_{}.json",
                reads.frame_count.0, reads.sim_tick.0
            ));
        if let Err(err) = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))
        {
            warn!(target: LOG_TARGET, "full render diagnostic mkdir failed: {err}");
        } else if let Err(err) = std::fs::write(&path, &payload) {
            warn!(target: LOG_TARGET, "full render diagnostic write failed: {err}");
        } else {
            info!(target: LOG_TARGET, "wrote {}", path.display());
        }
    } else {
        warn!(target: LOG_TARGET, "full render diagnostic snapshot serialization failed");
    }

    gate.captured = true;
    gate.armed = false;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FullRenderDiagnosticSet {
    Capture,
}

pub struct FullRenderDiagnosticPlugin;

impl Plugin for FullRenderDiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FullRenderDiagnosticGate>()
            .init_resource::<FullRenderUiLayoutProbe>()
            .init_resource::<FullRenderDiagnosticFeedback>()
            .init_resource::<FullRenderDiagnosticSummary>()
            .add_systems(Startup, arm_full_render_diagnostic_for_full_app)
            .add_systems(
                Update,
                arm_full_render_diagnostic_for_full_app
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                PostUpdate,
                emit_full_render_diagnostic_snapshot
                    .in_set(FullRenderDiagnosticSet::Capture)
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn full_render_diagnostic_gate_defaults_to_not_captured() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<FullRenderDiagnosticGate>();
        let gate = app.world().resource::<FullRenderDiagnosticGate>();
        assert!(!gate.captured);
    }
}
