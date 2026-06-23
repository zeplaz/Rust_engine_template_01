//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner render_pipeline_agent
//! @orchestrator-do-not-cleanup
//! Strategic / macro icon instance scaffold (Visual Aid v2 VA6).

use bevy::prelude::*;

use super::representation_governance::ScaffoldContract;
use super::representation_policy::RepresentationResult;
use super::world_representation::WorldLodBand;

pub const STRATEGIC_ICON_SCAFFOLD: ScaffoldContract = ScaffoldContract {
    owner: "gui/strategic_icon_instances",
    intended_replacement: "RenderProjectionGraph icon instance slice",
    exit_condition: "Macro band draws icon instances from RepresentationResult only",
    removal_trigger: "parallel icon ECS extract",
};

#[derive(Clone, Debug, Default)]
pub struct StrategicIconInstance {
    pub world_pos: Vec2,
    pub size: f32,
    pub tint: [f32; 4],
}

#[derive(Resource, Clone, Debug, Default)]
pub struct StrategicIconInstanceBuffer {
    pub instances: Vec<StrategicIconInstance>,
}

/// Scaffold: populate icon buffer when macro/strategic band (no parallel extract yet).
pub fn sync_strategic_icon_instances_scaffold(
    rep: Res<RepresentationResult>,
    harness: Option<Res<crate::dev::VisualAidV2HarnessState>>,
    mut buffer: ResMut<StrategicIconInstanceBuffer>,
    mut witness: ResMut<crate::dev::VisualAidV2Witness>,
) {
    buffer.instances.clear();
    let macro_band = matches!(
        rep.world_lod_band,
        WorldLodBand::Macro | WorldLodBand::Strategic
    );
    let harness_macro_probe = harness
        .as_ref()
        .is_some_and(|h| h.macro_icon_probe);
    if (macro_band && rep.building_visual_simplified) || harness_macro_probe {
        buffer.instances.push(StrategicIconInstance {
            world_pos: Vec2::ZERO,
            size: 12.0,
            tint: [0.2, 0.65, 0.95, 0.9],
        });
    }
    witness.macro_icon_instance_count = buffer.instances.len() as u32;
    let _ = STRATEGIC_ICON_SCAFFOLD.is_declared();
}

pub struct StrategicIconInstancesPlugin;

impl Plugin for StrategicIconInstancesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StrategicIconInstanceBuffer>()
            .add_systems(
                Update,
                sync_strategic_icon_instances_scaffold
                    .run_if(crate::gui::ui_gates::in_simulation_or_editor),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategic_icon_scaffold_contract_declared() {
        assert!(STRATEGIC_ICON_SCAFFOLD.is_declared());
    }
}
