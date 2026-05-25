//! Wave **S** product-shell persistence DTOs (presentation only).

use serde::{Deserialize, Serialize};

use super::layout_store::HudLayoutCollectionR8;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MinimapBookmarkEntryR8 {
    pub label: String,
    pub world_x: f32,
    pub world_y: f32,
    pub zoom: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MinimapBookmarkCollectionR8 {
    pub schema_version: u32,
    pub bookmarks: Vec<MinimapBookmarkEntryR8>,
}

impl MinimapBookmarkCollectionR8 {
    pub const CURRENT_SCHEMA: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            bookmarks: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OverlayPresetEntryR8 {
    pub overlay_tag: String,
    pub enabled: bool,
    pub opacity: f32,
    pub blend_weight: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OverlayPresetCollectionR8 {
    pub schema_version: u32,
    pub presets: Vec<OverlayPresetEntryR8>,
}

impl OverlayPresetCollectionR8 {
    pub const CURRENT_SCHEMA: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            presets: Vec::new(),
        }
    }
}

/// Bundle slot for Wave **S** interchange — binary envelope deferred (**BQ-133**).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProductShellPersistenceBundleR8 {
    pub schema_version: u32,
    pub layout: HudLayoutCollectionR8,
    pub minimap_bookmarks: MinimapBookmarkCollectionR8,
    pub overlay_presets: OverlayPresetCollectionR8,
    pub blueprint_preset_ref: Option<String>,
}

impl ProductShellPersistenceBundleR8 {
    pub const CURRENT_SCHEMA: u32 = 1;

    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            layout: HudLayoutCollectionR8::new(),
            minimap_bookmarks: MinimapBookmarkCollectionR8::new(),
            overlay_presets: OverlayPresetCollectionR8::new(),
            blueprint_preset_ref: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_shell_persistence_bundle_ron_roundtrip() {
        let bundle = ProductShellPersistenceBundleR8::new();
        let ron = ron::ser::to_string(&bundle).expect("serialize");
        let back: ProductShellPersistenceBundleR8 = ron::from_str(&ron).expect("deserialize");
        assert_eq!(bundle, back);

        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("debug_runs/wave_s_shell_roundtrip.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let payload = serde_json::json!({
            "profile": "WAVE_S_SHELL_ROUNDTRIP",
            "schema_version": bundle.schema_version,
            "ron_body": ron,
            "roundtrip_ok": bundle == back,
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&payload).expect("json")).expect("write fixture");
    }
}
