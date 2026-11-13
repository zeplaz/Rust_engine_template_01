//! LOG-D-05 logistics diagnostics panel feed (top saturated edges, starved facilities).

use bevy::prelude::*;

use crate::systems::transport::TransportEdgeDirectory;

use super::types::{LogisticsDiagnostics, ThroughputSolverState};

const LOG_D_05_TOP_EDGE_LIMIT: usize = 5;

/// Collect top saturated edges + starved facility catalog ids for the diagnostics panel (LOG-D-05).
pub fn collect_logistics_diagnostics_panel_system(
    solver: Res<ThroughputSolverState>,
    directory: Res<TransportEdgeDirectory>,
    facilities: Query<(
        &crate::economy::resource_flow::ResourceFlowNode,
        &crate::economy::resource_flow::FacilityFlowState,
    )>,
    mut diagnostics: ResMut<LogisticsDiagnostics>,
) {
    let mut ranked: Vec<(u64, f32, f32, f32)> = directory
        .by_edge
        .keys()
        .filter_map(|eid| {
            let idx = eid.0 as usize;
            let pressure = solver.edge_pressure.get(idx).copied().unwrap_or(0.0);
            if pressure <= 0.05 {
                return None;
            }
            Some((
                eid.0,
                pressure,
                solver.load.get(idx).copied().unwrap_or(0.0),
                solver.capacity.get(idx).copied().unwrap_or(0.0),
            ))
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(LOG_D_05_TOP_EDGE_LIMIT);
    diagnostics.top_saturated_edges = ranked
        .into_iter()
        .map(
            |(edge_id, pressure, load, capacity)| super::types::SaturatedEdgeSample {
                edge_id,
                pressure,
                load,
                capacity,
            },
        )
        .collect();

    diagnostics.starved_facilities.clear();
    for (node, flow) in facilities.iter() {
        if flow.starved {
            diagnostics.starved_facilities.push(node.catalog_id.clone());
        }
    }
    diagnostics.starved_facilities.sort();
    diagnostics.starved_facilities.dedup();
}
