//! Main egui window: editor layout + clipped painter viewport.

use super::interaction::{apply_pan, apply_zoom_toward, update_hover_tile};
use super::layers::PreviewLayers;
use super::minimap::world_preview_minimap;
use super::preview_render_contract::PreviewCameraState;
use super::texture_cache::WorldPreviewTexture;
use super::ui_sidebar::world_preview_sidebar;
use super::ui_statusbar::world_preview_status_bar;
use super::ui_toolbar::world_preview_toolbar;
use super::viewport::EditorViewport;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::std_floating;
use crate::gui::style::{neutral_image_tint, UiPalette, UiSpacing};
use crate::systems::terrain::TerrainRegistriesHandles;

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};

pub fn display_world_preview(
    mut contexts: EguiContexts,
    preview_texture: Res<WorldPreviewTexture>,
    mut preview_cam: ResMut<PreviewCameraState>,
    mut world_preview_ui: ResMut<super::WorldPreviewUiState>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut world_gen_params: ResMut<crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
    mut viewport: ResMut<EditorViewport>,
    handles: Res<TerrainRegistriesHandles>,
    tag_assets: Res<Assets<crate::terrain::material::TagRegistry>>,
    mobility_assets: Res<Assets<crate::terrain::mobility::MobilityProfileRegistry>>,
    mut last_tex: Local<(u32, u32)>,
    mut egui_tex_cache: Local<Option<(Handle<Image>, egui::TextureId)>>,
    palette: Res<UiPalette>,
    spacing: Res<UiSpacing>,
) -> Result {
    if !world_preview_ui.window_open {
        return Ok(());
    }

    let display_handle = preview_texture.texture.clone();
    let disp_w = preview_texture.width;
    let disp_h = preview_texture.height;

    let texture_id = match egui_tex_cache.as_ref() {
        Some((h, id)) if *h == display_handle => *id,
        _ => {
            let id = contexts.add_image(EguiTextureHandle::Strong(display_handle.clone()));
            *egui_tex_cache = Some((display_handle, id));
            id
        }
    };
    let tex_w = disp_w as f32;
    let tex_h = disp_h as f32;

    if *last_tex != (disp_w, disp_h) {
        *last_tex = (disp_w, disp_h);
        viewport.reset_camera_for_map(tex_w, tex_h);
    }

    let mut window_open = world_preview_ui.window_open;
    let panel_frame = egui::Frame::NONE;
    std_floating(egui::Window::new("World Preview"))
        .id(egui::Id::new("world_preview_main"))
        .default_size([960.0, 680.0])
        .min_size([480.0, 340.0])
        .open(&mut window_open)
        .show(contexts.ctx_mut()?, |ui| {
            let pal: &UiPalette = &*palette;
            let sp: &UiSpacing = &*spacing;
            // Top / bottom / central panels fill the window inner rect so the map track
            // tracks resize smoothly (plain `vertical` + `horizontal` only shrink-wrap height).
            egui::TopBottomPanel::top("world_preview_toolbar")
                .frame(panel_frame)
                .resizable(false)
                .show_inside(ui, |ui| {
                    world_preview_toolbar(
                        ui,
                        &mut world_gen_ui_state.preview_layers,
                        &mut viewport,
                        &mut preview_cam,
                        disp_w,
                        disp_h,
                        pal,
                    );
                });

            egui::TopBottomPanel::bottom("world_preview_status")
                .frame(panel_frame)
                .resizable(false)
                .show_inside(ui, |ui| {
                    world_preview_status_bar(
                        ui,
                        world_gen_ui_state.preview_layers,
                        &viewport,
                        disp_w,
                        disp_h,
                        pal,
                    );
                });

            egui::CentralPanel::default().frame(panel_frame).show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(248.0);
                        egui::ScrollArea::vertical()
                            .id_salt("world_preview_sidebar_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                world_preview_sidebar(
                                    ui,
                                    &mut world_gen_ui_state,
                                    &mut world_gen_params,
                                    &handles,
                                    &tag_assets,
                                    &mobility_assets,
                                    pal,
                                    sp,
                                );
                                ui.separator();
                                world_preview_minimap(
                                    ui,
                                    texture_id,
                                    disp_w,
                                    disp_h,
                                    pal,
                                );
                            });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        let map_col_w = ui.available_width();
                        ui.set_min_width(map_col_w.max(320.0));
                        let size = ui.available_size();
                        let (rect, response) =
                            ui.allocate_exact_size(size, egui::Sense::click_and_drag());

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
                            disp_w,
                            disp_h,
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
            });
        });

    world_preview_ui.window_open = window_open;
    Ok(())
}
