//! **Economy brain** — decides which building a faction needs next and where to put it. The tick
//! system sends the result through the construction commit funnel (`CommitConstructionSiteEvent`,
//! validated by `evaluate_site_placement_at_world_tile`) exactly like a player placement.
//!
//! What it wants, in order of how loudly the world asks for it:
//! * **Input shortage** — one of our consumers (a plant, a construction site) is short of a cargo
//!   and nothing we own or are building produces it: build a catalog producer of that cargo. Each
//!   producer in turn publishes its own input demands, so a short chain grows one link per decision.
//! * **Bootstrap** — a faction with no productive building starts with the cheapest producer of a
//!   construction staple.
//!
//! Pure over a [`LogisticsOrdersSnapshot`] and a catalog slice; no randomness except the seeded
//! placement jitter in [`choose_origin`].

use bevy::prelude::IVec2;

use crate::construction::BuildingDefinitionRegistry;
use crate::strategic::SiteArchetype;
use crate::economy::logistics::orders::construction_demand::CONSTRUCTION_MATERIALS;
use crate::economy::logistics::orders::plant_demand::is_cargo_tag;
use crate::economy::logistics::orders::{
    best_source_for, DemandHealth, LogisticsOrdersSnapshot, OwnerId, SiteRole,
};

use super::decide::{FactionDoctrine, ScoreTerm, StrategicGoal};
use super::orders::BuildReason;
use super::rng::DetRng;

/// Score a want needs before the faction commits to building it.
pub const BUILD_MIN_SCORE: f32 = 0.15;
/// Search radius (tiles) around the anchor for a placement.
pub const PLACEMENT_RADIUS_TILES: i32 = 10;
/// Construction sites a faction may have open at once.
pub const DEFAULT_MAX_ACTIVE_SITES: u32 = 2;
/// Sim ticks between two placements of one faction.
pub const DEFAULT_BUILD_COOLDOWN_TICKS: u64 = 120;

/// The slice of a catalog row the brain scores.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogRow {
    pub id: String,
    pub produces: Vec<String>,
    pub consumes: Vec<String>,
    pub cost: u32,
    pub footprint: (u32, u32),
    pub archetype: SiteArchetype,
}

impl CatalogRow {
    fn produces_cargo(&self, cargo: &str) -> bool {
        self.produces.iter().any(|p| p.eq_ignore_ascii_case(cargo))
    }

    fn hauled_inputs(&self) -> impl Iterator<Item = &String> {
        self.consumes.iter().filter(|c| is_cargo_tag(c))
    }
}

/// Productive catalog rows with a hauled output, sorted by id (the registry is a `HashMap`).
#[must_use]
pub fn catalog_rows(reg: &BuildingDefinitionRegistry) -> Vec<CatalogRow> {
    let mut rows: Vec<CatalogRow> = reg
        .by_id
        .values()
        .filter(|d| d.is_productive && !d.id.starts_with("builtin:"))
        .filter(|d| d.produces.iter().any(|p| is_cargo_tag(p)))
        .map(|d| CatalogRow {
            id: d.id.clone(),
            produces: d.produces.clone(),
            consumes: d.consumes.clone(),
            cost: d.construction_cost,
            footprint: (d.footprint.width.max(1), d.footprint.depth.max(1)),
            archetype: d.site_archetype,
        })
        .collect();
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    rows
}

/// A building the faction already has (or is building).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedBuilding {
    pub catalog_id: String,
    pub operational: bool,
}

/// What the economy brain reads.
pub struct EconomyView<'a> {
    pub owner: OwnerId,
    pub snapshot: &'a LogisticsOrdersSnapshot,
    pub catalog: &'a [CatalogRow],
    pub owned: &'a [OwnedBuilding],
}

/// A cargo one of our consumers is short of with no producer of ours behind it.
#[derive(Clone, Debug, PartialEq)]
pub struct Shortage {
    pub cargo: String,
    /// Mean demand severity (Starving 1.0, Low 0.5).
    pub severity: f32,
    pub waiting: u32,
    pub construction: u32,
}

/// One building the faction would like, with the terms that rank it.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildWant {
    pub catalog_id: String,
    pub footprint: (u32, u32),
    pub archetype: SiteArchetype,
    pub reason: BuildReason,
    pub subject: String,
    pub total: f32,
    pub terms: Vec<ScoreTerm>,
}

impl EconomyView<'_> {
    fn catalog_row(&self, id: &str) -> Option<&CatalogRow> {
        self.catalog.iter().find(|r| r.id == id)
    }

    fn owns(&self, id: &str) -> bool {
        self.owned.iter().any(|o| o.catalog_id == id)
    }

    /// Do we own or build anything that outputs `cargo`?
    fn produces(&self, cargo: &str) -> bool {
        self.owned
            .iter()
            .filter_map(|o| self.catalog_row(&o.catalog_id))
            .any(|r| r.produces_cargo(cargo))
    }

    /// Fraction of `row`'s hauled inputs nobody of ours produces or holds.
    fn inputs_unmet(&self, row: &CatalogRow) -> f32 {
        let inputs: Vec<&String> = row.hauled_inputs().collect();
        if inputs.is_empty() {
            return 0.0;
        }
        let held = |cargo: &str| {
            self.snapshot
                .sites
                .iter()
                .filter(|s| s.owner == self.owner)
                .any(|s| s.stock.iter().any(|r| r.cargo.eq_ignore_ascii_case(cargo) && r.on_hand > 0))
        };
        let unmet = inputs.iter().filter(|c| !self.produces(c) && !held(c)).count();
        unmet as f32 / inputs.len() as f32
    }
}

/// Cargo our consumers are short of that no source of ours can serve and nothing of ours makes.
#[must_use]
pub fn shortages(view: &EconomyView<'_>) -> Vec<Shortage> {
    let mut out: Vec<Shortage> = Vec::new();
    for d in view.snapshot.demands.iter().filter(|d| d.owner == view.owner) {
        if d.health == DemandHealth::Satisfied || !is_cargo_tag(&d.cargo) {
            continue;
        }
        if best_source_for(view.snapshot, d).is_some() || view.produces(&d.cargo) {
            continue;
        }
        let sev = if d.health == DemandHealth::Starving { 1.0 } else { 0.5 };
        let is_construction = u32::from(d.consumer == crate::economy::logistics::orders::ConsumerKind::Construction);
        match out.iter_mut().find(|s| s.cargo == d.cargo) {
            Some(s) => {
                s.severity = (s.severity * s.waiting as f32 + sev) / (s.waiting + 1) as f32;
                s.waiting += 1;
                s.construction += is_construction;
            }
            None => out.push(Shortage { cargo: d.cargo.clone(), severity: sev, waiting: 1, construction: is_construction }),
        }
    }
    out.sort_by(|a, b| a.cargo.cmp(&b.cargo));
    out
}

fn goal_term(goal: StrategicGoal) -> ScoreTerm {
    let input = match goal {
        StrategicGoal::Develop => 1.0,
        StrategicGoal::SecureSupply => 0.3,
        StrategicGoal::Hold | StrategicGoal::Advance => 0.0,
        StrategicGoal::DefendFront => -1.0,
    };
    ScoreTerm { label: "goal emphasis", input, weight: 0.3 }
}

fn common_terms(view: &EconomyView<'_>, row: &CatalogRow, doctrine: &FactionDoctrine, goal: StrategicGoal) -> Vec<ScoreTerm> {
    vec![
        ScoreTerm { label: "inputs unmet", input: -view.inputs_unmet(row), weight: 0.5 },
        ScoreTerm { label: "build cost", input: -(row.cost as f32 / 5000.0), weight: 0.5 },
        ScoreTerm { label: "development focus", input: 1.0, weight: 0.3 * doctrine.development_focus },
        goal_term(goal),
    ]
}

/// Every building worth building, best first (ties by catalog id).
#[must_use]
pub fn rank_builds(view: &EconomyView<'_>, doctrine: &FactionDoctrine, goal: StrategicGoal) -> Vec<BuildWant> {
    let mut wants: Vec<BuildWant> = Vec::new();

    for s in shortages(view) {
        for row in view.catalog.iter().filter(|r| r.produces_cargo(&s.cargo) && !view.owns(&r.id)) {
            let mut terms = vec![
                ScoreTerm { label: "shortage severity", input: s.severity, weight: 1.0 },
                ScoreTerm { label: "consumers waiting", input: s.waiting as f32, weight: 0.15 },
                ScoreTerm { label: "construction bill", input: f32::from(s.construction > 0), weight: 0.3 },
            ];
            terms.extend(common_terms(view, row, doctrine, goal));
            wants.push(BuildWant {
                catalog_id: row.id.clone(),
                footprint: row.footprint,
                archetype: row.archetype,
                reason: BuildReason::InputShort { cargo: s.cargo.clone() },
                subject: format!("build {} for {}", row.id, s.cargo),
                total: terms.iter().map(ScoreTerm::value).sum(),
                terms,
            });
        }
    }

    let has_productive = view.owned.iter().any(|o| view.catalog_row(&o.catalog_id).is_some());
    if !has_productive && wants.is_empty() {
        for row in view.catalog {
            let staple = row
                .produces
                .iter()
                .any(|p| CONSTRUCTION_MATERIALS.iter().any(|m| m.eq_ignore_ascii_case(p)));
            let mut terms = vec![
                ScoreTerm { label: "staple output", input: f32::from(staple), weight: 0.5 },
                ScoreTerm { label: "bootstrap", input: 1.0, weight: 0.3 },
            ];
            terms.extend(common_terms(view, row, doctrine, goal));
            wants.push(BuildWant {
                catalog_id: row.id.clone(),
                footprint: row.footprint,
                archetype: row.archetype,
                reason: BuildReason::Bootstrap,
                subject: format!("build {} (bootstrap)", row.id),
                total: terms.iter().map(ScoreTerm::value).sum(),
                terms,
            });
        }
    }

    wants.sort_by(|a, b| b.total.total_cmp(&a.total).then_with(|| a.catalog_id.cmp(&b.catalog_id)));
    wants
}

/// Depots of `owner` in the snapshot (first by id), as a placement anchor tile.
#[must_use]
pub fn depot_anchor(snap: &LogisticsOrdersSnapshot, owner: OwnerId) -> Option<IVec2> {
    snap.sites_of(owner, SiteRole::Depot)
        .min_by_key(|s| s.id)
        .map(|s| IVec2::new(s.tile[0], s.tile[1]))
}

/// Best origin within [`PLACEMENT_RADIUS_TILES`] of `anchor`. `score` returns `None` for an invalid
/// origin. Candidates are visited in a fixed ring order; ties are broken by a tiny seeded jitter so
/// two factions with different seeds do not stack on the same tile, and the same seed always picks
/// the same one.
pub fn choose_origin(
    anchor: IVec2,
    rng: &mut DetRng,
    mut score: impl FnMut(IVec2) -> Option<f32>,
) -> Option<(IVec2, f32)> {
    let mut best: Option<(IVec2, f32)> = None;
    for r in 0..=PLACEMENT_RADIUS_TILES {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                let origin = anchor + IVec2::new(dx, dy);
                if origin.x < 0 || origin.y < 0 {
                    continue;
                }
                let jitter = rng.next_f32() * 1e-3;
                let Some(base) = score(origin) else { continue };
                let s = base - 0.02 * (r as f32) + jitter;
                if best.map_or(true, |(_, b)| s > b) {
                    best = Some((origin, s));
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::{
        ConsumerKind, DemandRow, Priority, SiteId, SiteRow, StockRow,
    };

    fn row(id: &str, produces: &[&str], consumes: &[&str], cost: u32) -> CatalogRow {
        CatalogRow {
            id: id.into(),
            produces: produces.iter().map(|s| (*s).into()).collect(),
            consumes: consumes.iter().map(|s| (*s).into()).collect(),
            cost,
            footprint: (3, 2),
            archetype: SiteArchetype::Factory,
        }
    }

    fn catalog() -> Vec<CatalogRow> {
        vec![
            row("kiln", &["Cement"], &["Limestone", "Labour"], 2200),
            row("mixer", &["Concrete"], &["Cement", "Water", "Labour"], 900),
            row("mine", &["Limestone", "Gravel"], &["Diesel", "Labour"], 600),
        ]
    }

    fn site(id: u32, owner: u8, role: SiteRole, stock: &[(&str, u32)]) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("s{id}"),
            owner: OwnerId(owner),
            role,
            world_xz: [0.0, 0.0],
            tile: [5, 6],
            stock: stock
                .iter()
                .map(|(c, n)| StockRow { cargo: (*c).into(), unit: "t", on_hand: *n, reserved_out: 0, inbound: 0 })
                .collect(),
        }
    }

    fn demand(site: u32, cargo: &str, consumer: ConsumerKind, health: DemandHealth) -> DemandRow {
        DemandRow {
            site: SiteId(site),
            site_name: String::new(),
            owner: OwnerId(1),
            cargo: cargo.into(),
            unit: "t",
            consumer,
            on_hand: 0,
            inbound: 0,
            buffer_target: 50,
            reorder_point: 12,
            draw_rate_per_min: 0,
            priority: Priority::Normal,
            health,
            served_by: 0,
            starved_ticks: 0,
            shortfall_total: 0,
            time_to_empty_ticks: None,
            round_trip_ticks: None,
            trucks_needed: None,
        }
    }

    fn rank(snap: &LogisticsOrdersSnapshot, owned: &[OwnedBuilding]) -> Vec<BuildWant> {
        let cat = catalog();
        let view = EconomyView { owner: OwnerId(1), snapshot: snap, catalog: &cat, owned, };
        rank_builds(&view, &FactionDoctrine::default(), StrategicGoal::Develop)
    }

    #[test]
    fn empty_faction_bootstraps_a_construction_staple() {
        let snap = LogisticsOrdersSnapshot { sites: vec![site(1, 1, SiteRole::Depot, &[])], ..Default::default() };
        let wants = rank(&snap, &[]);
        assert_eq!(wants[0].reason, BuildReason::Bootstrap);
        assert_eq!(wants[0].catalog_id, "mixer", "Concrete is a construction staple: {wants:?}");
        assert!(wants[0].terms.iter().any(|t| t.label == "staple output" && t.value() > 0.0));
    }

    #[test]
    fn short_input_with_no_producer_asks_for_one() {
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(1, 1, SiteRole::Depot, &[]), site(2, 1, SiteRole::Plant, &[])],
            demands: vec![demand(2, "Cement", ConsumerKind::Plant, DemandHealth::Starving)],
            ..Default::default()
        };
        let owned = [OwnedBuilding { catalog_id: "mixer".into(), operational: true }];
        let wants = rank(&snap, &owned);
        assert_eq!(wants[0].catalog_id, "kiln");
        assert_eq!(wants[0].reason, BuildReason::InputShort { cargo: "Cement".into() });
        assert!(wants.iter().all(|w| w.reason != BuildReason::Bootstrap));
    }

    #[test]
    fn chain_grows_one_link_at_a_time() {
        // Kiln is being built: its Limestone demand is now the open gap, the mine the answer.
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(2, 1, SiteRole::Plant, &[]), site(3, 1, SiteRole::Plant, &[])],
            demands: vec![
                demand(2, "Cement", ConsumerKind::Plant, DemandHealth::Starving),
                demand(3, "Limestone", ConsumerKind::Plant, DemandHealth::Low),
            ],
            ..Default::default()
        };
        let owned = [
            OwnedBuilding { catalog_id: "mixer".into(), operational: true },
            OwnedBuilding { catalog_id: "kiln".into(), operational: false },
        ];
        let wants = rank(&snap, &owned);
        assert_eq!(wants.len(), 1, "Cement is covered by the kiln in progress: {wants:?}");
        assert_eq!(wants[0].catalog_id, "mine");
    }

    #[test]
    fn stocked_source_means_no_shortage() {
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(1, 1, SiteRole::Depot, &[("Cement", 80)]), site(2, 1, SiteRole::Plant, &[])],
            demands: vec![demand(2, "Cement", ConsumerKind::Plant, DemandHealth::Low)],
            ..Default::default()
        };
        let owned = [OwnedBuilding { catalog_id: "mixer".into(), operational: true }];
        assert!(rank(&snap, &owned).is_empty());
    }

    #[test]
    fn other_factions_demands_and_labour_are_ignored() {
        let mut d = demand(2, "Cement", ConsumerKind::Plant, DemandHealth::Starving);
        d.owner = OwnerId(2);
        let labour = demand(2, "Labour", ConsumerKind::Plant, DemandHealth::Starving);
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(2, 1, SiteRole::Plant, &[])],
            demands: vec![d, labour],
            ..Default::default()
        };
        let owned = [OwnedBuilding { catalog_id: "mixer".into(), operational: true }];
        assert!(rank(&snap, &owned).is_empty());
    }

    #[test]
    fn construction_bill_and_severity_raise_the_score() {
        let mk = |consumer, health| LogisticsOrdersSnapshot {
            sites: vec![site(2, 1, SiteRole::Plant, &[])],
            demands: vec![demand(2, "Cement", consumer, health)],
            ..Default::default()
        };
        let owned = [OwnedBuilding { catalog_id: "mixer".into(), operational: true }];
        let low = rank(&mk(ConsumerKind::Plant, DemandHealth::Low), &owned)[0].total;
        let starving = rank(&mk(ConsumerKind::Plant, DemandHealth::Starving), &owned)[0].total;
        let bill = rank(&mk(ConsumerKind::Construction, DemandHealth::Starving), &owned)[0].total;
        assert!(low < starving && starving < bill);
    }

    #[test]
    fn placement_is_seeded_and_prefers_near_the_anchor() {
        let pick = |seed| {
            let mut rng = DetRng::new(seed);
            choose_origin(IVec2::new(10, 10), &mut rng, |o| Some(if o == IVec2::new(12, 10) { 1.0 } else { 0.0 }))
        };
        assert_eq!(pick(1), pick(1));
        assert_eq!(pick(1).map(|p| p.0), Some(IVec2::new(12, 10)));
        let mut rng = DetRng::new(1);
        assert_eq!(choose_origin(IVec2::new(10, 10), &mut rng, |_| None), None);
    }

    #[test]
    fn placement_never_leaves_the_map() {
        let mut rng = DetRng::new(3);
        let (o, _) = choose_origin(IVec2::ZERO, &mut rng, |o| Some(-(o.x + o.y) as f32)).expect("a tile");
        assert!(o.x >= 0 && o.y >= 0);
    }

    #[test]
    fn depot_anchor_is_the_lowest_own_depot() {
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(4, 1, SiteRole::Depot, &[]), site(2, 1, SiteRole::Depot, &[]), site(1, 2, SiteRole::Depot, &[])],
            ..Default::default()
        };
        assert_eq!(depot_anchor(&snap, OwnerId(1)), Some(IVec2::new(5, 6)));
        assert_eq!(depot_anchor(&snap, OwnerId(3)), None);
    }
}
