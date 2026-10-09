//! **Faction AI perception** — one pure read of the authoritative overlay for one faction slot.
//!
//! Reads [`ChunkStrategicOverlay`] (control, threat, logistics strength) the same way the player's
//! map does. Returns a value type; nothing is stored as a second grid (`MAI-CUT-INFLUENCE`,
//! `MAI-CUT-BOARD`). The decision log keeps only the few scalars that drove a decision.

use bevy::prelude::IVec2;

use crate::strategic::{ChunkStrategicOverlay, MAX_STRATEGIC_FACTION_SLOTS};

/// A cell counts as held when the slot's control is above this and leads every other slot.
pub const HELD_CONTROL_MIN: f32 = 0.5;
/// Same tie band as [`crate::strategic::derive_frontline_from_control_system`].
pub const FRONT_TIE_LOW: f32 = 0.25;
pub const FRONT_TIE_BAND: f32 = 0.3;
/// Own logistics strength below this on a front cell counts as under-supplied.
pub const SUPPLY_SHORT_BELOW: f32 = 0.3;

/// One contested cell on this faction's front, in world tiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrontCell {
    pub tile: IVec2,
    pub chunk: IVec2,
    pub own_control: f32,
    pub enemy_control: f32,
    pub enemy_slot: u8,
    /// `threat[cell][slot]` — pressure this faction experiences on the cell.
    pub threat: f32,
    pub logistics_strength: f32,
}

impl FrontCell {
    /// Positive when this faction leads the cell.
    #[inline]
    #[must_use]
    pub fn advantage(&self) -> f32 {
        self.own_control - self.enemy_control
    }
}

/// Digest of one faction slot's position. Built fresh every decision; never a resource.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FactionPerception {
    pub slot: u8,
    pub cells_total: u32,
    pub cells_held: u32,
    pub mean_own_control: f32,
    pub mean_threat_held: f32,
    pub mean_logistics_held: f32,
    /// Contested cells involving this slot, sorted `(y, x)`.
    pub front: Vec<FrontCell>,
    /// Slots that hold any control somewhere (excluding `slot`), ascending.
    pub rival_slots: Vec<u8>,
    /// Held cell with the best logistics strength — the anchor a supply line draws from.
    pub rear_anchor: Option<IVec2>,
}

impl FactionPerception {
    #[must_use]
    pub fn mean_front_threat(&self) -> f32 {
        mean(self.front.iter().map(|c| c.threat))
    }

    #[must_use]
    pub fn mean_front_advantage(&self) -> f32 {
        mean(self.front.iter().map(FrontCell::advantage))
    }

    #[must_use]
    pub fn mean_front_logistics(&self) -> f32 {
        mean(self.front.iter().map(|c| c.logistics_strength))
    }

    /// Share of front cells below [`SUPPLY_SHORT_BELOW`].
    #[must_use]
    pub fn front_supply_shortfall(&self) -> f32 {
        if self.front.is_empty() {
            return 0.0;
        }
        let short = self
            .front
            .iter()
            .filter(|c| c.logistics_strength < SUPPLY_SHORT_BELOW)
            .count();
        short as f32 / self.front.len() as f32
    }

    /// Front cell under the most threat (ties: lower advantage, then `(y, x)`).
    #[must_use]
    pub fn most_threatened_front(&self) -> Option<&FrontCell> {
        self.front.iter().max_by(|a, b| {
            a.threat
                .total_cmp(&b.threat)
                .then(b.advantage().total_cmp(&a.advantage()))
                .then(b.tile.y.cmp(&a.tile.y))
                .then(b.tile.x.cmp(&a.tile.x))
        })
    }

    /// Front cell where this faction leads most with the least threat — the push point.
    #[must_use]
    pub fn best_push_front(&self) -> Option<&FrontCell> {
        self.front.iter().max_by(|a, b| {
            (a.advantage() - a.threat)
                .total_cmp(&(b.advantage() - b.threat))
                .then(b.tile.y.cmp(&a.tile.y))
                .then(b.tile.x.cmp(&a.tile.x))
        })
    }

    /// Worst-supplied front cell (ties `(y, x)`).
    #[must_use]
    pub fn worst_supplied_front(&self) -> Option<&FrontCell> {
        self.front.iter().min_by(|a, b| {
            a.logistics_strength
                .total_cmp(&b.logistics_strength)
                .then(a.tile.y.cmp(&b.tile.y))
                .then(a.tile.x.cmp(&b.tile.x))
        })
    }
}

fn mean(it: impl Iterator<Item = f32>) -> f32 {
    let (sum, n) = it.fold((0.0_f32, 0u32), |(s, n), v| (s + v, n + 1));
    if n == 0 {
        0.0
    } else {
        sum / n as f32
    }
}

/// World tile for a cell index on an overlay chunk.
#[inline]
#[must_use]
pub fn overlay_cell_tile(overlay: &ChunkStrategicOverlay, cell: usize) -> Option<IVec2> {
    let sx = overlay.size.x as usize;
    if sx == 0 || cell >= overlay.len_cells() {
        return None;
    }
    let lx = (cell % sx) as i32;
    let ly = (cell / sx) as i32;
    Some(IVec2::new(
        overlay.chunk_coord.x * overlay.size.x as i32 + lx,
        overlay.chunk_coord.y * overlay.size.y as i32 + ly,
    ))
}

/// Pure perception for `slot` over the given overlays. Deterministic: output order does not depend
/// on query iteration order.
#[must_use]
pub fn perceive_faction<'a>(
    slot: u8,
    overlays: impl IntoIterator<Item = &'a ChunkStrategicOverlay>,
) -> FactionPerception {
    let s = slot as usize;
    let mut out = FactionPerception {
        slot,
        ..FactionPerception::default()
    };
    if s >= MAX_STRATEGIC_FACTION_SLOTS {
        return out;
    }
    let mut rivals = [false; MAX_STRATEGIC_FACTION_SLOTS];
    let (mut control_sum, mut threat_held, mut logi_held) = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut rear: Option<(f32, IVec2)> = None;

    for ov in overlays {
        for cell in 0..ov.len_cells() {
            let Some(tile) = overlay_cell_tile(ov, cell) else {
                continue;
            };
            let control = &ov.faction_control[cell];
            let own = control[s];
            out.cells_total += 1;
            control_sum += own;

            let mut enemy_slot = None;
            let mut enemy = 0.0_f32;
            for (other, value) in control.iter().enumerate() {
                if other == s {
                    continue;
                }
                if *value > 0.0 {
                    rivals[other] = true;
                }
                if *value > enemy {
                    enemy = *value;
                    enemy_slot = Some(other as u8);
                }
            }
            let threat = ov.threat.get(cell).map_or(0.0, |r| r[s]);
            let logistics = ov.logistics_strength.get(cell).map_or(0.0, |r| r[s]);

            if own > HELD_CONTROL_MIN && own > enemy {
                out.cells_held += 1;
                threat_held += threat;
                logi_held += logistics;
                let better = rear.map_or(true, |(best, t)| {
                    logistics > best || (logistics == best && (tile.y, tile.x) < (t.y, t.x))
                });
                if better {
                    rear = Some((logistics, tile));
                }
            }
            if let Some(enemy_slot) = enemy_slot {
                if own > FRONT_TIE_LOW && enemy > FRONT_TIE_LOW && (own - enemy).abs() < FRONT_TIE_BAND {
                    out.front.push(FrontCell {
                        tile,
                        chunk: ov.chunk_coord,
                        own_control: own,
                        enemy_control: enemy,
                        enemy_slot,
                        threat,
                        logistics_strength: logistics,
                    });
                }
            }
        }
    }

    out.front.sort_by(|a, b| (a.tile.y, a.tile.x).cmp(&(b.tile.y, b.tile.x)));
    out.rival_slots = (0..MAX_STRATEGIC_FACTION_SLOTS as u8)
        .filter(|o| *o != slot && rivals[*o as usize])
        .collect();
    if out.cells_total > 0 {
        out.mean_own_control = control_sum / out.cells_total as f32;
    }
    if out.cells_held > 0 {
        out.mean_threat_held = threat_held / out.cells_held as f32;
        out.mean_logistics_held = logi_held / out.cells_held as f32;
    }
    out.rear_anchor = rear.map(|(_, t)| t);
    out
}

#[cfg(test)]
pub(crate) mod test_support {
    use bevy::prelude::{IVec2, UVec2};

    use crate::strategic::ChunkStrategicOverlay;

    /// 4×1 strip: slot 0 holds x=0, contested at x=1..2, slot 1 holds x=3.
    #[must_use]
    pub fn two_faction_strip() -> ChunkStrategicOverlay {
        let mut ov = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(4, 1));
        let rows = [(0.9, 0.0), (0.5, 0.45), (0.4, 0.5), (0.0, 0.9)];
        for (i, (a, b)) in rows.into_iter().enumerate() {
            ov.faction_control[i][0] = a;
            ov.faction_control[i][1] = b;
        }
        ov.logistics_strength[0][0] = 0.8;
        ov.logistics_strength[3][1] = 0.8;
        ov
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::two_faction_strip;
    use super::*;

    #[test]
    fn fai_perception_finds_front_and_rear() {
        let mut ov = two_faction_strip();
        ov.threat[2][0] = 0.7;
        let p = perceive_faction(0, [&ov]);
        assert_eq!(p.cells_total, 4);
        assert_eq!(p.cells_held, 1);
        assert_eq!(p.rival_slots, vec![1]);
        assert_eq!(p.front.len(), 2);
        assert_eq!(p.rear_anchor, Some(IVec2::new(0, 0)));
        assert_eq!(p.most_threatened_front().map(|c| c.tile), Some(IVec2::new(2, 0)));
        assert_eq!(p.best_push_front().map(|c| c.tile), Some(IVec2::new(1, 0)));
        assert!((p.front_supply_shortfall() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn fai_perception_is_symmetric_for_the_other_slot() {
        let ov = two_faction_strip();
        let p = perceive_faction(1, [&ov]);
        assert_eq!(p.cells_held, 1);
        assert_eq!(p.rival_slots, vec![0]);
        assert_eq!(p.front.len(), 2);
        assert_eq!(p.rear_anchor, Some(IVec2::new(3, 0)));
        assert!(p.front.iter().all(|c| c.enemy_slot == 0));
    }

    #[test]
    fn fai_perception_out_of_range_slot_is_empty() {
        let ov = two_faction_strip();
        let p = perceive_faction(MAX_STRATEGIC_FACTION_SLOTS as u8, [&ov]);
        assert_eq!(p.cells_total, 0);
        assert!(p.front.is_empty());
    }
}
