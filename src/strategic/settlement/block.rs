//! Block book — grid clusters within districts (SET-P5-002).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::ids::{BlockId, DistrictId};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockRecord {
    pub id: BlockId,
    pub district_id: DistrictId,
    pub tiles: HashSet<IVec2>,
    pub site_ids: Vec<u64>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct BlockBook {
    pub blocks: HashMap<BlockId, BlockRecord>,
    pub tile_to_block: HashMap<IVec2, BlockId>,
}
