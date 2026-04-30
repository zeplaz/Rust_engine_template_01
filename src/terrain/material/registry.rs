use std::collections::HashMap;
use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::de;
use serde::Deserialize;

use crate::terrain::biome::TerrainClass;

fn deserialize_terrain_class_from_json_str<'de, D>(deserializer: D) -> Result<TerrainClass, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    terrain_class_from_pascalcase_str(&s).map_err(de::Error::custom)
}

fn terrain_class_from_pascalcase_str(s: &str) -> Result<TerrainClass, String> {
    match s {
        "DeepWater" => Ok(TerrainClass::DeepWater),
        "ShallowWater" => Ok(TerrainClass::ShallowWater),
        "Beach" => Ok(TerrainClass::Beach),
        "Desert" => Ok(TerrainClass::Desert),
        "Grassland" => Ok(TerrainClass::Grassland),
        "Forest" => Ok(TerrainClass::Forest),
        "DenseForest" => Ok(TerrainClass::DenseForest),
        "Mountain" => Ok(TerrainClass::Mountain),
        "SnowCappedMountain" => Ok(TerrainClass::SnowCappedMountain),
        "Tundra" => Ok(TerrainClass::Tundra),
        "Swamp" => Ok(TerrainClass::Swamp),
        "Cliff" => Ok(TerrainClass::Cliff),
        "Concrete" => Ok(TerrainClass::Concrete),
        "Dirt" => Ok(TerrainClass::Dirt),
        "Snow" => Ok(TerrainClass::Snow),
        "Stone" => Ok(TerrainClass::Stone),
        _ => Err(format!("unknown TerrainClass / family string: {s}")),
    }
}

/// Runtime material variant id (dense index into a loaded registry).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaterialId(pub u16);

/// One row from `material_registry.json` after load (tag names preserved; intern against [`super::TagRegistry`] separately).
#[derive(Clone, Debug, Deserialize)]
pub struct MaterialDef {
    pub name: String,
    #[serde(deserialize_with = "deserialize_terrain_class_from_json_str")]
    pub family: TerrainClass,
    pub tags: Vec<String>,
    pub properties: serde_json::Value,
    pub preview_color: [u8; 4],
}

#[derive(Clone, Debug, Deserialize)]
struct MaterialRegistryFile {
    pub schema_version: u32,
    pub materials: Vec<MaterialDef>,
}

/// Loaded material table + deterministic name → id map (input row order defines ids).
#[derive(Asset, TypePath, Clone, Debug)]
pub struct MaterialRegistry {
    pub schema_version: u32,
    pub materials: Vec<MaterialDef>,
    pub name_to_id: HashMap<String, MaterialId>,
}

impl MaterialRegistry {
    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let file: MaterialRegistryFile = serde_json::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON: {e}"))
        })?;
        Ok(Self::from_file(file))
    }

    fn from_file(file: MaterialRegistryFile) -> Self {
        let mut name_to_id = HashMap::new();
        for (i, m) in file.materials.iter().enumerate() {
            let id = MaterialId(i as u16);
            name_to_id.insert(m.name.clone(), id);
        }
        Self {
            schema_version: file.schema_version,
            materials: file.materials,
            name_to_id,
        }
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
        Ok(MaterialRegistry::from_file(file))
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

    #[test]
    fn material_registry_loads_example() {
        let path = example_material_registry_path();
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        assert_eq!(reg.schema_version, 1);
        assert!(reg.materials.len() >= 4);
        assert_eq!(
            reg.name_to_id.get("loam_wet"),
            Some(&MaterialId(0))
        );
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
