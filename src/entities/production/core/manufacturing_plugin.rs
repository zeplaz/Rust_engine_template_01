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
use std::collections::HashMap;

use crate::entities::components::{MaintenanceTimer, Operational};
use crate::entities::production::core::manufacturing::{
    ManufacturingBlueprint, ManufacturingDomain, ManufacturingNode, ManufacturingOutputBuffers,
    BUFFER_TAG_DRAGON_TEETH_UNIT, BUFFER_TAG_MINE_UNIT, MFG_DRAGON_TEETH_V1, MFG_MINE_UNIT_V1,
};
use crate::systems::production::default_production_manifest;
use crate::systems::sim_control::SimControlState;

/// In-memory blueprints keyed by id (**INDUSTRIAL-MFG-01** + **COD-DEPLOYABLE-RECIPE-001**).
#[derive(Resource, Debug, Clone)]
pub struct ManufacturingBlueprintRegistry {
    pub by_id: HashMap<String, ManufacturingBlueprint>,
}

impl Default for ManufacturingBlueprintRegistry {
    fn default() -> Self {
        let mut by_id = HashMap::new();
        for bp in [
            ManufacturingBlueprint {
                id: "mfg_concrete_batch_v1".into(),
                domain: ManufacturingDomain::Concrete,
                process_tags: vec!["batch".into(), "aggregate".into()],
                throughput_target: 12.0,
                output_buffer_tag: None,
            },
            ManufacturingBlueprint {
                id: "mfg_aluminum_cast_v1".into(),
                domain: ManufacturingDomain::Aluminum,
                process_tags: vec!["cast".into(), "smelt".into()],
                throughput_target: 4.5,
                output_buffer_tag: None,
            },
            ManufacturingBlueprint {
                id: "mfg_power_aux_v1".into(),
                domain: ManufacturingDomain::Power,
                process_tags: vec!["aux".into()],
                throughput_target: 1.0,
                output_buffer_tag: None,
            },
            // Deployable prefabs — same Custom domain; heavy vs light = throughput only.
            ManufacturingBlueprint {
                id: MFG_DRAGON_TEETH_V1.into(),
                domain: ManufacturingDomain::Custom,
                process_tags: vec!["deployable".into(), "heavy_unit".into()],
                throughput_target: 1.0,
                output_buffer_tag: Some(BUFFER_TAG_DRAGON_TEETH_UNIT.into()),
            },
            ManufacturingBlueprint {
                id: MFG_MINE_UNIT_V1.into(),
                domain: ManufacturingDomain::Custom,
                process_tags: vec!["deployable".into(), "light_unit".into()],
                throughput_target: 3.0,
                output_buffer_tag: Some(BUFFER_TAG_MINE_UNIT.into()),
            },
        ] {
            by_id.insert(bp.id.clone(), bp);
        }
        Self { by_id }
    }
}

impl ManufacturingBlueprintRegistry {
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ManufacturingBlueprint> {
        self.by_id.get(id)
    }
}

/// Catalog → blueprint id for industrial activation (**MFG-03-DOMAIN-WIRE**).
///
/// Mirrors `IndustrialSupplyChainRole::from_catalog_id` without importing `construction`
/// (construction already depends on `entities` — avoid a cycle).
#[must_use]
pub fn manufacturing_blueprint_id_for_catalog(catalog_id: &str) -> Option<&'static str> {
    match catalog_id {
        "concrete_aggregate_mine"
        | "concrete_cement_kiln"
        | "concrete_cement_kiln_geopolymer"
        | "concrete_mixer_plant"
        | "concrete_mixer_geopolymer"
        | "concrete_basic_production_plant"
        | "concrete_production_plant_copy" => Some("mfg_concrete_batch_v1"),
        "aluminum_bauxite_mine"
        | "aluminum_alumina_refinery"
        | "aluminum_smelter1"
        | "aluminum_fabrication_plant" => Some("mfg_aluminum_cast_v1"),
        "utilities_coal_plant"
        | "grid_substation"
        | "grid_distribution_transformer" => Some("mfg_power_aux_v1"),
        _ => None,
    }
}

/// Build a fresh [`ManufacturingNode`] for a catalog building, if the id is in-domain.
#[must_use]
pub fn manufacturing_node_for_catalog(catalog_id: &str) -> Option<ManufacturingNode> {
    manufacturing_blueprint_id_for_catalog(catalog_id).map(|blueprint_id| ManufacturingNode {
        blueprint_id: blueprint_id.into(),
        local_efficiency: 1.0,
        current_throughput: 0.0,
    })
}

pub struct ManufacturingCorePlugin;

impl Plugin for ManufacturingCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(default_production_manifest())
            .init_resource::<ManufacturingBlueprintRegistry>()
            .add_systems(
                Update,
                (tick_manufacturing_nodes, operational_maintenance_timer_tick),
            );
    }
}

/// Drive node throughput toward blueprint target; honour `SimControlState` for determinism.
///
/// When the blueprint has `output_buffer_tag` and the entity has [`ManufacturingOutputBuffers`],
/// credits that tag by `current_throughput × dt_scale` (plant-local stock for COD-P3 haul).
pub fn tick_manufacturing_nodes(
    ctrl: Res<SimControlState>,
    blueprints: Res<ManufacturingBlueprintRegistry>,
    mut nodes: Query<(&mut ManufacturingNode, Option<&mut ManufacturingOutputBuffers>)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let scale = ctrl.dt_scale().max(0.0);
    if scale <= 0.0 {
        return;
    }
    for (mut node, buffers) in nodes.iter_mut() {
        let bp = blueprints.get(node.blueprint_id.as_str());
        let target = bp.map(|b| b.throughput_target).unwrap_or(1.0);
        // Efficiency drifts toward 1.0; throughput = target × efficiency.
        let eff = node.local_efficiency.clamp(0.05, 1.25);
        let toward = 1.0;
        node.local_efficiency = eff + (toward - eff) * 0.02 * scale;
        node.current_throughput = target * node.local_efficiency.clamp(0.05, 1.25);

        let Some(tag) = bp.and_then(|b| b.output_buffer_tag.as_deref()) else {
            continue;
        };
        let Some(mut buffers) = buffers else {
            continue;
        };
        let credit = node.current_throughput.max(0.0) * scale;
        buffers.credit(tag, credit);
    }
}

/// Spawn helpers for Custom deployable recipes (no catalog plant mapping — DR-MIL-MUNITIONS-INDUSTRY).
#[must_use]
pub fn manufacturing_deployable_bundle(
    blueprint_id: &str,
) -> Option<(ManufacturingNode, ManufacturingOutputBuffers)> {
    match blueprint_id {
        MFG_DRAGON_TEETH_V1 | MFG_MINE_UNIT_V1 => Some((
            ManufacturingNode {
                blueprint_id: blueprint_id.into(),
                local_efficiency: 1.0,
                current_throughput: 0.0,
            },
            ManufacturingOutputBuffers::default(),
        )),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manufacturing_tick_mutates_throughput() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimControlState>()
            .init_resource::<ManufacturingBlueprintRegistry>()
            .add_systems(Update, tick_manufacturing_nodes);

        let e = app
            .world_mut()
            .spawn(ManufacturingNode {
                blueprint_id: "mfg_concrete_batch_v1".into(),
                local_efficiency: 0.5,
                current_throughput: 0.0,
            })
            .id();

        app.update();
        let node = app.world().get::<ManufacturingNode>(e).unwrap();
        assert!(node.current_throughput > 0.0);
        assert!(node.local_efficiency > 0.5);
    }

    #[test]
    fn catalog_maps_to_domain_blueprints() {
        assert_eq!(
            manufacturing_blueprint_id_for_catalog("aluminum_smelter1"),
            Some("mfg_aluminum_cast_v1")
        );
        assert_eq!(
            manufacturing_blueprint_id_for_catalog("concrete_mixer_plant"),
            Some("mfg_concrete_batch_v1")
        );
        assert!(manufacturing_blueprint_id_for_catalog("builtin:road").is_none());
    }

    #[test]
    fn deployable_recipes_registered_custom_domain() {
        let reg = ManufacturingBlueprintRegistry::default();
        let teeth = reg.get(MFG_DRAGON_TEETH_V1).expect("dragon teeth recipe");
        assert_eq!(teeth.domain, ManufacturingDomain::Custom);
        assert_eq!(
            teeth.output_buffer_tag.as_deref(),
            Some(BUFFER_TAG_DRAGON_TEETH_UNIT)
        );
        let mines = reg.get(MFG_MINE_UNIT_V1).expect("mine unit recipe");
        assert_eq!(mines.domain, ManufacturingDomain::Custom);
        assert_eq!(
            mines.output_buffer_tag.as_deref(),
            Some(BUFFER_TAG_MINE_UNIT)
        );
        // Heavy vs light = throughput only (same domain, no authority fork).
        assert!(teeth.throughput_target < mines.throughput_target);
    }

    #[test]
    fn deployable_tick_credits_tagged_buffers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimControlState>()
            .init_resource::<ManufacturingBlueprintRegistry>()
            .add_systems(Update, tick_manufacturing_nodes);

        let (node, buffers) = manufacturing_deployable_bundle(MFG_DRAGON_TEETH_V1).unwrap();
        let e = app.world_mut().spawn((node, buffers)).id();
        let (node_m, buffers_m) = manufacturing_deployable_bundle(MFG_MINE_UNIT_V1).unwrap();
        let e_m = app.world_mut().spawn((node_m, buffers_m)).id();

        app.update();

        let teeth = app.world().get::<ManufacturingOutputBuffers>(e).unwrap();
        let mines = app.world().get::<ManufacturingOutputBuffers>(e_m).unwrap();
        assert!(
            teeth.get(BUFFER_TAG_DRAGON_TEETH_UNIT) > 0.0,
            "dragon_teeth_unit must credit on tick"
        );
        assert!(
            mines.get(BUFFER_TAG_MINE_UNIT) > 0.0,
            "mine_unit must credit on tick"
        );
        // Light throughput should credit more per tick at equal efficiency.
        assert!(mines.get(BUFFER_TAG_MINE_UNIT) > teeth.get(BUFFER_TAG_DRAGON_TEETH_UNIT));
    }
}
