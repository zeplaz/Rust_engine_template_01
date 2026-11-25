//! Movable play panels: press on a panel and drag to move it — on its top bar, or anywhere on it
//! for cards ([`HudPanelDrag::anywhere`]); a press on a button or slider inside never starts a
//! drag. The panel's own layout system still anchors it (rail slot, beside the picker, screen
//! corner …) and adds the player's offset, so a moved panel keeps its place when it re-opens.
//! Sole writer of [`HudPanelDrag::offset`]. Every draggable panel also blocks map clicks under it
//! (`simulation_pointer_gate`).

use bevy::prelude::*;
use bevy::ui_widgets::Button;
use bevy::window::PrimaryWindow;

use crate::gui::style::widgets::UiSlider;

/// Height of the grab bar at the top of a panel (logical px).
pub const PANEL_GRAB_BAR_PX: f32 = 30.0;
/// Right end of the bar kept for the close button (logical px).
const PANEL_GRAB_CLOSE_PX: f32 = 34.0;

/// A panel the player can move; `offset` is added to its anchored position.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct HudPanelDrag {
    pub offset: Vec2,
    /// Grab anywhere on the panel (cards), not only its top bar.
    pub anywhere: bool,
}

impl HudPanelDrag {
    /// A card moved by a press anywhere on it.
    #[must_use]
    pub fn card() -> Self {
        Self { offset: Vec2::ZERO, anywhere: true }
    }

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

/// Logical-px rect of a laid-out node, `None` when empty.
fn logical_rect(node: &ComputedNode, xf: &UiGlobalTransform, scale: f32) -> Option<Rect> {
    if node.is_empty() {
        return None;
    }
    let size = node.size() / scale;
    let centre = xf.translation / scale;
    Some(Rect::from_center_size(centre, size))
}

/// Press on a visible panel (its bar, or anywhere for cards — never on a widget) starts a drag;
/// moving the mouse moves the panel.
#[allow(clippy::type_complexity)]
pub fn drag_hud_panels_system(
    window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut panels: Query<(Entity, &ComputedNode, &UiGlobalTransform, &InheritedVisibility, &mut HudPanelDrag)>,
    widgets: Query<(&ComputedNode, &UiGlobalTransform, &InheritedVisibility), Or<(With<Button>, With<UiSlider>)>>,
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
        let on_widget = widgets
            .iter()
            .any(|(n, xf, vis)| vis.get() && logical_rect(n, xf, scale).is_some_and(|r| r.contains(cursor)));
        state.target = if on_widget {
            None
        } else {
            panels.iter().find_map(|(e, node, xf, vis, drag)| {
                if !vis.get() {
                    return None;
                }
                let r = logical_rect(node, xf, scale)?;
                let grab = if drag.anywhere {
                    r.contains(cursor)
                } else {
                    cursor.x >= r.min.x
                        && cursor.x <= r.max.x - PANEL_GRAB_CLOSE_PX
                        && cursor.y >= r.min.y
                        && cursor.y <= r.min.y + PANEL_GRAB_BAR_PX
                };
                grab.then_some((e, cursor))
            })
        };
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

/// A card anchored to the window's top-right corner (`right_top` px in from it), moved by the
/// player's [`HudPanelDrag`] offset. The offset is kept per `key` while the card is closed, so it
/// reopens where it was left.
#[derive(Component, Debug, Clone, Copy)]
pub struct HudCardAnchor {
    pub key: &'static str,
    pub right_top: Vec2,
}

/// Player offsets of closed and open cards, by [`HudCardAnchor::key`].
#[derive(Resource, Debug, Default, Clone)]
pub struct HudCardPlacements(pub std::collections::HashMap<&'static str, Vec2>);

impl HudCardPlacements {
    /// Node position + drag + anchor for a card about to be spawned (reopens where it was left).
    #[must_use]
    pub fn card(&self, key: &'static str, right_top: Vec2) -> (HudCardAnchor, HudPanelDrag, Val, Val) {
        let offset = self.0.get(key).copied().unwrap_or(Vec2::ZERO);
        (
            HudCardAnchor { key, right_top },
            HudPanelDrag { offset, anywhere: true },
            Val::Px(right_top.x - offset.x),
            Val::Px(right_top.y + offset.y),
        )
    }
}

/// Place every anchored card at its corner + the player's offset, and remember the offset.
pub fn place_hud_cards_system(
    mut placements: ResMut<HudCardPlacements>,
    mut cards: Query<(&mut Node, &HudPanelDrag, &HudCardAnchor), Changed<HudPanelDrag>>,
) {
    for (mut node, drag, anchor) in &mut cards {
        node.right = Val::Px(anchor.right_top.x - drag.offset.x);
        node.top = Val::Px(anchor.right_top.y + drag.offset.y);
        if placements.0.get(anchor.key) != Some(&drag.offset) {
            placements.0.insert(anchor.key, drag.offset);
        }
    }
}
