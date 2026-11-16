//! Legacy Bevy UI focus state (`Interaction` type alias) until HUD systems migrate to
//! `bevy::picking::hover::Hovered` + `bevy::ui::Pressed` (CQ-85).
pub use bevy::ui::DeprecatedInteraction;