//! Interaction latency instrumentation — click, drag, resize, hover, tooltip.

use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct InteractionLatencyMetrics {
    pub click_to_response_ms: f32,
    pub drag_latency_ms: f32,
    pub panel_resize_latency_ms: f32,
    pub tooltip_resolve_ms: f32,
    pub hover_resolve_ms: f32,
    pub scroll_latency_ms: f32,
    pub map_preview_redraw_ms: f32,
    pub map_minimap_redraw_ms: f32,
    pub last_widget_label: String,
    pending_click_secs: Option<f32>,
    pending_drag_secs: Option<f32>,
    pending_resize_secs: Option<f32>,
    pending_tooltip_secs: Option<f32>,
    pending_hover_secs: Option<f32>,
    pending_scroll_secs: Option<f32>,
}

impl InteractionLatencyMetrics {
    pub fn note_click(&mut self, now_secs: f32) {
        self.pending_click_secs = Some(now_secs);
    }

    pub fn note_click_response(&mut self, now_secs: f32) {
        if let Some(started) = self.pending_click_secs.take() {
            self.click_to_response_ms = ((now_secs - started) * 1000.0).max(0.0);
        }
    }

    pub fn note_drag_start(&mut self, now_secs: f32) {
        self.pending_drag_secs = Some(now_secs);
    }

    pub fn note_drag_frame(&mut self, now_secs: f32) {
        if let Some(started) = self.pending_drag_secs {
            self.drag_latency_ms = ((now_secs - started) * 1000.0).max(0.0);
        }
    }

    pub fn note_resize_start(&mut self, now_secs: f32) {
        self.pending_resize_secs = Some(now_secs);
    }

    pub fn note_resize_end(&mut self, now_secs: f32) {
        if let Some(started) = self.pending_resize_secs.take() {
            self.panel_resize_latency_ms = ((now_secs - started) * 1000.0).max(0.0);
        }
    }

    pub fn note_tooltip_pending(&mut self, now_secs: f32) {
        self.pending_tooltip_secs = Some(now_secs);
    }

    pub fn note_tooltip_resolved(&mut self, now_secs: f32) {
        if let Some(started) = self.pending_tooltip_secs.take() {
            self.tooltip_resolve_ms = ((now_secs - started) * 1000.0).max(0.0);
        }
    }

    pub fn note_hover_pending(&mut self, now_secs: f32) {
        self.pending_hover_secs = Some(now_secs);
    }

    pub fn note_hover_resolved(&mut self, now_secs: f32) {
        if let Some(started) = self.pending_hover_secs.take() {
            self.hover_resolve_ms = ((now_secs - started) * 1000.0).max(0.0);
        }
    }

    pub fn note_scroll(&mut self, now_secs: f32) {
        self.pending_scroll_secs = Some(now_secs);
    }

    pub fn note_scroll_frame(&mut self, now_secs: f32) {
        if let Some(started) = self.pending_scroll_secs {
            self.scroll_latency_ms = ((now_secs - started) * 1000.0).max(0.0);
        }
    }

    pub fn note_map_preview_redraw(&mut self, ms: f32) {
        self.map_preview_redraw_ms = ms.max(0.0);
        self.last_widget_label = "world_preview".into();
    }

    pub fn note_map_minimap_redraw(&mut self, ms: f32) {
        self.map_minimap_redraw_ms = ms.max(0.0);
        self.last_widget_label = "minimap".into();
    }
}

pub fn refresh_interaction_latency_metrics(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut metrics: ResMut<InteractionLatencyMetrics>,
) {
    let now = time.elapsed_secs();
    if mouse.just_pressed(MouseButton::Left) {
        metrics.note_click(now);
    }
    if mouse.just_released(MouseButton::Left) {
        metrics.note_click_response(now);
        metrics.note_resize_end(now);
    }
    if mouse.pressed(MouseButton::Left) {
        metrics.note_drag_start(now);
        metrics.note_drag_frame(now);
        metrics.note_resize_start(now);
    }
    if keys.any_pressed([KeyCode::PageUp, KeyCode::PageDown, KeyCode::Home, KeyCode::End]) {
        metrics.note_scroll(now);
    }
    metrics.note_scroll_frame(now);
}
