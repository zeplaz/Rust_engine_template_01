//! PG-2 / PG-3.5 — spawn [`ProceduralBuildExtract`] module scenes at site transform on **Operational**.

use bevy::prelude::*;

use crate::construction::iso_draw_scale::ConstructionIsoDrawScale;
use crate::construction::procedural::{
    footprint_grid_for_assembly, procedural_module_local_translation, FootprintGrid,
    ProceduralAssemblyRequest, ProceduralModuleRegistry, StylePackRegistry,
};
use crate::render::extraction::{
    assemble_procedural_build_instances, ProceduralBuildExtract, ProceduralBuildInstance,
    ProceduralModuleSceneCatalog, ProceduralModuleVisualPolicy,
};
use crate::economy::site_placement::site_world_position;
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
        (Entity, &ConstructionSite, &ProceduralBuildingSpec, &PlannedSite, Option<&Transform>),
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
        commands.entity(site_entity).with_children(|parent| {
            for inst in instances.iter().filter(|i| !i.hidden) {
                module_count += 1;
                let mut local = Transform::from_translation(procedural_module_local_translation(
                    inst.grid_x,
                    inst.grid_y,
                    inst.floor,
                ));
                local.scale = scale;
                if let Some(scene) = inst.scene.as_ref() {
                    parent.spawn((
                        SceneRoot(scene.clone()),
                        local,
                        ProceduralBuildModuleChild {
                            module_id: inst.module_id.clone(),
                        },
                    ));
                } else {
                    parent.spawn((
                        local,
                        ProceduralBuildModuleChild {
                            module_id: inst.module_id.clone(),
                        },
                    ));
                }
            }
        }).insert(ProceduralBuildSpawned { module_count });
    }
}

/// PG-3.5 witness rollup — spawn path + operational gate wired.
#[must_use]
pub fn procedural_pg2_spawn_witness_green() -> bool {
    procedural_pg2_spawn_self_check().is_ok()
}

fn procedural_pg2_spawn_self_check() -> Result<(), &'static str> {
    use crate::construction::procedural::{
        load_procedural_module_registry, load_style_pack_registry, StylePackId,
        ProceduralBuildingRequest,
    };
    use crate::render::extraction::ProceduralModuleSceneCatalog;

    let registry = load_procedural_module_registry();
    if !registry.load_errors.is_empty() {
        return Err("module_registry");
    }
    let packs = load_style_pack_registry();
    if packs.load_errors.is_empty() && packs.get("style_victorian").is_none() {
        return Err("style_packs");
    }
    let pack = packs.get("style_victorian").ok_or("style_victorian")?;
    let request = ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 1,
        arch_dna_preset_id: None,
    };
    let grid = FootprintGrid::from_request(&request);
    let catalog = ProceduralModuleSceneCatalog::default();
    let extract = assemble_procedural_build_instances(&request, pack, &grid, &registry, &catalog);
    let visible = extract
        .instances
        .iter()
        .filter(|i| !i.hidden)
        .count();
    if visible == 0 {
        return Err("no_visible_instances");
    }

    let pos = procedural_module_local_translation(1, 2, 1);
    if pos.y != 3.0 || pos.x != 1.0 || pos.z != 2.0 {
        return Err("local_translation");
    }

    spawn_operational_gate_self_check()?;
    Ok(())
}

fn spawn_operational_gate_self_check() -> Result<(), &'static str> {
    use crate::construction::procedural::{
        init_procedural_module_registry, init_style_pack_registry, load_procedural_module_registry,
        load_style_pack_registry, ProceduralAssemblyRequest,
    };
    use crate::construction::site_stage_tick::SiteStageTickPlugin;
    use crate::render::extraction::assemble_procedural_build_instances;
    use crate::strategic::{
        BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype,
        SiteConstructionBook, SiteConstructionPhase, SiteIdIssuer,
    };
    use crate::systems::sim_control::{SimControlState, SimTick};

    let modules = load_procedural_module_registry();
    if !modules.load_errors.is_empty() {
        return Err("module_registry_load");
    }
    let packs = load_style_pack_registry();
    if packs.load_errors.is_empty() && packs.get("style_victorian").is_none() {
        return Err("style_packs_load");
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SiteConstructionBook>()
        .init_resource::<SiteIdIssuer>()
        .init_resource::<SimControlState>()
        .init_resource::<SimTick>()
        .init_resource::<ProceduralAssemblyRequest>()
        .init_resource::<ProceduralBuildExtract>()
        .init_resource::<ProceduralModuleSceneCatalog>()
        .init_resource::<ProceduralModuleVisualPolicy>()
        .init_resource::<ConstructionIsoDrawScale>()
        .add_message::<CommitConstructionSiteEvent>()
        .add_plugins(SiteStageTickPlugin)
        .add_systems(Startup, (init_procedural_module_registry, init_style_pack_registry))
        .add_systems(
            Update,
            (
                crate::strategic::commit_construction_site_system,
                spawn_procedural_build_on_site_operational,
            )
                .chain(),
        );

    {
        let mut visual = app.world_mut().resource_mut::<ProceduralModuleVisualPolicy>();
        visual.meshes_active = true;
        let mut ctrl = app.world_mut().resource_mut::<SimControlState>();
        ctrl.paused = true;
    }

    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: crate::strategic::SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::CivilHousing,
        origin: BuildSiteTile { x: 10, z: 10 },
        footprint: FootprintTiles {
            width: 4,
            depth: 2,
        },
        layer: LayerType::Surface,
        catalog_id: None,
        placement: None,
    });
    app.update();

    let spec = {
        let world = app.world_mut();
        let phase = world
            .query::<&ConstructionSite>()
            .single(world)
            .map_err(|_| "missing site")?
            .phase;
        if phase != SiteConstructionPhase::Planned {
            return Err("commit_must_leave_planned");
        }
        world
            .query::<&ProceduralBuildingSpec>()
            .single(world)
            .map_err(|_| "missing_spec")?
            .0
            .clone()
    };

    app.insert_resource(ProceduralAssemblyRequest(spec.clone()));
    {
        let world = app.world_mut();
        let registry = world.resource::<ProceduralModuleRegistry>();
        let style_packs = world.resource::<StylePackRegistry>();
        let pack = style_packs
            .get(spec.style.as_str())
            .ok_or("style_pack_for_site")?;
        let grid = FootprintGrid::from_request(&spec);
        let extract = assemble_procedural_build_instances(
            &spec,
            pack,
            &grid,
            registry,
            world.resource::<ProceduralModuleSceneCatalog>(),
        );
        if extract.instances.iter().filter(|i| !i.hidden).count() == 0 {
            return Err("extract_empty");
        }
        world.insert_resource(extract);
    }

    {
        let world = app.world_mut();
        if let Ok(mut site) = world.query::<&mut ConstructionSite>().single_mut(world) {
            site.phase = SiteConstructionPhase::Operational;
        } else {
            return Err("missing site for operational");
        }
    }

    app.update();

    let world = app.world_mut();
    if world
        .query::<&ProceduralBuildSpawned>()
        .single(world)
        .is_err()
    {
        return Err("spawn_marker_missing");
    }
    let count = world
        .query::<&ProceduralBuildSpawned>()
        .single(world)
        .map_err(|_| "spawn_marker")?
        .module_count;
    if count == 0 {
        return Err("spawn_module_count_zero");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn pg2_spawn_witness_green() {
        assert!(super::procedural_pg2_spawn_witness_green());
    }
}
