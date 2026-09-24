//! Town book authority (SET-P5-001 fixture stub until A lands full loader).

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ids::TownId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TownRecord {
    pub id: TownId,
    pub name: String,
    pub center_tile: IVec2,
    pub population: u32,
    pub jobs: u32,
    pub housing: u32,
}

#[derive(Resource, Debug, Default, Clone)]
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
            center_tile: IVec2::new(64, 64),
            population: 12_000,
            jobs: 4_500,
            housing: 10_000,
        },
    );
    TownBook {
        towns,
        default_town: Some(id),
    }
}

/// **SET-P5-HIERARCHY-GAP** — seed playable town/district books when empty.
/// Save overlay hydrate remains authoritative and may overwrite after Startup.
pub fn seed_settlement_books_if_empty(
    mut towns: ResMut<TownBook>,
    mut districts: ResMut<super::district::DistrictBook>,
) {
    let need_town = towns.default_town.is_none() || towns.towns.is_empty();
    let need_district = districts.districts.is_empty();
    if !need_town && !need_district {
        return;
    }
    let seeded_town = portland_fixture_town();
    if need_town {
        *towns = seeded_town.clone();
    }
    if need_district {
        *districts = super::district::portland_fixture_district(&seeded_town);
    }
}
