//! Map-view startup gates and deferred interaction commits.
//!
//! **vm-07:** Deferred map input is keyed through [`MapViewInteractionByView`] (per-surface fields).
//! [`ViewHandle`] is the stable id for routing hover / diagnostics.

use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy_egui::egui;

use super::MapViewInstanceId;
use crate::gui::editor::world_preview::{
    PreviewPathAuthority, WorldPreviewLifecycle, WorldPreviewReady, WorldPreviewRenderTargetRegistry,
    WorldPreviewTexture,
};
use crate::gui::map_view::view_state::MapViewInstances;
use crate::gui::map_view::backend::{
    resolve_minimap_texture_source, resolve_world_preview_texture_source,
};
use crate::gui::map_view::ResolvedMapViewFrames;
use crate::gui::MinimapShellState;
use crate::render::MinimapRenderTargetRegistry;
use crate::render::{ResolvedViewports, TileWorldFallbackState};

/// Stable handle for routing map UI input to one surface (world preview vs minimap vs sim map).
pub type ViewHandle = MapViewInstanceId;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapViewReadyState {
    pub texture_ready: bool,
    pub projection_ready: bool,
    pub first_frame_committed: bool,
}

impl MapViewReadyState {
    #[must_use]
    pub fn ready_to_bind(&self) -> bool {
        self.texture_ready && self.projection_ready
    }

    #[must_use]
    pub fn ready_to_present(&self) -> bool {
        self.ready_to_bind() && self.first_frame_committed
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MapViewReadyStates {
    pub world_preview: MapViewReadyState,
    pub minimap: MapViewReadyState,
}

impl MapViewReadyStates {
    #[must_use]
    pub fn get(&self, id: MapViewInstanceId) -> &MapViewReadyState {
        match id {
            MapViewInstanceId::Minimap => &self.minimap,
            _ => &self.world_preview,
        }
    }

    pub fn mark_frame_committed(&mut self, id: MapViewInstanceId) {
        match id {
            MapViewInstanceId::Minimap => self.minimap.first_frame_committed = true,
            _ => self.world_preview.first_frame_committed = true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MapViewViewportSuggestion {
    pub active: bool,
    pub logical_size: Vec2,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MapViewViewportSuggestions {
    pub minimap_panel: MapViewViewportSuggestion,
}

/// Product-shell pointer is dragging chrome; map panels defer to [`MapShellPointerGate::shell_pointer_active`].
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MapShellPointerGate {
    pub shell_pointer_active: bool,
}

/// Deferred world-preview map input (editor / central panel).
#[derive(Clone, Debug, Default)]
pub struct WorldPreviewInteractionBuffer {
    pub pan_delta: Vec2,
    pub zoom_factor: f32,
    pub zoom_anchor: Option<Vec2>,
    pub zoom_center: Vec2,
}

impl WorldPreviewInteractionBuffer {
    pub fn queue_pan(&mut self, delta: Vec2) {
        self.pan_delta += delta;
    }

    pub fn queue_zoom(&mut self, factor: f32, anchor: Vec2, center: Vec2) {
        self.zoom_factor *= factor;
        self.zoom_anchor = Some(anchor);
        self.zoom_center = center;
    }

    fn reset_frame(&mut self) {
        self.pan_delta = Vec2::ZERO;
        self.zoom_factor = 1.0;
        self.zoom_anchor = None;
        self.zoom_center = Vec2::ZERO;
    }
}

/// Deferred minimap shell input.
#[derive(Clone, Debug, Default)]
pub struct MinimapInteractionBuffer {
    pub scroll_zoom: f32,
    pub focus_world: Option<Vec2>,
    pub focus_double_click: bool,
    pub panel_extent: Option<Vec2>,
}

impl MinimapInteractionBuffer {
    pub fn queue_scroll_zoom(&mut self, delta: f32) {
        self.scroll_zoom += delta;
    }

    pub fn queue_focus(&mut self, world: Vec2, double_clicked: bool) {
        self.focus_world = Some(world);
        self.focus_double_click = double_clicked;
    }

    pub fn queue_panel_extent(&mut self, extent: Vec2) {
        self.panel_extent = Some(extent);
    }

    fn reset_frame(&mut self) {
        self.scroll_zoom = 0.0;
        self.focus_world = None;
        self.focus_double_click = false;
        self.panel_extent = None;
    }
}

/// Single ECS resource for all deferred map UI queues (world preview + minimap shells).
#[derive(Resource, Clone, Debug, Default)]
pub struct MapViewInteractionByView {
    pub world_preview: WorldPreviewInteractionBuffer,
    pub minimap: MinimapInteractionBuffer,
}

/// Last map surface that received pointer hover (diagnostics / future routing). Optional.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActiveMapViewInput(pub Option<ViewHandle>);

impl ActiveMapViewInput {
    /// When true, gameplay [`crate::gui::map_camera::MapCameraDesired`] must not receive
    /// keyboard / edge / grip / scroll zoom — those surfaces own input for this frame family.
    ///
    /// World Preview blocks only while the preview window is open **and** hovered (see
    /// [`clear_active_map_view_input_before_map_widgets`] + preview window close cleanup).
    /// Minimap keeps its own zoom controls and must not steal main-world scroll.
    #[must_use]
    pub fn blocks_main_world_map_camera_input(self) -> bool {
        matches!(self.0, Some(MapViewInstanceId::WorldPreview))
    }
}

/// Clears hover routing **before** egui map widgets run so stale `Some(_)` cannot stick across frames.
pub fn clear_active_map_view_input_before_map_widgets(mut active: ResMut<ActiveMapViewInput>) {
    active.0 = None;
}

pub fn sync_shell_layout_drag_gate(
    mut contexts: bevy_egui::EguiContexts,
    mut pending_layout: ResMut<crate::gui::hud::PendingHudLayoutCommit>,
    mut gate: ResMut<MapShellPointerGate>,
) -> Result {
    let Ok(ctx) = contexts.ctx_mut() else {
        return Ok(());
    };
    // Only treat active drags as shell capture — `any_down` made every click freeze HUD layout
    // and fight world-gen / dock widgets for the whole press duration.
    let pointer_active = ctx.input(|input| input.pointer.is_decidedly_dragging());
    pending_layout.set_drag_active(pointer_active);
    gate.shell_pointer_active = pointer_active;
    Ok(())
}

pub fn sync_map_view_ready_states(
    frames: Res<ResolvedMapViewFrames>,
    resolved: Res<ResolvedViewports>,
    preview_tex: Res<WorldPreviewTexture>,
    minimap: Res<MinimapShellState>,
    fallback: Res<TileWorldFallbackState>,
    registry: Res<WorldPreviewRenderTargetRegistry>,
    minimap_registry: Res<MinimapRenderTargetRegistry>,
    path: Res<PreviewPathAuthority>,
    lifecycle: Res<WorldPreviewLifecycle>,
    preview_ready: Res<WorldPreviewReady>,
    mut ready: ResMut<MapViewReadyStates>,
) {
    let preview_source = resolve_world_preview_texture_source(&path, &registry, &preview_tex);
    let preview_frame = frames.get(MapViewInstanceId::WorldPreview);
    let lifecycle_allows = lifecycle.phase.allows_texture_bind();
    let gate_ready = preview_ready.0;
    ready.world_preview.texture_ready = gate_ready
        && lifecycle_allows
        && *preview_source.handle() != Handle::default();
    ready.world_preview.projection_ready = gate_ready
        && lifecycle_allows
        && resolved.world_preview.valid
        && viewport_extent_ready(preview_frame.viewport_extent);

    let minimap_source = resolve_minimap_texture_source(&minimap, &fallback, &minimap_registry);
    let minimap_frame = frames.get(MapViewInstanceId::Minimap);
    ready.minimap.texture_ready = *minimap_source.handle() != Handle::default();
    ready.minimap.projection_ready = viewport_extent_ready(minimap_frame.viewport_extent);
}

pub fn commit_map_view_viewport_suggestions(
    mut suggestions: ResMut<MapViewViewportSuggestions>,
    mut shell: ResMut<MinimapShellState>,
    pending_layout: Res<crate::gui::hud::PendingHudLayoutCommit>,
) {
    if !pending_layout.can_emit_layout_capture() {
        return;
    }
    if suggestions.minimap_panel.active {
        shell.panel_viewport_suggestion_active = true;
        shell.panel_viewport_suggestion_logical_size = suggestions.minimap_panel.logical_size;
        // PERF-INSTR-VFX-001: interaction-buffer path into the suggestion. After the prior fix the
        // `queue_panel_extent` caller was removed, so this should be SILENT — if it logs, a writer
        // re-activated `suggestions.minimap_panel`.
        crate::render::trace_minimap_size_writer(
            "commit_suggestions",
            shell.panel_viewport_suggestion_logical_size.x,
            shell.panel_viewport_suggestion_logical_size.y,
        );
        suggestions.minimap_panel.active = false;
    }
}

pub fn update_world_preview_view(
    gate: Res<MapShellPointerGate>,
    mut hub: ResMut<MapViewInteractionByView>,
    mut views: ResMut<MapViewInstances>,
) {
    if gate.shell_pointer_active {
        return;
    }

    let interaction = &mut hub.world_preview;
    let view = &mut views.world_preview;
    if interaction.pan_delta != Vec2::ZERO {
        view.camera_center += interaction.pan_delta;
    }
    if (interaction.zoom_factor - 1.0).abs() > f32::EPSILON {
        if let Some(anchor) = interaction.zoom_anchor {
            let center = interaction.zoom_center;
            let before = anchor - center;
            let after = before * interaction.zoom_factor;
            view.camera_center += before - after;
        }
        view.zoom = (view.zoom * interaction.zoom_factor).clamp(
            crate::gui::editor::world_preview::layers::PreviewLayers::ZOOM_MIN,
            crate::gui::editor::world_preview::layers::PreviewLayers::ZOOM_MAX,
        );
    }
}

pub fn update_minimap_view(
    gate: Res<MapShellPointerGate>,
    mut hub: ResMut<MapViewInteractionByView>,
    mut views: ResMut<MapViewInstances>,
    mut shell: ResMut<MinimapShellState>,
    mut suggestions: ResMut<MapViewViewportSuggestions>,
) {
    if gate.shell_pointer_active {
        return;
    }

    let interaction = &mut hub.minimap;
    let view = &mut views.minimap;
    if interaction.scroll_zoom != 0.0 {
        view.zoom_target =
            (view.zoom_target + interaction.scroll_zoom).clamp(0.35, 4.0);
    }
    if let Some(world) = interaction.focus_world {
        view.camera_center = world;
        shell.focus_world_tile(world);
        shell.pending_camera_focus_world = Some(world);
        shell.diagnostic_ui_wrote_camera = true;
        if interaction.focus_double_click {
            // Minimap zoom follows this view only (not main `MapCameraDesired` tactical scale).
            let base = view.zoom_target.max(0.001);
            let focus_zoom = (base * 1.15).clamp(0.35, 4.0);
            view.zoom_target = focus_zoom;
            shell.pending_camera_focus_zoom = Some(focus_zoom);
        }
    }
    if let Some(extent) = interaction.panel_extent {
        suggestions.minimap_panel.active = true;
        suggestions.minimap_panel.logical_size = extent;
    }
}

pub fn commit_map_view_interaction_system(
    gate: Res<MapShellPointerGate>,
    mut hub: ResMut<MapViewInteractionByView>,
) {
    if gate.shell_pointer_active {
        hub.world_preview.reset_frame();
        hub.minimap.reset_frame();
        return;
    }
    hub.world_preview.reset_frame();
    hub.minimap.reset_frame();
}

pub fn paint_map_view_placeholder(ui: &mut egui::Ui, rect: egui::Rect, label: &str) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 20, 24));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(12.0),
        ui.visuals().weak_text_color(),
    );
}

#[must_use]
pub fn viewport_extent_ready(extent: UVec2) -> bool {
    extent.x > 0 && extent.y > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_gate_requires_all_flags() {
        let mut ready = MapViewReadyState::default();
        assert!(!ready.ready_to_present());
        ready.texture_ready = true;
        ready.projection_ready = true;
        assert!(!ready.ready_to_present());
        ready.first_frame_committed = true;
        assert!(ready.ready_to_present());
    }

    #[test]
    fn active_map_input_blocks_world_preview_hover_only() {
        assert!(
            ActiveMapViewInput(Some(MapViewInstanceId::WorldPreview))
                .blocks_main_world_map_camera_input()
        );
        assert!(
            !ActiveMapViewInput(Some(MapViewInstanceId::Minimap))
                .blocks_main_world_map_camera_input()
        );
        assert!(!ActiveMapViewInput(None).blocks_main_world_map_camera_input());
        assert!(
            !ActiveMapViewInput(Some(MapViewInstanceId::SimulationMap))
                .blocks_main_world_map_camera_input()
        );
    }
}
