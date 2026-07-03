//! Simulation tactical map render-to-texture — camera → GPU image → Bevy UI [`ImageNode`].
//!
//! Replaces the legacy hole/scissor/latch pipeline ([`SimulationMapFillRect`] is the sole
//! screen-space rect for picks; no [`MainWorldCameraViewportLatch`]).

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{CameraOutputMode, ClearColorConfig, ImageRenderTarget, RenderTarget};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, IsDefaultUiCamera, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::gui::style::UiPalette;

use crate::gui::authoritative_viewport::{
    measure_sim_map_fill_corners_crosscheck, sync_simulation_map_fill_debug_trace,
};
use crate::gui::hud::{ViewportRectSanity, VIEWPORT_SIM_MAP_SAFE_MIN_H, VIEWPORT_SIM_MAP_SAFE_MIN_W};
use crate::gui::in_game_hud::SimulationMapViewportFill;
use crate::gui::map_camera::MainWorldCamera;
use crate::gui::SimulationViewportSyncSet;

/// Committed GPU image handle for the tactical map (camera [`RenderTarget::Image`]).
#[derive(Resource, Clone, Debug)]
pub struct SimulationMapTexture(pub Handle<Image>);

/// Window-space AABB of the Bevy UI node that displays [`SimulationMapTexture`].
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationMapFillRect {
    pub valid: bool,
    pub min: Vec2,
    pub max: Vec2,
    /// Primary window logical size (cursor chrome / legacy pick helpers).
    pub window_logical: Vec2,
}

impl SimulationMapFillRect {
    #[inline]
    #[must_use]
    pub fn logical_size(&self) -> Vec2 {
        (self.max - self.min).max(Vec2::ZERO)
    }

    #[inline]
    #[must_use]
    pub fn is_adequate_for_camera(&self) -> bool {
        let s = self.logical_size();
        s.x >= VIEWPORT_SIM_MAP_SAFE_MIN_W && s.y >= VIEWPORT_SIM_MAP_SAFE_MIN_H
    }

    #[inline]
    #[must_use]
    pub fn contains_cursor(self, cursor: Vec2) -> bool {
        self.valid
            && cursor.x >= self.min.x
            && cursor.x <= self.max.x
            && cursor.y >= self.min.y
            && cursor.y <= self.max.y
    }
}

const RTT_MIN_EXTENT: u32 = 64;
const RTT_MAX_EXTENT: u32 = 4096;

/// World map sprites/meshes use this layer; only [`MainWorldCamera`] (RTT) includes it.
pub const SIMULATION_MAP_RTT_RENDER_LAYER: usize = 1;

/// Primary-window camera for Bevy UI chrome + map [`ImageNode`] (not world Core2d).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SimulationHudUiCamera;

#[must_use]
pub fn simulation_map_rtt_render_layers() -> RenderLayers {
    RenderLayers::layer(SIMULATION_MAP_RTT_RENDER_LAYER)
}

/// RTT + window chrome clear — cool gray-blue tactical void (not paper / not Bevy default gray).
#[must_use]
pub fn simulation_map_rtt_clear_color(palette: &UiPalette) -> Color {
    palette.bevy_sim_map_field_clear()
}

/// Pre-pass + upscaling write clear for [`MainWorldCamera`] / HUD UI camera.
pub fn apply_simulation_map_camera_clear(camera: &mut Camera, palette: &UiPalette) {
    let color = simulation_map_rtt_clear_color(palette);
    let cfg = ClearColorConfig::Custom(color);
    camera.clear_color = cfg;
    camera.output_mode = CameraOutputMode::Write {
        blend_state: None,
        clear_color: cfg,
    };
}

/// Spawns the window-target UI camera (required once [`MainWorldCamera`] renders to RTT only).
pub fn spawn_simulation_hud_ui_camera(commands: &mut Commands, palette: &UiPalette) {
    let mut camera = Camera {
        order: 1,
        ..default()
    };
    apply_simulation_map_camera_clear(&mut camera, palette);
    commands.spawn((
        SimulationHudUiCamera,
        Camera2d,
        IsDefaultUiCamera,
        RenderLayers::none(),
        camera,
    ));
}

#[must_use]
pub fn simulation_map_rtt_image(width: u32, height: u32) -> Image {
    let w = width.max(RTT_MIN_EXTENT);
    let h = height.max(RTT_MIN_EXTENT);
    // Must match Core2d overlay SDR pipelines (`CORE2D_OVERLAY_SDR_FORMAT` = Rgba8UnormSrgb).
    let mut image = Image::new_target_texture(
        w,
        h,
        crate::render::CORE2D_OVERLAY_SDR_FORMAT,
        None,
    );
    image.texture_descriptor.label = Some("SimulationMapRenderTarget");
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image
}

#[inline]
#[must_use]
pub fn simulation_map_texture_extent(tex: &SimulationMapTexture, images: &Assets<Image>) -> Vec2 {
    images
        .get(&tex.0)
        .map(|img| {
            Vec2::new(img.width() as f32, img.height() as f32).max(Vec2::splat(1.0))
        })
        .unwrap_or(Vec2::new(1920.0, 1080.0))
}

fn clamp_aabb_to_window(min: Vec2, max: Vec2, window: Vec2) -> (Vec2, Vec2) {
    let win = Vec2::new(window.x.max(1.0), window.y.max(1.0));
    let min = Vec2::new(min.x.clamp(0.0, win.x), min.y.clamp(0.0, win.y));
    let mut max = Vec2::new(max.x.clamp(0.0, win.x), max.y.clamp(0.0, win.y));
    if max.x < min.x {
        max.x = min.x;
    }
    if max.y < min.y {
        max.y = min.y;
    }
    (min, max)
}

/// Measure UI fill node → [`SimulationMapFillRect`]; resize RT when layout changes.
pub fn sync_simulation_map_fill_rect_system(
    q: Query<(&ComputedNode, &UiGlobalTransform), With<SimulationMapViewportFill>>,
    mut fill: ResMut<SimulationMapFillRect>,
    mut tex: ResMut<SimulationMapTexture>,
    mut images: ResMut<Assets<Image>>,
    mut cam_rt: Query<&mut RenderTarget, With<MainWorldCamera>>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut sanity: ResMut<ViewportRectSanity>,
    mut trace: ResMut<crate::gui::SimulationMapViewportTrace>,
    mut sim_dbg: ResMut<crate::gui::SimulationMapViewportDebug>,
) {
    let Ok(w) = win.single() else {
        fill.valid = false;
        return;
    };
    let window_logical = Vec2::new(w.width(), w.height());
    let Ok((node, xf)) = q.single() else {
        fill.valid = false;
        return;
    };
    if node.is_empty() {
        fill.valid = false;
        return;
    }
    let scale = w.scale_factor().max(1e-6);
    let (raw_min, raw_max) = measure_sim_map_fill_corners_crosscheck(node, xf, scale);
    let (clamped_min, clamped_max) = clamp_aabb_to_window(raw_min, raw_max, window_logical);
    let fallback = crate::gui::simulation_map_fallback_logical_extent(window_logical);
    let (min, max, valid) =
        sanity.inspect_simulation_map_aabb(clamped_min, clamped_max, fallback, None);
    fill.min = min;
    fill.max = max;
    fill.valid = valid;
    fill.window_logical = window_logical;
    sync_simulation_map_fill_debug_trace(fill.as_ref(), trace.as_mut(), sim_dbg.as_mut());

    let size = fill.logical_size();
    if !fill.is_adequate_for_camera() {
        return;
    }
    let want_w = (size.x * scale).round() as u32;
    let want_h = (size.y * scale).round() as u32;
    let want_w = want_w.clamp(RTT_MIN_EXTENT, RTT_MAX_EXTENT);
    let want_h = want_h.clamp(RTT_MIN_EXTENT, RTT_MAX_EXTENT);
    let cur = images.get(&tex.0);
    let need_resize = cur.is_none_or(|img| img.width() != want_w || img.height() != want_h);
    if need_resize {
        let new_img = simulation_map_rtt_image(want_w, want_h);
        let handle = images.add(new_img);
        tex.0 = handle.clone();
        for mut rt in cam_rt.iter_mut() {
            *rt = RenderTarget::Image(ImageRenderTarget {
                handle: handle.clone(),
                scale_factor: scale,
            });
        }
    }
}

pub struct SimulationMapRttPlugin;

/// Bind committed RT handle to the tactical map [`ImageNode`].
pub fn sync_simulation_map_image_node_system(
    tex: Res<SimulationMapTexture>,
    mut q: Query<&mut bevy::ui::widget::ImageNode, With<SimulationMapViewportFill>>,
) {
    if tex.0 == Handle::default() {
        return;
    }
    for mut node in q.iter_mut() {
        if node.image != tex.0 {
            *node = bevy::ui::widget::ImageNode::new(tex.0.clone());
        }
    }
}

impl Plugin for SimulationMapRttPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::gui::sim_map_rtt::SimulationMapFillRect>()
            .add_systems(
                PostUpdate,
                (
                    sync_simulation_map_fill_rect_system,
                    sync_simulation_map_image_node_system,
                )
                    .chain()
                    .in_set(SimulationViewportSyncSet::MeasureUiHole),
            );
    }
}

/// Bootstrap RT handle before first layout measure (camera spawn path).
pub fn insert_simulation_map_texture(
    images: &mut Assets<Image>,
    commands: &mut Commands,
    width: u32,
    height: u32,
) -> Handle<Image> {
    let image = simulation_map_rtt_image(width, height);
    let handle = images.add(image);
    commands.insert_resource(SimulationMapTexture(handle.clone()));
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtt_image_data_matches_descriptor_extent() {
        let img = simulation_map_rtt_image(1920, 1080);
        let bytes = img.data.as_ref().expect("target texture data");
        assert_eq!(bytes.len(), 4 * 1920 as usize * 1080 as usize);
        assert_eq!(img.width(), 1920);
        assert_eq!(img.height(), 1080);
    }
}
