//! **COD-ART-HUD-ICON-ATLAS-001** — power HUD icon atlas (Lane D).
//!
//! Spec: `src/dev/design_hud_power_icons_v1.md`

use std::collections::HashMap;
use std::fmt;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::math::Rect;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};
use serde::Deserialize;

use crate::construction::PowerLineRoutingMode;
use crate::infrastructure::VoltageClass;

pub const POWER_HUD_ATLAS_TEXTURE_PATH: &str = "textures/ui/power_hud_atlas.png";
pub const POWER_HUD_ATLAS_MANIFEST_PATH: &str = "configs/ui/power_hud.manifest.ron";

/// Atlas cell id — `design_hud_power_icons_v1.md` §1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PowerHudIconId {
    PowerLineTool,
    VoltageLow,
    VoltageMedium,
    VoltageHigh,
    RouteCurved,
    Route90,
    SnapTransformer,
    SnapJunction,
    SubstationPlace,
    TransformerPlace,
    Diesel,
    Scram,
    Island,
    Repair,
}

impl PowerHudIconId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PowerLineTool => "PWR_LINE",
            Self::VoltageLow => "VOLT_L",
            Self::VoltageMedium => "VOLT_M",
            Self::VoltageHigh => "VOLT_H",
            Self::RouteCurved => "ROUTE_CURVE",
            Self::Route90 => "ROUTE_90",
            Self::SnapTransformer => "SNAP_TX",
            Self::SnapJunction => "SNAP_JX",
            Self::SubstationPlace => "PLACE_SUB",
            Self::TransformerPlace => "PLACE_TX",
            Self::Diesel => "DIESEL",
            Self::Scram => "SCRAM",
            Self::Island => "ISLAND",
            Self::Repair => "REPAIR",
        }
    }

    #[must_use]
    pub const fn inventory() -> &'static [Self] {
        &[
            Self::PowerLineTool,
            Self::VoltageLow,
            Self::VoltageMedium,
            Self::VoltageHigh,
            Self::RouteCurved,
            Self::Route90,
            Self::SnapTransformer,
            Self::SnapJunction,
            Self::SubstationPlace,
            Self::TransformerPlace,
            Self::Diesel,
            Self::Scram,
            Self::Island,
            Self::Repair,
        ]
    }

    #[must_use]
    pub fn for_voltage(voltage: VoltageClass) -> Self {
        match voltage {
            VoltageClass::Low => Self::VoltageLow,
            VoltageClass::Medium => Self::VoltageMedium,
            VoltageClass::High => Self::VoltageHigh,
        }
    }

    #[must_use]
    pub fn for_routing_mode(mode: PowerLineRoutingMode) -> Self {
        match mode {
            PowerLineRoutingMode::Curved => Self::RouteCurved,
            PowerLineRoutingMode::Orthogonal90 => Self::Route90,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PowerHudIconCellRon {
    pub col: u32,
    pub row: u32,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct PowerHudIconAtlasManifest {
    pub schema_version: u32,
    pub texture: String,
    pub cell_size: (u32, u32),
    #[serde(default)]
    pub chip_size: (u32, u32),
    #[serde(default)]
    pub atlas_size: (u32, u32),
    pub icons: HashMap<String, PowerHudIconCellRon>,
}

impl PowerHudIconAtlasManifest {
    #[must_use]
    pub fn cell_rect(&self, id: PowerHudIconId) -> Option<Rect> {
        let cell = self.icons.get(id.as_str())?;
        let (cw, ch) = self.cell_size;
        let x0 = (cell.col * cw) as f32;
        let y0 = (cell.row * ch) as f32;
        Some(Rect {
            min: Vec2::new(x0, y0),
            max: Vec2::new(x0 + cw as f32, y0 + ch as f32),
        })
    }

    #[must_use]
    pub fn egui_uv(&self, id: PowerHudIconId) -> Option<egui::Rect> {
        let pixel = self.cell_rect(id)?;
        let (aw, ah) = if self.atlas_size != (0, 0) {
            self.atlas_size
        } else {
            let (cw, ch) = self.cell_size;
            (cw * 4, ch * 4)
        };
        if aw == 0 || ah == 0 {
            return None;
        }
        Some(egui::Rect::from_min_max(
            egui::pos2(pixel.min.x / aw as f32, pixel.min.y / ah as f32),
            egui::pos2(pixel.max.x / aw as f32, pixel.max.y / ah as f32),
        ))
    }

    #[must_use]
    pub fn all_ids_registered(&self) -> bool {
        PowerHudIconId::inventory()
            .iter()
            .all(|id| self.icons.contains_key(id.as_str()))
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PowerHudEguiTextureCache {
    handle: Option<Handle<Image>>,
    texture_id: Option<egui::TextureId>,
}

impl PowerHudEguiTextureCache {
    pub fn resolve(&mut self, contexts: &mut EguiContexts, handle: &Handle<Image>) -> Option<egui::TextureId> {
        if *handle == Handle::default() {
            self.handle = None;
            self.texture_id = None;
            return None;
        }
        if let (Some(cached), Some(tex_id)) = (&self.handle, self.texture_id) {
            if cached == handle {
                return Some(tex_id);
            }
        }
        let tex_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));
        self.handle = Some(handle.clone());
        self.texture_id = Some(tex_id);
        Some(tex_id)
    }
}

/// Loaded power HUD atlas handles (Startup).
#[derive(Resource, Debug, Clone)]
pub struct PowerHudIconAtlasUi {
    pub atlas: Handle<Image>,
    pub manifest: Handle<PowerHudIconAtlasManifest>,
}

impl PowerHudIconAtlasUi {
    #[must_use]
    pub fn manifest_loaded(&self, manifests: &Assets<PowerHudIconAtlasManifest>) -> bool {
        manifests.get(&self.manifest).is_some()
    }

    #[must_use]
    pub fn image_node_for_id(
        &self,
        manifests: &Assets<PowerHudIconAtlasManifest>,
        id: PowerHudIconId,
    ) -> Option<bevy::ui::widget::ImageNode> {
        let manifest = manifests.get(&self.manifest)?;
        let rect = manifest.cell_rect(id)?;
        Some(
            bevy::ui::widget::ImageNode::new(self.atlas.clone())
                .with_rect(rect)
                .with_mode(bevy::ui::widget::NodeImageMode::Auto)
                .with_color(Color::WHITE),
        )
    }
}

#[derive(Default, TypePath)]
pub struct PowerHudIconAtlasManifestLoader;

#[derive(Debug)]
pub enum PowerHudIconAtlasManifestLoaderError {
    Io(std::io::Error),
    Ron(String),
}

impl fmt::Display for PowerHudIconAtlasManifestLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Ron(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PowerHudIconAtlasManifestLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Ron(_) => None,
        }
    }
}

impl From<std::io::Error> for PowerHudIconAtlasManifestLoaderError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl AssetLoader for PowerHudIconAtlasManifestLoader {
    type Asset = PowerHudIconAtlasManifest;
    type Settings = ();
    type Error = PowerHudIconAtlasManifestLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let s = std::str::from_utf8(&bytes)
            .map_err(|e| PowerHudIconAtlasManifestLoaderError::Ron(e.to_string()))?;
        ron::from_str(s).map_err(|e| PowerHudIconAtlasManifestLoaderError::Ron(format!("RON: {e}")))
    }

    fn extensions(&self) -> &[&str] {
        &["power_hud.ron", "manifest.ron"]
    }
}

pub fn load_power_hud_icon_atlas_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PowerHudIconAtlasUi {
        atlas: asset_server.load(POWER_HUD_ATLAS_TEXTURE_PATH),
        manifest: asset_server.load(POWER_HUD_ATLAS_MANIFEST_PATH),
    });
}

pub struct PowerHudIconAtlasPlugin;

impl Plugin for PowerHudIconAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PowerHudIconAtlasManifest>()
            .register_asset_loader(PowerHudIconAtlasManifestLoader)
            .init_resource::<PowerHudEguiTextureCache>()
            .add_systems(Startup, load_power_hud_icon_atlas_system);
    }
}

/// Draw atlas icon in egui with adjacent label (a11y — never icon-only controls).
pub fn draw_power_hud_icon_labeled(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    manifest: &PowerHudIconAtlasManifest,
    id: PowerHudIconId,
    size: f32,
    tint: egui::Color32,
    label: &str,
    selected: bool,
) -> egui::Response {
    ui.horizontal(|ui| {
        if let Some(uv) = manifest.egui_uv(id) {
            let stroke = if selected {
                egui::Stroke::new(1.5, tint)
            } else {
                egui::Stroke::new(1.0, tint.gamma_multiply(0.55))
            };
            let img = egui::Image::new((texture_id, egui::vec2(size, size)))
                .uv(uv)
                .tint(tint);
            let resp = ui.add(img);
            ui.painter().rect_stroke(
                resp.rect.expand(1.0),
                2.0,
                stroke,
                egui::StrokeKind::Outside,
            );
        }
        ui.label(label)
    })
    .inner
}

/// Horizontal gauge row: icon + fill bar (plant card).
pub fn draw_power_hud_gauge_row(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    manifest: &PowerHudIconAtlasManifest,
    icon: PowerHudIconId,
    label: &str,
    fill: f32,
    fill_color: egui::Color32,
    palette_track: egui::Color32,
) {
    let fill = fill.clamp(0.0, 1.0);
    ui.horizontal(|ui| {
        if let Some(uv) = manifest.egui_uv(icon) {
            ui.add(
                egui::Image::new((texture_id, egui::vec2(16.0, 16.0)))
                    .uv(uv)
                    .tint(fill_color),
            );
        }
        ui.label(label);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 10.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 2.0, palette_track);
        let mut fill_rect = rect;
        fill_rect.set_width(rect.width() * fill);
        ui.painter().rect_filled(fill_rect, 2.0, fill_color);
    });
}

#[must_use]
pub fn power_hud_atlas_assets_on_disk() -> bool {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    root.join("textures/ui/power_hud_atlas.png").is_file()
        && root.join("configs/ui/power_hud.manifest.ron").is_file()
}

#[must_use]
pub fn power_hud_icon_atlas_registration_witness_green() -> bool {
    let manifest: PowerHudIconAtlasManifest = ron::from_str(include_str!(
        "../../../assets/configs/ui/power_hud.manifest.ron"
    ))
    .expect("power hud manifest");
    power_hud_atlas_assets_on_disk()
        && manifest.all_ids_registered()
        && manifest.cell_size == (20, 20)
        && manifest.chip_size == (16, 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_hud_atlas_png_on_disk() {
        assert!(power_hud_atlas_assets_on_disk());
    }

    #[test]
    fn manifest_registers_all_inventory_ids() {
        assert!(power_hud_icon_atlas_registration_witness_green());
    }

    #[test]
    fn voltage_and_routing_icon_maps() {
        assert_eq!(
            PowerHudIconId::for_voltage(VoltageClass::Medium),
            PowerHudIconId::VoltageMedium
        );
        assert_eq!(
            PowerHudIconId::for_routing_mode(PowerLineRoutingMode::Orthogonal90),
            PowerHudIconId::Route90
        );
    }

    #[test]
    fn egui_uv_normalized() {
        let manifest: PowerHudIconAtlasManifest = ron::from_str(
            r#"
(
  schema_version: 1,
  texture: "textures/ui/power_hud_atlas.png",
  cell_size: (20, 20),
  atlas_size: (80, 80),
  icons: { "PWR_LINE": (col: 0, row: 0) },
)
"#,
        )
        .expect("ron");
        let uv = manifest.egui_uv(PowerHudIconId::PowerLineTool).expect("uv");
        assert_eq!(uv.min, egui::pos2(0.0, 0.0));
        assert_eq!(uv.max, egui::pos2(0.25, 0.25));
    }
}
