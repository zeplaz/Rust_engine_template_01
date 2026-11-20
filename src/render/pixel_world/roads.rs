//! Voxel-view road surface: the committed roads as drawn (ribbons on the ground, decks over
//! water), published once by the path layer. Readers: the pointer gate (a click on a bridge deck
//! hits the deck, not the riverbed behind it) and the truck layer (trucks sit on the road
//! surface, on the deck height over water).

use bevy::prelude::*;

use super::camera::PixelWorldCameraFrame;
use super::field::PixelWorldTerrainField;
use super::layers::{layer, mix64, p3, pixel_world_on, shade, PATH_LIFT};
use super::terrain::MeshBuf;
use crate::construction::ConstructionVisualRequests;
use crate::gui::WorldPresentationMode;
use crate::presentation::TruckPresentation;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// One drawn committed road segment (tactical tile XY).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoadSurfaceSeg {
    pub a: Vec2,
    pub b: Vec2,
    pub half_width: f32,
    /// Deck heights at `a` / `b` when the segment is a bridge over water.
    pub deck: Option<(f32, f32)>,
}

/// Committed roads as drawn in the voxel view (the path layer publishes, others read).
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct PixelWorldRoadSurface(pub Vec<RoadSurfaceSeg>);

impl PixelWorldRoadSurface {
    /// Deck top height under `p` when it lies on a bridge deck.
    #[must_use]
    pub fn deck_height_at(&self, p: Vec2) -> Option<f32> {
        self.0.iter().find_map(|s| {
            let (ha, hb) = s.deck?;
            let d = s.b - s.a;
            let len2 = d.length_squared();
            if len2 < 1e-6 {
                return None;
            }
            let t = (p - s.a).dot(d) / len2;
            let lateral = (p - (s.a + d * t.clamp(0.0, 1.0))).length();
            ((0.0..=1.0).contains(&t) && lateral <= s.half_width + 0.1).then(|| ha + (hb - ha) * t)
        })
    }
}

/// Nearest ray entry into a bridge deck: `(t, tactical xy on the road centreline)`. `slack` widens
/// the deck sideways (tiles) so a thin road stays clickable.
#[must_use]
pub fn ray_hit_road_decks(road: &PixelWorldRoadSurface, origin: Vec3, dir: Vec3, slack: f32) -> Option<(f32, Vec2)> {
    let mut best: Option<(f32, Vec2)> = None;
    for s in &road.0 {
        let Some((ha, hb)) = s.deck else { continue };
        let d = s.b - s.a;
        let len = d.length();
        if len < 1e-3 {
            continue;
        }
        let perp = Vec2::new(-d.y, d.x) / len;
        // World: x = tile x, y = height, z = -tile y.
        let a_w = Vec3::new(s.a.x, ha, -s.a.y);
        let u_w = Vec3::new(d.x, hb - ha, -d.y);
        let v_w = Vec3::new(perp.x, 0.0, -perp.y);
        let n = u_w.cross(v_w);
        let denom = dir.dot(n);
        if denom.abs() < 1e-8 {
            continue;
        }
        let t = (a_w - origin).dot(n) / denom;
        if t <= 0.0 || best.is_some_and(|(bt, _)| bt <= t) {
            continue;
        }
        let rel = origin + dir * t - a_w;
        let along = rel.dot(u_w) / u_w.length_squared();
        let lateral = rel.dot(v_w);
        if (0.0..=1.0).contains(&along) && lateral.abs() <= s.half_width + slack {
            best = Some((t, s.a + d * along));
        }
    }
    best
}

/// Publish every committed road as drawn (same segments and bridge test as the path layer).
pub(crate) fn publish_pixel_world_road_surface(
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    requests: Option<Res<ConstructionVisualRequests>>,
    bridges: Option<Res<crate::construction::bridges::BridgeBook>>,
    mut out: ResMut<PixelWorldRoadSurface>,
) {
    let mut segs = Vec::new();
    if let (true, Some(req)) = (pixel_world_on(&mode, &cam, &field), requests) {
        for p in req.paths.iter().filter(|p| p.committed) {
            let a = crate::gui::tactical::map_camera::map_plane_horizontal_xy(p.start);
            let b = crate::gui::tactical::map_camera::map_plane_horizontal_xy(p.end);
            if (b - a).length_squared() < 1e-6 {
                continue;
            }
            segs.push(RoadSurfaceSeg {
                a,
                b,
                half_width: crate::render::world_map_network_mesh::world_ribbon_width(p.width) * 0.5,
                // Same deck the path layer draws; a failed bridge has none to stand on.
                deck: super::layers::committed_deck_heights(&field, bridges.as_deref(), a, b).filter(|_| {
                    !bridges.as_deref().and_then(|bk| bk.on_stretch(a, b)).is_some_and(|r| r.fallen_sections() > 0)
                }),
            });
        }
    }
    if out.0 != segs {
        out.0 = segs;
    }
}

#[derive(Component)]
pub struct PixelWorldTruckLayer;

#[derive(Default)]
pub(crate) struct TruckLayerState {
    key: Option<u64>,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    entity: Option<Entity>,
}

/// Order-sim trucks as boxes standing on the road surface (deck height on a bridge).
pub(crate) fn sync_pixel_world_trucks(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    road: Res<PixelWorldRoadSurface>,
    trucks: Query<&Transform, With<TruckPresentation>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: Local<TruckLayerState>,
) {
    if !pixel_world_on(&mode, &cam, &field) {
        if let Some(e) = state.entity {
            commands.entity(e).try_insert(Visibility::Hidden);
        }
        state.key = None;
        return;
    }
    let tm = TILE_METRES_GAMEPLAY.max(1e-3);
    let placed: Vec<(Vec2, Vec2)> = trucks
        .iter()
        .map(|t| {
            let (yaw, _, _) = t.rotation.to_euler(EulerRot::ZYX);
            (t.translation.truncate() / tm, Vec2::new(yaw.cos(), yaw.sin()))
        })
        .collect();
    let q = |v: f32| (v * 16.0).round() as i64 as u64;
    let mut key = mix64(field.revision ^ (road.0.len() as u64) << 24 ^ (placed.len() as u64) << 40);
    for (p, d) in &placed {
        key = mix64(key ^ q(p.x) ^ q(p.y) << 20 ^ q(d.x) << 40 ^ q(d.y) << 52);
    }
    if state.key == Some(key) {
        return;
    }
    state.key = Some(key);

    let mut buf = MeshBuf::default();
    let (top, side) = (shade([0.85, 0.65, 0.15], 1.0, 1.0), shade([0.60, 0.45, 0.10], 1.0, 1.0));
    for (p, dir) in &placed {
        // Sit on the road: the deck over water, else the same draped height the ribbon uses.
        let base = road
            .deck_height_at(*p)
            .unwrap_or_else(|| field.surface_max_around(*p) + PATH_LIFT);
        let (f, s) = (*dir * 0.35, Vec2::new(-dir.y, dir.x) * 0.17);
        let (lo, hi) = (base + 0.02, base + 0.34);
        let c = [p - f - s, p + f - s, p + f + s, p - f + s];
        buf.quad([p3(c[0].x, c[0].y, hi), p3(c[1].x, c[1].y, hi), p3(c[2].x, c[2].y, hi), p3(c[3].x, c[3].y, hi)], Vec3::Y, [Vec2::ZERO; 4], top);
        for k in 0..4 {
            let (u, v) = (c[k], c[(k + 1) % 4]);
            let n = Vec2::new(v.y - u.y, u.x - v.x).normalize_or_zero();
            buf.quad(
                [p3(u.x, u.y, lo), p3(v.x, v.y, lo), p3(v.x, v.y, hi), p3(u.x, u.y, hi)],
                Vec3::new(n.x, 0.0, -n.y),
                [Vec2::ZERO; 4],
                side,
            );
        }
    }

    let material = state
        .material
        .get_or_insert_with(|| materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, double_sided: true, cull_mode: None, ..default() }))
        .clone();
    let before = state.mesh.as_ref().map(Handle::id);
    let drawable = !buf.is_empty() && crate::render::upload_layer_mesh(&mut meshes, &mut state.mesh, buf.into_mesh());
    let vis = if drawable { Visibility::Inherited } else { Visibility::Hidden };
    let Some(mesh) = state.mesh.clone() else {
        return;
    };
    match state.entity {
        Some(e) => {
            if before != Some(mesh.id()) {
                commands.entity(e).try_insert((Mesh3d(mesh), vis));
            } else {
                commands.entity(e).try_insert(vis);
            }
        }
        None => {
            state.entity = Some(
                commands
                    .spawn((PixelWorldTruckLayer, Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, vis, layer(), Pickable::IGNORE))
                    .id(),
            );
        }
    }
}
