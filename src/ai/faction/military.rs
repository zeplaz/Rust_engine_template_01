//! **Military brain** — reads the front's [`PressurePoint`]s for one faction and picks which of its
//! formations go where, as plain [`DeploymentKind`] orders for [`DeploymentOrderQueue::enqueue`]
//! (the queue the player's order UI uses). Pure over its inputs; the tick system does the enqueue.
//!
//! * **Threatened** points of ours pull formations in as `Reinforce` (not supply-limited — a defender
//!   falls back on what it has).
//! * **Opportunity** points are `Attack` targets, sized by the formations we can spare and by how
//!   well the front is supplied: no attack without ammunition behind it.
//! * Every candidate point carries the pressure book's signed drivers as score terms, scaled by the
//!   faction's doctrine and current goal, so the log can say why a point won.
//!
//! [`DeploymentKind`]: crate::combat::orders::DeploymentKind
//! [`DeploymentOrderQueue::enqueue`]: crate::combat::orders::DeploymentOrderQueue::enqueue

use bevy::prelude::{Entity, IVec2};

use crate::combat::formation::{Formation, Posture};
use crate::combat::pressure::{PressureKind, PressurePoint};

use super::decide::{FactionDoctrine, ScoreTerm, StrategicGoal};
use super::orders::DeployKind;

/// Sizing and eligibility knobs of the military brain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MilitaryParams {
    /// Deployment orders one decision may issue.
    pub max_orders: usize,
    /// Points scoring below this (after doctrine weighting) are ignored.
    pub min_point_score: f32,
    /// Extra formation per this much point score (`want = 1 + floor(score / force_per_score)`).
    pub force_per_score: f32,
    /// Most formations one point may ask for.
    pub max_want: u32,
    pub min_attack_supply: f32,
    pub min_attack_strength: f32,
    pub min_attack_morale: f32,
    pub min_reinforce_strength: f32,
    /// Below this front fill no attack is ordered at all.
    pub min_front_fill_for_attack: f32,
}

impl Default for MilitaryParams {
    fn default() -> Self {
        Self {
            max_orders: 6,
            min_point_score: 0.05,
            force_per_score: 0.8,
            max_want: 4,
            min_attack_supply: 0.25,
            min_attack_strength: 0.4,
            min_attack_morale: 0.3,
            min_reinforce_strength: 0.2,
            min_front_fill_for_attack: 0.15,
        }
    }
}

/// The parts of a [`Formation`] the brain reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FormationView {
    pub entity: Entity,
    pub strength: f32,
    pub morale: f32,
    pub supply: f32,
    pub posture: Posture,
    pub tile: IVec2,
    pub target: Option<IVec2>,
    pub engaged: bool,
}

impl FormationView {
    #[must_use]
    pub fn of(entity: Entity, f: &Formation) -> Self {
        Self {
            entity,
            strength: f.strength,
            morale: f.morale,
            supply: f.supply,
            posture: f.posture,
            tile: f.tile,
            target: f.target,
            engaged: f.engaged,
        }
    }

    /// Holding still with nothing to do (free to be given a new task).
    fn idle(&self) -> bool {
        self.posture == Posture::Hold && self.target.is_none() && !self.engaged
    }
}

/// One chosen deployment with the reasons behind it.
#[derive(Clone, Debug, PartialEq)]
pub struct DeployDecision {
    pub formation: Entity,
    pub kind: DeployKind,
    pub theater: u32,
    pub tile: IVec2,
    pub subject: String,
    pub total: f32,
    pub terms: Vec<ScoreTerm>,
}

fn point_terms(p: &PressurePoint, goal: StrategicGoal, d: &FactionDoctrine, front_fill: f32) -> Vec<ScoreTerm> {
    let (weight, emphasised) = match p.kind {
        PressureKind::Threatened => (0.7 + 0.6 * d.caution, goal == StrategicGoal::DefendFront),
        PressureKind::Opportunity => (0.4 + 0.8 * d.aggression, goal == StrategicGoal::Advance),
    };
    let w = weight * if emphasised { 1.25 } else { 1.0 };
    let dr = p.drivers;
    let mut terms = vec![
        ScoreTerm { label: "threat", input: dr.threat, weight: w },
        ScoreTerm { label: "artillery", input: dr.artillery, weight: w },
        ScoreTerm { label: "contest", input: dr.contest, weight: w },
        ScoreTerm { label: "supply", input: dr.supply, weight: w },
        ScoreTerm { label: "mobility", input: dr.mobility, weight: w },
        ScoreTerm { label: "fortification", input: dr.fortification, weight: w },
    ];
    if p.kind == PressureKind::Opportunity {
        terms.push(ScoreTerm { label: "front supply readiness", input: front_fill - 0.5, weight: 0.6 });
    }
    if emphasised {
        terms.push(ScoreTerm { label: "goal emphasis", input: 1.0, weight: 0.2 });
    }
    terms
}

fn dist(a: IVec2, b: IVec2) -> f32 {
    (a - b).as_vec2().length()
}

/// Choose deployments for faction `slot`.
///
/// `front_fill` is the mean fill (0..1) of the faction's front supply points; pass the overlay's
/// mean logistics strength when it has no front supply points yet.
#[must_use]
pub fn plan_deployments(
    slot: u8,
    goal: StrategicGoal,
    doctrine: &FactionDoctrine,
    points: &[PressurePoint],
    formations: &[FormationView],
    front_fill: f32,
    params: &MilitaryParams,
) -> Vec<DeployDecision> {
    let front_fill = front_fill.clamp(0.0, 1.0);

    // Score every point of ours.
    let mut cands: Vec<(f32, Vec<ScoreTerm>, &PressurePoint)> = points
        .iter()
        .filter(|p| p.side == slot)
        .map(|p| {
            let terms = point_terms(p, goal, doctrine, front_fill);
            (terms.iter().map(ScoreTerm::value).sum::<f32>(), terms, p)
        })
        .filter(|(total, _, _)| *total >= params.min_point_score)
        .collect();
    cands.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then(a.2.kind.cmp(&b.2.kind))
            .then(a.2.tile.y.cmp(&b.2.tile.y))
            .then(a.2.tile.x.cmp(&b.2.tile.x))
    });

    // Attack budget: how many formations may be committed to attacks given supply and temperament.
    let usable = formations
        .iter()
        .filter(|f| f.posture != Posture::Retreat && f.strength >= params.min_reinforce_strength)
        .count() as f32;
    let attack_cap = if front_fill < params.min_front_fill_for_attack {
        0
    } else {
        (usable * front_fill * (0.3 + 0.7 * doctrine.aggression)).ceil() as u32
    };
    let mut attacking = formations
        .iter()
        .filter(|f| f.posture == Posture::Attack && f.strength >= params.min_reinforce_strength)
        .count() as u32;

    let guarding: Vec<IVec2> = cands
        .iter()
        .filter(|(_, _, p)| p.kind == PressureKind::Threatened)
        .map(|(_, _, p)| p.tile)
        .collect();

    let mut taken: Vec<Entity> = Vec::new();
    let mut out: Vec<DeployDecision> = Vec::new();

    for (total, terms, p) in &cands {
        if out.len() >= params.max_orders {
            break;
        }
        let attack = p.kind == PressureKind::Opportunity;
        let want = (1 + (*total / params.force_per_score.max(1e-3)).floor() as u32).min(params.max_want);
        // Formations already on their way to (or sitting on) this point count toward the force.
        let covered = formations
            .iter()
            .filter(|f| f.target == Some(p.tile) || (f.tile == p.tile && f.posture != Posture::Retreat))
            .count() as u32;
        let mut need = want.saturating_sub(covered);

        while need > 0 && out.len() < params.max_orders {
            if attack && attacking >= attack_cap {
                break;
            }
            // Nearest eligible idle formation; never pull one off a threatened point it is guarding.
            let pick = formations
                .iter()
                .filter(|f| !taken.contains(&f.entity) && f.idle() && !guarding.contains(&f.tile))
                .filter(|f| {
                    if attack {
                        f.supply >= params.min_attack_supply
                            && f.strength >= params.min_attack_strength
                            && f.morale >= params.min_attack_morale
                    } else {
                        f.strength >= params.min_reinforce_strength
                    }
                })
                .min_by(|a, b| {
                    dist(a.tile, p.tile)
                        .total_cmp(&dist(b.tile, p.tile))
                        .then(a.tile.y.cmp(&b.tile.y))
                        .then(a.tile.x.cmp(&b.tile.x))
                        .then(a.entity.to_bits().cmp(&b.entity.to_bits()))
                });
            let Some(f) = pick else { break };
            taken.push(f.entity);
            if attack {
                attacking += 1;
            }
            let mut all = terms.clone();
            all.push(ScoreTerm { label: "distance tiles", input: dist(f.tile, p.tile), weight: -0.02 });
            all.push(ScoreTerm { label: "formation strength", input: f.strength, weight: 0.3 });
            all.push(ScoreTerm { label: "formation supply", input: f.supply, weight: 0.3 });
            out.push(DeployDecision {
                formation: f.entity,
                kind: if attack { DeployKind::Attack } else { DeployKind::Reinforce },
                theater: p.theater.0,
                tile: p.tile,
                subject: format!(
                    "{} ({}, {})",
                    if attack { "attack" } else { "reinforce" },
                    p.tile.x,
                    p.tile.y
                ),
                total: all.iter().map(ScoreTerm::value).sum(),
                terms: all,
            });
            need -= 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::pressure::PressureDrivers;
    use crate::combat::theater::TheaterId;

    fn point(side: u8, kind: PressureKind, x: i32, y: i32, threat: f32) -> PressurePoint {
        let drivers = PressureDrivers { threat, contest: 0.3, ..PressureDrivers::default() };
        PressurePoint {
            theater: TheaterId(1),
            side,
            kind,
            tile: IVec2::new(x, y),
            score: drivers.total(),
            drivers,
        }
    }

    fn fv(n: u32, x: i32, y: i32) -> FormationView {
        FormationView {
            entity: Entity::from_raw_u32(n).expect("raw id"),
            strength: 1.0,
            morale: 1.0,
            supply: 1.0,
            posture: Posture::Hold,
            tile: IVec2::new(x, y),
            target: None,
            engaged: false,
        }
    }

    fn plan(points: &[PressurePoint], fs: &[FormationView], fill: f32) -> Vec<DeployDecision> {
        plan_deployments(
            1,
            StrategicGoal::Hold,
            &FactionDoctrine::default(),
            points,
            fs,
            fill,
            &MilitaryParams::default(),
        )
    }

    #[test]
    fn nearest_idle_formation_reinforces_a_threatened_point() {
        let pts = [point(1, PressureKind::Threatened, 10, 10, 1.0)];
        let fs = [fv(1, 0, 0), fv(2, 9, 10), fv(3, 30, 30)];
        let out = plan(&pts, &fs, 1.0);
        assert!(!out.is_empty());
        assert_eq!(out[0].formation, fs[1].entity);
        assert_eq!(out[0].kind, DeployKind::Reinforce);
        assert_eq!(out[0].tile, IVec2::new(10, 10));
        assert!(out[0].terms.iter().any(|t| t.label == "threat" && t.value() > 0.0));
    }

    #[test]
    fn ignores_other_sides_points() {
        let pts = [point(0, PressureKind::Threatened, 10, 10, 1.0)];
        assert!(plan(&pts, &[fv(1, 0, 0)], 1.0).is_empty());
    }

    #[test]
    fn stronger_points_ask_for_more_formations() {
        let weak = [point(1, PressureKind::Threatened, 10, 10, 0.2)];
        let strong = [point(1, PressureKind::Threatened, 10, 10, 2.0)];
        let fs: Vec<FormationView> = (1..=6).map(|n| fv(n, n as i32, 0)).collect();
        assert!(plan(&strong, &fs, 1.0).len() > plan(&weak, &fs, 1.0).len());
    }

    #[test]
    fn no_attack_without_supply_but_defence_goes_on() {
        let pts = [
            point(1, PressureKind::Opportunity, 20, 20, 1.5),
            point(1, PressureKind::Threatened, 10, 10, 1.0),
        ];
        let fs: Vec<FormationView> = (1..=4).map(|n| fv(n, n as i32, 0)).collect();
        let starved = plan(&pts, &fs, 0.05);
        assert!(starved.iter().all(|d| d.kind == DeployKind::Reinforce), "{starved:?}");
        assert!(!starved.is_empty());
        let fed = plan(&pts, &fs, 1.0);
        assert!(fed.iter().any(|d| d.kind == DeployKind::Attack), "{fed:?}");
    }

    #[test]
    fn attack_is_sized_by_supply_and_aggression() {
        let pts = [point(1, PressureKind::Opportunity, 20, 20, 3.0)];
        let fs: Vec<FormationView> = (1..=8).map(|n| fv(n, n as i32, 0)).collect();
        let half = plan(&pts, &fs, 0.5).len();
        let full = plan(&pts, &fs, 1.0).len();
        assert!(half <= full, "half {half} full {full}");
        assert!(full <= MilitaryParams::default().max_want as usize);
    }

    #[test]
    fn busy_broken_and_weak_formations_are_not_redirected() {
        let pts = [point(1, PressureKind::Threatened, 10, 10, 1.0)];
        let mut moving = fv(1, 9, 10);
        moving.posture = Posture::Move;
        moving.target = Some(IVec2::new(50, 50));
        let mut broken = fv(2, 9, 11);
        broken.posture = Posture::Retreat;
        let mut weak = fv(3, 9, 12);
        weak.strength = 0.05;
        assert!(plan(&pts, &[moving, broken, weak], 1.0).is_empty());
    }

    #[test]
    fn formations_en_route_count_as_covering_the_point() {
        let pts = [point(1, PressureKind::Threatened, 10, 10, 0.3)];
        let mut en_route = fv(1, 5, 5);
        en_route.posture = Posture::Move;
        en_route.target = Some(IVec2::new(10, 10));
        let out = plan(&pts, &[en_route, fv(2, 9, 10)], 1.0);
        assert!(out.is_empty(), "already covered: {out:?}");
    }

    #[test]
    fn plan_is_order_independent() {
        let pts = [
            point(1, PressureKind::Threatened, 10, 10, 1.0),
            point(1, PressureKind::Threatened, 30, 10, 0.9),
        ];
        let fs: Vec<FormationView> = (1..=5).map(|n| fv(n, (n * 7) as i32, (n * 3) as i32)).collect();
        let mut rev = fs.clone();
        rev.reverse();
        assert_eq!(plan(&pts, &fs, 1.0), plan(&pts, &rev, 1.0));
    }
}
