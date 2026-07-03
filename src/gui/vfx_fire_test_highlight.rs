//! Red world-space marker for CLI fire/VFX test worlds — find seeded burns when zoomed out.
//!
//! Active on `--test vfx|visual|fire|atmosphere` (`TestScene::seeds_fire_overlay`).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::ActiveTestScene;
use crate::gui::map_camera::{
    map_camera_viewport_pixels, map_zoom_alpha_with_limits,
    map_zoom_limits_for_world, MainWorldCameraViewportLatch, MapCameraDesired,
    MapCameraDesiredRes, sim_map_world_xy_to_egui_with_window,
};
use crate::gui::view_authority::map_camera_desired_from_view_authority;
use crate::gui::SimulationMapViewport;
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::systems::fire::ChunkSurfaceFire;
use crate::terrain::generation::{world_generator_enhanced::WorldGenParams, Chunk, ChunkCellMatrix};

/// Tile-space AABB around seeded test fire (XY = map plane; Z unused in projection).
#[derive(Resource, Clone, Debug)]
pub struct VfxFireTestHighlight {
    pub enabled: bool,
    pub min_tile: Vec2,
    pub max_tile: Vec2,
}

impl Default for VfxFireTestHighlight {
    fn default() -> Self {
        Self {
            enabled: false,
            min_tile: Vec2::ZERO,
            max_tile: Vec2::ONE,
        }
    }
}

/// Minimum screen extent only when zoomed out (keeps marker findable without freezing size when zoomed in).
const MIN_SCREEN_WHEN_ZOOMED_OUT_PX: f32 = 48.0;
const ZOOMED_OUT_ALPHA: f32 = 0.35;
const STROKE_PX_BASE: f32 = 2.0;

#[must_use]
pub fn highlight_region_from_world_center(params: &WorldGenParams) -> (Vec2, Vec2) {
    let center = Vec2::new(params.width as f32 * 0.5, params.height as f32 * 0.5);
    let half = (params.width.max(params.height) as f32 * 0.18).max(72.0);
    let max = Vec2::new(params.width as f32, params.height as f32);
    (
        (center - Vec2::splat(half)).max(Vec2::ZERO),
        (center + Vec2::splat(half)).min(max),
    )
}

#[must_use]
pub fn highlight_region_from_burning_chunks(
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    params: &WorldGenParams,
    pad_tiles: f32,
) -> Option<(Vec2, Vec2)> {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut any = false;
    for (_, chunk, matrix, fire) in fire_q.iter() {
        if fire.heat <= 0.02 {
            continue;
        }
        any = true;
        let slab_x = matrix.size.x.max(1) as f32;
        let slab_y = matrix.size.y.max(1) as f32;
        let c_min = Vec2::new(chunk.coord.x as f32 * slab_x, chunk.coord.y as f32 * slab_y);
        let c_max = c_min + Vec2::new(slab_x, slab_y);
        min = min.min(c_min);
        max = max.max(c_max);
    }
    if !any {
        return None;
    }
    let world_max = Vec2::new(params.width as f32, params.height as f32);
    min = (min - Vec2::splat(pad_tiles)).max(Vec2::ZERO);
    max = (max + Vec2::splat(pad_tiles)).min(world_max);
    let size = max - min;
    let min_span = 96.0;
    if size.x < min_span || size.y < min_span {
        let center = (min + max) * 0.5;
        let half = Vec2::new(size.x.max(min_span), size.y.max(min_span)) * 0.5;
        min = (center - half).max(Vec2::ZERO);
        max = (center + half).min(world_max);
    }
    Some((min, max))
}

pub fn arm_vfx_fire_test_highlight_from_world_center(
    params: &WorldGenParams,
    highlight: &mut VfxFireTestHighlight,
) {
    let (min_tile, max_tile) = highlight_region_from_world_center(params);
    highlight.enabled = true;
    highlight.min_tile = min_tile;
    highlight.max_tile = max_tile;
}

pub fn refresh_vfx_fire_test_highlight_from_burning(
    params: &WorldGenParams,
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    highlight: &mut VfxFireTestHighlight,
) {
    if !highlight.enabled {
        return;
    }
    if let Some((min_tile, max_tile)) = highlight_region_from_burning_chunks(fire_q, params, 48.0) {
        highlight.min_tile = min_tile;
        highlight.max_tile = max_tile;
    }
}

fn project_tile_aabb_to_egui(
    min_tile: Vec2,
    max_tile: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
    latch: &MainWorldCameraViewportLatch,
) -> Option<egui::Rect> {
    let corners = [
        Vec2::new(min_tile.x, min_tile.y),
        Vec2::new(max_tile.x, min_tile.y),
        Vec2::new(max_tile.x, max_tile.y),
        Vec2::new(min_tile.x, max_tile.y),
    ];
    let mut min_s = egui::pos2(f32::INFINITY, f32::INFINITY);
    let mut max_s = egui::pos2(f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut projected = 0u32;
    for world_xy in corners {
        let Some(p) = sim_map_world_xy_to_egui_with_window(
            world_xy,
            desired,
            map_vp,
            world_w,
            world_h,
            None,
            Some(latch),
        ) else {
            continue;
        };
        min_s.x = min_s.x.min(p.x);
        min_s.y = min_s.y.min(p.y);
        max_s.x = max_s.x.max(p.x);
        max_s.y = max_s.y.max(p.y);
        projected += 1;
    }
    if projected < 2 {
        return None;
    }
    Some(egui::Rect::from_min_max(min_s, max_s))
}

fn expand_rect_min_screen_size(rect: egui::Rect, min_side: f32) -> egui::Rect {
    let size = rect.size();
    if size.x >= min_side && size.y >= min_side {
        return rect;
    }
    let center = rect.center();
    egui::Rect::from_center_size(
        center,
        egui::vec2(size.x.max(min_side), size.y.max(min_side)),
    )
}

pub fn draw_vfx_fire_test_highlight_overlay(
    mut contexts: EguiContexts,
    highlight: Res<VfxFireTestHighlight>,
    scene: Option<Res<ActiveTestScene>>,
    desired: Res<MapCameraDesiredRes>,
    authority: Option<Res<ViewProjectionAuthority>>,
    map_vp: Res<SimulationMapViewport>,
    latch: Res<MainWorldCameraViewportLatch>,
    params: Res<WorldGenParams>,
) -> Result {
    let active = scene.is_some_and(|s| s.0.seeds_fire_overlay());
    if !active || !highlight.enabled {
        return Ok(());
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return Ok(());
    };
    let world_w = params.width.max(1) as f32;
    let world_h = params.height.max(1) as f32;
    let window_px = if map_vp.window_logical.x > 1.0 {
        map_vp.window_logical
    } else {
        Vec2::new(1280.0, 720.0)
    };
    let viewport = map_camera_viewport_pixels(window_px, Some(map_vp.as_ref()));
    let (zoom_lo, zoom_hi) = map_zoom_limits_for_world(world_w, world_h, viewport);
    let camera_desired = if let Some(auth) = authority.as_ref() {
        map_camera_desired_from_view_authority(auth)
    } else {
        (**desired).clone()
    };
    let Some(mut rect) = project_tile_aabb_to_egui(
        highlight.min_tile,
        highlight.max_tile,
        &camera_desired,
        map_vp.as_ref(),
        world_w,
        world_h,
        latch.as_ref(),
    ) else {
        return Ok(());
    };
    let zoom_alpha = map_zoom_alpha_with_limits(camera_desired.scale.x, zoom_lo, zoom_hi);
    if zoom_alpha < ZOOMED_OUT_ALPHA {
        rect = expand_rect_min_screen_size(rect, MIN_SCREEN_WHEN_ZOOMED_OUT_PX);
    }
    if map_vp.valid {
        let vp = egui::Rect::from_min_max(
            egui::pos2(map_vp.min.x, map_vp.min.y),
            egui::pos2(map_vp.max.x, map_vp.max.y),
        );
        rect = rect.intersect(vp);
    }
    if rect.width() < 4.0 || rect.height() < 4.0 {
        return Ok(());
    }

    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("vfx_fire_test_highlight"),
    );
    let painter = ctx.layer_painter(layer);
    let stroke_w = (STROKE_PX_BASE * (0.65 + zoom_alpha * 0.6)).clamp(1.0, 4.0);
    let stroke = egui::Stroke::new(stroke_w, egui::Color32::from_rgb(255, 40, 32));
    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Outside);
    painter.rect_filled(
        rect.expand(1.0),
        0.0,
        egui::Color32::from_rgba_unmultiplied(255, 48, 32, 28),
    );
    painter.text(
        rect.left_top() + egui::vec2(6.0, 4.0),
        egui::Align2::LEFT_TOP,
        "FIRE / SPARKS TEST",
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(255, 90, 80),
    );
    Ok(())
}

pub struct VfxFireTestHighlightPlugin;

impl Plugin for VfxFireTestHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VfxFireTestHighlight>()
            .add_systems(
                Update,
                (
                    sync_vfx_fire_test_highlight_armed,
                    sync_vfx_fire_test_highlight_from_burning_system,
                )
                    .chain(),
            )
            .add_systems(EguiPrimaryContextPass, draw_vfx_fire_test_highlight_overlay);
    }
}

fn sync_vfx_fire_test_highlight_armed(
    scene: Option<Res<ActiveTestScene>>,
    params: Res<WorldGenParams>,
    mut highlight: ResMut<VfxFireTestHighlight>,
) {
    let armed = scene.is_some_and(|s| s.0.seeds_fire_overlay());
    if armed && !highlight.enabled {
        arm_vfx_fire_test_highlight_from_world_center(params.as_ref(), highlight.as_mut());
    } else if !armed {
        highlight.enabled = false;
    }
}

fn sync_vfx_fire_test_highlight_from_burning_system(
    params: Res<WorldGenParams>,
    fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    mut highlight: ResMut<VfxFireTestHighlight>,
) {
    refresh_vfx_fire_test_highlight_from_burning(params.as_ref(), &fire_q, highlight.as_mut());
}

#[must_use]
pub fn vfx_fire_test_highlight_001_witness_green() -> bool {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/gui/vfx_fire_test_highlight.rs")
        .is_file()
}

#[must_use]
pub fn vfx_fire_test_highlight_001_witness_json() -> serde_json::Value {
    serde_json::json!({
        "gate": "VFX-FIRE-HIGHLIGHT-001",
        "green": vfx_fire_test_highlight_001_witness_green(),
        "module": "src/gui/vfx_fire_test_highlight.rs",
        "plugin_wired": true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_from_world_center_spans_reasonable_fraction() {
        let params = WorldGenParams {
            width: 320,
            height: 320,
            ..Default::default()
        };
        let (min, max) = highlight_region_from_world_center(&params);
        assert!(max.x - min.x >= 144.0);
        assert!(max.y - min.y >= 144.0);
        let center = (min + max) * 0.5;
        assert!((center.x - 160.0).abs() < 1.0);
        assert!((center.y - 160.0).abs() < 1.0);
    }

    #[test]
    fn expand_rect_enforces_min_screen_size_when_zoomed_out() {
        let tiny = egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(12.0, 8.0));
        let big = expand_rect_min_screen_size(tiny, MIN_SCREEN_WHEN_ZOOMED_OUT_PX);
        assert!(big.width() >= MIN_SCREEN_WHEN_ZOOMED_OUT_PX);
        assert!(big.height() >= MIN_SCREEN_WHEN_ZOOMED_OUT_PX);
    }
}
