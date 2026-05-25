//! Construction **round 3** — catalog, topology, visual authority, performance (post Phase 2 + operational green).
//!
//! Plan: [`super::construction_round3_plan.md`](super::construction_round3_plan.md)
//! Invariants: [`super::construction_invariants.md`](super::construction_invariants.md)
//! Spec: [`super::recovery_construction.md`](super::recovery_construction.md) § Round 3 (line 962+).
//!
//! **Prerequisite:** Phase 2 P6–P8 green. **Not** Stage 5.

use bevy::log::info;
use bevy::prelude::{App, Resource};

use super::construction_live_todos::TodoStatus;
use super::stage5_live_todos::Stage5LiveTodo;

pub const CONSTRUCTION_ROUND3_TODO_COUNT: usize = 27;

pub static CONSTRUCTION_ROUND3_TODOS: &[Stage5LiveTodo] = &[
    // ── R3-A Catalog runtime ─────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-A01",
        status: TodoStatus::Done,
        file: "src/construction/building_definitions.rs",
        system: "BuildingDefinitionLoader",
        goal: "`BuildingDefinition` serde type + load `assets/configs/buildings/*` (RON/JSON dispatch).",
        runtime_check: "Startup or on-demand load populates registry with ≥1 real asset file.",
        failure_mode: "Duplex/footprint still hardcoded in `building_catalog.rs` only.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-A02",
        status: TodoStatus::Done,
        file: "src/construction/building_definitions.rs",
        system: "BuildingDefinitionRegistry",
        goal: "Resource `BuildingDefinitionRegistry` — lookup by id, index from `_building_types_index.json`.",
        runtime_check: "Toolbox pick resolves def by id; missing id fails validation not silent stub.",
        failure_mode: "No central registry; scattered string ids.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-A03",
        status: TodoStatus::Done,
        file: "src/construction/residential_menu.rs",
        system: "CatalogToolboxWiring",
        goal: "Structure picks (Duplex, Quadplex, …) set `building_intent` from registry defs not `default_preview_for_apartment` only.",
        runtime_check: "Change JSON construction_cost → intent panel updates in app.",
        failure_mode: "UI and assets disconnected.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-A04",
        status: TodoStatus::Done,
        file: "src/construction/build_footprint_overlay.rs",
        system: "CatalogFootprintGhost",
        goal: "Ghost footprint from def matrix / building_size_x/y; intent panel shows workers, power, water from def.",
        runtime_check: "Non-2×2 asset footprint renders correct ghost tiles.",
        failure_mode: "All buildings use 2×2 default.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-A05",
        status: TodoStatus::Done,
        file: "src/construction/build_interaction.rs",
        system: "CatalogCommitArchetype",
        goal: "Commit uses `BuildingDefinition` archetype + footprint; no land/housing value fields.",
        runtime_check: "Factory JSON → Factory site; WaterPlant JSON → WaterPlant site.",
        failure_mode: "Every building still maps to Housing/Factory stub.",
    },
    // ── R3-B Transport topology ────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-B01",
        status: TodoStatus::Done,
        file: "src/construction/roads/intersections.rs",
        system: "IntersectionIdMap",
        goal: "`IntersectionId` + `HashMap` registry; stable ids at tile crossings.",
        runtime_check: "Registry API insert/lookup by tile; unit test two roads → one node.",
        failure_mode: "Vec stub only; no dedupe at crossing.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-B02",
        status: TodoStatus::Done,
        file: "src/construction/construction_pipeline.rs",
        system: "IntersectionCommitHook",
        goal: "Road/rail execute registers intersection when segment endpoint meets existing network node.",
        runtime_check: "Crossing commit increments `IntersectionRegistry` node degree ≥2.",
        failure_mode: "Segments only; graph topology absent.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-B03",
        status: TodoStatus::Done,
        file: "src/construction/roads/intersections.rs",
        system: "SegmentIntersectionLink",
        goal: "Segments (or plan ids) linked to `IntersectionNode::connected_segments`.",
        runtime_check: "Undo road segment updates intersection membership.",
        failure_mode: "Orphan segments after undo/commit.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-B04",
        status: TodoStatus::Done,
        file: "src/construction/roads/intersections.rs",
        system: "IntersectionQueryApi",
        goal: "Query neighbors / connected segments at tile for future traffic/pathfinding.",
        runtime_check: "Public API returns segment list at intersection; documented in plan.",
        failure_mode: "No query surface; consumers reach into registry internals.",
    },
    // ── R3-C Visual authority ────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-C01",
        status: TodoStatus::Done,
        file: "src/construction/visual_authority.rs",
        system: "ConstructionVisualRequest",
        goal: "Buffer `ConstructionVisualRequest` (path polyline, zone tiles, footprint rect) per frame.",
        runtime_check: "Road ghost draw reads requests not ad-hoc placement res only.",
        failure_mode: "Each tool owns egui layer ids independently.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-C02",
        status: TodoStatus::Done,
        file: "src/construction/visual_authority.rs",
        system: "UnifiedConstructionDraw",
        goal: "Single egui pass draws all construction previews from request buffer.",
        runtime_check: "Disable road draw system → only unified pass renders paths.",
        failure_mode: "Duplicate ghosts from road + rail + zone systems.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-C03",
        status: TodoStatus::Done,
        file: "src/dev/construction_ownership.md",
        system: "ViewportVisualBoundary",
        goal: "Document frame order + viewport/representation boundary; no construction-owned camera/hole latch.",
        runtime_check: "Doc lists systems + mutation rights; linked from invariants.",
        failure_mode: "Construction mutates viewport authority.",
    },
    // ── R3-D Brush systems ───────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-D01",
        status: TodoStatus::Done,
        file: "src/construction/sessions.rs",
        system: "PlacementBrushMode",
        goal: "`PlacementBrushMode` on session or tool: Single, Line, Rectangle, Paint.",
        runtime_check: "Toolbox or key cycle switches brush; hint overlay shows mode.",
        failure_mode: "Only single-tile and alt-drag.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-D02",
        status: TodoStatus::Done,
        file: "src/construction/build_interaction.rs",
        system: "BuildingLineBrush",
        goal: "Line brush queues multiple building pending entries along drag axis.",
        runtime_check: "Drag row → N pending blueprints without N separate clicks.",
        failure_mode: "Alt-drag only; no axis-aligned row.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-D03",
        status: TodoStatus::Done,
        file: "src/construction/zones/input.rs",
        system: "ZoneRectangleBrush",
        goal: "Rectangle brush paints filled zone tile set on drag release.",
        runtime_check: "Drag rect → painted area queued; matches brush mode.",
        failure_mode: "Freehand paint only.",
    },
    // ── R3-E Undo / history ─────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-E01",
        status: TodoStatus::Done,
        file: "src/construction/history.rs",
        system: "DemolishUndo",
        goal: "Demolish undo restores despawned site snapshot or documents blocked undo with reason.",
        runtime_check: "Demolish → Ctrl+Z → site entity restored OR explicit UI message.",
        failure_mode: "Demolish irreversible silently.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-E02",
        status: TodoStatus::Done,
        file: "src/construction/history.rs",
        system: "ConstructionRedo",
        goal: "Redo stack mirrors undo (Ctrl+Y); max depth matches undo.",
        runtime_check: "Undo then redo restores committed state.",
        failure_mode: "Undo-only stack.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-E03",
        status: TodoStatus::Done,
        file: "src/construction/history.rs",
        system: "HistoryActionLabels",
        goal: "Undo records typed actions (road/rail/site/zone/demolish) for debug + proof JSON.",
        runtime_check: "Proof JSON includes last_action_kind after commit/undo.",
        failure_mode: "Opaque undo stack.",
    },
    // ── R3-F Rail expansion ─────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-F01",
        status: TodoStatus::Done,
        file: "src/construction/rail/",
        system: "RailSwitchNode",
        goal: "Switch node placement stub (control point type or tool sub-mode).",
        runtime_check: "Rail tool can place switch; registered in junction authority.",
        failure_mode: "Rail is only point-to-point spline.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-F02",
        status: TodoStatus::Done,
        file: "src/construction/rail/",
        system: "JunctionAuthority",
        goal: "`RailJunctionAuthority` resource links switches + intersections.",
        runtime_check: "Switch commit creates junction record queryable by id.",
        failure_mode: "No junction model.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-F03",
        status: TodoStatus::Done,
        file: "src/construction/live_proof.rs",
        system: "RailTopologyProof",
        goal: "Proof JSON distinguishes rail markers/commits from road in witness flags.",
        runtime_check: "construction_stage_live.json has rail_segment_count or equivalent.",
        failure_mode: "Rail invisible in observability.",
    },
    // ── R3-G Performance ────────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-G01",
        status: TodoStatus::Done,
        file: "src/construction/",
        system: "PreviewPooling",
        goal: "No per-frame `commands.spawn` for ghosts; reuse preview state / pooled handles.",
        runtime_check: "Profile or test: ghost systems do not increase entity count every frame.",
        failure_mode: "ECS churn on mouse move.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-G02",
        status: TodoStatus::Done,
        file: "src/construction/roads/pathing.rs",
        system: "IncrementalPathPreview",
        goal: "Path preview rebuild appends from last control point only when possible.",
        runtime_check: "Adding one point does not re-sample entire chain from scratch (logged or tested).",
        failure_mode: "Full chain rebuild every cursor move.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-G03",
        status: TodoStatus::Done,
        file: "src/construction/zones/ghost.rs",
        system: "BatchedZoneOverlay",
        goal: "Zone paint draws all painted tiles in one batched egui pass.",
        runtime_check: "100+ painted tiles: single painter batch, acceptable frame time.",
        failure_mode: "Per-tile draw calls.",
    },
    // ── R3-H Governance ─────────────────────────────────────────────────────
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-H01",
        status: TodoStatus::Done,
        file: "src/dev/construction_invariants.md",
        system: "InvariantsInAgents",
        goal: "AGENTS.md links construction invariants; agents treat as hard rules.",
        runtime_check: "AGENTS.md contains link to construction_invariants.md.",
        failure_mode: "Invariants doc orphaned.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-H02",
        status: TodoStatus::Done,
        file: "src/dev/construction_ownership.md",
        system: "OwnershipDoc",
        goal: "Frame order diagram: tool sync → preview → validation → UI → execute; mutation table.",
        runtime_check: "Doc exists and matches BuildPlanningPlugin system chains.",
        failure_mode: "Unclear who may mutate transport/sites.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-R3-H03",
        status: TodoStatus::Done,
        file: "src/construction/",
        system: "AuthorityAudit",
        goal: "Script or test: `rg` fails on construction placement outside `src/construction/` (allowlist dev/engine hooks).",
        runtime_check: "CI/local audit command documented in plan; zero violations.",
        failure_mode: "New gui helper spawns roads directly.",
    },
];

#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionRound3Witness {
    pub catalog_loader: bool,
    pub catalog_registry: bool,
    pub catalog_toolbox: bool,
    pub catalog_footprint: bool,
    pub catalog_commit: bool,
    pub intersection_map: bool,
    pub intersection_commit: bool,
    pub intersection_link: bool,
    pub intersection_query: bool,
    pub visual_request: bool,
    pub visual_unified_draw: bool,
    pub visual_viewport_doc: bool,
    pub brush_mode: bool,
    pub building_line_brush: bool,
    pub zone_rect_brush: bool,
    pub demolish_undo: bool,
    pub redo_stack: bool,
    pub history_labels: bool,
    pub rail_switch: bool,
    pub rail_junction: bool,
    pub rail_proof: bool,
    pub preview_pooling: bool,
    pub incremental_path: bool,
    pub batched_zone: bool,
    pub invariants_agents: bool,
    pub ownership_doc: bool,
    pub authority_audit: bool,
}

#[derive(Resource, Default)]
pub struct ConstructionRound3TodoBoard {
    pub status: Vec<TodoStatus>,
}

impl ConstructionRound3TodoBoard {
    pub fn sync_from_witness(&mut self, w: &ConstructionRound3Witness) {
        let flags = [
            w.catalog_loader,
            w.catalog_registry,
            w.catalog_toolbox,
            w.catalog_footprint,
            w.catalog_commit,
            w.intersection_map,
            w.intersection_commit,
            w.intersection_link,
            w.intersection_query,
            w.visual_request,
            w.visual_unified_draw,
            w.visual_viewport_doc,
            w.brush_mode,
            w.building_line_brush,
            w.zone_rect_brush,
            w.demolish_undo,
            w.redo_stack,
            w.history_labels,
            w.rail_switch,
            w.rail_junction,
            w.rail_proof,
            w.preview_pooling,
            w.incremental_path,
            w.batched_zone,
            w.invariants_agents,
            w.ownership_doc,
            w.authority_audit,
        ];
        debug_assert_eq!(flags.len(), CONSTRUCTION_ROUND3_TODO_COUNT);
        for (slot, ok) in self.status.iter_mut().zip(flags) {
            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };
        }
    }

    #[must_use]
    pub fn open_count(&self) -> usize {
        self.status.iter().filter(|s| **s == TodoStatus::Open).count()
    }
}

pub fn register_construction_round3_todo_hooks(app: &mut App) {
    app.init_resource::<ConstructionRound3TodoBoard>()
        .init_resource::<ConstructionRound3Witness>();
    let mut board = ConstructionRound3TodoBoard::default();
    board.status = vec![TodoStatus::Open; CONSTRUCTION_ROUND3_TODO_COUNT];
    app.insert_resource(board);
}

pub fn sync_construction_round3_board_from_witness(
    witness: &ConstructionRound3Witness,
    board: &mut ConstructionRound3TodoBoard,
) {
    board.sync_from_witness(witness);
    let done = board.status.iter().filter(|s| **s == TodoStatus::Done).count();
    if done == CONSTRUCTION_ROUND3_TODOS.len() {
        info!(
            target: "construction_round3_todos",
            "CONSTRUCTION_ROUND3_COMPLETE done={done}/{}",
            CONSTRUCTION_ROUND3_TODOS.len()
        );
    }
}
