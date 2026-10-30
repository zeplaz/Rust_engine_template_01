//! WorldGen session shell — preview viewport + containers for the control panel.
//!
//! Owns the full-screen root, the preview raster placement (viewport requests, pan / zoom,
//! image + corner inset), the status strip and the close-while-running confirm dialog. The
//! toolbar and parameter sheet are filled by [`super::world_gen_panel`]. Raster, hydrology
//! and tile sampling stay in their existing systems.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{MouseButton, MouseWheel};
use bevy::prelude::*;
use bevy::ui::widget::ImageNode;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::gui::editor::world_gen_ui::{CancelActiveWorldGenEvent, WorldGenUiState};
use crate::gui::map_presentation_fit::{
    compute_map_fit_strict, fit_viewport_to_map, MAP_PANEL_INSET_PX,
};
use crate::gui::map_view_projection::{
    ensure_viewport_camera_initialized, map_presentation_image_rect,
};
use crate::gui::resolve_world_preview_texture_source;
use crate::gui::style::widgets::{
    panel_bundle, spawn_button, spawn_note, ui_button_fired, UiButton, UiButtonVariant, UiKit,
    FONT_BODY, FONT_SMALL, GAP, SECTION_GAP,
};
use crate::gui::style::{CmdUiMonoFont, UiPalette};

use super::interaction::{apply_pan, apply_zoom_toward};
use super::layers::PreviewLayers;
use super::preview_render_contract::{PreviewCameraState, PreviewPathAuthority};
use super::texture_cache::WorldPreviewTexture;
use super::viewport_suggestion::{
    clear_world_preview_viewport_requests, write_world_preview_viewport_request,
};
use super::{d07_inset_side_px, WorldPreviewLifecycle, WorldPreviewRenderTargetRegistry, WorldPreviewUiState};

#[derive(Component)]
pub struct WorldGenShellRoot;

/// Filled by [`super::world_gen_panel`].
#[derive(Component)]
pub(super) struct WorldGenSheetRoot;

/// Filled by [`super::world_gen_panel`].
#[derive(Component)]
pub(super) struct WorldGenToolbarRoot;

#[derive(Component)]
struct WorldGenPreviewPanel;

#[derive(Component)]
struct WorldGenPreviewImage;

#[derive(Component)]
struct WorldGenCornerInset;

#[derive(Component)]
struct WorldGenMapDim;

#[derive(Component)]
struct WorldGenPlaceholderText;

#[derive(Component)]
struct WorldGenStatusText;

#[derive(Component)]
struct WorldGenConfirmRoot;

#[derive(Component, Clone, Copy)]
pub(crate) enum WorldGenAction {
    KeepOpen,
    CloseAndCancel,
}

/// Spawn the session shell: toolbar + sheet containers (filled by
/// [`super::world_gen_panel`]), the preview viewport, a status strip and the
/// close-while-running confirm dialog.
pub(crate) fn spawn_world_gen_bevy_shell(
    mut commands: Commands,
    existing: Query<Entity, With<WorldGenShellRoot>>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
) {
    if !existing.is_empty() {
        return;
    }
    let kit = UiKit::new(&palette, mono.0.clone());
    let muted = palette.bevy_text_muted();
    commands
        .spawn((
            WorldGenShellRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(palette.bevy_backdrop()),
            ZIndex(800),
        ))
        .with_children(|root| {
            root.spawn((WorldGenToolbarRoot, Node::default()));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                column_gap: Val::Px(6.0),
                padding: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            })
            .with_children(|body| {
                body.spawn((WorldGenSheetRoot, Node::default()));
                body.spawn((
                    WorldGenPreviewPanel,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        WorldGenPreviewImage,
                        ImageNode::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    ));
                    panel.spawn((
                        WorldGenCornerInset,
                        ImageNode::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(8.0),
                            bottom: Val::Px(8.0),
                            width: Val::Px(d07_inset_side_px()),
                            height: Val::Px(d07_inset_side_px()),
                            ..default()
                        },
                    ));
                    panel.spawn((
                        WorldGenMapDim,
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 102.0 / 255.0)),
                        Visibility::Hidden,
                    ));
                    panel.spawn((
                        WorldGenPlaceholderText,
                        Text::new("Preview initializing"),
                        kit.text_font(FONT_BODY),
                        TextColor(muted),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(12.0),
                            top: Val::Px(12.0),
                            ..default()
                        },
                    ));
                });
            });
            root.spawn((
                WorldGenStatusText,
                Text::new(""),
                kit.text_font(FONT_SMALL),
                TextColor(muted),
                Node {
                    padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
                    ..default()
                },
            ));
            root.spawn((
                WorldGenConfirmRoot,
                Visibility::Hidden,
                panel_bundle(
                    &palette,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(40.0),
                        margin: UiRect::left(Val::Px(-180.0)),
                        width: Val::Px(360.0),
                        ..default()
                    },
                ),
                ZIndex(20),
            ))
            .insert(BorderColor::all(palette.bevy_accent_gold()))
            .with_children(|confirm| {
                spawn_note(
                    confirm,
                    &kit,
                    "Generation is still running. Close anyway and discard partial work?",
                    false,
                    (),
                );
                confirm
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        column_gap: Val::Px(GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_button(
                            row,
                            &kit,
                            "Keep open",
                            UiButton::default(),
                            WorldGenAction::KeepOpen,
                        );
                        spawn_button(
                            row,
                            &kit,
                            "Close and cancel",
                            UiButton::new(UiButtonVariant::Danger),
                            WorldGenAction::CloseAndCancel,
                        );
                    });
            });
        });
}

pub(crate) fn despawn_world_gen_bevy_shell(
    mut commands: Commands,
    roots: Query<Entity, With<WorldGenShellRoot>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

#[derive(Default)]
struct PreviewPointer {
    last: Option<Vec2>,
    tex: (u32, u32),
}

#[derive(SystemParam)]
pub(crate) struct ShellSyncA<'w> {
    ui_state: Res<'w, WorldGenUiState>,
    preview_ui: ResMut<'w, WorldPreviewUiState>,
    preview_tex: Res<'w, WorldPreviewTexture>,
    path: Res<'w, PreviewPathAuthority>,
    registry: Res<'w, WorldPreviewRenderTargetRegistry>,
    lifecycle: Res<'w, WorldPreviewLifecycle>,
    views: ResMut<'w, crate::gui::MapViewInstances>,
    preview_cam: ResMut<'w, PreviewCameraState>,
}

#[derive(SystemParam)]
pub(crate) struct ShellSyncB<'w, 's> {
    viewport_requests: ResMut<'w, crate::gui::ViewportAuthority>,
    viewport_sanity: ResMut<'w, crate::gui::hud::ViewportRectSanity>,
    shell_diag: Option<ResMut<'w, crate::gui::hud::ProductShellDiagnostics>>,
    pending_layout: Res<'w, crate::gui::hud::PendingHudLayoutCommit>,
    // Marker-disjoint: one entity never carries two of these chrome markers.
    // Shared `Visibility` / `Text` writes panic at schedule init (B0001) otherwise.
    shell: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    sheet: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenSheetRoot>,
            Without<WorldGenShellRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    confirm: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenConfirmRoot>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    dim: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenMapDim>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    image_q: Query<
        'w,
        's,
        (&'static mut ImageNode, &'static mut Node, &'static mut Visibility),
        (
            With<WorldGenPreviewImage>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    inset_q: Query<
        'w,
        's,
        (&'static mut ImageNode, &'static mut Visibility),
        (
            With<WorldGenCornerInset>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
}

#[derive(SystemParam)]
pub(crate) struct ShellSyncC<'w, 's> {
    placeholder: Query<
        'w,
        's,
        (&'static mut Text, &'static mut Visibility),
        (
            With<WorldGenPlaceholderText>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenStatusText>,
        ),
    >,
    status: Query<
        'w,
        's,
        &'static mut Text,
        (With<WorldGenStatusText>, Without<WorldGenPlaceholderText>),
    >,
    panel_q: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<WorldGenPreviewPanel>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    wheel: MessageReader<'w, 's, MouseWheel>,
    pointer: Local<'s, PreviewPointer>,
}

pub(crate) fn sync_world_gen_bevy_shell(
    a: ShellSyncA,
    b: ShellSyncB,
    c: ShellSyncC,
) {
    let ShellSyncA {
        ui_state,
        mut preview_ui,
        preview_tex,
        path,
        registry,
        lifecycle,
        mut views,
        mut preview_cam,
    } = a;
    let ShellSyncB {
        mut viewport_requests,
        mut viewport_sanity,
        mut shell_diag,
        pending_layout,
        mut shell,
        mut sheet,
        mut confirm,
        mut dim,
        mut image_q,
        mut inset_q,
    } = b;
    let ShellSyncC {
        mut placeholder,
        mut status,
        panel_q,
        windows,
        mouse,
        keys,
        mut wheel,
        mut pointer,
    } = c;
    let show = preview_ui.window_open || ui_state.visible;
    for mut vis in &mut shell {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
    }
    if !show {
        preview_ui.d07_corner_inset_active = false;
        preview_ui.d07_inset_side_px = 0.0;
        clear_world_preview_viewport_requests(&mut viewport_requests);
        return;
    }
    let sheet_open = ui_state.visible && ui_state.generator_sheet_open;
    let vis_sheet = if sheet_open { Visibility::Visible } else { Visibility::Hidden };
    for mut vis in &mut sheet {
        *vis = vis_sheet;
    }
    let confirm_vis = if ui_state.confirm_close_pending {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut confirm {
        *vis = confirm_vis;
    }
    for mut vis in &mut dim {
        *vis = if sheet_open { Visibility::Visible } else { Visibility::Hidden };
    }

    let view = &mut views.world_preview;
    let disp_w = preview_tex.width.max(1);
    let disp_h = preview_tex.height.max(1);
    if pointer.tex != (disp_w, disp_h) {
        pointer.tex = (disp_w, disp_h);
        let panel = if view.viewport_size.x > 0.0 {
            view.viewport_size
        } else {
            Vec2::new(disp_w as f32, disp_h as f32)
        };
        fit_viewport_to_map(view, panel, disp_w as f32, disp_h as f32);
    }
    ensure_viewport_camera_initialized(view, disp_w as f32, disp_h as f32);
    view.zoom = view.zoom.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);

    let window = windows.iter().next();
    let scale = window.map(|w| w.scale_factor()).unwrap_or(1.0);
    let cursor = window.and_then(|w| w.cursor_position());
    if let Ok((node, xf)) = panel_q.single() {
        if !node.is_empty() {
            let panel_rect = node_logical_rect(node, xf, scale);
            preview_ui.last_viewport_rect = Some(panel_rect);
            preview_ui.last_window_rect = Some(panel_rect);
            write_world_preview_viewport_request(
                &mut viewport_requests,
                panel_rect,
                disp_w,
                disp_h,
                &mut viewport_sanity,
                shell_diag.as_deref_mut(),
                pending_layout.can_emit_layout_capture(),
            );
            let inner = bevy::prelude::Rect::from_corners(
                panel_rect.min + bevy::prelude::Vec2::splat(MAP_PANEL_INSET_PX),
                panel_rect.max - bevy::prelude::Vec2::splat(MAP_PANEL_INSET_PX),
            );
            let fit = compute_map_fit_strict(inner, UVec2::new(disp_w, disp_h), view.fit_mode);
            let gpu_painted = path.authoritative_surface
                == super::preview_render_contract::PreviewAuthoritativeSurface::GpuRenderTarget;
            let image_rect = if gpu_painted {
                fit.image_rect
            } else {
                map_presentation_image_rect(
                    fit.image_rect,
                    view.camera_center,
                    view.zoom,
                    disp_w as f32,
                    disp_h as f32,
                )
            };
            if let Some(cursor) = cursor {
                let p = bevy::prelude::Vec2::new(cursor.x, cursor.y);
                if panel_rect.contains(p) {
                    let scroll: f32 = wheel.read().map(|w| w.y).sum();
                    let zoom_mod = keys.pressed(KeyCode::ControlLeft)
                        || keys.pressed(KeyCode::ControlRight)
                        || keys.pressed(KeyCode::SuperLeft);
                    if zoom_mod && scroll != 0.0 {
                        let factor = 1.0 + scroll * 0.00125;
                        apply_zoom_toward(view, p, image_rect.center(), factor);
                        preview_cam.zoom = view.zoom;
                        preview_cam.center = view.camera_center;
                    }
                    if mouse.pressed(MouseButton::Middle) {
                        if let Some(last) = pointer.last {
                            apply_pan(view, cursor - last);
                            preview_cam.center = view.camera_center;
                        }
                        pointer.last = Some(cursor);
                    } else {
                        pointer.last = None;
                    }
                } else {
                    pointer.last = None;
                }
            }
            let source = resolve_world_preview_texture_source(
                path.as_ref(),
                registry.as_ref(),
                preview_tex.as_ref(),
            );
            let handle = source.handle().clone();
            let ready = handle != Handle::default()
                && lifecycle.phase.allows_egui_present();
            if let Ok((mut image, mut node, mut vis)) = image_q.single_mut() {
                *image = ImageNode::from(handle.clone());
                node.position_type = PositionType::Absolute;
                node.left = Val::Px((image_rect.min.x - panel_rect.min.x).max(0.0));
                node.top = Val::Px((image_rect.min.y - panel_rect.min.y).max(0.0));
                node.width = Val::Px(image_rect.width().max(1.0));
                node.height = Val::Px(image_rect.height().max(1.0));
                *vis = if ready { Visibility::Visible } else { Visibility::Hidden };
            }
            if let Ok((mut inset, mut vis)) = inset_q.single_mut() {
                *inset = ImageNode::from(handle.clone());
                let side = d07_inset_side_px();
                preview_ui.d07_corner_inset_active = ready;
                preview_ui.d07_inset_side_px = if ready { side } else { 0.0 };
                *vis = if ready { Visibility::Visible } else { Visibility::Hidden };
            }
            if let Ok((mut text, mut vis)) = placeholder.single_mut() {
                crate::gui::style::widgets::set_text_if_changed(&mut text, lifecycle.phase.placeholder_label());
                *vis = if ready { Visibility::Hidden } else { Visibility::Visible };
            }
            let _ = gpu_painted;
        }
    }

    if let Ok(mut text) = status.single_mut() {
        let next = format!(
            "Tile: {} | {}x{} world | z={:.2} | {}",
            view.hovered_tile
                .map(|t| format!("{} , {}", t.x, t.y))
                .unwrap_or_else(|| "-".to_string()),
            disp_w,
            disp_h,
            view.zoom,
            layer_summary(view.layers),
        );
        if text.0 != next {
            text.0 = next;
        }
    }
}

fn layer_summary(layers: PreviewLayers) -> String {
    let base = layers.base_bits();
    let mut parts = Vec::new();
    if base.is_empty() {
        parts.push("base: none");
    } else if base.contains(PreviewLayers::REGIONS) {
        parts.push("regions");
    } else if base.contains(PreviewLayers::ECOLOGY) {
        parts.push("ecology");
    } else if base.contains(PreviewLayers::BIOME) {
        parts.push("biome");
    } else if base.contains(PreviewLayers::HEIGHT) {
        parts.push("height");
    } else if base.contains(PreviewLayers::MOISTURE) {
        parts.push("moisture");
    } else if base.contains(PreviewLayers::TEMPERATURE) {
        parts.push("temperature");
    }
    if layers.contains(PreviewLayers::TAG_OVERLAY) {
        parts.push("+tags");
    }
    if layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
        parts.push("+slope");
    }
    if layers.contains(PreviewLayers::MOBILITY_OVERLAY) {
        parts.push("+mobility");
    }
    parts.join(" ")
}

fn node_logical_rect(node: &ComputedNode, xf: &UiGlobalTransform, scale: f32) -> bevy::prelude::Rect {
    let scale = scale.max(1e-6);
    let half = node.size() * 0.5;
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];
    let mut pmin = Vec2::splat(f32::INFINITY);
    let mut pmax = Vec2::splat(f32::NEG_INFINITY);
    for corner in corners {
        let p = (*xf) * corner;
        pmin = pmin.min(p);
        pmax = pmax.max(p);
    }
    pmin /= scale;
    pmax /= scale;
    bevy::prelude::Rect::from_corners(bevy::prelude::Vec2::new(pmin.x, pmin.y), bevy::prelude::Vec2::new(pmax.x, pmax.y))
}

/// Esc closes the sheet; the close-while-running dialog buttons.
pub(crate) fn world_gen_bevy_shell_actions(
    buttons: Query<(&Interaction, &UiButton, &WorldGenAction), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
    mut cancel: MessageWriter<CancelActiveWorldGenEvent>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if ui_state.confirm_close_pending {
            ui_state.confirm_close_pending = false;
        } else if ui_state.generator_sheet_open {
            ui_state.generator_sheet_open = false;
        }
    }
    for (interaction, button, action) in &buttons {
        if !ui_button_fired(interaction, button) {
            continue;
        }
        match action {
            WorldGenAction::KeepOpen => ui_state.confirm_close_pending = false,
            WorldGenAction::CloseAndCancel => {
                cancel.write(CancelActiveWorldGenEvent);
                preview_ui.window_open = false;
                ui_state.visible = false;
                ui_state.generator_sheet_open = false;
                ui_state.confirm_close_pending = false;
            }
        }
    }
}

#[cfg(test)]
mod wgen_tests {
    use super::*;

    #[test]
    fn wgen_base_layer_replace_does_not_touch_height_math() {
        let mut layers = PreviewLayers::HEIGHT;
        layers.replace_base(PreviewLayers::BIOME);
        assert!(layers.contains(PreviewLayers::BIOME));
        assert!(!layers.contains(PreviewLayers::HEIGHT));
    }

    /// Schedule init of `sync_world_gen_bevy_shell` builds the same query pair
    /// that panics with B0001 when `Visibility` writers are not marker-disjoint.
    #[test]
    fn wgen_shell_sync_visibility_queries_are_disjoint() {
        use bevy::ecs::schedule::Schedule;
        use bevy::ecs::world::World;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_world_gen_bevy_shell);
        schedule
            .initialize(&mut world)
            .expect("sync_world_gen_bevy_shell schedule must build");
    }
}
