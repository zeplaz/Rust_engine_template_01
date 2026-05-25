//! Revision-keyed retained widget slices — skip static egui rebuilds when unchanged.

use bevy::prelude::*;
use bevy_egui::egui;

use super::shell_framework::ProductShellWidgetId;

#[derive(Clone, Debug, Default)]
pub struct RetainedWidgetFrame {
    pub content_revision: u64,
    pub layout_revision: u64,
    pub texture_revision: u64,
    pub cached_primitives: Vec<egui::ClippedPrimitive>,
    pub cached_lines: Vec<String>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct RetainedWidgetCache {
    pub frames: [RetainedWidgetFrame; ProductShellWidgetId::SLOT_COUNT],
    pub lookups: u64,
    pub hits: u64,
    pub misses: u64,
    pub skipped_layout: u64,
    pub skipped_paint: u64,
}

impl RetainedWidgetCache {
    pub fn frame(&self, id: ProductShellWidgetId) -> &RetainedWidgetFrame {
        &self.frames[id.index()]
    }

    pub fn frame_mut(&mut self, id: ProductShellWidgetId) -> &mut RetainedWidgetFrame {
        &mut self.frames[id.index()]
    }

    pub fn cache_hit_rate(&self) -> f32 {
        if self.lookups == 0 {
            0.0
        } else {
            self.hits as f32 / self.lookups as f32
        }
    }

    pub fn should_skip_static(
        &mut self,
        id: ProductShellWidgetId,
        content_revision: u64,
        layout_revision: u64,
    ) -> bool {
        self.lookups = self.lookups.wrapping_add(1);
        let frame = self.frame(id);
        if frame.content_revision == content_revision && frame.layout_revision == layout_revision {
            self.hits = self.hits.wrapping_add(1);
            self.skipped_layout = self.skipped_layout.wrapping_add(1);
            self.skipped_paint = self.skipped_paint.wrapping_add(1);
            true
        } else {
            self.misses = self.misses.wrapping_add(1);
            false
        }
    }

    pub fn store_static(
        &mut self,
        id: ProductShellWidgetId,
        content_revision: u64,
        layout_revision: u64,
        texture_revision: u64,
        lines: Vec<String>,
    ) {
        let frame = self.frame_mut(id);
        frame.content_revision = content_revision;
        frame.layout_revision = layout_revision;
        frame.texture_revision = texture_revision;
        frame.cached_lines = lines;
        frame.cached_primitives.clear();
    }
}

pub fn draw_retained_lines_or_build(
    ui: &mut egui::Ui,
    cache: &mut RetainedWidgetCache,
    id: ProductShellWidgetId,
    content_revision: u64,
    layout_revision: u64,
    texture_revision: u64,
    mut build: impl FnMut(&mut egui::Ui) -> Vec<String>,
) {
    if cache.should_skip_static(id, content_revision, layout_revision) {
        for line in cache.frame(id).cached_lines.clone() {
            ui.label(line);
        }
        return;
    }
    let lines = build(ui);
    cache.store_static(id, content_revision, layout_revision, texture_revision, lines.clone());
    for line in lines {
        ui.label(line);
    }
}
