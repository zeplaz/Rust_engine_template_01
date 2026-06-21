//! LG-5 landscape iso atlas registry — `assets/configs/landscape/_landscape_atlas_index.*`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use super::landscape_grammar_map::{LANDSCAPE_ATLAS_INDEX_RON, LG5_ATLAS_ID};

pub const LANDSCAPE_ATLAS_INDEX_JSON: &str = "assets/configs/landscape/_landscape_atlas_index.json";

#[derive(Debug, Clone, Deserialize)]
struct LandscapeAtlasIndexFile {
    #[serde(default)]
    schema_version: u32,
    entries: Vec<LandscapeAtlasIndexEntryRaw>,
}

#[derive(Debug, Clone, Deserialize)]
struct LandscapeAtlasIndexEntryRaw {
    atlas_id: String,
    batch_id: String,
    tile_id: String,
    atlas_png: String,
    meta_json: String,
    #[serde(default)]
    development_tier: String,
    #[serde(default)]
    style_pack_id: String,
    #[serde(default)]
    ship_allowed: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct AtlasMetaFile {
    #[serde(default)]
    tiles: Vec<AtlasMetaTile>,
}

#[derive(Debug, Clone, Deserialize)]
struct AtlasMetaTile {
    variant_key: String,
    #[serde(default)]
    uv: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct LandscapeAtlasEntry {
    pub atlas_id: String,
    pub batch_id: String,
    pub tile_id: String,
    pub atlas_png: String,
    pub atlas_asset: String,
    pub meta_json: String,
    pub development_tier: String,
    pub style_pack_id: String,
    pub ship_allowed: bool,
    pub variants: HashMap<String, [f32; 4]>,
}

impl LandscapeAtlasEntry {
    /// VEG-F03-REGISTRY-STAMP-001 — UV meta on disk; PNG optional when dry-run bake meta is authoritative.
    #[must_use]
    pub fn chunk_stamp_allowed(&self) -> bool {
        !self.variants.is_empty()
            && (repo_asset_path(&self.atlas_png).is_file() || self.variants.len() >= 3)
    }

    #[must_use]
    pub fn resolve_variant_uv(&self, variant_key: &str) -> Option<[f32; 4]> {
        self.variants.get(variant_key).copied()
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LandscapeAtlasRegistry {
    pub schema_version: u32,
    pub entries: Vec<LandscapeAtlasEntry>,
    pub by_atlas_id: HashMap<String, LandscapeAtlasEntry>,
    pub load_errors: Vec<String>,
}

impl LandscapeAtlasRegistry {
    #[must_use]
    pub fn get(&self, atlas_id: &str) -> Option<&LandscapeAtlasEntry> {
        self.by_atlas_id.get(atlas_id)
    }

    #[must_use]
    pub fn lg5_entry(&self) -> Option<&LandscapeAtlasEntry> {
        self.get(LG5_ATLAS_ID).filter(|e| e.chunk_stamp_allowed())
    }
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

fn atlas_png_to_asset_path(atlas_png: &str) -> String {
    let trimmed = atlas_png.trim_start_matches('\\').trim_start_matches('/');
    trimmed
        .strip_prefix("assets/")
        .or_else(|| trimmed.strip_prefix("assets\\"))
        .unwrap_or(trimmed)
        .replace('\\', "/")
}

fn load_meta_variants(meta_path: &Path) -> Result<HashMap<String, [f32; 4]>, String> {
    let text = std::fs::read_to_string(meta_path)
        .map_err(|e| format!("meta_json read {}: {e}", meta_path.display()))?;
    let meta: AtlasMetaFile = serde_json::from_str(&text)
        .map_err(|e| format!("meta_json parse {}: {e}", meta_path.display()))?;
    let mut variants = HashMap::new();
    for tile in meta.tiles {
        if tile.uv.len() >= 4 {
            variants.insert(
                tile.variant_key,
                [tile.uv[0], tile.uv[1], tile.uv[2], tile.uv[3]],
            );
        }
    }
    Ok(variants)
}

fn normalize_entry(raw: LandscapeAtlasIndexEntryRaw) -> Result<LandscapeAtlasEntry, String> {
    let atlas_png = raw.atlas_png.replace('\\', "/");
    let meta_json = raw.meta_json.replace('\\', "/");
    let variants = load_meta_variants(&repo_asset_path(&meta_json))?;
    Ok(LandscapeAtlasEntry {
        atlas_id: raw.atlas_id,
        batch_id: raw.batch_id,
        tile_id: raw.tile_id,
        atlas_asset: atlas_png_to_asset_path(&atlas_png),
        atlas_png,
        meta_json,
        development_tier: raw.development_tier,
        style_pack_id: raw.style_pack_id,
        ship_allowed: raw.ship_allowed,
        variants,
    })
}

#[must_use]
pub fn load_landscape_atlas_registry_from_path(path: &Path) -> LandscapeAtlasRegistry {
    let mut registry = LandscapeAtlasRegistry::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        registry
            .load_errors
            .push(format!("landscape atlas index missing: {}", path.display()));
        return registry;
    };
    let file: LandscapeAtlasIndexFile = if path.extension().is_some_and(|e| e == "json") {
        match serde_json::from_str(&text) {
            Ok(f) => f,
            Err(_) => {
                registry
                    .load_errors
                    .push(format!("landscape atlas index parse: {}", path.display()));
                return registry;
            }
        }
    } else {
        match ron::from_str(&text) {
            Ok(f) => f,
            Err(_) => {
                registry
                    .load_errors
                    .push(format!("landscape atlas index parse: {}", path.display()));
                return registry;
            }
        }
    };
    registry.schema_version = file.schema_version;
    for raw in file.entries {
        match normalize_entry(raw) {
            Ok(entry) => {
                registry
                    .by_atlas_id
                    .insert(entry.atlas_id.clone(), entry.clone());
                registry.entries.push(entry);
            }
            Err(err) => registry.load_errors.push(err),
        }
    }
    registry
}

#[must_use]
pub fn load_landscape_atlas_registry() -> LandscapeAtlasRegistry {
    let json_path = repo_asset_path(LANDSCAPE_ATLAS_INDEX_JSON);
    if json_path.is_file() {
        return load_landscape_atlas_registry_from_path(&json_path);
    }
    load_landscape_atlas_registry_from_path(&repo_asset_path(LANDSCAPE_ATLAS_INDEX_RON))
}

/// Map LG-4 topology kind label → LG-5 atlas variant_key.
#[must_use]
pub fn topology_kind_to_variant_key(kind: &str) -> Option<&'static str> {
    match kind {
        "Patch" => Some("topology_patch"),
        "Corridor" => Some("topology_corridor"),
        "Ring" => Some("topology_ring"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_lg5_registry_loads_three_variants() {
        let reg = load_landscape_atlas_registry();
        if !repo_asset_path(LANDSCAPE_ATLAS_INDEX_JSON).is_file() {
            return;
        }
        let Some(entry) = reg.lg5_entry() else {
            panic!("lg5 entry missing: {:?}", reg.load_errors);
        };
        assert!(entry.chunk_stamp_allowed());
        for key in ["topology_patch", "topology_corridor", "topology_ring"] {
            assert!(
                entry.resolve_variant_uv(key).is_some(),
                "missing {key}"
            );
        }
    }
}
