use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MaterialDefFile {
    name: String,
    family: String,
    tags: Vec<String>,
    properties: serde_json::Value,
    preview_color: [u8; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

fn default_family_registry_disk_path() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain/");
    let ron = dir.join("terrain_family_registry.example.ron");
    let json = dir.join("terrain_family_registry.example.json");
    if ron.exists() {
        ron
    } else {
        json
    }
}

impl MaterialRegistry {
    /// Load material registry from **`*.ron`** or **`*.json`**; resolves families via default example registry (`.ron` preferred when present).
    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        let families = TerrainFamilyRegistry::load_from_path(
            default_family_registry_disk_path()
                .to_str()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "family registry path not utf-8")
                })?,
        )?;
        let file: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(Path::new(path))?;
        Self::from_file(file, &families).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Resolve families using `terrain_family_registry.example.{ron,json}` beside `material_path` when present, else default disk path (RON preferred).
    pub fn load_from_json_with_adjacent_families(material_path: &str) -> std::io::Result<Self> {
        let parent = Path::new(material_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let ron = parent.join("terrain_family_registry.example.ron");
        let json = parent.join("terrain_family_registry.example.json");
        let families_path = if ron.exists() {
            ron
        } else if json.exists() {
            json
        } else {
            default_family_registry_disk_path()
        };
        let families =
            TerrainFamilyRegistry::load_from_path(families_path.to_str().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "family path utf-8")
            })?)?;
        let file: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(Path::new(material_path))?;
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
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes).map_err(|e| {
            MaterialRegistryLoaderError::Json(format!("UTF-8: {e}"))
        })?;
        let ext = load_context.path().get_full_extension();
        let file: MaterialRegistryFile =
            crate::terrain::registry_serde_path::deserialize_from_str_with_extension_opt(text, ext.as_deref())
                .map_err(|e| MaterialRegistryLoaderError::Json(e.to_string()))?;

        let families = TerrainFamilyRegistry::load_from_path(
            default_family_registry_disk_path().to_str().ok_or_else(|| {
                MaterialRegistryLoaderError::Json("default family registry path not utf-8".into())
            })?,
        )?;

        MaterialRegistry::from_file(file, &families).map_err(MaterialRegistryLoaderError::Json)
    }

    fn extensions(&self) -> &[&str] {
        &["material_registry.json", "material_registry.ron"]
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
        TerrainFamilyRegistry::load_from_path(
            default_family_registry_disk_path().to_str().unwrap(),
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
        assert!(loader.extensions().contains(&"material_registry.json"));
        assert!(loader.extensions().contains(&"material_registry.ron"));
    }

    #[test]
    fn material_example_json_round_trips_ron_wire_format() {
        let path = example_material_registry_path();
        let s = std::fs::read_to_string(&path).unwrap();
        let file: MaterialRegistryFile = serde_json::from_str(&s).unwrap();
        let cfg = ron::ser::PrettyConfig::new().depth_limit(64).indentor("    ".into());
        let ron_s = ron::ser::to_string_pretty(&file, cfg).unwrap();
        let file2: MaterialRegistryFile = ron::de::from_str(&ron_s).unwrap();
        assert_eq!(
            serde_json::to_value(&file).unwrap(),
            serde_json::to_value(&file2).unwrap()
        );
    }

    #[test]
    fn material_minimal_ron_loads_via_path() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_registry_minimal.example.ron");
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        assert_eq!(reg.schema_version, 2);
        assert_eq!(reg.materials.len(), 1);
        assert!(reg.name_to_id.contains_key("water_deep"));
    }

    #[test]
    fn material_full_example_json_and_ron_decode_to_same_file() {
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain");
        let j: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(
                &dir.join("material_registry.example.json"),
            )
            .unwrap();
        let r: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(
                &dir.join("material_registry.example.ron"),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(&j).unwrap(),
            serde_json::to_value(&r).unwrap()
        );
    }

    #[test]
    fn material_full_example_json_and_ron_load_same_registry() {
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain");
        let a =
            MaterialRegistry::load_from_json(dir.join("material_registry.example.json").to_str().unwrap())
                .unwrap();
        let b =
            MaterialRegistry::load_from_json(dir.join("material_registry.example.ron").to_str().unwrap())
                .unwrap();
        assert_eq!(a.schema_version, b.schema_version);
        assert_eq!(a.materials.len(), b.materials.len());
        assert_eq!(a.name_to_id, b.name_to_id);
        for (ma, mb) in a.materials.iter().zip(b.materials.iter()) {
            assert_eq!(ma.name, mb.name);
            assert_eq!(ma.family, mb.family);
            assert_eq!(ma.tags, mb.tags);
            assert_eq!(ma.properties, mb.properties);
            assert_eq!(ma.preview_color, mb.preview_color);
        }
    }

    #[test]
    #[ignore = "Regenerates material_registry.example.ron from JSON. Run: cargo test emit_material_registry_example_ron_fixture -- --ignored --nocapture"]
    fn emit_material_registry_example_ron_fixture() {
        let path = example_material_registry_path();
        let s = std::fs::read_to_string(&path).unwrap();
        let file: MaterialRegistryFile = serde_json::from_str(&s).unwrap();
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_registry.example.ron");
        let cfg = ron::ser::PrettyConfig::new().depth_limit(64).indentor("    ".into());
        let ron_s = ron::ser::to_string_pretty(&file, cfg).unwrap();
        std::fs::write(&out, format!("{}\n", ron_s.trim_end())).unwrap();
    }
}
