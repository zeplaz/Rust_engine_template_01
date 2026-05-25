//! Construction **phase 2** — complete remaining recovery work after BUILD-P* + FINISH-BUILD-*.
//!
//! Registry: [`CONSTRUCTION_PHASE2_TODOS`]. Witness: [`ConstructionPhase2Witness`].
//! Spec: [`super::construction_recovery_todos.md`](super::construction_recovery_todos.md) § Phase 2.
//!
//! **Not** Stage 5. Close rows via witness predicates + running-app checks (not template defaults alone).

use bevy::log::info;
use bevy::prelude::{App, Resource};

use super::construction_live_todos::TodoStatus;
use super::stage5_live_todos::Stage5LiveTodo;

/// Phase 2 rows — authority violations and stubs left after 2026-05 migration.
pub static CONSTRUCTION_PHASE2_TODOS: &[Stage5LiveTodo] = &[
    // ── P6 Authority & cleanup ─────────────────────────────────────────────
    Stage5LiveTodo {
        id: "PHASE2-BUILD-01",
        status: TodoStatus::Done,
        file: "src/gui/build/mod.rs",
        system: "Shim removal",
        goal: "Delete `gui::build` shim; all imports `crate::construction` only.",
        runtime_check: "Only `src/construction/` contains build sources; `rg gui::build` empty in src/.",
        failure_mode: "Dual import paths regress on every edit.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-02",
        status: TodoStatus::Done,
        file: "src/construction/demolish.rs",
        system: "DemolishExecute",
        goal: "Demolish intent → validate target → execute removes/despawns site or corridor (no MilitaryBase placeholder).",
        runtime_check: "LMB pick → pending demolish → approve → world entity/topology changes.",
        failure_mode: "Demolish queues fake housing blueprints.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-03",
        status: TodoStatus::Done,
        file: "src/construction/zones/",
        system: "ZoneStrategicCommit",
        goal: "Zone paint commit spawns `Zone` / residential overlay semantics — not mislabeled `CivilHousing` site rows.",
        runtime_check: "Shift+LMB zone commit → strategic zone overlay or dedicated pending type; housing uses building tool.",
        failure_mode: "Zoning and housing share one site archetype.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-04",
        status: TodoStatus::Done,
        file: "src/construction/construction_pipeline.rs",
        system: "Legacy tile roads",
        goal: "Remove `RoadConstructionDraft` tile-chain + `build_road_segment_intent_system` / confirm (path tool is sole road UX).",
        runtime_check: "rg build_road_segment_intent returns none; roads only via ActiveRoadPlacement.",
        failure_mode: "Two road placement models confuse operators and tests.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-05",
        status: TodoStatus::Done,
        file: "src/construction/build_tool_authority.rs",
        system: "BuildingArchetypeMap",
        goal: "`BuildingArchetypeId` maps to real `SiteArchetype` (Factory, WaterPlant, PowerPlant, …) — drop StubFactory default.",
        runtime_check: "Each toolbox building category commits correct archetype after approve.",
        failure_mode: "Every building places Factory stub.",
    },
    // ── P7 Tool UX (recovery § toolbox) ────────────────────────────────────
    Stage5LiveTodo {
        id: "PHASE2-BUILD-06",
        status: TodoStatus::Done,
        file: "src/construction/commercial/",
        system: "CommercialTool",
        goal: "Commercial submenu (office, retail stub) + building placement ghost.",
        runtime_check: "Toolbox Commercial → submenu → ghost → pending → commit.",
        failure_mode: "Commercial button sets Factory stub with no UX.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-07",
        status: TodoStatus::Done,
        file: "src/construction/industrial/",
        system: "IndustrialTool",
        goal: "Industrial submenu (factory, depot stub) + placement pipeline.",
        runtime_check: "Industrial tool end-to-end distinct from Commercial.",
        failure_mode: "Industrial duplicates Commercial/Factory.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-08",
        status: TodoStatus::Done,
        file: "src/construction/utilities/",
        system: "UtilitiesTool",
        goal: "Utilities submenu (power, water stub) → WaterPlant / PowerPlant archetypes.",
        runtime_check: "Utilities commit produces utility site archetypes.",
        failure_mode: "Utilities button is Factory stub.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-09",
        status: TodoStatus::Done,
        file: "src/construction/build_interaction.rs",
        system: "BuildingIntentPipeline",
        goal: "Building tools use intent→ghost→validate→pending only (zone/road/demolish input isolated).",
        runtime_check: "BuildTool::Building never shares zone paint or tile-road pick paths.",
        failure_mode: "Building placement still piggybacks civil tile ghost.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-10",
        status: TodoStatus::Done,
        file: "src/construction/rail/",
        system: "RailModule",
        goal: "Dedicated `rail/` module (types, width, popup copy) — reuses path solver but separate from `roads/`.",
        runtime_check: "Rail-only changes touch rail/; road regressions caught by road tests.",
        failure_mode: "Rail and road share one module with `#ifdef`-style branches.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-11",
        status: TodoStatus::Done,
        file: "src/construction/roads/popup.rs",
        system: "RoadCostEstimate",
        goal: "Road popup shows segment count + cost/length estimate from transport validator (stub formula OK).",
        runtime_check: "Popup Build disabled when any segment invalid; cost label updates live.",
        failure_mode: "Popup is static text; Build always enabled.",
    },
    // ── P8 Hardening & tests ───────────────────────────────────────────────
    Stage5LiveTodo {
        id: "PHASE2-BUILD-12",
        status: TodoStatus::Done,
        file: "src/construction/build_ghost.rs",
        system: "GhostPolicy",
        goal: "RULE 1: ghost is preview-only — document or remove `BuildGhostRoot` entity commit side effects.",
        runtime_check: "No CommitConstructionSiteEvent from ghost sync/pick except confirm path.",
        failure_mode: "Hidden entity mutation path bypasses pending queue.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-13",
        status: TodoStatus::Done,
        file: "src/construction/roads/",
        system: "RoadE2eTest",
        goal: "Integration test: 3 control points → validate → execute → `ExecutedRoadNetwork` / transport reflects chain.",
        runtime_check: "`cargo test` road e2e green; witness `road_e2e_integration`.",
        failure_mode: "Road slice only manually tested in app.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-14",
        status: TodoStatus::Done,
        file: "src/construction/zones/",
        system: "ZoneE2eTest",
        goal: "Integration test: paint tiles → pending zone entries → approve → overlay or zone component.",
        runtime_check: "`cargo test` zone e2e green.",
        failure_mode: "Zone paint unverified in CI.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-15",
        status: TodoStatus::Done,
        file: "src/construction/",
        system: "InputConflictMatrix",
        goal: "Shift+LMB / Alt+LMB / RMB semantics documented per tool; no cross-tool double-fire.",
        runtime_check: "Table in construction_recovery_todos.md + unit test for shift gate.",
        failure_mode: "Zone shift-commit fires building queue or road finalize.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-16",
        status: TodoStatus::Done,
        file: "debug_runs/construction_stage_live.json",
        system: "ConstructionProofJson",
        goal: "Running sim writes construction proof JSON (boards + witness flags) like stage5_full_app_live.",
        runtime_check: "debug_runs/construction_stage_live.json includes phase2 board snapshot.",
        failure_mode: "Construction closure not machine-verifiable.",
    },
    // ── P9 Advanced (recovery “Later” — after P6–P8 green) ───────────────
    Stage5LiveTodo {
        id: "PHASE2-BUILD-17",
        status: TodoStatus::Done,
        file: "src/construction/roads/spline.rs",
        system: "CurvedRoadSpline",
        goal: "Curved spline preview (not polyline only); mouse move updates tangent handle stub.",
        runtime_check: "Road ghost shows curve between points; commit samples polyline approximation.",
        failure_mode: "Highways always piecewise linear.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-18",
        status: TodoStatus::Done,
        file: "src/construction/snap.rs",
        system: "GridAndNodeSnap",
        goal: "Grid snap + snap to transport node / existing road endpoint (toggle in popup).",
        runtime_check: "Snap on: cursor locks to grid/node; off: free placement.",
        failure_mode: "Roads float off network graph.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-19",
        status: TodoStatus::Done,
        file: "src/construction/upgrade.rs",
        system: "RoadUpgradeLane",
        goal: "Upgrade road type / lane width on existing corridor (select → upgrade intent).",
        runtime_check: "Select executed road segment → upgrade enqueues plan.",
        failure_mode: "Must demolish and rebuild to widen.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-20",
        status: TodoStatus::Done,
        file: "src/construction/terrain_conform.rs",
        system: "TerrainConform",
        goal: "Road/site placement samples terrain height for ghost Y (conform stub).",
        runtime_check: "Ghost Y follows heightfield at tile; commit uses same sample.",
        failure_mode: "All previews at y=0 through slopes.",
    },
];

pub const CONSTRUCTION_PHASE2_TODO_COUNT: usize = CONSTRUCTION_PHASE2_TODOS.len();

#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionPhase2Witness {
    pub shim_removed: bool,
    pub demolish_execute: bool,
    pub zone_strategic_commit: bool,
    pub legacy_roads_removed: bool,
    pub building_archetype_map: bool,
    pub commercial_tool: bool,
    pub industrial_tool: bool,
    pub utilities_tool: bool,
    pub building_intent_pipeline: bool,
    pub rail_module: bool,
    pub road_cost_estimate: bool,
    pub ghost_policy: bool,
    pub road_e2e_integration: bool,
    pub zone_e2e_integration: bool,
    pub input_conflict_matrix: bool,
    pub construction_proof_json: bool,
    pub curved_road_spline: bool,
    pub grid_and_node_snap: bool,
    pub road_upgrade_lane: bool,
    pub terrain_conform: bool,
}

#[derive(Resource, Debug)]
pub struct ConstructionPhase2TodoBoard {
    pub status: Vec<TodoStatus>,
}

impl Default for ConstructionPhase2TodoBoard {
    fn default() -> Self {
        Self::from_template()
    }
}

impl ConstructionPhase2TodoBoard {
    #[must_use]
    pub fn from_template() -> Self {
        Self {
            status: CONSTRUCTION_PHASE2_TODOS
                .iter()
                .map(|t| t.status)
                .collect(),
        }
    }

    pub fn sync_from_witness(&mut self, w: &ConstructionPhase2Witness) {
        let flags = [
            w.shim_removed,
            w.demolish_execute,
            w.zone_strategic_commit,
            w.legacy_roads_removed,
            w.building_archetype_map,
            w.commercial_tool,
            w.industrial_tool,
            w.utilities_tool,
            w.building_intent_pipeline,
            w.rail_module,
            w.road_cost_estimate,
            w.ghost_policy,
            w.road_e2e_integration,
            w.zone_e2e_integration,
            w.input_conflict_matrix,
            w.construction_proof_json,
            w.curved_road_spline,
            w.grid_and_node_snap,
            w.road_upgrade_lane,
            w.terrain_conform,
        ];
        debug_assert_eq!(flags.len(), CONSTRUCTION_PHASE2_TODO_COUNT);
        for (slot, ok) in self.status.iter_mut().zip(flags) {
            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };
        }
    }

    #[must_use]
    pub fn open_count(&self) -> usize {
        self.status.iter().filter(|s| **s == TodoStatus::Open).count()
    }
}

pub fn register_construction_phase2_todo_hooks(app: &mut App) {
    app.init_resource::<ConstructionPhase2TodoBoard>()
        .init_resource::<ConstructionPhase2Witness>();
}

pub fn sync_construction_phase2_board_from_witness(
    witness: &ConstructionPhase2Witness,
    board: &mut ConstructionPhase2TodoBoard,
) {
    board.sync_from_witness(witness);
    let done = board
        .status
        .iter()
        .filter(|s| **s == TodoStatus::Done)
        .count();
    if done == CONSTRUCTION_PHASE2_TODOS.len() {
        info!(
            target: "construction_phase2_todos",
            "CONSTRUCTION_PHASE2_COMPLETE done={done}/{}",
            CONSTRUCTION_PHASE2_TODOS.len()
        );
    }
}
