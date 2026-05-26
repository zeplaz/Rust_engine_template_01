//! Main egui window: editor layout + clipped painter viewport.

use super::interaction::{apply_pan, apply_zoom_toward, update_hover_tile};
use super::layers::PreviewLayers;
use super::minimap::paint_world_preview_minimap_corner_inset;
use super::preview_render_contract::{
    preview_authoritative_surface, PreviewAuthoritativeSurface, PreviewCameraState,
};
use super::WorldPreviewGpuRuntime;
use super::registry_inspector::{world_preview_registry_inspector, PreviewRegistryInspectorHost};
use super::texture_cache::WorldPreviewTexture;
use super::WorldPreviewReady;
use super::ui_sidebar::world_preview_sidebar;
use super::ui_statusbar::world_preview_status_bar;
use super::ui_toolbar::world_preview_toolbar;
use super::viewport_suggestion::{
    clear_world_preview_viewport_requests, write_world_preview_viewport_request,
};
use crate::gui::ViewportAuthority;
use crate::gui::hud::{ProductShellDiagnostics, ViewportRectSanity, ViewportRectSource};
use crate::gui::map_view_projection::{
    ensure_viewport_camera_initialized, map_presentation_image_rect, map_texture_uv_rect,
};
use crate::gui::map_presentation_fit::{
    compute_map_fit_strict, fit_viewport_to_map, MAP_PANEL_INSET_PX,
};
use crate::gui::map_view::{paint_map_display_debug_outlines, paint_map_view_placeholder, MapViewInstances};
use crate::gui::{
    world_preview::resolve_world_preview_egui_texture, ActiveMapViewInput, MapPresentationDiagnostics,
    MapShellPointerGate, MapViewInstanceId, MapViewTextureCache, ResolvedMapViewFrames,
    MapViewInteractionByView, ViewManager,
};
use crate::gui::editor::world_gen_ui::{draw_world_gen_panel, WorldGenUiContext};
use crate::gui::hud::HudDevOverlayState;
use crate::gui::std_floating;
use crate::gui::style::{neutral_image_tint, widget_scroll_vertical_fill};
use crate::systems::terrain::TerrainRegistriesHandles;

use bevy::ecs::system::SystemParam;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(SystemParam)]
pub struct WorldPreviewDisplayState<'w> {
    preview_texture: Res<'w, WorldPreviewTexture>,
    preview_cam: ResMut<'w, PreviewCameraState>,
    viewport_requests: ResMut<'w, ViewportAuthority>,
    map_views: ResMut<'w, MapViewInstances>,
    view_manager: Res<'w, ViewManager>,
    frames: Res<'w, ResolvedMapViewFrames>,
    map_ready: ResMut<'w, crate::gui::MapViewReadyStates>,
    tex_cache: ResMut<'w, MapViewTextureCache>,
    map_presentation_diag: ResMut<'w, MapPresentationDiagnostics>,
    map_view_interaction: ResMut<'w, MapViewInteractionByView>,
    shell_pointer_gate: Res<'w, MapShellPointerGate>,
    active_map_input: ResMut<'w, ActiveMapViewInput>,
    pending_layout: Res<'w, crate::gui::hud::PendingHudLayoutCommit>,
    preview_ready: Res<'w, WorldPreviewReady>,
    gpu_rt: Res<'w, WorldPreviewGpuRuntime>,
    dev_overlay: Res<'w, HudDevOverlayState>,
    handles: Res<'w, TerrainRegistriesHandles>,
    tag_assets: Res<'w, Assets<crate::terrain::material::TagRegistry>>,
    material_assets: Res<'w, Assets<crate::terrain::material::MaterialRegistry>>,
    mobility_assets: Res<'w, Assets<crate::terrain::mobility::MobilityProfileRegistry>>,
    viewport_rect_sanity: ResMut<'w, ViewportRectSanity>,
    shell_diag: Option<ResMut<'w, ProductShellDiagnostics>>,
}

pub(crate) fn display_world_preview(
    mut contexts: EguiContexts,
    mut state: WorldPreviewDisplayState,
    mut world_gen: WorldGenUiContext,
    mut tuning_io_hint: Local<String>,
    mut last_tex: Local<(u32, u32)>,
) -> Result {
    if !world_gen.world_preview_ui.window_open {
        world_gen.world_preview_ui.d07_corner_inset_active = false;
        world_gen.world_preview_ui.d07_inset_side_px = 0.0;
        clear_world_preview_viewport_requests(&mut state.viewport_requests);
        if state.active_map_input.0 == Some(MapViewInstanceId::WorldPreview) {
            state.active_map_input.0 = None;
        }
        if crate::engine::worldgen_chrome_debug::worldgen_chrome_debug_enabled() {
            static SKIPPED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = SKIPPED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if n % 120 == 0 {
                info!(
                    target: "worldgen_chrome::egui",
                    skipped_frames = n,
                    "PREVIEW_EGUI_SKIP window_open=false"
                );
            }
        }
        return Ok(());
    }

    let disp_w = state.preview_texture.width;
    let disp_h = state.preview_texture.height;
    let tex_w = disp_w as f32;
    let tex_h = disp_h as f32;

    let interaction_frozen = !state.pending_layout.can_emit_layout_capture();
    let texture_id = resolve_world_preview_egui_texture(
        &mut contexts,
        &state.frames,
        &mut state.tex_cache,
        &mut state.map_ready,
        interaction_frozen,
    );
    let preview_ready = state.preview_ready.0
        && world_gen.lifecycle.phase.allows_egui_present()
        && state.map_ready.world_preview.ready_to_present();

    if *last_tex != (disp_w, disp_h) {
        *last_tex = (disp_w, disp_h);
        let panel = state.map_views.world_preview.viewport_size;
        fit_viewport_to_map(
            &mut state.map_views.world_preview,
            if panel.x > 0.0 { panel } else { Vec2::new(tex_w, tex_h) },
            tex_w,
            tex_h,
        );
    }
    ensure_viewport_camera_initialized(&mut state.map_views.world_preview, tex_w, tex_h);

    let view = &mut state.map_views.world_preview;
    let preview_cam = &mut state.preview_cam;
    let viewport_requests = &mut state.viewport_requests;
    let handles = &state.handles;
    let tag_assets = &state.tag_assets;
    let material_assets = &state.material_assets;
    let mobility_assets = &state.mobility_assets;
    let viewport_rect_sanity = &mut state.viewport_rect_sanity;
    let mut shell_diag = state.shell_diag.as_deref_mut();

    let show_map_debug = state.dev_overlay.visible && state.dev_overlay.show_map_transform;
    let mut window_open = world_gen.world_preview_ui.window_open;
    let panel_frame = egui::Frame::NONE;
    let unified = super::world_preview_unified_workspace(&world_gen.world_preview_ui);
    let sheet_open = unified && world_gen.world_gen_ui_state.generator_sheet_open;
    let window_response = std_floating(egui::Window::new("Operational Archive — World Index"))
        .id(egui::Id::new("world_preview_workspace"))
        .default_size([960.0, 640.0])
        .min_size([640.0, 480.0])
        .open(&mut window_open)
        .show(contexts.ctx_mut()?, |ui| {
            if sheet_open && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                world_gen.world_gen_ui_state.generator_sheet_open = false;
            }

            let pal = world_gen.palette.clone();
            let sp = world_gen.spacing.clone();
            let sheet_width = super::d04_sheet_width_px(ui.available_width());

            egui::TopBottomPanel::top("world_preview_toolbar")
                .frame(panel_frame)
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        world_preview_toolbar(
                            ui,
                            view,
                            preview_cam,
                            disp_w,
                            disp_h,
                            &pal,
                        );
                        if unified && world_gen.world_gen_ui_state.visible {
                            ui.separator();
                            let sheet_label = if world_gen.world_gen_ui_state.generator_sheet_open {
                                "Parameters ◂"
                            } else {
                                "Parameters ▸"
                            };
                            if ui
                                .selectable_label(
                                    world_gen.world_gen_ui_state.generator_sheet_open,
                                    sheet_label,
                                )
                                .clicked()
                            {
                                world_gen.world_gen_ui_state.generator_sheet_open =
                                    !world_gen.world_gen_ui_state.generator_sheet_open;
                            }
                        }
                    });
                });

            egui::TopBottomPanel::bottom("world_preview_status")
                .frame(panel_frame)
                .resizable(false)
                .show_inside(ui, |ui| {
                    world_preview_status_bar(
                        ui,
                        view.layers,
                        view,
                        disp_w,
                        disp_h,
                        &pal,
                    );
                });

            let sidebar_max = super::d02_sidebar_max_width_px(ui.available_width());
            egui::SidePanel::left("world_preview_sidebar")
                .frame(panel_frame)
                .resizable(true)
                .default_width(180.0_f32.min(sidebar_max))
                .width_range(super::D02_SIDEBAR_MIN_W..=sidebar_max)
                .show_inside(ui, |ui| {
                    widget_scroll_vertical_fill("world_preview_sidebar_scroll", ui.available_height())
                        .show(ui, |ui| {
                            world_preview_sidebar(
                                ui,
                                &mut world_gen.world_gen_ui_state,
                                &mut world_gen.world_gen_params,
                                &handles,
                                &tag_assets,
                                &mobility_assets,
                                &pal,
                                &sp,
                            );
                            world_preview_registry_inspector(
                                ui,
                                PreviewRegistryInspectorHost::default(),
                                &handles,
                                &material_assets,
                                &tag_assets,
                                &pal,
                                &sp,
                            );
                        });
                });

            if sheet_open {
                let sheet_id = egui::Id::new("world_gen_parameters_sheet");
                ui.ctx().memory_mut(|m| m.request_focus(sheet_id));
                egui::SidePanel::left("world_gen_parameters_sheet")
                    .frame(panel_frame)
                    .resizable(true)
                    .default_width(sheet_width)
                    .width_range(super::D04_SHEET_WIDTH_MIN..=super::D04_SHEET_WIDTH_MAX)
                    .show_inside(ui, |ui| {
                        ui.push_id(sheet_id, |ui| {
                            draw_world_gen_panel(ui, &mut world_gen, &mut tuning_io_hint, true);
                        });
                    });
            }

            let mut d07_corner_active = false;
            let mut d07_inset_side = 0.0f32;
            egui::CentralPanel::default()
                .frame(panel_frame)
                .show_inside(ui, |ui| {
                    let available = ui.available_size();
                    let size = viewport_rect_sanity.inspect_logical_size(
                        Vec2::new(available.x, available.y),
                        ViewportRectSource::WorldPreviewCentralPanel,
                        Vec2::new(tex_w.max(1.0), tex_h.max(1.0)),
                        shell_diag.as_deref_mut(),
                    );
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(size.x, size.y),
                        egui::Sense::click_and_drag(),
                    );
                    world_gen.world_preview_ui.last_viewport_rect = Some(rect);

                    let z = view
                        .zoom
                        .clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
                    view.zoom = z;

                    let fit_mode = view.fit_mode;
                    let inner = rect.shrink(MAP_PANEL_INSET_PX);
                    let fit = compute_map_fit_strict(
                        inner,
                        UVec2::new(disp_w, disp_h),
                        fit_mode,
                    );
                    let gpu_painted = preview_authoritative_surface(&state.gpu_rt, preview_cam)
                        == PreviewAuthoritativeSurface::GpuRenderTarget;
                    let interaction_center = if gpu_painted {
                        fit.image_rect.center()
                    } else {
                        map_presentation_image_rect(
                            fit.image_rect,
                            view.camera_center,
                            view.zoom,
                            tex_w,
                            tex_h,
                        )
                        .center()
                    };

                    let buffer_interaction = state.shell_pointer_gate.shell_pointer_active;
                    if response.hovered() {
                        state.active_map_input.0 = Some(MapViewInstanceId::WorldPreview);
                        let zoom_mod =
                            ui.ctx().input(|i| i.modifiers.ctrl || i.modifiers.command);
                        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                        if zoom_mod && scroll != 0.0 {
                            let factor = 1.0 + scroll * 0.00125;
                            if let Some(p) = ui.ctx().pointer_hover_pos() {
                                if buffer_interaction {
                                    state.map_view_interaction.world_preview.queue_zoom(
                                        factor,
                                        Vec2::new(p.x, p.y),
                                        Vec2::new(interaction_center.x, interaction_center.y),
                                    );
                                } else {
                                    apply_zoom_toward(view, p, interaction_center, factor);
                                }
                                if crate::engine::worldgen_chrome_debug::worldgen_chrome_debug_enabled() {
                                    info!(
                                        target: "worldgen_chrome::preview_zoom",
                                        zoom = view.zoom,
                                        center = ?view.camera_center,
                                        gpu_painted,
                                        "PREVIEW_ZOOM_SCROLL"
                                    );
                                }
                            }
                        }
                    }

                    if response.dragged_by(egui::PointerButton::Middle) {
                        let d = response.drag_delta();
                        if buffer_interaction {
                            state.map_view_interaction.world_preview.queue_pan(Vec2::new(d.x, d.y));
                        } else {
                            apply_pan(view, Vec2::new(d.x, d.y));
                        }
                    }

                    let scrolling_zoom = response.hovered()
                        && ui.ctx().input(|i| {
                            (i.modifiers.ctrl || i.modifiers.command) && i.smooth_scroll_delta.y != 0.0
                        });
                    let snap_interaction = response.dragged_by(egui::PointerButton::Middle)
                        || scrolling_zoom;
                    let smooth = &view.interaction;
                    let zoom_vis = if snap_interaction {
                        view.zoom
                    } else {
                        smooth.current_zoom
                    };
                    let pan_vis = if snap_interaction {
                        view.camera_center
                    } else {
                        smooth.current_pan
                    };

                    let image_rect = if gpu_painted {
                        fit.image_rect
                    } else {
                        map_presentation_image_rect(
                            fit.image_rect,
                            pan_vis,
                            zoom_vis,
                            tex_w,
                            tex_h,
                        )
                    };
                    let _interaction_center = if gpu_painted {
                        fit.image_rect.center()
                    } else {
                        image_rect.center()
                    };
                    update_hover_tile(
                        &state.view_manager,
                        view,
                        ui.ctx().pointer_hover_pos(),
                        rect,
                        image_rect,
                        disp_w,
                        disp_h,
                    );

                    write_world_preview_viewport_request(
                        viewport_requests,
                        rect,
                        disp_w,
                        disp_h,
                        viewport_rect_sanity,
                        shell_diag.as_deref_mut(),
                        state.pending_layout.can_emit_layout_capture(),
                    );
                    let sample_uv = map_texture_uv_rect();
                    let painter = ui.painter().with_clip_rect(rect);
                    if preview_ready {
                        if let Some(tex_id) = texture_id {
                            painter.image(tex_id, image_rect, sample_uv, neutral_image_tint());
                        } else {
                            paint_map_view_placeholder(
                                ui,
                                rect,
                                world_gen.lifecycle.phase.placeholder_label(),
                            );
                        }
                    } else {
                        paint_map_view_placeholder(
                            ui,
                            rect,
                            world_gen.lifecycle.phase.placeholder_label(),
                        );
                    }
                    if sheet_open {
                        let dim =
                            egui::Color32::from_black_alpha(super::D04_MAP_DIM_ALPHA);
                        painter.rect_filled(rect, 0.0, dim);
                    }
                    if let Some(tex_id) = texture_id {
                        d07_inset_side = paint_world_preview_minimap_corner_inset(
                            ui,
                            rect,
                            tex_id,
                            disp_w,
                            disp_h,
                            &pal,
                        );
                        d07_corner_active = true;
                    }
                    if show_map_debug {
                        paint_map_display_debug_outlines(
                            &painter,
                            &crate::gui::MapDisplayTransform::from_fit_truth(
                                rect,
                                UVec2::new(disp_w, disp_h),
                                fit_mode,
                                MAP_PANEL_INSET_PX,
                                image_rect,
                                sample_uv,
                                pan_vis,
                                zoom_vis,
                                gpu_painted,
                            ),
                            egui::Stroke::new(1.0, ui.visuals().warn_fg_color),
                        );
                    }
                    state.map_presentation_diag.record_fit_truth(
                        MapViewInstanceId::WorldPreview,
                        rect,
                        UVec2::new(disp_w, disp_h),
                        fit_mode,
                        MAP_PANEL_INSET_PX,
                        image_rect,
                        sample_uv,
                        state.frames.get(MapViewInstanceId::WorldPreview).viewport_extent,
                        pan_vis,
                        zoom_vis,
                        Some(state.frames.get(MapViewInstanceId::WorldPreview).world_bounds),
                        gpu_painted,
                    );
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
                        egui::StrokeKind::Inside,
                    );
                });
            world_gen.world_preview_ui.d07_corner_inset_active = d07_corner_active;
            world_gen.world_preview_ui.d07_inset_side_px = d07_inset_side;
        });

    if let Some(inner) = window_response {
        world_gen.world_preview_ui.last_window_rect = Some(inner.response.rect);
    }

    world_gen.world_preview_ui.window_open = window_open;
    world_gen.world_gen_ui_state.visible = window_open;
    if !window_open {
        world_gen.world_gen_ui_state.generator_sheet_open = false;
    }
    world_gen.world_gen_ui_state.preview_layers = view.layers;

    let frame = state.frames.get(MapViewInstanceId::WorldPreview);
    crate::engine::worldgen_chrome_debug::trace_preview_egui_chrome(
        window_open,
        preview_ready,
        texture_id.is_some(),
        world_gen.lifecycle.phase,
        &state.tex_cache,
        frame.projection_revision,
        false,
    );

    Ok(())
}
