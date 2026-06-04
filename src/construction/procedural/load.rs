//! RON loaders for StylePack files (PG-1).

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use super::types::{FallbackPolicy, StylePack, StylePackId, StylePackRegistry};

pub const STYLE_PACKS_DIR: &str = "assets/configs/buildings/style_packs";

#[derive(Debug, Clone, Deserialize)]
struct StylePackFileRon {
    schema_version: u32,
    style_pack_id: String,
    label: String,
    usage_bias: Vec<String>,
    style_tags: Vec<String>,
    slots: StylePackSlotsRon,
    fallback_policy: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct StylePackSlotsRon {
    #[serde(default)]
    wall_1u: String,
    #[serde(default)]
    wall_2u: String,
    #[serde(default)]
    door_default: String,
    #[serde(default)]
    door_wide: String,
    #[serde(default)]
    window_1u: String,
    #[serde(default)]
    window_2u: String,
    #[serde(default)]
    window_industrial: String,
    #[serde(default)]
    roof_default: String,
    #[serde(default)]
    roof_flat: String,
    #[serde(default)]
    roof_industrial: String,
    #[serde(default)]
    corner_outer: String,
    #[serde(default)]
    corner_inner: String,
    #[serde(default)]
    prop_clutter: String,
}

impl StylePackSlotsRon {
    fn into_map(self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        let pairs = [
            ("wall_1u", self.wall_1u),
            ("wall_2u", self.wall_2u),
            ("door_default", self.door_default),
            ("door_wide", self.door_wide),
            ("window_1u", self.window_1u),
            ("window_2u", self.window_2u),
            ("window_industrial", self.window_industrial),
            ("roof_default", self.roof_default),
            ("roof_flat", self.roof_flat),
            ("roof_industrial", self.roof_industrial),
            ("corner_outer", self.corner_outer),
            ("corner_inner", self.corner_inner),
            ("prop_clutter", self.prop_clutter),
        ];
        for (key, value) in pairs {
            if !value.is_empty() {
                map.insert(key.to_owned(), value);
            }
        }
        map
    }
}

fn parse_fallback_policy(raw: &str) -> FallbackPolicy {
    match raw {
        "primitive_footprint" => FallbackPolicy::PrimitiveFootprint,
        _ => FallbackPolicy::HideSlot,
    }
}

fn repo_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

#[must_use]
pub fn default_style_packs_dir() -> PathBuf {
    repo_path(STYLE_PACKS_DIR)
}

fn parse_style_pack_file(path: &Path, text: &str) -> Result<StylePack, String> {
    let raw: StylePackFileRon =
        ron::from_str(text).map_err(|e| format!("{}: {e}", path.display()))?;
    if raw.schema_version != 1 {
        return Err(format!(
            "{}: unsupported schema_version {}",
            path.display(),
            raw.schema_version
        ));
    }
    if raw.style_pack_id.is_empty() {
        return Err(format!("{}: empty style_pack_id", path.display()));
    }
    let slots = raw.slots.into_map();
    if slots.is_empty() {
        return Err(format!("{}: no slots", path.display()));
    }
    Ok(StylePack {
        schema_version: raw.schema_version,
        id: StylePackId(raw.style_pack_id),
        label: raw.label,
        usage_bias: raw.usage_bias,
        style_tags: raw.style_tags,
        slots,
        fallback_policy: parse_fallback_policy(&raw.fallback_policy),
    })
}

#[must_use]
pub fn load_style_packs_from_dir(dir: &Path) -> StylePackRegistry {
    let mut registry = StylePackRegistry::default();
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(err) => {
            registry
                .load_errors
                .push(format!("read_dir {}: {err}", dir.display()));
            return registry;
        }
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if !file_name.starts_with("style_") || !file_name.ends_with(".ron") {
            continue;
        }
        if file_name.starts_with("style_") && file_name.contains("_manifest") {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(err) => {
                registry
                    .load_errors
                    .push(format!("read {}: {err}", path.display()));
                continue;
            }
        };
        match parse_style_pack_file(&path, &text) {
            Ok(pack) => {
                let id = pack.id.as_str().to_owned();
                if registry.packs.contains_key(&id) {
                    registry.load_errors.push(format!(
                        "duplicate style_pack_id `{id}` ({})",
                        path.display()
                    ));
                    continue;
                }
                registry.packs.insert(id, pack);
            }
            Err(err) => registry.load_errors.push(err),
        }
    }

    registry
}

#[must_use]
pub fn load_style_pack_registry() -> StylePackRegistry {
    load_style_packs_from_dir(&default_style_packs_dir())
}

pub fn init_style_pack_registry(mut commands: Commands) {
    let registry = load_style_pack_registry();
    if !registry.load_errors.is_empty() {
        for err in &registry.load_errors {
            warn!(target: "procedural_style_pack", "{err}");
        }
    } else {
        info!(
            target: "procedural_style_pack",
            "StylePackRegistry: {} packs loaded",
            registry.len()
        );
    }
    commands.insert_resource(registry);
}
