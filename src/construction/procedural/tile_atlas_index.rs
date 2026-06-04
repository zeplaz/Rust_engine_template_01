//! MCP tile atlas catalog — `assets/configs/buildings/_tile_atlas_index.ron` (atlas_id → png + variant UVs).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use super::module_index::DevelopmentTier;

pub const TILE_ATLAS_INDEX_RON: &str = "assets/configs/buildings/_tile_atlas_index.ron";
pub const TILE_ATLAS_INDEX_JSON: &str = "assets/configs/buildings/_tile_atlas_index.json";
pub const TILE_ATLAS_INDEX_ARCHIVE_RON: &str =
    "assets/configs/buildings/_tile_atlas_index_archive.ron";

#[derive(Debug, Clone, Deserialize)]
struct TileAtlasIndexFile {
    #[serde(default)]
    schema_version: u32,
    entries: Vec<TileAtlasIndexEntryRaw>,
}

#[derive(Debug, Clone, Deserialize)]
struct TileAtlasIndexEntryRaw {
    atlas_id: String,
    batch_id: String,
    #[serde(default)]
    assembly_id: String,
    tile_id: String,
    atlas_png: String,
    meta_json: String,
    #[serde(default)]
    development_tier: String,
    #[serde(default)]
    style_pack_id: String,
    #[serde(default = "default_ship_allowed")]
    ship_allowed: bool,
}

fn default_ship_allowed() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
struct AtlasMetaFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    batch_id: String,
    #[serde(default)]
    tile_id: String,
    #[serde(default)]
    atlas_id: String,
    #[serde(default)]
    tiles: Vec<AtlasMetaTile>,
    #[serde(default)]
    render_contract: Option<AtlasMetaRenderContract>,
    #[serde(default)]
    lookups: Vec<AtlasMetaLookupRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct AtlasMetaRenderContract {
    #[serde(default)]
    facings: u8,
    #[serde(default)]
    quarter_turn_fallback: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct AtlasMetaLookupRow {
    variant: String,
    #[serde(default)]
    facing: u8,
    #[serde(default)]
    frame: u8,
    #[serde(default)]
    uv: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct AtlasMetaTile {
    variant_key: String,
    #[serde(default)]
    uv: Vec<f32>,
}

/// One promoted tile atlas row from the MCP tile batch pipeline.
#[derive(Debug, Clone)]
pub struct TileAtlasEntry {
    pub atlas_id: String,
    pub batch_id: String,
    pub assembly_id: String,
    pub tile_id: String,
    /// Repo-relative path (`assets/textures/tiles/...`).
    pub atlas_png: String,
    /// Bevy `AssetServer` path (no `assets/` prefix).
    pub atlas_asset: String,
    pub meta_json: String,
    pub development_tier: DevelopmentTier,
    pub style_pack_id: String,
    /// Explicit ship flag; defaults from tier when omitted in index file.
    pub ship_allowed: bool,
    /// v1 fallback: variant_key → UV (facing 0 / frame 0).
    pub variants: HashMap<String, [f32; 4]>,
    pub meta_schema_version: u32,
    pub render_facings: u8,
    pub quarter_turn_fallback: bool,
    /// v2: (variant, facing, frame) → UV.
    pub lookups: HashMap<(String, u8, u8), [f32; 4]>,
}

impl TileAtlasEntry {
    #[must_use]
    pub fn runtime_stamp_allowed(&self) -> bool {
        self.ship_allowed && self.development_tier.atlas_runtime_stamp_allowed()
    }

    #[must_use]
    pub fn lookup_uv(&self, variant: &str, facing: u8, frame: u8) -> Option<[f32; 4]> {
        self.lookups
            .get(&(variant.to_owned(), facing, frame))
            .copied()
    }
}

#[derive(Resource, Debug, Default)]
pub struct TileAtlasRegistry {
    pub entries: Vec<TileAtlasEntry>,
    pub by_atlas_id: HashMap<String, TileAtlasEntry>,
    pub by_batch_id: HashMap<String, String>,
    pub by_assembly_id: HashMap<String, String>,
    pub by_tile_id: HashMap<String, String>,
    pub load_errors: Vec<String>,
}

impl TileAtlasRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn get(&self, atlas_id: &str) -> Option<&TileAtlasEntry> {
        self.by_atlas_id.get(atlas_id)
    }

    #[must_use]
    pub fn atlas_id_for_batch(&self, batch_id: &str) -> Option<&str> {
        self.by_batch_id.get(batch_id).map(|s| s.as_str())
    }

    #[must_use]
    pub fn atlas_asset(&self, atlas_id: &str) -> Option<&str> {
        self.get(atlas_id).map(|e| e.atlas_asset.as_str())
    }

    #[must_use]
    pub fn atlas_for_assembly(&self, assembly_id: &str) -> Option<&TileAtlasEntry> {
        self.by_assembly_id
            .get(assembly_id)
            .and_then(|id| self.by_atlas_id.get(id))
            .filter(|e| e.runtime_stamp_allowed())
    }

    #[must_use]
    pub fn atlas_for_tile_id(&self, tile_id: &str) -> Option<&TileAtlasEntry> {
        self.by_tile_id
            .get(tile_id)
            .and_then(|id| self.by_atlas_id.get(id))
            .filter(|e| e.runtime_stamp_allowed())
    }

    /// Resolved atlas asset + normalized UV rect for a map/tactical material swap (v1 / facing0).
    #[must_use]
    pub fn resolve_variant_uv(&self, atlas_id: &str, variant_key: &str) -> Option<[f32; 4]> {
        self.get(atlas_id).and_then(|e| {
            e.lookup_uv(variant_key, 0, 0)
                .or_else(|| e.variants.get(variant_key).copied())
        })
    }

    /// Resolve by assembly id + variant key (preferred for PG-2 assembly snapshots).
    #[must_use]
    pub fn resolve_assembly_variant_uv(
        &self,
        assembly_id: &str,
        variant_key: &str,
    ) -> Option<([f32; 4], &TileAtlasEntry)> {
        let entry = self.atlas_for_assembly(assembly_id)?;
        let uv = entry.variants.get(variant_key).copied()?;
        Some((uv, entry))
    }
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

#[must_use]
pub fn default_tile_atlas_index_ron_path() -> PathBuf {
    repo_asset_path(TILE_ATLAS_INDEX_RON)
}

fn repo_relative_path(raw: &str) -> String {
    let trimmed = raw.trim_start_matches('\\').trim_start_matches('/');
    let stripped = trimmed
        .strip_prefix("assets/")
        .or_else(|| trimmed.strip_prefix("assets\\"))
        .unwrap_or(trimmed);
    stripped.replace('\\', "/")
}

fn atlas_png_to_asset_path(atlas_png: &str) -> String {
    repo_relative_path(atlas_png)
}

fn uv_from_meta_tile(tile: &AtlasMetaTile) -> Option<[f32; 4]> {
    if tile.uv.len() >= 4 {
        return Some([tile.uv[0], tile.uv[1], tile.uv[2], tile.uv[3]]);
    }
    None
}

struct MetaLoadResult {
    variants: HashMap<String, [f32; 4]>,
    schema_version: u32,
    render_facings: u8,
    quarter_turn_fallback: bool,
    lookups: HashMap<(String, u8, u8), [f32; 4]>,
}

fn load_meta(meta_path: &Path) -> Result<MetaLoadResult, String> {
    let text = std::fs::read_to_string(meta_path)
        .map_err(|e| format!("meta_json read {}: {e}", meta_path.display()))?;
    let meta: AtlasMetaFile = serde_json::from_str(&text)
        .map_err(|e| format!("meta_json parse {}: {e}", meta_path.display()))?;
    let schema_version = meta.schema_version;
    let render = meta.render_contract.as_ref();
    let render_facings = render.map(|r| r.facings).unwrap_or(0);
    let quarter_turn_fallback = render.map(|r| r.quarter_turn_fallback).unwrap_or(true);
    let mut variants = HashMap::new();
    let mut lookups = HashMap::new();
    if schema_version >= 2 {
        for row in meta.lookups {
            if row.uv.len() >= 4 {
                let uv = [row.uv[0], row.uv[1], row.uv[2], row.uv[3]];
                lookups.insert((row.variant.clone(), row.facing, row.frame), uv);
                if row.facing == 0 && row.frame == 0 {
                    variants.insert(row.variant, uv);
                }
            }
        }
    } else {
        for tile in meta.tiles {
            if let Some(uv) = uv_from_meta_tile(&tile) {
                let key = tile.variant_key.clone();
                variants.insert(key.clone(), uv);
                lookups.insert((key, 0, 0), uv);
            }
        }
        if render_facings == 0 {
            return Ok(MetaLoadResult {
                variants,
                schema_version,
                render_facings: 1,
                quarter_turn_fallback: true,
                lookups,
            });
        }
    }
    let facings = if render_facings == 0 { 8 } else { render_facings };
    Ok(MetaLoadResult {
        variants,
        schema_version,
        render_facings: facings,
        quarter_turn_fallback,
        lookups,
    })
}

fn normalize_entry(raw: TileAtlasIndexEntryRaw) -> Result<TileAtlasEntry, String> {
    let atlas_png = raw.atlas_png.replace('\\', "/");
    let meta_json = raw.meta_json.replace('\\', "/");
    let meta_path = repo_asset_path(&meta_json);
    let meta = load_meta(&meta_path)?;
    let tier = DevelopmentTier::parse(&raw.development_tier, &raw.batch_id);
    let ship_allowed = raw.ship_allowed && tier.atlas_runtime_stamp_allowed();
    Ok(TileAtlasEntry {
        atlas_id: raw.atlas_id.clone(),
        batch_id: raw.batch_id.clone(),
        assembly_id: raw.assembly_id,
        tile_id: raw.tile_id,
        atlas_asset: atlas_png_to_asset_path(&atlas_png),
        atlas_png,
        meta_json,
        development_tier: tier,
        style_pack_id: raw.style_pack_id,
        ship_allowed,
        variants: meta.variants,
        meta_schema_version: meta.schema_version,
        render_facings: meta.render_facings,
        quarter_turn_fallback: meta.quarter_turn_fallback,
        lookups: meta.lookups,
    })
}

fn ingest_entry(registry: &mut TileAtlasRegistry, entry: TileAtlasEntry, active_runtime_only: bool) {
    if active_runtime_only && !entry.runtime_stamp_allowed() {
        return;
    }
    let atlas_id = entry.atlas_id.clone();
    let batch_id = entry.batch_id.clone();
    if !entry.assembly_id.is_empty() {
        let replace = match registry.by_assembly_id.get(&entry.assembly_id) {
            None => true,
            Some(existing_id) => registry
                .by_atlas_id
                .get(existing_id)
                .is_none_or(|existing| entry.development_tier > existing.development_tier),
        };
        if replace {
            registry
                .by_assembly_id
                .insert(entry.assembly_id.clone(), atlas_id.clone());
        }
    }
    if !entry.tile_id.is_empty() {
        registry
            .by_tile_id
            .insert(entry.tile_id.clone(), atlas_id.clone());
    }
    registry.by_batch_id.insert(batch_id, atlas_id.clone());
    registry.by_atlas_id.insert(atlas_id, entry.clone());
    registry.entries.push(entry);
}

#[must_use]
pub fn load_tile_atlas_registry_from_path(path: &Path) -> TileAtlasRegistry {
    let mut registry = TileAtlasRegistry::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        registry
            .load_errors
            .push(format!("tile atlas index not found: {}", path.display()));
        return registry;
    };

    let parse_result = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str::<TileAtlasIndexFile>(&text)
            .map_err(|e| format!("JSON parse: {e}"))
    } else {
        ron::from_str::<TileAtlasIndexFile>(&text)
            .map_err(|e| format!("RON parse: {e}"))
    };

    match parse_result {
        Ok(file) => {
            for raw in file.entries {
                match normalize_entry(raw) {
                    Ok(entry) => ingest_entry(&mut registry, entry, true),
                    Err(e) => registry.load_errors.push(e),
                }
            }
        }
        Err(e) => registry.load_errors.push(e),
    }
    registry
}

#[must_use]
pub fn load_tile_atlas_registry() -> TileAtlasRegistry {
    let ron_path = default_tile_atlas_index_ron_path();
    if ron_path.is_file() {
        return load_tile_atlas_registry_from_path(&ron_path);
    }
    let json_path = repo_asset_path(TILE_ATLAS_INDEX_JSON);
    if json_path.is_file() {
        return load_tile_atlas_registry_from_path(&json_path);
    }
    let mut registry = TileAtlasRegistry::default();
    registry
        .load_errors
        .push("missing _tile_atlas_index.ron and _tile_atlas_index.json".into());
    registry
}

pub fn init_tile_atlas_registry(mut commands: Commands) {
    let registry = load_tile_atlas_registry();
    if !registry.load_errors.is_empty() {
        for err in &registry.load_errors {
            warn!(target: "tile_atlas", "{err}");
        }
    } else if !registry.is_empty() {
        info!(
            target: "tile_atlas",
            "TileAtlasRegistry: {} atlas(es)",
            registry.len()
        );
    }
    commands.insert_resource(registry);
}

#[must_use]
pub fn load_tile_atlas_archive_registry() -> TileAtlasRegistry {
    load_tile_atlas_archive_registry_from_path(&repo_asset_path(TILE_ATLAS_INDEX_ARCHIVE_RON))
}

#[must_use]
pub fn load_tile_atlas_archive_registry_from_path(path: &Path) -> TileAtlasRegistry {
    let mut registry = TileAtlasRegistry::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        registry
            .load_errors
            .push(format!("tile atlas archive not found: {}", path.display()));
        return registry;
    };
    let parse_result = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        serde_json::from_str::<TileAtlasIndexFile>(&text).map_err(|e| format!("JSON parse: {e}"))
    } else {
        ron::from_str::<TileAtlasIndexFile>(&text).map_err(|e| format!("RON parse: {e}"))
    };
    match parse_result {
        Ok(file) => {
            for raw in file.entries {
                match normalize_entry(raw) {
                    Ok(entry) => ingest_entry(&mut registry, entry, false),
                    Err(e) => registry.load_errors.push(e),
                }
            }
        }
        Err(e) => registry.load_errors.push(e),
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_atlas_active_index_empty_after_greybox_freeze() {
        let reg = load_tile_atlas_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        assert!(
            reg.is_empty(),
            "TILE-FIX-001: active index empty until atlas v2 ships"
        );
    }

    #[test]
    fn tile_atlas_archive_loads_pilot_rowhouse() {
        let reg = load_tile_atlas_archive_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        let entry = reg
            .get("rowhouse_victorian_pilot_v1")
            .expect("archived pilot");
        assert_eq!(entry.batch_id, "tile_rowhouse_victorian_pilot_v1");
        assert!(!entry.runtime_stamp_allowed());
        assert!(
            entry
                .atlas_png
                .contains("archive/lod0_tile_pilots_2026-06")
        );
        assert_eq!(entry.variants.len(), 2);
    }

    #[test]
    fn tile_atlas_archive_resolve_variant_uv() {
        let reg = load_tile_atlas_archive_registry();
        let uv = reg
            .resolve_variant_uv("rowhouse_victorian_pilot_v1", "clean_day")
            .expect("clean_day uv");
        assert_eq!(uv, [0.0, 0.0, 0.5, 1.0]);
    }

    #[test]
    fn tile_atlas_registry_empty_when_index_missing() {
        let reg = load_tile_atlas_registry_from_path(Path::new(
            "assets/configs/buildings/_tile_atlas_index_missing_test.ron",
        ));
        assert!(!reg.load_errors.is_empty());
        assert!(reg.is_empty());
    }

    #[test]
    fn tile_atlas_for_assembly_none_until_v2_promoted() {
        let reg = load_tile_atlas_registry();
        assert!(
            reg.atlas_for_assembly("victorian_4x3_s42_a7cb").is_none(),
            "greybox production v1 de-indexed"
        );
    }
}
