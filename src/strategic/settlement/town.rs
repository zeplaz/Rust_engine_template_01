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
