//! Resources shared by the site logistics HUD and the logistics target picker.

use bevy::prelude::*;

/// Resolved focus: a **site hub** (roll-up) or a **standalone** storage entity.
/// Set by world pick, logistics target-list window, cycle-focus hotkey, or captured bindings.
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

/// The committed road segment the player selected: endpoint tiles (`x`, row) of one drawn
/// segment of `ExecutedRoadNetwork`. Written by the map pick only.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudRoadFocus {
    pub segment: Option<(IVec2, IVec2)>,
}

/// Info-card lines for [`HudRoadFocus`] (empty = no card). Written by the road report system only.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct HudRoadReport(pub Vec<String>);
