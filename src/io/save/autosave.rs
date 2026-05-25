//! Autosave cadence for Wave S incremental bundles.

use bevy::prelude::*;

use super::pipeline::SaveFlushRequested;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSaveAutosaveSettings {
    pub enabled: bool,
    pub interval_secs: f32,
    accumulator_secs: f32,
}

impl Default for WorldSaveAutosaveSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 120.0,
            accumulator_secs: 0.0,
        }
    }
}

pub fn tick_world_save_autosave(
    time: Res<Time>,
    mut settings: ResMut<WorldSaveAutosaveSettings>,
    mut flush: ResMut<SaveFlushRequested>,
) {
    if !settings.enabled {
        return;
    }
    settings.accumulator_secs += time.delta_secs();
    if settings.accumulator_secs < settings.interval_secs {
        return;
    }
    settings.accumulator_secs = 0.0;
    flush.0 = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autosave_defaults_disabled() {
        let settings = WorldSaveAutosaveSettings::default();
        assert!(!settings.enabled);
    }
}
