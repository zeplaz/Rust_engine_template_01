//! Writes `debug_runs/industrial_activation_live.json` during simulation (I1-05).

use std::path::PathBuf;

use bevy::prelude::*;

use crate::dev::construction_live_todos::TodoStatus;
use crate::dev::industrial_activation_todos::{
    IndustrialActivationTodoBoard, IndustrialActivationWitness, INDUSTRIAL_ACTIVATION_TODO_COUNT,
    INDUSTRIAL_ACTIVATION_TODOS,
};
use crate::engine::states::BaseState;

#[derive(Resource, Debug)]
pub struct IndustrialActivationLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for IndustrialActivationLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
        }
    }
}

#[allow(dead_code)]
fn proof_output_path() -> PathBuf {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("debug_runs")
        .join("industrial_activation_live.json")
}

/// **IND-E03-WITNESS-001** — canonical `grid_overload` block for `industrial_activation_live.json`.
#[must_use]
pub fn grid_overload_witness_export(
    witness: &IndustrialActivationWitness,
    chain: &super::concrete_chain_e2e::ConcreteChainE2eWitness,
    flow: Option<&crate::economy::resource_flow::ResourceFlowSimWitness>,
) -> serde_json::Value {
    let overload_events_total = flow.map(|f| f.overload_events_total).unwrap_or(0);
    let ind_e03_green = chain.production_green()
        && witness.grid_membership
        && witness.grid_overload_hook;
    serde_json::json!({
        "grid_overload_hook": witness.grid_overload_hook,
        "grid_overload_sim_green": witness.grid_overload_hook,
        "grid_membership": witness.grid_membership,
        "overload_events_total": overload_events_total,
        "production_green": chain.production_green(),
        "ind_e03_green": ind_e03_green,
        "green": ind_e03_green,
        "ind_e03_witness_001_green": ind_e03_green && overload_events_total >= 1,
    })
}

#[must_use]
pub fn ind_e03_witness_001_green(grid_overload: &serde_json::Value) -> bool {
    grid_overload
        .get("ind_e03_witness_001_green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn board_snapshot(ids: &[&str], statuses: &[TodoStatus]) -> serde_json::Value {
    serde_json::json!(
        ids.iter()
            .zip(statuses.iter())
            .map(|(id, st)| {
                serde_json::json!({
                    "id": id,
                    "status": format!("{st:?}"),
                })
            })
            .collect::<Vec<_>>()
    )
}

fn build_proof_payload(
    board: Option<&IndustrialActivationTodoBoard>,
    witness: &IndustrialActivationWitness,
    governance_violations: usize,
    district: Option<&crate::economy::spatial_district::IndustrialDistrictSnapshot>,
    chain: &super::concrete_chain_e2e::ConcreteChainE2eWitness,
    flow: Option<&crate::economy::resource_flow::ResourceFlowSimWitness>,
    toast: Option<&super::grid_overload_ux::GridOverloadToastState>,
) -> serde_json::Value {
    let ids: Vec<&str> = INDUSTRIAL_ACTIVATION_TODOS.iter().map(|t| t.id).collect();
    let open = board.map(|b| b.open_count()).unwrap_or(INDUSTRIAL_ACTIVATION_TODO_COUNT);
    let grid_overload = grid_overload_witness_export(witness, chain, flow);
    let overload_events = flow.map(|f| f.overload_events_total).unwrap_or(0);
    let toast_ui_wired = super::grid_overload_ux::s7p_grid_ux_toast_ui_wired();
    let toast_shown = toast.map(|t| t.show_count).unwrap_or(0);
    let s7p_grid_ux = serde_json::json!({
        "gate": "S7P-GRID-UX-001",
        "toast_message": super::grid_overload_ux::GRID_OVERLOAD_TOAST_MESSAGE,
        "toast_ui_wired": toast_ui_wired,
        "toast_armed": overload_events > 0,
        "toast_shown_count": toast_shown,
        "toast_active": toast_shown > 0,
        "green": super::grid_overload_ux::s7p_grid_ux_001_green(
            toast.unwrap_or(&super::grid_overload_ux::GridOverloadToastState::default()),
            overload_events,
        ),
    });
    serde_json::json!({
        "profile": "INDUSTRIAL_ACTIVATION",
        "activation_green": open == 0,
        "open_todos": open,
        "todo_total": INDUSTRIAL_ACTIVATION_TODO_COUNT,
        "board": board.map(|b| board_snapshot(&ids, &b.status)),
        "concrete_chain_e2e": {
            "chain_id": super::concrete_chain_e2e::CONCRETE_PORTLAND_CHAIN,
            "operational_mine": chain.operational_mine,
            "operational_kiln": chain.operational_kiln,
            "operational_mixer": chain.operational_mixer,
            "activated_mine": chain.activated_mine,
            "activated_kiln": chain.activated_kiln,
            "activated_mixer": chain.activated_mixer,
            "flow_edges": chain.flow_edges,
            "production_ticks": chain.production_ticks,
            "chain_operational": chain.chain_operational(),
            "production_green": chain.production_green(),
            "placed_via_construction": chain.placed_via_construction,
            "sites_committed": chain.sites_committed,
            "ind_e02_green": chain.in_play_green(),
        },
        "witness": {
            "catalog_id_on_commit": witness.catalog_id_on_commit,
            "activation_system": witness.activation_system,
            "supply_chain_index": witness.supply_chain_index,
            "supply_chain_catalog_complete": witness.supply_chain_catalog_complete,
            "role_based_activation": witness.role_based_activation,
            "resource_flow_node": witness.resource_flow_node,
            "register_node_on_activate": witness.register_node_on_activate,
            "transformer_catalog": witness.transformer_catalog,
            "transformer_activation": witness.transformer_activation,
            "no_mega_factory_collapse": witness.no_mega_factory_collapse,
            "proof_json": true,
        },
        "governance_violation_count": governance_violations,
        "spatial_district": district.map(|d| serde_json::json!({
            "host_count": d.hosts.len(),
            "dominant_load_ratio": d.dominant_host_load_ratio(),
            "clustered_hosts": d.clustered_host_count(),
        })),
        "grid_overload": grid_overload.clone(),
        "ind_e03": grid_overload,
        "industrial_i3_02_green": witness.grid_overload_hook
            && flow.map(|f| f.overload_events_total > 0).unwrap_or(false),
        "s7p_grid_ux_001": s7p_grid_ux,
    })
}

pub fn write_industrial_activation_live_proof_system(
    base: Option<Res<State<BaseState>>>,
    mut state: ResMut<IndustrialActivationLiveProofState>,
    board: Option<Res<IndustrialActivationTodoBoard>>,
    witness: Res<IndustrialActivationWitness>,
    chain: Res<super::concrete_chain_e2e::ConcreteChainE2eWitness>,
    buildings: Option<Res<crate::construction::BuildingDefinitionRegistry>>,
    district: Option<Res<crate::economy::spatial_district::IndustrialDistrictSnapshot>>,
    flow: Option<Res<crate::economy::resource_flow::ResourceFlowSimWitness>>,
    toast: Option<Res<super::grid_overload_ux::GridOverloadToastState>>,
) {
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    let gov_count = buildings
        .as_deref()
        .map(|r| r.governance_violations.len())
        .unwrap_or(0);
    let mut witness_snap = witness.as_ref().clone();
    witness_snap.proof_json = true;

    let payload = build_proof_payload(
        board.as_deref(),
        &witness_snap,
        gov_count,
        district.as_deref(),
        chain.as_ref(),
        flow.as_deref(),
        toast.as_deref(),
    );
    const PROOF_PATH: &str = "debug_runs/industrial_activation_live.json";
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INDUSTRIAL_ACTIVATION",
        "industrial_activation_live_proof",
        PROOF_PATH,
        payload,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, wrapped) {
        state.written = true;
    }
}

pub fn sync_industrial_proof_witness_flags(
    proof: Res<IndustrialActivationLiveProofState>,
    mut witness: ResMut<IndustrialActivationWitness>,
) {
    if proof.written {
        witness.proof_json = true;
    }
}

/// Headless proof tests share one JSON path — serialize writes/reads.
static PROOF_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn industrial_proof_file_lock() -> std::sync::MutexGuard<'static, ()> {
    PROOF_FILE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn assemble_industrial_proof_app() -> App {
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
    use crate::dev::industrial_activation_todos::register_industrial_activation_todo_hooks;
    use crate::economy::activation::bridge::{
        activate_industrial_facilities_system, refresh_industrial_activation_witness_system,
        sync_industrial_activation_board_system,
    };
    use crate::economy::activation::concrete_chain_e2e::{
        refresh_concrete_chain_e2e_witness_system, ConcreteChainE2eWitness,
    };
    use crate::economy::resource_flow::{
        collect_grid_overload_witness_system, register_resource_flow_nodes_system,
    };
    use crate::economy::ResourceFlowPlugin;
    use crate::entities::production::power::PowerRuntimePlugin;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_state::<BaseState>();
    app.insert_state(BaseState::Simulation);

    register_industrial_activation_todo_hooks(&mut app);
    app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
    app.insert_resource(crate::systems::sim_control::SimControlState::default());
    app.insert_resource(crate::systems::sim_control::SimTick(0));
    app.init_resource::<crate::strategic::LogisticsGraph>();
    app.init_resource::<crate::economy::spatial_district::IndustrialDistrictSnapshot>();

    app.init_resource::<IndustrialActivationLiveProofState>();
    app.world_mut()
        .resource_mut::<IndustrialActivationLiveProofState>()
        .write_interval = 5;

    app.init_resource::<ConcreteChainE2eWitness>();

    app.add_plugins((ResourceFlowPlugin, PowerRuntimePlugin));
    app.add_plugins(super::grid_overload_ux::GridOverloadUxPlugin);

    app.add_systems(
        Update,
        activate_industrial_facilities_system.before(register_resource_flow_nodes_system),
    );
    app.add_systems(
        Update,
        (
            refresh_concrete_chain_e2e_witness_system,
            refresh_industrial_activation_witness_system,
            sync_industrial_activation_board_system,
            sync_industrial_proof_witness_flags,
            write_industrial_activation_live_proof_system,
        )
            .chain()
            .after(collect_grid_overload_witness_system),
    );
    app
}

fn prime_industrial_activation_proof_entities(app: &mut App) {
    use crate::economy::activation::concrete_chain_e2e::{
        commit_concrete_portland_chain_in_play, spawn_ind_e03_grid_overload_cluster,
        ConcreteChainE2eWitness,
    };
    use crate::strategic::BuildSiteTile;
    use bevy::ecs::system::RunSystemOnce;

    app.init_resource::<crate::strategic::SiteConstructionBook>()
        .init_resource::<crate::strategic::SiteIdIssuer>()
        .add_message::<crate::strategic::CommitConstructionSiteEvent>()
        .add_systems(
            Update,
            (
                crate::strategic::commit_construction_site_system,
                crate::economy::activation::concrete_chain_e2e::fast_forward_portland_chain_sites_to_operational,
            )
                .chain(),
        );

    let origin = BuildSiteTile { x: 32, z: 32 };
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut()
        .run_system_once(
            move |mut writer: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
                  mut witness: ResMut<ConcreteChainE2eWitness>| {
                commit_concrete_portland_chain_in_play(
                    &mut writer,
                    witness.as_mut(),
                    owner,
                    origin,
                );
            },
        )
        .expect("enqueue portland commits for proof sim");
    app.update();
    spawn_ind_e03_grid_overload_cluster(
        &mut app.world_mut().commands(),
        BuildSiteTile {
            x: origin.x.saturating_add(2),
            z: origin.z.saturating_add(2),
        },
    );
}

fn run_industrial_proof_frames(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
    }
}

/// **IND-E02-DEFAULT** — live JSON with `ind_e02_green: true` (construction commit path).
pub fn refresh_ind_e02_default_live_witness() -> bool {
    use crate::economy::activation::bridge::activate_industrial_facilities_system;
    use crate::economy::activation::concrete_chain_e2e::{
        commit_concrete_portland_chain_in_play, fast_forward_portland_chain_sites_to_operational,
        spawn_ind_e03_grid_overload_cluster, ConcreteChainE2eWitness,
    };
    use bevy::ecs::system::RunSystemOnce;

    let _guard = industrial_proof_file_lock();
    let _ = std::fs::remove_file(proof_output_path());
    let mut app = assemble_industrial_proof_app();
    app.init_resource::<crate::strategic::SiteConstructionBook>()
        .init_resource::<crate::strategic::SiteIdIssuer>()
        .add_message::<crate::strategic::CommitConstructionSiteEvent>()
        .add_systems(
            Update,
            (
                crate::strategic::commit_construction_site_system,
                fast_forward_portland_chain_sites_to_operational,
            )
                .before(activate_industrial_facilities_system),
        );

    let owner = app.world_mut().spawn_empty().id();
    app.world_mut()
        .run_system_once(
            move |mut writer: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
                  mut witness: ResMut<ConcreteChainE2eWitness>| {
                commit_concrete_portland_chain_in_play(
                    &mut writer,
                    witness.as_mut(),
                    owner,
                    crate::strategic::BuildSiteTile { x: 40, z: 40 },
                );
            },
        )
        .expect("enqueue portland commits");

    spawn_ind_e03_grid_overload_cluster(
        &mut app.world_mut().commands(),
        crate::strategic::BuildSiteTile { x: 52, z: 52 },
    );
    run_industrial_proof_frames(&mut app, 32);

    let path = proof_output_path();
    if !path.exists() {
        return false;
    }
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("parse");
    json["concrete_chain_e2e"]["ind_e02_green"]
        .as_bool()
        .unwrap_or(false)
}

#[cfg(test)]
mod live_proof_tests {
    use super::*;
    use std::fs;

    use crate::economy::activation::bridge::activate_industrial_facilities_system;
    use crate::economy::activation::concrete_chain_e2e::{
        commit_concrete_portland_chain_in_play, fast_forward_portland_chain_sites_to_operational,
        spawn_ind_e03_grid_overload_cluster, ConcreteChainE2eWitness,
    };
    use bevy::ecs::system::RunSystemOnce;
    use crate::strategic::BuildSiteTile;

    #[test]
    fn simulation_writes_industrial_activation_live_json_concrete_chain_e2e() {
        let _guard = industrial_proof_file_lock();
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        prime_industrial_activation_proof_entities(&mut app);
        run_industrial_proof_frames(&mut app, 32);
        let path = proof_output_path();
        assert!(path.exists(), "expected {:?}", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(json["profile"], "INDUSTRIAL_ACTIVATION");
        assert!(
            json["concrete_chain_e2e"]["production_green"]
                .as_bool()
                .unwrap_or(false),
            "expected IND-E01 production_green in proof JSON: {}",
            json["concrete_chain_e2e"]
        );
        assert!(
            json["activation_green"].as_bool().unwrap_or(false),
            "expected activation_green, open_todos={}",
            json["open_todos"]
        );
        assert!(
            json["concrete_chain_e2e"]["production_ticks"]
                .as_u64()
                .unwrap_or(0)
                >= 1,
            "expected at least one production tick"
        );
        assert!(app.world().resource::<IndustrialActivationLiveProofState>().written);
    }

    #[test]
    fn simulation_writes_industrial_activation_live_json_ind_e02_in_play() {
        let _guard = industrial_proof_file_lock();
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        app.init_resource::<crate::strategic::SiteConstructionBook>()
            .init_resource::<crate::strategic::SiteIdIssuer>()
            .add_message::<crate::strategic::CommitConstructionSiteEvent>()
            .add_systems(
                Update,
                (
                    crate::strategic::commit_construction_site_system,
                    fast_forward_portland_chain_sites_to_operational,
                )
                    .before(activate_industrial_facilities_system),
            );

        let owner = app.world_mut().spawn_empty().id();
        app.world_mut()
            .run_system_once(
                move |mut writer: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
                      mut witness: ResMut<ConcreteChainE2eWitness>| {
                    commit_concrete_portland_chain_in_play(
                        &mut writer,
                        witness.as_mut(),
                        owner,
                        crate::strategic::BuildSiteTile { x: 40, z: 40 },
                    );
                },
            )
            .expect("enqueue portland commits");

        spawn_ind_e03_grid_overload_cluster(
            &mut app.world_mut().commands(),
            BuildSiteTile { x: 52, z: 52 },
        );
        run_industrial_proof_frames(&mut app, 32);

        let path = proof_output_path();
        assert!(path.exists(), "expected {:?}", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert!(
            json["concrete_chain_e2e"]["ind_e02_green"]
                .as_bool()
                .unwrap_or(false),
            "IND-E02 in_play_green: {}",
            json["concrete_chain_e2e"]
        );
        assert!(
            json["concrete_chain_e2e"]["placed_via_construction"]
                .as_bool()
                .unwrap_or(false)
        );
        assert!(
            json["activation_green"].as_bool().unwrap_or(false),
            "expected activation_green after board sync: open_todos={}",
            json["open_todos"]
        );
        assert_eq!(json["open_todos"], serde_json::json!(0));
    }

    #[test]
    fn simulation_writes_industrial_activation_live_json() {
        let _guard = industrial_proof_file_lock();
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        prime_industrial_activation_proof_entities(&mut app);
        run_industrial_proof_frames(&mut app, 32);
        let path = proof_output_path();
        assert!(path.exists(), "expected {:?}", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(json["profile"], "INDUSTRIAL_ACTIVATION");
        assert!(
            json["activation_green"].as_bool().unwrap_or(false),
            "expected activation_green, open_todos={}",
            json["open_todos"]
        );
        assert!(
            json["industrial_i3_02_green"].as_bool().unwrap_or(false),
            "INDUSTRIAL-I3-02: expected grid overload in proof JSON: {}",
            json["ind_e03"]
        );
        assert_board_row_done(&json, "INDUSTRIAL-I3-02");
        assert!(app.world().resource::<IndustrialActivationLiveProofState>().written);
    }

    fn assert_board_row_done(json: &serde_json::Value, row_id: &str) {
        let board = json["board"].as_array().expect("board array");
        let row = board
            .iter()
            .find(|r| r["id"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing board row {row_id}"));
        assert_eq!(
            row["status"].as_str(),
            Some("Done"),
            "expected {row_id} Done"
        );
    }

    #[test]
    fn grid_overload_witness_export_ind_e03_witness_001_green_predicate() {
        let mut witness = IndustrialActivationWitness::default();
        witness.grid_overload_hook = true;
        witness.grid_membership = true;
        let mut chain = ConcreteChainE2eWitness::default();
        chain.operational_mine = 1;
        chain.operational_kiln = 1;
        chain.operational_mixer = 1;
        chain.activated_mine = 1;
        chain.activated_kiln = 1;
        chain.activated_mixer = 1;
        chain.flow_edges = 2;
        chain.production_ticks = 1;
        let flow = crate::economy::resource_flow::ResourceFlowSimWitness {
            overload_events_total: 2,
            ..Default::default()
        };
        let block = grid_overload_witness_export(&witness, &chain, Some(&flow));
        assert!(block["ind_e03_green"].as_bool().unwrap_or(false));
        assert!(ind_e03_witness_001_green(&block));
        let no_events = grid_overload_witness_export(&witness, &chain, None);
        assert!(!ind_e03_witness_001_green(&no_events));
    }

    /// **INDUSTRIAL-I3-02** — `GridOverloadEvent` / brownout hook + live proof rollup.
    #[test]
    fn simulation_writes_industrial_activation_live_json_i3_02_grid_overload() {
        let _guard = industrial_proof_file_lock();
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        prime_industrial_activation_proof_entities(&mut app);
        run_industrial_proof_frames(&mut app, 32);
        let path = proof_output_path();
        assert!(path.exists(), "expected {:?}", path);
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert!(
            json["concrete_chain_e2e"]["production_green"]
                .as_bool()
                .unwrap_or(false),
            "IND-E03: expected stable concrete E2E production_green"
        );
        let grid_overload = &json["grid_overload"];
        assert!(
            grid_overload["ind_e03_green"].as_bool().unwrap_or(false),
            "IND-E03-WITNESS-001: expected grid_overload.ind_e03_green: {}",
            grid_overload
        );
        assert!(
            grid_overload["grid_overload_sim_green"]
                .as_bool()
                .unwrap_or(false),
            "IND-E03-WITNESS-001: expected grid_overload_sim_green"
        );
        assert!(
            grid_overload["overload_events_total"].as_u64().unwrap_or(0) >= 1,
            "IND-E03-WITNESS-001: expected overload_events_total > 0: {}",
            grid_overload["overload_events_total"]
        );
        assert!(
            ind_e03_witness_001_green(grid_overload),
            "IND-E03-WITNESS-001: expected ind_e03_witness_001_green"
        );
        assert_eq!(json["ind_e03"], *grid_overload, "ind_e03 mirrors grid_overload");
        assert!(
            json["industrial_i3_02_green"].as_bool().unwrap_or(false),
            "INDUSTRIAL-I3-02: expected industrial_i3_02_green: {}",
            json
        );
        assert_board_row_done(&json, "INDUSTRIAL-I3-02");
        assert!(
            app.world()
                .resource::<crate::dev::IndustrialActivationWitness>()
                .grid_overload_hook
        );
        let s7p = &json["s7p_grid_ux_001"];
        assert_eq!(s7p["toast_ui_wired"], serde_json::json!(true));
        assert!(
            s7p["toast_shown_count"].as_u64().unwrap_or(0) >= 1,
            "S7P-GRID-UX-UI-001: expected toast after overload: {s7p}"
        );
        assert!(
            s7p["green"].as_bool().unwrap_or(false),
            "S7P-GRID-UX-001 green: {s7p}"
        );
    }

    /// **IND-E02-DEFAULT-PLAY-001** — default sim writer sets `ind_e02_green` without seed env.
    #[test]
    fn simulation_ind_e02_default_play_writer_sets_ind_e02_green() {
        let _guard = industrial_proof_file_lock();
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        app.init_resource::<crate::strategic::SiteConstructionBook>()
            .init_resource::<crate::strategic::SiteIdIssuer>()
            .init_resource::<crate::economy::activation::concrete_chain_e2e::IndE02DefaultPlaySeedState>()
            .add_message::<crate::strategic::CommitConstructionSiteEvent>()
            .add_systems(
                Update,
                (
                    crate::economy::activation::concrete_chain_e2e::seed_ind_e02_default_play_once,
                    crate::strategic::commit_construction_site_system,
                    crate::economy::activation::concrete_chain_e2e::fast_forward_portland_chain_sites_to_operational,
                )
                    .chain(),
            );
        run_industrial_proof_frames(&mut app, 48);
        let json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(proof_output_path()).expect("read industrial proof"),
        )
        .expect("parse");
        assert!(
            json["concrete_chain_e2e"]["ind_e02_green"]
                .as_bool()
                .unwrap_or(false),
            "IND-E02-DEFAULT-PLAY-001: {}",
            json["concrete_chain_e2e"]
        );
        assert!(
            json["concrete_chain_e2e"]["placed_via_construction"]
                .as_bool()
                .unwrap_or(false)
        );
    }
}
