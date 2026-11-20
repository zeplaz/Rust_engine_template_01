//! Roads + trucks dump section (debug only, counts no verdicts): the committed road network as
//! drawn, whether each drawn segment answers the same picks a click runs (2D segment pick, 3D
//! deck ray), and where each truck proxy drives (built road vs cross-country, on the deck).

use bevy::prelude::*;
use serde_json::json;

use crate::construction::CorridorNetwork;
use crate::dev::sim_run_dump::{SimDumpArm, SimRunDump};
use crate::entities::components::Waypoints;
use crate::gui::hud::road_info_card::pick_road_segment;
use crate::presentation::TruckPresentation;
use crate::render::pixel_world::{ray_hit_road_decks, PixelWorldRoadSurface};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Writes the `roads` section.
pub fn dump_roads_section(
    arm: Res<SimDumpArm>,
    roads: Option<Res<CorridorNetwork>>,
    surface: Option<Res<PixelWorldRoadSurface>>,
    trucks: Query<(&Transform, &Waypoints), With<TruckPresentation>>,
    legs: Option<Res<crate::economy::logistics::orders::SiteLegRouteBook>>,
    placements: Option<Res<crate::scenario::place_corridor::CorridorPlacementLog>>,
    bridges: Option<Res<crate::construction::bridges::BridgeBook>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let tm = TILE_METRES_GAMEPLAY.max(1e-3);
    let empty = CorridorNetwork::default();
    let roads = roads.as_deref().unwrap_or(&empty);
    let c = |t: crate::strategic::BuildSiteTile| Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5);
    let segs: Vec<(Vec2, Vec2)> = roads.road_spans().map(|(_, a, b)| (c(a), c(b))).collect();
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
    // Planned legs: metres on roads vs off-road, and trip time vs the old straight-line estimate
    // (straight x 1.3 at the same 12 m/s truck).
    let offroad = crate::infrastructure::road_tier_table().offroad_speed_mps();
    let (mut road_m, mut off_m) = (0.0_f32, 0.0_f32);
    let leg_rows: Vec<_> = legs
        .as_deref()
        .map(|b| {
            b.legs
                .iter()
                .map(|((from, to), l)| {
                    let (r, o) = l.route.road_offroad_m(offroad);
                    road_m += r;
                    off_m += o;
                    let straight = match (l.route.points.first(), l.route.points.last()) {
                        (Some(a), Some(z)) => a.distance(*z) * tm,
                        _ => 0.0,
                    };
                    json!({
                        "from": from.0, "to": to.0, "road_m": r, "offroad_m": o,
                        "time_s_12mps": l.route.time_s(12.0),
                        "old_estimate_s": straight * 1.3 / 12.0,
                        "points": l.route.points.len(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    dump.put_value(
        "roads",
        json!({
            "road_segments": roads.segments.iter().filter(|s| !s.kind.is_rail()).count(),
            "rail_segments": roads.segments.iter().filter(|s| s.kind.is_rail()).count(),
            "tiers": roads.segments.iter().filter_map(|s| s.kind.road_tier()).map(|t| t.label()).collect::<Vec<_>>(),
            "spans": segs.len(),
            "drawn_segments": drawn.len(),
            "deck_segments": deck_segs,
            "pick_2d_ok": pick_2d_ok,
            "deck_ray_ok": deck_ray_ok,
            "trucks": rows.len(),
            "trucks_on_road": on_road,
            "legs_planned": leg_rows.len(),
            "leg_road_share": if road_m + off_m > 0.0 { road_m / (road_m + off_m) } else { 0.0 },
            "leg_rows": leg_rows.into_iter().take(16).collect::<Vec<_>>(),
            "bridges": bridges.as_deref().map(|b| b.bridges.iter().map(|r| json!({
                "segment": r.segment_id, "a": [r.a.x, r.a.y], "b": [r.b.x, r.b.y],
                "type": r.bridge_type.label(), "length_m": r.length_m(), "deck_m": r.deck_m,
                "sections": r.sections.len(), "fallen": r.fallen_sections(), "min_integrity": r.integrity(),
                "hits": r.sections.iter().map(|s| s.hits.len()).sum::<usize>(),
                "overtopped": r.overtopped, "peak_load": r.peak_load,
                "section_rows": r.sections.iter().map(|s| json!({
                    "integrity": s.integrity, "fallen": s.fallen,
                    "dropped_by": s.dropped_by.map(|c| c.label()),
                    "blast_loss": s.blast_loss, "flood_loss": s.flood_loss, "fire_loss": s.fire_loss,
                    "hits": s.hits.len(),
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>()).unwrap_or_default(),
            "placements": placements.as_deref().map(|l| l.0.iter().map(|r| json!({
                "kind": r.kind, "queued": r.queued, "length_m": r.length_m,
                "check_us": r.check_us as u64, "stretches": r.stretches,
            })).collect::<Vec<_>>()).unwrap_or_default(),
            "truck_rows": rows.into_iter().take(12).collect::<Vec<_>>(),
        }),
    );
}
