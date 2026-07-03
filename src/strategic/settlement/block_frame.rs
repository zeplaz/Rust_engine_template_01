//! **CITY-G1-C2-001** — BlockFrame from BlockBook + transport junctions.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::infrastructure::transport::junction::JunctionKind;
use crate::infrastructure::transport::TransportGraph;

use super::block::{BlockBook, BlockRecord};
use super::block_archetype::BlockArchetype;
use super::ids::BlockId;

pub const CITY_G1_C2_LIVE_JSON: &str = "debug_runs/city_g1_c2_001_live.json";

/// Which block edge faces the nearest street / junction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StreetSide {
    PosX,
    NegX,
    PosZ,
    NegZ,
}

impl StreetSide {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PosX => "+X",
            Self::NegX => "-X",
            Self::PosZ => "+Z",
            Self::NegZ => "-Z",
        }
    }
}

/// Anchor-tiled frame for block-relative lot placement (CITY-C2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockFrame {
    pub block_id: BlockId,
    pub anchor: IVec2,
    pub extent: UVec2,
    pub street_side: StreetSide,
    /// Clockwise quarter-turns from +X street axis (design §6).
    pub orientation_quarter_turns: u8,
    pub junction_tile: Option<IVec2>,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct BlockFrameBook {
    pub frames: HashMap<BlockId, BlockFrame>,
}

#[must_use]
pub fn block_bounds_from_tiles(tiles: &HashSet<IVec2>) -> Option<(IVec2, UVec2)> {
    if tiles.is_empty() {
        return None;
    }
    let mut min = IVec2::new(i32::MAX, i32::MAX);
    let mut max = IVec2::new(i32::MIN, i32::MIN);
    for &t in tiles {
        min = min.min(t);
        max = max.max(t);
    }
    let extent = UVec2::new(
        (max.x - min.x + 1).max(1) as u32,
        (max.y - min.y + 1).max(1) as u32,
    );
    Some((min, extent))
}

#[inline]
fn vec3_to_tile(pos: Vec3) -> IVec2 {
    IVec2::new(pos.x.round() as i32, pos.z.round() as i32)
}

#[must_use]
fn tile_centroid(tiles: &HashSet<IVec2>) -> Vec2 {
    if tiles.is_empty() {
        return Vec2::ZERO;
    }
    let sum = tiles.iter().copied().fold(IVec2::ZERO, |a, b| a + b);
    let n = tiles.len() as f32;
    Vec2::new(sum.x as f32 / n, sum.y as f32 / n)
}

#[must_use]
fn junction_priority(kind: JunctionKind) -> u8 {
    match kind {
        JunctionKind::Junction { degree } => degree.saturating_add(10),
        JunctionKind::PassThrough => 5,
        JunctionKind::Endpoint => 1,
    }
}

#[must_use]
pub fn nearest_junction_tile(graph: &TransportGraph, centroid: Vec2) -> Option<(IVec2, JunctionKind)> {
    graph
        .nodes
        .values()
        .map(|node| {
            let tile = vec3_to_tile(node.position);
            let dx = tile.x as f32 - centroid.x;
            let dz = tile.y as f32 - centroid.y;
            (tile, node.junction_kind, dx * dx + dz * dz)
        })
        .min_by(|a, b| {
            a.2.partial_cmp(&b.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| junction_priority(b.1).cmp(&junction_priority(a.1)))
        })
        .map(|(tile, kind, _)| (tile, kind))
}

#[must_use]
pub fn street_side_from_junction(anchor: IVec2, extent: UVec2, junction_tile: IVec2) -> StreetSide {
    let center = IVec2::new(
        anchor.x + extent.x as i32 / 2,
        anchor.y + extent.y as i32 / 2,
    );
    let to_junction = junction_tile - center;
    if to_junction.x.abs() >= to_junction.y.abs() {
        if to_junction.x >= 0 {
            StreetSide::PosX
        } else {
            StreetSide::NegX
        }
    } else if to_junction.y >= 0 {
        StreetSide::PosZ
    } else {
        StreetSide::NegZ
    }
}

#[must_use]
pub fn orientation_from_street_side(side: StreetSide) -> u8 {
    match side {
        StreetSide::PosX => 1,
        StreetSide::NegX => 3,
        StreetSide::PosZ => 2,
        StreetSide::NegZ => 0,
    }
}

#[must_use]
pub fn build_block_frame(block: &BlockRecord, graph: Option<&TransportGraph>) -> Option<BlockFrame> {
    let (anchor, extent) = block_bounds_from_tiles(&block.tiles)?;
    let centroid = tile_centroid(&block.tiles);

    let (street_side, orientation_quarter_turns, junction_tile) =
        if let Some(g) = graph {
            if let Some((jt, _kind)) = nearest_junction_tile(g, centroid) {
                let side = street_side_from_junction(anchor, extent, jt);
                (side, orientation_from_street_side(side), Some(jt))
            } else {
                (StreetSide::NegZ, 0, None)
            }
        } else {
            (StreetSide::NegZ, 0, None)
        };

    Some(BlockFrame {
        block_id: block.id.clone(),
        anchor,
        extent,
        street_side,
        orientation_quarter_turns,
        junction_tile,
    })
}

#[must_use]
pub fn rebuild_block_frames(blocks: &BlockBook, graph: Option<&TransportGraph>) -> BlockFrameBook {
    let mut frames = HashMap::new();
    for block in blocks.blocks.values() {
        if let Some(frame) = build_block_frame(block, graph) {
            frames.insert(frame.block_id.clone(), frame);
        }
    }
    BlockFrameBook { frames }
}

pub fn sync_block_frames_system(
    blocks: Res<BlockBook>,
    graph: Option<Res<TransportGraph>>,
    mut frames: ResMut<BlockFrameBook>,
) {
    let rebuilt = rebuild_block_frames(blocks.as_ref(), graph.as_deref());
    if frames.frames != rebuilt.frames {
        *frames = rebuilt;
    }
}

/// Tiles on the block edge that face the street (debug overlay / recipe edge primitive).
#[must_use]
pub fn street_edge_tiles(frame: &BlockFrame) -> HashSet<IVec2> {
    let mut out = HashSet::new();
    let w = frame.extent.x as i32;
    let d = frame.extent.y as i32;
    match frame.street_side {
        StreetSide::PosX => {
            let x = frame.anchor.x + w - 1;
            for z in 0..d {
                out.insert(IVec2::new(x, frame.anchor.y + z));
            }
        }
        StreetSide::NegX => {
            for z in 0..d {
                out.insert(IVec2::new(frame.anchor.x, frame.anchor.y + z));
            }
        }
        StreetSide::PosZ => {
            let z = frame.anchor.y + d - 1;
            for x in 0..w {
                out.insert(IVec2::new(frame.anchor.x + x, z));
            }
        }
        StreetSide::NegZ => {
            for x in 0..w {
                out.insert(IVec2::new(frame.anchor.x + x, frame.anchor.y));
            }
        }
    }
    out
}

/// Interior scatter band (one tile inset from street edge).
#[must_use]
pub fn scatter_interior_tiles(frame: &BlockFrame, block_tiles: &HashSet<IVec2>) -> HashSet<IVec2> {
    let edge = street_edge_tiles(frame);
    block_tiles
        .iter()
        .copied()
        .filter(|t| !edge.contains(t))
        .collect()
}

#[must_use]
pub fn fixture_transport_graph_for_block_frame() -> TransportGraph {
    use crate::infrastructure::transport::graph::TransportEdge;
    use crate::infrastructure::transport::junction::{ensure_edge_endpoints, rebuild_junction_metadata};
    use crate::systems::transport::{CorridorClass, TransportEdgeId};

    let mut graph = TransportGraph::default();
    let junction = Vec3::new(20.0, 0.0, 16.0);
    let e0 = TransportEdgeId(10);
    let e1 = TransportEdgeId(11);
    let (h0, t0) = ensure_edge_endpoints(
        &mut graph,
        e0,
        Vec3::new(10.0, 0.0, 16.0),
        junction,
    );
    let (h1, t1) = ensure_edge_endpoints(
        &mut graph,
        e1,
        junction,
        Vec3::new(20.0, 0.0, 24.0),
    );
    graph.insert_edge(
        e0,
        TransportEdge {
            head: h0,
            tail: t0,
            profile_id: "default_road".into(),
            control_points: vec![[10.0, 0.0, 16.0], [20.0, 0.0, 16.0]],
            corridor: CorridorClass::Road,
            allowed_agents: vec![],
        },
    );
    graph.insert_edge(
        e1,
        TransportEdge {
            head: h1,
            tail: t1,
            profile_id: "default_road".into(),
            control_points: vec![[20.0, 0.0, 16.0], [20.0, 0.0, 24.0]],
            corridor: CorridorClass::Road,
            allowed_agents: vec![],
        },
    );
    rebuild_junction_metadata(&mut graph);
    graph
}

#[must_use]
pub fn fixture_block_record_with_tiles() -> BlockRecord {
    use super::ids::DistrictId;

    let mut tiles = HashSet::new();
    for x in 16..20 {
        for z in 14..18 {
            tiles.insert(IVec2::new(x, z));
        }
    }
    BlockRecord {
        id: BlockId("fixture_block_frame".into()),
        district_id: DistrictId("d_fixture".into()),
        tiles,
        site_ids: vec![1, 2],
        archetype: Some(BlockArchetype::LowDensityRes),
    }
}

/// Overlay wiring witness — structural check for debug tile classification (no egui).
#[must_use]
pub fn block_frame_debug_overlay_wired_witness_green() -> bool {
    let graph = fixture_transport_graph_for_block_frame();
    let block = fixture_block_record_with_tiles();
    let tiles = block.tiles.clone();
    let frame = build_block_frame(&block, Some(&graph)).expect("frame");
    let book = rebuild_block_frames(
        &BlockBook {
            blocks: HashMap::from([(block.id.clone(), block)]),
            tile_to_block: HashMap::new(),
        },
        Some(&graph),
    );
    book.frames.contains_key(&frame.block_id)
        && street_edge_tiles(&frame).len() == frame.extent.y as usize
        && scatter_interior_tiles(&frame, &tiles).len() == 12
}

#[must_use]
pub fn city_g1_c2_001_block_frame_witness_green() -> bool {
    build_city_g1_c2_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn build_city_g1_c2_001_witness_body() -> serde_json::Value {
    use super::block_archetype::load_block_archetype_registry;

    let bounds_ok = {
        let mut tiles = HashSet::new();
        tiles.insert(IVec2::new(3, 4));
        tiles.insert(IVec2::new(5, 6));
        block_bounds_from_tiles(&tiles) == Some((IVec2::new(3, 4), UVec2::new(3, 3)))
    };

    let graph = fixture_transport_graph_for_block_frame();
    let block = fixture_block_record_with_tiles();
    let frame = build_block_frame(&block, Some(&graph)).expect("frame");
    let junction_ok = frame.junction_tile == Some(IVec2::new(20, 16));
    let street_ok = frame.street_side == StreetSide::PosX;
    let orient_ok = frame.orientation_quarter_turns == 1;
    let edge_ok = street_edge_tiles(&frame).len() == 4;
    let scatter_ok = scatter_interior_tiles(&frame, &block.tiles).len() == 12;
    let overlay_wired = block_frame_debug_overlay_wired_witness_green();
    let table_still = load_block_archetype_registry().table.is_some();
    let g0_wit = crate::construction::procedural::city_g0_wit_001_determinism_witness_green();

    let green = bounds_ok
        && junction_ok
        && street_ok
        && orient_ok
        && edge_ok
        && scatter_ok
        && overlay_wired
        && table_still;

    serde_json::json!({
        "gate": "CITY-G1-C2-001",
        "issue": "CITY-C2",
        "green": green,
        "bounds_ok": bounds_ok,
        "junction_ok": junction_ok,
        "street_side": frame.street_side.as_str(),
        "orientation_quarter_turns": frame.orientation_quarter_turns,
        "edge_tile_count": street_edge_tiles(&frame).len(),
        "scatter_tile_count": scatter_interior_tiles(&frame, &block.tiles).len(),
        "overlay_wired": overlay_wired,
        "block_archetype_table_still_ok": table_still,
        "city_g0_wit_still_green": g0_wit,
    })
}

#[must_use]
pub fn refresh_city_g1_c2_001_block_frame_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_city_g1_c2_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CITY-G1-C2-001",
        "refresh_city_g1_c2_001_block_frame_witness",
        CITY_G1_C2_LIVE_JSON,
        body,
    );
    write_debug_run_json(CITY_G1_C2_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_bounds_from_tile_set() {
        let mut tiles = HashSet::new();
        tiles.insert(IVec2::new(8, 8));
        tiles.insert(IVec2::new(9, 10));
        assert_eq!(
            block_bounds_from_tiles(&tiles),
            Some((IVec2::new(8, 8), UVec2::new(2, 3)))
        );
    }

    #[test]
    fn block_frame_orients_to_nearest_junction() {
        let graph = fixture_transport_graph_for_block_frame();
        let block = fixture_block_record_with_tiles();
        let frame = build_block_frame(&block, Some(&graph)).expect("frame");
        assert_eq!(frame.junction_tile, Some(IVec2::new(20, 16)));
        assert_eq!(frame.street_side, StreetSide::PosX);
        assert_eq!(frame.orientation_quarter_turns, 1);
    }

    #[test]
    fn block_frame_without_graph_defaults_neg_z() {
        let block = fixture_block_record_with_tiles();
        let frame = build_block_frame(&block, None).expect("frame");
        assert_eq!(frame.street_side, StreetSide::NegZ);
        assert_eq!(frame.orientation_quarter_turns, 0);
        assert!(frame.junction_tile.is_none());
    }

    #[test]
    fn city_g1_c2_001_witness_green_lib() {
        assert!(city_g1_c2_001_block_frame_witness_green());
    }
}
