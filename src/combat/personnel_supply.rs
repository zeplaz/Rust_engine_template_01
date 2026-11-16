//! Spine P5 — ground UnitKind personnel restock (toe_personnel deficit → front PublishDemand).
//!
//! Four sustainment tags (`ammo` / `Fuel` / `rations` / `engineering`) stay on FC-6
//! [`super::supply`]. This module only closes the `personnel` cargo loop:
//! deficit publish → ConsumeStock → credits applied inside [`super::resolve::combat_resolve_system`]
//! (sole writer of [`Formation::strength`]).

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::balance::{FrontBalance, UnitKind};
use super::formation::Formation;
use super::front_stock_pool::FrontStockPool;
use crate::economy::logistics::orders::LogisticsOrderRequest;

/// Side → replacement personnel still owed to under-strength formations.
///
/// `published_deficit` writer: [`super::toe_demand::publish_toe_cargo_demands_system`].
/// Credits writer: [`refill_personnel_credits_sink`] via
/// [`super::toe_refill::refill_toe_from_front_system`]. Consume via
/// [`take_personnel_credit`] from combat resolve only.
#[derive(Resource, Debug, Default, Clone)]
pub struct PersonnelRestockBook {
    pub by_side: BTreeMap<u8, u32>,
    pub published_deficit: BTreeMap<u8, u32>,
}

/// Persons missing vs TOE for one formation: `ceil(toe_personnel * (1 - strength))`.
#[must_use]
pub fn formation_personnel_deficit(f: &Formation, balance: &FrontBalance) -> u32 {
    if f.strength >= 0.999 {
        return 0;
    }
    let toe = balance.unit(f.kind).toe_personnel.max(1) as f32;
    let missing = toe * (1.0 - f.strength.clamp(0.0, 1.0));
    missing.ceil().max(0.0) as u32
}

/// Aggregate deficit for `side` (all formations of that slot).
#[must_use]
pub fn side_personnel_deficit<'a>(
    side: u8,
    balance: &FrontBalance,
    formations: impl Iterator<Item = &'a Formation>,
) -> u32 {
    formations
        .filter(|f| f.faction_slot == side)
        .map(|f| formation_personnel_deficit(f, balance))
        .fold(0u32, |a, b| a.saturating_add(b))
}

/// Typed sink: pull front `personnel` into credits (not Formation.strength) (**CS-6**).
/// Scheduled only via [`super::toe_refill::refill_toe_from_front_system`].
pub fn refill_personnel_credits_sink(
    pool: &mut FrontStockPool,
    book: &mut PersonnelRestockBook,
    requests: &mut MessageWriter<LogisticsOrderRequest>,
) {
    for (side, need) in book.published_deficit.clone() {
        if need == 0 {
            continue;
        }
        let already = book.by_side.get(&side).copied().unwrap_or(0);
        if already >= need {
            continue;
        }
        let want = need.saturating_sub(already);
        let take = pool.consume_tag_up_to(side, "personnel", want, requests);
        if take == 0 {
            continue;
        }
        *book.by_side.entry(side).or_insert(0) = already.saturating_add(take);
    }
}

/// Take up to `n` replacement persons for `side` (combat resolve only).
pub fn take_personnel_credit(book: &mut PersonnelRestockBook, side: u8, n: u32) -> u32 {
    let Some(have) = book.by_side.get_mut(&side) else {
        return 0;
    };
    let got = (*have).min(n);
    *have = have.saturating_sub(got);
    if *have == 0 {
        book.by_side.remove(&side);
    }
    got
}

/// Strength delta from applying `persons` against TOE (resolve-owned).
#[must_use]
pub fn strength_from_replacements(kind: UnitKind, balance: &FrontBalance, persons: u32) -> f32 {
    let toe = balance.unit(kind).toe_personnel.max(1) as f32;
    (persons as f32 / toe).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    #[test]
    fn deficit_scales_with_missing_strength() {
        let b = FrontBalance::default();
        let mut f = Formation::new(0, UnitKind::Infantry, IVec2::ZERO);
        f.strength = 1.0;
        assert_eq!(formation_personnel_deficit(&f, &b), 0);
        f.strength = 0.5;
        let half = formation_personnel_deficit(&f, &b);
        assert!(half >= b.unit(UnitKind::Infantry).toe_personnel / 2);
        f.strength = 0.0;
        assert_eq!(
            formation_personnel_deficit(&f, &b),
            b.unit(UnitKind::Infantry).toe_personnel
        );
    }

    #[test]
    fn credit_take_saturates() {
        let mut book = PersonnelRestockBook::default();
        book.by_side.insert(0, 10);
        assert_eq!(take_personnel_credit(&mut book, 0, 4), 4);
        assert_eq!(book.by_side.get(&0).copied(), Some(6));
        assert_eq!(take_personnel_credit(&mut book, 0, 100), 6);
        assert!(book.by_side.get(&0).is_none());
    }
}
