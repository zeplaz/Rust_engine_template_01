//! Live path-tool feedback for HUD (snap + validation actions).

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct ConstructionPathFeedback {
    pub snap_hint: Option<String>,
    pub required_actions: Vec<String>,
}
