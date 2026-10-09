//! City faction progress — lib launch beside [`super::product_fire_vfx_live_proof`].
//!
//! That VFX proof is a structured body that stays pending or red until the draw
//! counters are real, plus `debug_runs/product_fire_vfx_live.json`. This launch
//! steps a seeded settlement for [`CITY_FACTION_PROGRESS_TICKS`] and returns an
//! issue list that is empty only when power, haul, growth, and both factions hold.
//!
//! ```text
//! cargo test -p proc_A_dine01 --lib city_faction_progress -- --test-threads=8
//! cargo run -p proc_A_dine01 --release -- --test city
//! ```
//!
//! `--test city` opens the same stay-open window as `--test vfx`. Settlement startup
//! seeds an empty town book. The window stamps this JSON once from
//! [`city_faction_progress`] on the live book. `green` follows an empty issue list.

use bevy::prelude::*;

use crate::construction::{cut_power_line_segment, PendingBuildBlueprint, PendingConstructionQueue, PowerLineDamageBook};
use crate::infrastructure::utility::{
    fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot, plant_reaches_consumer,
    UtilityGraph, UtilityGraphEdge, UtilityGraphNode,
};
use crate::strategic::settlement::{
    enqueue_town_growth_once, generated_town_book, pending_blueprint_from_growth_proposal,
    seed_settlement_books_if_empty, ArchetypeId, BuildingUsage, DistrictBook, DistrictId,
    GrowthActorLayer, GrowthProposal, TownBook,
};
use crate::strategic::{
    apply_shot_impacts_to_power_lines_system, book_impact_for_faction, committed_power_link_at_cell,
    disruption_after_cell_heat, edge_flow_for_overlay, logistics_net_inject_into_overlays,
    record_shot, shot_impact_at, ChunkStrategicOverlay, LogisticsEdge, LogisticsGraph,
    LogisticsNode, LogisticsNodeId, ShotPulse, ShotPulseBook, StrategicOverlayCouplingScratch,
    SHOT_IMPACT_SIDE_FACTION,
};
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::systems::fire::ChunkFireOverlay;
use crate::systems::transport::TransportEdgeId;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::ChunkCellKey;

pub const CITY_FACTION_PROGRESS_LIVE_JSON: &str = "debug_runs/city_faction_progress_live.json";

/// Same bound as the shot book cap in `shot_pulse` (32 retained shots).
pub const CITY_FACTION_PROGRESS_TICKS: u64 = 32;

const LAUNCH: &str = "city_faction_progress";
const SIDE_LINE: u64 = 99;
const HAUL_CAPACITY: f32 = 10.0;

#[derive(Clone, Debug)]
pub struct CityFactionFixture {
    pub seed: u64,
    /// Kept when non-empty. [`seed_settlement_books_if_empty`] does not replace it.
    pub saved_towns: TownBook,
    pub cut_both_feeders: bool,
    pub edge_disruption: f32,
    pub edge_heat: f32,
    /// When set, this row is the faction growth candidate instead of town sites.
    pub growth_label: Option<String>,
    pub faction_two_hits_side_line: bool,
}

impl CityFactionFixture {
    #[must_use]
    pub fn healthy(seed: u64) -> Self {
        Self {
            seed,
            saved_towns: TownBook::default(),
            cut_both_feeders: false,
            edge_disruption: 0.0,
            edge_heat: 0.0,
            growth_label: None,
            faction_two_hits_side_line: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CityFactionIssue {
    /// Faction id or town id.
    pub id: String,
    pub system: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CityFactionProgressReport {
    pub launch: &'static str,
    pub seed: u64,
    pub ticks: u64,
    pub issues: Vec<CityFactionIssue>,
    pub faction_progress: Vec<String>,
    pub player_rows: Vec<String>,
    pub town_ids: Vec<String>,
    pub factions: Vec<u8>,
    pub haul_moved: bool,
    pub side_lines_damaged: bool,
    pub side_shot_on_committed_line: bool,
}

#[derive(Resource)]
struct CityFactionStep {
    tick: u64,
}

#[must_use]
pub fn city_faction_progress(fixture: &CityFactionFixture) -> CityFactionProgressReport {
    let (towns, districts) = settle_books(fixture);
    let mut town_ids: Vec<String> = towns.towns.keys().map(|id| id.0.clone()).collect();
    town_ids.sort();
    let subject = town_ids.first().cloned().unwrap_or_else(|| "town".into());

    let utility = grid_with_side_line();
    let mut damage = PowerLineDamageBook::default();
    if fixture.cut_both_feeders {
        cut_power_line_segment(&mut damage, 10);
        cut_power_line_segment(&mut damage, 11);
    }

    let shot_cell = IVec2::new(1, 0);
    let mut shots = ShotPulseBook::default();
    record_shot(
        &mut shots,
        ShotPulse {
            origin: IVec2::new(80, 80),
            target: IVec2::new(80, 80),
            fired_tick: 0,
            faction: 1,
        },
    );
    let faction_two_target = if fixture.faction_two_hits_side_line {
        shot_cell
    } else {
        IVec2::new(90, 90)
    };
    record_shot(
        &mut shots,
        ShotPulse {
            origin: faction_two_target,
            target: faction_two_target,
            fired_tick: 0,
            faction: 2,
        },
    );
    let mut factions: Vec<u8> = shots.recent.iter().map(|shot| shot.faction).collect();
    factions.sort_unstable();
    factions.dedup();

    let side_shot_on_committed_line = fixture.faction_two_hits_side_line
        && committed_power_link_at_cell(&utility, shot_cell) == Some(SIDE_LINE)
        && book_impact_for_faction(&shots, 0, shot_cell, SHOT_IMPACT_SIDE_FACTION) == 0.0
        && shot_impact_at(shots.recent.iter().find(|s| s.faction == 2).unwrap(), 0, shot_cell)
            > 0.0;

    let mut pending = PendingConstructionQueue::default();
    let mut refused = false;
    if let Some(label) = &fixture.growth_label {
        if let Some(row) = labeled_row(label, IVec2::new(8, 8)) {
            pending.push(row);
        } else {
            refused = true;
        }
    } else {
        let mut placed = false;
        enqueue_town_growth_once(&towns, &districts, &mut pending, &mut placed);
    }
    if let Some(player) = labeled_row("player:warehouse", IVec2::new(12, 8)) {
        pending.push(player);
    }
    let offered: Vec<String> = pending.entries.iter().map(|entry| entry.label.clone()).collect();

    let edge = haul_edge(fixture.edge_disruption);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(utility.clone())
        .insert_resource(damage)
        .insert_resource(shots)
        .insert_resource(haul_graph(&edge))
        .insert_resource(StrategicOverlayCouplingScratch::default())
        .insert_resource(pending)
        .insert_resource(CityFactionStep { tick: 0 })
        .insert_resource(SimTick(0))
        .insert_resource(SimControlState {
            paused: false,
            steps_remaining: 0,
            speed: 1.0,
        })
        .add_message::<crate::strategic::CommitConstructionSiteEvent>()
        .add_systems(
            Update,
            (
                publish_city_tick,
                apply_shot_impacts_to_power_lines_system,
                logistics_net_inject_into_overlays,
                crate::strategic::settlement::commit_growth_pending_to_funnel_system,
            )
                .chain(),
        );
    app.world_mut().spawn((
        Chunk { coord: IVec2::ZERO },
        ChunkCellMatrix::new(UVec2::new(2, 1)),
        ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 1)),
        fire_overlay(fixture.edge_heat),
    ));

    for _ in 0..CITY_FACTION_PROGRESS_TICKS {
        app.update();
    }

    let step = app.world().resource::<CityFactionStep>().tick;
    let damage = app.world().resource::<PowerLineDamageBook>().clone();
    let kept = app
        .world()
        .resource::<PendingConstructionQueue>()
        .entries
        .iter()
        .map(|entry| entry.label.clone())
        .collect::<Vec<_>>();
    let mut overlays = app.world_mut().query::<&ChunkStrategicOverlay>();
    let haul_moved = overlays
        .iter(app.world())
        .flat_map(|overlay| overlay.logistics_throughput.iter().copied())
        .any(|value| value > 0.0);

    let side_lines_damaged = damage
        .segments
        .get(&SIDE_LINE)
        .is_some_and(|segment| segment.hp < segment.max_hp || segment.circuit_open());

    let mut judged = edge;
    judged.disruption = disruption_after_cell_heat(fixture.edge_disruption, fixture.edge_heat);
    let flow = edge_flow_for_overlay(&judged, None);
    let edge_closed = flow <= 0.0 && fixture.edge_disruption.max(fixture.edge_heat) > 0.0;

    let feeds = plant_reaches_consumer(&utility, &damage.destroyed_link_ids());

    let mut faction_progress: Vec<String> = offered
        .iter()
        .filter(|label| label.starts_with("growth:") && !kept.iter().any(|row| row == *label))
        .cloned()
        .collect();
    faction_progress.sort();
    let player_rows: Vec<String> = kept
        .iter()
        .filter(|label| label.starts_with("player:"))
        .cloned()
        .collect();

    let mut issues = Vec::new();
    if !feeds {
        issues.push(CityFactionIssue {
            id: subject.clone(),
            system: "power".into(),
            reason: "plant feeders open; consumer not fed".into(),
        });
    }
    if edge_closed {
        issues.push(CityFactionIssue {
            id: subject.clone(),
            system: "logistics".into(),
            reason: "haul stopped; edge closed".into(),
        });
    }
    if refused {
        issues.push(CityFactionIssue {
            id: subject.clone(),
            system: "growth".into(),
            reason: "refused label was not enqueued".into(),
        });
    }
    if let Some(label) = &fixture.growth_label {
        let counted = faction_progress.iter().any(|row| row == label);
        if !label.starts_with("growth:") && !counted {
            issues.push(CityFactionIssue {
                id: subject.clone(),
                system: "growth".into(),
                reason: format!("row {label} is not prefixed growth:"),
            });
        }
    }
    if !factions.contains(&1) || !factions.contains(&2) {
        let missing = if !factions.contains(&1) { "1" } else { "2" };
        issues.push(CityFactionIssue {
            id: missing.into(),
            system: "conflict".into(),
            reason: "faction missing from the shot book".into(),
        });
    }
    if fixture.faction_two_hits_side_line && side_lines_damaged {
        issues.push(CityFactionIssue {
            id: "2".into(),
            system: "conflict".into(),
            reason: "faction 2 shot damaged faction 1 lines".into(),
        });
    }

    issues.sort_by(|a, b| {
        a.system
            .cmp(&b.system)
            .then(a.id.cmp(&b.id))
            .then(a.reason.cmp(&b.reason))
    });

    CityFactionProgressReport {
        launch: LAUNCH,
        seed: fixture.seed,
        ticks: step,
        issues,
        faction_progress,
        player_rows,
        town_ids,
        factions,
        haul_moved,
        side_lines_damaged,
        side_shot_on_committed_line,
    }
}

#[must_use]
pub fn city_faction_progress_body(report: &CityFactionProgressReport) -> serde_json::Value {
    let green = report.issues.is_empty();
    serde_json::json!({
        "launch": report.launch,
        "gate": "CITY-FACTION-PROGRESS",
        "seed": report.seed,
        "ticks": report.ticks,
        "green": green,
        "issues": report.issues.iter().map(|issue| serde_json::json!({
            "id": issue.id,
            "system": issue.system,
            "reason": issue.reason,
        })).collect::<Vec<_>>(),
        "faction_progress": report.faction_progress,
        "player_rows": report.player_rows,
        "town_ids": report.town_ids,
        "factions": report.factions,
        "haul_moved": report.haul_moved,
        "verify_cmd": "cargo test -p proc_A_dine01 --lib city_faction_progress -- --test-threads=8",
    })
}

/// Writes the live file from a real report. `green` follows `issues`.
#[must_use]
pub fn write_city_faction_progress_live(report: &CityFactionProgressReport) -> bool {
    let body = city_faction_progress_body(report);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CITY-FACTION-PROGRESS",
        LAUNCH,
        CITY_FACTION_PROGRESS_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(CITY_FACTION_PROGRESS_LIVE_JSON, wrapped)
}

fn settle_books(fixture: &CityFactionFixture) -> (TownBook, DistrictBook) {
    let towns = if fixture.saved_towns.towns.is_empty() {
        generated_town_book(fixture.seed, 4)
    } else {
        fixture.saved_towns.clone()
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(towns)
        .insert_resource(DistrictBook::default())
        .add_systems(Update, seed_settlement_books_if_empty);
    app.update();
    (
        app.world().resource::<TownBook>().clone(),
        app.world().resource::<DistrictBook>().clone(),
    )
}

fn grid_with_side_line() -> UtilityGraph {
    let mut graph = hydrate_utility_graph_from_snapshot(&fixture_utility_network_snapshot());
    graph.nodes.push(UtilityGraphNode {
        id: 50,
        key: "tap".into(),
        position: Vec2::new(0.5, 0.5),
    });
    graph.nodes.push(UtilityGraphNode {
        id: 51,
        key: "bus".into(),
        position: Vec2::new(2.5, 0.5),
    });
    graph.power_edges.push(UtilityGraphEdge {
        from: 50,
        to: 51,
        link_id: SIDE_LINE,
        throughput: 1.0,
    });
    graph
}

fn haul_edge(disruption: f32) -> LogisticsEdge {
    LogisticsEdge {
        from: LogisticsNodeId(0),
        to: LogisticsNodeId(1),
        transport_edge: Some(TransportEdgeId(0)),
        capacity: HAUL_CAPACITY,
        disruption,
        traversal_cost: 1.0,
    }
}

fn haul_graph(edge: &LogisticsEdge) -> LogisticsGraph {
    LogisticsGraph {
        revision: 1,
        nodes: vec![
            LogisticsNode {
                id: LogisticsNodeId(0),
                throughput: 0.0,
                stockpile: 0.0,
                anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
            },
            LogisticsNode {
                id: LogisticsNodeId(1),
                throughput: 0.0,
                stockpile: 0.0,
                anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
            },
        ],
        edges: vec![edge.clone()],
    }
}

fn fire_overlay(heat: f32) -> ChunkFireOverlay {
    ChunkFireOverlay {
        heat: vec![heat, 0.0],
        fuel: vec![0.0, 0.0],
        smoke: vec![0.0, 0.0],
        toxic: vec![0.0, 0.0],
        burn_hold: vec![0, 0],
    }
}

fn refused_label(label: &str) -> bool {
    label
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|word| matches!(word, "castle" | "keep" | "wall" | "gate"))
}

fn labeled_row(label: &str, anchor: IVec2) -> Option<PendingBuildBlueprint> {
    if refused_label(label) {
        return None;
    }
    let mut row = pending_blueprint_from_growth_proposal(&GrowthProposal {
        district_id: DistrictId("city-faction".into()),
        block_id: None,
        archetype_id: ArchetypeId("warehouse".into()),
        usage: BuildingUsage::Industrial,
        actor_layer: GrowthActorLayer::Growth,
        anchor_tile: anchor,
        priority: 1.0,
        seed: 1,
        reason_codes: Vec::new(),
        saturation_at_submit: 0.0,
    });
    row.label = label.to_string();
    Some(row)
}

fn publish_city_tick(mut step: ResMut<CityFactionStep>, mut tick: ResMut<SimTick>) {
    tick.0 = step.tick;
    step.tick = step.tick.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::settlement::portland_fixture_town;

    #[test]
    fn city_faction_progress_cli_city_flag_matches_vfx_window() {
        use crate::engine::{DebugManeuver, EngineLaunchArgs, TestScene};
        let city = EngineLaunchArgs::from_cli(Some("city".into()), false, None);
        let vfx = EngineLaunchArgs::from_cli(Some("vfx".into()), false, None);
        assert_eq!(city.test_scene, TestScene::City);
        assert_eq!(city.maneuver, DebugManeuver::DemoOpen);
        assert_eq!(city.maneuver, vfx.maneuver);
        assert!(!city.visual_auto_exit);
        assert_eq!(city.visual_auto_exit, vfx.visual_auto_exit);
        assert!(city.test_mode());
        assert!(!city.test_scene.seeds_fire_overlay());
    }

    #[test]
    fn city_faction_progress_healthy_is_empty() {
        let report = city_faction_progress(&CityFactionFixture::healthy(99_001));
        assert_eq!(report.ticks, CITY_FACTION_PROGRESS_TICKS);
        assert_eq!(report.factions, vec![1, 2]);
        assert!(report.haul_moved, "open haul should move");
        assert!(!report.faction_progress.is_empty());
        assert!(report
            .faction_progress
            .iter()
            .all(|label| label.starts_with("growth:")));
        assert!(report.player_rows.iter().any(|label| label == "player:warehouse"));
        assert!(report.town_ids.len() >= 2, "{:?}", report.town_ids);
        assert!(report.issues.is_empty(), "{:?}", report.issues);
    }

    #[test]
    fn city_faction_progress_cut_feeders_are_a_power_issue() {
        let mut fixture = CityFactionFixture::healthy(99_001);
        fixture.cut_both_feeders = true;
        let report = city_faction_progress(&fixture);
        assert!(
            report.issues.iter().any(|issue| issue.system == "power"),
            "{:?}",
            report.issues
        );
        assert!(!report.issues.is_empty());
        let body = city_faction_progress_body(&report);
        assert_eq!(body["green"], false);
        assert!(write_city_faction_progress_live(&report));
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(CITY_FACTION_PROGRESS_LIVE_JSON);
        let text = std::fs::read_to_string(&path).expect("live json");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        let issues = value["issues"].as_array().expect("issues");
        assert!(issues.iter().any(|issue| issue["system"] == "power"));
        assert_eq!(value["green"], false);
    }

    #[test]
    fn city_faction_progress_full_disruption_is_a_logistics_issue() {
        let mut fixture = CityFactionFixture::healthy(99_001);
        fixture.edge_disruption = 1.0;
        let report = city_faction_progress(&fixture);
        assert!(
            report.issues.iter().any(|issue| issue.system == "logistics"),
            "{:?}",
            report.issues
        );
        assert!(!report.haul_moved);
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn city_faction_progress_unprefixed_growth_is_not_progress() {
        let mut fixture = CityFactionFixture::healthy(99_001);
        fixture.growth_label = Some("mill".into());
        let report = city_faction_progress(&fixture);
        assert!(report
            .faction_progress
            .iter()
            .all(|label| label.starts_with("growth:")));
        assert!(!report.faction_progress.iter().any(|label| label == "mill"));
        assert!(report.player_rows.iter().any(|label| label == "player:warehouse"));
        assert!(
            report.issues.iter().any(|issue| issue.system == "growth"),
            "{:?}",
            report.issues
        );
    }

    #[test]
    fn city_faction_progress_faction_two_does_not_blame_side() {
        let mut fixture = CityFactionFixture::healthy(99_001);
        fixture.faction_two_hits_side_line = true;
        let report = city_faction_progress(&fixture);
        assert!(report.side_shot_on_committed_line);
        assert!(!report.side_lines_damaged);
        assert!(report.issues.iter().all(|issue| {
            !(issue.system == "conflict" && issue.id == "1")
        }));
        assert_eq!(report.factions, vec![1, 2]);
    }

    #[test]
    fn city_faction_progress_same_seed_is_stable() {
        let fixture = CityFactionFixture::healthy(42_042);
        let first = city_faction_progress(&fixture);
        let second = city_faction_progress(&fixture);
        assert_eq!(first.issues, second.issues);
        assert_eq!(first.town_ids, second.town_ids);
        assert_eq!(first.faction_progress, second.faction_progress);

        let mut cut = CityFactionFixture::healthy(42_042);
        cut.cut_both_feeders = true;
        let broken_a = city_faction_progress(&cut);
        let broken_b = city_faction_progress(&cut);
        assert_eq!(broken_a.issues, broken_b.issues);
        assert!(broken_a.issues.iter().any(|issue| issue.system == "power"));
    }

    #[test]
    fn city_faction_progress_saved_towns_stay() {
        let mut fixture = CityFactionFixture::healthy(1);
        fixture.saved_towns = portland_fixture_town();
        let report = city_faction_progress(&fixture);
        assert!(report.town_ids.iter().any(|id| id == "portland"));
        assert!(!report.town_ids.iter().any(|id| id.starts_with("gen-")));
    }
}
