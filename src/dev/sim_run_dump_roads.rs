//! Roads + trucks dump section (debug only, counts no verdicts): the committed road network as
//! drawn, whether each drawn segment answers the same picks a click runs (2D segment pick, 3D
//! deck ray), and where each truck proxy drives (built road vs cross-country, on the deck).

use bevy::prelude::*;
use serde_json::json;

use crate::construction::ExecutedRoadNetwork;
use crate::dev::sim_run_dump::{SimDumpArm, SimRunDump};
use crate::entities::components::Waypoints;
use crate::gui::hud::road_info_card::pick_road_segment;
use crate::presentation::TruckPresentation;
use crate::render::pixel_world::{ray_hit_road_decks, PixelWorldRoadSurface};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Writes the `roads` section.
pub fn dump_roads_section(
    arm: Res<SimDumpArm>,
    roads: Option<Res<ExecutedRoadNetwork>>,
    surface: Option<Res<PixelWorldRoadSurface>>,
    trucks: Query<(&Transform, &Waypoints), With<TruckPresentation>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let tm = TILE_METRES_GAMEPLAY.max(1e-3);
    let empty = ExecutedRoadNetwork::default();
    let roads = roads.as_deref().unwrap_or(&empty);
    let segs: Vec<(Vec2, Vec2)> = roads
        .tiles
        .windows(2)
        .filter(|w| w[0] != w[1])
        .map(|w| (Vec2::new(w[0].x as f32 + 0.5, w[0].z as f32 + 0.5), Vec2::new(w[1].x as f32 + 0.5, w[1].z as f32 + 0.5)))
        .collect();
    let drawn = surface.as_deref().map_or(&[][..], |s| &s.0[..]);
    // The picks a click runs, at each drawn segment's midpoint.
    let mut pick_2d_ok = 0;
    let mut deck_ray_ok = 0;
    let mut deck_segs = 0;
    for s in drawn {
        let mid = (s.a + s.b) * 0.5;
        if pick_road_segment(roads, mid, 0.35).is_some() {
            pick_2d_ok += 1;
        }
        if let (Some((ha, hb)), Some(surf)) = (s.deck, surface.as_deref()) {
            deck_segs += 1;
            let top = (ha + hb) * 0.5 + 40.0;
            // Oblique ray down onto the deck midpoint, as from a tilted camera.
            let origin = Vec3::new(mid.x - 6.0, top, -(mid.y - 6.0));
            let target = Vec3::new(mid.x, (ha + hb) * 0.5, -mid.y);
            if let Some((_, hit)) = ray_hit_road_decks(surf, origin, (target - origin).normalize(), 0.3) {
                if hit.distance(mid) < 1.0 {
                    deck_ray_ok += 1;
                }
            }
        }
    }
    let road_tile = |p: Vec2| segs.iter().any(|(a, b)| p.distance(*a) < 0.05 || p.distance(*b) < 0.05);
    let rows: Vec<_> = trucks
        .iter()
        .map(|(tf, wp)| {
            let pts: Vec<Vec2> = wp.points.iter().map(|p| *p / tm).collect();
            let inner = if pts.len() > 2 { &pts[1..pts.len() - 1] } else { &[][..] };
            let on_road_nodes = inner.iter().filter(|p| road_tile(**p)).count();
            let pos = tf.translation.truncate() / tm;
            json!({
                "pos_tiles": [pos.x, pos.y],
                "waypoints": pts.len(),
                "road_nodes": on_road_nodes,
                "route": if on_road_nodes > 0 { "road" } else { "cross_country" },
                "on_deck": surface.as_deref().and_then(|s| s.deck_height_at(pos)),
                "from": pts.first().map(|p| [p.x, p.y]),
                "to": pts.last().map(|p| [p.x, p.y]),
            })
        })
        .collect();
    let on_road = rows.iter().filter(|r| r["route"] == "road").count();
    dump.put_value(
        "roads",
        json!({
            "road_tiles": roads.tiles.len(),
            "segments": segs.len(),
            "drawn_segments": drawn.len(),
            "deck_segments": deck_segs,
            "pick_2d_ok": pick_2d_ok,
            "deck_ray_ok": deck_ray_ok,
            "trucks": rows.len(),
            "trucks_on_road": on_road,
            "truck_rows": rows.into_iter().take(12).collect::<Vec<_>>(),
        }),
    );
}
