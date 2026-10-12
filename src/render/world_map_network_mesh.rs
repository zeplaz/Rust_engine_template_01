//! World ribbons and zone rectangles on the terrain-sprite plane.
//!
//! One writer per component. [`crate::construction::world_map_frame`] keeps the
//! building cells. This plugin does not spawn those cells.
//!
//! Map axes follow [`crate::gui::tactical::map_camera::map_plane_horizontal_xy`]:
//! a path stored as `(x, 0, row)` lands on `(x, row)`, the same axes as
//! [`crate::construction::world_map_tile_center`]. Z is just in front of that
//! sprite so the world camera sees the line. Width is in world tiles, never a
//! screen-pixel zoom stamp.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::construction::{
    CorridorPhaseDrawKind, ConstructionVisualRequests, VisualPathKind, world_map_tile_center,
};
use crate::economy::logistics::intent_draw::LogisticsIntentDrawFrameRes;
use crate::engine::states::BaseState;
use crate::gui::tactical::map_camera::map_plane_horizontal_xy;
use crate::render::pipelines::infrastructure_overlay::{
    InfrastructureNetworkLayer, InfrastructureOverlayDrawRequests, InfrastructureOverlaySettings,
    PowerLineOverlayState, PowerMapOverlayPresentation,
};
use crate::strategic::settlement::{admin_zone_drag_rect, AdminZoneDrag};
use crate::strategic::{ConstructionSite, SiteConstructionPhase, SiteFootprint};

/// Just in front of `world_map_tile_center` (z = 1).
const PLANE_Z: f32 = 1.04;

#[derive(Component)]
pub(crate) struct WorldMapRoadSeg;

#[derive(Component)]
pub(crate) struct WorldMapRailSeg;

#[derive(Component)]
pub(crate) struct WorldMapZoneCell;

#[derive(Component)]
pub(crate) struct WorldMapCorridorSeg;

#[derive(Component)]
pub(crate) struct WorldMapZoneDrag;

#[derive(Component)]
pub(crate) struct WorldMapPowerSeg;

#[derive(Component)]
pub(crate) struct WorldMapPowerMark;

#[derive(Component)]
pub(crate) struct WorldMapLogisticsSeg;

#[derive(Component)]
pub(crate) struct WorldMapOrdersSeg;

#[derive(Component)]
pub(crate) struct WorldMapPhaseLabel;

#[derive(Component)]
pub(crate) struct WorldMapLockRing;

#[derive(Component)]
pub(crate) struct WorldMapTownLabel;

#[derive(Component)]
pub(crate) struct WorldMapZoneTint;

#[derive(Default)]
pub(crate) struct LayerStamp {
    key: u64,
}

pub struct WorldMapNetworkMeshPlugin;

impl Plugin for WorldMapNetworkMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_world_map_path_meshes
                    .after(crate::construction::sync_zone_visual_requests),
                sync_world_map_power_meshes
                    .after(crate::construction::sync_power_line_preview_overlay_system),
                sync_world_map_phase_and_locks
                    .after(crate::construction::sync_zone_visual_requests),
                sync_world_map_town_names_and_zone_shade,
                default_city_zone_shade,
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                sync_world_map_logistics_meshes
                    .after(crate::economy::logistics::intent_draw::sync_logistics_intent_draw_frame),
                sync_world_map_orders_meshes,
            ),
        );
    }
}

/// Construction / utility point onto the terrain sprite.
#[must_use]
pub fn construction_point_on_world_map(world: Vec3) -> Vec3 {
    let xy = map_plane_horizontal_xy(world);
    Vec3::new(xy.x, xy.y, PLANE_Z)
}

fn color32(c: crate::gui::UiRgba) -> Color {
    Color::srgba_u8(c.r, c.g, c.b, c.a)
}

fn mix(key: &mut u64, n: u64) {
    *key = key.wrapping_mul(0x9E37_79B9).wrapping_add(n);
}

fn q(v: f32) -> u64 {
    (v * 32.0).round() as i64 as u64
}

/// Pixel stroke weights (8, 64) are egui. A ribbon on the tile grid is a fraction of a cell.
fn world_ribbon_width(request_width: f32) -> f32 {
    if request_width > 2.0 {
        0.22
    } else {
        request_width.clamp(0.06, 0.9)
    }
}

fn ribbon_transform(start: Vec3, end: Vec3, width: f32) -> Transform {
    let a = map_plane_horizontal_xy(start);
    let b = map_plane_horizontal_xy(end);
    let delta = b - a;
    let len = delta.length().max(0.05);
    let angle = delta.y.atan2(delta.x);
    let mid = (a + b) * 0.5;
    Transform {
        translation: Vec3::new(mid.x, mid.y, PLANE_Z),
        rotation: Quat::from_rotation_z(angle),
        scale: Vec3::new(len, world_ribbon_width(width), 1.0),
    }
}

fn unit_mesh(meshes: &mut Assets<Mesh>, slot: &mut Option<Handle<Mesh>>) -> Handle<Mesh> {
    slot.get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
        .clone()
}

fn material_for(
    cache: &mut HashMap<u32, Handle<ColorMaterial>>,
    materials: &mut Assets<ColorMaterial>,
    color: Color,
) -> Handle<ColorMaterial> {
    let s = color.to_srgba();
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u32;
    let key = (byte(s.red) << 24) | (byte(s.green) << 16) | (byte(s.blue) << 8) | byte(s.alpha);
    cache
        .entry(key)
        .or_insert_with(|| materials.add(ColorMaterial::from_color(color)))
        .clone()
}

fn despawn_query<T: Component>(commands: &mut Commands, query: &Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn path_color(kind: VisualPathKind, valid: bool, slope_ok: bool, committed: bool) -> Color {
    let token = match (kind, valid, slope_ok, committed) {
        (VisualPathKind::Road, true, _, true) => crate::construction::road_committed_color(),
        (VisualPathKind::Road, true, _, false) => crate::construction::road_segment_color(true),
        (VisualPathKind::Rail, true, true, _) => {
            crate::gui::UiRgba::from_rgba_unmultiplied(180, 120, 255, 150)
        }
        (VisualPathKind::Rail, _, false, _) => {
            crate::gui::UiRgba::from_rgba_unmultiplied(255, 140, 60, 180)
        }
        _ => crate::gui::UiRgba::from_rgba_unmultiplied(240, 90, 90, 160),
    };
    color32(token)
}

pub(crate) fn sync_world_map_path_meshes(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    requests: Option<Res<ConstructionVisualRequests>>,
    drag: Option<Res<AdminZoneDrag>>,
    roads: Query<Entity, With<WorldMapRoadSeg>>,
    rails: Query<Entity, With<WorldMapRailSeg>>,
    zones: Query<Entity, With<WorldMapZoneCell>>,
    corridors: Query<Entity, With<WorldMapCorridorSeg>>,
    zone_drag: Query<Entity, With<WorldMapZoneDrag>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mesh_slot: Local<Option<Handle<Mesh>>>,
    mut mat_cache: Local<HashMap<u32, Handle<ColorMaterial>>>,
    mut stamp: Local<LayerStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        despawn_query(&mut commands, &roads);
        despawn_query(&mut commands, &rails);
        despawn_query(&mut commands, &zones);
        despawn_query(&mut commands, &corridors);
        despawn_query(&mut commands, &zone_drag);
        stamp.key = 0;
        return;
    }
    let mut key = 0u64;
    let req = requests.as_deref();
    if let Some(req) = req {
        mix(&mut key, req.paths.len() as u64);
        for path in &req.paths {
            mix(&mut key, q(path.start.x) ^ q(path.end.z).rotate_left(8));
            mix(&mut key, path.committed as u64);
        }
        mix(&mut key, req.zone_tiles.len() as u64);
        mix(&mut key, req.corridor_paths.len() as u64);
    }
    if let Some(drag) = drag.as_deref() {
        mix(&mut key, drag.armed as u64);
        if let Some(rect) = admin_zone_drag_rect(drag) {
            mix(&mut key, rect.min.x as u64);
            mix(&mut key, rect.max.y as u64);
        }
    }
    if key == stamp.key {
        return;
    }
    despawn_query(&mut commands, &roads);
    despawn_query(&mut commands, &rails);
    despawn_query(&mut commands, &zones);
    despawn_query(&mut commands, &corridors);
    despawn_query(&mut commands, &zone_drag);
    stamp.key = key;
    let Some(req) = req else {
        return;
    };
    let unit = unit_mesh(&mut meshes, &mut mesh_slot);
    for path in &req.paths {
        let color = path_color(path.kind, path.valid, path.slope_ok, path.committed);
        let mat = material_for(&mut mat_cache, &mut materials, color);
        match path.kind {
            VisualPathKind::Road => {
                commands.spawn((
                    WorldMapRoadSeg,
                    Mesh2d(unit.clone()),
                    MeshMaterial2d(mat),
                    ribbon_transform(path.start, path.end, path.width),
                ));
            }
            VisualPathKind::Rail => {
                commands.spawn((
                    WorldMapRailSeg,
                    Mesh2d(unit.clone()),
                    MeshMaterial2d(mat),
                    ribbon_transform(path.start, path.end, path.width),
                ));
            }
        }
    }
    let cell = meshes.add(Rectangle::new(0.92, 0.92));
    for zone in &req.zone_tiles {
        let mat = material_for(&mut mat_cache, &mut materials, color32(zone.color));
        let xy = map_plane_horizontal_xy(zone.center);
        commands.spawn((
            WorldMapZoneCell,
            Mesh2d(cell.clone()),
            MeshMaterial2d(mat),
            Transform::from_translation(Vec3::new(xy.x, xy.y, PLANE_Z - 0.01)),
        ));
    }
    for corridor in &req.corridor_paths {
        let token = match corridor.kind {
            CorridorPhaseDrawKind::Planned => crate::construction::corridor_planned_color(),
            CorridorPhaseDrawKind::InProgress => crate::construction::corridor_in_progress_color(),
        };
        let mat = material_for(&mut mat_cache, &mut materials, color32(token));
        for pair in corridor.points.windows(2) {
            commands.spawn((
                WorldMapCorridorSeg,
                Mesh2d(unit.clone()),
                MeshMaterial2d(mat.clone()),
                ribbon_transform(pair[0], pair[1], 0.22),
            ));
        }
    }
    if let Some(drag) = drag.as_deref().filter(|d| d.armed) {
        if let Some(rect) = admin_zone_drag_rect(drag) {
            spawn_zone_drag_rect(&mut commands, &unit, &mut mat_cache, &mut materials, rect);
        }
    }
}

fn spawn_zone_drag_rect(
    commands: &mut Commands,
    unit: &Handle<Mesh>,
    mat_cache: &mut HashMap<u32, Handle<ColorMaterial>>,
    materials: &mut Assets<ColorMaterial>,
    rect: IRect,
) {
    let w = (rect.max.x - rect.min.x) as f32;
    let h = (rect.max.y - rect.min.y) as f32;
    if w < 1.0 || h < 1.0 {
        return;
    }
    let center = Vec3::new(
        rect.min.x as f32 + w * 0.5,
        rect.min.y as f32 + h * 0.5,
        PLANE_Z,
    );
    let fill = material_for(
        mat_cache,
        materials,
        Color::srgba(0.91, 0.77, 0.38, 0.28),
    );
    let edge = material_for(
        mat_cache,
        materials,
        Color::srgba(0.91, 0.77, 0.38, 0.9),
    );
    commands.spawn((
        WorldMapZoneDrag,
        Mesh2d(unit.clone()),
        MeshMaterial2d(fill),
        Transform::from_translation(center).with_scale(Vec3::new(w * 0.96, h * 0.96, 1.0)),
    ));
    let t = 0.08_f32;
    let strips = [
        (Vec3::new(center.x, rect.min.y as f32 + t * 0.5, PLANE_Z + 0.01), Vec3::new(w, t, 1.0)),
        (Vec3::new(center.x, rect.max.y as f32 - t * 0.5, PLANE_Z + 0.01), Vec3::new(w, t, 1.0)),
        (Vec3::new(rect.min.x as f32 + t * 0.5, center.y, PLANE_Z + 0.01), Vec3::new(t, h, 1.0)),
        (Vec3::new(rect.max.x as f32 - t * 0.5, center.y, PLANE_Z + 0.01), Vec3::new(t, h, 1.0)),
    ];
    for (pos, scale) in strips {
        commands.spawn((
            WorldMapZoneDrag,
            Mesh2d(unit.clone()),
            MeshMaterial2d(edge.clone()),
            Transform::from_translation(pos).with_scale(scale),
        ));
    }
    let hatch = material_for(
        mat_cache,
        materials,
        Color::srgba(0.95, 0.9, 0.7, 0.55),
    );
    let span = w.min(h);
    let steps = (span.round() as i32).clamp(1, 6);
    for i in 0..steps {
        let t = (i as f32 + 0.5) / steps as f32;
        let x = rect.min.x as f32 + t * w;
        let y0 = rect.min.y as f32 + 0.15;
        let y1 = rect.max.y as f32 - 0.15;
        commands.spawn((
            WorldMapZoneDrag,
            Mesh2d(unit.clone()),
            MeshMaterial2d(hatch.clone()),
            ribbon_transform(Vec3::new(x, 0.0, y0), Vec3::new(x + 0.35, 0.0, y1), 0.06),
        ));
    }
}

pub(crate) fn sync_world_map_power_meshes(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    settings: Option<Res<InfrastructureOverlaySettings>>,
    presentation: Option<Res<PowerMapOverlayPresentation>>,
    overlays: Option<Res<InfrastructureOverlayDrawRequests>>,
    segs: Query<Entity, With<WorldMapPowerSeg>>,
    marks: Query<Entity, With<WorldMapPowerMark>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mesh_slot: Local<Option<Handle<Mesh>>>,
    mut mat_cache: Local<HashMap<u32, Handle<ColorMaterial>>>,
    mut stamp: Local<LayerStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        despawn_query(&mut commands, &segs);
        despawn_query(&mut commands, &marks);
        stamp.key = 0;
        return;
    }
    let show = settings.as_deref().is_some_and(|s| s.enabled && s.power)
        || presentation.as_deref().is_some_and(|p| p.island_highlight_active);
    if !show {
        despawn_query(&mut commands, &segs);
        despawn_query(&mut commands, &marks);
        stamp.key = 0;
        return;
    }
    let mut key = 1u64;
    if let Some(overlays) = overlays.as_deref() {
        for edge in overlays.edges.iter().filter(|e| e.layer == InfrastructureNetworkLayer::Power) {
            mix(&mut key, edge.link_id);
            mix(&mut key, q(edge.head.x) ^ q(edge.tail.y));
        }
    }
    if let Some(presentation) = presentation.as_deref() {
        mix(&mut key, presentation.preview_segments.len() as u64);
        mix(&mut key, presentation.island_offline_world_positions.len() as u64);
    }
    if key == stamp.key {
        return;
    }
    despawn_query(&mut commands, &segs);
    despawn_query(&mut commands, &marks);
    stamp.key = key;
    let unit = unit_mesh(&mut meshes, &mut mesh_slot);
    if let Some(overlays) = overlays.as_deref() {
        for edge in overlays.edges.iter().filter(|e| e.layer == InfrastructureNetworkLayer::Power) {
            let rgb = edge.stroke.color_rgb;
            let color = Color::srgba(
                rgb[0] as f32 / 255.0,
                rgb[1] as f32 / 255.0,
                rgb[2] as f32 / 255.0,
                edge.stroke.alpha,
            );
            let mat = material_for(&mut mat_cache, &mut materials, color);
            commands.spawn((
                WorldMapPowerSeg,
                Mesh2d(unit.clone()),
                MeshMaterial2d(mat.clone()),
                ribbon_transform(edge.head, edge.tail, edge.stroke.weight_px),
            ));
            if matches!(
                edge.line_state,
                Some(PowerLineOverlayState::Damaged) | Some(PowerLineOverlayState::Destroyed)
            ) {
                let mid = construction_point_on_world_map((edge.head + edge.tail) * 0.5);
                let mark = meshes.add(Rectangle::new(0.18, 0.18));
                commands.spawn((
                    WorldMapPowerMark,
                    Mesh2d(mark),
                    MeshMaterial2d(mat),
                    Transform::from_translation(mid + Vec3::new(0.0, 0.0, 0.02)),
                ));
            }
        }
    }
    if let Some(presentation) = presentation.as_deref() {
        let preview = material_for(
            &mut mat_cache,
            &mut materials,
            Color::srgba(0.95, 0.85, 0.4, 0.6),
        );
        for &(head, tail, _) in &presentation.preview_segments {
            commands.spawn((
                WorldMapPowerSeg,
                Mesh2d(unit.clone()),
                MeshMaterial2d(preview.clone()),
                ribbon_transform(
                    Vec3::new(head.x, head.y, 0.0),
                    Vec3::new(tail.x, tail.y, 0.0),
                    0.16,
                ),
            ));
        }
        if presentation.island_highlight_active {
            let off = material_for(
                &mut mat_cache,
                &mut materials,
                Color::srgba(0.86, 0.31, 0.31, 0.85),
            );
            let dot = meshes.add(Rectangle::new(0.22, 0.22));
            for world in &presentation.island_offline_world_positions {
                commands.spawn((
                    WorldMapPowerMark,
                    Mesh2d(dot.clone()),
                    MeshMaterial2d(off.clone()),
                    Transform::from_translation(construction_point_on_world_map(Vec3::new(
                        world.x, world.y, 0.0,
                    ))),
                ));
            }
        }
    }
}

pub(crate) fn sync_world_map_phase_and_locks(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    requests: Option<Res<ConstructionVisualRequests>>,
    sites: Query<(&ConstructionSite, &SiteFootprint)>,
    labels: Query<Entity, With<WorldMapPhaseLabel>>,
    locks: Query<Entity, With<WorldMapLockRing>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ring: Local<Option<Handle<Mesh>>>,
    mut stamp: Local<LayerStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        despawn_query(&mut commands, &labels);
        despawn_query(&mut commands, &locks);
        stamp.key = 0;
        return;
    }
    let mut key = sites.iter().count() as u64;
    for (site, foot) in &sites {
        mix(&mut key, site.site_id);
        mix(&mut key, phase_id(site.phase));
        mix(&mut key, foot.tiles.len() as u64);
    }
    if let Some(req) = requests.as_deref() {
        mix(&mut key, req.footprint_lock_tiles.len() as u64);
    }
    if key == stamp.key {
        return;
    }
    despawn_query(&mut commands, &labels);
    despawn_query(&mut commands, &locks);
    stamp.key = key;
    for (site, foot) in &sites {
        if foot.tiles.is_empty() {
            continue;
        }
        let mut acc = Vec3::ZERO;
        for tile in &foot.tiles {
            acc += world_map_tile_center(tile.x, tile.y);
        }
        acc /= foot.tiles.len() as f32;
        acc.z = PLANE_Z + 0.03;
        commands.spawn((
            WorldMapPhaseLabel,
            Text2d::new(crate::construction::phase_map_word(site.phase)),
            TextFont {
                font_size: FontSize::Px(32.0),
                ..default()
            },
            TextColor(Color::srgb(0.93, 0.93, 0.9)),
            bevy::sprite::Anchor::CENTER,
            Transform::from_translation(acc).with_scale(Vec3::splat(0.28 / 32.0)),
        ));
    }
    let Some(req) = requests.as_deref() else {
        return;
    };
    if req.footprint_lock_tiles.is_empty() {
        return;
    }
    let mesh = ring
        .get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
        .clone();
    let mat = materials.add(ColorMaterial::from_color(Color::srgba(0.91, 0.77, 0.38, 0.9)));
    let t = 0.06_f32;
    for tile in &req.footprint_lock_tiles {
        let center = world_map_tile_center(tile.x, tile.y);
        let z = PLANE_Z + 0.02;
        let strips = [
            (Vec3::new(center.x, center.y - 0.46, z), Vec3::new(0.92, t, 1.0)),
            (Vec3::new(center.x, center.y + 0.46, z), Vec3::new(0.92, t, 1.0)),
            (Vec3::new(center.x - 0.46, center.y, z), Vec3::new(t, 0.92, 1.0)),
            (Vec3::new(center.x + 0.46, center.y, z), Vec3::new(t, 0.92, 1.0)),
        ];
        for (pos, scale) in strips {
            commands.spawn((
                WorldMapLockRing,
                Mesh2d(mesh.clone()),
                MeshMaterial2d(mat.clone()),
                Transform::from_translation(pos).with_scale(scale),
            ));
        }
    }
}

fn phase_id(phase: SiteConstructionPhase) -> u64 {
    crate::construction::phase_map_word(phase).len() as u64
}

pub(crate) fn sync_world_map_logistics_meshes(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    frame: Option<Res<LogisticsIntentDrawFrameRes>>,
    segs: Query<Entity, With<WorldMapLogisticsSeg>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mesh_slot: Local<Option<Handle<Mesh>>>,
    mut mat_cache: Local<HashMap<u32, Handle<ColorMaterial>>>,
    mut stamp: Local<LayerStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        despawn_query(&mut commands, &segs);
        stamp.key = 0;
        return;
    }
    let Some(frame) = frame else {
        despawn_query(&mut commands, &segs);
        stamp.key = 0;
        return;
    };
    let mut key = frame.0.haul.len() as u64;
    for haul in &frame.0.haul {
        mix(&mut key, haul.points.len() as u64);
        if let Some(p) = haul.points.first() {
            mix(&mut key, q(p.x) ^ q(p.z));
        }
    }
    mix(&mut key, frame.0.ai_sites.len() as u64);
    if key == stamp.key {
        return;
    }
    despawn_query(&mut commands, &segs);
    stamp.key = key;
    let unit = unit_mesh(&mut meshes, &mut mesh_slot);
    for haul in &frame.0.haul {
        let rgba = haul.kind.rgba();
        let mat = material_for(
            &mut mat_cache,
            &mut materials,
            Color::srgba_u8(rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        for pair in haul.points.windows(2) {
            commands.spawn((
                WorldMapLogisticsSeg,
                Mesh2d(unit.clone()),
                MeshMaterial2d(mat.clone()),
                ribbon_transform(pair[0], pair[1], 0.16),
            ));
        }
    }
    let outline = material_for(
        &mut mat_cache,
        &mut materials,
        Color::srgba(0.88, 0.35, 0.85, 0.82),
    );
    for site in &frame.0.ai_sites {
        let x0 = site.origin.x as f32;
        let z0 = site.origin.z as f32;
        let x1 = x0 + site.footprint.width.max(1) as f32;
        let z1 = z0 + site.footprint.depth.max(1) as f32;
        let corners = [
            Vec3::new(x0, 0.0, z0),
            Vec3::new(x1, 0.0, z0),
            Vec3::new(x1, 0.0, z1),
            Vec3::new(x0, 0.0, z1),
        ];
        for i in 0..4 {
            commands.spawn((
                WorldMapLogisticsSeg,
                Mesh2d(unit.clone()),
                MeshMaterial2d(outline.clone()),
                ribbon_transform(corners[i], corners[(i + 1) % 4], 0.08),
            ));
        }
    }
}

/// Orders overlay ribbons from [`build_orders_overlay`] (PORT-LOGISTICS-OVERLAY stretch).
pub(crate) fn sync_world_map_orders_meshes(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    snap: Option<Res<crate::economy::logistics::orders::LogisticsOrdersSnapshotRes>>,
    panel: Option<Res<crate::gui::LogisticsOrdersPanelState>>,
    segs: Query<Entity, With<WorldMapOrdersSeg>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut mesh_slot: Local<Option<Handle<Mesh>>>,
    mut mat_cache: Local<HashMap<u32, Handle<ColorMaterial>>>,
    mut stamp: Local<LayerStamp>,
) {
    use crate::economy::logistics::orders::snapshot::METRES_PER_TILE;
    use crate::gui::{build_orders_overlay, RouteTone};

    let show = matches!(base.get(), BaseState::Simulation)
        && panel
            .as_ref()
            .is_some_and(|p| p.visible || p.always_show_routes);
    if !show {
        despawn_query(&mut commands, &segs);
        stamp.key = 0;
        return;
    }
    let Some(snap) = snap else {
        despawn_query(&mut commands, &segs);
        stamp.key = 0;
        return;
    };
    let with_sites = panel.as_ref().is_some_and(|p| p.visible);
    let frame = build_orders_overlay(&snap.0, with_sites);
    let mut key = frame.routes.len() as u64 + frame.vehicles.len() as u64 * 17;
    for r in &frame.routes {
        mix(&mut key, q(r.from.x) ^ q(r.from.y));
        mix(&mut key, q(r.to.x) ^ q(r.to.y));
        let tone = match r.tone {
            RouteTone::Flowing => 1u64,
            RouteTone::Waiting => 2,
            RouteTone::Paused => 3,
        };
        mix(&mut key, r.standing as u64 + tone);
    }
    if key == stamp.key {
        return;
    }
    despawn_query(&mut commands, &segs);
    stamp.key = key;
    let unit = unit_mesh(&mut meshes, &mut mesh_slot);
    let tile_world = |t: Vec2| Vec3::new(t.x * METRES_PER_TILE, 0.0, t.y * METRES_PER_TILE);
    for r in &frame.routes {
        let color = match r.tone {
            RouteTone::Flowing => Color::srgba(0.35, 0.85, 0.55, 0.9),
            RouteTone::Waiting => Color::srgba(0.95, 0.75, 0.25, 0.9),
            RouteTone::Paused => Color::srgba(0.7, 0.7, 0.7, 0.75),
        };
        let mat = material_for(&mut mat_cache, &mut materials, color);
        let width = if r.standing { 0.14 } else { 0.1 };
        commands.spawn((
            WorldMapOrdersSeg,
            Mesh2d(unit.clone()),
            MeshMaterial2d(mat),
            ribbon_transform(tile_world(r.from), tile_world(r.to), width),
        ));
    }
    let loaded = material_for(
        &mut mat_cache,
        &mut materials,
        Color::srgba(0.2, 0.9, 1.0, 0.95),
    );
    let empty = material_for(
        &mut mat_cache,
        &mut materials,
        Color::srgba(0.85, 0.85, 0.9, 0.7),
    );
    for v in &frame.vehicles {
        let mat = if v.loaded {
            loaded.clone()
        } else {
            empty.clone()
        };
        let p = tile_world(v.at);
        let xy = map_plane_horizontal_xy(p);
        commands.spawn((
            WorldMapOrdersSeg,
            Mesh2d(unit.clone()),
            MeshMaterial2d(mat),
            Transform {
                translation: Vec3::new(xy.x, xy.y, PLANE_Z + 0.01),
                scale: Vec3::splat(0.22),
                ..default()
            },
        ));
    }
}

fn default_city_zone_shade(
    scene: Option<Res<crate::engine::ActiveTestScene>>,
    shade: Option<ResMut<crate::gui::hud::simulation_shell_phase2::ZoneShadeToggle>>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    let Some(mut shade) = shade else {
        return;
    };
    let Some(scene) = scene else {
        return;
    };
    *applied = true;
    if scene.0 == crate::engine::TestScene::City {
        shade.visible = true;
    }
}

fn zone_tint_rgb(id: &str) -> [f32; 3] {
    const TINTS: [[f32; 3]; 8] = [
        [0.35, 0.55, 0.85],
        [0.40, 0.72, 0.42],
        [0.85, 0.62, 0.28],
        [0.68, 0.40, 0.72],
        [0.28, 0.68, 0.70],
        [0.78, 0.38, 0.38],
        [0.55, 0.52, 0.28],
        [0.42, 0.46, 0.78],
    ];
    let mut h = 2166136261u32;
    for b in id.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(16777619);
    }
    TINTS[(h as usize) % TINTS.len()]
}

/// Tiles the shade mesh covers. A closed ring uses its interior. An open ring uses the rectangle.
fn zone_shade_interior_tiles(zone: &crate::strategic::DistrictRecord) -> Vec<IVec2> {
    let rect = zone.tile_rect;
    if zone.ring.len() < 3 {
        let mut tiles = Vec::new();
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                tiles.push(IVec2::new(x, y));
            }
        }
        return tiles;
    }
    let mut tiles = Vec::new();
    for y in rect.min.y..rect.max.y {
        for x in rect.min.x..rect.max.x {
            let tile = IVec2::new(x, y);
            if zone.contains_tile(tile) {
                tiles.push(tile);
            }
        }
    }
    tiles
}

fn ring_shade_mesh(tiles: &[IVec2]) -> Mesh {
    let mut positions = Vec::with_capacity(tiles.len() * 4);
    let mut indices = Vec::with_capacity(tiles.len() * 6);
    for tile in tiles {
        let i = positions.len() as u32;
        let x = tile.x as f32;
        let y = tile.y as f32;
        positions.push([x, y, 0.0]);
        positions.push([x + 1.0, y, 0.0]);
        positions.push([x + 1.0, y + 1.0, 0.0]);
        positions.push([x, y + 1.0, 0.0]);
        indices.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
    }
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}
pub(crate) fn sync_world_map_town_names_and_zone_shade(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    towns: Option<Res<crate::strategic::TownBook>>,
    districts: Option<Res<crate::strategic::DistrictBook>>,
    shade: Option<Res<crate::gui::hud::simulation_shell_phase2::ZoneShadeToggle>>,
    labels: Query<Entity, With<WorldMapTownLabel>>,
    tints: Query<Entity, With<WorldMapZoneTint>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut stamp: Local<LayerStamp>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        despawn_query(&mut commands, &labels);
        despawn_query(&mut commands, &tints);
        stamp.key = 0;
        return;
    }
    let show_tint = shade.is_some_and(|s| s.visible);
    let mut key = show_tint as u64;
    if let Some(book) = towns.as_deref() {
        mix(&mut key, book.towns.len() as u64);
        let mut ids: Vec<_> = book.towns.keys().map(|id| id.0.as_str()).collect();
        ids.sort_unstable();
        for id in ids {
            for b in id.bytes() {
                mix(&mut key, u64::from(b));
            }
        }
    }
    if show_tint {
        if let Some(book) = districts.as_deref() {
            mix(&mut key, book.districts.len() as u64);
            let mut ids: Vec<_> = book.districts.keys().map(|id| id.0.as_str()).collect();
            ids.sort_unstable();
            for id in ids {
                for b in id.bytes() {
                    mix(&mut key, u64::from(b));
                }
                if let Some(zone) = book.districts.values().find(|zone| zone.id.0 == id) {
                    mix(&mut key, zone.ring.len() as u64);
                    mix(&mut key, zone.tile_rect.min.x as u64);
                    mix(&mut key, zone.tile_rect.min.y as u64);
                    mix(&mut key, zone.tile_rect.max.x as u64);
                    mix(&mut key, zone.tile_rect.max.y as u64);
                    for vertex in &zone.ring {
                        mix(&mut key, vertex.x as u64);
                        mix(&mut key, vertex.y as u64);
                    }
                }
            }
        }
    }
    if key == stamp.key {
        return;
    }
    despawn_query(&mut commands, &labels);
    despawn_query(&mut commands, &tints);
    stamp.key = key;
    if let Some(book) = towns.as_deref() {
        let mut rows: Vec<_> = book.towns.values().collect();
        rows.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        for town in rows {
            let label = if town.region_name.is_empty() {
                town.name.as_str()
            } else {
                town.region_name.as_str()
            };
            if label.is_empty() {
                continue;
            }
            let mut at = world_map_tile_center(town.center_tile.x, town.center_tile.y);
            at.z = PLANE_Z + 0.06;
            commands.spawn((
                WorldMapTownLabel,
                Text2d::new(label.to_string()),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::srgb(0.93, 0.95, 0.9)),
                bevy::sprite::Anchor::CENTER,
                Transform::from_translation(at).with_scale(Vec3::splat(1.4 / 32.0)),
            ));
        }
    }
    if !show_tint {
        return;
    }
    let Some(book) = districts.as_deref() else {
        return;
    };
    let mut zones: Vec<_> = book.districts.values().collect();
    zones.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    for zone in zones {
        let rgb = zone_tint_rgb(&zone.id.0);
        let mat = materials.add(ColorMaterial::from_color(Color::srgba(
            rgb[0], rgb[1], rgb[2], 0.22,
        )));
        if zone.ring.len() >= 3 {
            let tiles = zone_shade_interior_tiles(zone);
            if tiles.is_empty() {
                continue;
            }
            let mesh = meshes.add(ring_shade_mesh(&tiles));
            commands.spawn((
                WorldMapZoneTint,
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::from_translation(Vec3::new(0.0, 0.0, PLANE_Z - 0.02)),
            ));
            continue;
        }
        let rect = zone.tile_rect;
        let w = (rect.max.x - rect.min.x).max(1) as f32;
        let h = (rect.max.y - rect.min.y).max(1) as f32;
        let mesh = meshes.add(Rectangle::new(w, h));
        commands.spawn((
            WorldMapZoneTint,
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_translation(Vec3::new(
                rect.min.x as f32 + w * 0.5,
                rect.min.y as f32 + h * 0.5,
                PLANE_Z - 0.02,
            )),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::PathLineRequest;

    #[test]
    fn egr_path_on_world_map_plane_matches_tile_center() {
        let tile = world_map_tile_center(4, 9);
        let mapped = construction_point_on_world_map(Vec3::new(4.5, 0.0, 9.5));
        assert!((mapped.x - tile.x).abs() < 1e-5);
        assert!((mapped.y - tile.y).abs() < 1e-5);
        assert!(mapped.z > tile.z);
        assert!((mapped.z - tile.z) < 0.2);
    }

    #[test]
    fn egr_ribbon_width_is_world_tiles_not_a_pixel_stamp() {
        assert!((world_ribbon_width(64.0) - 0.22).abs() < 1e-5);
        assert!(world_ribbon_width(8.0) < 1.0);
    }

    #[test]
    fn egr_one_road_one_rail_one_zone_spawn_meshes() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            AssetPlugin::default(),
        ));
        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();
        app.init_state::<BaseState>();
        app.insert_resource(ConstructionVisualRequests {
            paths: vec![
                PathLineRequest {
                    kind: VisualPathKind::Road,
                    start: Vec3::new(1.5, 0.0, 2.5),
                    end: Vec3::new(4.5, 0.0, 2.5),
                    width: 8.0,
                    valid: true,
                    slope_ok: true,
                    committed: true,
                },
                PathLineRequest {
                    kind: VisualPathKind::Rail,
                    start: Vec3::new(1.5, 0.0, 5.5),
                    end: Vec3::new(3.5, 0.0, 5.5),
                    width: 8.0,
                    valid: true,
                    slope_ok: true,
                    committed: false,
                },
            ],
            zone_tiles: vec![crate::construction::ZoneTileRequest {
                center: Vec3::new(2.5, 0.0, 8.5),
                color: crate::gui::UiRgba::from_rgba_unmultiplied(40, 80, 160, 140),
            }],
            ..Default::default()
        });
        app.add_systems(Update, sync_world_map_path_meshes);
        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Simulation);
        app.update();
        app.update();
        let mut roads_q = app.world_mut().query_filtered::<Entity, With<WorldMapRoadSeg>>();
        let roads = roads_q.iter(app.world()).count();
        let mut rails_q = app.world_mut().query_filtered::<Entity, With<WorldMapRailSeg>>();
        let rails = rails_q.iter(app.world()).count();
        let mut zones_q = app.world_mut().query_filtered::<Entity, With<WorldMapZoneCell>>();
        let zones = zones_q.iter(app.world()).count();
        assert_eq!(roads, 1);
        assert_eq!(rails, 1);
        assert_eq!(zones, 1);
    }

    #[test]
    fn cityfix_zone_tint_off_hides_tint() {
        use crate::gui::hud::simulation_shell_phase2::ZoneShadeToggle;
        use crate::strategic::settlement::DistrictStyleRules;
        use crate::strategic::{
            DistrictBook, DistrictId, DistrictRecord, TownBook, TownId, ZoningClass,
        };
        use bevy::math::{IRect, IVec2};
        use std::collections::HashMap;

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            AssetPlugin::default(),
        ));
        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();
        app.init_state::<BaseState>();
        app.insert_resource(ZoneShadeToggle { visible: false });
        app.insert_resource(TownBook::default());
        let id = DistrictId("north".into());
        let mut districts = HashMap::new();
        districts.insert(
            id.clone(),
            DistrictRecord {
                id,
                town_id: TownId("town-a".into()),
                name: "North".into(),
                tile_rect: IRect::from_corners(IVec2::new(2, 3), IVec2::new(6, 8)),
                zoning_default: ZoningClass::Residential,
                ring: Vec::new(),
                style_rules: DistrictStyleRules::default(),
                site_ids: Vec::new(),
            },
        );
        app.insert_resource(DistrictBook {
            districts,
            default_district: None,
        });
        app.add_systems(Update, sync_world_map_town_names_and_zone_shade);
        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Simulation);
        app.update();
        app.update();

        let tint_count = |app: &mut App| {
            let mut q = app
                .world_mut()
                .query_filtered::<Entity, With<WorldMapZoneTint>>();
            q.iter(app.world()).count()
        };
        assert_eq!(tint_count(&mut app), 0);
        app.world_mut()
            .resource_mut::<ZoneShadeToggle>()
            .visible = true;
        app.update();
        assert_eq!(tint_count(&mut app), 1);
        app.world_mut()
            .resource_mut::<ZoneShadeToggle>()
            .visible = false;
        app.update();
        assert_eq!(tint_count(&mut app), 0);
    }

    #[test]
    fn twn_ov_draw_commit() {
        use crate::strategic::settlement::{
            add_admin_zone, append_admin_ring_vertex, arm_admin_zone_edit, commit_admin_zone_ring,
            delete_admin_zone, move_admin_ring_vertex, AdminZoneDrag, DistrictBook, InkSite, TownId,
        };
        use crate::gui::hud::simulation_shell_phase2::ZoneShadeToggle;

        let town = TownId("ash".into());
        let mut districts = DistrictBook::default();
        let zone = add_admin_zone(
            &mut districts,
            &town,
            "limit",
            IRect::from_corners(IVec2::ZERO, IVec2::ONE),
        );
        let mut drag = AdminZoneDrag::default();
        drag.zone = Some(zone.clone());
        drag.armed = true;
        for tile in [
            IVec2::new(0, 0),
            IVec2::new(4, 0),
            IVec2::new(0, 4),
        ] {
            append_admin_ring_vertex(&mut drag, tile);
        }
        let sites = vec![InkSite {
            id: 4,
            town_id: town.clone(),
            tile: IVec2::new(1, 1),
        }];
        assert!(commit_admin_zone_ring(
            &mut districts,
            &zone,
            &drag.vertices,
            &sites
        ));
        let drawn = &districts.districts[&zone];
        assert!(drawn.contains_tile(IVec2::new(1, 1)));
        assert!(drawn.site_ids.contains(&4));
        let interior = zone_shade_interior_tiles(drawn);
        assert!(interior.contains(&IVec2::new(0, 0)));
        assert!(interior.contains(&IVec2::new(1, 1)));
        assert!(!interior.contains(&IVec2::new(3, 3)));

        arm_admin_zone_edit(&mut drag, zone.clone(), drawn.ring.clone(), 2);
        move_admin_ring_vertex(&mut drag, 2, IVec2::new(0, 1));
        assert!(commit_admin_zone_ring(
            &mut districts,
            &zone,
            &drag.vertices,
            &sites
        ));
        assert!(!districts.districts[&zone].contains_tile(IVec2::new(0, 3)));

        let building = {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                bevy::state::app::StatesPlugin,
                AssetPlugin::default(),
            ));
            app.init_asset::<Mesh>();
            app.init_asset::<ColorMaterial>();
            app.init_state::<BaseState>();
            app.insert_resource(ZoneShadeToggle { visible: true });
            app.insert_resource(crate::strategic::TownBook::default());
            app.insert_resource(districts.clone());
            app.add_systems(Update, sync_world_map_town_names_and_zone_shade);
            let building = app.world_mut().spawn_empty().id();
            app.world_mut()
                .resource_mut::<NextState<BaseState>>()
                .set(BaseState::Simulation);
            app.update();
            app.update();
            let mut tints = app
                .world_mut()
                .query_filtered::<Entity, With<WorldMapZoneTint>>();
            assert_eq!(tints.iter(app.world()).count(), 1);
            let mut cells = app
                .world_mut()
                .query_filtered::<Entity, With<WorldMapZoneCell>>();
            assert_eq!(cells.iter(app.world()).count(), 0);
            app.world_mut()
                .resource_mut::<ZoneShadeToggle>()
                .visible = false;
            app.update();
            let mut tints = app
                .world_mut()
                .query_filtered::<Entity, With<WorldMapZoneTint>>();
            assert_eq!(tints.iter(app.world()).count(), 0);
            assert!(app.world().get_entity(building).is_ok());
            building
        };
        assert!(delete_admin_zone(&mut districts, &zone));
        assert!(!districts.districts.contains_key(&zone));
        let _still = building;
    }
}
