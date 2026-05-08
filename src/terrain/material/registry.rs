use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};

/// Runtime material variant id (dense index into a loaded registry).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaterialId(pub u16);

/// One row from `material_registry.json` after load (tag names preserved; intern against [`super::TagRegistry`] separately).
///
/// **`family`:** resolved [`TerrainFamilyId`] from the string in JSON via [`TerrainFamilyRegistry`].
///
/// **`properties`:** opaque JSON. Example registries with **`schema_version` ≥ 2** use **dot-separated namespaces**
/// (`facts.*`, `sim.*`, `render.*`, `gen.*`, `mobility.*`, `build.*`, `warfare.*`) — see `material_tag_rule_system_v1.md` §4.1 under `prompts/designer_questions/terrain_world/`.
#[derive(Clone, Debug)]
pub struct MaterialDef {
    pub name: String,
    pub family: TerrainFamilyId,
    pub tags: Vec<String>,
    pub properties: serde_json::Value,
    pub preview_color: [u8; 4],
}

impl MaterialDef {
    #[inline]
    fn dotted(ns: &str, key: &str) -> String {
        format!("{ns}.{key}")
    }

    pub fn sim_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("sim", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    pub fn build_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("build", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    pub fn warfare_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("warfare", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    pub fn facts_str(&self, key: &str) -> Option<String> {
        self.properties
            .get(&Self::dotted("facts", key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn facts_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("facts", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct MaterialDefFile {
    name: String,
    family: String,
    tags: Vec<String>,
    properties: serde_json::Value,
    preview_color: [u8; 4],
}

#[derive(Clone, Debug, Deserialize)]
struct MaterialRegistryFile {
    pub schema_version: u32,
    pub materials: Vec<MaterialDefFile>,
}

/// Loaded material table + deterministic name → id map (input row order defines ids).
#[derive(Asset, TypePath, Clone, Debug)]
pub struct MaterialRegistry {
    pub schema_version: u32,
    pub materials: Vec<MaterialDef>,
    pub name_to_id: HashMap<String, MaterialId>,
}

/// First material definition matching `family` (registry row order — same as `resolve_material` fallback).
pub fn family_default_material_def<'a>(
    registry: &'a MaterialRegistry,
    family: TerrainFamilyId,
) -> Option<&'a MaterialDef> {
    registry.materials.iter().find(|m| m.family == family)
}

/// Schema versions accepted when parsing registry JSON. Bump when breaking; document migration.
pub const SUPPORTED_MATERIAL_REGISTRY_SCHEMA_VERSIONS: &[u32] = &[1, 2];

fn default_family_registry_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/config/terrain/terrain_family_registry.example.json")
}

impl MaterialRegistry {
    /// Load material JSON and resolve `family` strings using the default example terrain family registry path.
    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        let families = TerrainFamilyRegistry::load_from_json(
            default_family_registry_path().to_str().unwrap(),
        )?;
        let s = std::fs::read_to_string(path)?;
        let file: MaterialRegistryFile = serde_json::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON: {e}"))
        })?;
        Self::from_file(file, &families).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Resolve families using `terrain_family_registry.json` beside `material_registry.json` when present, else default example path.
    pub fn load_from_json_with_adjacent_families(material_path: &str) -> std::io::Result<Self> {
        let mut fam_path = Path::new(material_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("terrain_family_registry.example.json");
        if !fam_path.exists() {
            fam_path = default_family_registry_path();
        }
        let families =
            TerrainFamilyRegistry::load_from_json(fam_path.to_str().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "family path utf-8")
            })?)?;
        let s = std::fs::read_to_string(material_path)?;
        let file: MaterialRegistryFile = serde_json::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON: {e}"))
        })?;
        Self::from_file(file, &families).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn from_file(file: MaterialRegistryFile, families: &TerrainFamilyRegistry) -> Result<Self, String> {
        if !SUPPORTED_MATERIAL_REGISTRY_SCHEMA_VERSIONS.contains(&file.schema_version) {
            return Err(format!(
                "Material registry schema_version={} unsupported. Expected one of {:?}. Update assets or run migration.",
                file.schema_version, SUPPORTED_MATERIAL_REGISTRY_SCHEMA_VERSIONS
            ));
        }
        let materials: Vec<MaterialDef> = file
            .materials
            .into_iter()
            .map(|m| {
                let family = families.require_id(&m.family).map_err(|e| {
                    format!("material {:?}: unknown family {:?}: {}", m.name, m.family, e)
                })?;
                Ok(MaterialDef {
                    name: m.name,
                    family,
                    tags: m.tags,
                    properties: m.properties,
                    preview_color: m.preview_color,
                })
            })
            .collect::<Result<_, String>>()?;

        let mut name_to_id = HashMap::new();
        for (i, m) in materials.iter().enumerate() {
            let id = MaterialId(i as u16);
            name_to_id.insert(m.name.clone(), id);
        }
        Ok(Self {
            schema_version: file.schema_version,
            materials,
            name_to_id,
        })
    }
}

/// Bevy loader for `*.material_registry.json`.
#[derive(Default, TypePath)]
pub struct MaterialRegistryLoader;

#[derive(Debug)]
pub enum MaterialRegistryLoaderError {
    Io(std::io::Error),
    Json(String),
}

impl fmt::Display for MaterialRegistryLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for MaterialRegistryLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(_) => None,
        }
    }
}

impl From<std::io::Error> for MaterialRegistryLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for MaterialRegistryLoaderError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

impl AssetLoader for MaterialRegistryLoader {
    type Asset = MaterialRegistry;
    type Settings = ();
    type Error = MaterialRegistryLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let file: MaterialRegistryFile = serde_json::from_slice(&bytes)?;

        let families = TerrainFamilyRegistry::load_from_json(
            default_family_registry_path().to_str().ok_or_else(|| {
                MaterialRegistryLoaderError::Json("default family registry path not utf-8".into())
            })?,
        )?;

        MaterialRegistry::from_file(file, &families).map_err(MaterialRegistryLoaderError::Json)
    }

    fn extensions(&self) -> &[&str] {
        &["material_registry.json"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_material_registry_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_registry.example.json")
    }

    fn families() -> TerrainFamilyRegistry {
        TerrainFamilyRegistry::load_from_json(
            default_family_registry_path().to_str().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn material_registry_loads_example() {
        let path = example_material_registry_path();
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        assert_eq!(reg.schema_version, 2);
        assert!(reg.materials.len() >= 4);
        assert_eq!(reg.name_to_id.get("loam_wet"), Some(&MaterialId(2)));
    }

    #[test]
    fn sim_f32_reads_namespaced_key() {
        let path = example_material_registry_path();
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        let loam = &reg.materials[reg.name_to_id["loam_wet"].0 as usize];
        assert!((loam.sim_f32("traction_mod").unwrap() - 0.8).abs() < 1e-4);
        assert!((loam.sim_f32("water_retention").unwrap() - 0.9).abs() < 1e-4);
    }

    #[test]
    fn material_registry_rejects_unknown_schema_version() {
        let raw = r#"{"schema_version":99,"materials":[]}"#;
        let file: MaterialRegistryFile = serde_json::from_str(raw).unwrap();
        let fam = families();
        assert!(MaterialRegistry::from_file(file, &fam).is_err());
    }

    #[test]
    fn material_registry_loader_extension() {
        let loader = MaterialRegistryLoader::default();
        assert!(
            loader.extensions().contains(&"material_registry.json"),
            "extensions: {:?}",
            loader.extensions()
        );
    }
}
