//! Which planning overlays are visible during ghost placement.

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct BuildOverlayVisibility {
    pub terrain: bool,
    pub network: bool,
    pub cost: bool,
}

impl Default for BuildOverlayVisibility {
    fn default() -> Self {
        Self {
            terrain: true,
            network: true,
            cost: false,
        }
    }
}
