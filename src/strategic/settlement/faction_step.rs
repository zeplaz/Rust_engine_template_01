//! One faction growth step. Reads stance, threat, and the town gates.
//! Writes one `growth:` proposal. Does not write the logistics graph, occupation, or player rows.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::construction::{
    building_may_occupy_water, land_building_blocked_on_water, PendingConstructionQueue,
    PowerLineDamageBook,
};
use crate::infrastructure::utility::{plant_reaches_consumer, UtilityGraph};
use crate::strategic::{
    haul_crosses_town_flow, sample_overlay_site, ChunkStrategicOverlay, Faction, FrontlineState,
    HaulTrafficClass, LogisticsAiRuntime, SiteSampleLabel,
    MAX_STRATEGIC_FACTION_SLOTS,
};

use super::actors::{BuildingUsage, GrowthActorLayer, GrowthReasonCode};
use super::district::{DistrictBook, DistrictRecord};
use super::execute::approve_growth_proposal_into_pending;
use super::growth::{GrowthProposal, GrowthProposalQueue};
use super::ids::{ArchetypeId, DistrictId};
use super::need_utility::rank_town_need;
use super::town::{town_allows_entry, town_allows_vehicle, TownBook, TownRecord};

/// Chebyshev tiles from a town center. A defensive site must sit inside this range.
pub const THREAT_RANGE_TILES: i32 = 16;

/// Pair stance. Missing pairs are neutral. This is not a town access policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactionPairStance {
    Ally,
    Enemy,
    Neutral,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FactionStanceBook {
    pairs: HashMap<(u8, u8), FactionPairStance>,
}

impl FactionStanceBook {
    pub fn set_pair(&mut self, a: u8, b: u8, stance: FactionPairStance) {
        if a == b {
            return;
        }
        self.pairs.insert((a, b), stance);
        self.pairs.insert((b, a), stance);
    }

    #[must_use]
    pub fn stance(&self, a: u8, b: u8) -> FactionPairStance {
        if a == b {
            return FactionPairStance::Neutral;
        }
        self.pairs
            .get(&(a, b))
            .copied()
            .unwrap_or(FactionPairStance::Neutral)
    }
}

/// One overlay cell. The step keeps the winning cell only.
#[derive(Clone, Copy, Debug)]
pub struct GrowthCellRead {
    pub logistics_strength: f32,
    pub fire_risk: f32,
    pub threat: f32,
    pub friendly_control: f32,
    pub enemy_control: f32,
    pub chunk_is_contested: bool,
    pub power_flow: f32,
    pub artillery_danger: f32,
}

/// Inputs the step reads. It does not own a second zone book or a graph writer.
#[derive(Clone, Copy)]
pub struct FactionGrowthLook<'a> {
    pub towns: &'a TownBook,
    pub districts: &'a DistrictBook,
    pub stance: &'a FactionStanceBook,
    pub resources: f32,
    pub threat_tiles: &'a [IVec2],
    pub shot_targets: &'a [IVec2],
    pub haul: Option<HaulTrafficClass>,
    pub plant_feeds: bool,
    pub stockpile_fill_ratio: f32,
    pub industrial_output_proxy: f32,
    pub is_water: &'a dyn Fn(IVec2) -> bool,
    pub cell_at: &'a dyn Fn(IVec2, u8) -> Option<GrowthCellRead>,
}

#[must_use]
pub fn select_faction_growth(look: &FactionGrowthLook<'_>) -> Option<GrowthProposal> {
    let mut towns: Vec<&TownRecord> = look.towns.towns.values().collect();
    towns.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    for town in towns {
        if let Some(proposal) = defend_town(town, look) {
            return Some(proposal);
        }
        if let Some(proposal) = help_ally(town, look) {
            return Some(proposal);
        }
        if let Some(proposal) = meet_need(town, look) {
            return Some(proposal);
        }
    }
    None
}

fn defend_town(town: &TownRecord, look: &FactionGrowthLook<'_>) -> Option<GrowthProposal> {
    let mut tiles = Vec::new();
    for tile in look.threat_tiles.iter().chain(look.shot_targets.iter()) {
        if chebyshev(town.center_tile, *tile) <= THREAT_RANGE_TILES {
            tiles.push(*tile);
        }
    }
    tiles.sort_by(|a, b| a.x.cmp(&b.x).then(a.y.cmp(&b.y)));
    tiles.dedup();
    for tile in tiles {
        let Some(district) = district_containing(look.districts, town, tile) else {
            continue;
        };
        if (look.is_water)(tile) && !building_may_occupy_water(Some("warehouse")) {
            continue;
        }
        return Some(proposal(
            district,
            tile,
            "warehouse",
            BuildingUsage::Industrial,
            vec![GrowthReasonCode::ThreatInRange],
            look.resources,
            town,
        ));
    }
    None
}

fn help_ally(town: &TownRecord, look: &FactionGrowthLook<'_>) -> Option<GrowthProposal> {
    let haul = look.haul?;
    let helper = (0..MAX_STRATEGIC_FACTION_SLOTS as u8).find(|slot| {
        *slot != town.faction && look.stance.stance(*slot, town.faction) == FactionPairStance::Ally
    })?;
    if !town_allows_entry(town, helper, haul.clearance) {
        return None;
    }
    if !town_allows_vehicle(town.access, haul.kind, haul.clearance) {
        return None;
    }
    if haul_crosses_town_flow(1.0, town.access, haul.kind, haul.clearance) <= 0.0 {
        return None;
    }
    let (district, tile) = legal_anchor(look, town, "warehouse", town.center_tile)?;
    Some(proposal(
        district,
        tile,
        "warehouse",
        BuildingUsage::Logistics,
        Vec::new(),
        look.resources,
        town,
    ))
}

fn meet_need(town: &TownRecord, look: &FactionGrowthLook<'_>) -> Option<GrowthProposal> {
    let (mean_power, ring_threat, ring_artillery) = ring_field_means(town, look);
    let ranked = rank_town_need(
        town.housing,
        town.population,
        look.stockpile_fill_ratio,
        look.industrial_output_proxy,
        mean_power,
        look.plant_feeds,
        ring_threat,
        ring_artillery,
    )?;
    let (district, tile, label) = best_sampled_anchor(look, town, ranked.token)?;
    let _cell_name = label;
    Some(proposal(
        district,
        tile,
        ranked.token,
        usage_for_need(ranked.reason),
        vec![ranked.reason],
        look.resources,
        town,
    ))
}

fn usage_for_need(reason: GrowthReasonCode) -> BuildingUsage {
    match reason {
        GrowthReasonCode::HousingDeficit => BuildingUsage::Residential,
        GrowthReasonCode::TransportHigh => BuildingUsage::Logistics,
        GrowthReasonCode::UtilityMissing => BuildingUsage::Industrial,
        GrowthReasonCode::ThreatInRange
        | GrowthReasonCode::EmploymentDemand
        | GrowthReasonCode::MarketSaturated
        | GrowthReasonCode::PolicyBlocked => BuildingUsage::Industrial,
    }
}

fn proposal(
    district: DistrictId,
    tile: IVec2,
    token: &str,
    usage: BuildingUsage,
    reason_codes: Vec<GrowthReasonCode>,
    resources: f32,
    town: &TownRecord,
) -> GrowthProposal {
    GrowthProposal {
        district_id: district,
        block_id: None,
        archetype_id: ArchetypeId(token.to_string()),
        usage,
        actor_layer: GrowthActorLayer::Growth,
        anchor_tile: tile,
        priority: if resources > 0.0 { resources } else { 1.0 },
        seed: town.center_tile.x.max(0) as u64,
        reason_codes,
        saturation_at_submit: 0.0,
    }
}

fn district_containing(
    districts: &DistrictBook,
    town: &TownRecord,
    tile: IVec2,
) -> Option<DistrictId> {
    let mut rows: Vec<_> = districts
        .districts
        .values()
        .filter(|district| district.town_id == town.id && district.contains_tile(tile))
        .map(|district| district.id.clone())
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter().next()
}

fn legal_anchor(
    look: &FactionGrowthLook<'_>,
    town: &TownRecord,
    token: &str,
    prefer: IVec2,
) -> Option<(DistrictId, IVec2)> {
    if let Some(district) = district_containing(look.districts, town, prefer) {
        if tile_allows_token(token, prefer, look.is_water) {
            return Some((district, prefer));
        }
    }
    let mut rows: Vec<_> = look
        .districts
        .districts
        .values()
        .filter(|district| district.town_id == town.id)
        .collect();
    rows.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    for district in rows {
        let rect = district.tile_rect;
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let tile = IVec2::new(x, y);
                if district.contains_tile(tile) && tile_allows_token(token, tile, look.is_water) {
                    return Some((district.id.clone(), tile));
                }
            }
        }
    }
    None
}

fn tile_allows_token(token: &str, tile: IVec2, is_water: &dyn Fn(IVec2) -> bool) -> bool {
    if !is_water(tile) {
        return true;
    }
    building_may_occupy_water(Some(token))
        && !land_building_blocked_on_water(Some(token), &[tile], |x, y| {
            is_water(IVec2::new(x, y))
        })
}

fn districts_for<'a>(book: &'a DistrictBook, town: &TownRecord) -> Vec<&'a DistrictRecord> {
    let mut rows: Vec<_> = book
        .districts
        .values()
        .filter(|district| district.town_id == town.id)
        .collect();
    rows.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    rows
}

fn ring_field_means(town: &TownRecord, look: &FactionGrowthLook<'_>) -> (f32, f32, f32) {
    let mut power = 0.0;
    let mut threat = 0.0;
    let mut artillery = 0.0;
    let mut count = 0.0;
    for district in districts_for(look.districts, town) {
        let rect = district.tile_rect;
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let tile = IVec2::new(x, y);
                if !district.contains_tile(tile) {
                    continue;
                }
                let Some(cell) = (look.cell_at)(tile, town.faction) else {
                    continue;
                };
                power += cell.power_flow;
                threat += cell.threat;
                artillery += cell.artillery_danger;
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    (power / count, threat / count, artillery / count)
}

fn best_sampled_anchor(
    look: &FactionGrowthLook<'_>,
    town: &TownRecord,
    token: &str,
) -> Option<(DistrictId, IVec2, SiteSampleLabel)> {
    let mut best: Option<(f32, IVec2, DistrictId, SiteSampleLabel)> = None;
    for district in districts_for(look.districts, town) {
        let rect = district.tile_rect;
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let tile = IVec2::new(x, y);
                if !district.contains_tile(tile) || !tile_allows_token(token, tile, look.is_water)
                {
                    continue;
                }
                let Some(cell) = (look.cell_at)(tile, town.faction) else {
                    continue;
                };
                let sample = sample_overlay_site(
                    cell.logistics_strength,
                    cell.fire_risk,
                    cell.threat,
                    cell.friendly_control,
                    cell.enemy_control,
                    cell.chunk_is_contested,
                );
                let replace = match &best {
                    None => true,
                    Some((best_score, best_tile, _, _)) => {
                        sample.score > *best_score
                            || (sample.score == *best_score
                                && (tile.y, tile.x) < (best_tile.y, best_tile.x))
                    }
                };
                if replace {
                    best = Some((
                        sample.score,
                        tile,
                        district.id.clone(),
                        sample.label,
                    ));
                }
            }
        }
    }
    best.map(|(_, tile, district, label)| (district, tile, label))
}

fn read_growth_cell(
    overlays: &Query<&ChunkStrategicOverlay>,
    contested_chunks: &[IVec2],
    tile: IVec2,
    friendly: u8,
) -> Option<GrowthCellRead> {
    let slot = friendly as usize;
    if slot >= MAX_STRATEGIC_FACTION_SLOTS {
        return None;
    }
    let other = if slot == 0 { 1 } else { 0 };
    for overlay in overlays {
        let Some(index) = overlay_cell_index(overlay, tile) else {
            continue;
        };
        let row = |field: &[[f32; MAX_STRATEGIC_FACTION_SLOTS]]| {
            field.get(index).map(|values| values[slot]).unwrap_or(0.0)
        };
        let other_control = overlay
            .faction_control
            .get(index)
            .map(|values| values[other])
            .unwrap_or(0.0);
        return Some(GrowthCellRead {
            logistics_strength: row(&overlay.logistics_strength),
            fire_risk: overlay.fire_risk.get(index).copied().unwrap_or(0.0),
            threat: row(&overlay.threat),
            friendly_control: row(&overlay.faction_control),
            enemy_control: other_control,
            chunk_is_contested: contested_chunks.contains(&overlay.chunk_coord),
            power_flow: overlay.power_flow.get(index).copied().unwrap_or(0.0),
            artillery_danger: row(&overlay.artillery_danger),
        });
    }
    None
}

fn overlay_cell_index(overlay: &ChunkStrategicOverlay, tile: IVec2) -> Option<usize> {
    let sx = overlay.size.x as i32;
    let sy = overlay.size.y as i32;
    if sx <= 0 || sy <= 0 {
        return None;
    }
    let origin_x = overlay.chunk_coord.x.saturating_mul(sx);
    let origin_y = overlay.chunk_coord.y.saturating_mul(sy);
    let lx = tile.x.checked_sub(origin_x)?;
    let ly = tile.y.checked_sub(origin_y)?;
    if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
        return None;
    }
    let index = (ly as usize)
        .saturating_mul(sx as usize)
        .saturating_add(lx as usize);
    if index >= overlay.len_cells() {
        None
    } else {
        Some(index)
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct GrowthPathStock<'w, 's> {
    routes: Option<Res<'w, crate::economy::logistics::RouteCache>>,
    paths: Option<Res<'w, crate::economy::logistics::RoutePathStore>>,
    costs: Option<Res<'w, crate::systems::transport::TransportCostCache>>,
    nav: Option<Res<'w, crate::systems::transport::TransportNavExport>>,
    storages: Query<'w, 's, &'static crate::entities::production::core::ResourceStorage>,
}

fn chebyshev(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn traversal_cost_lookup(
    id: crate::systems::transport::TransportEdgeId,
    cache: Option<&crate::systems::transport::TransportCostCache>,
    nav: Option<&crate::systems::transport::TransportNavExport>,
) -> Option<f32> {
    if let Some(cost) = cache.and_then(|book| book.by_edge.get(&id).copied()) {
        return Some(cost);
    }
    nav.and_then(|export| export.edges.iter().find(|edge| edge.id == id).map(|edge| edge.cost))
}

fn stored_path_cost_sum(
    cache: Option<&crate::economy::logistics::RouteCache>,
    paths: Option<&crate::economy::logistics::RoutePathStore>,
    costs: Option<&crate::systems::transport::TransportCostCache>,
    nav: Option<&crate::systems::transport::TransportNavExport>,
) -> Option<f32> {
    let cache = cache?;
    let paths = paths?;
    let route = cache.routes.values().min_by_key(|row| row.handle.id)?;
    let slice = paths.edge_slice(route.path);
    crate::economy::logistics::routes::sum_path_traversal_cost(slice, |id| {
        traversal_cost_lookup(id, costs, nav)
    })
}

fn world_stock_sum(
    storages: &Query<&crate::entities::production::core::ResourceStorage>,
) -> Option<f32> {
    if storages.is_empty() {
        return None;
    }
    let mut sum = 0.0_f32;
    for storage in storages {
        for amount in storage.amounts.values() {
            sum += *amount;
        }
    }
    Some(sum)
}

/// After pressure and saturation, before the growth commit funnel.
pub fn faction_growth_step_system(
    towns: Option<Res<TownBook>>,
    districts: Option<Res<DistrictBook>>,
    stance: Option<Res<FactionStanceBook>>,
    factions: Query<&Faction>,
    overlays: Query<&ChunkStrategicOverlay>,
    shots: Option<Res<super::super::shot_pulse::ShotPulseBook>>,
    haul: Option<Res<HaulTrafficClass>>,
    logistics_ai: Option<Res<LogisticsAiRuntime>>,
    front: Option<Res<FrontlineState>>,
    utility: Option<Res<UtilityGraph>>,
    damage: Option<Res<PowerLineDamageBook>>,
    water: Option<Res<crate::render::WaterSurfaceVisualCatalog>>,
    queue: Option<ResMut<GrowthProposalQueue>>,
    pending: Option<ResMut<PendingConstructionQueue>>,
    gate: GrowthPathStock,
) {
    let (Some(towns), Some(districts), Some(mut queue), Some(mut pending)) =
        (towns, districts, queue, pending)
    else {
        return;
    };
    let resources = factions
        .iter()
        .map(|faction| faction.resources)
        .fold(0.0_f32, f32::max);
    let mut threat_tiles = Vec::new();
    for overlay in &overlays {
        let sx = overlay.size.x as usize;
        let sy = overlay.size.y as usize;
        if sx == 0 || sy == 0 {
            continue;
        }
        let limit = sx.saturating_mul(sy);
        for (cell, row) in overlay.threat.iter().enumerate().take(limit) {
            if !row.iter().any(|value| *value > 0.0) {
                continue;
            }
            let lx = (cell % sx) as i32;
            let ly = (cell / sx) as i32;
            let origin_x = overlay.chunk_coord.x.saturating_mul(overlay.size.x as i32);
            let origin_y = overlay.chunk_coord.y.saturating_mul(overlay.size.y as i32);
            threat_tiles.push(IVec2::new(origin_x + lx, origin_y + ly));
        }
    }
    threat_tiles.sort_by(|a, b| a.x.cmp(&b.x).then(a.y.cmp(&b.y)));
    threat_tiles.dedup();
    let shot_targets: Vec<IVec2> = shots
        .as_ref()
        .map(|book| book.recent.iter().map(|shot| shot.target).collect())
        .unwrap_or_default();
    let plant_feeds = match utility.as_deref() {
        None => true,
        Some(graph) => {
            let blocked = damage
                .as_deref()
                .map(|book| book.destroyed_link_ids())
                .unwrap_or_default();
            plant_reaches_consumer(graph, &blocked)
        }
    };
    let dry = |_tile: IVec2| false;
    let wet = |tile: IVec2| -> bool {
        let Some(catalog) = water.as_deref() else {
            return false;
        };
        if tile.x < 0 || tile.y < 0 {
            return true;
        }
        catalog.tile_kind(tile.x as u32, tile.y as u32) != crate::render::WaterSurfaceKind::None
    };
    let is_water: &dyn Fn(IVec2) -> bool = if water.is_some() { &wet } else { &dry };
    let stance_book = stance.as_deref().cloned().unwrap_or_default();
    let contested_chunks = front
        .as_ref()
        .map(|state| state.contested_chunks.clone())
        .unwrap_or_default();
    let (stockpile_fill_ratio, industrial_output_proxy) = logistics_ai
        .as_deref()
        .map(|ai| (ai.stockpile_fill_ratio, ai.industrial_output_proxy))
        .unwrap_or((1.0, 0.0));
    let cell_at = |tile: IVec2, friendly: u8| {
        read_growth_cell(&overlays, &contested_chunks, tile, friendly)
    };
    let look = FactionGrowthLook {
        towns: &towns,
        districts: &districts,
        stance: &stance_book,
        resources,
        threat_tiles: &threat_tiles,
        shot_targets: &shot_targets,
        haul: haul.as_deref().copied(),
        plant_feeds,
        stockpile_fill_ratio,
        industrial_output_proxy,
        is_water,
        cell_at: &cell_at,
    };
    let Some(proposal) = select_faction_growth(&look) else {
        return;
    };
    let path_sum = stored_path_cost_sum(
        gate.routes.as_deref(),
        gate.paths.as_deref(),
        gate.costs.as_deref(),
        gate.nav.as_deref(),
    );
    let stock = world_stock_sum(&gate.storages);
    if path_sum.is_none() || !stock.is_some_and(|sum| sum.is_finite() && sum > 0.0) {
        return;
    }
    queue.enqueue(proposal);
    let index = queue.proposals.len() - 1;
    let _ = approve_growth_proposal_into_pending(&mut queue, index, &mut pending);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::MilitaryCivilian;
    use crate::construction::{
        PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
    };
    use crate::strategic::settlement::district::{add_admin_zone, stamp_core_rings};
    use crate::strategic::settlement::execute::pending_blueprint_from_growth_proposal;
    use crate::strategic::settlement::town::{TownAccessPolicy, TownRecord};
    use crate::strategic::settlement::TownId;
    use crate::strategic::{
        BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype,
    };

    fn insert_finite_haul_and_stock(app: &mut App) {
        use crate::economy::logistics::types::CachedRoute;
        use crate::economy::logistics::{RouteCache, RouteHandle, RoutePathStore};
        use crate::entities::production::core::{ResourceStorage, ResourceType};
        use crate::systems::transport::{TransportCostCache, TransportEdgeId};

        let from = app.world_mut().spawn_empty().id();
        let to = app.world_mut().spawn_empty().id();
        let mut paths = RoutePathStore::default();
        let path = paths.insert_path(&[TransportEdgeId(1), TransportEdgeId(2)]);
        let mut routes = RouteCache::default();
        routes.routes.insert(
            (from, to),
            CachedRoute {
                handle: RouteHandle {
                    id: 1,
                    topology_revision: 1,
                },
                path,
                reachable: true,
                bottleneck_capacity: 1.0,
            },
        );
        let mut costs = TransportCostCache::default();
        costs.by_edge.insert(TransportEdgeId(1), 2.0);
        costs.by_edge.insert(TransportEdgeId(2), 3.0);
        app.insert_resource(paths);
        app.insert_resource(routes);
        app.insert_resource(costs);
        let mut amounts = std::collections::HashMap::new();
        amounts.insert(ResourceType::Wood, 4.0);
        app.world_mut().spawn(ResourceStorage { amounts });
    }

    fn insert_missing_id_haul(app: &mut App) {
        use crate::economy::logistics::types::CachedRoute;
        use crate::economy::logistics::{RouteCache, RouteHandle, RoutePathStore};
        use crate::entities::production::core::{ResourceStorage, ResourceType};
        use crate::systems::transport::{TransportCostCache, TransportEdgeId};

        let from = app.world_mut().spawn_empty().id();
        let to = app.world_mut().spawn_empty().id();
        let mut paths = RoutePathStore::default();
        let path = paths.insert_path(&[TransportEdgeId(1), TransportEdgeId(9)]);
        let mut routes = RouteCache::default();
        routes.routes.insert(
            (from, to),
            CachedRoute {
                handle: RouteHandle {
                    id: 1,
                    topology_revision: 1,
                },
                path,
                reachable: true,
                bottleneck_capacity: 1.0,
            },
        );
        let mut costs = TransportCostCache::default();
        costs.by_edge.insert(TransportEdgeId(1), 2.0);
        app.insert_resource(paths);
        app.insert_resource(routes);
        app.insert_resource(costs);
        let mut amounts = std::collections::HashMap::new();
        amounts.insert(ResourceType::Wood, 4.0);
        app.world_mut().spawn(ResourceStorage { amounts });
    }

    fn satisfied_town(access: TownAccessPolicy) -> (TownBook, DistrictBook) {
        let id = TownId("ash".into());
        let mut towns = TownBook::default();
        towns.towns.insert(
            id.clone(),
            TownRecord {
                id: id.clone(),
                name: "Ashford".into(),
                region_name: "Ashford".into(),
                center_tile: IVec2::new(10, 10),
                population: 20,
                jobs: 20,
                housing: 20,
                faction: 1,
                access,
                required_clearance: crate::strategic::settlement::town::required_clearance_for(
                    access,
                ),
            },
        );
        towns.default_town = Some(id.clone());
        let mut districts = DistrictBook::default();
        let _zone = add_admin_zone(
            &mut districts,
            &id,
            "core",
            IRect::from_corners(IVec2::new(2, 2), IVec2::new(18, 18)),
        );
        stamp_core_rings(&mut districts);
        (towns, districts)
    }

    fn resting_cell(_tile: IVec2, _friendly: u8) -> Option<GrowthCellRead> {
        Some(GrowthCellRead {
            logistics_strength: 0.0,
            fire_risk: 0.0,
            threat: 0.0,
            friendly_control: 1.0,
            enemy_control: 0.0,
            chunk_is_contested: false,
            power_flow: 1.0,
            artillery_danger: 0.0,
        })
    }

    fn look<'a>(
        towns: &'a TownBook,
        districts: &'a DistrictBook,
        stance: &'a FactionStanceBook,
        threats: &'a [IVec2],
        haul: Option<HaulTrafficClass>,
        is_water: &'a dyn Fn(IVec2) -> bool,
    ) -> FactionGrowthLook<'a> {
        FactionGrowthLook {
            towns,
            districts,
            stance,
            resources: 12.0,
            threat_tiles: threats,
            shot_targets: &[],
            haul,
            plant_feeds: true,
            stockpile_fill_ratio: 1.0,
            industrial_output_proxy: 0.0,
            is_water,
            cell_at: &resting_cell,
        }
    }

    #[test]
    fn twn_ai_one_growth_site() {
        let dry = |_tile: IVec2| false;
        let (towns, districts) = satisfied_town(TownAccessPolicy::FreeLabor);
        let stance = FactionStanceBook::default();
        let near = [IVec2::new(12, 12)];
        let chosen = select_faction_growth(&look(
            &towns,
            &districts,
            &stance,
            &near,
            None,
            &dry,
        ))
        .expect("defend");
        assert_eq!(chosen.anchor_tile, IVec2::new(12, 12));
        assert_eq!(chosen.archetype_id.0, "warehouse");
        assert!(chosen
            .reason_codes
            .contains(&GrowthReasonCode::ThreatInRange));
        let row = pending_blueprint_from_growth_proposal(&chosen);
        assert!(row.label.starts_with("growth:"));

        let far = [IVec2::new(10 + THREAT_RANGE_TILES + 1, 10)];
        assert!(select_faction_growth(&look(&towns, &districts, &stance, &far, None, &dry)).is_none());

        let wet = |tile: IVec2| tile == IVec2::new(12, 12);
        assert!(select_faction_growth(&look(&towns, &districts, &stance, &near, None, &wet)).is_none());

        let (closed_towns, closed_districts) = satisfied_town(TownAccessPolicy::MilitaryClosed);
        let mut ally = FactionStanceBook::default();
        ally.set_pair(2, 1, FactionPairStance::Ally);
        let civilian = HaulTrafficClass {
            kind: MilitaryCivilian::Civilian,
            clearance: 2,
            tile: IVec2::new(10, 10),
        };
        assert!(select_faction_growth(&look(
            &closed_towns,
            &closed_districts,
            &ally,
            &[],
            Some(civilian),
            &dry,
        ))
        .is_none());

        let mut enemy = FactionStanceBook::default();
        enemy.set_pair(2, 1, FactionPairStance::Enemy);
        let free_haul = HaulTrafficClass {
            kind: MilitaryCivilian::Civilian,
            clearance: 0,
            tile: IVec2::new(10, 10),
        };
        assert!(select_faction_growth(&look(
            &towns,
            &districts,
            &enemy,
            &[],
            Some(free_haul),
            &dry,
        ))
        .is_none());

        ally.set_pair(2, 1, FactionPairStance::Ally);
        let helped = select_faction_growth(&look(
            &towns,
            &districts,
            &ally,
            &[],
            Some(free_haul),
            &dry,
        ))
        .expect("ally help");
        assert_eq!(helped.usage, BuildingUsage::Logistics);
        assert_eq!(helped.archetype_id.0, "warehouse");
        assert!(!helped
            .reason_codes
            .contains(&GrowthReasonCode::ThreatInRange));

        #[derive(Resource, Default)]
        struct CommitTally(usize);

        fn tally(
            mut reader: MessageReader<CommitConstructionSiteEvent>,
            mut tally: ResMut<CommitTally>,
        ) {
            tally.0 += reader.read().count();
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GrowthProposalQueue>()
            .init_resource::<FactionStanceBook>()
            .insert_resource(towns)
            .insert_resource(districts)
            .insert_resource(PendingConstructionQueue {
                entries: vec![PendingBuildBlueprint {
                    kind: PendingEntryKind::BuildSite,
                    label: "player:house".into(),
                    archetype: SiteArchetype::CivilHousing,
                    origin: BuildSiteTile { x: 1, z: 1 },
                    footprint: FootprintTiles {
                        width: 1,
                        depth: 1,
                    },
                    layer: LayerType::Surface,
                    rotation_quarter_turns: 0,
                    mirror_x: false,
                    approved: true,
                    catalog_id: Some("house".into()),
                }],
            })
            .init_resource::<CommitTally>()
            .add_message::<CommitConstructionSiteEvent>();
        app.world_mut().spawn(Faction {
            resources: 12.0,
            ..Default::default()
        });
        let mut overlay = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(32, 32));
        let cell = 12 * 32 + 12;
        overlay.threat[cell][0] = 1.0;
        app.world_mut().spawn(overlay);
        app.add_systems(
            Update,
            (faction_growth_step_system, crate::strategic::settlement::commit_growth_pending_to_funnel_system, tally).chain(),
        );
        insert_finite_haul_and_stock(&mut app);
        app.update();
        assert_eq!(app.world().resource::<CommitTally>().0, 1);
        let pending = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player:house");
    }

    #[derive(Resource, Default)]
    struct PerceptionCommitTally(usize);

    fn tally_perception_commits(
        mut reader: MessageReader<CommitConstructionSiteEvent>,
        mut tally: ResMut<PerceptionCommitTally>,
    ) {
        tally.0 += reader.read().count();
    }

    fn run_perception_update(dormant: bool) -> App {
        perception_commit_app(dormant, 1)
    }

    fn perception_commit_app(dormant: bool, haul: u8) -> App {
        use crate::infrastructure::utility::{UtilityGraph, UtilityGraphEdge, UtilityGraphNode};
        use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNode, LogisticsNodeId};
        use crate::systems::chunk_sim_lod::ChunkSimLod;

        let (towns, districts) = satisfied_town(TownAccessPolicy::FreeLabor);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GrowthProposalQueue>()
            .init_resource::<FactionStanceBook>()
            .insert_resource(towns)
            .insert_resource(districts)
            .insert_resource(PendingConstructionQueue {
                entries: vec![PendingBuildBlueprint {
                    kind: PendingEntryKind::BuildSite,
                    label: "player:house".into(),
                    archetype: SiteArchetype::CivilHousing,
                    origin: BuildSiteTile { x: 1, z: 1 },
                    footprint: FootprintTiles {
                        width: 1,
                        depth: 1,
                    },
                    layer: LayerType::Surface,
                    rotation_quarter_turns: 0,
                    mirror_x: false,
                    approved: true,
                    catalog_id: Some("house".into()),
                }],
            })
            .insert_resource(LogisticsGraph {
                revision: 7,
                nodes: vec![LogisticsNode {
                    id: LogisticsNodeId(0),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: None,
                }],
                edges: vec![LogisticsEdge {
                    from: LogisticsNodeId(0),
                    to: LogisticsNodeId(0),
                    transport_edge: None,
                    capacity: 4.0,
                    disruption: 1.0,
                    traversal_cost: 2.0,
                }],
            })
            .insert_resource(UtilityGraph {
                nodes: vec![UtilityGraphNode {
                    id: 1,
                    key: "node".into(),
                    position: Vec2::ZERO,
                }],
                power_edges: vec![UtilityGraphEdge {
                    from: 1,
                    to: 1,
                    link_id: 1,
                    throughput: 3.0,
                }],
                hydrated_from_snapshot: false,
            })
            .init_resource::<PerceptionCommitTally>()
            .add_message::<CommitConstructionSiteEvent>();
        app.world_mut().spawn(Faction {
            resources: 12.0,
            ..Default::default()
        });
        let mut overlay = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(32, 32));
        let cell = 12 * 32 + 12;
        overlay.threat[cell][0] = 1.0;
        let overlay_id = app.world_mut().spawn(overlay).id();
        if dormant {
            app.world_mut()
                .entity_mut(overlay_id)
                .insert(ChunkSimLod::Dormant);
        }
        app.add_systems(
            Update,
            (
                faction_growth_step_system,
                crate::strategic::settlement::commit_growth_pending_to_funnel_system,
                tally_perception_commits,
            )
                .chain(),
        );
        if haul == 2 {
            insert_missing_id_haul(&mut app);
        } else if haul == 1 {
            insert_finite_haul_and_stock(&mut app);
        }
        app.update();
        app
    }

    fn assert_graphs_untouched(app: &App) {
        use crate::infrastructure::utility::UtilityGraph;
        use crate::strategic::LogisticsGraph;

        let logistics = app.world().resource::<LogisticsGraph>();
        assert_eq!(logistics.revision, 7);
        assert_eq!(logistics.nodes.len(), 1);
        assert_eq!(logistics.edges.len(), 1);
        assert_eq!(logistics.edges[0].capacity, 4.0);
        assert_eq!(logistics.edges[0].disruption, 1.0);
        assert_eq!(logistics.edges[0].traversal_cost, 2.0);
        let utility = app.world().resource::<UtilityGraph>();
        assert_eq!(utility.nodes.len(), 1);
        assert_eq!(utility.power_edges.len(), 1);
        assert_eq!(utility.power_edges[0].throughput, 3.0);
    }

    #[test]
    fn ecs_perception_does_not_write_logistics() {
        let app = run_perception_update(false);
        assert_graphs_untouched(&app);
        assert_eq!(app.world().resource::<PerceptionCommitTally>().0, 1);
        let pending = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player:house");
    }

    #[test]
    fn ecs_faction_step_ignores_camera_lod() {
        let app = run_perception_update(true);
        assert_eq!(app.world().resource::<PerceptionCommitTally>().0, 1);
        let pending = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player:house");
    }

    fn scored_cell(tile: IVec2, _friendly: u8) -> Option<GrowthCellRead> {
        let (logistics_strength, fire_risk) = if tile == IVec2::new(4, 8) {
            (0.9, 1.0)
        } else if tile == IVec2::new(7, 6) {
            (0.5, 0.0)
        } else {
            (0.0, 0.0)
        };
        Some(GrowthCellRead {
            logistics_strength,
            fire_risk,
            threat: 0.0,
            friendly_control: 0.7,
            enemy_control: 0.2,
            chunk_is_contested: false,
            power_flow: 1.0,
            artillery_danger: 0.0,
        })
    }

    #[test]
    fn mai_one_site_uses_sample() {
        let dry = |_tile: IVec2| false;
        let (mut towns, districts) = satisfied_town(TownAccessPolicy::FreeLabor);
        let town = towns.towns.get_mut(&TownId("ash".into())).expect("town");
        town.housing = 2;
        town.population = 9;
        let stance = FactionStanceBook::default();
        let growth_look = FactionGrowthLook {
            towns: &towns,
            districts: &districts,
            stance: &stance,
            resources: 12.0,
            threat_tiles: &[],
            shot_targets: &[],
            haul: None,
            plant_feeds: true,
            stockpile_fill_ratio: 1.0,
            industrial_output_proxy: 0.9,
            is_water: &dry,
            cell_at: &scored_cell,
        };
        let chosen = select_faction_growth(&growth_look).expect("need");
        assert_eq!(chosen.anchor_tile, IVec2::new(7, 6));
        assert_eq!(chosen.archetype_id.0, "house");
        assert_eq!(chosen.actor_layer, GrowthActorLayer::Growth);
        assert!(chosen
            .reason_codes
            .contains(&GrowthReasonCode::HousingDeficit));
        let row = pending_blueprint_from_growth_proposal(&chosen);
        assert!(row.label.starts_with("growth:"));
        let winner = sample_overlay_site(0.5, 0.0, 0.0, 0.7, 0.2, false);
        let flooded = sample_overlay_site(0.9, 1.0, 0.0, 0.7, 0.2, false);
        assert!(winner.score > flooded.score);
        assert_eq!(winner.label, SiteSampleLabel::Internal);

        let wet_best = |tile: IVec2| tile == IVec2::new(7, 6);
        let shifted = FactionGrowthLook {
            is_water: &wet_best,
            ..growth_look
        };
        let placed = select_faction_growth(&shifted).expect("dry neighbor");
        assert_eq!(placed.anchor_tile, IVec2::new(4, 8));
        assert_ne!(placed.anchor_tile, IVec2::new(7, 6));
        assert_eq!(placed.archetype_id.0, "house");

        let all_wet = |_tile: IVec2| true;
        let blocked = FactionGrowthLook {
            is_water: &all_wet,
            ..growth_look
        };
        assert!(select_faction_growth(&blocked).is_none());

        let (mut closed, closed_districts) = satisfied_town(TownAccessPolicy::RestrictedLabor);
        let record = closed.towns.get_mut(&TownId("ash".into())).expect("town");
        record.required_clearance = 0;
        let mut ally = FactionStanceBook::default();
        ally.set_pair(2, 1, FactionPairStance::Ally);
        let haul = HaulTrafficClass {
            kind: MilitaryCivilian::Civilian,
            clearance: 0,
            tile: IVec2::new(10, 10),
        };
        let town_ref = closed.towns.get(&TownId("ash".into())).expect("town");
        assert!(town_allows_entry(town_ref, 2, 0));
        assert!(!town_allows_vehicle(
            TownAccessPolicy::RestrictedLabor,
            MilitaryCivilian::Civilian,
            0
        ));
        assert!(select_faction_growth(&look(
            &closed,
            &closed_districts,
            &ally,
            &[],
            Some(haul),
            &dry,
        ))
        .is_none());

        #[derive(Resource, Default)]
        struct CommitSeen {
            count: usize,
            origin: Option<BuildSiteTile>,
            catalog: Option<String>,
        }

        fn tally(
            mut reader: MessageReader<CommitConstructionSiteEvent>,
            mut seen: ResMut<CommitSeen>,
        ) {
            for event in reader.read() {
                seen.count += 1;
                seen.origin = Some(event.origin);
                seen.catalog = event.catalog_id.clone();
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GrowthProposalQueue>()
            .init_resource::<FactionStanceBook>()
            .insert_resource(towns)
            .insert_resource(districts)
            .insert_resource(LogisticsAiRuntime {
                stockpile_fill_ratio: 1.0,
                industrial_output_proxy: 0.9,
                ..Default::default()
            })
            .insert_resource(PendingConstructionQueue {
                entries: vec![PendingBuildBlueprint {
                    kind: PendingEntryKind::BuildSite,
                    label: "player:house".into(),
                    archetype: SiteArchetype::CivilHousing,
                    origin: BuildSiteTile { x: 1, z: 1 },
                    footprint: FootprintTiles {
                        width: 1,
                        depth: 1,
                    },
                    layer: LayerType::Surface,
                    rotation_quarter_turns: 0,
                    mirror_x: false,
                    approved: true,
                    catalog_id: Some("house".into()),
                }],
            })
            .init_resource::<CommitSeen>()
            .add_message::<CommitConstructionSiteEvent>();
        app.world_mut().spawn(Faction {
            resources: 12.0,
            ..Default::default()
        });
        let mut overlay = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(32, 32));
        overlay.power_flow.fill(1.0);
        let flooded_cell = 8 * 32 + 4;
        let winner_cell = 6 * 32 + 7;
        overlay.logistics_strength[flooded_cell][1] = 0.9;
        overlay.fire_risk[flooded_cell] = 1.0;
        overlay.logistics_strength[winner_cell][1] = 0.5;
        overlay.faction_control[winner_cell][1] = 0.7;
        overlay.faction_control[winner_cell][0] = 0.2;
        app.world_mut().spawn(overlay);
        app.add_systems(
            Update,
            (
                faction_growth_step_system,
                crate::strategic::settlement::commit_growth_pending_to_funnel_system,
                tally,
            )
                .chain(),
        );
        insert_finite_haul_and_stock(&mut app);
        app.update();
        let seen = app.world().resource::<CommitSeen>();
        assert_eq!(seen.count, 1);
        assert_eq!(seen.origin, Some(BuildSiteTile { x: 7, z: 6 }));
        assert_eq!(seen.catalog.as_deref(), Some("house"));
        let pending = app.world().resource::<PendingConstructionQueue>();
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player:house");
    }

    #[test]
    fn path_cost_sum_gates_the_enqueue() {
        use crate::economy::logistics::routes::sum_path_traversal_cost;
        use crate::systems::transport::{TransportCostCache, TransportEdgeId};

        let mut costs = TransportCostCache::default();
        costs.by_edge.insert(TransportEdgeId(1), 2.0);
        costs.by_edge.insert(TransportEdgeId(2), 3.0);
        let summed = sum_path_traversal_cost(&[TransportEdgeId(1), TransportEdgeId(2)], |id| {
            costs.by_edge.get(&id).copied()
        });
        assert_eq!(summed, Some(5.0));
        let missing = sum_path_traversal_cost(&[TransportEdgeId(1), TransportEdgeId(9)], |id| {
            costs.by_edge.get(&id).copied()
        });
        assert_eq!(missing, None);

        let bare = perception_commit_app(false, 0);
        assert_eq!(bare.world().resource::<PerceptionCommitTally>().0, 0);
        let pending = bare.world().resource::<PendingConstructionQueue>();
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player:house");

        let broken = perception_commit_app(false, 2);
        assert_eq!(broken.world().resource::<PerceptionCommitTally>().0, 0);
        let pending = broken.world().resource::<PendingConstructionQueue>();
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player:house");
    }
}
