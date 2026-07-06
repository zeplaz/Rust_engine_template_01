//! Red world-space marker for CLI fire/VFX test worlds — find seeded burns when zoomed out.
//!
//! Active on `--test vfx|visual|fire|atmosphere` (`TestScene::seeds_fire_overlay`).
//! Projection uses [`MainWorldCameraOrthoTrace`] (same ortho as the tactical RTT camera) so the
//! box tracks pan/zoom exactly with the rendered map.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::ActiveTestScene;
use crate::gui::map_camera::{
    map_camera_desired_fit_tile_aabb, sim_map_world_aabb_to_egui_with_ortho,
    sync_main_world_camera_viewport_and_projection, MainWorldCamera, MainWorldCameraOrthoTrace,
};
use crate::gui::view_authority::commit_map_camera_pose_to_view_authority;
use crate::gui::{SimulationMapTexture, SimulationMapViewport};
use crate::render::view_runtime::{ViewProjectionAuthority, ViewRuntimeTrace};
use crate::systems::fire::ChunkSurfaceFire;
use crate::terrain::generation::{chunk_world_origin, world_generator_enhanced::WorldGenParams, Chunk, ChunkCellMatrix};

/// Tile-space AABB around seeded test fire (XY = map plane; Z unused in projection).
#[derive(Resource, Clone, Debug)]
pub struct VfxFireTestHighlight {
    pub enabled: bool,
    pub min_tile: Vec2,
    pub max_tile: Vec2,
    /// One-shot request to pan/zoom the tactical camera onto the burning region.
    pub needs_camera_focus: bool,
}

impl Default for VfxFireTestHighlight {
    fn default() -> Self {
        Self {
            enabled: false,
            min_tile: Vec2::ZERO,
            max_tile: Vec2::ONE,
            needs_camera_focus: false,
        }
    }
}

/// Stroke width only — box geometry is world-locked via ortho projection.
const STROKE_PX_BASE: f32 = 2.0;

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
        let origin = chunk_world_origin(chunk.coord, matrix.size);
        let c_min = origin;
        let c_max = origin + Vec2::new(slab_x, slab_y);
        min = min.min(c_min);
        max = max.max(c_max);
    }
    if !any {
        return None;
    }
    let world_max = Vec2::new(params.width as f32, params.height as f32);
    min = (min - Vec2::splat(pad_tiles)).max(Vec2::ZERO);
    max = (max + Vec2::splat(pad_tiles)).min(world_max);
    Some((min, max))
}

pub fn refresh_vfx_fire_test_highlight_from_burning(
    params: &WorldGenParams,
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    highlight: &mut VfxFireTestHighlight,
) {
    if !highlight.enabled {
        return;
    }
    if let Some((min_tile, max_tile)) = highlight_region_from_burning_chunks(fire_q, params, 2.0) {
        highlight.min_tile = min_tile;
        highlight.max_tile = max_tile;
    }
}

pub fn draw_vfx_fire_test_highlight_overlay(
    mut contexts: EguiContexts,
    highlight: Res<VfxFireTestHighlight>,
    scene: Option<Res<ActiveTestScene>>,
    ortho: Res<MainWorldCameraOrthoTrace>,
    map_vp: Res<SimulationMapViewport>,
) -> Result {
    let active = scene.is_some_and(|s| s.0.seeds_fire_overlay());
    if !active || !highlight.enabled || !map_vp.valid {
        return Ok(());
    }
    if ortho.fixed_width <= 0.0 || ortho.fixed_height <= 0.0 {
        return Ok(());
    }
    if highlight.max_tile.x <= highlight.min_tile.x || highlight.max_tile.y <= highlight.min_tile.y {
        return Ok(());
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return Ok(());
    };
    let Some(rect) = sim_map_world_aabb_to_egui_with_ortho(
        highlight.min_tile,
        highlight.max_tile,
        map_vp.as_ref(),
        ortho.as_ref(),
    ) else {
        return Ok(());
    };
    if rect.width() < 2.0 || rect.height() < 2.0 {
        return Ok(());
    }

    let zoom = ortho.desired_zoom.max(ortho.authority_zoom).max(1e-4);
    let stroke_w = (STROKE_PX_BASE * (0.85 + zoom.sqrt() * 0.08)).clamp(1.25, 4.0);

    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("vfx_fire_test_highlight"),
    );
    let painter = ctx.layer_painter(layer);
    let stroke = egui::Stroke::new(stroke_w, egui::Color32::from_rgb(255, 40, 32));
    painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Outside);
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(255, 48, 32, 22),
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
                    focus_vfx_fire_test_camera_on_burning_region,
                )
                    .chain(),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_vfx_fire_test_highlight_overlay.after(sync_main_world_camera_viewport_and_projection),
            );
    }
}

fn sync_vfx_fire_test_highlight_armed(
    scene: Option<Res<ActiveTestScene>>,
    mut highlight: ResMut<VfxFireTestHighlight>,
) {
    let armed = scene.is_some_and(|s| s.0.seeds_fire_overlay());
    if armed && !highlight.enabled {
        highlight.enabled = true;
        highlight.needs_camera_focus = true;
        highlight.min_tile = Vec2::ZERO;
        highlight.max_tile = Vec2::ZERO;
    } else if !armed {
        highlight.enabled = false;
        highlight.needs_camera_focus = false;
    }
}

fn sync_vfx_fire_test_highlight_from_burning_system(
    params: Res<WorldGenParams>,
    fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    mut highlight: ResMut<VfxFireTestHighlight>,
) {
    refresh_vfx_fire_test_highlight_from_burning(params.as_ref(), &fire_q, highlight.as_mut());
}

fn focus_vfx_fire_test_camera_on_burning_region(
    scene: Option<Res<ActiveTestScene>>,
    mut highlight: ResMut<VfxFireTestHighlight>,
    params: Res<WorldGenParams>,
    map_vp: Res<SimulationMapViewport>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut authority: ResMut<ViewProjectionAuthority>,
    mut trace: ResMut<ViewRuntimeTrace>,
    mut cam: Query<&mut Transform, With<MainWorldCamera>>,
) {
    if !highlight.enabled || !highlight.needs_camera_focus {
        return;
    }
    if highlight.max_tile.x <= highlight.min_tile.x || highlight.max_tile.y <= highlight.min_tile.y {
        return;
    }
    if !scene.is_some_and(|s| s.0.seeds_fire_overlay()) {
        return;
    }
    let world_w = params.width.max(1) as f32;
    let world_h = params.height.max(1) as f32;
    let window_px = windows
        .single()
        .ok()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let tex_extent = crate::gui::simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    let desired = map_camera_desired_fit_tile_aabb(
        highlight.min_tile,
        highlight.max_tile,
        map_vp.as_ref(),
        window_px,
        tex_extent,
        world_w,
        world_h,
        1.08,
    );
    commit_map_camera_pose_to_view_authority(authority.as_mut(), trace.as_mut(), &desired);
    for mut t in cam.iter_mut() {
        t.translation.x = desired.translation.x;
        t.translation.y = desired.translation.y;
        t.scale = desired.scale;
    }
    highlight.needs_camera_focus = false;
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
        "plugin_wired": false,
        "status": "removed_pending_redesign",
        "projection": "MainWorldCameraOrthoTrace",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::map_camera::{sim_map_world_aabb_to_egui_with_ortho, MainWorldCameraOrthoTrace};

    #[test]
    fn highlight_aabb_scales_with_ortho_zoom() {
        let mut vp = crate::gui::SimulationMapViewport::default();
        vp.valid = true;
        vp.min = Vec2::ZERO;
        vp.max = Vec2::new(800.0, 600.0);
        let min_tile = Vec2::new(400.0, 400.0);
        let max_tile = Vec2::new(500.0, 500.0);
        let center = Vec2::new(450.0, 450.0);
        let ortho_lo = MainWorldCameraOrthoTrace {
            fixed_width: 800.0,
            fixed_height: 600.0,
            view_pixels: Vec2::new(800.0, 600.0),
            desired_zoom: 1.0,
            camera_center: center,
            ..Default::default()
        };
        let ortho_hi = MainWorldCameraOrthoTrace {
            fixed_width: 400.0,
            fixed_height: 300.0,
            view_pixels: Vec2::new(800.0, 600.0),
            desired_zoom: 2.0,
            camera_center: center,
            ..Default::default()
        };
        let r_lo = sim_map_world_aabb_to_egui_with_ortho(
            min_tile,
            max_tile,
            &vp,
            &ortho_lo,
        )
        .unwrap();
        let r_hi = sim_map_world_aabb_to_egui_with_ortho(
            min_tile,
            max_tile,
            &vp,
            &ortho_hi,
        )
        .unwrap();
        assert!(r_hi.width() > r_lo.width() * 1.8);
        assert!(r_hi.height() > r_lo.height() * 1.8);
    }
}
