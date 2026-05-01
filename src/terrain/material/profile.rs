//! World profile asset — bundles terrain registry paths for scenarios (U7).

use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, AssetServer, LoadContext};
use bevy::prelude::{Handle, Resource};
use bevy::reflect::TypePath;
use serde::Deserialize;

use super::registry::MaterialRegistry;
use super::rules::RuleSet;
use super::tags::TagRegistry;

/// Scenario bundle: one profile picks material/tag/rule assets (and optional tuning path).
#[derive(Asset, TypePath, Clone, Debug, Deserialize)]
pub struct WorldProfile {
    pub schema_version: u32,
    pub material_registry: String,
    pub tag_registry: String,
    pub material_rules: String,
    pub tuning: Option<String>,
}

/// Handles produced when applying a loaded profile (not yet inserted as resources — caller decides).
pub struct ProfileHandles {
    pub material_registry: Handle<MaterialRegistry>,
    pub tag_registry: Handle<TagRegistry>,
    pub rule_set: Handle<RuleSet>,
}

#[derive(Resource, Default, Debug)]
pub struct WorldProfileSelector {
    pub active: Option<Handle<WorldProfile>>,
}

/// Load terrain registry assets from a [`WorldProfile`].
///
/// **Persistence:** saves, snapshots, and network payloads must store [`MaterialDef`](super::registry::MaterialDef::name)
/// **names**, not raw [`MaterialId`](super::registry::MaterialId) — dense ids are valid only for the currently loaded registry.
pub fn apply_profile(asset_server: &AssetServer, profile: &WorldProfile) -> ProfileHandles {
    ProfileHandles {
        material_registry: asset_server.load(profile.material_registry.clone()),
        tag_registry: asset_server.load(profile.tag_registry.clone()),
        rule_set: asset_server.load(profile.material_rules.clone()),
    }
}

#[derive(Default, TypePath)]
pub struct WorldProfileLoader;

#[derive(Debug)]
pub enum WorldProfileLoaderError {
    Io(std::io::Error),
    Ron(String),
}

impl fmt::Display for WorldProfileLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Ron(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WorldProfileLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Ron(_) => None,
        }
    }
}

impl From<std::io::Error> for WorldProfileLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl AssetLoader for WorldProfileLoader {
    type Asset = WorldProfile;
    type Settings = ();
    type Error = WorldProfileLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let s = std::str::from_utf8(&bytes).map_err(|e| WorldProfileLoaderError::Ron(e.to_string()))?;
        ron::de::from_str(s).map_err(|e| WorldProfileLoaderError::Ron(format!("RON: {e}")))
    }

    fn extensions(&self) -> &[&str] {
        &["world_profile.ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_profile_loads_default_ron() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("assets/config/terrain/profiles/default.world_profile.ron");
        let s = std::fs::read_to_string(&path).unwrap();
        let p: WorldProfile = ron::de::from_str(&s).unwrap();
        assert_eq!(p.schema_version, 1);
        assert!(p.material_registry.contains("material_registry"));
    }
}
