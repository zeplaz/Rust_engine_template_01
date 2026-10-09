//! Town book authority (SET-P5-001 fixture stub until A lands full loader).

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ids::{DistrictId, TownId};

/// How labor and growth may enter a town. One policy, stored on the town.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TownAccessPolicy {
    /// Clearance must meet [`TownRecord::required_clearance`] (2 on generated towns).
    /// A foreign faction does not skip that check, and is refused even with clearance.
    MilitaryClosed,
    /// Labor moves in with no clearance check.
    #[default]
    FreeLabor,
    /// Clearance must meet [`TownRecord::required_clearance`] (1 on generated towns).
    /// Not a military closure.
    RestrictedLabor,
}

/// Generated order: first town military, second restricted, every later town free.
#[must_use]
pub fn access_policy_for_generated_index(index: u32) -> TownAccessPolicy {
    match index {
        0 => TownAccessPolicy::MilitaryClosed,
        1 => TownAccessPolicy::RestrictedLabor,
        _ => TownAccessPolicy::FreeLabor,
    }
}

#[must_use]
pub fn required_clearance_for(policy: TownAccessPolicy) -> u8 {
    match policy {
        TownAccessPolicy::MilitaryClosed => 2,
        TownAccessPolicy::RestrictedLabor => 1,
        TownAccessPolicy::FreeLabor => 0,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TownRecord {
    pub id: TownId,
    pub name: String,
    /// Plain region label. Same world seed and town index always yield the same string.
    #[serde(default)]
    pub region_name: String,
    pub center_tile: IVec2,
    pub population: u32,
    pub jobs: u32,
    pub housing: u32,
    /// Owner faction. Military entry compares visitors against this id.
    #[serde(default)]
    pub faction: u8,
    #[serde(default)]
    pub access: TownAccessPolicy,
    /// Clearance a visitor must hold. Free labor ignores it.
    #[serde(default)]
    pub required_clearance: u8,
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct TownBook {
    pub towns: HashMap<TownId, TownRecord>,
    pub default_town: Option<TownId>,
}

pub fn portland_fixture_town() -> TownBook {
    let id = TownId("portland".into());
    let mut towns = HashMap::new();
    towns.insert(
        id.clone(),
        TownRecord {
            id: id.clone(),
            name: "Portland".into(),
            region_name: "Portland".into(),
            center_tile: IVec2::new(64, 64),
            population: 12_000,
            jobs: 4_500,
            housing: 10_000,
            faction: 1,
            access: TownAccessPolicy::FreeLabor,
            required_clearance: 0,
        },
    );
    TownBook {
        towns,
        default_town: Some(id),
    }
}

/// Seeded towns. Same world seed and count always yield the same centers.
/// Every cell is treated as dry. Use [`generated_town_book_on_land`] when water is known.
pub fn generated_town_book(world_seed: u64, count: u32) -> TownBook {
    generated_town_book_on_land(world_seed, count, &|_| false)
}

/// Same book as [`generated_town_book`], then any center that `is_water` reports
/// moves to the nearest dry cell in the seed band, or that town is skipped.
pub fn generated_town_book_on_land(
    world_seed: u64,
    count: u32,
    is_water: &dyn Fn(IVec2) -> bool,
) -> TownBook {
    let n = generated_town_count(count);
    let mut towns = HashMap::new();
    let mut default_town = None;
    for i in 0..n {
        let id = TownId(format!("gen-{world_seed}-{i}"));
        let seed = super::seed_chain::town_seed(world_seed, &id);
        let center_tile = IVec2::new(
            16 + (seed & 0x7F) as i32,
            16 + ((seed >> 8) & 0x7F) as i32,
        );
        if default_town.is_none() {
            default_town = Some(id.clone());
        }
        let pop = 400 + (seed & 0x3FF) as u32;
        let access = access_policy_for_generated_index(i);
        let region = region_name_for_seed(world_seed, i);
        towns.insert(
            id.clone(),
            TownRecord {
                id,
                name: region.clone(),
                region_name: region,
                center_tile,
                population: pop,
                jobs: town_service_count(pop, 120 + ((seed >> 10) & 0xFF) as u32),
                housing: town_service_count(pop, 300 + ((seed >> 18) & 0x1FF) as u32),
                faction: 1,
                access,
                required_clearance: required_clearance_for(access),
            },
        );
    }
    let mut book = TownBook { towns, default_town };
    spread_generated_centers(&mut book);
    settle_centers_onto_dry(&mut book, is_water);
    book
}

const REGION_LEFT: [&str; 16] = [
    "Ash", "Elm", "Rye", "Oak", "Fen", "Dun", "Holt", "Mere", "Wold", "Beck", "Shaw", "Tarn",
    "Glen", "Moss", "Keld", "Bourne",
];
const REGION_RIGHT: [&str; 8] = [
    "ford", "field", "wick", "stead", "haven", "bridge", "market", "yard",
];

/// Region label from the world seed and town index. Plain string, no catalog.
#[must_use]
pub fn region_name_for_seed(world_seed: u64, index: u32) -> String {
    let h = super::seed_chain::mix_u64(world_seed, "region", &index.to_string());
    let left = REGION_LEFT[(h as usize) % REGION_LEFT.len()];
    let right = REGION_RIGHT[((h >> 8) as usize) % REGION_RIGHT.len()];
    format!("{left}{right}")
}

/// Logistics entry and growth commits share this gate.
#[must_use]
pub fn town_allows_entry(town: &TownRecord, visitor_faction: u8, clearance: u8) -> bool {
    match town.access {
        TownAccessPolicy::FreeLabor => true,
        TownAccessPolicy::RestrictedLabor => clearance >= town.required_clearance,
        TownAccessPolicy::MilitaryClosed => {
            visitor_faction == town.faction && clearance >= town.required_clearance
        }
    }
}

#[must_use]
pub fn town_allows_logistics_entry(town: &TownRecord, visitor_faction: u8, clearance: u8) -> bool {
    town_allows_entry(town, visitor_faction, clearance)
}

#[must_use]
pub fn town_allows_growth_commit(town: &TownRecord, visitor_faction: u8, clearance: u8) -> bool {
    town_allows_entry(town, visitor_faction, clearance)
}

/// Whether a haul of this kind may cross the town. Kind is the existing
/// [`crate::entities::MilitaryCivilian`] flag, not a new vehicle.
#[must_use]
pub fn town_allows_vehicle(
    policy: TownAccessPolicy,
    kind: crate::entities::MilitaryCivilian,
    clearance: u8,
) -> bool {
    use crate::entities::MilitaryCivilian;
    match policy {
        TownAccessPolicy::FreeLabor => true,
        TownAccessPolicy::RestrictedLabor => clearance >= 1,
        TownAccessPolicy::MilitaryClosed => {
            kind == MilitaryCivilian::Military && clearance >= 2
        }
    }
}

/// Generated books stay between one and eight towns.
#[must_use]
pub fn generated_town_count(count: u32) -> u32 {
    count.clamp(1, 8)
}

/// Planned jobs or housing cannot outrun the people already in the town.
#[must_use]
pub fn town_service_count(population: u32, planned: u32) -> u32 {
    planned.min(population)
}

fn seed_band_axis(v: i32) -> i32 {
    16 + (v - 16).rem_euclid(128)
}

/// Pulls an out-of-band center into 16..=143, then steps until it is four tiles from every placed town.
#[must_use]
pub fn separate_town_center(center: IVec2, blocked: &[IVec2]) -> IVec2 {
    let mut c = IVec2::new(seed_band_axis(center.x), seed_band_axis(center.y));
    let mut guard = 0;
    while blocked.iter().any(|p| towns_need_separation(*p, c)) && guard < 128 {
        c.x = 16 + ((c.x - 16 + 17) & 0x7F);
        c.y = 16 + ((c.y - 16 + 9) & 0x7F);
        guard += 1;
    }
    c
}

fn chebyshev(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Centers closer than four tiles still need a nudge. Four tiles is enough.
#[must_use]
pub fn towns_need_separation(a: IVec2, b: IVec2) -> bool {
    chebyshev(a, b) < 4
}

fn town_generation_index(id: &TownId) -> u32 {
    id.0
        .rsplit('-')
        .next()
        .and_then(|suffix| suffix.parse().ok())
        .unwrap_or(0)
}

const SEED_BAND_MIN: i32 = 16;
const SEED_BAND_MAX: i32 = 143;

fn in_seed_band(cell: IVec2) -> bool {
    (SEED_BAND_MIN..=SEED_BAND_MAX).contains(&cell.x)
        && (SEED_BAND_MIN..=SEED_BAND_MAX).contains(&cell.y)
}

/// Nearest dry cell inside the seed band that stays at least four tiles from `blocked`.
#[must_use]
pub fn nearest_dry_town_center(
    origin: IVec2,
    blocked: &[IVec2],
    is_water: &dyn Fn(IVec2) -> bool,
) -> Option<IVec2> {
    let origin = IVec2::new(seed_band_axis(origin.x), seed_band_axis(origin.y));
    let mut best: Option<(i32, i32, i32)> = None;
    for y in SEED_BAND_MIN..=SEED_BAND_MAX {
        for x in SEED_BAND_MIN..=SEED_BAND_MAX {
            let cell = IVec2::new(x, y);
            if is_water(cell) {
                continue;
            }
            if blocked.iter().any(|placed| towns_need_separation(*placed, cell)) {
                continue;
            }
            let key = (chebyshev(origin, cell), x, y);
            if best.is_none_or(|current| key < current) {
                best = Some(key);
            }
        }
    }
    best.map(|(_, x, y)| IVec2::new(x, y))
}

fn settle_centers_onto_dry(book: &mut TownBook, is_water: &dyn Fn(IVec2) -> bool) {
    if !book.towns.values().any(|town| is_water(town.center_tile)) {
        return;
    }
    let mut ids: Vec<_> = book.towns.keys().cloned().collect();
    ids.sort_by_key(|id| town_generation_index(id));
    let mut placed = Vec::new();
    let mut drop_ids = Vec::new();
    for id in ids {
        let Some(town) = book.towns.get_mut(&id) else {
            continue;
        };
        let cell = town.center_tile;
        let blocked = placed
            .iter()
            .any(|placed| towns_need_separation(*placed, cell));
        if !is_water(cell) && !blocked && in_seed_band(cell) {
            placed.push(cell);
            continue;
        }
        if let Some(next) = nearest_dry_town_center(cell, &placed, is_water) {
            town.center_tile = next;
            placed.push(next);
        } else {
            drop_ids.push(id);
        }
    }
    for id in drop_ids {
        book.towns.remove(&id);
        if book.default_town.as_ref() == Some(&id) {
            book.default_town = None;
        }
    }
    if book.default_town.is_none() {
        let mut keys: Vec<_> = book.towns.keys().cloned().collect();
        keys.sort_by_key(|id| town_generation_index(id));
        book.default_town = keys.into_iter().next();
    }
}

/// One-shot: generated centers that sit on [`crate::render::WaterSurfaceVisualCatalog`] move to dry land.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct GeneratedTownDrySnap {
    pub pending: bool,
}

pub fn snap_generated_towns_off_water(
    mut towns: ResMut<TownBook>,
    mut districts: ResMut<super::district::DistrictBook>,
    catalog: Option<Res<crate::render::WaterSurfaceVisualCatalog>>,
    mut snap: ResMut<GeneratedTownDrySnap>,
) {
    if !snap.pending {
        return;
    }
    let Some(catalog) = catalog else {
        return;
    };
    if catalog.grid_width == 0 || catalog.grid_height == 0 {
        return;
    }
    let is_water = |cell: IVec2| {
        if cell.x < 0 || cell.y < 0 {
            return true;
        }
        catalog.tile_kind(cell.x as u32, cell.y as u32)
            != crate::render::WaterSurfaceKind::None
    };
    let wet: Vec<IVec2> = towns
        .towns
        .iter()
        .filter(|(id, _)| id.0.starts_with("gen-"))
        .map(|(_, town)| town.center_tile)
        .filter(|cell| is_water(*cell))
        .collect();
    if wet.is_empty() {
        snap.pending = false;
        return;
    }
    settle_centers_onto_dry(&mut towns, &is_water);
    refresh_generated_zone_rects(&towns, &mut districts);
    snap.pending = false;
}

fn refresh_generated_zone_rects(
    towns: &TownBook,
    districts: &mut super::district::DistrictBook,
) {
    let live: Vec<_> = towns.towns.keys().cloned().collect();
    let stale: Vec<_> = districts
        .districts
        .iter()
        .filter(|(_, district)| {
            district.id.0.starts_with("dist-gen-") && !live.contains(&district.town_id)
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in stale {
        if districts.default_district.as_ref() == Some(&id) {
            districts.default_district = None;
        }
        districts.districts.remove(&id);
    }
    for town in towns.towns.values() {
        let id = DistrictId(format!("dist-{}", town.id.0));
        if let Some(district) = districts.districts.get_mut(&id) {
            let center = town.center_tile;
            district.tile_rect =
                IRect::from_corners(center - IVec2::new(8, 8), center + IVec2::new(8, 8));
            district.ring = super::district::core_ring_for_rect(district.tile_rect);
        }
    }
}

/// Adds one town on dry land, separated from towns already in the book.
/// `policy` defaults to [`TownAccessPolicy::FreeLabor`]. Does not touch construction rows.
#[must_use]
pub fn found_settlement(
    towns: &mut TownBook,
    districts: &mut super::district::DistrictBook,
    world_seed: u64,
    is_water: &dyn Fn(IVec2) -> bool,
    policy: Option<TownAccessPolicy>,
) -> Option<TownId> {
    let index = towns
        .towns
        .keys()
        .map(town_generation_index)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let id = TownId(format!("found-{world_seed}-{index}"));
    if towns.towns.contains_key(&id) {
        return None;
    }
    let hash = super::seed_chain::mix_u64(world_seed, "found", &index.to_string());
    let guess = IVec2::new(
        16 + (hash & 0x7F) as i32,
        16 + ((hash >> 8) & 0x7F) as i32,
    );
    let blocked: Vec<_> = towns.towns.values().map(|town| town.center_tile).collect();
    let center = nearest_dry_town_center(guess, &blocked, is_water)?;
    if blocked.iter().any(|placed| *placed == center || towns_need_separation(*placed, center)) {
        return None;
    }
    let access = policy.unwrap_or(TownAccessPolicy::FreeLabor);
    let region = region_name_for_seed(world_seed, index);
    let town = TownRecord {
        id: id.clone(),
        name: region.clone(),
        region_name: region,
        center_tile: center,
        population: 200,
        jobs: 40,
        housing: 80,
        faction: 1,
        access,
        required_clearance: required_clearance_for(access),
    };
    if towns.default_town.is_none() {
        towns.default_town = Some(id.clone());
    }
    towns.towns.insert(id.clone(), town.clone());
    let zone = DistrictId(format!("dist-{}", id.0));
    districts.districts.insert(
        zone.clone(),
        super::district::DistrictRecord {
            id: zone.clone(),
            town_id: id.clone(),
            name: format!("{} core", town.name),
            tile_rect: IRect::from_corners(center - IVec2::new(8, 8), center + IVec2::new(8, 8)),
            zoning_default: super::zoning::ZoningClass::Residential,
            ring: Vec::new(),
            style_rules: super::district::DistrictStyleRules::default(),
            site_ids: Vec::new(),
        },
    );
    if districts.default_district.is_none() {
        districts.default_district = Some(zone);
    }
    Some(id)
}

fn spread_generated_centers(towns: &mut TownBook) {
    let mut ids: Vec<_> = towns.towns.keys().cloned().collect();
    ids.sort_by_key(|id| town_generation_index(id));
    let mut placed = Vec::new();
    for id in ids {
        let Some(town) = towns.towns.get_mut(&id) else {
            continue;
        };
        let mut guard = 0;
        while placed
            .iter()
            .any(|p| towns_need_separation(*p, town.center_tile))
            && guard < 128
        {
            town.center_tile = separate_town_center(town.center_tile, &placed);
            if !placed.iter().any(|p| chebyshev(*p, town.center_tile) < 4) {
                break;
            }
            guard += 1;
        }
        placed.push(town.center_tile);
    }
}

/// Full generation writes a book only when the map asked for settlements and none are stored.
/// Count 0 is a request for no towns. Preview does not place. A later call does not replace a book.
pub fn place_towns_for_full_generation(
    full_job: bool,
    world_seed: u64,
    settlement_count: u32,
    towns: &mut TownBook,
    districts: &mut super::district::DistrictBook,
    snap: &mut GeneratedTownDrySnap,
    is_water: &dyn Fn(IVec2) -> bool,
) -> bool {
    if !full_job {
        return false;
    }
    snap.pending = false;
    if settlement_count == 0 || !towns.towns.is_empty() {
        return false;
    }
    *towns = generated_town_book_on_land(world_seed, settlement_count, is_water);
    *districts = super::district::generated_districts_for_towns(towns);
    super::district::stamp_core_rings(districts);
    snap.pending = false;
    true
}

/// Count passed to [`generated_town_book`] when a play session still has an empty town book.
pub const EMPTY_TOWN_BOOK_COUNT: u32 = 4;

/// **SET-P5-HIERARCHY-GAP** — seed playable town/district books when empty.
/// Save overlay hydrate remains authoritative and may overwrite after Startup.
/// `--test vfx` and `--test subfx` skip this so those worlds stay without towns.
pub fn seed_settlement_books_if_empty(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    mut towns: ResMut<TownBook>,
    mut districts: ResMut<super::district::DistrictBook>,
    mut snap: Option<ResMut<GeneratedTownDrySnap>>,
) {
    if launch
        .as_ref()
        .is_some_and(|args| args.test_scene.suppresses_town_book_seed())
    {
        return;
    }
    let need_town = towns.default_town.is_none() || towns.towns.is_empty();
    let need_district = districts.districts.is_empty();
    if !need_town && !need_district {
        return;
    }
    if need_town {
        *towns = generated_town_book(super::seed_chain::DEFAULT_WORLD_SEED, EMPTY_TOWN_BOOK_COUNT);
        if let Some(snap) = snap.as_mut() {
            snap.pending = true;
        }
    }
    if need_district {
        let generated = towns.towns.keys().any(|id| id.0.starts_with("gen-"));
        *districts = if generated {
            super::district::generated_districts_for_towns(&towns)
        } else if towns.towns.is_empty() {
            super::district::portland_fixture_district(&portland_fixture_town())
        } else {
            super::district::portland_fixture_district(&towns)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_town_book_is_deterministic() {
        let a = generated_town_book(99_001, 3);
        let b = generated_town_book(99_001, 3);
        assert_eq!(a, b);
        assert_eq!(a.towns.len(), 3);
        let id_a = TownId("gen-99001-0".into());
        let id_b = TownId("gen-12-0".into());
        let other = generated_town_book(12, 3);
        assert_ne!(a.towns[&id_a].center_tile, other.towns[&id_b].center_tile);
    }

    #[test]
    fn saved_town_book_is_kept_when_seeding() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(portland_fixture_town())
            .init_resource::<crate::strategic::settlement::DistrictBook>()
            .add_systems(Startup, seed_settlement_books_if_empty);
        app.update();
        let towns = app.world().resource::<TownBook>();
        assert!(towns.towns.contains_key(&TownId("portland".into())));
        assert!(!towns.towns.keys().any(|id| id.0.starts_with("gen-")));
    }

    #[test]
    fn empty_town_book_uses_generated_towns() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<TownBook>()
            .init_resource::<crate::strategic::settlement::DistrictBook>()
            .add_systems(Startup, seed_settlement_books_if_empty);
        app.update();
        let towns = app.world().resource::<TownBook>();
        assert!(towns.towns.keys().any(|id| id.0.starts_with("gen-")));
        let n = towns.towns.len();
        let districts = app.world().resource::<crate::strategic::settlement::DistrictBook>();
        assert_eq!(districts.districts.len(), n);
        for district in districts.districts.values() {
            assert!(towns.towns.contains_key(&district.town_id));
        }
    }

    #[test]
    fn generated_towns_each_get_a_district_and_stay_apart() {
        let towns = generated_town_book(99_001, 4);
        let districts = super::super::district::generated_districts_for_towns(&towns);
        assert_eq!(towns.towns.len(), 4);
        assert_eq!(districts.districts.len(), 4);
        for district in districts.districts.values() {
            assert!(towns.towns.contains_key(&district.town_id));
        }
        let centers: Vec<_> = towns.towns.values().map(|t| t.center_tile).collect();
        for i in 0..centers.len() {
            for j in (i + 1)..centers.len() {
                let d = (centers[i].x - centers[j].x)
                    .abs()
                    .max((centers[i].y - centers[j].y).abs());
                assert!(d >= 4, "{:?} {:?}", centers[i], centers[j]);
            }
        }
    }

    #[test]
    fn drain_later_town_does_not_move_an_earlier_center() {
        let seed = 44_201u64;
        let few = generated_town_book(seed, 3);
        let more = generated_town_book(seed, 6);
        for index in 0..3u32 {
            let id = super::TownId(format!("gen-{seed}-{index}"));
            let early = few.towns.get(&id).expect("early town").center_tile;
            let later = more.towns.get(&id).expect("later town").center_tile;
            assert_eq!(early, later, "town {index}");
        }
        assert_eq!(more.towns.len(), 6);
    }

    #[test]
    fn live_generated_housing_does_not_exceed_population() {
        assert_eq!(town_service_count(400, 811), 400);
        assert_eq!(town_service_count(500, 120), 120);
        let book = generated_town_book(7, 6);
        for town in book.towns.values() {
            assert!(town.housing <= town.population);
            assert!(town.jobs <= town.population);
        }
    }

    #[test]
    fn push_generated_town_count_stays_in_bounds() {
        assert_eq!(generated_town_count(0), 1);
        assert_eq!(generated_town_count(9), 8);
        assert_eq!(generated_town_book(3, 0).towns.len(), 1);
        assert_eq!(generated_town_book(3, 9).towns.len(), 8);
    }

    #[test]
    fn push_stacked_town_centers_stay_four_apart() {
        let blocked = [IVec2::new(20, 20)];
        let next = separate_town_center(IVec2::new(20, 20), &blocked);
        let gap = (next.x - 20).abs().max((next.y - 20).abs());
        assert!(gap >= 4, "{next:?}");
        assert!((16..=143).contains(&next.x));
        assert!((16..=143).contains(&next.y));
    }

    #[test]
    fn next_four_tiles_is_enough_separation() {
        assert!(towns_need_separation(IVec2::new(10, 10), IVec2::new(13, 10)));
        assert!(!towns_need_separation(IVec2::new(10, 10), IVec2::new(14, 10)));
        let stayed = separate_town_center(IVec2::new(24, 20), &[IVec2::new(20, 20)]);
        assert_eq!(stayed, IVec2::new(24, 20));
    }

    #[test]
    fn cont_town_center_stays_inside_the_seed_band() {
        let pulled = separate_town_center(IVec2::new(200, -20), &[]);
        assert!((16..=143).contains(&pulled.x));
        assert!((16..=143).contains(&pulled.y));
        let kept = separate_town_center(IVec2::new(24, 20), &[]);
        assert_eq!(kept, IVec2::new(24, 20));
    }

    #[test]
    fn over_zero_population_plans_no_housing() {
        assert_eq!(town_service_count(0, 1_200), 0);
        assert_eq!(town_service_count(0, 0), 0);
        assert_eq!(town_service_count(0, 80), 0);
    }

    #[test]
    fn over_seed_band_wraps_an_escaped_center() {
        let wrapped = separate_town_center(IVec2::new(144, 15), &[]);
        assert_eq!(wrapped, IVec2::new(16, 143));
        let rim = separate_town_center(IVec2::new(16, 143), &[]);
        assert_eq!(rim, IVec2::new(16, 143));
    }

    #[test]
    fn zone_water_cell_is_not_a_town_center() {
        let dry = generated_town_book(77_001, 2);
        let wet: Vec<_> = dry.towns.values().map(|town| town.center_tile).collect();
        let book = generated_town_book_on_land(77_001, 2, &|cell| wet.contains(&cell));
        assert!(!book.towns.is_empty());
        let mut seen = Vec::new();
        for town in book.towns.values() {
            assert!(!wet.contains(&town.center_tile), "{:?}", town.center_tile);
            assert!((16..=143).contains(&town.center_tile.x));
            assert!((16..=143).contains(&town.center_tile.y));
            assert!(!seen.contains(&town.center_tile));
            for prior in &seen {
                let gap = (town.center_tile.x - prior.x)
                    .abs()
                    .max((town.center_tile.y - prior.y).abs());
                assert!(gap >= 4, "{:?} {:?}", prior, town.center_tile);
            }
            seen.push(town.center_tile);
        }
    }

    #[test]
    fn zone_stamp_rect_scales_with_zoom() {
        let lo = crate::gui::map_tile_raster::map_tile_stamp_rect(10, 4, 3.0);
        let hi = crate::gui::map_tile_raster::map_tile_stamp_rect(12, 4, 6.0);
        let lo_a = crate::gui::map_tile_raster::map_tile_stamp_rect(10, 4, 3.0);
        let hi_a = crate::gui::map_tile_raster::map_tile_stamp_rect(10, 4, 6.0);
        let delta_lo = crate::gui::map_tile_raster::map_tile_stamp_rect(12, 4, 3.0)[0] - lo_a[0];
        let delta_hi = hi[0] - hi_a[0];
        assert!((delta_hi / delta_lo - 2.0).abs() < 1e-4, "{delta_lo} {delta_hi}");
        assert!((lo[0] / 3.0 - 10.0).abs() < 1e-4);
        assert!((hi_a[0] / 6.0 - 10.0).abs() < 1e-4);
        assert!((lo[2] * 2.0 - hi_a[2]).abs() < 1e-4);
    }

    #[test]
    fn zone_region_name_stable_for_seed() {
        assert_eq!(region_name_for_seed(42_001, 0), region_name_for_seed(42_001, 0));
        assert_ne!(region_name_for_seed(42_001, 0), region_name_for_seed(42_002, 0));
        let book = generated_town_book(42_001, 3);
        let id = TownId("gen-42001-0".into());
        assert_eq!(book.towns[&id].region_name, region_name_for_seed(42_001, 0));
        assert_eq!(book.towns[&id].region_name, book.towns[&id].name);
    }

    #[test]
    fn zone_founding_once_adds_one_and_again_does_not_overlap() {
        let mut pending = crate::construction::PendingConstructionQueue::default();
        let mut row = super::super::execute::pending_blueprint_from_growth_proposal(
            &super::super::growth::GrowthProposal {
                district_id: super::super::ids::DistrictId("player".into()),
                block_id: None,
                archetype_id: super::super::ids::ArchetypeId("warehouse".into()),
                usage: super::super::actors::BuildingUsage::Industrial,
                actor_layer: super::super::actors::GrowthActorLayer::Growth,
                anchor_tile: IVec2::new(3, 3),
                priority: 1.0,
                seed: 1,
                reason_codes: Vec::new(),
                saturation_at_submit: 0.0,
            },
        );
        row.label = "player:warehouse".into();
        pending.push(row);
        let before = pending.entries.len();
        let mut towns = generated_town_book(15, 2);
        let mut districts = super::super::district::generated_districts_for_towns(&towns);
        let n = towns.towns.len();
        let first = found_settlement(&mut towns, &mut districts, 15, &|_| false, None)
            .expect("first founding");
        assert_eq!(towns.towns.len(), n + 1);
        assert_eq!(towns.towns[&first].access, TownAccessPolicy::FreeLabor);
        let second = found_settlement(&mut towns, &mut districts, 15, &|_| false, None)
            .expect("second founding");
        assert_ne!(first, second);
        let a = towns.towns[&first].center_tile;
        let b = towns.towns[&second].center_tile;
        assert_ne!(a, b);
        let gap = (a.x - b.x).abs().max((a.y - b.y).abs());
        assert!(gap >= 4, "{a:?} {b:?}");
        assert_eq!(pending.entries.len(), before);
        assert!(pending.entries[0].label.starts_with("player:"));
    }

    #[test]
    fn zone_move_site_same_town_keeps_one_site() {
        let towns = generated_town_book(3, 2);
        let mut districts = super::super::district::generated_districts_for_towns(&towns);
        let town_id = TownId("gen-3-0".into());
        let home = DistrictId(format!("dist-{}", town_id.0));
        let other = super::super::district::add_admin_zone(
            &mut districts,
            &town_id,
            "yard",
            IRect::from_corners(IVec2::ZERO, IVec2::new(4, 4)),
        );
        assert!(super::super::district::place_site_in_zone(&mut districts, 9, &home));
        let policy = towns.towns[&town_id].access;
        assert!(super::super::district::move_building_zone(&mut districts, 9, &other));
        let copies = districts
            .districts
            .values()
            .filter(|district| district.site_ids.contains(&9))
            .count();
        assert_eq!(copies, 1);
        assert!(districts.districts[&other].site_ids.contains(&9));
        assert!(!districts.districts[&home].site_ids.contains(&9));
        assert_eq!(towns.towns[&town_id].access, policy);
    }

    #[test]
    fn zone_move_site_other_town_rejected() {
        let towns = generated_town_book(3, 2);
        let mut districts = super::super::district::generated_districts_for_towns(&towns);
        let home_town = TownId("gen-3-0".into());
        let foreign_town = TownId("gen-3-1".into());
        let home = DistrictId(format!("dist-{}", home_town.0));
        let foreign = DistrictId(format!("dist-{}", foreign_town.0));
        assert!(super::super::district::place_site_in_zone(&mut districts, 4, &home));
        assert!(!super::super::district::move_building_zone(
            &mut districts, 4, &foreign
        ));
        assert!(districts.districts[&home].site_ids.contains(&4));
        assert!(!districts.districts[&foreign].site_ids.contains(&4));
    }

    #[test]
    fn zone_military_closed_rejects_low_clearance_and_accepts_two() {
        let book = generated_town_book(8, 1);
        let town = book.towns.values().next().expect("town");
        assert_eq!(town.access, TownAccessPolicy::MilitaryClosed);
        assert_eq!(town.required_clearance, 2);
        assert!(!town_allows_entry(town, town.faction, 1));
        assert!(town_allows_growth_commit(town, town.faction, 2));
        assert!(!town_allows_logistics_entry(town, town.faction.wrapping_add(1), 2));
    }

    #[test]
    fn zone_free_labor_accepts_clearance_zero() {
        let book = generated_town_book(8, 3);
        let id = TownId("gen-8-2".into());
        let town = &book.towns[&id];
        assert_eq!(town.access, TownAccessPolicy::FreeLabor);
        assert!(town_allows_logistics_entry(town, 9, 0));
        assert!(town_allows_growth_commit(town, 9, 0));
    }

    #[test]
    fn zone_restricted_labor_rejects_zero_and_accepts_one() {
        let book = generated_town_book(8, 2);
        let id = TownId("gen-8-1".into());
        let town = &book.towns[&id];
        assert_eq!(town.access, TownAccessPolicy::RestrictedLabor);
        assert_eq!(town.required_clearance, 1);
        assert_ne!(town.access, TownAccessPolicy::MilitaryClosed);
        assert!(!town_allows_entry(town, town.faction, 0));
        assert!(town_allows_entry(town, 4, 1));
    }

    #[test]
    fn zone_policy_stable_for_seed() {
        let a = generated_town_book(55_010, 4);
        let b = generated_town_book(55_010, 4);
        assert_eq!(
            a.towns[&TownId("gen-55010-0".into())].access,
            TownAccessPolicy::MilitaryClosed
        );
        assert_eq!(
            a.towns[&TownId("gen-55010-1".into())].access,
            TownAccessPolicy::RestrictedLabor
        );
        assert_eq!(
            a.towns[&TownId("gen-55010-2".into())].access,
            TownAccessPolicy::FreeLabor
        );
        assert_eq!(
            a.towns[&TownId("gen-55010-3".into())].access,
            TownAccessPolicy::FreeLabor
        );
        for (id, town) in &a.towns {
            assert_eq!(town.access, b.towns[id].access);
            assert_eq!(town.region_name, b.towns[id].region_name);
            assert_eq!(town.required_clearance, b.towns[id].required_clearance);
        }
    }

    #[test]
    fn zone_transfer_does_not_retune_policy() {
        let towns = generated_town_book(12, 3);
        let mut districts = super::super::district::generated_districts_for_towns(&towns);
        let town_id = TownId("gen-12-0".into());
        let before = towns.towns[&town_id].access;
        assert_eq!(before, TownAccessPolicy::MilitaryClosed);
        let home = DistrictId(format!("dist-{}", town_id.0));
        let yard = super::super::district::add_admin_zone(
            &mut districts,
            &town_id,
            "yard",
            IRect::from_corners(IVec2::new(1, 1), IVec2::new(3, 3)),
        );
        assert!(super::super::district::place_site_in_zone(&mut districts, 1, &home));
        assert!(super::super::district::move_building_zone(&mut districts, 1, &yard));
        assert_eq!(towns.towns[&town_id].access, before);
        assert_eq!(towns.towns[&town_id].required_clearance, 2);
        let open = &towns.towns[&TownId("gen-12-2".into())];
        assert_eq!(open.access, TownAccessPolicy::FreeLabor);
        assert_ne!(open.access, before);
    }

    #[test]
    fn twn_gen_places_on_dry_land() {
        assert_eq!(generated_town_count(0), 1);
        let water = |cell: IVec2| cell.y == 0;
        let mut towns = TownBook::default();
        let mut districts = super::super::district::DistrictBook::default();
        let mut snap = GeneratedTownDrySnap { pending: true };
        assert!(!place_towns_for_full_generation(
            true, 99_001, 0, &mut towns, &mut districts, &mut snap, &water
        ));
        assert!(towns.towns.is_empty());
        assert!(!snap.pending);
        snap.pending = true;
        assert!(!place_towns_for_full_generation(
            false, 99_001, 3, &mut towns, &mut districts, &mut snap, &water
        ));
        assert!(towns.towns.is_empty());
        assert!(snap.pending);
        assert!(place_towns_for_full_generation(
            true, 99_001, 3, &mut towns, &mut districts, &mut snap, &water
        ));
        assert!(!snap.pending);
        assert_eq!(towns.towns.len(), 3);
        assert_eq!(districts.districts.len(), 3);
        let centers: Vec<_> = towns.towns.values().map(|town| town.center_tile).collect();
        for town in towns.towns.values() {
            assert!(!water(town.center_tile));
            assert!(!town.region_name.is_empty());
            let district = districts
                .districts
                .values()
                .find(|district| district.town_id == town.id)
                .expect("district");
            assert_eq!(district.ring.len(), 4);
            assert!(district.contains_tile(town.center_tile));
            assert_eq!(
                district.ring,
                super::super::district::core_ring_for_rect(district.tile_rect)
            );
        }
        for i in 0..centers.len() {
            for j in (i + 1)..centers.len() {
                assert!(!towns_need_separation(centers[i], centers[j]));
            }
        }
        let kept: Vec<_> = towns
            .towns
            .values()
            .map(|town| (town.id.0.clone(), town.center_tile, town.region_name.clone()))
            .collect();
        assert!(!place_towns_for_full_generation(
            true, 1, 4, &mut towns, &mut districts, &mut snap, &water
        ));
        assert_eq!(towns.towns.len(), kept.len());
        for (id, center, region) in kept {
            let town = &towns.towns[&TownId(id)];
            assert_eq!(town.center_tile, center);
            assert_eq!(town.region_name, region);
        }
        assert!(!snap.pending);
    }
}
