//! Keybinding snippets for always-on HUD hints (rebindable via [`InputBindings`](crate::gui::InputBindings)).

use crate::gui::input_bindings::InputBindings;

#[inline]
pub fn format_build_commit_key(bindings: &InputBindings) -> String {
    InputBindings::format_key(bindings.confirm_build_placement)
}

#[inline]
pub fn format_build_cycle_key(bindings: &InputBindings) -> String {
    InputBindings::format_key(bindings.cycle_build_planning_tool)
}

#[inline]
pub fn format_map_rotate_keys(bindings: &InputBindings) -> String {
    format!(
        "{}/{}",
        InputBindings::format_key(bindings.map_rotate_ccw),
        InputBindings::format_key(bindings.map_rotate_cw)
    )
}
