//! Planning strip tool context (roads / utilities / military / …).

use bevy::prelude::*;

/// High-level build mode for the operational planning strip (P2-F).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToolContext {
    #[default]
    None,
    Roads,
    Rail,
    Utilities,
    Military,
    Industry,
    Ecology,
    Civil,
}

impl ToolContext {
    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            ToolContext::None => "none",
            ToolContext::Roads => "roads",
            ToolContext::Rail => "rail",
            ToolContext::Utilities => "utilities",
            ToolContext::Military => "military",
            ToolContext::Industry => "industry",
            ToolContext::Ecology => "ecology",
            ToolContext::Civil => "civil",
        }
    }

    #[inline]
    pub fn next(self) -> Self {
        match self {
            ToolContext::None => ToolContext::Roads,
            ToolContext::Roads => ToolContext::Rail,
            ToolContext::Rail => ToolContext::Utilities,
            ToolContext::Utilities => ToolContext::Military,
            ToolContext::Military => ToolContext::Industry,
            ToolContext::Industry => ToolContext::Ecology,
            ToolContext::Ecology => ToolContext::Civil,
            ToolContext::Civil => ToolContext::None,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct BuildStripState {
    pub active: ToolContext,
}

impl Default for BuildStripState {
    fn default() -> Self {
        Self {
            active: ToolContext::None,
        }
    }
}
