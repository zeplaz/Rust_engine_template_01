//! Committed domain snapshots for logistics + ecology (Stage 5 spine).

use bevy::prelude::*;

use crate::render::sim_visual_extract::FireVisualFrame;
use crate::render::{ClimateVisualAggregate, EcologyVisualSnapshot, LogisticsVisualSnapshot};
use crate::strategic::CorridorConstructionBook;

fn fill_logistics_snapshot(
    fire: &FireVisualFrame,
    book: Option<&CorridorConstructionBook>,
    snapshot: &mut LogisticsVisualSnapshot,
) {
    snapshot.stamp = fire.stamp;
    snapshot.edge_rows.clear();
    if let Some(book) = book {
        let mut edges: Vec<_> = book.by_edge.iter().collect();
        edges.sort_by_key(|(id, _)| id.0);
        for (eid, status) in edges {
            snapshot
                .edge_rows
                .push((eid.0 as u32, status.traffic_factor()));
        }
    }
    snapshot.corridor_revision = snapshot.edge_rows.len() as u64;
    snapshot.active_overlay_rows = snapshot.edge_rows.len() as u32;
}

fn fill_ecology_snapshot(
    fire: &FireVisualFrame,
    climate: &ClimateVisualAggregate,
    snapshot: &mut EcologyVisualSnapshot,
) {
    snapshot.stamp = fire.stamp;
    snapshot.ecology_chunk_count = climate.ecology_chunk_count;
    snapshot.mean_biomass = climate.mean_biomass;
    snapshot.mean_fire_risk = climate.mean_fire_risk;
    snapshot.chunk_rows.clear();
    let rows = climate.ecology_chunk_count.max(1) as usize;
    snapshot.chunk_rows.reserve(rows);
    for i in 0..rows {
        snapshot.chunk_rows.push(Vec4::new(
            i as f32,
            climate.mean_biomass,
            climate.mean_fire_risk,
            fire.stamp.tick as f32,
        ));
    }
}

pub fn publish_logistics_visual_snapshot(
    fire: Res<FireVisualFrame>,
    book: Option<Res<CorridorConstructionBook>>,
    mut snapshot: ResMut<LogisticsVisualSnapshot>,
) {
    fill_logistics_snapshot(&fire, book.as_deref(), &mut snapshot);
}

pub fn publish_ecology_visual_snapshot(
    fire: Res<FireVisualFrame>,
    climate: Res<ClimateVisualAggregate>,
    mut snapshot: ResMut<EcologyVisualSnapshot>,
) {
    fill_ecology_snapshot(&fire, &climate, &mut snapshot);
}

mod tests {
    use super::*;
    use crate::systems::sim_control::SimStepStamp;
    use crate::systems::transport::TransportEdgeId;

    #[test]
    fn logistics_snapshot_tracks_corridor_rows() {
        let fire = FireVisualFrame {
            stamp: SimStepStamp::new(3, 0),
            ..Default::default()
        };
        let mut book = CorridorConstructionBook::default();
        book.by_edge.insert(
            TransportEdgeId(1),
            crate::strategic::CorridorConstructionStatus::default(),
        );
        let mut snapshot = LogisticsVisualSnapshot::default();
        fill_logistics_snapshot(&fire, Some(&book), &mut snapshot);
        assert_eq!(snapshot.stamp.tick, 3);
        assert_eq!(snapshot.active_overlay_rows, 1);
        assert_eq!(snapshot.edge_rows.len(), 1);
        assert!((snapshot.edge_rows[0].1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ecology_snapshot_copies_climate_aggregate() {
        let fire = FireVisualFrame {
            stamp: SimStepStamp::new(4, 0),
            ..Default::default()
        };
        let climate = ClimateVisualAggregate {
            ecology_chunk_count: 5,
            mean_biomass: 0.42,
            mean_fire_risk: 0.11,
            ..Default::default()
        };
        let mut snapshot = EcologyVisualSnapshot::default();
        fill_ecology_snapshot(&fire, &climate, &mut snapshot);
        assert_eq!(snapshot.ecology_chunk_count, 5);
        assert!((snapshot.mean_biomass - 0.42).abs() < 1e-5);
        assert_eq!(snapshot.chunk_rows.len(), 5);
    }
}
