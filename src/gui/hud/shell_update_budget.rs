//! Frame-budgeted product-shell panel refresh tiers.

use bevy::prelude::*;

use super::shell_framework::{HudDockRegistry, ProductShellRegistry, ProductShellWidgetId};

pub const BACKGROUND_PANEL_HZ: f32 = 5.0;
pub const DETACHED_UNFOCUSED_HZ: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShellRefreshPolicy {
    EveryFrame,
    BudgetedHz(f32),
    EventDriven,
    Suspended,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellWidgetRuntimeState {
    pub visible: bool,
    pub occluded: bool,
    pub focused: bool,
    pub last_refresh_frame: u64,
    pub refresh_policy: ShellRefreshPolicy,
}

impl Default for ShellWidgetRuntimeState {
    fn default() -> Self {
        Self {
            visible: false,
            occluded: false,
            focused: false,
            last_refresh_frame: 0,
            refresh_policy: ShellRefreshPolicy::Suspended,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellRefreshTier {
    Active,
    Background,
    MinimizedEventOnly,
    Suspended,
}

#[derive(Resource, Clone, Debug)]
pub struct ProductShellUpdateBudget {
    pub background_hz: f32,
    pub detached_hz: f32,
    pub frame_index: u64,
    pub bypass_throttle: bool,
    runtime: [ShellWidgetRuntimeState; ProductShellWidgetId::SLOT_COUNT],
    last_refresh_secs: [f32; ProductShellWidgetId::SLOT_COUNT],
    event_pending: [bool; ProductShellWidgetId::SLOT_COUNT],
    hidden: [bool; ProductShellWidgetId::SLOT_COUNT],
    occluded: [bool; ProductShellWidgetId::SLOT_COUNT],
}

impl Default for ProductShellUpdateBudget {
    fn default() -> Self {
        Self {
            background_hz: BACKGROUND_PANEL_HZ,
            detached_hz: DETACHED_UNFOCUSED_HZ,
            frame_index: 0,
            bypass_throttle: false,
            runtime: [ShellWidgetRuntimeState::default(); ProductShellWidgetId::SLOT_COUNT],
            last_refresh_secs: [-1.0; ProductShellWidgetId::SLOT_COUNT],
            event_pending: [false; ProductShellWidgetId::SLOT_COUNT],
            hidden: [false; ProductShellWidgetId::SLOT_COUNT],
            occluded: [false; ProductShellWidgetId::SLOT_COUNT],
        }
    }
}

impl ProductShellUpdateBudget {
    pub fn begin_frame(&mut self) {
        self.frame_index = self.frame_index.wrapping_add(1);
    }

    /// Deep-debug witness: shell refresh queue shape (no widget internals).
    #[must_use]
    pub fn debug_queue_snapshot(&self) -> serde_json::Value {
        let mut visible = 0u32;
        let mut occluded = 0u32;
        let mut focused = 0u32;
        let mut suspended = 0u32;
        for rt in &self.runtime {
            if rt.visible {
                visible += 1;
            }
            if rt.occluded {
                occluded += 1;
            }
            if rt.focused {
                focused += 1;
            }
            if matches!(rt.refresh_policy, ShellRefreshPolicy::Suspended) {
                suspended += 1;
            }
        }
        serde_json::json!({
            "frame_index": self.frame_index,
            "bypass_throttle": self.bypass_throttle,
            "background_hz": self.background_hz,
            "detached_hz": self.detached_hz,
            "visible_widgets": visible,
            "occluded_widgets": occluded,
            "focused_widgets": focused,
            "suspended_widgets": suspended,
        })
    }

    pub fn set_bypass_throttle(&mut self, bypass: bool) {
        self.bypass_throttle = bypass;
    }

    pub fn set_hidden(&mut self, id: ProductShellWidgetId, hidden: bool) {
        self.hidden[id.index()] = hidden;
    }

    pub fn set_occluded(&mut self, id: ProductShellWidgetId, occluded: bool) {
        self.occluded[id.index()] = occluded;
    }

    pub fn bump_widget_event(&mut self, id: ProductShellWidgetId) {
        self.event_pending[id.index()] = true;
    }

    pub fn runtime(&self, id: ProductShellWidgetId) -> ShellWidgetRuntimeState {
        self.runtime[id.index()]
    }

    pub fn resolve_refresh_policy(
        &self,
        registry: &ProductShellRegistry,
        id: ProductShellWidgetId,
        host_open: bool,
    ) -> ShellRefreshPolicy {
        let slot = registry.slot(id);
        if !host_open || !slot.visible || self.hidden[id.index()] {
            return ShellRefreshPolicy::Suspended;
        }
        if slot.minimized {
            return ShellRefreshPolicy::EventDriven;
        }
        if registry.focused == Some(id) {
            return ShellRefreshPolicy::EveryFrame;
        }
        if slot.detached {
            return ShellRefreshPolicy::BudgetedHz(self.detached_hz);
        }
        if self.occluded[id.index()] {
            return ShellRefreshPolicy::BudgetedHz(self.background_hz);
        }
        ShellRefreshPolicy::BudgetedHz(self.background_hz)
    }

    pub fn refresh_tier(
        &self,
        registry: &ProductShellRegistry,
        id: ProductShellWidgetId,
        host_open: bool,
    ) -> ShellRefreshTier {
        match self.resolve_refresh_policy(registry, id, host_open) {
            ShellRefreshPolicy::EveryFrame => ShellRefreshTier::Active,
            ShellRefreshPolicy::BudgetedHz(_) => ShellRefreshTier::Background,
            ShellRefreshPolicy::EventDriven => ShellRefreshTier::MinimizedEventOnly,
            ShellRefreshPolicy::Suspended => ShellRefreshTier::Suspended,
        }
    }

    pub fn should_refresh(
        &mut self,
        registry: &ProductShellRegistry,
        id: ProductShellWidgetId,
        host_open: bool,
        now_secs: f32,
    ) -> bool {
        if self.bypass_throttle {
            self.note_widget_refreshed(id, now_secs);
            return true;
        }
        let policy = self.resolve_refresh_policy(registry, id, host_open);
        let slot = registry.slot(id);
        let focused = registry.focused == Some(id);
        self.runtime[id.index()] = ShellWidgetRuntimeState {
            visible: slot.visible && host_open,
            occluded: self.occluded[id.index()],
            focused,
            last_refresh_frame: self.runtime[id.index()].last_refresh_frame,
            refresh_policy: policy,
        };
        let refresh = match policy {
            ShellRefreshPolicy::Suspended => false,
            ShellRefreshPolicy::EveryFrame => true,
            ShellRefreshPolicy::BudgetedHz(hz) => {
                let interval = 1.0 / hz.max(0.25);
                let last = self.last_refresh_secs[id.index()];
                last < 0.0 || now_secs - last >= interval
            }
            ShellRefreshPolicy::EventDriven => self.event_pending[id.index()],
        };
        if refresh {
            self.note_widget_refreshed(id, now_secs);
        }
        refresh
    }

    pub fn note_widget_refreshed(&mut self, id: ProductShellWidgetId, now_secs: f32) {
        self.last_refresh_secs[id.index()] = now_secs;
        self.runtime[id.index()].last_refresh_frame = self.frame_index;
        self.event_pending[id.index()] = false;
    }

    pub fn sync_occlusion_from_registry(&mut self, registry: &ProductShellRegistry) {
        let top = registry.focused;
        for id in ProductShellWidgetId::ALL {
            let slot = registry.slot(id);
            self.hidden[id.index()] = !slot.visible;
            self.occluded[id.index()] =
                top.is_some_and(|focused| focused != id && slot.visible && !slot.minimized);
        }
    }
}

pub fn sync_product_shell_update_budget(
    dock: Res<HudDockRegistry>,
    mut budget: ResMut<ProductShellUpdateBudget>,
    interaction_budget: Res<super::hud_interaction_budget::HudFrameBudget>,
) {
    budget.background_hz = interaction_budget.dynamic_background_hz;
    budget.sync_occlusion_from_registry(&dock);
}

pub fn advance_product_shell_update_budget(mut budget: ResMut<ProductShellUpdateBudget>) {
    budget.begin_frame();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_panel_throttles_to_five_hz() {
        let mut budget = ProductShellUpdateBudget::default();
        let mut registry = ProductShellRegistry::default();
        registry
            .slot_mut(ProductShellWidgetId::Transmission)
            .visible = true;
        assert!(budget.should_refresh(&registry, ProductShellWidgetId::Transmission, true, 0.0));
        assert!(!budget.should_refresh(&registry, ProductShellWidgetId::Transmission, true, 0.05));
        assert!(budget.should_refresh(&registry, ProductShellWidgetId::Transmission, true, 0.21));
    }

    #[test]
    fn detached_unfocused_uses_ten_hz_policy() {
        let budget = ProductShellUpdateBudget::default();
        let mut registry = ProductShellRegistry::default();
        registry
            .slot_mut(ProductShellWidgetId::OverlaysPanel)
            .visible = true;
        registry.slot_mut(ProductShellWidgetId::OverlaysPanel).detached = true;
        assert!(matches!(
            budget.resolve_refresh_policy(&registry, ProductShellWidgetId::OverlaysPanel, true),
            ShellRefreshPolicy::BudgetedHz(hz) if (hz - DETACHED_UNFOCUSED_HZ).abs() < f32::EPSILON
        ));
    }
}
