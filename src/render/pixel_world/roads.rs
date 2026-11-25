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

/// Points `back` tile units behind `pos` along `points` (the route driven, tile units), each with
/// its heading; falls back to straight behind along `dir` when there is no route (RD-7 wagons
/// follow the track round curves).
#[must_use]
pub fn trail_along(points: &[Vec2], pos: Vec2, dir: Vec2, back: &[f32]) -> Vec<(Vec2, Vec2)> {
    let straight = || back.iter().map(|b| (pos - dir * *b, dir)).collect();
    if points.len() < 2 {
        return straight();
    }
    // Nearest stretch to the engine, and how far along it the engine is.
    let mut best = (f32::INFINITY, 0usize, 0.0f32);
    for (i, w) in points.windows(2).enumerate() {
        let d = w[1] - w[0];
        let len2 = d.length_squared().max(1e-6);
        let t = ((pos - w[0]).dot(d) / len2).clamp(0.0, 1.0);
        let dist = (w[0] + d * t).distance(pos);
        if dist < best.0 {
            best = (dist, i, t * len2.sqrt());
        }
    }
    if best.0 > 2.0 {
        return straight();
    }
    back.iter()
        .map(|b| {
            let (mut i, mut along, mut left) = (best.1, best.2, *b);
            loop {
                let (a, c) = (points[i], points[i + 1]);
                let seg = a.distance(c).max(1e-6);
                if left <= along || i == 0 {
                    let s = (along - left).max(0.0);
                    let h = (c - a) / seg;
                    return (a + h * s, h);
                }
                left -= along;
                i -= 1;
                along = points[i].distance(points[i + 1]);
            }
        })
        .collect()
}

/// Order-sim trucks as boxes standing on the road surface (deck height on a bridge); trains as an
/// engine with its wagons trailing along the track (RD-7).
pub(crate) fn sync_pixel_world_trucks(
    mut commands: Commands,
    mode: Option<Res<WorldPresentationMode>>,
    cam: Res<PixelWorldCameraFrame>,
    field: Res<PixelWorldTerrainField>,
    road: Res<PixelWorldRoadSurface>,
    trucks: Query<(&Transform, &TruckPresentation, Option<&crate::entities::components::Waypoints>)>,
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
    // (position, heading, half length, colour index): trucks 1 box; trains an engine + wagons.
    const CAR_GAP: f32 = 0.95;
    let mut placed: Vec<(Vec2, Vec2, f32, u8)> = Vec::new();
    for (t, tp, wp) in &trucks {
        let (yaw, _, _) = t.rotation.to_euler(EulerRot::ZYX);
        let (pos, dir) = (t.translation.truncate() / tm, Vec2::new(yaw.cos(), yaw.sin()));
        if tp.wagons == 0 {
            placed.push((pos, dir, 0.35, 0));
            continue;
        }
        let route: Vec<Vec2> = wp.map(|w| w.points.iter().map(|p| *p / tm).collect()).unwrap_or_default();
        let backs: Vec<f32> = (1..=tp.wagons).map(|k| CAR_GAP * f32::from(k)).collect();
        placed.push((pos, dir, 0.48, 1));
        for (p, h) in trail_along(&route, pos, dir, &backs) {
            placed.push((p, h, 0.42, 2));
        }
    }
    let q = |v: f32| (v * 16.0).round() as i64 as u64;
    let mut key = mix64(field.revision ^ (road.0.len() as u64) << 24 ^ (placed.len() as u64) << 40);
    for (p, d, _, _) in &placed {
        key = mix64(key ^ q(p.x) ^ q(p.y) << 20 ^ q(d.x) << 40 ^ q(d.y) << 52);
    }
    if state.key == Some(key) {
        return;
    }
    state.key = Some(key);

    let mut buf = MeshBuf::default();
    let colours = [
        (shade([0.85, 0.65, 0.15], 1.0, 1.0), shade([0.60, 0.45, 0.10], 1.0, 1.0)), // truck
        (shade([0.62, 0.16, 0.12], 1.0, 1.0), shade([0.42, 0.10, 0.08], 1.0, 1.0)), // engine
        (shade([0.45, 0.40, 0.34], 1.0, 1.0), shade([0.30, 0.27, 0.23], 1.0, 1.0)), // wagon
    ];
    for (p, dir, half_len, c) in &placed {
        let (top, side) = colours[usize::from(*c)];
        // Sit on the road: the deck over water, else the same draped height the ribbon uses.
        let base = road
            .deck_height_at(*p)
            .unwrap_or_else(|| field.surface_max_around(*p) + PATH_LIFT);
        let (f, s) = (*dir * *half_len, Vec2::new(-dir.y, dir.x) * if *c == 0 { 0.17 } else { 0.22 });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wagons_trail_along_the_track_round_a_bend() {
        // An L-shaped track: east along y = 0, then north along x = 10. Engine at (10, 3).
        let route = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let cars = trail_along(&route, Vec2::new(10.0, 3.0), Vec2::Y, &[1.0, 5.0]);
        assert!((cars[0].0 - Vec2::new(10.0, 2.0)).length() < 1e-3, "{cars:?}");
        assert!((cars[1].0 - Vec2::new(8.0, 0.0)).length() < 1e-3, "round the bend: {cars:?}");
        assert!((cars[1].1 - Vec2::X).length() < 1e-3, "heading of the first stretch");
        // No route: straight behind.
        let straight = trail_along(&[], Vec2::ZERO, Vec2::X, &[2.0]);
        assert_eq!(straight[0].0, Vec2::new(-2.0, 0.0));
    }
}
