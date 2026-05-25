//! Unified product shell — dock, minimize, resize, focus, and layout persistence.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::engine::states::BaseState;

use super::layout_store::HudLayoutStore;
use super::pending_hud_layout_commit::PendingHudLayoutCommit;
use super::shell_update_budget::ProductShellUpdateBudget;
use super::shell_widget_timing::{ShellWidgetDiagnostics, WidgetRebuildReason};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProductShellWidgetId {
    Minimap,
    Transmission,
    OverlayTray,
    CommandShell,
    OverlaysPanel,
    ConstructionQueue,
    BuildToolbox,
    IntelTimeline,
    Explainability,
}

impl ProductShellWidgetId {
    /// Length of `ALL` and every per-widget parallel array in the HUD shell stack.
    pub const SLOT_COUNT: usize = 9;

    pub const ALL: [Self; Self::SLOT_COUNT] = [
        Self::Minimap,
        Self::Transmission,
        Self::OverlayTray,
        Self::CommandShell,
        Self::OverlaysPanel,
        Self::ConstructionQueue,
        Self::BuildToolbox,
        Self::IntelTimeline,
        Self::Explainability,
    ];

    /// Floating first-run placement (user drags/resizes; layout persists). Not corner-anchored.
    #[must_use]
    pub const fn uses_unanchored_default(self) -> bool {
        matches!(
            self,
            Self::Minimap | Self::ConstructionQueue | Self::BuildToolbox
        )
    }

    #[inline]
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Minimap => 0,
            Self::Transmission => 1,
            Self::OverlayTray => 2,
            Self::CommandShell => 3,
            Self::OverlaysPanel => 4,
            Self::ConstructionQueue => 5,
            Self::BuildToolbox => 6,
            Self::IntelTimeline => 7,
            Self::Explainability => 8,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Minimap => "Minimap",
            Self::Transmission => "Transmission",
            Self::OverlayTray => "Overlays",
            Self::CommandShell => "Command",
            Self::OverlaysPanel => "Overlay shell",
            Self::ConstructionQueue => "Pending builds",
            Self::BuildToolbox => "Construction",
            Self::IntelTimeline => "Intel",
            Self::Explainability => "Explain",
        }
    }

    #[must_use]
    pub const fn storage_key(self) -> &'static str {
        match self {
            Self::Minimap => "minimap",
            Self::Transmission => "transmission",
            Self::OverlayTray => "overlay_tray",
            Self::CommandShell => "command_shell",
            Self::OverlaysPanel => "overlays_panel",
            Self::ConstructionQueue => "construction_queue",
            Self::BuildToolbox => "build_toolbox",
            Self::IntelTimeline => "intel_timeline",
            Self::Explainability => "explainability",
        }
    }

    #[must_use]
    pub fn from_storage_key(key: &str) -> Option<Self> {
        match key {
            "minimap" => Some(Self::Minimap),
            "transmission" => Some(Self::Transmission),
            "overlay_tray" => Some(Self::OverlayTray),
            "command_shell" => Some(Self::CommandShell),
            "overlays_panel" => Some(Self::OverlaysPanel),
            "construction_queue" => Some(Self::ConstructionQueue),
            "build_toolbox" => Some(Self::BuildToolbox),
            "intel_timeline" => Some(Self::IntelTimeline),
            "explainability" => Some(Self::Explainability),
            _ => None,
        }
    }

    #[must_use]
    pub fn egui_window_id(self) -> egui::Id {
        egui::Id::new(match self {
            Self::Minimap => "product_shell_minimap",
            Self::Transmission => "product_shell_transmission",
            Self::OverlayTray => "product_shell_overlay_tray",
            Self::CommandShell => "product_shell_command",
            Self::OverlaysPanel => "product_shell_overlays",
            Self::ConstructionQueue => "product_shell_construction_queue",
            Self::BuildToolbox => "product_shell_build_toolbox",
            Self::IntelTimeline => "product_shell_intel",
            Self::Explainability => "product_shell_explainability",
        })
    }
}

pub type HudWidgetId = ProductShellWidgetId;

/// Allowed egui surfaces during `BaseState::Simulation` (PLAY-01 Phase 2B witness list).
pub const EGUI_SIM_SHELL_WIDGETS: &[&str] = &["Diagnostics_F3", "Editor_tools"];

/// Floating egui windows closed in simulation — Bevy Phase 2 chrome only (Sprint 3.2 audit).
pub const SIM_SUPPRESSED_FLOATING_SHELLS: &[ProductShellWidgetId] = &[
    ProductShellWidgetId::OverlaysPanel,
    ProductShellWidgetId::OverlayTray,
    ProductShellWidgetId::CommandShell,
    ProductShellWidgetId::BuildToolbox,
    ProductShellWidgetId::IntelTimeline,
    ProductShellWidgetId::Explainability,
    ProductShellWidgetId::ConstructionQueue,
    ProductShellWidgetId::Transmission,
];

/// Whether a floating product-shell egui body may run in this session profile.
#[must_use]
pub fn floating_product_shell_egui_active(id: ProductShellWidgetId, base: BaseState) -> bool {
    product_egui_shell_base_active(base) && !sim_suppresses_floating_shell(id, base)
}

/// PLAY-01 Sprint 3.2 — floating shells on the suppression list are editor-only.
#[must_use]
pub fn sim_suppresses_floating_shell(id: ProductShellWidgetId, base: BaseState) -> bool {
    matches!(base, BaseState::Simulation) && SIM_SUPPRESSED_FLOATING_SHELLS.contains(&id)
}

/// Whether a docked product widget may bind an egui window/texture body in this session.
#[must_use]
pub fn product_shell_widget_egui_dock_active(id: ProductShellWidgetId, base: BaseState) -> bool {
    if !product_egui_shell_base_active(base) {
        return false;
    }
    match id {
        ProductShellWidgetId::BuildToolbox => build_toolbox_egui_dock_active(base),
        ProductShellWidgetId::Minimap => minimap_egui_texture_dock_active(base),
        _ => true,
    }
}

/// Minimap egui texture dock — editor only until UX-E01 GPU minimap in sim.
#[must_use]
pub const fn minimap_egui_texture_dock_active(base: BaseState) -> bool {
    product_egui_shell_base_active(base)
}

/// BuildToolbox egui window — editor only; sim uses Bevy build rail (Phase 2B).
#[must_use]
pub const fn build_toolbox_egui_dock_active(base: BaseState) -> bool {
    product_egui_shell_base_active(base)
}

/// Left egui status rail — editor only; sim uses Bevy context rail (Phase 2B).
#[must_use]
pub const fn side_status_rail_egui_dock_active(base: BaseState) -> bool {
    product_egui_shell_base_active(base)
}

pub use crate::gui::ui_gates::product_egui_shell_base_active;

/// Force floating egui dock slots closed (PLAY-01 sim enter + enforce).
pub fn suppress_simulation_floating_shell_slots(dock: &mut ProductShellRegistry) {
    for &id in SIM_SUPPRESSED_FLOATING_SHELLS {
        let slot = dock.slot_mut(id);
        slot.visible = false;
        slot.minimized = true;
        slot.detached = false;
    }
}

/// True when every suppressed floating shell slot is closed in the dock registry.
#[must_use]
pub fn simulation_floating_shells_gated(dock: &ProductShellRegistry) -> bool {
    SIM_SUPPRESSED_FLOATING_SHELLS.iter().all(|&id| {
        let slot = dock.slot(id);
        !slot.visible && slot.minimized && !slot.detached
    })
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

/// Raise the focused product-shell window so click-to-focus wins over fixed draw order.
pub fn raise_focused_product_shell_window(ctx: &mut egui::Context, registry: &ProductShellRegistry) {
    if let Some(id) = registry.focused {
        let layer = egui::LayerId::new(egui::Order::Middle, id.egui_window_id());
        ctx.move_to_top(layer);
    }
}

pub struct ShellWindowHost<'a> {
    pub id: ProductShellWidgetId,
    pub title: &'a str,
    pub default_pos: egui::Pos2,
    pub default_size: [f32; 2],
    pub min_size: [f32; 2],
}

/// Movable first-run placement for minimap / construction / build toolbox (not corner-locked).
#[must_use]
pub fn floating_unanchored_default_pos(
    ctx: &egui::Context,
    id: ProductShellWidgetId,
    size: [f32; 2],
) -> egui::Pos2 {
    let screen = ctx.content_rect();
    let w = size[0].min(screen.width() * 0.42);
    let h = size[1].min(screen.height() * 0.5);
    let margin = 12.0;
    let top = screen.min.y + 64.0;
    let (x, y) = match id {
        ProductShellWidgetId::Minimap => (
            screen.max.x - w - margin,
            top + margin,
        ),
        ProductShellWidgetId::BuildToolbox => (
            margin,
            screen.center().y - h * 0.55,
        ),
        ProductShellWidgetId::ConstructionQueue => (
            margin,
            screen.center().y + h * 0.05,
        ),
        _ => {
            let hash = id.index() as f32;
            (
                screen.center().x - w * 0.5 + (hash - 3.0) * 36.0,
                screen.center().y - h * 0.5 + (hash - 3.0) * 28.0,
            )
        }
    };
    egui::pos2(
        x.clamp(screen.min.x + margin, screen.max.x - w - margin),
        y.clamp(top, screen.max.y - h - margin),
    )
}

/// Screen-aware grid for transmission, info rail, command shell, intel, explainability.
#[must_use]
pub fn shell_anchored_default_pos(
    ctx: &egui::Context,
    id: ProductShellWidgetId,
    size: [f32; 2],
) -> egui::Pos2 {
    let screen = ctx.content_rect();
    let w = size[0].min(screen.width() * 0.38);
    let h = size[1].min(screen.height() * 0.45);
    let margin = 12.0;
    let top = screen.min.y + 64.0;
    let right_x = screen.max.x - w - margin;
    match id {
        ProductShellWidgetId::Transmission => egui::pos2(right_x, top),
        ProductShellWidgetId::OverlayTray => egui::pos2(margin, top + 52.0),
        ProductShellWidgetId::OverlaysPanel => egui::pos2(right_x, top + 56.0),
        ProductShellWidgetId::CommandShell => {
            egui::pos2(margin, screen.max.y - h - 72.0)
        }
        ProductShellWidgetId::IntelTimeline => {
            egui::pos2(right_x, screen.center().y - h * 0.35)
        }
        ProductShellWidgetId::Explainability => egui::pos2(
            (screen.center().x - w * 0.5).clamp(margin, screen.max.x - w - margin),
            screen.max.y - h - 56.0,
        ),
        _ => egui::pos2(margin + 220.0, top + 120.0),
    }
}

#[must_use]
pub fn shell_default_window_pos(
    ctx: &egui::Context,
    id: ProductShellWidgetId,
    size: [f32; 2],
) -> egui::Pos2 {
    if id.uses_unanchored_default() {
        floating_unanchored_default_pos(ctx, id, size)
    } else {
        shell_anchored_default_pos(ctx, id, size)
    }
}

pub struct ShellWindowOutcome {
    pub open: bool,
    pub minimized: bool,
    pub detached: bool,
    pub focused: bool,
}

#[must_use]
pub fn shell_widget_runs_egui(
    registry: &ProductShellRegistry,
    id: ProductShellWidgetId,
    host_open: bool,
) -> bool {
    shell_widget_runs_egui_with_budget(registry, id, host_open, None, 0.0)
}

#[must_use]
pub fn shell_widget_runs_egui_with_budget(
    registry: &ProductShellRegistry,
    id: ProductShellWidgetId,
    host_open: bool,
    budget: Option<&mut ProductShellUpdateBudget>,
    now_secs: f32,
) -> bool {
    let slot = registry.slot(id);
    if !host_open || !slot.visible || slot.minimized {
        return false;
    }
    if let Some(budget) = budget {
        budget.should_refresh(registry, id, host_open, now_secs)
    } else {
        host_open && slot.visible && !slot.minimized
    }
}

pub fn draw_shell_window_chrome(ui: &mut egui::Ui, minimized: &mut bool, detached: &mut bool, lightweight: bool) {
    if lightweight {
        ui.label(egui::RichText::new("Dragging…").small().weak());
        return;
    }
    ui.horizontal(|ui| {
        if ui.button("Min").clicked() {
            *minimized = true;
        }
        if ui.button("Detach").clicked() {
            *detached = !*detached;
        }
    });
}

pub fn show_product_shell_window(
    ctx: &mut egui::Context,
    host: ShellWindowHost<'_>,
    layout: &HudLayoutStore,
    registry: &mut ProductShellRegistry,
    open: &mut bool,
    mut body: impl FnMut(&mut egui::Ui),
    widget_timing: Option<&mut ShellWidgetDiagnostics>,
) -> Option<egui::Response> {
    let slot = registry.slot(host.id);
    if slot.minimized || !*open || !slot.visible {
        return None;
    }

    let window = layout.apply_window(
        host.id,
        crate::gui::std_floating(egui::Window::new(host.title))
            .id(host.id.egui_window_id())
            .min_size(host.min_size),
        host.default_pos,
        host.default_size,
    );
    let mut minimized = slot.minimized;
    let mut detached = slot.detached;
    let started = std::time::Instant::now();
    let inner = window.open(open).show(ctx, |ui| {
        let response = ui.response();
        let lightweight = response.dragged() || response.drag_started();
        draw_shell_window_chrome(ui, &mut minimized, &mut detached, lightweight);
        body(ui);
    })?;
    if let Some(timing) = widget_timing {
        let elapsed = started.elapsed();
        let total_us = elapsed.as_micros() as u32;
        timing.record(
            host.id,
            total_us / 3,
            total_us.saturating_sub(total_us / 3),
            0,
            0,
            WidgetRebuildReason::EguiWindow,
        );
    }
    let focused = inner.response.hovered() || inner.response.has_focus();
    if focused {
        registry.focus(host.id);
    }
    sync_shell_slot_from_outcome(
        registry,
        host.id,
        &ShellWindowOutcome {
            open: *open,
            minimized,
            detached,
            focused,
        },
    );
    Some(inner.response)
}

pub fn draw_minimized_shell_chip(
    ui: &mut egui::Ui,
    id: ProductShellWidgetId,
    registry: &mut ProductShellRegistry,
) -> bool {
    let slot = registry.slot(id);
    if !slot.minimized {
        return false;
    }
    let clicked = ui.button(format!("{} ○", id.label())).clicked();
    if clicked {
        registry.restore(id);
    }
    clicked
}

pub fn capture_shell_layout(
    layout: &mut HudLayoutStore,
    id: ProductShellWidgetId,
    response: &egui::Response,
    pending: Option<&mut PendingHudLayoutCommit>,
) {
    if let Some(pending) = pending {
        if !pending.can_emit_layout_capture() {
            return;
        }
        let _ = pending.queue_capture(id, response, layout);
        return;
    }
    let _ = layout.capture_window_if_changed(id, response);
}

pub fn sync_shell_slot_from_outcome(
    registry: &mut ProductShellRegistry,
    id: ProductShellWidgetId,
    outcome: &ShellWindowOutcome,
) {
    let slot = registry.slot_mut(id);
    slot.visible = outcome.open;
    slot.minimized = outcome.minimized;
    slot.detached = outcome.detached;
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
    fn shell_widget_suspends_when_minimized() {
        let mut registry = ProductShellRegistry::default();
        registry.slot_mut(ProductShellWidgetId::Minimap).minimized = true;
        assert!(!shell_widget_runs_egui(
            &registry,
            ProductShellWidgetId::Minimap,
            true
        ));
    }

    #[test]
    fn shell_focus_increments_z_order() {
        let mut registry = ProductShellRegistry::default();
        registry.focus(ProductShellWidgetId::Minimap);
        let first = registry.slot(ProductShellWidgetId::Minimap).z_order;
        registry.focus(ProductShellWidgetId::Transmission);
        assert!(registry.slot(ProductShellWidgetId::Transmission).z_order > first);
    }

    #[test]
    fn phase2b_product_egui_shell_editor_only() {
        use crate::engine::states::BaseState;

        assert!(!product_egui_shell_base_active(BaseState::Simulation));
        assert!(product_egui_shell_base_active(BaseState::Editor));
        assert!(!product_shell_widget_egui_dock_active(
            ProductShellWidgetId::BuildToolbox,
            BaseState::Simulation
        ));
        assert!(!side_status_rail_egui_dock_active(BaseState::Simulation));
        assert!(!minimap_egui_texture_dock_active(BaseState::Simulation));
        assert!(product_shell_widget_egui_dock_active(
            ProductShellWidgetId::BuildToolbox,
            BaseState::Editor
        ));
        assert!(side_status_rail_egui_dock_active(BaseState::Editor));
        assert!(!floating_product_shell_egui_active(
            ProductShellWidgetId::OverlaysPanel,
            BaseState::Simulation
        ));
        let mut registry = ProductShellRegistry::default();
        suppress_simulation_floating_shell_slots(&mut registry);
        assert!(simulation_floating_shells_gated(&registry));
    }
}
