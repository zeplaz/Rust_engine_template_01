//! Blast casualties: formations caught in a detonation lose strength and morale.
//!
//! Construction owns the blast (`BlastCraterBook.recent` is the sim record of every detonation,
//! hit or miss). This module is the combat domain's side: [`queue_blast_casualties_system`] turns
//! each new detonation into a [`BlastStrike`] on the combat-owned [`BlastCasualtyQueue`];
//! `combat_resolve_system` (the sole strength/morale writer) drains the queue into its combatants
//! before the exchange, so a blast casualty goes through the same write-back, frag-map record and
//! destroyed-formation despawn as any other loss. Nothing here touches `Formation` directly.
//!
//! Loss is deterministic: `damage x linear falloff / FULL_DAMAGE`, scaled by how soft the unit
//! kind is and halved at full dig-in. Debt not taken: fortification cover from the fortification
//! index and line-of-effect (terrain shielding) are not modelled; a strike hits a formation by its
//! tile centre only.

use bevy::prelude::*;

use super::balance::{FrontBalance, UnitKind};
use super::resolve::Combatant;
use crate::construction::{blast_radius_falloff, BlastCraterBook};

/// Effective blast damage (after falloff) that takes a fully soft unit's whole strength.
pub const BLAST_CASUALTY_FULL_DAMAGE: f32 = 100.0;
/// Morale lost per unit of falloff at the blast, for a fully soft unit (shock, on top of losses).
const BLAST_MORALE_SHOCK: f32 = 0.25;

/// One detonation as combat sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlastStrike {
    pub centre_m: Vec2,
    pub radius_m: f32,
    /// Peak damage at the centre.
    pub damage: f32,
}

/// Combat-owned request queue of detonations to apply to formations. Producer:
/// [`queue_blast_casualties_system`]; sole consumer: `combat_resolve_system`.
#[derive(Resource, Default, Debug)]
pub struct BlastCasualtyQueue {
    pending: Vec<BlastStrike>,
}

impl BlastCasualtyQueue {
    pub fn push(&mut self, strike: BlastStrike) {
        self.pending.push(strike);
    }

    /// Take every queued strike (consumer side).
    pub fn take(&mut self) -> Vec<BlastStrike> {
        std::mem::take(&mut self.pending)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// How soft a unit kind is to blast (1 = infantry in the open).
#[must_use]
pub const fn blast_softness(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Infantry => 1.0,
        UnitKind::Recon | UnitKind::Engineer => 0.9,
        UnitKind::Mechanized => 0.55,
        UnitKind::Artillery | UnitKind::AirDefense => 0.5,
        UnitKind::Armor => 0.3,
    }
}

/// Narrow producer: new detonations in the crater book (seq cursor) become strikes.
pub fn queue_blast_casualties_system(
    craters: Option<Res<BlastCraterBook>>,
    mut queue: ResMut<BlastCasualtyQueue>,
    mut cursor: Local<u64>,
) {
    let Some(craters) = craters else {
        return;
    };
    let from = *cursor;
    for &(seq, centre_m, radius_m, damage) in craters.recent.iter().filter(|(s, ..)| *s > from) {
        *cursor = (*cursor).max(seq);
        queue.push(BlastStrike { centre_m, radius_m, damage });
    }
}

/// Apply strikes to combatants (pure core, called by `combat_resolve_system`): strength and morale
/// fall with distance from each centre. Returns the total strength removed.
pub fn apply_blast_strikes(units: &mut [Combatant], strikes: &[BlastStrike], b: &FrontBalance) -> f32 {
    let tile_m = b.tile_size_m();
    let mut removed = 0.0;
    for u in units.iter_mut() {
        if u.strength <= 0.0 {
            continue;
        }
        let at_m = (u.tile.as_vec2() + Vec2::splat(0.5)) * tile_m;
        let soft = blast_softness(u.kind) * (1.0 - 0.5 * u.dug_in.clamp(0.0, 1.0));
        for s in strikes {
            let fall = blast_radius_falloff(at_m.distance(s.centre_m), s.radius_m);
            if fall <= 0.0 {
                continue;
            }
            let loss = ((s.damage * fall / BLAST_CASUALTY_FULL_DAMAGE).clamp(0.0, 1.0) * soft).min(u.strength);
            u.strength -= loss;
            u.morale = (u.morale - (loss * 0.5 + BLAST_MORALE_SHOCK * fall * soft)).clamp(0.0, 1.0);
            removed += loss;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::formation::{CombatClock, Formation};
    use crate::combat::resolve::combat_resolve_system;
    use crate::systems::weather::WeatherPollingStations;

    fn combatant(kind: UnitKind, tile: IVec2) -> Combatant {
        Combatant {
            key: 1,
            slot: 0,
            kind,
            tile,
            strength: 1.0,
            morale: 1.0,
            supply: 1.0,
            posture: crate::combat::formation::Posture::Hold,
            dug_in: 0.0,
            visibility_scale: 1.0,
            firepower_mul: 1.0,
            fire_exposure: 0.0,
        }
    }

    #[test]
    fn loss_falls_with_distance_and_armor_is_harder_than_infantry() {
        let b = FrontBalance::default();
        let m = b.tile_size_m();
        let strike = BlastStrike { centre_m: Vec2::new(10.5 * m, 10.5 * m), radius_m: 6.0 * m, damage: 120.0 };
        let mut near = [combatant(UnitKind::Infantry, IVec2::new(10, 10))];
        let mut mid = [combatant(UnitKind::Infantry, IVec2::new(13, 10))];
        let mut far = [combatant(UnitKind::Infantry, IVec2::new(30, 10))];
        let mut armor = [combatant(UnitKind::Armor, IVec2::new(10, 10))];
        for u in [&mut near, &mut mid, &mut far, &mut armor] {
            apply_blast_strikes(u, &[strike], &b);
        }
        assert!(near[0].strength < mid[0].strength, "closer to the centre loses more");
        assert!(mid[0].strength < 1.0);
        assert_eq!(far[0].strength, 1.0, "outside the radius is untouched");
        assert!(armor[0].strength > near[0].strength);
        assert!(near[0].morale < 1.0);
    }

    /// End to end through the owner: a detonation in the crater book lowers the strength of the
    /// formation under it (and only it) via the combat queue and `combat_resolve_system`.
    #[test]
    fn a_formation_in_a_blast_loses_strength() {
        let mut app = App::new();
        let b = FrontBalance::default();
        let m = b.tile_size_m();
        let mut clock = CombatClock::default();
        clock.resolve_acc_s = 10.0;
        let mut book = BlastCraterBook::default();
        book.recent.push((1, Vec2::new(20.5 * m, 20.5 * m), 5.0 * m, 70.0));
        app.insert_resource(b)
            .insert_resource(clock)
            .insert_resource(book)
            .init_resource::<WeatherPollingStations>()
            .init_resource::<BlastCasualtyQueue>()
            .add_systems(Update, (queue_blast_casualties_system, combat_resolve_system).chain());
        let hit = app.world_mut().spawn(Formation::new(0, UnitKind::Infantry, IVec2::new(20, 20))).id();
        let spared = app.world_mut().spawn(Formation::new(0, UnitKind::Infantry, IVec2::new(60, 60))).id();
        app.update();
        let hit_strength = app.world().get::<Formation>(hit).map_or(0.0, |f| f.strength);
        assert!(
            hit_strength > 0.0 && hit_strength < 0.5,
            "formation at the blast centre was hit hard but survives: {hit_strength}"
        );
        assert_eq!(app.world().get::<Formation>(spared).unwrap().strength, 1.0);
        assert!(app.world().resource::<BlastCasualtyQueue>().is_empty(), "queue drained by the owner");
        // The same detonation is not applied twice.
        app.world_mut().resource_mut::<CombatClock>().resolve_acc_s = 10.0;
        app.update();
        let again = app.world().get::<Formation>(hit).map_or(0.0, |f| f.strength);
        assert!(again >= hit_strength - 0.2, "no second application: {again} vs {hit_strength}");
    }
}
