// LEGACY MODULE (not actively wired):
// This file uses Bevy 0.9 / pre-stable APIs:
//   - `AppBuilder` (removed in 0.10; use `App`)
//   - `Light` (renamed to `PointLight` in 0.10)
//   - `Handle<ColorMaterial>::set_uniform` (removed — does not exist in wgpu-based Bevy)
//   - `.system()` suffix (removed in 0.12; use `add_systems(Update, fn)`)
//
// TODO: rewrite using `PointLight`, `DirectionalLight`, wgpu shaders
//       and `add_systems(Update, ...)` in a new `LightRuntimePlugin`.

use bevy::prelude::*;

const MAX_LIGHTS: usize = 16;

#[derive(Debug)]
pub struct LightData {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
}

#[derive(Debug, Default)]
pub struct ActiveLights {
    pub lights: Vec<LightData>,
}

pub struct LocalLightPlugin;

impl Plugin for LocalLightPlugin {
    fn build(&self, _app: &mut App) {
        // Intentionally empty: systems have been removed pending rewrite.
    }
}
