//! Construction **round 2** — feel / flow / city-builder UX (post authority + phase 2).

//!

//! Plan: [`super::construction_round2_plan.md`](super::construction_round2_plan.md)

//! Spec: [`super::recovery_construction.md`](super::recovery_construction.md) § Round 2.



use bevy::prelude::{App, Resource};



use super::construction_live_todos::TodoStatus;

use super::stage5_live_todos::Stage5LiveTodo;



pub const CONSTRUCTION_ROUND2_TODO_COUNT: usize = 15;



pub static CONSTRUCTION_ROUND2_TODOS: &[Stage5LiveTodo] = &[

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-01",

        status: TodoStatus::Done,

        file: "src/construction/sessions.rs",

        system: "ActiveToolSession",

        goal: "Persistent tool session after commit; continuous_mode + keep_tool_after_commit.",

        runtime_check: "Place 3 buildings or road segments without re-opening toolbox.",

        failure_mode: "Every commit drops tool back to None.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-02",

        status: TodoStatus::Done,

        file: "src/construction/roads/input.rs",

        system: "BuildHoverPipeline",

        goal: "Cursor → tile → ghost → validation → overlay every frame before UI.",

        runtime_check: "Ghost tracks cursor with zero dead frames while tool active.",

        failure_mode: "Ghost only updates on click.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-03",

        status: TodoStatus::Done,

        file: "src/construction/roads/commit.rs",

        system: "ContinuousRoadDraw",

        goal: "Road tool stays alive after commit for district-scale chaining.",

        runtime_check: "Commit road segment → placement session continues without toolbox click.",

        failure_mode: "Road tool resets after each commit.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-04",

        status: TodoStatus::Done,

        file: "src/construction/snap.rs",

        system: "SmartSnap",

        goal: "SnapTarget magnetism for road/rail nodes, grid, intersections.",

        runtime_check: "Endpoint snaps within threshold; tangent alignment on roads.",

        failure_mode: "Only raw grid snap.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-05",

        status: TodoStatus::Done,

        file: "src/construction/ghost_visual.rs",

        system: "GhostVisualLanguage",

        goal: "Soft transparent valid/invalid/pending/committed ghost colors (not debug flat).",

        runtime_check: "Visual proof or screenshot witness: distinct states without editor flat lines.",

        failure_mode: "Single-color debug overlay.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-06",

        status: TodoStatus::Done,

        file: "src/construction/building_catalog.rs",

        system: "BuildingCatalogUX",

        goal: "Residential submenu + intent panel use catalog (duplex/quadplex/units/footprint/cost) — no land/housing value.",

        runtime_check: "Select Duplex → preview shows unit mix + footprint + construction cost only.",

        failure_mode: "Only ZoneTool density; economy fluff in panel.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-07",

        status: TodoStatus::Done,

        file: "src/construction/build_interaction.rs",

        system: "PlacementBrush",

        goal: "PlacementBrushMode line/rectangle/paint for rows (housing, poles, farms).",

        runtime_check: "Drag paint creates multiple pending entries in one gesture.",

        failure_mode: "Single-click only.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-08",

        status: TodoStatus::Done,

        file: "src/construction/roads/intersections.rs",

        system: "IntersectionNode",

        goal: "IntersectionNode registry for connected segments.",

        runtime_check: "Crossing roads register shared intersection entity.",

        failure_mode: "Emergent overlap only.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-09",

        status: TodoStatus::Done,

        file: "src/construction/tool_hints.rs",

        system: "ToolHints",

        goal: "Bottom-left tool hint overlay (LMB/RMB/Shift/ESC).",

        runtime_check: "Hints visible per active BuildTool.",

        failure_mode: "No in-world guidance.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-10",

        status: TodoStatus::Done,

        file: "src/construction/build_confidence.rs",

        system: "BuildConfidence",

        goal: "BuildConfidence gradient (Perfect/Good/Risky/Invalid) from terrain/overlap/access/utilities.",

        runtime_check: "Risky placement shows amber confidence, not binary red only.",

        failure_mode: "Valid/invalid only.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-11",

        status: TodoStatus::Done,

        file: "src/construction/zones/input.rs",

        system: "ZoneBrushPersistence",

        goal: "Zone paint continuous brush without per-tile confirm friction.",

        runtime_check: "Paint large residential zone in one drag session.",

        failure_mode: "Per-tile queue confirm.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-12",

        status: TodoStatus::Done,

        file: "src/construction/build_toolbox.rs",

        system: "HierarchicalToolbox",

        goal: "Nested Construction / Zoning / Buildings / Editing categories per recovery spec.",

        runtime_check: "Toolbox tree matches recovery § hierarchical layout.",

        failure_mode: "Flat button list only.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-13",

        status: TodoStatus::Done,

        file: "src/construction/",

        system: "ConstructionHistory",

        goal: "ConstructionHistory undo stack for last N actions.",

        runtime_check: "Undo reverses last committed build in session.",

        failure_mode: "No undo.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-14",

        status: TodoStatus::Done,

        file: "src/construction/",

        system: "ConstructionPhases",

        goal: "ConstructionPhase visuals on sites (Surveying → Complete).",

        runtime_check: "Active site shows phase marker during build ticks.",

        failure_mode: "Instant pop-in only.",

    },

    Stage5LiveTodo {

        id: "CONSTRUCTION-R2-15",

        status: TodoStatus::Done,

        file: "src/construction/rail/",

        system: "RailSplineAuthority",

        goal: "Rail distinct from road clone: radius, slope, switches.",

        runtime_check: "Rail commit uses rail-specific validation not road spline only.",

        failure_mode: "Rail is road path duplicate.",

    },

];



#[derive(Resource, Clone, Debug, Default)]

pub struct ConstructionRound2Witness {

    pub tool_session: bool,

    pub hover_pipeline: bool,

    pub continuous_road: bool,

    pub smart_snap: bool,

    pub ghost_visual_language: bool,

    pub building_catalog_ux: bool,

    pub placement_brush: bool,

    pub intersection_registry: bool,

    pub tool_hints: bool,

    pub build_confidence: bool,

    pub zone_brush_persistence: bool,

    pub hierarchical_toolbox: bool,

    pub construction_history: bool,

    pub construction_phases: bool,

    pub rail_spline_authority: bool,

}



#[derive(Resource, Default)]

pub struct ConstructionRound2TodoBoard {

    pub status: Vec<TodoStatus>,

}



impl ConstructionRound2TodoBoard {

    pub fn sync_from_witness(&mut self, w: &ConstructionRound2Witness) {

        let flags = [

            w.tool_session,

            w.hover_pipeline,

            w.continuous_road,

            w.smart_snap,

            w.ghost_visual_language,

            w.building_catalog_ux,

            w.placement_brush,

            w.intersection_registry,

            w.tool_hints,

            w.build_confidence,

            w.zone_brush_persistence,

            w.hierarchical_toolbox,

            w.construction_history,

            w.construction_phases,

            w.rail_spline_authority,

        ];

        debug_assert_eq!(flags.len(), CONSTRUCTION_ROUND2_TODO_COUNT);

        for (slot, ok) in self.status.iter_mut().zip(flags) {

            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };

        }

    }



    #[must_use]

    pub fn open_count(&self) -> usize {

        self.status.iter().filter(|s| **s == TodoStatus::Open).count()

    }

}



pub fn register_construction_round2_todo_hooks(app: &mut App) {

    app.init_resource::<ConstructionRound2TodoBoard>()

        .init_resource::<ConstructionRound2Witness>();

    let mut board = ConstructionRound2TodoBoard::default();

    board.status = vec![TodoStatus::Open; CONSTRUCTION_ROUND2_TODO_COUNT];

    app.insert_resource(board);

}



pub fn sync_construction_round2_board_from_witness(
    witness: &ConstructionRound2Witness,
    board: &mut ConstructionRound2TodoBoard,
) {
    board.sync_from_witness(witness);
}


