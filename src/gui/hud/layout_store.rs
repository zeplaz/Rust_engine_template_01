//! HUD widget layout persistence — Wave **S** RON DTOs (presentation only).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

use super::shell_framework::{HudDockRegistry, ProductShellWidgetId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HudWidgetRectR8 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HudWidgetLayoutEntryR8 {
    pub widget: String,
    pub rect: HudWidgetRectR8,
    pub minimized: bool,
    pub detached: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HudLayoutCollectionR8 {
    pub schema_version: u32,
    pub widgets: Vec<HudWidgetLayoutEntryR8>,
}

impl HudLayoutCollectionR8 {
    pub const CURRENT_SCHEMA: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            widgets: Vec::new(),
        }
    }

    pub fn upsert(&mut self, entry: HudWidgetLayoutEntryR8) {
        if let Some(row) = self.widgets.iter_mut().find(|row| row.widget == entry.widget) {
            *row = entry;
        } else {
            self.widgets.push(entry);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HudWidgetFrame {
    pub pos: Vec2,
    pub size: Vec2,
    pub initialized: bool,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct HudLayoutStore {
    frames: HashMap<ProductShellWidgetId, HudWidgetFrame>,
    layout_captures_applied: u64,
    frame_captures: u32,
    frame_captures_by_widget: HashMap<ProductShellWidgetId, u32>,
}

impl HudLayoutStore {
    pub(crate) const CAPTURE_EPS: f32 = 0.5;

    #[must_use]
    pub fn frame(&self, id: ProductShellWidgetId) -> HudWidgetFrame {
        self.frames.get(&id).copied().unwrap_or_default()
    }

    pub fn set_frame(&mut self, id: ProductShellWidgetId, frame: HudWidgetFrame) {
        self.frames.insert(id, frame);
    }

    /// Drop persisted rects so the next open uses [`super::shell_framework::shell_default_window_pos`].
    pub fn reset_all_frames(&mut self) {
        self.frames.clear();
    }

    #[must_use]
    pub fn layout_captures_applied(&self) -> u64 {
        self.layout_captures_applied
    }

    pub fn bump_layout_captures_applied(&mut self, id: ProductShellWidgetId) {
        self.layout_captures_applied = self.layout_captures_applied.wrapping_add(1);
        self.frame_captures = self.frame_captures.saturating_add(1);
        *self.frame_captures_by_widget.entry(id).or_insert(0) += 1;
    }

    #[must_use]
    pub fn frame_captures(&self) -> u32 {
        self.frame_captures
    }

    #[must_use]
    pub fn top_frame_capture_offender(&self) -> Option<ProductShellWidgetId> {
        self.frame_captures_by_widget
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(id, _)| *id)
    }

    pub fn begin_frame(&mut self) {
        self.frame_captures = 0;
        self.frame_captures_by_widget.clear();
    }

    #[must_use]
    pub fn capture_window_if_changed(
        &mut self,
        id: ProductShellWidgetId,
        response: &egui::Response,
    ) -> bool {
        let rect = response.rect;
        let next = HudWidgetFrame {
            pos: Vec2::new(rect.min.x, rect.min.y),
            size: Vec2::new(rect.size().x, rect.size().y),
            initialized: true,
        };
        let prev = self.frame(id);
        if prev.initialized {
            let pos_delta = (prev.pos - next.pos).length_squared();
            let size_delta = (prev.size - next.size).length_squared();
            if pos_delta < Self::CAPTURE_EPS * Self::CAPTURE_EPS
                && size_delta < Self::CAPTURE_EPS * Self::CAPTURE_EPS
            {
                return false;
            }
        }
        self.set_frame(id, next);
        self.bump_layout_captures_applied(id);
        true
    }

    pub fn capture_window(&mut self, id: ProductShellWidgetId, response: &egui::Response) {
        let _ = self.capture_window_if_changed(id, response);
    }

    pub fn apply_window<'a>(
        &self,
        id: ProductShellWidgetId,
        window: egui::Window<'a>,
        default_pos: egui::Pos2,
        default_size: [f32; 2],
    ) -> egui::Window<'a> {
        let frame = self.frame(id);
        if frame.initialized {
            window
                .default_pos(egui::pos2(frame.pos.x, frame.pos.y))
                .default_size([frame.size.x, frame.size.y])
        } else {
            window.default_pos(default_pos).default_size(default_size)
        }
    }

    #[must_use]
    pub fn to_collection(&self, dock: &HudDockRegistry) -> HudLayoutCollectionR8 {
        let mut collection = HudLayoutCollectionR8::new();
        for id in ProductShellWidgetId::ALL {
            let frame = self.frame(id);
            if !frame.initialized {
                continue;
            }
            let state = dock.slot(id);
            collection.upsert(HudWidgetLayoutEntryR8 {
                widget: id.storage_key().into(),
                rect: HudWidgetRectR8 {
                    x: frame.pos.x,
                    y: frame.pos.y,
                    width: frame.size.x,
                    height: frame.size.y,
                },
                minimized: state.minimized,
                detached: state.detached,
            });
        }
        collection
    }

    pub fn apply_collection(&mut self, collection: &HudLayoutCollectionR8) {
        for entry in &collection.widgets {
            let Some(id) = ProductShellWidgetId::from_storage_key(&entry.widget) else {
                continue;
            };
            self.set_frame(
                id,
                HudWidgetFrame {
                    pos: Vec2::new(entry.rect.x, entry.rect.y),
                    size: Vec2::new(entry.rect.width, entry.rect.height),
                    initialized: true,
                },
            );
        }
    }

    /// Applies Wave **S** layout DTO to frames and dock slot flags.
    pub fn apply_collection_with_dock(
        &mut self,
        collection: &HudLayoutCollectionR8,
        dock: &mut HudDockRegistry,
    ) {
        self.apply_collection(collection);
        for entry in &collection.widgets {
            let Some(id) = ProductShellWidgetId::from_storage_key(&entry.widget) else {
                continue;
            };
            let slot = dock.slot_mut(id);
            slot.minimized = entry.minimized;
            slot.detached = entry.detached;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_layout_collection_ron_roundtrip() {
        let mut collection = HudLayoutCollectionR8::new();
        collection.upsert(HudWidgetLayoutEntryR8 {
            widget: "minimap".into(),
            rect: HudWidgetRectR8 {
                x: 24.0,
                y: 96.0,
                width: 320.0,
                height: 280.0,
            },
            minimized: false,
            detached: true,
        });
        let ron = ron::ser::to_string(&collection).expect("serialize");
        let back: HudLayoutCollectionR8 = ron::from_str(&ron).expect("deserialize");
        assert_eq!(collection, back);
    }
}
