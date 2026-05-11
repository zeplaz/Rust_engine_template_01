//! Main egui window: editor layout + clipped painter viewport.

use super::interaction::{apply_pan, apply_zoom_toward, update_hover_tile};
use super::layers::PreviewLayers;
use super::minimap::world_preview_minimap;
use super::texture_cache::WorldPreviewTexture;
use super::ui_sidebar::world_preview_sidebar;
use super::ui_statusbar::world_preview_status_bar;
use super::ui_toolbar::world_preview_toolbar;
use super::viewport::EditorViewport;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::style::neutral_image_tint;
use crate::systems::terrain::TerrainRegistriesHandles;

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};

pub fn display_world_preview(
    mut contexts: EguiContexts,
    preview_texture: Res<WorldPreviewTexture>,
    mut world_preview_ui: ResMut<super::WorldPreviewUiState>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut world_gen_params: ResMut<crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
    mut viewport: ResMut<EditorViewport>,
    handles: Res<TerrainRegistriesHandles>,
    tag_assets: Res<Assets<crate::terrain::material::TagRegistry>>,
    mobility_assets: Res<Assets<crate::terrain::mobility::MobilityProfileRegistry>>,
    mut last_tex: Local<(u32, u32)>,
) -> Result {
    if !world_preview_ui.window_open {
        return Ok(());
    }

    let texture_id = contexts.add_image(EguiTextureHandle::Strong(preview_texture.texture.clone()));
    let tex_w = preview_texture.width as f32;
    let tex_h = preview_texture.height as f32;

    if *last_tex != (preview_texture.width, preview_texture.height) {
        *last_tex = (preview_texture.width, preview_texture.height);
        viewport.reset_camera_for_map(tex_w, tex_h);
    }

    let mut window_open = world_preview_ui.window_open;
    egui::Window::new("World Preview")
        .resizable(true)
        .default_size([920.0, 640.0])
        .open(&mut window_open)
        .show(contexts.ctx_mut()?, |ui| {
            ui.vertical(|ui| {
                world_preview_toolbar(
                    ui,
                    &mut world_gen_ui_state.preview_layers,
                    &mut viewport,
                    preview_texture.width,
                    preview_texture.height,
                );

                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(248.0);
                        world_preview_sidebar(
                            ui,
                            &mut world_gen_ui_state,
                            &mut world_gen_params,
                            &handles,
                            &tag_assets,
                            &mobility_assets,
                        );
                        ui.separator();
                        world_preview_minimap(ui, texture_id, preview_texture.width, preview_texture.height);
                    });

                    ui.vertical(|ui| {
                        let sz = ui.available_size();
                        let view_h = sz.y.max(200.0);
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(sz.x, view_h),
                            egui::Sense::click_and_drag(),
                        );

                        let z = viewport
                            .zoom
                            .clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
                        viewport.zoom = z;
                        let center = rect.center();

                        if response.hovered() {
                            let zoom_mod =
                                ui.ctx().input(|i| i.modifiers.ctrl || i.modifiers.command);
                            let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                            if zoom_mod && scroll != 0.0 {
                                let factor = 1.0 + scroll * 0.002;
                                if let Some(p) = ui.ctx().pointer_hover_pos() {
                                    apply_zoom_toward(&mut viewport, p, center, factor);
                                }
                            }
                        }

                        if response.dragged_by(egui::PointerButton::Middle) {
                            let d = response.drag_delta();
                            apply_pan(&mut viewport, Vec2::new(d.x, d.y));
                        }

                        update_hover_tile(
                            &mut viewport,
                            ui.ctx().pointer_hover_pos(),
                            rect,
                            center,
                            preview_texture.width,
                            preview_texture.height,
                        );

                        let rect_min = egui::pos2(
                            center.x - viewport.camera_center.x * z,
                            center.y - viewport.camera_center.y * z,
                        );
                        let rect_max = rect_min + egui::vec2(tex_w * z, tex_h * z);
                        let map_rect = egui::Rect::from_min_max(rect_min, rect_max);
                        let painter = ui.painter().with_clip_rect(rect);
                        painter.image(
                            texture_id,
                            map_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            neutral_image_tint(),
                        );
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
                            egui::StrokeKind::Inside,
                        );
                    });
                });

                ui.separator();
                world_preview_status_bar(
                    ui,
                    world_gen_ui_state.preview_layers,
                    &viewport,
                    preview_texture.width,
                    preview_texture.height,
                );
            });
        });

    world_preview_ui.window_open = window_open;
    Ok(())
}
