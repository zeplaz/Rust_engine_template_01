//! Manufacturing-core runtime plugin — closes the manifest gap.
//!
//! `ProductionManifest` lists `manufacturing_core` (`src/systems/production/manifest.rs`)
//! but no plugin previously registered systems for `ManufacturingNode`. This plugin
//! gives that row a real owner without changing semantics yet.
//!
//! Designer:
//! - `prompts/designer_questions/production_economy/spec/01_data_model_manifest.md`
//! - `prompts/designer_questions/production_economy/implementation_questions_v1.md` §12–13.

use bevy::prelude::*;

use crate::entities::production::core::manufacturing::ManufacturingNode;
use crate::systems::sim_control::SimControlState;

pub struct ManufacturingCorePlugin;

impl Plugin for ManufacturingCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_manufacturing_nodes);
    }
}

/// Per-tick scaffold; honours `SimControlState` so iteration sims stay deterministic.
fn tick_manufacturing_nodes(
    ctrl: Res<SimControlState>,
    mut nodes: Query<&mut ManufacturingNode>,
) {
    if !ctrl.should_tick() {
        return;
    }
    for mut node in nodes.iter_mut() {
        // TODO: drive throughput vs blueprint, decay/efficiency curves, alert events.
        let _ = &mut *node;
    }
}
