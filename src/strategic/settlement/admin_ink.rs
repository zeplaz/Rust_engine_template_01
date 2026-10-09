//! Administrative zone drag. The rectangle lives in map tiles.
//! Preview writes only [`AdminZoneDrag`]. Membership changes on commit,
//! on the existing [`super::district::DistrictBook`].

use bevy::prelude::*;

use crate::gui::map_tile_raster::map_tile_stamp_rect;

use super::district::{
    add_admin_zone, core_ring_for_rect, move_building_zone, place_site_in_zone, ring_bounds,
    DistrictBook,
};
use super::ids::{DistrictId, TownId};

/// Drag in progress. Not a second zone book.
#[derive(Resource, Clone, Debug, Default)]
pub struct AdminZoneDrag {
    pub zone: Option<DistrictId>,
    pub anchor: Option<IVec2>,
    pub cursor: Option<IVec2>,
    /// Tile vertices of the ring being drawn or edited. Not a second zone book.
    pub vertices: Vec<IVec2>,
    /// Vertex index while the player moves one corner of an existing ring.
    pub moving: Option<usize>,
    /// While set, this drag owns the pointer. Zone paint leaves it alone.
    pub armed: bool,
}

/// A building the commit can see. The town keeps this record; the zone only borrows membership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InkSite {
    pub id: u64,
    pub town_id: TownId,
    pub tile: IVec2,
}

/// Same floor the zone paint pointer uses: world X/Z, not screen pixels.
#[must_use]
pub fn admin_zone_tile_from_world(x: f32, z: f32) -> IVec2 {
    IVec2::new(x.floor().max(0.0) as i32, z.floor().max(0.0) as i32)
}

/// Start a drag on `zone`. Does not touch districts or pending rows.
pub fn begin_admin_zone_drag(drag: &mut AdminZoneDrag, zone: DistrictId, tile: IVec2) {
    drag.armed = true;
    drag.zone = Some(zone);
    drag.anchor = Some(tile);
    drag.cursor = Some(tile);
}

/// Move the free corner. Membership stays as it was.
pub fn move_admin_zone_drag(drag: &mut AdminZoneDrag, tile: IVec2) {
    if drag.anchor.is_some() {
        drag.cursor = Some(tile);
    }
}

/// Inclusive tile rectangle, stored the way [`IRect::contains`] reads it
/// (max is one past the far corner). `None` when the drag has not started.
#[must_use]
pub fn admin_zone_drag_rect(drag: &AdminZoneDrag) -> Option<IRect> {
    let anchor = drag.anchor?;
    let cursor = drag.cursor?;
    let min = anchor.min(cursor);
    let max = anchor.max(cursor).saturating_add(IVec2::ONE);
    if max.x <= min.x || max.y <= min.y {
        None
    } else {
        Some(IRect { min, max })
    }
}

/// Screen stamps of the two inclusive corners. Tile identity is `origin / px_per_tile`.
#[must_use]
pub fn admin_zone_drag_stamps(a: IVec2, b: IVec2, px_per_tile: f32) -> [[f32; 4]; 2] {
    let min = a.min(b);
    let max = a.max(b);
    [
        map_tile_stamp_rect(min.x, min.y, px_per_tile),
        map_tile_stamp_rect(max.x, max.y, px_per_tile),
    ]
}

/// Tile under a stamp from [`admin_zone_drag_stamps`].
#[must_use]
pub fn admin_zone_corner_tile(stamp: [f32; 4], px_per_tile: f32) -> IVec2 {
    let px = px_per_tile.abs().max(1e-6);
    IVec2::new((stamp[0] / px).round() as i32, (stamp[1] / px).round() as i32)
}

/// Start a new zone on `town` at `tile` and arm the drag. Uses [`add_admin_zone`].
#[must_use]
pub fn start_admin_zone_drag(
    districts: &mut DistrictBook,
    drag: &mut AdminZoneDrag,
    town_id: &TownId,
    tile: IVec2,
) -> DistrictId {
    let id = add_admin_zone(
        districts,
        town_id,
        "zone",
        IRect::from_corners(tile, tile + IVec2::ONE),
    );
    begin_admin_zone_drag(drag, id.clone(), tile);
    id
}

/// Append one tile. Consecutive repeats are ignored.
pub fn append_admin_ring_vertex(drag: &mut AdminZoneDrag, tile: IVec2) {
    if drag.vertices.last().copied() != Some(tile) {
        drag.vertices.push(tile);
    }
}

/// Closing click is the first vertex, once the ring has three corners.
#[must_use]
pub fn admin_ring_should_close(drag: &AdminZoneDrag, tile: IVec2) -> bool {
    drag.vertices.len() >= 3 && drag.vertices.first().copied() == Some(tile)
}

pub fn move_admin_ring_vertex(drag: &mut AdminZoneDrag, index: usize, tile: IVec2) {
    if let Some(slot) = drag.vertices.get_mut(index) {
        *slot = tile;
    }
}

/// Drop a duplicated closing vertex. The stored ring still cycles last → first.
#[must_use]
pub fn closed_ring_vertices(vertices: &[IVec2]) -> Vec<IVec2> {
    let mut ring = vertices.to_vec();
    if ring.len() >= 2 && ring.first() == ring.last() {
        ring.pop();
    }
    ring
}

/// Arm an existing zone so one vertex can move. Does not write membership.
pub fn arm_admin_zone_edit(
    drag: &mut AdminZoneDrag,
    zone: DistrictId,
    ring: Vec<IVec2>,
    index: usize,
) {
    drag.armed = true;
    drag.zone = Some(zone);
    drag.vertices = ring;
    drag.moving = Some(index);
    drag.anchor = None;
    drag.cursor = None;
}

/// Removes the district row and drops its site ids. Building entities stay in the world.
#[must_use]
pub fn delete_admin_zone(districts: &mut DistrictBook, zone: &DistrictId) -> bool {
    if districts.districts.remove(zone).is_none() {
        return false;
    }
    if districts.default_district.as_ref() == Some(zone) {
        districts.default_district = None;
    }
    true
}

/// One commit. The zone's cells become the ring of the drag rectangle.
/// Calls [`commit_admin_zone_ring`]. An empty drag returns false and writes nothing.
#[must_use]
pub fn commit_admin_zone_drag(
    districts: &mut DistrictBook,
    drag: &AdminZoneDrag,
    sites: &[InkSite],
) -> bool {
    let Some(zone_id) = drag.zone.clone() else {
        return false;
    };
    let Some(rect) = admin_zone_drag_rect(drag) else {
        return false;
    };
    let ring = core_ring_for_rect(rect);
    commit_admin_zone_ring(districts, &zone_id, &ring, sites)
}

/// Writes a closed ring onto the existing district row.
/// Sites of this town inside the ring join through [`place_site_in_zone`]
/// or [`move_building_zone`]. Sites outside leave this zone and stay on the town.
/// Another town is not scanned in. `tile_rect` becomes the ring bounds.
#[must_use]
pub fn commit_admin_zone_ring(
    districts: &mut DistrictBook,
    zone_id: &DistrictId,
    ring: &[IVec2],
    sites: &[InkSite],
) -> bool {
    let ring = closed_ring_vertices(ring);
    if ring.len() < 3 {
        return false;
    }
    let Some(bounds) = ring_bounds(&ring) else {
        return false;
    };
    let Some(town_id) = districts
        .districts
        .get(zone_id)
        .map(|district| district.town_id.clone())
    else {
        return false;
    };

    let contains = |tile: IVec2| super::district::tile_in_ring(&ring, tile);
    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for site in sites {
        if site.town_id != town_id {
            continue;
        }
        if contains(site.tile) {
            if !inside.contains(&site.id) {
                inside.push(site.id);
            }
        } else if !outside.contains(&site.id) {
            outside.push(site.id);
        }
    }

    {
        let Some(zone) = districts.districts.get_mut(zone_id) else {
            return false;
        };
        zone.ring = ring;
        zone.tile_rect = bounds;
        zone.site_ids.retain(|id| !outside.contains(id));
    }

    for id in inside {
        let current = districts
            .districts
            .values()
            .find(|district| district.site_ids.contains(&id))
            .map(|district| district.id.clone());
        let joined = match current {
            Some(current) if current == *zone_id => true,
            Some(_) => move_building_zone(districts, id, zone_id),
            None => place_site_in_zone(districts, id, zone_id),
        };
        if !joined {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::MilitaryCivilian;
    use crate::strategic::settlement::district::{add_admin_zone, place_site_in_zone};
    use crate::strategic::settlement::town::{
        required_clearance_for, town_allows_vehicle, TownAccessPolicy, TownBook, TownRecord,
    };
    use crate::strategic::{
        logistics_net_inject_into_overlays, ChunkStrategicOverlay, HaulTrafficClass, LogisticsEdge,
        LogisticsGraph, LogisticsNode, LogisticsNodeId, StrategicOverlayCouplingScratch,
    };
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use crate::terrain::ChunkCellKey;
    use std::collections::HashMap;

    fn town_id(name: &str) -> TownId {
        TownId(name.into())
    }

    fn book_with_zone(town: &str, zone_name: &str) -> (DistrictBook, DistrictId, TownId) {
        let town_id = town_id(town);
        let mut districts = DistrictBook::default();
        let zone = add_admin_zone(
            &mut districts,
            &town_id,
            zone_name,
            IRect::from_corners(IVec2::ZERO, IVec2::new(2, 2)),
        );
        (districts, zone, town_id)
    }

    fn drag_over(zone: DistrictId, a: IVec2, b: IVec2) -> AdminZoneDrag {
        let mut drag = AdminZoneDrag::default();
        begin_admin_zone_drag(&mut drag, zone, a);
        move_admin_zone_drag(&mut drag, b);
        drag
    }

    #[test]
    fn ink_rect_tiles_unchanged_after_zoom_scale() {
        let a = IVec2::new(12, 7);
        let b = IVec2::new(18, 11);
        let near = admin_zone_drag_stamps(a, b, 8.0);
        let far = admin_zone_drag_stamps(a, b, 16.0);
        assert_eq!(admin_zone_corner_tile(near[0], 8.0), a);
        assert_eq!(admin_zone_corner_tile(near[1], 8.0), b);
        assert_eq!(admin_zone_corner_tile(far[0], 16.0), a);
        assert_eq!(admin_zone_corner_tile(far[1], 16.0), b);
        assert!((far[0][0] / near[0][0] - 2.0).abs() < 1e-4);
        assert!((far[1][2] / near[1][2] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn ink_commit_includes_inside_excludes_outside() {
        let (mut districts, zone, town) = book_with_zone("ash", "yard");
        let sites = vec![
            InkSite {
                id: 1,
                town_id: town.clone(),
                tile: IVec2::new(2, 2),
            },
            InkSite {
                id: 2,
                town_id: town,
                tile: IVec2::new(9, 9),
            },
        ];
        let drag = drag_over(zone.clone(), IVec2::new(1, 1), IVec2::new(4, 4));
        assert!(commit_admin_zone_drag(&mut districts, &drag, &sites));
        let members = &districts.districts[&zone].site_ids;
        assert!(members.contains(&1));
        assert!(!members.contains(&2));
    }

    #[test]
    fn ink_redraw_drops_site_still_in_town() {
        let (mut districts, zone, town) = book_with_zone("elm", "yard");
        let mut town_sites = vec![
            InkSite {
                id: 3,
                town_id: town.clone(),
                tile: IVec2::new(2, 2),
            },
            InkSite {
                id: 4,
                town_id: town,
                tile: IVec2::new(6, 6),
            },
        ];
        let wide = drag_over(zone.clone(), IVec2::ZERO, IVec2::new(8, 8));
        assert!(commit_admin_zone_drag(&mut districts, &wide, &town_sites));
        assert!(districts.districts[&zone].site_ids.contains(&4));
        let tight = drag_over(zone.clone(), IVec2::ZERO, IVec2::new(3, 3));
        assert!(commit_admin_zone_drag(&mut districts, &tight, &town_sites));
        assert!(!districts.districts[&zone].site_ids.contains(&4));
        assert!(town_sites.iter().any(|site| site.id == 4));
        town_sites.retain(|site| site.id == 4);
        assert_eq!(town_sites.len(), 1);
    }

    #[test]
    fn ink_site_not_member_of_two_zones() {
        let (mut districts, west, town) = book_with_zone("rye", "west");
        let east = add_admin_zone(
            &mut districts,
            &town,
            "east",
            IRect::from_corners(IVec2::new(10, 0), IVec2::new(14, 4)),
        );
        assert!(place_site_in_zone(&mut districts, 7, &east));
        let sites = vec![InkSite {
            id: 7,
            town_id: town,
            tile: IVec2::new(2, 2),
        }];
        let drag = drag_over(west.clone(), IVec2::ZERO, IVec2::new(4, 4));
        assert!(commit_admin_zone_drag(&mut districts, &drag, &sites));
        let copies = districts
            .districts
            .values()
            .filter(|district| district.site_ids.contains(&7))
            .count();
        assert_eq!(copies, 1);
        assert!(districts.districts[&west].site_ids.contains(&7));
        assert!(!districts.districts[&east].site_ids.contains(&7));
    }

    #[test]
    fn ink_other_town_site_is_not_stolen() {
        let (mut districts, home, _home_town) = book_with_zone("oak", "home");
        let other = town_id("fen");
        let foreign = add_admin_zone(
            &mut districts,
            &other,
            "foreign",
            IRect::from_corners(IVec2::ZERO, IVec2::new(6, 6)),
        );
        assert!(place_site_in_zone(&mut districts, 8, &foreign));
        let sites = vec![InkSite {
            id: 8,
            town_id: other,
            tile: IVec2::new(2, 2),
        }];
        let drag = drag_over(home.clone(), IVec2::ZERO, IVec2::new(5, 5));
        assert!(commit_admin_zone_drag(&mut districts, &drag, &sites));
        assert!(!districts.districts[&home].site_ids.contains(&8));
        assert!(districts.districts[&foreign].site_ids.contains(&8));
    }

    #[test]
    fn ink_empty_rect_commits_nothing() {
        let (mut districts, zone, town) = book_with_zone("dun", "yard");
        assert!(place_site_in_zone(&mut districts, 5, &zone));
        let before = districts.districts[&zone].site_ids.clone();
        let mut drag = AdminZoneDrag::default();
        drag.zone = Some(zone.clone());
        let sites = vec![InkSite {
            id: 5,
            town_id: town,
            tile: IVec2::new(1, 1),
        }];
        assert!(!commit_admin_zone_drag(&mut districts, &drag, &sites));
        assert_eq!(districts.districts[&zone].site_ids, before);
    }

    #[test]
    fn ink_civilian_refused_by_military_closed_accepted_by_free() {
        assert!(!town_allows_vehicle(
            TownAccessPolicy::MilitaryClosed,
            MilitaryCivilian::Civilian,
            2
        ));
        assert!(town_allows_vehicle(
            TownAccessPolicy::FreeLabor,
            MilitaryCivilian::Civilian,
            0
        ));
        assert!(!town_allows_vehicle(
            TownAccessPolicy::RestrictedLabor,
            MilitaryCivilian::Civilian,
            0
        ));
        assert!(town_allows_vehicle(
            TownAccessPolicy::RestrictedLabor,
            MilitaryCivilian::Civilian,
            1
        ));
    }

    #[test]
    fn ink_military_clearance_2_enters_closed_civilian_does_not() {
        assert!(town_allows_vehicle(
            TownAccessPolicy::MilitaryClosed,
            MilitaryCivilian::Military,
            2
        ));
        assert!(!town_allows_vehicle(
            TownAccessPolicy::MilitaryClosed,
            MilitaryCivilian::Military,
            1
        ));
        assert!(!town_allows_vehicle(
            TownAccessPolicy::MilitaryClosed,
            MilitaryCivilian::Civilian,
            9
        ));
        assert!(town_allows_vehicle(
            TownAccessPolicy::FreeLabor,
            MilitaryCivilian::Military,
            0
        ));
    }

    #[test]
    fn ink_preview_drag_does_not_change_membership() {
        let (mut districts, zone, town) = book_with_zone("holt", "yard");
        assert!(place_site_in_zone(&mut districts, 11, &zone));
        let before = districts.clone();
        let mut drag = AdminZoneDrag::default();
        begin_admin_zone_drag(&mut drag, zone.clone(), IVec2::ZERO);
        move_admin_zone_drag(&mut drag, IVec2::new(6, 6));
        let _ = (town, drag);
        assert_eq!(
            districts.districts[&zone].site_ids,
            before.districts[&zone].site_ids
        );
        assert_eq!(
            districts.districts[&zone].tile_rect,
            before.districts[&zone].tile_rect
        );
    }

    fn town_book(policy: TownAccessPolicy) -> (TownBook, DistrictBook) {
        let id = town_id("gate");
        let mut towns = HashMap::new();
        towns.insert(
            id.clone(),
            TownRecord {
                id: id.clone(),
                name: "gate".into(),
                region_name: "gate".into(),
                center_tile: IVec2::new(2, 2),
                population: 10,
                jobs: 4,
                housing: 8,
                faction: 1,
                access: policy,
                required_clearance: required_clearance_for(policy),
            },
        );
        let towns = TownBook {
            towns,
            default_town: Some(id.clone()),
        };
        let mut districts = DistrictBook::default();
        let _ = add_admin_zone(
            &mut districts,
            &id,
            "gate",
            IRect::from_corners(IVec2::ZERO, IVec2::new(8, 8)),
        );
        (towns, districts)
    }

    fn haul_app(policy: TownAccessPolicy, kind: MilitaryCivilian, clearance: u8) -> App {
        let (towns, districts) = town_book(policy);
        let mut app = App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins)
            .init_resource::<StrategicOverlayCouplingScratch>()
            .insert_resource(LogisticsGraph {
                revision: 1,
                nodes: vec![
                    LogisticsNode {
                        id: LogisticsNodeId(0),
                        throughput: 0.0,
                        stockpile: 0.0,
                        anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
                    },
                    LogisticsNode {
                        id: LogisticsNodeId(1),
                        throughput: 0.0,
                        stockpile: 0.0,
                        anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
                    },
                ],
                edges: vec![LogisticsEdge {
                    from: LogisticsNodeId(0),
                    to: LogisticsNodeId(1),
                    transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
                    capacity: 10.0,
                    disruption: 0.0,
                    traversal_cost: 1.0,
                }],
            })
            .insert_resource(towns)
            .insert_resource(districts)
            .insert_resource(HaulTrafficClass {
                kind,
                clearance,
                tile: IVec2::new(1, 1),
            })
            .add_systems(Update, logistics_net_inject_into_overlays);
        app.world_mut().spawn((
            Chunk { coord: IVec2::ZERO },
            ChunkCellMatrix::new(UVec2::new(2, 1)),
            ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 1)),
        ));
        app
    }

    fn throughput_sum(app: &mut App) -> f32 {
        app.update();
        let mut query = app.world_mut().query::<&ChunkStrategicOverlay>();
        let overlay = query.iter(app.world()).next().expect("overlay");
        overlay.logistics_throughput.iter().sum()
    }

    #[test]
    fn ink_refused_traffic_adds_no_haul_flow() {
        let mut refused = haul_app(TownAccessPolicy::MilitaryClosed, MilitaryCivilian::Civilian, 2);
        assert_eq!(throughput_sum(&mut refused), 0.0);
        assert_eq!(
            refused.world().resource::<LogisticsGraph>().edges[0].disruption,
            0.0
        );
        let mut accepted = haul_app(TownAccessPolicy::FreeLabor, MilitaryCivilian::Civilian, 0);
        assert!(throughput_sum(&mut accepted) > 0.0);
        let mut military = haul_app(TownAccessPolicy::MilitaryClosed, MilitaryCivilian::Military, 2);
        assert!(throughput_sum(&mut military) > 0.0);
    }

    #[test]
    fn twn_ov_polygon_membership() {
        use crate::strategic::settlement::district::tile_in_ring;

        let (mut districts, zone, town) = book_with_zone("ash", "yard");
        let empty = districts.districts[&zone].clone();
        assert!(empty.ring.is_empty());
        assert!(empty.contains_tile(IVec2::new(1, 1)));
        assert!(!empty.contains_tile(IVec2::new(9, 9)));

        let mut short = empty.clone();
        short.ring = vec![IVec2::ZERO, IVec2::new(2, 0)];
        assert_eq!(
            short.contains_tile(IVec2::new(1, 1)),
            short.tile_rect.contains(IVec2::new(1, 1))
        );

        let square = vec![
            IVec2::new(0, 0),
            IVec2::new(5, 0),
            IVec2::new(5, 5),
            IVec2::new(0, 5),
        ];
        assert!(tile_in_ring(&square, IVec2::new(0, 0)));
        assert!(tile_in_ring(&square, IVec2::new(5, 2)));
        assert!(tile_in_ring(&square, IVec2::new(2, 2)));
        assert!(!tile_in_ring(&square, IVec2::new(6, 2)));

        let bowtie = vec![
            IVec2::new(0, 0),
            IVec2::new(4, 4),
            IVec2::new(0, 4),
            IVec2::new(4, 0),
        ];
        assert!(tile_in_ring(&bowtie, IVec2::new(1, 3)));
        assert!(!tile_in_ring(&bowtie, IVec2::new(4, 2)));

        let foreign_town = town_id("foreign");
        let foreign_zone = add_admin_zone(
            &mut districts,
            &foreign_town,
            "far",
            IRect::from_corners(IVec2::ZERO, IVec2::new(3, 3)),
        );
        assert!(place_site_in_zone(&mut districts, 7, &foreign_zone));
        let other = add_admin_zone(
            &mut districts,
            &town,
            "other",
            IRect::from_corners(IVec2::new(20, 20), IVec2::new(24, 24)),
        );
        assert!(place_site_in_zone(&mut districts, 8, &other));

        let sites = vec![
            InkSite {
                id: 1,
                town_id: town.clone(),
                tile: IVec2::new(1, 1),
            },
            InkSite {
                id: 2,
                town_id: town.clone(),
                tile: IVec2::new(9, 9),
            },
            InkSite {
                id: 7,
                town_id: foreign_town,
                tile: IVec2::new(1, 1),
            },
            InkSite {
                id: 8,
                town_id: town,
                tile: IVec2::new(2, 2),
            },
        ];
        let ring = vec![
            IVec2::new(0, 0),
            IVec2::new(4, 0),
            IVec2::new(4, 4),
            IVec2::new(0, 4),
        ];
        assert!(commit_admin_zone_ring(
            &mut districts,
            &zone,
            &ring,
            &sites
        ));
        let committed = &districts.districts[&zone];
        assert_eq!(committed.ring, ring);
        assert!(committed.site_ids.contains(&1));
        assert!(committed.site_ids.contains(&8));
        assert!(!committed.site_ids.contains(&2));
        assert!(!committed.site_ids.contains(&7));
        assert!(districts.districts[&foreign_zone].site_ids.contains(&7));
        assert!(!districts.districts[&other].site_ids.contains(&8));
        assert!(committed.contains_tile(IVec2::new(0, 0)));
        assert!(committed.contains_tile(IVec2::new(2, 2)));
        assert!(!committed.contains_tile(IVec2::new(9, 9)));
    }
}
