//! CLI / process launch configuration inserted before [`crate::engine::EnginePlugin`] runs.

use bevy::prelude::Resource;

/// `--test weather|fire|atmosphere|visual`: generated world + sim debug defaults for VFX / systems checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TestScene {
    #[default]
    None,
    Weather,
    Fire,
    /// Strong wind + hot chunks for atmosphere field / advection smoke tests.
    Atmosphere,
    /// **Recommended** smoke / fire / GPU field / precip demo: combines wind, fire, weather, and fuel cues.
    Visual,
}

/// Effective launch flags (from `main`, environment wrappers, etc.).
#[derive(Clone, Debug, Resource, Default)]
pub struct EngineLaunchArgs {
    pub test_scene: TestScene,
}

impl EngineLaunchArgs {
    #[must_use]
    pub fn from_test_cli_flag(raw: Option<String>) -> Self {
        let test_scene = match raw {
            None => TestScene::None,
            Some(s) => match s.to_lowercase().as_str() {
                "weather" => TestScene::Weather,
                "fire" => TestScene::Fire,
                "atmosphere" => TestScene::Atmosphere,
                "visual" | "vfx" => TestScene::Visual,
                other => {
                    bevy::log::warn!(
                        "Unknown --test mode {other:?}; use `weather`, `fire`, `atmosphere`, or `visual`. Ignored."
                    );
                    TestScene::None
                }
            },
        };

        Self { test_scene }
    }

    #[must_use]
    pub fn test_mode(&self) -> bool {
        self.test_scene != TestScene::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_visual_maps() {
        let a = EngineLaunchArgs::from_test_cli_flag(Some("visual".into()));
        assert_eq!(a.test_scene, TestScene::Visual);
        let b = EngineLaunchArgs::from_test_cli_flag(Some("vfx".into()));
        assert_eq!(b.test_scene, TestScene::Visual);
    }

    #[test]
    fn cli_atmosphere_maps() {
        let a = EngineLaunchArgs::from_test_cli_flag(Some("atmosphere".into()));
        assert_eq!(a.test_scene, TestScene::Atmosphere);
        assert!(a.test_mode());
    }
}
