//! Legacy re-exports — full tree dump lives in [`crate::gui::debug::ui_layout_tree_debug`].

pub use crate::gui::debug::{
    ui_layout_tree_debug_enabled,
    ui_layout_tree_debug_enabled as ui_layout_debug_enabled,
    DebugLayoutTag,
    UiLayoutTreeDebugPlugin as HudLayoutDebugPlugin,
    UI_LAYOUT_TREE_TARGET as UI_LAYOUT_DEBUG_TARGET,
};
