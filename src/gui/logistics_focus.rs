//! Resources shared by the site logistics HUD and the logistics target picker.

use bevy::prelude::*;

/// Resolved focus: a **site hub** (roll-up) or a **standalone** storage entity.
/// Set by 3D pick, toolbar / panels strip, target-list window, cycle-focus hotkey, or captured bindings.
#[derive(Resource, Default, Debug)]
pub struct HudLogisticsFocus {
    pub tracked_entity: Option<Entity>,
}

/// Refresh cadence for the logistics HUD text (read-only).
#[derive(Resource, Debug)]
pub struct HudAggregateSettings {
    pub summary_interval_secs: f32,
    pub(crate) accumulator: f32,
}

impl Default for HudAggregateSettings {
    fn default() -> Self {
        Self {
            summary_interval_secs: 0.25,
            accumulator: 0.0,
        }
    }
}
