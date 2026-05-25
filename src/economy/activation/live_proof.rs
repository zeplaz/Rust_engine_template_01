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
) -> serde_json::Value {
    let ids: Vec<&str> = INDUSTRIAL_ACTIVATION_TODOS.iter().map(|t| t.id).collect();
    let open = board.map(|b| b.open_count()).unwrap_or(INDUSTRIAL_ACTIVATION_TODO_COUNT);
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

#[cfg(test)]
mod live_proof_tests {
    use super::*;
    use std::fs;
    use crate::dev::industrial_activation_todos::register_industrial_activation_todo_hooks;
    use crate::economy::activation::bridge::{
        activate_industrial_facilities_system, refresh_industrial_activation_witness_system,
        sync_industrial_activation_board_system,
    };
    use crate::economy::activation::concrete_chain_e2e::{
        commit_concrete_portland_chain_in_play, fast_forward_portland_chain_sites_to_operational,
        refresh_concrete_chain_e2e_witness_system, spawn_concrete_portland_chain_operational,
        ConcreteChainE2eWitness,
    };
    use bevy::ecs::system::RunSystemOnce;
    use crate::economy::resource_flow::register_resource_flow_nodes_system;
    use crate::economy::resource_flow::link_supply_chain_edges_system;
    use crate::construction::{default_buildings_dir, load_building_definitions_from_dir};
    use crate::economy::ResourceFlowPlugin;

    fn assemble_industrial_proof_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<BaseState>();
        app.insert_state(BaseState::Simulation);

        register_industrial_activation_todo_hooks(&mut app);
        app.insert_resource(load_building_definitions_from_dir(default_buildings_dir()));
        app.insert_resource(crate::systems::sim_control::SimControlState::default());
        app.init_resource::<crate::strategic::LogisticsGraph>();
        app.init_resource::<crate::economy::spatial_district::IndustrialDistrictSnapshot>();

        app.init_resource::<IndustrialActivationLiveProofState>();
        app.world_mut()
            .resource_mut::<IndustrialActivationLiveProofState>()
            .write_interval = 5;

        app.init_resource::<ConcreteChainE2eWitness>();

        app.add_plugins(ResourceFlowPlugin);
        app.add_systems(
            Update,
            (
                refresh_industrial_activation_witness_system,
                sync_industrial_activation_board_system,
                activate_industrial_facilities_system,
                register_resource_flow_nodes_system,
                link_supply_chain_edges_system,
                refresh_concrete_chain_e2e_witness_system,
                sync_industrial_proof_witness_flags,
                write_industrial_activation_live_proof_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn simulation_writes_industrial_activation_live_json_concrete_chain_e2e() {
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        spawn_concrete_portland_chain_operational(
            &mut app.world_mut().commands(),
            crate::strategic::BuildSiteTile { x: 32, z: 32 },
        );
        for _ in 0..24 {
            app.update();
        }
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
                    .before(refresh_industrial_activation_witness_system),
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

        for _ in 0..32 {
            app.update();
        }

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
        let _ = fs::remove_file(proof_output_path());
        let mut app = assemble_industrial_proof_app();
        for _ in 0..24 {
            app.update();
        }
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
        assert!(app.world().resource::<IndustrialActivationLiveProofState>().written);
    }
}
