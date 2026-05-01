//! Bit-packed tag sets. **ASK:** if total distinct tags can exceed **256** — drives designer `§43` (`implementation_questions_v1.md`).

use std::collections::HashMap;
use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::Deserialize;

/// Interned tag id (bit index into [`TagSet`], max **256** tags).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TagId(pub u16);

/// Fixed-width set of up to **256** tags (four `u64` lanes).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TagSet([u64; 4]);

impl TagSet {
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

    /// True iff every tag set in `required` is also present in `self`.
    pub fn intersects_all(&self, required: &Self) -> bool {
        (0..4).all(|lane| (self.0[lane] & required.0[lane]) == required.0[lane])
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TagDef {
    pub name: String,
    pub category: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TagRegistryFile {
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
    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let file: TagRegistryFile = serde_json::from_str(&s).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON: {e}"))
        })?;
        Ok(Self::from_file(file))
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
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let file: TagRegistryFile = serde_json::from_slice(&bytes)?;
        Ok(TagRegistry::from_file(file))
    }

    fn extensions(&self) -> &[&str] {
        &["tag_registry.json"]
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
    }
}
