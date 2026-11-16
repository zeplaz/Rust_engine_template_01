//! **Faction AI perception** — one pure read of the authoritative overlay for one faction slot.
//!
//! Reads [`ChunkStrategicOverlay`] (control, threat, logistics strength) the same way the player's
//! map does. Returns a value type; nothing is stored as a second grid (`MAI-CUT-INFLUENCE`,
//! `MAI-CUT-BOARD`). The decision log keeps only the few scalars that drove a decision.
//!
//! **DEC-12** (aig3 5.9): this is a *knowledge-source local view*, not a second blackboard.
//! Shared board = [`crate::strategic::WorldReadSnapshot`]; experts write via
//! [`crate::strategic::BlackboardWriteLane`] queues only.
//!
//! **SENSE-03** (aig3 11.4 *Sense Management*): [`SensoryModality`] / [`SenseMask`]
//! gate which channels [`perceive_faction_masked`] may read. Theaters bound AOR
//! (`combat/theater.rs`); modalities bound *what* is knowable.
//! **SENSE-05** (aig3 11.4.1 *Faking It*): [`PerceptionFidelity::PerfectKnowledge`]
//! is an explicit cheat path — use for soft AI / debug only; players notice omniscience.

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
    /// TAC-05: mean gen-time defensibility along the front (0 when no analysis).
    pub mean_front_defensibility: f32,
    /// TAC-05: mean gen-time chokepoint along the front (0 when no analysis).
    pub mean_front_chokepoint: f32,
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

    /// TAC-05 RO: sample [`TerrainAnalysisBook`](crate::terrain::TerrainAnalysisBook) onto the front digest.
    pub fn apply_terrain_analysis(&mut self, book: &crate::terrain::TerrainAnalysisBook) {
        self.mean_front_defensibility =
            book.mean_defensibility(self.front.iter().map(|c| &c.tile));
        self.mean_front_chokepoint = book.mean_chokepoint(self.front.iter().map(|c| &c.tile));
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


/// SENSE-03: sensory channel a faction may possess.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensoryModality {
    /// Control / ownership fields.
    Visual,
    /// Threat pressure.
    Threat,
    /// Logistics strength / supply.
    Logistics,
}

/// SENSE-03: bit-set of enabled modalities (default = all).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SenseMask {
    pub visual: bool,
    pub threat: bool,
    pub logistics: bool,
}

impl Default for SenseMask {
    fn default() -> Self {
        Self {
            visual: true,
            threat: true,
            logistics: true,
        }
    }
}

impl SenseMask {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            visual: true,
            threat: true,
            logistics: true,
        }
    }

    #[must_use]
    pub const fn only(modality: SensoryModality) -> Self {
        match modality {
            SensoryModality::Visual => Self {
                visual: true,
                threat: false,
                logistics: false,
            },
            SensoryModality::Threat => Self {
                visual: false,
                threat: true,
                logistics: false,
            },
            SensoryModality::Logistics => Self {
                visual: false,
                threat: false,
                logistics: true,
            },
        }
    }

    #[must_use]
    pub fn allows(self, modality: SensoryModality) -> bool {
        match modality {
            SensoryModality::Visual => self.visual,
            SensoryModality::Threat => self.threat,
            SensoryModality::Logistics => self.logistics,
        }
    }
}


/// SENSE-05: how honest perception is. Perfect knowledge is a deliberate cheat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PerceptionFidelity {
    /// Respect [`SenseMask`] — fair fog of war.
    #[default]
    Honest,
    /// Omniscient overlay read (all modalities). Acceptable for soft opponents / tools;
    /// players notice if hard AI always knows hidden logistics.
    PerfectKnowledge,
}

/// SENSE-05: perceive with fidelity policy. PerfectKnowledge ignores `mask` and uses all channels.
#[must_use]
pub fn perceive_with_fidelity<'a>(
    slot: u8,
    overlays: impl IntoIterator<Item = &'a ChunkStrategicOverlay>,
    mask: SenseMask,
    fidelity: PerceptionFidelity,
) -> FactionPerception {
    match fidelity {
        PerceptionFidelity::Honest => perceive_faction_masked(slot, overlays, mask),
        PerceptionFidelity::PerfectKnowledge => {
            perceive_faction_masked(slot, overlays, SenseMask::all())
        }
    }
}

/// True when fidelity is a cheat players can notice (SENSE-05).
#[inline]
#[must_use]
pub const fn perception_is_faked(fidelity: PerceptionFidelity) -> bool {
    matches!(fidelity, PerceptionFidelity::PerfectKnowledge)
}

/// SENSE-03: perceive with modality gates — disabled channels zero their contributions.
#[must_use]
pub fn perceive_faction_masked<'a>(
    slot: u8,
    overlays: impl IntoIterator<Item = &'a ChunkStrategicOverlay>,
    mask: SenseMask,
) -> FactionPerception {
    let mut p = perceive_faction(slot, overlays);
    if !mask.allows(SensoryModality::Threat) {
        p.mean_threat_held = 0.0;
        for c in &mut p.front {
            c.threat = 0.0;
        }
    }
    if !mask.allows(SensoryModality::Logistics) {
        p.mean_logistics_held = 0.0;
        for c in &mut p.front {
            c.logistics_strength = 0.0;
        }
        p.rear_anchor = None;
    }
    if !mask.allows(SensoryModality::Visual) {
        p.cells_held = 0;
        p.mean_own_control = 0.0;
        p.front.clear();
        p.rival_slots.clear();
        p.rear_anchor = None;
    }
    p
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
    #[test]
    fn sense03_modality_mask_blinds_threat_and_logistics() {
        let mut ov = two_faction_strip();
        ov.threat[2][0] = 0.9;
        ov.logistics_strength[0][0] = 0.9;
        let full = perceive_faction_masked(0, [&ov], SenseMask::all());
        assert!(full.mean_threat_held > 0.0 || full.most_threatened_front().is_some());
        let no_threat = perceive_faction_masked(0, [&ov], SenseMask {
            visual: true,
            threat: false,
            logistics: true,
        });
        assert_eq!(no_threat.mean_threat_held, 0.0);
        assert!(no_threat.front.iter().all(|c| c.threat == 0.0));
        let blind = perceive_faction_masked(0, [&ov], SenseMask::only(SensoryModality::Logistics));
        assert!(blind.front.is_empty());
        assert_eq!(blind.cells_held, 0);
    }

    #[test]
    fn sense05_perfect_knowledge_bypasses_honest_mask() {
        let mut ov = two_faction_strip();
        ov.threat[2][0] = 0.9;
        let limited = SenseMask {
            visual: true,
            threat: false,
            logistics: true,
        };
        let honest = perceive_with_fidelity(0, [&ov], limited, PerceptionFidelity::Honest);
        let fake = perceive_with_fidelity(0, [&ov], limited, PerceptionFidelity::PerfectKnowledge);
        assert_eq!(honest.mean_threat_held, 0.0);
        assert!(fake.front.iter().any(|c| c.threat > 0.0));
        assert!(perception_is_faked(PerceptionFidelity::PerfectKnowledge));
        assert!(!perception_is_faked(PerceptionFidelity::Honest));
    }

}
