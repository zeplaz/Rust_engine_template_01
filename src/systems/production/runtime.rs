use bevy::prelude::*;

use crate::entities::production::aluminum::AluminumRuntimePlugin;
use crate::entities::production::concrete::ConcreteRuntimePlugin;
use crate::entities::production::power::PowerRuntimePlugin;
use crate::systems::production::manifest::default_production_manifest;

/// Runtime-only production simulation plugin (ECS state mutation on main thread).
pub struct ProductionRuntimePlugin;

impl Plugin for ProductionRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(default_production_manifest())
            .add_plugins((ConcreteRuntimePlugin, AluminumRuntimePlugin, PowerRuntimePlugin));
    }
}
