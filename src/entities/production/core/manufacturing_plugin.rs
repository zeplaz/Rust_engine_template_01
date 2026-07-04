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

use crate::entities::components::{MaintenanceTimer, Operational};
use crate::entities::production::core::manufacturing::ManufacturingNode;
use crate::systems::production::default_production_manifest;
use crate::systems::sim_control::SimControlState;

pub struct ManufacturingCorePlugin;

impl Plugin for ManufacturingCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(default_production_manifest());
        app.add_systems(
            Update,
            (tick_manufacturing_nodes, operational_maintenance_timer_tick),
        );
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
        // **INDUSTRIAL-MFG-01** (board: `industrial_activation_todos.rs`) — throughput vs blueprint.
        let _ = &mut *node;
    }
}

/// Interval maintenance for entities with [`Operational`] + [`MaintenanceTimer`].
fn operational_maintenance_timer_tick(
    time: Res<Time>,
    ctrl: Res<SimControlState>,
    mut q: Query<(&mut Operational, &mut MaintenanceTimer)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }
    for (mut op, mut mt) in &mut q {
        mt.check.tick(std::time::Duration::from_secs_f32(dt));
        if mt.check.just_finished() {
            if op.emergencies.is_empty() && op.malfunctions.is_empty() {
                op.maintenance_level =
                    (op.maintenance_level + 0.04 * (1.0 - op.maintenance_level)).min(1.0);
            }
        }
    }
}
