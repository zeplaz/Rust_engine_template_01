//! Tag → `AudioSource` handles. Empty until assets are promoted (MCP / packs).

use std::collections::HashMap;

use bevy::prelude::*;

/// Maps mix `tag` strings to optional audio handles.
#[derive(Resource, Default, Debug)]
pub struct SfxCatalog {
    pub by_tag: HashMap<&'static str, Handle<AudioSource>>,
}

impl SfxCatalog {
    #[must_use]
    pub fn get(&self, tag: &str) -> Option<&Handle<AudioSource>> {
        self.by_tag.get(tag)
    }

    pub fn insert(&mut self, tag: &'static str, handle: Handle<AudioSource>) {
        self.by_tag.insert(tag, handle);
    }
}

/// Built-in tags the mix emits today — register assets under these keys.
pub mod tags {
    pub const AMBIENT_RAIN: &str = "ambient.rain";
    pub const AMBIENT_WIND: &str = "ambient.wind";
    pub const SPATIAL_NEAR: &str = "spatial.near_field";
    pub const UI_CLICK: &str = "ui.click";
    pub const UI_CONFIRM: &str = "ui.confirm";
    pub const UI_DENY: &str = "ui.deny";
    pub const SHOT: &str = "combat.shot";
    pub const EXPLOSION: &str = "combat.explosion";
    pub const MISSILE_LAUNCH: &str = "combat.missile_launch";
    pub const LAUNCH_WARNING: &str = "warning.launch";
    pub const WEATHER_ALERT: &str = "warning.weather";
}
