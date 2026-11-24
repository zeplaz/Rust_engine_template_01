//! Movable play panels: press on a panel's top bar and drag to move it. The panel's own layout
//! system still anchors it (rail slot, beside the picker …) and adds the player's offset, so a
//! moved panel keeps its place when it re-opens. Sole writer of [`HudPanelDrag::offset`].

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Height of the grab bar at the top of a panel (logical px).
pub const PANEL_GRAB_BAR_PX: f32 = 30.0;
/// Right end of the bar kept for the close button (logical px).
const PANEL_GRAB_CLOSE_PX: f32 = 34.0;

/// A panel the player can move by its top bar; `offset` is added to its anchored position.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct HudPanelDrag {
    pub offset: Vec2,
}

impl HudPanelDrag {
    /// Anchored position + the player's offset.
    #[must_use]
    pub fn place(anchor: Vec2, drag: Option<&HudPanelDrag>) -> Vec2 {
        anchor + drag.map_or(Vec2::ZERO, |d| d.offset)
    }
}

/// Panel being dragged and the cursor at the last frame (logical px).
#[derive(Default)]
pub struct PanelDragState {
    target: Option<(Entity, Vec2)>,
}

/// Press on a visible panel's top bar starts a drag; moving the mouse moves the panel.
pub fn drag_hud_panels_system(
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut panels: Query<(Entity, &ComputedNode, &UiGlobalTransform, &InheritedVisibility, &mut HudPanelDrag)>,
    mut state: Local<PanelDragState>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        state.target = None;
        return;
    };
    let scale = window.scale_factor().max(1e-3);
    if buttons.just_pressed(MouseButton::Left) {
        state.target = panels.iter().find_map(|(e, node, xf, vis, _)| {
            if !vis.get() || node.is_empty() {
                return None;
            }
            // Node rect in logical px (ComputedNode / UiGlobalTransform are physical).
            let size = node.size() / scale;
            let centre = xf.translation / scale;
            let min = centre - size * 0.5;
            let in_x = cursor.x >= min.x && cursor.x <= min.x + size.x - PANEL_GRAB_CLOSE_PX;
            let in_bar = cursor.y >= min.y && cursor.y <= min.y + PANEL_GRAB_BAR_PX;
            (in_x && in_bar).then_some((e, cursor))
        });
    }
    if !buttons.pressed(MouseButton::Left) {
        state.target = None;
        return;
    }
    let Some((e, last)) = state.target else {
        return;
    };
    let delta = cursor - last;
    if delta.length_squared() > 0.0 {
        if let Ok((_, _, _, _, mut drag)) = panels.get_mut(e) {
            drag.offset += delta;
            // Keep at least the bar on screen.
            let w = Vec2::new(window.width(), window.height());
            drag.offset = drag.offset.clamp(-w, w);
        }
        state.target = Some((e, cursor));
    }
}
