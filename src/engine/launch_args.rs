//! CLI / process launch configuration inserted before [`crate::engine::EnginePlugin`] runs.

use bevy::prelude::Resource;

/// `--test weather|fire`: generate a tiny world, enter simulation, and surface sim debug defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TestScene {
    #[default]
    None,
    Weather,
    Fire,
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
                other => {
                    bevy::log::warn!(
                        "Unknown --test mode {other:?}; use `weather` or `fire`. Ignored."
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
