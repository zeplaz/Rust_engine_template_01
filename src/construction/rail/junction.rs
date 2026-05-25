//! Rail junction / switch authority (Round 3-F).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;

use super::super::roads::IntersectionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RailNodeKind {
    Switch,
    Junction,
}

#[derive(Debug, Clone)]
pub struct RailJunctionRecord {
    pub tile: BuildSiteTile,
    pub kind: RailNodeKind,
    pub intersection: Option<IntersectionId>,
}

#[derive(Resource, Default, Debug)]
pub struct RailJunctionAuthority {
    pub junctions: Vec<RailJunctionRecord>,
}

impl RailJunctionAuthority {
    pub fn register_switch(&mut self, tile: BuildSiteTile, intersection: Option<IntersectionId>) {
        self.junctions.push(RailJunctionRecord {
            tile,
            kind: RailNodeKind::Switch,
            intersection,
        });
    }
}
