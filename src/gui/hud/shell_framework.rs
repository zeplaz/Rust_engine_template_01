//! Product shell widget registry — visibility / minimize / focus slots for the persisted HUD layout.

use bevy::prelude::*;

pub use crate::gui::ui_gates::product_shell_base_active;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProductShellWidgetId {
    Minimap,
    OverlayTray,
    CommandShell,
    ConstructionQueue,
    BuildToolbox,
    IntelTimeline,
}

impl ProductShellWidgetId {
    /// Length of `ALL` and every per-widget parallel array in the HUD shell stack.
    pub const SLOT_COUNT: usize = 6;

    pub const ALL: [Self; Self::SLOT_COUNT] = [
        Self::Minimap,
        Self::OverlayTray,
        Self::CommandShell,
        Self::ConstructionQueue,
        Self::BuildToolbox,
        Self::IntelTimeline,
    ];

    #[inline]
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Minimap => 0,
            Self::OverlayTray => 1,
            Self::CommandShell => 2,
            Self::ConstructionQueue => 3,
            Self::BuildToolbox => 4,
            Self::IntelTimeline => 5,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Minimap => "Minimap",
            Self::OverlayTray => "Overlays",
            Self::CommandShell => "Command",
            Self::ConstructionQueue => "Pending builds",
            Self::BuildToolbox => "Construction",
            Self::IntelTimeline => "Intel",
        }
    }

    #[must_use]
    pub const fn storage_key(self) -> &'static str {
        match self {
            Self::Minimap => "minimap",
            Self::OverlayTray => "overlay_tray",
            Self::CommandShell => "command_shell",
            Self::ConstructionQueue => "construction_queue",
            Self::BuildToolbox => "build_toolbox",
            Self::IntelTimeline => "intel_timeline",
        }
    }

    #[must_use]
    pub fn from_storage_key(key: &str) -> Option<Self> {
        match key {
            "minimap" => Some(Self::Minimap),
            "overlay_tray" => Some(Self::OverlayTray),
            "command_shell" => Some(Self::CommandShell),
            "construction_queue" => Some(Self::ConstructionQueue),
            "build_toolbox" => Some(Self::BuildToolbox),
            "intel_timeline" => Some(Self::IntelTimeline),
            _ => None,
        }
    }
}

pub type HudWidgetId = ProductShellWidgetId;

/// Floating shells that stay closed during `BaseState::Simulation` (the Bevy rail / tray own play chrome).
pub const SIM_SUPPRESSED_FLOATING_SHELLS: &[ProductShellWidgetId] = &[
    ProductShellWidgetId::OverlayTray,
    ProductShellWidgetId::CommandShell,
    ProductShellWidgetId::BuildToolbox,
    ProductShellWidgetId::IntelTimeline,
    ProductShellWidgetId::ConstructionQueue,
];

/// Force floating dock slots closed (sim enter + enforce).
pub fn suppress_simulation_floating_shell_slots(dock: &mut ProductShellRegistry) {
    for &id in SIM_SUPPRESSED_FLOATING_SHELLS {
        let slot = dock.slot_mut(id);
        slot.visible = false;
        slot.minimized = true;
        slot.detached = false;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShellWidgetRuntime {
    pub visible: bool,
    pub minimized: bool,
    pub detached: bool,
    pub z_order: u32,
}

pub type HudWidgetDockState = ShellWidgetRuntime;

#[derive(Resource, Clone, Debug, Default)]
pub struct ProductShellRegistry {
    pub slots: [ShellWidgetRuntime; ProductShellWidgetId::SLOT_COUNT],
    pub focused: Option<ProductShellWidgetId>,
    pub next_z: u32,
}

pub type HudDockRegistry = ProductShellRegistry;

impl ProductShellRegistry {
    pub fn slot(&self, id: ProductShellWidgetId) -> ShellWidgetRuntime {
        self.slots[id.index()]
    }

    pub fn slot_mut(&mut self, id: ProductShellWidgetId) -> &mut ShellWidgetRuntime {
        &mut self.slots[id.index()]
    }

    #[must_use]
    pub fn state(self, id: ProductShellWidgetId) -> ShellWidgetRuntime {
        self.slot(id)
    }

    pub fn set_state(&mut self, id: ProductShellWidgetId, state: ShellWidgetRuntime) {
        *self.slot_mut(id) = state;
    }

    pub fn widget_mut(&mut self, id: ProductShellWidgetId) -> &mut ShellWidgetRuntime {
        self.slot_mut(id)
    }

    pub fn focus(&mut self, id: ProductShellWidgetId) {
        self.focused = Some(id);
        let order = self.next_z;
        self.next_z = self.next_z.wrapping_add(1);
        self.slot_mut(id).z_order = order;
    }

    pub fn restore(&mut self, id: ProductShellWidgetId) {
        let slot = self.slot_mut(id);
        slot.minimized = false;
        slot.visible = true;
        self.focus(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_widget_indices_fit_slot_count() {
        assert_eq!(ProductShellWidgetId::ALL.len(), ProductShellWidgetId::SLOT_COUNT);
        for id in ProductShellWidgetId::ALL {
            assert!(id.index() < ProductShellWidgetId::SLOT_COUNT);
        }
    }

    #[test]
    fn shell_widget_storage_keys_roundtrip() {
        for id in ProductShellWidgetId::ALL {
            assert_eq!(
                ProductShellWidgetId::from_storage_key(id.storage_key()),
                Some(id)
            );
        }
    }

    #[test]
    fn shell_focus_increments_z_order() {
        let mut registry = ProductShellRegistry::default();
        registry.focus(ProductShellWidgetId::Minimap);
        let first = registry.slot(ProductShellWidgetId::Minimap).z_order;
        registry.focus(ProductShellWidgetId::BuildToolbox);
        assert!(registry.slot(ProductShellWidgetId::BuildToolbox).z_order > first);
    }
}
