//! Corridor works + road traffic dump sections (debug only, counts no verdicts): each placed
//! line's material bill and what was delivered (construction), and road volume / capacity per
//! edge (logistics, RD-4). One system per domain.

use bevy::prelude::*;
use serde_json::json;

use crate::construction::corridor_works::CorridorWorksBook;
use crate::dev::sim_run_dump::{SimDumpArm, SimRunDump};
use crate::economy::logistics::road_flow::{bpr_time_mul, RoadFlowBook};

/// Writes the `corridor_works` section.
pub fn dump_corridor_works_section(arm: Res<SimDumpArm>, works: Option<Res<CorridorWorksBook>>, mut dump: ResMut<SimRunDump>) {
    if !arm.due {
        return;
    }
    let Some(works) = works else {
        return;
    };
    dump.put_value(
        "corridor_works",
        json!({
            "lines": works.works.len(),
            "supplied": works.works.values().filter(|w| w.supplied()).count(),
            "ordered_total": works.ordered_total,
            "rows": works.works.values().take(16).map(|w| json!({
                "segment": w.segment_id,
                "length_m": w.length_m,
                "delivered_ratio": w.delivered_ratio(),
                "source": w.last_source.map(|s| s.0),
                "works_site": w.site.map(|s| s.0),
                "closed": w.closed,
                "bill": w.bill.iter().map(|m| json!({"cargo": m.cargo, "needed": m.needed, "delivered": m.delivered})).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        }),
    );
}

/// Writes the `offroad_field` section (RD-2): map size, wall tiles, mean speed multiplier.
pub fn dump_offroad_field_section(
    arm: Res<SimDumpArm>,
    field: Option<Res<crate::systems::navigation::offroad_speed::OffroadSpeedField>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let Some(field) = field else {
        return;
    };
    let passable: Vec<f32> = field.mul.iter().copied().filter(|m| *m > 0.0).collect();
    let mean = if passable.is_empty() { 0.0 } else { passable.iter().sum::<f32>() / passable.len() as f32 };
    let slow = passable.iter().filter(|m| **m < 0.5).count();
    dump.put_value(
        "offroad_field",
        json!({
            "width": field.width,
            "height": field.height,
            "walls": field.walls,
            "slow_tiles": slow,
            "mean_mul": mean,
            "revision": field.revision,
            "rebuilds": field.rebuilds,
        }),
    );
}

/// Writes the `track_wear` section (RD-5): worn tiles, the most-worn tile, tracks formed.
pub fn dump_track_wear_section(
    arm: Res<SimDumpArm>,
    book: Option<Res<crate::economy::logistics::track_wear::TrackWearBook>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let Some(book) = book else {
        return;
    };
    let peak = book.wear.values().copied().fold(0.0_f32, f32::max);
    let to_form = crate::infrastructure::road_tier_table().worn_tracks.passes_to_form;
    dump.put_value(
        "track_wear",
        json!({
            "worn_tiles": book.wear.len(),
            "at_threshold": book.wear.values().filter(|w| **w >= to_form).count(),
            "peak_passes": peak,
            "passes_to_form": to_form,
            "tracks_formed": book.tracks_formed,
            "formed_tiles": book.formed.len(),
        }),
    );
}

/// Writes the `road_flow` section.
pub fn dump_road_flow_section(arm: Res<SimDumpArm>, flow: Option<Res<RoadFlowBook>>, mut dump: ResMut<SimRunDump>) {
    if !arm.due {
        return;
    }
    let Some(flow) = flow else {
        return;
    };
    let mut rows: Vec<_> = flow.by_edge.iter().map(|(id, f)| (id.0, f.veh_h, f.cap_veh_h, f.vc())).collect();
    rows.sort_by(|a, b| b.3.total_cmp(&a.3));
    dump.put_value(
        "road_flow",
        json!({
            "edges_with_traffic": rows.len(),
            "max_vc": flow.max_vc(),
            "max_time_mul": bpr_time_mul(flow.max_vc()),
            "revision": flow.revision,
            "peak_vc": flow.peak_vc,
            "peak_time_mul": bpr_time_mul(flow.peak_vc),
            "peak_edges_with_traffic": flow.peak_edges,
            "busiest": rows.iter().take(8).map(|(id, v, c, vc)| json!({"edge": id, "veh_h": v, "cap_veh_h": c, "vc": vc})).collect::<Vec<_>>(),
        }),
    );
}
