//! **Combat resolution** (PLAN-FRONT-COMBAT-v1 FC-5). Expected-value Lanchester exchange between hostile
//! formations, no RNG: the same inputs always give the same outcome.
//!
//! ```text
//! loss_B = k_attrition · strength_A · firepower_A · eff[A.kind][B.kind] · posture_mult(A)
//!          · supply_factor(A) · morale_factor(A) · (1 − cover_B) · dt
//! ```
//!
//! Each formation fires at the nearest hostile formation inside its range (ties: lowest key). All
//! exchanges in a step use the state at the start of the step. Morale falls with losses (and with
//! missing supply while in contact), recovers out of contact; below `break_threshold` a formation
//! breaks and retreats, and rallies at `rally_threshold`.
//!
//! The pure core is [`resolve_step`]; [`combat_resolve_system`] is the only writer of strength / morale
//! and of the break / rally postures.

use bevy::prelude::*;

use super::balance::{FortificationProfile, FrontBalance, UnitKind};
use super::formation::{CombatClock, Formation, Posture};
use super::fortification::FortificationIndex;
use super::front::{hostility_from_stances, Hostility};
use crate::strategic::settlement::FactionStanceBook;

#[derive(Clone, Debug, PartialEq)]
pub struct Combatant {
    /// Stable ordering key (ties and determinism).
    pub key: u64,
    pub slot: u8,
    pub kind: UnitKind,
    pub tile: IVec2,
    pub strength: f32,
    pub morale: f32,
    pub supply: f32,
    pub posture: Posture,
    /// Field works finished (0..1), from a Fortify order.
    pub dug_in: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Outcome {
    /// Index of the unit this one fired at.
    pub fired_at: Option<usize>,
    /// Tile of the nearest enemy that fired at this unit.
    pub hit_from: Option<IVec2>,
    pub broke: bool,
    pub rallied: bool,
    pub destroyed: bool,
}

impl Outcome {
    #[must_use]
    pub fn engaged(&self) -> bool {
        self.fired_at.is_some() || self.hit_from.is_some()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Fraction of incoming fire the target's tile absorbs from this attacker: the better of the
/// fortification it holds and its own field works (a trench scaled by how far it dug in).
fn cover_against(
    target: &Combatant,
    attacker: UnitKind,
    fort: Option<&FortificationProfile>,
    trench: Option<&FortificationProfile>,
    b: &FrontBalance,
) -> f32 {
    let one = |f: &FortificationProfile, k: f32| (f.cover * k) + if attacker.is_armored() { f.anti_armor * k } else { 0.0 };
    let mut cover = fort.map_or(0.0, |f| one(f, 1.0));
    if target.dug_in > 0.0 {
        cover = cover.max(trench.map_or(0.0, |t| one(t, target.dug_in)));
    }
    if target.posture == Posture::Attack {
        // An attacker is out of its works.
        cover *= b.combat.attacker_cover_scale;
    }
    cover.clamp(0.0, 0.95)
}

/// One exchange of `dt` sim seconds. `fort(tile, slot)` is the fortification profile `slot` holds there.
pub fn resolve_step(
    units: &mut [Combatant],
    hostility: &Hostility,
    b: &FrontBalance,
    fort: &dyn Fn(IVec2, u8) -> Option<FortificationProfile>,
    dt: f32,
) -> Vec<Outcome> {
    let n = units.len();
    let mut out = vec![Outcome::default(); n];
    let mut loss = vec![0.0f32; n];
    let c = &b.combat;
    let mut hit_dist: Vec<f32> = vec![f32::INFINITY; n];
    for i in 0..n {
        let a = &units[i];
        if a.strength <= 0.0 {
            continue;
        }
        let range_m = b.unit(a.kind).range_m;
        let mut best: Option<(usize, f32)> = None;
        for (j, t) in units.iter().enumerate() {
            if j == i || t.strength <= 0.0 || !hostility.hostile(a.slot, t.slot) {
                continue;
            }
            let d = (a.tile - t.tile).as_vec2().length() * b.tile_size_m;
            if d > range_m {
                continue;
            }
            if best.is_none_or(|(bj, bd)| d < bd || (d == bd && t.key < units[bj].key)) {
                best = Some((j, d));
            }
        }
        let Some((j, d)) = best else { continue };
        out[i].fired_at = Some(j);
        let t = &units[j];
        let posture_mult = match a.posture {
            Posture::Attack => c.attack_mult,
            Posture::Retreat => c.retreat_mult,
            Posture::Hold | Posture::Move => c.defend_mult,
        };
        let fort_here = fort(t.tile, t.slot);
        let cover = cover_against(t, a.kind, fort_here.as_ref(), b.fortification(super::balance::FortKind::Trench), b);
        let l = c.k_attrition
            * a.strength
            * b.unit(a.kind).firepower
            * b.effectiveness(a.kind, t.kind)
            * posture_mult
            * lerp(c.min_supply_factor, 1.0, a.supply)
            * lerp(c.min_morale_factor, 1.0, a.morale)
            * (1.0 - cover)
            * dt;
        loss[j] += l;
        if d < hit_dist[j] {
            hit_dist[j] = d;
            out[j].hit_from = Some(a.tile);
        }
    }
    for i in 0..n {
        let u = &mut units[i];
        let taken = loss[i].min(u.strength);
        u.strength -= taken;
        let engaged = out[i].engaged();
        if engaged {
            u.morale -= c.morale_loss_per_strength * taken
                + c.morale_loss_unsupplied_per_s * (1.0 - u.supply.clamp(0.0, 1.0)) * dt;
        } else {
            u.morale += c.morale_recovery_per_s * dt;
        }
        u.morale = u.morale.clamp(0.0, 1.0);
        if u.strength <= c.destroyed_strength {
            out[i].destroyed = true;
        } else if u.posture != Posture::Retreat && u.morale < c.break_threshold {
            u.posture = Posture::Retreat;
            out[i].broke = true;
        } else if u.posture == Posture::Retreat && !engaged && u.morale >= c.rally_threshold {
            u.posture = Posture::Hold;
            out[i].rallied = true;
        }
    }
    out
}

/// Direction a broken formation falls back: away from the enemy that hit it.
fn retreat_target(from: IVec2, threat: Option<IVec2>, slot: u8, tiles: f32) -> IVec2 {
    let d = threat.map_or(IVec2::new(if slot % 2 == 0 { -1 } else { 1 }, 0), |t| from - t);
    let step = if d == IVec2::ZERO {
        IVec2::new(-1, 0)
    } else if d.x.abs() >= d.y.abs() {
        IVec2::new(d.x.signum(), 0)
    } else {
        IVec2::new(0, d.y.signum())
    };
    from + step * tiles.round().max(1.0) as i32
}

pub fn combat_resolve_system(
    mut commands: Commands,
    mut clock: ResMut<CombatClock>,
    balance: Res<FrontBalance>,
    stances: Option<Res<FactionStanceBook>>,
    forts: Option<Res<FortificationIndex>>,
    mut formations: Query<(Entity, &mut Formation)>,
) {
    if clock.resolve_acc_s < balance.combat.step_s {
        return;
    }
    let dt = std::mem::take(&mut clock.resolve_acc_s);
    let mut order: Vec<Entity> = formations.iter().map(|(e, _)| e).collect();
    order.sort();
    let mut units: Vec<Combatant> = order
        .iter()
        .filter_map(|e| formations.get(*e).ok().map(|(e, f)| (e, f)))
        .map(|(e, f)| Combatant {
            key: e.to_bits(),
            slot: f.faction_slot,
            kind: f.kind,
            tile: f.tile,
            strength: f.strength,
            morale: f.morale,
            supply: f.supply,
            posture: f.posture,
            dug_in: f.dug_in,
        })
        .collect();
    let hostility = hostility_from_stances(stances.as_deref());
    let fort = |tile: IVec2, slot: u8| {
        let (owner, kind) = forts.as_ref()?.get(tile)?;
        (owner == slot).then(|| balance.fortification(kind).cloned()).flatten()
    };
    let outcomes = resolve_step(&mut units, &hostility, &balance, &fort, dt);
    for ((e, u), o) in order.iter().zip(&units).zip(&outcomes) {
        if o.destroyed {
            if let Ok(mut ec) = commands.get_entity(*e) {
                ec.despawn();
            }
            continue;
        }
        let Ok((_, mut f)) = formations.get_mut(*e) else { continue };
        f.strength = u.strength;
        f.morale = u.morale;
        f.engaged = o.engaged();
        if u.posture != f.posture {
            f.posture = u.posture;
            if o.broke {
                let threat = o.hit_from.or_else(|| o.fired_at.map(|j| units[j].tile));
                f.target = Some(retreat_target(f.tile, threat, f.faction_slot, balance.combat.retreat_tiles));
                f.move_progress = 0.0;
            } else if o.rallied {
                f.target = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(key: u64, slot: u8, kind: UnitKind, x: i32) -> Combatant {
        Combatant {
            key,
            slot,
            kind,
            tile: IVec2::new(x, 0),
            strength: 1.0,
            morale: 1.0,
            supply: 1.0,
            posture: Posture::Hold,
            dug_in: 0.0,
        }
    }

    fn no_fort(_: IVec2, _: u8) -> Option<FortificationProfile> {
        None
    }

    fn run(units: &mut Vec<Combatant>, secs: u32) {
        let b = FrontBalance::default();
        for _ in 0..secs {
            resolve_step(units, &Hostility::legacy_pair(), &b, &no_fort, 1.0);
        }
    }

    #[test]
    fn symmetric_fight_is_symmetric_and_deterministic() {
        let mut a = vec![unit(1, 0, UnitKind::Infantry, 0), unit(2, 1, UnitKind::Infantry, 4)];
        let mut b = a.clone();
        run(&mut a, 120);
        run(&mut b, 120);
        assert_eq!(a, b);
        assert!((a[0].strength - a[1].strength).abs() < 1e-5);
        assert!(a[0].strength < 1.0);
    }

    #[test]
    fn three_to_one_with_supply_destroys_the_defender() {
        let mut u = vec![
            unit(1, 0, UnitKind::Infantry, 0),
            unit(2, 0, UnitKind::Infantry, 0),
            unit(3, 0, UnitKind::Infantry, 1),
            unit(4, 1, UnitKind::Infantry, 5),
        ];
        let b = FrontBalance::default();
        let mut broke = false;
        for _ in 0..600 {
            let o = resolve_step(&mut u, &Hostility::legacy_pair(), &b, &no_fort, 1.0);
            broke |= o[3].broke || o[3].destroyed;
        }
        assert!(broke, "defender breaks or dies");
        assert!(u[3].strength < 0.5, "defender is ground down");
        assert!(u.iter().filter(|x| x.slot == 0).all(|x| x.strength > 0.6), "attackers stay strong");
    }

    #[test]
    fn starved_formation_breaks_first() {
        let mut u = vec![unit(1, 0, UnitKind::Infantry, 0), unit(2, 1, UnitKind::Infantry, 4)];
        u[1].supply = 0.0;
        let b = FrontBalance::default();
        let mut who = None;
        for _ in 0..1500 {
            let o = resolve_step(&mut u, &Hostility::legacy_pair(), &b, &no_fort, 1.0);
            if let Some(i) = o.iter().position(|x| x.broke) {
                who = Some(i);
                break;
            }
        }
        assert_eq!(who, Some(1));
        assert_eq!(u[1].posture, Posture::Retreat);
        assert_eq!(u[0].posture, Posture::Hold);
    }

    #[test]
    fn armor_wins_open_ground_but_bunkered_infantry_wins() {
        let b = FrontBalance::default();
        let bunker = b.fortification(crate::combat::FortKind::Bunker).cloned().unwrap();
        let fight = |fortified: bool| {
            let mut u = vec![unit(1, 0, UnitKind::Armor, 0), unit(2, 1, UnitKind::Infantry, 5)];
            u[0].posture = Posture::Attack;
            let f = |_: IVec2, s: u8| (fortified && s == 1).then(|| bunker.clone());
            for _ in 0..300 {
                resolve_step(&mut u, &Hostility::legacy_pair(), &b, &f, 1.0);
            }
            (u[0].strength, u[1].strength)
        };
        let (a_open, i_open) = fight(false);
        let (a_fort, i_fort) = fight(true);
        assert!(a_open > i_open, "armor wins in the open");
        assert!(i_fort > a_fort, "infantry in a bunker beats the same armor");
    }

    #[test]
    fn dug_in_defender_takes_less_fire() {
        let b = FrontBalance::default();
        let take = |dug: f32| {
            let mut u = vec![unit(1, 0, UnitKind::Infantry, 0), unit(2, 1, UnitKind::Infantry, 5)];
            u[0].posture = Posture::Attack;
            u[1].dug_in = dug;
            for _ in 0..60 {
                resolve_step(&mut u, &Hostility::legacy_pair(), &b, &no_fort, 1.0);
            }
            1.0 - u[1].strength
        };
        assert!(take(1.0) < take(0.0) * 0.7);
    }
}
