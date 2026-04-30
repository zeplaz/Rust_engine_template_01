//! Per-domain serialization plugin boundary: register save/load hooks and DTO adapters only.
//! ECS mutation stays in runtime plugins on the main thread.

use bevy::prelude::*;

/// Placeholder: concrete manufacturing snapshot / manifest registration.
pub struct ConcreteSerializationPlugin;

impl Plugin for ConcreteSerializationPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register `ConcreteProductionConfig` load/save from RON/JSON.
    }
}

/// Placeholder: aluminum chain DTO registration.
pub struct AluminumSerializationPlugin;

impl Plugin for AluminumSerializationPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register `AluminumProductionConfig` persistence.
    }
}

/// Placeholder: electrical topology design-data (not runtime grid cache).
pub struct PowerSerializationPlugin;

impl Plugin for PowerSerializationPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: persist substation graph edges + plant specs as serializable DTOs.
    }
}

/// Aggregates domain serialization plugins for one `add_plugins` call.
pub struct ProductionSerializationPlugin;

impl Plugin for ProductionSerializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ConcreteSerializationPlugin,
            AluminumSerializationPlugin,
            PowerSerializationPlugin,
        ));
    }
}
