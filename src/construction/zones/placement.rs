//! Painted zone tiles (preview only until shift-commit → pending queue).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;

use super::super::build_tool_authority::ZoneTool;

#[derive(Resource, Debug, Default)]
pub struct ActiveZonePaint {
    pub zone: Option<ZoneTool>,
    pub painted: Vec<BuildSiteTile>,
    pub drag_active: bool,
    pub rect_anchor: Option<BuildSiteTile>,
}

impl ActiveZonePaint {
    pub fn contains(&self, tile: BuildSiteTile) -> bool {
        self.painted.iter().any(|t| *t == tile)
    }

    pub fn push_unique(&mut self, tile: BuildSiteTile) {
        if !self.contains(tile) {
            self.painted.push(tile);
        }
    }

    pub fn pop_last(&mut self) {
        self.painted.pop();
    }

    pub fn clear(&mut self) {
        self.painted.clear();
        self.drag_active = false;
        self.rect_anchor = None;
    }

    pub fn fill_rectangle(&mut self, a: BuildSiteTile, b: BuildSiteTile) {
        let x0 = a.x.min(b.x);
        let x1 = a.x.max(b.x);
        let z0 = a.z.min(b.z);
        let z1 = a.z.max(b.z);
        for z in z0..=z1 {
            for x in x0..=x1 {
                self.push_unique(BuildSiteTile { x, z });
            }
        }
    }
}
