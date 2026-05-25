//! Presentation backend abstraction for future native shell hosts.

use bevy::prelude::*;

use super::shell_framework::{HudDockRegistry, ProductShellWidgetId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WidgetPresentationBackendKind {
    #[default]
    Egui,
    NativeStub,
}

pub trait WidgetPresentationBackend {
    fn backend_kind(self) -> WidgetPresentationBackendKind;
    fn uses_egui(self) -> bool;
}

impl WidgetPresentationBackend for WidgetPresentationBackendKind {
    fn backend_kind(self) -> WidgetPresentationBackendKind {
        self
    }

    fn uses_egui(self) -> bool {
        matches!(self, Self::Egui)
    }
}

#[derive(Clone, Debug, Default)]
pub struct WidgetShellState {
    pub visible: bool,
    pub focused: bool,
    pub content_revision: u64,
    pub layout_revision: u64,
    pub interaction_revision: u64,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct HudShellInteractionRouter {
    pub widgets: [WidgetShellState; ProductShellWidgetId::SLOT_COUNT],
    pub dock_revision: u64,
    pub telemetry_revision: u64,
}

impl HudShellInteractionRouter {
    pub fn sync_from_dock(&mut self, dock: &HudDockRegistry) {
        self.dock_revision = self.dock_revision.wrapping_add(1);
        for id in ProductShellWidgetId::ALL {
            let slot = dock.slot(id);
            let state = &mut self.widgets[id.index()];
            state.visible = slot.visible;
            state.focused = dock.focused == Some(id);
            if slot.visible {
                state.content_revision = state.content_revision.wrapping_add(1);
                state.layout_revision = state.layout_revision.wrapping_add(1);
            }
            if state.focused {
                state.interaction_revision = state.interaction_revision.wrapping_add(1);
            }
        }
        self.telemetry_revision = self.telemetry_revision.wrapping_add(1);
    }
}

#[derive(Resource, Clone, Debug)]
pub struct WidgetPresentationPolicy {
    pub default_backend: WidgetPresentationBackendKind,
    /// Transmission / briefing shell — off while build + map visuals are the priority lane.
    pub transmission_enabled: bool,
}

impl Default for WidgetPresentationPolicy {
    fn default() -> Self {
        Self {
            default_backend: WidgetPresentationBackendKind::Egui,
            transmission_enabled: false,
        }
    }
}

impl WidgetPresentationPolicy {
    #[must_use]
    pub const fn backend_for_widget(&self, _widget: ProductShellWidgetId) -> WidgetPresentationBackendKind {
        self.default_backend
    }

    #[must_use]
    pub const fn widget_enabled(&self, widget: ProductShellWidgetId) -> bool {
        match widget {
            ProductShellWidgetId::Transmission => self.transmission_enabled,
            _ => true,
        }
    }

    #[must_use]
    pub fn uses_egui(&self, backend: WidgetPresentationBackendKind) -> bool {
        backend.uses_egui()
    }
}
