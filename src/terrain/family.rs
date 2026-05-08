//! Config-driven terrain **families** — dense ids, names, and gameplay buckets.
//! Classification resolves to [`TerrainFamilyId`] via [`TerrainFamilyRegistry`] (asset), not a Rust enum.

use std::collections::HashMap;
use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::terrain::biome::{compute_biome_weights, BiomeBucket, BiomeTuning, BiomeWeights};

/// Dense runtime id into [`TerrainFamilyRegistry::families`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct TerrainFamilyId(pub u16);

/// Unclassified cells before pass 3 — **`Grassland`** row in `terrain_family_registry.example.json` (index **4**).
/// If your project reorders that file, set this to match or call [`TerrainFamilyRegistry::id`]`("Grassland")` at runtime.
pub const DEFAULT_TERRAIN_FAMILY_ID: TerrainFamilyId = TerrainFamilyId(4);

#[derive(Clone, Copy, Debug)]
pub struct BiomeClassification {
    pub terrain_family: TerrainFamilyId,
    pub biome_weights: BiomeWeights,
}

/// Height / moisture / temperature → soft weights + **hard** family name looked up in `families`.
/// Family **names** produced here must exist in `terrain_family_registry.json` (classification outputs
/// are a subset of all materials-only families like `Cliff`).
pub fn classify_biome(
    height: f32,
    moisture: f32,
    temperature: f32,
    tuning: &BiomeTuning,
    families: &TerrainFamilyRegistry,
) -> BiomeClassification {
    let weights = compute_biome_weights(height, moisture, temperature, tuning);
    let name = classify_terrain_family_name(height, moisture, temperature, tuning);
    let terrain_family = families.require_id(name).unwrap_or_else(|e| {
        panic!(
            "classify_biome: registry missing classification outcome {:?}: {}",
            name, e
        )
    });
    BiomeClassification {
        terrain_family,
        biome_weights: weights,
    }
}

fn classify_terrain_family_name(
    height: f32,
    moisture: f32,
    temperature: f32,
    tuning: &BiomeTuning,
) -> &'static str {
    if height < tuning.deep_water_height_max {
        "DeepWater"
    } else if height < tuning.shallow_water_height_max {
        "ShallowWater"
    } else if height < tuning.beach_height_max {
        "Beach"
    } else if height > tuning.mountain_height_min {
        if temperature < tuning.snow_peak_temperature_max {
            "SnowCappedMountain"
        } else {
            "Mountain"
        }
    } else if temperature < tuning.tundra_temperature_max {
        "Tundra"
    } else if temperature > tuning.hot_lowlands_temperature_min {
        if moisture < tuning.desert_moisture_max {
            "Desert"
        } else if moisture > tuning.swamp_moisture_min {
            "Swamp"
        } else {
            "Grassland"
        }
    } else if moisture < tuning.grassland_moisture_max {
        "Grassland"
    } else if moisture < tuning.forest_moisture_max {
        "Forest"
    } else {
        "DenseForest"
    }
}

#[derive(Clone, Debug, Deserialize)]
struct TerrainFamilyEntryFile {
    name: String,
    biome_bucket: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TerrainFamilyRegistryFile {
    pub schema_version: u32,
    families: Vec<TerrainFamilyEntryFile>,
}

/// Loaded family table + name → id (row order defines ids).
#[derive(Asset, TypePath, Clone, Debug)]
pub struct TerrainFamilyRegistry {
    pub schema_version: u32,
    pub families: Vec<TerrainFamilyDef>,
    pub name_to_id: HashMap<String, TerrainFamilyId>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TerrainFamilyDef {
    pub name: String,
    pub biome_bucket: BiomeBucket,
}

pub const SUPPORTED_TERRAIN_FAMILY_REGISTRY_SCHEMA_VERSIONS: &[u32] = &[1];

impl TerrainFamilyRegistry {
    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let file: TerrainFamilyRegistryFile = serde_json::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON: {e}"))
        })?;
        Self::from_file(file).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn from_file(file: TerrainFamilyRegistryFile) -> Result<Self, String> {
        if !SUPPORTED_TERRAIN_FAMILY_REGISTRY_SCHEMA_VERSIONS.contains(&file.schema_version) {
            return Err(format!(
                "terrain_family_registry schema_version={} unsupported; expected one of {:?}",
                file.schema_version, SUPPORTED_TERRAIN_FAMILY_REGISTRY_SCHEMA_VERSIONS
            ));
        }
        if file.families.len() > u16::MAX as usize {
            return Err(format!(
                "too many terrain families ({}); max {}",
                file.families.len(),
                u16::MAX
            ));
        }
        let mut name_to_id = HashMap::new();
        let mut families = Vec::with_capacity(file.families.len());
        for (i, e) in file.families.into_iter().enumerate() {
            let id = TerrainFamilyId(i as u16);
            let bucket = crate::terrain::biome::biome_bucket_from_str(&e.biome_bucket).map_err(
                |err| {
                    format!(
                        "family {:?}: invalid biome_bucket {:?}: {}",
                        e.name, e.biome_bucket, err
                    )
                },
            )?;
            if name_to_id.insert(e.name.clone(), id).is_some() {
                return Err(format!("duplicate terrain family name {:?}", e.name));
            }
            families.push(TerrainFamilyDef {
                name: e.name,
                biome_bucket: bucket,
            });
        }
        Ok(Self {
            schema_version: file.schema_version,
            families,
            name_to_id,
        })
    }

    #[inline]
    pub fn id(&self, name: &str) -> Option<TerrainFamilyId> {
        self.name_to_id.get(name).copied()
    }

    pub fn require_id(&self, name: &str) -> Result<TerrainFamilyId, String> {
        self.id(name).ok_or_else(|| {
            format!("unknown terrain family {:?}; add it to terrain_family_registry.json", name)
        })
    }

    #[inline]
    pub fn def(&self, id: TerrainFamilyId) -> Option<&TerrainFamilyDef> {
        self.families.get(id.0 as usize)
    }

    #[inline]
    pub fn biome_bucket(&self, id: TerrainFamilyId) -> Option<BiomeBucket> {
        self.def(id).map(|d| d.biome_bucket)
    }
}

/// Bevy loader `*.terrain_family_registry.json`.
#[derive(Default, TypePath)]
pub struct TerrainFamilyRegistryLoader;

#[derive(Debug)]
pub enum TerrainFamilyRegistryLoaderError {
    Io(std::io::Error),
    Json(String),
}

impl fmt::Display for TerrainFamilyRegistryLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TerrainFamilyRegistryLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(_) => None,
        }
    }
}

impl From<std::io::Error> for TerrainFamilyRegistryLoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl AssetLoader for TerrainFamilyRegistryLoader {
    type Asset = TerrainFamilyRegistry;
    type Settings = ();
    type Error = TerrainFamilyRegistryLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let file: TerrainFamilyRegistryFile = serde_json::from_slice(&bytes)
            .map_err(|e| TerrainFamilyRegistryLoaderError::Json(e.to_string()))?;
        TerrainFamilyRegistry::from_file(file).map_err(TerrainFamilyRegistryLoaderError::Json)
    }

    fn extensions(&self) -> &[&str] {
        &["terrain_family_registry.json"]
    }
}

pub fn hash_terrain_family_registry(reg: &TerrainFamilyRegistry) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    reg.schema_version.hash(&mut h);
    reg.families.len().hash(&mut h);
    for f in &reg.families {
        f.name.hash(&mut h);
        f.biome_bucket.hash(&mut h);
    }
    h.finish()
}

/// Process-wide cache of the default example terrain family registry (same path material JSON uses).
pub fn default_terrain_families() -> &'static TerrainFamilyRegistry {
    static CACHE: std::sync::OnceLock<TerrainFamilyRegistry> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/terrain_family_registry.example.json");
        TerrainFamilyRegistry::load_from_json(path.to_str().expect("utf8 path"))
            .expect("load default terrain_family_registry.example.json")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/terrain_family_registry.example.json")
    }

    #[test]
    fn loads_example() {
        let r = TerrainFamilyRegistry::load_from_json(example_path().to_str().unwrap()).unwrap();
        assert!(!r.families.is_empty());
        assert!(r.id("Grassland").is_some());
    }

    #[test]
    fn classify_uses_registry() {
        let r = TerrainFamilyRegistry::load_from_json(example_path().to_str().unwrap()).unwrap();
        let t = BiomeTuning::default();
        let c = classify_biome(0.55, 0.35, 0.48, &t, &r);
        assert_eq!(r.def(c.terrain_family).unwrap().name, "Grassland");
    }
}
