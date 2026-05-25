//! Phase 4.1 — build-rail icon atlas (texture + RON UV manifest).
//!
//! Spec: `prompts/guides/ui/ui_phase4_icon_atlas_brief_v1.md` §10–§11.

use std::collections::HashMap;
use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::math::Rect;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::construction::ToolContext;

pub const ICON_ATLAS_TEXTURE_PATH: &str = "textures/ui/icon_atlas_phase4_v1.png";
pub const ICON_ATLAS_MANIFEST_PATH: &str = "configs/ui/icon_atlas_phase4.icon_atlas.ron";

/// Atlas cell id (brief §3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconId {
    Rd,
    Rl,
    Ut,
    In,
    Cv,
    UtTx,
    UtMg,
    Truck,
    Ural,
    Bus,
    P5Br,
}

impl IconId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rd => "RD",
            Self::Rl => "RL",
            Self::Ut => "UT",
            Self::In => "IN",
            Self::Cv => "CV",
            Self::UtTx => "UT_TX",
            Self::UtMg => "UT_MG",
            Self::Truck => "TRUCK",
            Self::Ural => "URAL",
            Self::Bus => "BUS",
            Self::P5Br => "P5_BR",
        }
    }
}

#[must_use]
pub fn tool_context_icon_id(ctx: ToolContext) -> Option<IconId> {
    match ctx {
        ToolContext::Roads => Some(IconId::Rd),
        ToolContext::Rail => Some(IconId::Rl),
        ToolContext::Utilities => Some(IconId::Ut),
        ToolContext::Industry => Some(IconId::In),
        ToolContext::Civil => Some(IconId::Cv),
        _ => None,
    }
}

#[must_use]
pub fn tool_context_uses_icon_atlas(ctx: ToolContext) -> bool {
    tool_context_icon_id(ctx).is_some()
}

#[derive(Debug, Clone, Deserialize)]
pub struct IconAtlasCellRon {
    pub col: u32,
    pub row: u32,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct IconAtlasManifest {
    pub schema_version: u32,
    pub texture: String,
    pub cell_size: (u32, u32),
    #[serde(default)]
    pub atlas_size: (u32, u32),
    pub icons: HashMap<String, IconAtlasCellRon>,
}

impl IconAtlasManifest {
    #[must_use]
    pub fn cell_rect(&self, id: IconId) -> Option<Rect> {
        let cell = self.icons.get(id.as_str())?;
        let (cw, ch) = self.cell_size;
        let x0 = (cell.col * cw) as f32;
        let y0 = (cell.row * ch) as f32;
        Some(Rect {
            min: Vec2::new(x0, y0),
            max: Vec2::new(x0 + cw as f32, y0 + ch as f32),
        })
    }
}

/// Loaded atlas handles (Startup).
#[derive(Resource, Debug, Clone)]
pub struct IconAtlasUi {
    pub atlas: Handle<Image>,
    pub manifest: Handle<IconAtlasManifest>,
}

impl IconAtlasUi {
    #[must_use]
    pub fn image_node_for_id(
        &self,
        manifests: &Assets<IconAtlasManifest>,
        id: IconId,
    ) -> Option<bevy::ui::widget::ImageNode> {
        let manifest = manifests.get(&self.manifest)?;
        let rect = manifest.cell_rect(id)?;
        Some(
            bevy::ui::widget::ImageNode::new(self.atlas.clone())
                .with_rect(rect)
                .with_mode(bevy::ui::widget::NodeImageMode::Auto),
        )
    }

    #[must_use]
    pub fn image_node_for_tool(
        &self,
        manifests: &Assets<IconAtlasManifest>,
        ctx: ToolContext,
    ) -> Option<bevy::ui::widget::ImageNode> {
        let id = tool_context_icon_id(ctx)?;
        self.image_node_for_id(manifests, id)
    }

    #[must_use]
    pub fn manifest_loaded(&self, manifests: &Assets<IconAtlasManifest>) -> bool {
        manifests.get(&self.manifest).is_some()
    }
}

#[derive(Default, TypePath)]
pub struct IconAtlasManifestLoader;

#[derive(Debug)]
pub enum IconAtlasManifestLoaderError {
    Io(std::io::Error),
    Ron(String),
}

impl fmt::Display for IconAtlasManifestLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Ron(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for IconAtlasManifestLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Ron(_) => None,
        }
    }
}

impl From<std::io::Error> for IconAtlasManifestLoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl AssetLoader for IconAtlasManifestLoader {
    type Asset = IconAtlasManifest;
    type Settings = ();
    type Error = IconAtlasManifestLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let s = std::str::from_utf8(&bytes)
            .map_err(|e| IconAtlasManifestLoaderError::Ron(e.to_string()))?;
        ron::from_str(s).map_err(|e| IconAtlasManifestLoaderError::Ron(format!("RON: {e}")))
    }

    fn extensions(&self) -> &[&str] {
        &["icon_atlas.ron"]
    }
}

pub fn load_icon_atlas_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(IconAtlasUi {
        atlas: asset_server.load(ICON_ATLAS_TEXTURE_PATH),
        manifest: asset_server.load(ICON_ATLAS_MANIFEST_PATH),
    });
}

pub struct IconAtlasPlugin;

impl Plugin for IconAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<IconAtlasManifest>()
            .register_asset_loader(IconAtlasManifestLoader)
            .add_systems(Startup, load_icon_atlas_ui_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_cell_rect_matches_brief_uv_grid() {
        let manifest: IconAtlasManifest = ron::from_str(
            r#"
(
  schema_version: 1,
  texture: "textures/ui/icon_atlas_phase4_v1.png",
  cell_size: (32, 32),
  icons: {
    "UT": (col: 2, row: 0),
    "UT_MG": (col: 1, row: 1),
  },
)
"#,
        )
        .expect("ron");
        let ut = manifest.cell_rect(IconId::Ut).expect("UT");
        assert_eq!(ut.min, Vec2::new(64.0, 0.0));
        assert_eq!(ut.max, Vec2::new(96.0, 32.0));
        let mg = manifest.cell_rect(IconId::UtMg).expect("UT_MG");
        assert_eq!(mg.min, Vec2::new(32.0, 32.0));
        assert_eq!(mg.max, Vec2::new(64.0, 64.0));
        assert_ne!(ut.min, mg.min);
    }

    #[test]
    fn p5_br_cell_rect_row_three() {
        let manifest: IconAtlasManifest = ron::from_str(
            r#"
(
  schema_version: 1,
  texture: "textures/ui/icon_atlas_phase4_v1.png",
  cell_size: (32, 32),
  icons: { "P5_BR": (col: 0, row: 3) },
)
"#,
        )
        .expect("ron");
        let p5 = manifest.cell_rect(IconId::P5Br).expect("P5_BR");
        assert_eq!(p5.min, Vec2::new(0.0, 96.0));
        assert_eq!(p5.max, Vec2::new(32.0, 128.0));
    }

    #[test]
    fn image_node_for_id_uses_manifest_uv() {
        let mut manifests = Assets::<IconAtlasManifest>::default();
        let handle = manifests.add(IconAtlasManifest {
            schema_version: 1,
            texture: ICON_ATLAS_TEXTURE_PATH.into(),
            cell_size: (32, 32),
            atlas_size: (256, 128),
            icons: HashMap::from([(
                "UT".into(),
                IconAtlasCellRon { col: 2, row: 0 },
            )]),
        });
        let atlas = IconAtlasUi {
            atlas: Handle::default(),
            manifest: handle,
        };
        let node = atlas
            .image_node_for_id(&manifests, IconId::Ut)
            .expect("node");
        assert_eq!(
            node.rect,
            Some(Rect::new(64.0, 0.0, 96.0, 32.0))
        );
    }

    #[test]
    fn vehicle_row_cell_rects() {
        let manifest: IconAtlasManifest = ron::from_str(
            r#"
(
  schema_version: 1,
  texture: "textures/ui/icon_atlas_phase4_v1.png",
  cell_size: (32, 32),
  icons: {
    "TRUCK": (col: 0, row: 2),
    "URAL": (col: 1, row: 2),
    "BUS": (col: 2, row: 2),
  },
)
"#,
        )
        .expect("ron");
        let truck = manifest.cell_rect(IconId::Truck).expect("TRUCK");
        assert_eq!(truck.min, Vec2::new(0.0, 64.0));
        let ural = manifest.cell_rect(IconId::Ural).expect("URAL");
        assert_eq!(ural.min, Vec2::new(32.0, 64.0));
        let bus = manifest.cell_rect(IconId::Bus).expect("BUS");
        assert_eq!(bus.min, Vec2::new(64.0, 64.0));
    }

    #[test]
    fn image_node_for_id_p5_br() {
        let mut manifests = Assets::<IconAtlasManifest>::default();
        let handle = manifests.add(IconAtlasManifest {
            schema_version: 1,
            texture: ICON_ATLAS_TEXTURE_PATH.into(),
            cell_size: (32, 32),
            atlas_size: (256, 128),
            icons: HashMap::from([(
                "P5_BR".into(),
                IconAtlasCellRon { col: 0, row: 3 },
            )]),
        });
        let atlas = IconAtlasUi {
            atlas: Handle::default(),
            manifest: handle,
        };
        let node = atlas
            .image_node_for_id(&manifests, IconId::P5Br)
            .expect("P5Br node");
        assert_eq!(
            node.rect,
            Some(Rect::new(0.0, 96.0, 32.0, 128.0))
        );
    }

    #[test]
    fn tool_context_maps_row_zero_icons() {
        assert_eq!(tool_context_icon_id(ToolContext::Utilities), Some(IconId::Ut));
        assert_eq!(tool_context_icon_id(ToolContext::Military), None);
    }
}
