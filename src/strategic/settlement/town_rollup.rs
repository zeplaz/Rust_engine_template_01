//! **PROC-OG-4-ROLLUP-001** — district / block site counts → town population, jobs, housing.

use std::collections::HashMap;

use bevy::prelude::*;

use super::block::BlockBook;
use super::district::DistrictBook;
use super::ids::DistrictId;
use super::town::{portland_fixture_town, TownBook};

const BASE_POPULATION: u32 = 8_000;
const JOBS_PER_SITE: u32 = 120;
const HOUSING_UNITS_PER_SITE: u32 = 10;
const POPULATION_PER_HOUSING_UNIT: u32 = 3;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DistrictSiteRollup {
    pub site_count: u32,
    pub jobs: u32,
    pub housing: u32,
    pub population: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TownRollupSnapshot {
    pub population: u32,
    pub jobs: u32,
    pub housing: u32,
    pub district_rollups: HashMap<DistrictId, DistrictSiteRollup>,
}

#[must_use]
pub fn rollup_district_site_counts(
    blocks: &BlockBook,
    district_book: &DistrictBook,
) -> HashMap<DistrictId, DistrictSiteRollup> {
    let mut by_district: HashMap<DistrictId, DistrictSiteRollup> = HashMap::new();
    for block in blocks.blocks.values() {
        let site_count = block.site_ids.len() as u32;
        if site_count == 0 {
            continue;
        }
        let entry = by_district.entry(block.district_id.clone()).or_default();
        entry.site_count = entry.site_count.saturating_add(site_count);
        entry.jobs = entry.jobs.saturating_add(site_count.saturating_mul(JOBS_PER_SITE));
        entry.housing = entry
            .housing
            .saturating_add(site_count.saturating_mul(HOUSING_UNITS_PER_SITE));
        entry.population = entry
            .population
            .saturating_add(site_count.saturating_mul(HOUSING_UNITS_PER_SITE * POPULATION_PER_HOUSING_UNIT));
    }
    for district in district_book.districts.keys() {
        by_district.entry(district.clone()).or_default();
    }
    by_district
}

/// Apply block site counts onto the default town record (runs before district metrics).
pub fn rollup_town_metrics_from_districts(
    town_book: &mut TownBook,
    blocks: &BlockBook,
    district_book: &DistrictBook,
) -> TownRollupSnapshot {
    let district_rollups = rollup_district_site_counts(blocks, district_book);
    let jobs = district_rollups.values().map(|r| r.jobs).sum::<u32>();
    let housing = district_rollups.values().map(|r| r.housing).sum::<u32>();
    let population = district_rollups
        .values()
        .map(|r| r.population)
        .sum::<u32>();

    if jobs == 0 && housing == 0 {
        if let Some(town_id) = town_book.default_town.clone() {
            if let Some(town) = town_book.towns.get(&town_id) {
                return TownRollupSnapshot {
                    population: town.population,
                    jobs: town.jobs,
                    housing: town.housing,
                    district_rollups,
                };
            }
        }
    }

    let population = population.max(BASE_POPULATION);
    if let Some(town_id) = town_book.default_town.clone() {
        if let Some(town) = town_book.towns.get_mut(&town_id) {
            town.population = population;
            town.jobs = jobs;
            town.housing = housing;
        }
    }

    TownRollupSnapshot {
        population,
        jobs,
        housing,
        district_rollups,
    }
}

pub fn rollup_town_metrics_from_districts_system(
    mut town_book: ResMut<TownBook>,
    blocks: Res<BlockBook>,
    district_book: Res<DistrictBook>,
) {
    rollup_town_metrics_from_districts(
        town_book.as_mut(),
        blocks.as_ref(),
        district_book.as_ref(),
    );
}

#[must_use]
pub fn proc_og4_rollup_witness_green() -> bool {
    use super::assign::register_site_on_commit;
    use super::district::portland_fixture_district;
    use crate::strategic::SiteId;

    let mut town_book = portland_fixture_town();
    let district_book = portland_fixture_district(&town_book);
    let mut blocks = BlockBook::default();

    for (i, (x, z)) in [(12, 12), (14, 12), (16, 12)].iter().enumerate() {
        register_site_on_commit(
            &district_book,
            &mut blocks,
            SiteId(i as u64 + 1),
            &[IVec2::new(*x, *z)],
        );
    }

    let district_rollups = rollup_district_site_counts(&blocks, &district_book);
    let site_total: u32 = district_rollups.values().map(|r| r.site_count).sum();
    if site_total != 3 {
        return false;
    }

    let snap = rollup_town_metrics_from_districts(&mut town_book, &blocks, &district_book);
    snap.jobs == 3 * JOBS_PER_SITE
        && snap.housing == 3 * HOUSING_UNITS_PER_SITE
        && snap.population >= BASE_POPULATION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_og4_rollup_witness_green() {
        assert!(super::proc_og4_rollup_witness_green());
    }
}
