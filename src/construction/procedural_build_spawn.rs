//! PG-2 / PG-3.5 — spawn [`ProceduralBuildExtract`] module scenes at site transform on **Operational**.

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;

use crate::construction::iso_draw_scale::ConstructionIsoDrawScale;
use crate::construction::procedural::{
    footprint_grid_for_assembly,
    ProceduralAssemblyRequest, ProceduralModuleRegistry, StylePackRegistry,
};
use crate::economy::site_placement::site_world_position;
use crate::render::extraction::{
    assemble_procedural_build_instances, ProceduralBuildExtract, ProceduralBuildInstance,
    ProceduralModuleSceneCatalog, ProceduralModuleVisualPolicy,
};
use crate::strategic::{
    ConstructionSite, PlannedSite, ProceduralBuildingSpec, SiteConstructionPhase,
};

/// Marks a site whose PG-2 module scenes were spawned at Operational.
#[derive(Component, Debug, Clone, Copy)]
pub struct ProceduralBuildSpawned {
    pub module_count: u32,
}

/// Child marker for spawned procedural module scene roots.
#[derive(Component, Debug, Clone)]
pub struct ProceduralBuildModuleChild {
    pub module_id: String,
}

#[must_use]
fn instances_for_operational_site(
    spec: &ProceduralBuildingSpec,
    assembly_request: &ProceduralAssemblyRequest,
    extract: &ProceduralBuildExtract,
    style_packs: &StylePackRegistry,
    registry: &ProceduralModuleRegistry,
    catalog: &ProceduralModuleSceneCatalog,
) -> Vec<ProceduralBuildInstance> {
    if spec.0 == assembly_request.0
        && extract.pg2_wired
        && !extract.instances.is_empty()
        && extract.style_pack_id == spec.0.style.as_str()
    {
        return extract.instances.clone();
    }

    let Some(pack) = style_packs.get(spec.0.style.as_str()) else {
        return Vec::new();
    };
    let grid = footprint_grid_for_assembly(&spec.0);
    assemble_procedural_build_instances(&spec.0, pack, &grid, registry, catalog).instances
}

/// Spawn PG-2 module scenes under the site when phase reaches **Operational** (PG-3.5).
pub fn spawn_procedural_build_on_site_operational(
    mut commands: Commands,
    assembly_request: Res<ProceduralAssemblyRequest>,
    extract: Res<ProceduralBuildExtract>,
    style_packs: Res<StylePackRegistry>,
    registry: Res<ProceduralModuleRegistry>,
    catalog: Res<ProceduralModuleSceneCatalog>,
    visual: Res<ProceduralModuleVisualPolicy>,
    iso_draw: Res<ConstructionIsoDrawScale>,
    q: Query<
        (
            Entity,
            &ConstructionSite,
            &ProceduralBuildingSpec,
            &PlannedSite,
            Option<&Transform>,
        ),
        (With<PlannedSite>, Without<ProceduralBuildSpawned>),
    >,
) {
    let _perf = crate::render::PerfScope::new("upd_repr_proc_spawn");
    if !visual.meshes_active {
        return;
    }

    for (site_entity, site, spec, planned, site_transform) in &q {
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }

        if site_transform.is_none() {
            commands.entity(site_entity).insert((
                Transform::from_translation(site_world_position(planned.origin)),
                GlobalTransform::default(),
                crate::economy::site_placement::SiteWorldTransformApplied,
            ));
        }

        let instances = instances_for_operational_site(
            spec,
            &assembly_request,
            &extract,
            &style_packs,
            &registry,
            &catalog,
        );

        let mut module_count = 0u32;
        let scale = iso_draw.visual_scale_vec3();
        commands
            .entity(site_entity)
            .with_children(|parent| {
                for inst in instances.iter().filter(|i| !i.hidden) {
                    module_count += 1;
                    let mut local = Transform::from_translation(inst.local_translation);
                    local.rotation = Quat::from_rotation_y(inst.yaw_y);
                    local.scale = scale;
                    if let Some(scene) = inst.scene.as_ref() {
                        parent.spawn((
                            WorldAssetRoot(scene.clone()),
                            local,
                            crate::render::mig_a_static_bulk_bundle(),
                            ProceduralBuildModuleChild {
                                module_id: inst.module_id.clone(),
                            },
                        ));
                    } else {
                        parent.spawn((
                            local,
                            crate::render::mig_a_static_bulk_bundle(),
                            ProceduralBuildModuleChild {
                                module_id: inst.module_id.clone(),
                            },
                        ));
                    }
                }
            })
            .insert(ProceduralBuildSpawned { module_count });
    }
}

#[cfg(test)]
mod tests {
}
