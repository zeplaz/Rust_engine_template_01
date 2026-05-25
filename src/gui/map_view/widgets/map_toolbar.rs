//! Shared map toolbar for preview + minimap consumers.

use bevy_egui::egui;

use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::editor::world_preview::{PreviewCameraState, PreviewRenderMode};
use crate::gui::map_view::MapViewState as EditorViewport;
use crate::gui::map_view::MapViewState;
use crate::gui::map_presentation_controls::map_overlay_controls_ui;
use crate::gui::style::{muted_label, primary_label, section_heading, CmdHeadingStyle, UiPalette};
use crate::gui::{MinimapCameraBookmark, MinimapShellState};

#[derive(Clone, Copy, Debug, Default)]
pub struct MapToolbarConfig {
    pub show_follow: bool,
    pub show_bookmarks: bool,
    pub show_generation_tools: bool,
    pub show_render_mode: bool,
    pub show_zoom_reset: bool,
}

pub fn map_toolbar(
    ui: &mut egui::Ui,
    presentation: &mut MapViewState,
    palette: &UiPalette,
    id_prefix: &str,
    config: MapToolbarConfig,
) {
    map_overlay_controls_ui(ui, presentation, palette, id_prefix);
    if config.show_bookmarks && !presentation.bookmarks.is_empty() {
        ui.horizontal_wrapped(|ui| {
            for bookmark in &presentation.bookmarks {
                ui.label(format!(
                    "{} @ ({:.0},{:.0})",
                    bookmark.label, bookmark.world.x, bookmark.world.y
                ));
            }
        });
    }
    let _ = config.show_generation_tools;
}

pub fn map_toolbar_preview_zoom(
    ui: &mut egui::Ui,
    viewport: &mut EditorViewport,
    preview_cam: &mut PreviewCameraState,
    tex_w: u32,
    tex_h: u32,
    palette: &UiPalette,
    config: MapToolbarConfig,
) {
    if !config.show_render_mode && !config.show_zoom_reset {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        if config.show_render_mode {
            section_heading(ui, palette, CmdHeadingStyle::None, "Render");
            let mut gpu = preview_cam.mode == PreviewRenderMode::GpuRenderTarget;
            if ui.checkbox(&mut gpu, "GPU target").changed() {
                preview_cam.mode = if gpu {
                    PreviewRenderMode::GpuRenderTarget
                } else {
                    PreviewRenderMode::CpuRaster
                };
            }
        }
        if config.show_zoom_reset {
            primary_label(ui, palette, "Zoom");
            ui.add(
                egui::Slider::new(
                    &mut viewport.zoom,
                    PreviewLayers::ZOOM_MIN..=PreviewLayers::ZOOM_MAX,
                )
                .show_value(false),
            );
            if ui.small_button("1:1").clicked() {
                viewport.zoom = 1.0;
            }
            if ui.small_button("Fit").clicked() {
                viewport.reset_camera_for_map(tex_w as f32, tex_h as f32);
                viewport.zoom = 1.0;
            }
        }
    });
    muted_label(
        ui,
        palette,
        "Ctrl / ⌘ + scroll: zoom toward cursor. Middle-drag: pan.",
    );
}

pub fn map_toolbar_minimap_zoom(
    ui: &mut egui::Ui,
    shell: &mut MinimapShellState,
    presentation: &mut MapViewState,
) {
    ui.horizontal(|ui| {
        ui.label(format!("Zoom {:.2}x", presentation.zoom));
        ui.add(egui::Slider::new(&mut presentation.zoom_target, 0.35..=4.0).text("Target"));
    });
    shell.zoom = presentation.zoom;
    shell.zoom_target = presentation.zoom_target;
    if ui.button("Bookmark here").clicked() {
        let label = format!("bm{}", presentation.bookmarks.len() + 1);
        presentation.bookmarks.push(MinimapCameraBookmark {
            label,
            world: presentation.camera_center,
            zoom: presentation.zoom_target,
        });
        presentation.bump_revision();
    }
}
