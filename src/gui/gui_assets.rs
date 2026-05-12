//! Splash / menu image handles — loaded once via [`FromWorld`] for Bevy UI paths.

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct SplashMenuAssets {
    pub splash: Handle<Image>,
    /// Reserved for main-menu chrome (`AppShellPlugin`); loaded with splash for one `AssetServer` batch.
    #[allow(dead_code)]
    pub menu_background: Handle<Image>,
}

impl FromWorld for SplashMenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            splash: assets.load("splash/splash_01.png"),
            // Reserved for main-menu chrome; use same asset until a dedicated menu plate ships.
            menu_background: assets.load("splash/splash_01.png"),
        }
    }
}
