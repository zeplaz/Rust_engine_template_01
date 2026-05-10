//! Bit-packed tag sets. **ASK:** if total distinct tags can exceed **256** — drives designer `§43` (`implementation_questions_v1.md`).

use std::collections::HashMap;
use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

/// Interned tag id (bit index into [`TagSet`], max **256** tags).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TagId(pub u16);

/// Fixed-width set of up to **256** tags (four `u64` lanes).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct TagSet([u64; 4]);

impl TagSet {
    /// Every tag id **0..256** allowed (material / preview pool default).
    pub const ALL: Self = Self([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

    #[inline]
    pub fn bits(&self) -> [u64; 4] {
        self.0
    }

    pub fn insert(&mut self, id: TagId) {
        let i = id.0 as usize;
        if i >= 256 {
            debug_assert!(false, "TagId {} out of TagSet range (ASK: §43)", i);
            return;
        }
        let lane = i / 64;
        let bit = i % 64;
        self.0[lane] |= 1u64 << bit;
    }

    pub fn remove(&mut self, id: TagId) {
        let i = id.0 as usize;
        if i >= 256 {
            return;
        }
        let lane = i / 64;
        let bit = i % 64;
        self.0[lane] &= !(1u64 << bit);
    }

    pub fn contains(&self, id: TagId) -> bool {
        let i = id.0 as usize;
        if i >= 256 {
            return false;
        }
        let lane = i / 64;
        let bit = i % 64;
        (self.0[lane] & (1u64 << bit)) != 0
    }

    pub fn union(self, other: &Self) -> Self {
        Self([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }

    /// True if any tag is present in both sets.
    pub fn intersects(&self, other: &Self) -> bool {
        (0..4).any(|lane| (self.0[lane] & other.0[lane]) != 0)
    }

    /// True iff every tag set in `required` is also present in `self`.
    pub fn intersects_all(&self, required: &Self) -> bool {
        (0..4).all(|lane| (self.0[lane] & required.0[lane]) == required.0[lane])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagDef {
    pub name: String,
    pub category: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TagRegistryFile {
    pub schema_version: u32,
    pub tags: Vec<TagDef>,
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct TagRegistry {
    pub schema_version: u32,
    pub tags: Vec<TagDef>,
    pub name_to_id: HashMap<String, TagId>,
}

impl TagRegistry {
    /// Load from **`*.ron`** or **`*.json`** (extension selects parser; unknown tries RON then JSON).
    pub fn load_from_path(path: &std::path::Path) -> std::io::Result<Self> {
        let file: TagRegistryFile = crate::terrain::registry_serde_path::read_to_deserializable(path)?;
        Ok(Self::from_file(file))
    }

    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        Self::load_from_path(std::path::Path::new(path))
    }

    pub(crate) fn from_file(file: TagRegistryFile) -> Self {
        let mut name_to_id = HashMap::new();
        for (i, t) in file.tags.iter().enumerate() {
            name_to_id.insert(t.name.clone(), TagId(i as u16));
        }
        Self {
            schema_version: file.schema_version,
            tags: file.tags,
            name_to_id,
        }
    }

    pub fn tag_id(&self, name: &str) -> Option<TagId> {
        self.name_to_id.get(name).copied()
    }
}

/// Bevy loader for `*.tag_registry.json`.
#[derive(Default, TypePath)]
pub struct TagRegistryLoader;

#[derive(Debug)]
pub enum TagRegistryLoaderError {
    Io(std::io::Error),
    Json(String),
}

impl fmt::Display for TagRegistryLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TagRegistryLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(_) => None,
        }
    }
}

impl From<std::io::Error> for TagRegistryLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for TagRegistryLoaderError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

impl AssetLoader for TagRegistryLoader {
    type Asset = TagRegistry;
    type Settings = ();
    type Error = TagRegistryLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|e| TagRegistryLoaderError::Json(format!("UTF-8: {e}")))?;
        let ext = load_context.path().get_full_extension();
        let file: TagRegistryFile = crate::terrain::registry_serde_path::deserialize_from_str_with_extension_opt(
            text,
            ext.as_deref(),
        )
        .map_err(|e| TagRegistryLoaderError::Json(e.to_string()))?;
        Ok(TagRegistry::from_file(file))
    }

    fn extensions(&self) -> &[&str] {
        &["tag_registry.json", "tag_registry.ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_tag_registry_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.json")
    }

    #[test]
    fn material_tag_registry_loads_example() {
        let reg = TagRegistry::load_from_json(
            example_tag_registry_path().to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(reg.schema_version, 1);
        assert!(!reg.tags.is_empty());
        assert!(reg.tag_id("wet").is_some());
    }

    #[test]
    fn material_tag_set_set_ops() {
        let mut a = TagSet::default();
        let mut b = TagSet::default();
        a.insert(TagId(3));
        b.insert(TagId(7));
        assert!(a.contains(TagId(3)));
        assert!(!a.contains(TagId(7)));
        let u = a.union(&b);
        assert!(u.contains(TagId(3)) && u.contains(TagId(7)));

        let mut req = TagSet::default();
        req.insert(TagId(3));
        assert!(u.intersects_all(&req));
        req.insert(TagId(99));
        assert!(!u.intersects_all(&req));
    }

    #[test]
    fn tag_registry_loader_extensions() {
        let loader = TagRegistryLoader::default();
        assert!(loader.extensions().contains(&"tag_registry.json"));
        assert!(loader.extensions().contains(&"tag_registry.ron"));
    }

    #[test]
    fn tag_example_json_round_trips_ron() {
        let json_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain/tag_registry.example.json");
        let ron_path =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain/tag_registry.example.ron");
        let j = TagRegistry::load_from_path(&json_path).unwrap();
        let r = TagRegistry::load_from_path(&ron_path).unwrap();
        assert_eq!(j.schema_version, r.schema_version);
        assert_eq!(j.tags.len(), r.tags.len());
        assert_eq!(j.name_to_id.len(), r.name_to_id.len());
    }

    #[test]
    fn tag_load_respects_unknown_extension_as_loose_ron_json() {
        let tmp = std::env::temp_dir().join("tag_registry_noext_test.txt");
        let reg = TagRegistry::load_from_path(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/config/terrain/tag_registry.example.json"),
        )
        .unwrap();
        let cfg = ron::ser::PrettyConfig::new().depth_limit(8).indentor("    ".into());
        let body = TagRegistryFile {
            schema_version: reg.schema_version,
            tags: reg.tags.clone(),
        };
        let s = ron::ser::to_string_pretty(&body, cfg).unwrap();
        std::fs::write(&tmp, &s).unwrap();
        let _: TagRegistryFile = crate::terrain::registry_serde_path::read_to_deserializable(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);
    }
}
