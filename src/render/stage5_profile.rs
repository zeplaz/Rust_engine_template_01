//! Minimal render-profile marker kept only until `gui/tactical/map_camera.rs` drops its
//! `Stage5ReadinessProfile` trace hook (file was under concurrent edit during the witness purge).

use bevy::prelude::*;

/// Always [`Self::HEADLESS`] — nothing inserts `FULL_APP` any more.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stage5ReadinessProfile {
    pub full_app: bool,
}

impl Stage5ReadinessProfile {
    pub const FULL_APP: Self = Self { full_app: true };
    pub const HEADLESS: Self = Self { full_app: false };
}
