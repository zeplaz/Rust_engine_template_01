//! **INDUSTRIAL_ACTIVATION_GREEN** — construction → live industrial economy bridge.
//!
//! Spec (assessment §1811+): [`super::industrial_activation_phase_todos.md`](super::industrial_activation_phase_todos.md)  
//! Pipeline: [`super::industrial_activation_pipeline.md`](super::industrial_activation_pipeline.md)  
//! **Prerequisite:** `CONSTRUCTION_OPERATIONAL_GREEN`. **Not** Stage 5.

use bevy::log::info;
use bevy::prelude::{App, Resource};

use super::construction_live_todos::TodoStatus;
use super::stage5_live_todos::Stage5LiveTodo;

pub const INDUSTRIAL_ACTIVATION_TODO_COUNT: usize = 31;

pub static INDUSTRIAL_ACTIVATION_TODOS: &[Stage5LiveTodo] = &[
    // ── I1 Bridge ───────────────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "INDUSTRIAL-I1-01",
        status: TodoStatus::Done,
        file: "src/strategic/site/events.rs",
        system: "CatalogIdOnCommit",
        goal: "`catalog_id` on commit → `BuildingDefinitionRef` on site entity.",
        runtime_check: "Commit event + planned site carry catalog id.",
        failure_mode: "Sites only carry `SiteArchetype`; JSON defs disconnected.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I1-02",
        status: TodoStatus::Done,
        file: "src/economy/activation/bridge.rs",
        system: "ActivationOnOperational",
        goal: "`activate_industrial_facilities_system` when `SiteConstructionPhase::Operational`.",
        runtime_check: "Role-based bundles via `insert_supply_chain_runtime_for_catalog`.",
        failure_mode: "Operational sites have no ECS production bundle.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I1-03",
        status: TodoStatus::Done,
        file: "src/economy/supply_chain.rs",
        system: "ElectricalLoadFromDef",
        goal: "`ElectricalComponent` from `BuildingDefinition.power_consumption`.",
        runtime_check: "Smelter base_load ≫ mine in unit test.",
        failure_mode: "Power JSON ignored at runtime.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I1-04",
        status: TodoStatus::Done,
        file: "src/economy/activation/bridge.rs",
        system: "ActivationUnitTest",
        goal: "Unit tests: integrated concrete, aggregate-only, aluminum chain steps.",
        runtime_check: "`cargo test -p proc_A_dine01 economy:: --lib`.",
        failure_mode: "Bridge untested.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I1-05",
        status: TodoStatus::Done,
        file: "debug_runs/industrial_activation_live.json",
        system: "ActivationProofJson",
        goal: "Machine proof JSON: witness flags + open/done board snapshot.",
        runtime_check: "Written in sim; includes supply_chain rows.",
        failure_mode: "Closure not measurable.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I1-06",
        status: TodoStatus::Done,
        file: "src/economy/activation/bridge.rs",
        system: "PowerPlantActivation",
        goal: "Utilities picks spawn `PowerPlant` with `definition_id` from `plant_definitions.json`.",
        runtime_check: "Coal/gas row drives generation after Operational.",
        failure_mode: "Power plants remain archetype-only.",
    },
    // ── SC Supply-chain granularity (recovery §1811–1940) ─────────────────
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-01",
        status: TodoStatus::Done,
        file: "assets/configs/industrial_supply_chains.json",
        system: "SupplyChainIndex",
        goal: "Authoritative chain index: concrete Portland/Geopolymer + aluminum primary.",
        runtime_check: "JSON lists roles, catalog_ids, power, produces/consumes.",
        failure_mode: "Chains only documented in prose.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-02",
        status: TodoStatus::Done,
        file: "assets/configs/buildings/",
        system: "PerStepCatalog",
        goal: "One JSON per process step (not mega-building only).",
        runtime_check: "`supply_chain_catalog_covers` test — 7+ step ids in registry.",
        failure_mode: "Player cannot place kiln/mine/refinery separately.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-03",
        status: TodoStatus::Done,
        file: "src/economy/supply_chain.rs",
        system: "RoleBasedActivation",
        goal: "`supply_chain_role` → exactly one runtime bundle (no hardcoded id match only).",
        runtime_check: "Aggregate mine spawns `AggregateMineRuntime` without kiln.",
        failure_mode: "Activation re-collapses to monolith stubs.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-04",
        status: TodoStatus::Done,
        file: "src/construction/industrial_menu.rs",
        system: "ChainGroupedMenu",
        goal: "Industrial submenu grouped by `supply_chain` with power labels.",
        runtime_check: "UI shows concrete_portland / aluminum_primary sections.",
        failure_mode: "Flat list hides chain topology.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-05",
        status: TodoStatus::Done,
        file: "assets/configs/buildings/concrete_mixer_geopolymer.json",
        system: "GeopolymerPath",
        goal: "Geopolymer kiln + mixer defs with `concrete_type: Geopolymer`.",
        runtime_check: "Registry resolves Geopolymer on mixer def.",
        failure_mode: "Only Portland concrete technology path.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-06",
        status: TodoStatus::Done,
        file: "assets/configs/buildings/aluminum_fabrication_plant.json",
        system: "AluminumFourSteps",
        goal: "Bauxite → alumina → smelter → fabrication as separate catalog rows.",
        runtime_check: "Distinct runtime per step in `economy::activation` tests.",
        failure_mode: "Smelter-only aluminum industry.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-07",
        status: TodoStatus::Done,
        file: "src/economy/supply_chain.rs",
        system: "SupplyChainMembership",
        goal: "`IndustrialSupplyChainMembership { chain_id, role }` on activated sites.",
        runtime_check: "Component present when def has `supply_chain`.",
        failure_mode: "No anchor for I2 edges between facilities.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-SC-08",
        status: TodoStatus::Done,
        file: "src/economy/activation/bridge.rs",
        system: "PowerAsymmetry",
        goal: "Upstream low power / downstream high power (mine 22 vs smelter 200).",
        runtime_check: "Unit test: smelter_load > mine_load * 5.",
        failure_mode: "Flat power — no grid scaling pressure.",
    },
    // ── I2 Resource flow graph (recovery §2026–2093) ────────────────────────
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-01",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "ResourceFlowNode",
        goal: "`ResourceFlowNode` resource: inventory, throughput_limit, production/consumption rates.",
        runtime_check: "Registry lookup by site entity returns rates matching JSON.",
        failure_mode: "Produces/consumes are display-only.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-02",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "ResourceFlowEdge",
        goal: "`ResourceFlowEdge { from, to, transport_mode, max_rate, latency }`.",
        runtime_check: "Edge registry stores at least one mine→refinery link in test.",
        failure_mode: "No transfer graph — facilities isolated.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-03",
        status: TodoStatus::Open,
        file: "src/economy/activation/bridge.rs",
        system: "RegisterNodeOnActivate",
        goal: "On activation, register `ResourceFlowNode` from def produces/consumes + membership.",
        runtime_check: "Operational smelter node lists Alumina consume + Aluminum produce.",
        failure_mode: "Activation spawns ECS but not economic graph.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-04",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "ResourceTypeMapping",
        goal: "Map JSON resource strings → `ResourceType` where names align.",
        runtime_check: "Concrete plant output registers as `ResourceType::Concrete`.",
        failure_mode: "String tags never join sim resource enum.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-05",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "FacilityInventory",
        goal: "Per-node `inventory` buffer on `ResourceFlowNode`.",
        runtime_check: "Sim tick: kiln inventory holds cement after production stub.",
        failure_mode: "Stateless flavor factories.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-06",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "ThroughputPropagation",
        goal: "Tick moves resources along edges respecting `max_rate` and `throughput_limit`.",
        runtime_check: "Edge saturation reduces downstream intake in test.",
        failure_mode: "Instant global resource teleport.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I2-07",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "StarvationCascade",
        goal: "Starved refinery → smelter stall witness (emergent failure loop).",
        runtime_check: "Test: cut refinery input → smelter `current_efficiency` or output drops.",
        failure_mode: "No second-order industrial failure.",
    },
    // ── I3 Grid stress + placeable distribution ───────────────────────────
    Stage5LiveTodo {
        id: "INDUSTRIAL-I3-01",
        status: TodoStatus::Done,
        file: "src/entities/production/power/grid_topology.rs",
        system: "GridMembership",
        goal: "Activated industrial buildings increase `ElectricalGrid.total_load` within radius.",
        runtime_check: "Smelter near transformer host raises grid total in sim.",
        failure_mode: "Loads exist but grid rebuild ignores them.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I3-02",
        status: TodoStatus::Done,
        file: "src/entities/production/power/",
        system: "GridOverloadGameplay",
        goal: "Overload / brownout when cluster load exceeds transformer capacity.",
        runtime_check: "`GridOverloadEvent` after N smelters on one bus.",
        failure_mode: "Load exists but no strategic failure feedback.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I3-03",
        status: TodoStatus::Done,
        file: "assets/configs/buildings/",
        system: "TransformerCatalog",
        goal: "Placeable transformer + substation JSON in utilities catalog.",
        runtime_check: "Registry loads defs; industrial menu or utilities lists them.",
        failure_mode: "Transformers only in asset editor — not in game placement.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I3-04",
        status: TodoStatus::Done,
        file: "src/economy/supply_chain.rs",
        system: "TransformerActivation",
        goal: "Operational transformer/substation sites spawn `TransformerComponent` / substation runtime.",
        runtime_check: "Place utility → Operational → grid topology node exists.",
        failure_mode: "Distribution remains abstract.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I3-05",
        status: TodoStatus::Done,
        file: "src/entities/production/power/",
        system: "CapacityBottleneck",
        goal: "Transformers as capacity bottlenecks (overload, stress), not decoration.",
        runtime_check: "Witness: thermal/overload state changes under smelter cluster.",
        failure_mode: "Power system cosmetic only.",
    },
    // ── I4 Logistics physicalization ──────────────────────────────────────
    Stage5LiveTodo {
        id: "INDUSTRIAL-I4-01",
        status: TodoStatus::Done,
        file: "src/strategic/logistics/",
        system: "LogisticsNodeRegistration",
        goal: "Activated facility registers on `LogisticsGraph` (spatial node).",
        runtime_check: "Node entity id retrievable from site after Operational.",
        failure_mode: "Production without physical movement.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I4-02",
        status: TodoStatus::Done,
        file: "src/economy/",
        system: "ConcreteBatchStub",
        goal: "Concrete batch: mix, move cost, cure stub per industry runbook.",
        runtime_check: "Component + one integration test for batch state.",
        failure_mode: "Concrete is instant global resource.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I4-03",
        status: TodoStatus::Done,
        file: "src/economy/resource_flow.rs",
        system: "LogisticsPathRequired",
        goal: "`ResourceFlowEdge` requires valid logistics/rail/road path (no teleport).",
        runtime_check: "Blocked edge → zero transfer in test.",
        failure_mode: "Resources ignore geography.",
    },
    Stage5LiveTodo {
        id: "INDUSTRIAL-I4-04",
        status: TodoStatus::Done,
        file: "src/entities/production/power/grid_topology.rs",
        system: "SpatialIndustrialDistrict",
        goal: "Clustered smelters stress one transformer — localized district gameplay.",
        runtime_check: "Test: two smelters same tile region vs separated — load concentration differs.",
        failure_mode: "Industry load globally averaged.",
    },
    // ── GOV Anti-collapse ─────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "INDUSTRIAL-GOV-01",
        status: TodoStatus::Done,
        file: "src/construction/building_definitions.rs",
        system: "NoMegaFactoryCollapse",
        goal: "New industrial JSON must declare `supply_chain_role` or explicit `integrated_plant`.",
        runtime_check: "Unit test or loader warning on productive Industry def missing role.",
        failure_mode: "Chains re-merge into undifferentiated mega-buildings.",
    },
];

#[derive(Resource, Clone, Debug, Default)]
pub struct IndustrialActivationWitness {
    pub catalog_id_on_commit: bool,
    pub activation_system: bool,
    pub electrical_load_from_def: bool,
    pub activation_test: bool,
    pub proof_json: bool,
    pub power_plant_activation: bool,
    // Supply chain (SC)
    pub supply_chain_index: bool,
    pub supply_chain_catalog_complete: bool,
    pub role_based_activation: bool,
    pub chain_grouped_menu: bool,
    pub geopolymer_path: bool,
    pub aluminum_four_steps: bool,
    pub supply_chain_membership: bool,
    pub power_asymmetry_test: bool,
    // Resource flow (I2)
    pub resource_flow_node: bool,
    pub resource_flow_edge: bool,
    pub register_node_on_activate: bool,
    pub resource_type_mapping: bool,
    pub facility_inventory: bool,
    pub throughput_propagation: bool,
    pub starvation_cascade: bool,
    // Grid (I3)
    pub grid_membership: bool,
    pub grid_overload_hook: bool,
    pub transformer_catalog: bool,
    pub transformer_activation: bool,
    pub capacity_bottleneck: bool,
    // Logistics (I4)
    pub logistics_node: bool,
    pub concrete_batch_stub: bool,
    pub logistics_path_required: bool,
    pub spatial_industrial_district: bool,
    // Governance
    pub no_mega_factory_collapse: bool,
}

#[derive(Resource, Default)]
pub struct IndustrialActivationTodoBoard {
    pub status: Vec<TodoStatus>,
}

/// Per-row closure from witness + repo artifacts (assessment-driven).
#[must_use]
pub fn industrial_activation_todo_predicate(id: &str, w: &IndustrialActivationWitness) -> bool {
    match id {
        "INDUSTRIAL-I1-01" => w.catalog_id_on_commit,
        "INDUSTRIAL-I1-02" => w.activation_system,
        "INDUSTRIAL-I1-03" => w.electrical_load_from_def,
        "INDUSTRIAL-I1-04" => w.activation_test,
        "INDUSTRIAL-I1-05" => w.proof_json,
        "INDUSTRIAL-I1-06" => w.power_plant_activation,
        "INDUSTRIAL-SC-01" => w.supply_chain_index,
        "INDUSTRIAL-SC-02" => w.supply_chain_catalog_complete,
        "INDUSTRIAL-SC-03" => w.role_based_activation,
        "INDUSTRIAL-SC-04" => w.chain_grouped_menu,
        "INDUSTRIAL-SC-05" => w.geopolymer_path,
        "INDUSTRIAL-SC-06" => w.aluminum_four_steps,
        "INDUSTRIAL-SC-07" => w.supply_chain_membership,
        "INDUSTRIAL-SC-08" => w.power_asymmetry_test,
        "INDUSTRIAL-I2-01" => w.resource_flow_node,
        "INDUSTRIAL-I2-02" => w.resource_flow_edge,
        "INDUSTRIAL-I2-03" => w.register_node_on_activate,
        "INDUSTRIAL-I2-04" => w.resource_type_mapping,
        "INDUSTRIAL-I2-05" => w.facility_inventory,
        "INDUSTRIAL-I2-06" => w.throughput_propagation,
        "INDUSTRIAL-I2-07" => w.starvation_cascade,
        "INDUSTRIAL-I3-01" => w.grid_membership,
        "INDUSTRIAL-I3-02" => w.grid_overload_hook,
        "INDUSTRIAL-I3-03" => w.transformer_catalog,
        "INDUSTRIAL-I3-04" => w.transformer_activation,
        "INDUSTRIAL-I3-05" => w.capacity_bottleneck,
        "INDUSTRIAL-I4-01" => w.logistics_node,
        "INDUSTRIAL-I4-02" => w.concrete_batch_stub,
        "INDUSTRIAL-I4-03" => w.logistics_path_required,
        "INDUSTRIAL-I4-04" => w.spatial_industrial_district,
        "INDUSTRIAL-GOV-01" => w.no_mega_factory_collapse,
        _ => false,
    }
}

impl IndustrialActivationTodoBoard {
    pub fn sync_from_witness(&mut self, w: &IndustrialActivationWitness) {
        debug_assert_eq!(self.status.len(), INDUSTRIAL_ACTIVATION_TODO_COUNT);
        debug_assert_eq!(INDUSTRIAL_ACTIVATION_TODOS.len(), INDUSTRIAL_ACTIVATION_TODO_COUNT);
        for (slot, row) in self.status.iter_mut().zip(INDUSTRIAL_ACTIVATION_TODOS.iter()) {
            *slot = if industrial_activation_todo_predicate(row.id, w) {
                TodoStatus::Done
            } else {
                TodoStatus::Open
            };
        }
    }

    #[must_use]
    pub fn is_green(&self) -> bool {
        self.open_count() == 0
    }

    #[must_use]
    pub fn open_count(&self) -> usize {
        self.status.iter().filter(|s| **s == TodoStatus::Open).count()
    }
}

pub fn register_industrial_activation_todo_hooks(app: &mut App) {
    app.init_resource::<IndustrialActivationTodoBoard>()
        .init_resource::<IndustrialActivationWitness>();
    let mut board = IndustrialActivationTodoBoard::default();
    board.status = vec![TodoStatus::Open; INDUSTRIAL_ACTIVATION_TODO_COUNT];
    app.insert_resource(board);
}

pub fn sync_industrial_activation_board_from_witness(
    witness: &IndustrialActivationWitness,
    board: &mut IndustrialActivationTodoBoard,
) {
    board.sync_from_witness(witness);
    if board.is_green() {
        info!(
            target: "industrial_activation_todos",
            "INDUSTRIAL_ACTIVATION_GREEN"
        );
    }
}
