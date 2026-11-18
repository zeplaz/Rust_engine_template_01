//! PLAN-WATER-DEPTH-TRUTH-v1 **W5 — DEFER**.
//!
//! Optional per-cell salinity scalar (`0 = fresh`). Ocean / Dead Sea are chemistry + look,
//! not a second water type. No gameplay fork; wire look/effects only when a consumer needs it.

use bevy::prelude::*;

/// Deferred salinity field. Not written by world-gen yet; resource exists so depth-truth
/// callers can opt in without inventing a `WaterClass::Saline` enum.
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldWaterSalinity {
    pub width: u32,
    pub height: u32,
    /// 0.0 = fresh … 1.0 = saturated brine (relative, not ppt).
    pub salinity: Vec<f32>,
    pub stamp: u64,
}

impl WorldWaterSalinity {
    #[inline]
    #[must_use]
    pub fn at_world(&self, wx: i32, wz: i32) -> Option<f32> {
        if wx < 0 || wz < 0 || self.width == 0 || self.height == 0 {
            return None;
        }
        let i = (wz as u32)
            .checked_mul(self.width)
            .and_then(|row| row.checked_add(wx as u32))? as usize;
        self.salinity.get(i).copied()
    }
}
