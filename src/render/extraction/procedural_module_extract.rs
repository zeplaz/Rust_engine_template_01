//! Procedural module GLB handles — read [`ProceduralModuleRegistry`] + [`RepresentationResult`].

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;

use crate::construction::procedural::{ProceduralModuleRegistry, StylePackRegistry};
use crate::gui::RepresentationResult;

/// `AssetServer` scene handles keyed by `module_id` (from `_module_index.ron`).
#[derive(Resource, Debug, Default)]
pub struct ProceduralModuleSceneCatalog {
    pub scenes: HashMap<String, Handle<WorldAsset>>,
    pub load_started: bool,
}

/// Policy mirror: when false, consumers must not spawn procedural module meshes.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ProceduralModuleVisualPolicy {
    pub meshes_active: bool,
}

pub fn load_procedural_module_scenes(
    mut catalog: ResMut<ProceduralModuleSceneCatalog>,
    registry: Res<ProceduralModuleRegistry>,
    style_packs: Option<Res<StylePackRegistry>>,
    asset_server: Res<AssetServer>,
) {
    if catalog.load_started {
        return;
    }
    if registry.by_module_id.is_empty() {
        catalog.load_started = true;
        return;
    }

    let module_ids: HashSet<String> = if let Some(packs) = style_packs.as_ref() {
        if packs.packs.is_empty() {
            registry
                .modules_for_stylepack()
                .map(|e| e.module_id.clone())
                .collect()
        } else {
            packs
                .referenced_module_ids()
                .map(str::to_owned)
                .collect()
        }
    } else {
        registry
            .modules_for_stylepack()
            .map(|e| e.module_id.clone())
            .collect()
    };

    for module_id in module_ids {
        let Some(entry) = registry.resolve_module_id(&module_id) else {
            continue;
        };
        let label = format!("{}#Scene0", entry.glb_asset);
        catalog
            .scenes
            .insert(entry.module_id.clone(), asset_server.load(label));
    }
    catalog.load_started = true;
    info!(
        target: "procedural_module",
        "ProceduralModuleSceneCatalog: {} scene handles (style-pack scoped)",
        catalog.scenes.len()
    );
}

pub fn sync_procedural_module_visual_policy(
    policy: Res<RepresentationResult>,
    mut visual: ResMut<ProceduralModuleVisualPolicy>,
) {
    visual.meshes_active = policy.procedural_module_meshes;
}

#[must_use]
pub fn scene_for_module<'a>(
    catalog: &'a ProceduralModuleSceneCatalog,
    registry: &ProceduralModuleRegistry,
    module_id: &str,
) -> Option<&'a Handle<WorldAsset>> {
    let canonical = registry.resolve_canonical_module_id(module_id);
    if let Some(handle) = catalog.scenes.get(canonical) {
        return Some(handle);
    }
    registry
        .resolve_module_id(module_id)
        .and_then(|entry| catalog.scenes.get(&entry.module_id))
}
