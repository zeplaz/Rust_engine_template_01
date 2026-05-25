//! Temporary GUI layout / viewport observability (Stage 5 convergence).

pub mod ui_layout_tree_debug;

pub use ui_layout_tree_debug::{
    ui_layout_tree_debug_enabled, DebugLayoutTag, UiLayoutTreeDebugPlugin, UI_LAYOUT_TREE_TARGET,
};
