//! Terrain material registries + chunk materialization (material unification U5 / U7).

use bevy::asset::{AssetEvent, AssetEventSystems};
use bevy::prelude::*;

use crate::terrain::generation::passes::{
    apply_agent_overlay, apply_hydrology_chunk, apply_threshold_tags, classify_cells, fill_fields, materialize,
};
#[cfg(feature = "dev_tools")]
use crate::terrain::generation::passes::materialize_traced;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{
    compute_chunk_dependency,
    hash_pass1_bucket,
    hash_tuning_bucket,
    lowest_dirty_pass,
    ChunkDependency,
    ChunkDirty,
    MaterialRegistry,
    MaterialRegistryLoader,
    MaterializedChunk,
    MaterializedResources,
    RuleSet,
    RuleSetLoader,
    TagRegistry,
    TagRegistryLoader,
    WorldProfile,
    WorldProfileLoader,
    WorldProfileSelector,
    DIRTY_ALL,
    DIRTY_PASS6,
    DIRTY_PASSES_2_THROUGH_6,
};
#[cfg(feature = "dev_tools")]
use crate::terrain::material::RuleTrace;

/// Strong handles for the dev/example terrain registries loaded at startup.
#[derive(Resource, Clone)]
pub struct TerrainRegistriesHandles {
    pub material_registry: Handle<MaterialRegistry>,
    pub tag_registry: Handle<TagRegistry>,
    pub rule_set: Handle<RuleSet>,
}

fn terrain_registries_startup(
    mut commands: Commands,
    mut materials: ResMut<Assets<MaterialRegistry>>,
    mut tags: ResMut<Assets<TagRegistry>>,
    mut rules: ResMut<Assets<RuleSet>>,
) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mat = MaterialRegistry::load_from_json(
        root
            .join("assets/config/terrain/material_registry.example.json")
            .to_str()
            .unwrap(),
    )
    .expect("load example material registry");
    let tag = TagRegistry::load_from_json(
        root
            .join("assets/config/terrain/tag_registry.example.json")
            .to_str()
            .unwrap(),
    )
    .expect("load example tag registry");
    let rule = RuleSet::load_from_ron(
        root
            .join("assets/config/terrain/material_rules.example.ron")
            .to_str()
            .unwrap(),
    )
    .expect("load example material rules");
    let material_registry = materials.add(mat);
    let tag_registry = tags.add(tag);
    let rule_set = rules.add(rule);
    commands.insert_resource(TerrainRegistriesHandles {
        material_registry,
        tag_registry,
        rule_set,
    });
}

fn mark_chunks_dirty_on_asset_change(
    mut mat: MessageReader<AssetEvent<MaterialRegistry>>,
    mut tag: MessageReader<AssetEvent<TagRegistry>>,
    mut rule: MessageReader<AssetEvent<RuleSet>>,
    mut q: Query<&mut ChunkDirty, With<ChunkDependency>>,
) {
    let mut mask = 0u8;
    for e in mat.read() {
        if matches!(e, AssetEvent::Added { .. } | AssetEvent::Modified { .. }) {
            mask |= crate::terrain::material::dependency::DIRTY_PASS6;
        }
    }
    for e in tag.read() {
        if matches!(e, AssetEvent::Added { .. } | AssetEvent::Modified { .. }) {
            mask |= DIRTY_PASSES_2_THROUGH_6;
        }
    }
    for e in rule.read() {
        if matches!(e, AssetEvent::Added { .. } | AssetEvent::Modified { .. }) {
            mask |= DIRTY_PASS6;
        }
    }
    if mask == 0 {
        return;
    }
    for mut d in q.iter_mut() {
        d.passes |= mask;
    }
}

fn mark_chunks_dirty_on_world_gen_params_change(
    params: Res<WorldGenParams>,
    mut q: Query<&mut ChunkDirty, With<ChunkDependency>>,
    mut prev1: Local<Option<u64>>,
    mut prev2: Local<Option<u64>>,
) {
    let h1 = hash_pass1_bucket(&params);
    let h2 = hash_tuning_bucket(&params);
    let mut mask = 0u8;
    if let Some(p1) = *prev1 {
        if p1 != h1 {
            mask |= DIRTY_ALL;
        }
    }
    if let Some(p2) = *prev2 {
        if p2 != h2 {
            mask |= DIRTY_PASSES_2_THROUGH_6;
        }
    }
    *prev1 = Some(h1);
    *prev2 = Some(h2);
    if mask == 0 {
        return;
    }
    for mut d in q.iter_mut() {
        d.passes |= mask;
    }
}

fn run_passes_from(
    low: u32,
    matrix: &mut ChunkCellMatrix,
    chunk_coord: IVec2,
    params: &WorldGenParams,
    tag_reg: &TagRegistry,
) {
    for p in low..5 {
        match p {
            0 => fill_fields(matrix, chunk_coord, params, None),
            1 => apply_threshold_tags(matrix, &params.biome_tuning, tag_reg),
            2 => classify_cells(matrix, &params.biome_tuning, tag_reg),
            3 => apply_hydrology_chunk(matrix, &params.biome_tuning, tag_reg),
            4 => apply_agent_overlay(matrix),
            _ => {}
        }
    }
}

fn rebuild_dirty_chunks(
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tag_assets: Res<Assets<TagRegistry>>,
    rule_assets: Res<Assets<RuleSet>>,
    params: Res<WorldGenParams>,
    mut commands: Commands,
    mut q: Query<
        (
            Entity,
            &Chunk,
            &mut ChunkCellMatrix,
            &mut ChunkDirty,
            &mut ChunkDependency,
        ),
        With<MaterializedChunk>,
    >,
) {
    let Some(reg) = materials.get(&handles.material_registry) else {
        return;
    };
    let Some(tag_reg) = tag_assets.get(&handles.tag_registry) else {
        return;
    };
    let Some(rule_set) = rule_assets.get(&handles.rule_set) else {
        return;
    };

    for (entity, chunk, mut matrix, mut dirty, mut dep) in q.iter_mut() {
        if dirty.passes == 0 {
            continue;
        }
        let Some(low) = lowest_dirty_pass(dirty.passes) else {
            continue;
        };
        run_passes_from(
            low,
            &mut matrix,
            chunk.coord,
            &params,
            tag_reg,
        );
        #[cfg(feature = "dev_tools")]
        {
            let (data, trace) = materialize_traced(&matrix, rule_set, reg, tag_reg);
            commands.entity(entity).insert(MaterializedChunk::from(data.clone()));
            commands
                .entity(entity)
                .insert(MaterializedResources {
                    ids: data.materials.clone(),
                });
            commands.entity(entity).insert(trace);
        }
        #[cfg(not(feature = "dev_tools"))]
        {
            let data = materialize(&matrix, rule_set, reg, tag_reg);
            commands.entity(entity).insert(MaterializedChunk::from(data.clone()));
            commands
                .entity(entity)
                .insert(MaterializedResources {
                    ids: data.materials.clone(),
                });
        }
        *dep = compute_chunk_dependency(chunk.coord, &params, reg, rule_set, tag_reg);
        dirty.passes = 0;
    }
}

pub fn materialize_chunks(
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tag_assets: Res<Assets<TagRegistry>>,
    rule_assets: Res<Assets<RuleSet>>,
    params: Res<WorldGenParams>,
    mut commands: Commands,
    mut q: Query<
        (Entity, &Chunk, &mut ChunkCellMatrix),
        Without<MaterializedChunk>,
    >,
) {
    let Some(reg) = materials.get(&handles.material_registry) else {
        return;
    };
    let Some(tag_reg) = tag_assets.get(&handles.tag_registry) else {
        return;
    };
    let Some(rule_set) = rule_assets.get(&handles.rule_set) else {
        return;
    };

    for (entity, chunk, mut matrix) in q.iter_mut() {
        fill_fields(&mut matrix, chunk.coord, &params, None);
        apply_threshold_tags(&mut matrix, &params.biome_tuning, tag_reg);
        classify_cells(&mut matrix, &params.biome_tuning, tag_reg);
        apply_hydrology_chunk(&mut matrix, &params.biome_tuning, tag_reg);
        apply_agent_overlay(&mut matrix);
        #[cfg(feature = "dev_tools")]
        let (data, trace) = materialize_traced(&matrix, rule_set, reg, tag_reg);
        #[cfg(not(feature = "dev_tools"))]
        let data = materialize(&matrix, rule_set, reg, tag_reg);
        let deps = compute_chunk_dependency(chunk.coord, &params, reg, rule_set, tag_reg);
        let res = MaterializedResources {
            ids: data.materials.clone(),
        };
        let mat_chunk = MaterializedChunk::from(data);
        commands.entity(entity).insert((
            mat_chunk,
            res,
            deps,
            ChunkDirty::default(),
        ));
        #[cfg(feature = "dev_tools")]
        commands.entity(entity).insert(trace);
    }
}

pub struct MaterialUnificationPlugin;

impl Plugin for MaterialUnificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MaterialRegistry>()
            .init_asset::<TagRegistry>()
            .init_asset::<RuleSet>()
            .init_asset::<WorldProfile>()
            .register_asset_loader(MaterialRegistryLoader::default())
            .register_asset_loader(TagRegistryLoader::default())
            .register_asset_loader(RuleSetLoader::default())
            .register_asset_loader(WorldProfileLoader::default())
            .init_resource::<WorldProfileSelector>()
            .add_systems(Startup, terrain_registries_startup)
            .add_systems(Update, materialize_chunks)
            .add_systems(
                PostUpdate,
                (
                    mark_chunks_dirty_on_asset_change,
                    mark_chunks_dirty_on_world_gen_params_change,
                    rebuild_dirty_chunks,
                )
                    .chain()
                    .after(AssetEventSystems),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use crate::terrain::material::hash_material_registry;
    use bevy::asset::AssetPlugin;
    use bevy::prelude::{MinimalPlugins, IVec2, UVec2};

    #[test]
    fn material_plugin_app_boot() {
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default())
            .add_plugins(bevy::asset::AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .add_plugins(MaterialUnificationPlugin);
        app.update();
    }

    #[test]
    fn dirty_marker_set_on_registry_change() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .add_plugins(MaterialUnificationPlugin);

        app.update();

        let e = app
            .world_mut()
            .spawn((
                Chunk {
                    coord: IVec2::ZERO,
                },
                ChunkCellMatrix::new(UVec2::new(2, 2)),
                ChunkDependency {
                    source_noise_id: 0,
                    registry_hash: 0,
                    rules_hash: 0,
                    tags_hash: 0,
                    tuning_hash: 0,
                },
                ChunkDirty::default(),
                MaterializedChunk {
                    size: UVec2::new(2, 2),
                    materials: vec![crate::terrain::material::MaterialId(0); 4],
                },
            ))
            .id();

        let h = app.world().resource::<TerrainRegistriesHandles>().clone();
        let reg = app
            .world()
            .resource::<Assets<MaterialRegistry>>()
            .get(h.material_registry.id())
            .unwrap();
        let expected_reg_hash = hash_material_registry(reg);
        assert_eq!(
            app.world().entity(e).get::<ChunkDependency>().unwrap().registry_hash,
            0
        );

        {
            let mut assets = app.world_mut().resource_mut::<Assets<MaterialRegistry>>();
            let _ = assets.get_mut(h.material_registry.id());
        }

        app.update();

        let dep = app.world().entity(e).get::<ChunkDependency>().unwrap();
        assert_eq!(dep.registry_hash, expected_reg_hash);
        assert_ne!(dep.registry_hash, 0);
    }

    #[test]
    fn partial_rebuild_registry_only_runs_pass6() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .add_plugins(MaterialUnificationPlugin);

        let e = app
            .world_mut()
            .spawn((
                Chunk {
                    coord: IVec2::ZERO,
                },
                ChunkCellMatrix::new(UVec2::new(2, 2)),
            ))
            .id();

        app.update();

        {
            let mut world = app.world_mut();
            let mut ent = world.entity_mut(e);
            ent.get_mut::<ChunkCellMatrix>().unwrap().elevation[0] = 999.0;
            ent.get_mut::<ChunkDirty>().unwrap().passes = DIRTY_PASS6;
        }

        app.update();

        {
            let world = app.world();
            let m = world.entity(e).get::<ChunkCellMatrix>().unwrap();
            assert_eq!(m.elevation[0], 999.0, "pass 1 must not run for registry-only dirty");
        }

        {
            let mut world = app.world_mut();
            world
                .entity_mut(e)
                .get_mut::<ChunkDirty>()
                .unwrap()
                .passes = DIRTY_ALL;
        }

        app.update();

        {
            let world = app.world();
            let m = world.entity(e).get::<ChunkCellMatrix>().unwrap();
            assert_ne!(
                m.elevation[0], 999.0,
                "full dirty mask must re-run pass 1 and reset fields"
            );
        }
    }
}
