Construction / Infrastructure Build System Recovery Plan

You need a unified Build Tool Authority System instead of:

scattered hotkeys
direct world placement
temporary ghost entities
separate road hacks
separate rail hacks
menu-specific logic

Current symptoms indicate:

UI and placement logic coupled
ghost logic duplicated
infrastructure pathing incomplete
no centralized build state machine
no tool authority ownership

The correct architecture is:

Build Toolbox
    ->
Selected Build Tool
    ->
Build Intent
    ->
Ghost Preview System
    ->
Validation
    ->
Commit Build
REQUIRED HIGH LEVEL FLOW
BUILD TOOLBOX

Single left-side toolbox:

[ Residential ]
[ Commercial ]
[ Industrial ]
[ Roads ]
[ Rail ]
[ Utilities ]
[ Demolish ]
BUILD STATE MACHINE
REQUIRED STATES
#[derive(Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum BuildMode {
    None,

    ZoneMenu,
    BuildingMenu,

    PlaceBuilding,

    RoadPlacement,
    RailPlacement,

    GhostPreview,

    ConfirmPlacement,
}
REQUIRED UI FLOW
1. CLICK RESIDENTIAL

Opens submenu:

Residential
    Low Density
    Medium Density
    High Density
    Apartments
    Mixed Use

Placeholder UI acceptable initially:

ui.button("Low Density");
ui.button("High Density");
2. SELECT BUILDING TYPE

Creates active tool:

#[derive(Resource)]
pub struct ActiveBuildTool {
    pub tool: BuildTool,
}
BUILD TOOL ENUM
#[derive(Debug, Clone)]
pub enum BuildTool {
    None,

    Zone(ZoneTool),

    Building(BuildingTool),

    Road(RoadTool),

    Rail(RailTool),

    Demolish,
}
BUILDING TOOL
#[derive(Debug, Clone)]
pub struct BuildingTool {
    pub archetype: BuildingArchetypeId,
}
ROAD TOOL
#[derive(Debug, Clone)]
pub struct RoadTool {
    pub road_type: RoadType,
}
GHOST PREVIEW PIPELINE

NEVER directly place into world.

Instead:

cursor
    ->
raycast
    ->
ghost preview
    ->
validation
    ->
commit
REQUIRED GHOST ENTITY
#[derive(Component)]
pub struct BuildGhost;

#[derive(Component)]
pub struct GhostValid(pub bool);
BUILD GHOST RESOURCE
#[derive(Resource, Default)]
pub struct ActiveGhostPath {
    pub points: Vec<Vec3>,
}
ROAD / RAIL FLOW

Current issue:
likely procedural placement trying to instantly finalize.

Correct flow:

Click
    add point

Move Mouse
    preview spline/path

Click
    confirm next point

Shift+Click
    finalize build

Right Click
    remove last point

Escape
    cancel entire ghost
REQUIRED INPUT MODEL
LEFT CLICK
add control point
RIGHT CLICK
remove previous point
SHIFT + LEFT CLICK
commit path
REQUIRED ROAD PIPELINE
Input Points
    ->
Path Solver
    ->
Curve Generator
    ->
Segment Generator
    ->
Ghost Mesh
    ->
Validation
    ->
Commit
REQUIRED DATA MODEL
ROAD PATH
#[derive(Resource, Default)]
pub struct ActiveRoadPlacement {
    pub control_points: Vec<Vec3>,
    pub generated_segments: Vec<RoadSegmentPreview>,
}
GENERATED SEGMENT
#[derive(Debug, Clone)]
pub struct RoadSegmentPreview {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub valid: bool,
}
REQUIRED TOOLBOX UI
src/gui/build_toolbox.rs
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::construction::*;

pub fn build_toolbox_ui(
    mut contexts: EguiContexts,
    mut tool: ResMut<ActiveBuildTool>,
) {
    egui::SidePanel::left("build_toolbox")
        .show(contexts.ctx_mut(), |ui| {

        ui.heading("Construction");

        if ui.button("Residential").clicked() {
            tool.tool = BuildTool::Zone(
                ZoneTool::ResidentialLow
            );
        }

        if ui.button("Road").clicked() {
            tool.tool = BuildTool::Road(
                RoadTool {
                    road_type: RoadType::Street,
                }
            );
        }

        if ui.button("Rail").clicked() {
            tool.tool = BuildTool::Rail(
                RailTool {
                    rail_type: RailType::Standard,
                }
            );
        }

        if ui.button("Demolish").clicked() {
            tool.tool = BuildTool::Demolish;
        }
    });
}
REQUIRED ROAD GHOST SYSTEM
src/construction/road_ghost.rs
use bevy::prelude::*;

use crate::construction::*;

pub fn update_road_ghost(
    active_tool: Res<ActiveBuildTool>,
    mut road: ResMut<ActiveRoadPlacement>,
    cursor: Res<WorldCursor>,
) {
    match &active_tool.tool {
        BuildTool::Road(_) => {

            if let Some(last) = road.control_points.last() {

                let preview = RoadSegmentPreview {
                    start: *last,
                    end: cursor.world,
                    width: 8.0,
                    valid: true,
                };

                if road.generated_segments.is_empty() {
                    road.generated_segments.push(preview);
                } else {
                    road.generated_segments[0] = preview;
                }
            }
        }
        _ => {}
    }
}
REQUIRED INPUT SYSTEM
src/construction/road_input.rs
pub fn road_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<WorldCursor>,
    mut road: ResMut<ActiveRoadPlacement>,
) {
    if buttons.just_pressed(MouseButton::Left) {

        if keys.pressed(KeyCode::ShiftLeft) {

            info!("Commit road build");

        } else {

            road.control_points.push(cursor.world);
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        road.control_points.pop();
    }
}
REQUIRED BUILD COMMIT SYSTEM

ONLY THIS SYSTEM SPAWNS REAL WORLD OBJECTS

ghost
    ->
validated
    ->
commit
REQUIRED DIRECTORY STRUCTURE
src/
    construction/
        mod.rs

        toolbox.rs
        state.rs

        ghost/
            building_ghost.rs
            road_ghost.rs
            rail_ghost.rs

        roads/
            input.rs
            pathing.rs
            spline.rs
            validation.rs
            commit.rs

        rail/
            input.rs
            spline.rs
            commit.rs

        ui/
            toolbox.rs
            residential_menu.rs
            road_popup.rs
REQUIRED ROAD POPUP

When road tool active:

Road Tool
--------------------------------
Current Type: Street
Segments: 12
Estimated Cost: 120

[ Build ]
[ Cancel ]
CRITICAL RULES
RULE 1

Ghosts are NOT world entities.

They are previews only.

RULE 2

Only commit system creates gameplay entities.

RULE 3

Road placement must be spline/path based.

NOT per-tile clicking.

RULE 4

UI never mutates world directly.

RULE 5

All construction flows through:

BuildToolAuthority
REQUIRED FUTURE SYSTEMS
Later Additions
bulldozer tool
elevated roads
bridges
tunnels
snapping
grid snapping
curved rail
parallel road placement
upgrade roads
lane editing
zoning paint
terrain conforming
IMMEDIATE PRIORITY TASKS
PHASE 1

Stabilize toolbox:

remove ; menu
create persistent toolbox
PHASE 2

Centralize build state:

ActiveBuildTool
BuildMode
PHASE 3

Implement unified ghost system

PHASE 4

Road spline/path preview

PHASE 5

Commit pipeline

MOST IMPORTANT CHANGE

Stop thinking in:

"place object immediately"

Everything should become:

intent
    ->
ghost
    ->
validate
    ->
commit

That shift will clean up:

roads
rail
zoning
buildings
future utilities
demolish
upgrades
drag placement
procedural placement
multiplayer sync later

---

## Building catalog (legacy asset tools → construction)

**Implementation mirror:** [`src/construction/building_catalog.rs`](../construction/building_catalog.rs)  
**Todo lane:** `CONSTRUCTION-R2-06` in [`construction_round2_todos.rs`](construction_round2_todos.rs)

### Zoning vs placed buildings

| Lane | Tool | Meaning |
|------|------|---------|
| **Zoning** | `ZoneTool` | District **paint** → strategic zone overlays (low/med/high, mixed use) |
| **Building** | `BuildingArchetypeId` + catalog forms | A **structure** with footprint matrix, units, cost/time, power/workers |

### Legacy sources (keep)

| Source | Content |
|--------|---------|
| `schemas/flatbuffers/skrukturave_01.fbs` | `ResidenceType`, `ApartmentUnitUnion`, `ApartmentUnitType` |
| `utils/asset_tools/src/templates/buildings.py` | `RESEDENCY_TYPES`, `APARTMENT_TYPES`, `APARTMENT_UNIT_TYPES`, `BUILDING_TYPES` |
| `utils/asset_tools/src/pages/buildings_pages.py` | Duplex max 2 units, Quadplex max 4, high-rise floors × units/floor |
| `assets/configs/buildings/_building_types_index.json` | Designer labels → engine `BuildingType` / factory / mine |
| `assets/configs/buildings/*.json` | `building_size_x/y`, height, `construction_cost`, power, produces/consumes |

### Residential taxonomy (from asset tools — do not drop)

**Detached:** Small House, Medium House, Large House, Estate

**Multi-unit (`ApartmentForm`):** High-rise, **Duplex** (2 units), **Quadplex** (4 units), Three-story block, Five-story block  
*(Colloquial “triplex” → use three-story block or Quadplex per asset data — there is no separate Triplex enum in legacy FBS.)*

**Unit kinds (`ApartmentUnitKind`):** Studio, Single, Double, ThreeBedrooms, Family, Luxury

### Non-residential families

Legacy `BUILDING_TYPES`: Depanneur, Burocracy, Field Depo, Warehouse, Factory, Mine, Fuel, Power, Research, Farm, Rail — align with [`BuildingFamily`](../construction/building_catalog.rs) and toolbox Commercial / Industrial / Utilities.

### Footprint matrix

Legacy `BuildingMatrixGrid`: occupancy grid drives ghost size (not only 2×2 defaults). Load from per-asset JSON when wiring confirm panel.

### Intent preview — show / hide

**Show:** footprint (tiles), unit mix / capacity, `construction_cost`, `construction_time`, power/water hooks, `workers_required`, produces/consumes where relevant.

**Do not show:** land value, housing value, abstract market grades — not operational build parameters.

**Example (Duplex):**

```text
Duplex — 2 units
-----------------
Footprint: 2×2 tiles
Units: Single + Single
Construction cost: 240
Build time: 120 ticks
Power: 12
Workers: 0
```

---

## Round 2 — Feel, flow, city-builder UX

**Board:** [`CONSTRUCTION-R2-*`](construction_round2_todos.rs) (15 rows). Authority / phase-2 lanes are separate; this is polish only.

**Assessment:** Missing authority and direct world mutation are largely solved. Remaining gap: friction, editor-like ghosts, weak tool momentum.

**Priority tiers:** Tier 1 — tool sessions, live cursor ghost, continuous roads, smart snap, visual cleanup. Tier 2 — drag brushes, intersections, hints, overlays, catalog-driven building preview. Tier 3 — undo, construction phases, rail spline authority, upgrade viz.

---

Round 2 notes (original reflection — item detail below)

Construction System Reflection + Improvement Plan

Current architecture is actually in a strong direction now.

The biggest remaining issues are no longer:

missing authority
scattered placement systems
direct world mutation

Those are mostly solved.

The current weakness is now:

interaction smoothness
player readability
flow continuity
visual cohesion
tool feel

You’ve transitioned from:

chaotic prototype

to:

correct but mechanical/editor-like

The next phase is:

make construction FEEL good
HIGHEST VALUE IMPROVEMENTS
1. UNIFY BUILD EXPERIENCE INTO “ACTIVE TOOL SESSION”

Right now tools likely feel:

click button
place thing
confirm
repeat

Instead construction should feel like:

enter mode
stay in mode
fluidly continue building
ADD TOOL SESSION RESOURCE
#[derive(Resource)]
pub struct ActiveToolSession {
    pub started_at: f64,
    pub actions_committed: usize,
    pub continuous_mode: bool,
    pub keep_tool_after_commit: bool,
}
RESULT

Player can:

place multiple buildings rapidly
continue roads seamlessly
continue zoning without reopening menus
commit chains quickly

This dramatically improves feel.

2. ADD TRUE CURSOR-CENTRIC BUILDING

Current likely flow:

cursor -> tile -> queue -> confirm

Needs:

cursor always previews live

At all times.

No dead interaction frames.

REQUIRED

Ghost update should happen:

every frame
before UI draw
before placement validation
ADD BUILD HOVER PIPELINE
Cursor
    ->
Hover Tile
    ->
Ghost Projection
    ->
Validation
    ->
Overlay
3. BUILDING PLACEMENT SHOULD SUPPORT DRAG PAINT

Huge gameplay gain.

CURRENT

Single-click buildings.

IMPROVED

Allow:

click + drag

For:

row housing
power poles
wall structures
farms
industrial strips
REQUIRED TYPE
pub enum PlacementBrushMode {
    Single,
    Line,
    Rectangle,
    Paint,
}
4. ROADS NEED “LIVE SMART SNAP”

Current roads probably still feel segmented.

Need:

automatic endpoint snap
intersection preview
angle assist
soft curvature
ADD SNAP PRIORITIES
pub enum SnapTarget {
    RoadNode(Entity),
    RailNode(Entity),
    Grid(Vec3),
    TerrainFeature(Vec3),
}
SMART SNAP BEHAVIOR

Roads should:

magnetize near intersections
automatically align tangent direction
preserve curve continuity
5. ROAD TOOL NEEDS “CONTINUOUS DRAW MODE”

Current likely:

click
click
shift click

Needs:

road stays alive after commit

Player should:

build an entire district continuously

WITHOUT:

reopening tool
rebuilding ghost state
6. VISUAL FEEDBACK IS CURRENTLY PROBABLY TOO DEBUG-LIKE

Likely symptoms:

plain lines
flat colors
editor feel
hard edges
too much text

Need:

soft transparent previews
animated validity
hover glow
build pulse
REQUIRED GHOST VISUALS
VALID
soft cyan / green
INVALID
red pulsing
PENDING QUEUE
orange
COMMITTED
construction yellow
7. ADD “BUILD CONFIDENCE” VISUALIZATION

Massive UX improvement.

CURRENT

Probably:

valid / invalid
IMPROVED

Gradient confidence:

terrain issue
overlap risk
access issue
infrastructure missing
TYPE
pub enum BuildConfidence {
    Perfect,
    Good,
    Risky,
    Invalid,
}
8. CONSTRUCTION SHOULD FEEL “PHYSICAL”

Currently likely abstract.

Need:

scaffold visuals
construction phases
build progress markers
road paving animation
ADD CONSTRUCTION STAGES
pub enum ConstructionPhase {
    Surveying,
    Groundwork,
    Framing,
    Finishing,
    Complete,
}
9. ZONING NEEDS BETTER FLOW

Currently:

tool
paint
queue
confirm

Needs:

paint continuously

Like:

SimCity
Cities Skylines
REQUIRED

Zone brush persistence.

Player should:

paint large areas fluidly
10. TOOLBOX NEEDS HIERARCHICAL TOOL ORGANIZATION

Right now likely too flat.

Need categories:

Construction
    Roads
    Rail
    Utilities

Zoning
    Residential
    Commercial
    Industrial

Buildings
    Civic
    Utility
    Decoration

Editing
    Demolish
    Upgrade
11. ADD “BUILD INTENT PREVIEW”

Before committing, show **catalog-backed operational fields** (see § Building catalog above):

- footprint tiles / matrix size
- unit mix or capacity (residential)
- construction cost and build time
- power / water / workers
- produces/consumes (industrial) when defined in asset JSON

**Do not** show land value, housing value, or generic “market grade” labels.

PANEL EXAMPLE (Duplex from asset tools)

```text
Duplex — 2 units
-----------------
Footprint: 2×2 tiles
Units: Single + Single
Construction cost: 240
Build time: 120 ticks
Power: 12
Workers: 0
```
12. ROAD INTERSECTIONS NEED THEIR OWN SYSTEM

Currently probably emergent.

This becomes unstable later.

REQUIRED
pub struct IntersectionNode {
    pub connected_segments: Vec<Entity>,
}
WHY IMPORTANT

Future:

traffic
lane logic
signals
pathfinding
zoning frontage

ALL depend on this.

13. RAIL SHOULD BECOME TRUE SPLINE TRACK

Not road clone.

Need:

wider curve radius
slope constraints
switch nodes
junction authority
14. ADD TOOL HINT OVERLAY

Bottom-left:

LMB: Add Point
RMB: Undo
Shift+LMB: Commit
ESC: Cancel

Huge usability gain.

15. ADD BUILD HISTORY / UNDO

Critical eventually.

RESOURCE
pub struct ConstructionHistory {
    pub actions: Vec<ConstructionAction>,
}
16. PREVIEW PERFORMANCE PIPELINE

Very important later.

Ghosts should:

never spawn meshes every frame
use pooled preview meshes
batch overlays
avoid ECS churn
17. YOU NEED A “CONSTRUCTION VISUAL AUTHORITY”

Right now viewport cleanup and construction visuals are likely disconnected.

Need:

construction tool
    ->
visual overlay authority
    ->
viewport render layer
RECOMMENDED NEW SUBSYSTEMS
construction/
    interaction/
    visuals/
    overlays/
    snapping/
    sessions/
    history/
MOST IMPORTANT GAMEPLAY CHANGE

Current system:

editor driven

Needs to become:

city-builder-flow driven

That means:

fewer confirmations
persistent tools
continuous placement
visual confidence
soft snapping
momentum
STRONGEST NEXT IMPROVEMENTS

Priority order:

TIER 1
persistent tool sessions
live cursor ghost
continuous road drawing
smart snapping
visual cleanup
TIER 2
drag building placement
road intersections
tool hints
better overlays
TIER 3
undo/history
construction phases
advanced rail
upgrade visualization
OVERALL ASSESSMENT

The architecture is now substantially healthier than before.

The major remaining problem is not correctness.

It is:

friction
visual roughness
editor-feel
temporary visuals
interaction momentum

The system now needs:

interaction polish
visual authority
continuous flow


Critical Observation

The engine is now entering a new risk category:

architectural drift through expansion

Meaning:

The current construction core is finally coherent enough that:

new features could easily re-fragment authority
new UX work could bypass queues
visual systems could start mutating gameplay
road/intersection systems could fork logic again

So the next stage should aggressively protect:

construction invariants
Recommended Immediate Priority
CLOSE PHASE 2 FIRST

You already identified the correct sequence:

P6
 -> P7
 -> P8
 -> P9

This order is correct because:

P6 = REMOVE LEGACY CONTAMINATION

Most important.

This closes:

old road remnants
fake zone commits
archetype mismatch
demolish shims

This stabilizes ontology.

Without this:
future systems inherit ambiguity.

P7 = NORMALIZE CONTENT FLOW

This transitions construction from:

hardcoded placement

toward:

catalog-driven runtime

Very important before scaling.

P8 = VERIFY SYSTEM INVARIANTS

This is the hidden critical phase.

Especially:

input conflict matrix
ghost policy
e2e validation

This prevents future entropy.

You should strongly formalize:

ownership boundaries
who may mutate what
frame order guarantees
P9 = PROOF / OBSERVABILITY

This is where the subsystem becomes:

maintainable

Without proof generation:
future refactors become dangerous.

Strong Recommendation

Add this:

construction_invariants.md

Document hard rules.

Example:

# Construction Invariants

1. Preview systems never mutate gameplay state
2. Only execute systems spawn committed entities
3. All roads enter through ConstructionPlanQueue
4. BuildToolAuthority is the sole active tool source
5. Ghost visuals are disposable
6. Zone paint never spawns structures directly
7. Validation must run before commit
8. Demolish requires pending confirmation
9. No construction logic outside src/construction/

This becomes extremely important later.

Round 3 Analysis

Your proposed Round 3 categories are exactly the correct evolution path.

Notably:

MOST IMPORTANT ROUND 3 ITEM
Catalog → Runtime

This is the largest future scalability unlock.

Right now archetypes are still semi-hardcoded.

You need:

assets/configs/buildings/

driving:

footprint
cost
variants
workers
pollution
utility demand
construction stages
meshes later
Recommended Runtime Model
BuildingDefinition
#[derive(Deserialize, Reflect, Clone)]
pub struct BuildingDefinition {
    pub id: String,
    pub display_name: String,

    pub footprint: UVec2,

    pub cost: u32,

    pub residents: u32,
    pub workers: u32,

    pub power_usage: f32,
    pub water_usage: f32,

    pub archetype: SiteArchetype,

    pub category: BuildingCategory,
}
INTERSECTIONS ARE THE NEXT HUGE ARCHITECTURAL POINT

You correctly identified this.

Currently roads are:

segments

But transport simulation eventually needs:

graph topology

Meaning:

nodes
intersections
lane authority
turn resolution
signals
pathfinding anchors

Without a true intersection registry:
future traffic systems become painful.

RECOMMENDED
IntersectionRegistry
#[derive(Resource, Default)]
pub struct IntersectionRegistry {
    pub nodes: HashMap<IntersectionId, IntersectionNode>,
}
VISUAL AUTHORITY IS THE OTHER BIG ONE

This connects directly to your viewport cleanup work.

Construction visuals should NOT:

directly render ad-hoc gizmos
spawn unmanaged overlays
own viewport state

Need:

construction
    ->
visual request
    ->
viewport visual authority
    ->
render extraction

Especially for:

pooled ghosts
overlays
path previews
zoning paint
road curves
PERFORMANCE WARNING

You are approaching the point where:

preview ECS churn

will become expensive.

Especially:

spline previews
zone overlays
drag paint
path rebuilds

So Round 3 should include:

preview pooling
incremental mesh updates
batched overlays

before construction scale increases.

Recommended Round 3 Structure
construction_round3_plan.md
R3-A Catalog Runtime
R3-B Transport Topology
R3-C Visual Authority
R3-D Brush Systems
R3-E Undo / History
R3-F Rail Expansion
R3-G Overlay Performance
R3-H Interaction Polish
VERY IMPORTANT NEXT RULE

You should now begin enforcing:

construction features MUST integrate into authority model

No more:

quick helper placement
temporary spawn paths
direct road injection
debug-only build modes

Every feature should go through:

tool
 -> intent
 -> preview
 -> validation
 -> queue
 -> execute

No exceptions.

Recommended Additional Gate

After Phase 2:

CONSTRUCTION_OPERATIONAL_GREEN

Separate from FULL_APP.

Requirements:

toolbox functional
road path commit functional
zone paint functional
building placement functional
demolish functional
undo functional
proof JSON generated
no legacy placement paths
no authority violations

That would be an excellent milestone.

Overall Assessment

You are now beyond:

prototype recovery

The subsystem is becoming:

real engine infrastructure

The next danger is no longer missing systems.

It is:

future entropy

So the most important things now are:

invariants
authority enforcement
topology correctness
catalog-driven runtime
visual authority unification
proof generation
subsystem boundaries

---

## Round 3 + operational gate (authoritative boards)

**Created from § Critical Observation (line 962+).** Round 2 (`CONSTRUCTION-R2-*`) is **done** — see [`construction_round2_plan.md`](construction_round2_plan.md).

### Prerequisite

1. Close **Phase 2** P6 → P7 → P8 → P9 ([`construction_phase2_todos.rs`](construction_phase2_todos.rs))  
2. Hit **CONSTRUCTION_OPERATIONAL_GREEN** ([`construction_operational_gate.md`](construction_operational_gate.md))  
3. Then execute **Round 3** ([`construction_round3_plan.md`](construction_round3_plan.md))

### Hard rules

[`construction_invariants.md`](construction_invariants.md) — no preview mutation, single execute funnel, catalog over hardcoded archetypes, intersection graph, visual authority boundary.

### Round 3 lanes (27× `CONSTRUCTION-R3-*`)

| Lane | Focus | Board IDs |
|------|--------|-----------|
| **R3-A** | Catalog runtime (`BuildingDefinition` ← `assets/configs/buildings/`) | A01–A05 |
| **R3-B** | Transport topology / intersections | B01–B04 |
| **R3-C** | Visual authority → viewport | C01–C03 |
| **R3-D** | Brush modes (line, rect, paint) | D01–D03 |
| **R3-E** | Undo++ (demolish, redo, labels) | E01–E03 |
| **R3-F** | Rail switches / junctions | F01–F03 |
| **R3-G** | Preview pooling / batching | G01–G03 |
| **R3-H** | Governance (invariants, ownership, audit) | H01–H03 |

Registry: [`construction_round3_todos.rs`](construction_round3_todos.rs)

### Operational green (8× `CONSTRUCTION-OP-*`)

Toolbox, road, zone, building, demolish, undo, proof JSON, no legacy paths — **separate from Stage 5 FULL_APP**.

Registry: [`construction_operational_todos.rs`](construction_operational_todos.rs)

### Ownership

[`construction_ownership.md`](construction_ownership.md) — frame order and mutation table for `BuildPlanningPlugin`.

---

## Phase 4 — Industrial activation (next major phase)

**Authoritative guide:** [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md)  
**Todo board:** [`industrial_activation_todos.rs`](industrial_activation_todos.rs) (**31×** `INDUSTRIAL-*`, exit gate `INDUSTRIAL_ACTIVATION_GREEN`)  
**Phase spec (§1811+):** [`industrial_activation_phase_todos.md`](industrial_activation_phase_todos.md)  
**Code spine (started):** `src/economy/activation/bridge.rs` — `IndustrialActivationPlugin`, `BuildingDefinitionRef`, activate on `Operational`.

Prerequisite: `CONSTRUCTION_OPERATIONAL_GREEN` (construction lane). Not Stage 5.

### Assessment summary (line 1323+ notes)

CURRENT REAL ARCHITECTURE
LAYER 1 — CONSTRUCTION (MOST COMPLETE)

This is now your strongest subsystem.

You have:

authority
placement
validation
queues
commit funnel
catalog loading
proof generation
operational gates
invariants

This is now:

production-grade foundation
LAYER 2 — INDUSTRIAL DEFINITIONS (GOOD SHAPE)

You already have:

JSON assets
taxonomy
runtime ECS types
config structs
partial plugins
domain separation

This is excellent.

You already avoided:

hardcoded monolithic industry logic

which is one of the biggest long-term mistakes.

LAYER 3 — LIVE ECONOMIC SIMULATION (MOSTLY MISSING)

This is the gap.

Right now:

construction
    ->
site entities

But NOT:

construction
    ->
operational industrial facility
    ->
resource throughput
    ->
logistics
    ->
power draw
    ->
economic effects

That missing bridge is now the core problem.

THIS IS ACTUALLY GOOD NEWS

Because:

The hard part was:

authority structure

And you now largely have that.

The current missing work is:

system integration

which is much more manageable.

WHAT YOU ALREADY HAVE (IMPORTANT)

You already possess the beginnings of:

CONCRETE INDUSTRY MODEL

You already modeled:

kilns
aggregate
mix plants
recipes
ratios
curing concepts
energy intensity

This is actually significantly deeper than most games.

ALUMINUM INDUSTRY MODEL

You already modeled:

bauxite
alumina refining
smelting
fabrication
cryolite/anodes
emissions
power intensity

This is extremely promising.

Most games stop at:

ore -> aluminum

You already started modeling:

industrial process chains

That’s the correct direction.

POWER SYSTEM

This is the most strategically important subsystem.

You already have:

plant archetypes
grid grouping
topology rebuild
transformers
substations
overload concepts
operational status
load/capacity

This is enough foundation for:

true infrastructure simulation
THE MAIN MISSING PIECE

This is the critical missing spine:

Construction
    ->
Operational Facility Activation
    ->
Resource Registration
    ->
Economic Throughput
    ->
Grid/Logistics Integration

Right now:

construction exists
production runtime exists

BUT:
they do not become the same entity lifecycle.

THE MOST IMPORTANT NEXT SYSTEM

You need:

IndustrialActivationBridge
WHAT IT SHOULD DO

When:

ConstructionSite
    ->
Complete

Then:

BuildingDefinition
    ->
spawn operational industrial bundle
THIS IS THE TRUE MISSING LINK

Right now the engine has:

construction ontology

and:

industrial ontology

But not:

construction -> industrial transition
RECOMMENDED PIPELINE
CURRENT
Construction
    ->
ConstructionSite
REQUIRED
Construction
    ->
ConstructionSite
    ->
Commissioning
    ->
Operational Facility
    ->
Economic Registration
    ->
Grid + Logistics Membership
THIS CREATES HUGE EMERGENT GAMEPLAY
EXAMPLE

Player builds:

aluminum smelter

Then automatically:

registers huge power demand
requires freight throughput
emits pollution
consumes labor
stresses substations
increases transformer demand
affects regional economy

This is the gameplay gold.

YOU ALREADY HAVE THE DATA

That’s the important realization.

The repo already contains:

industrial semantics
process semantics
topology concepts
runtime ECS

What’s missing is:

cross-system activation lifecycle
CURRENT ENGINE MATURITY
CONSTRUCTION
MATURE
INDUSTRIAL DEFINITIONS
ADVANCED PROTOTYPE
LIVE INDUSTRIAL ECONOMY
EARLY STUB
VERY IMPORTANT OBSERVATION

Your architecture already naturally supports:

capital-intensive industrial gameplay

because:

facilities are distinct
topology exists
logistics concepts exist
power exists
strategic construction exists

This is not a fake “resource counter” architecture.

That’s extremely important.

WHAT SHOULD HAPPEN NEXT
PRIORITY 1
CONSTRUCTION → OPERATIONAL BRIDGE

This is now the single highest leverage system.

RECOMMENDED FILE
src/economy/activation/
    construction_activation_bridge.rs
EXAMPLE
pub fn activate_completed_sites(
    mut commands: Commands,
    completed: Query<(
        Entity,
        &ConstructionSite,
        &BuildingDefinitionRef,
    ), Added<Operational>>,
) {
    for (entity, site, def_ref) in completed.iter() {

        match def_ref.id.as_str() {

            "concrete_basic_production_plant" => {

                commands.entity(entity)
                    .insert(CementKilnRuntime::default())
                    .insert(ConcreteMixerRuntime::default())
                    .insert(ElectricalComponent::default());
            }

            "aluminum_smelter1" => {

                commands.entity(entity)
                    .insert(AluminumSmelterRuntime::default())
                    .insert(ElectricalComponent {
                        demand_mw: 220.0,
                    });
            }

            _ => {}
        }
    }
}
PRIORITY 2
RESOURCE FLOW REGISTRY

Need:

who produces what
who consumes what
REQUIRED RESOURCE GRAPH
pub struct ResourceFlowNode {
    pub inputs: Vec<ResourceRate>,
    pub outputs: Vec<ResourceRate>,
}
PRIORITY 3
GRID STRESS GAMEPLAY

This is where your power system becomes truly strategic.

EXAMPLE

Aluminum smelter activates:

+220 MW load

Grid response:

overload
transformer stress
voltage instability
rolling brownouts

This creates:

infrastructure management gameplay
PRIORITY 4
LOGISTICS BECOMES PHYSICAL

Need:

resource movement

NOT:

global magic inventory
IMPORTANT FOR CONCRETE

Concrete should:

expire
be expensive to move
require local batching

That naturally creates:

regional industrial planning
YOU SHOULD LEAN INTO THIS

Because the repo already suggests the engine wants to become:

infrastructure civilization simulator

NOT merely:

city painter
VERY IMPORTANT WARNING

Do NOT let:

JSON assets become disconnected flavor data

They must become:

authoritative industrial semantics

Meaning:

production rates
energy use
labor
throughput
emissions
maintenance

must actually drive runtime systems.

CURRENT BEST SUBSYSTEM

Ironically:

construction

is now mature enough to serve as the backbone for:

economy
logistics
infrastructure
industrial activation

That’s a huge milestone.

MOST IMPORTANT MISSING DOC

You now need:

industrial_activation_pipeline.md

Define:

construction
    ->
commissioning
    ->
activation
    ->
grid registration
    ->
resource registration
    ->
logistics registration
    ->
maintenance registration

Without formalizing this:
the systems may drift apart again.

OVERALL ASSESSMENT

The codebase is no longer:

random prototype fragments

It is now:

multiple partially-complete major simulation domains

that are waiting for:

integration spines

Specifically:

construction spine
industrial activation spine
logistics spine
power topology spine
resource throughput spine

The exciting part is:

The difficult authority architecture is already substantially solved in construction.

Now the challenge becomes:

making the industrial world come alive

through:

operational activation
resource movement
grid stress
maintenance
throughput bottlenecks
strategic infrastructure dependency

Which is exactly the direction your current ECS/domain layout already hints at.


the ECS/runtime layer already contained:

differentiated industrial ontology
process specialization
per-stage infrastructure semantics

But:
the construction/catalog path had flattened them into:

single integrated mega-buildings

which erased:

logistics depth
grid dynamics
throughput staging
strategic geography
industrial scaling gameplay

So the work you just described is extremely important because it restores:

industrial granularity

which is the foundation for meaningful infrastructure gameplay.

WHAT ACTUALLY CHANGED (IMPORTANT)

Before:

runtime complexity existed

but:

player agency over the chain did not

Now:

the player can physically construct the industrial graph

That is a massive conceptual upgrade.

THIS IS THE CORRECT DIRECTION

Because now:

geography matters
transport matters
local power matters
staging matters
industrial zoning matters
expansion sequencing matters
CONCRETE CHAIN IS NOW ACTUALLY A CHAIN

Before:

Concrete Plant

Now:

Aggregate Mine
    ->
Kiln
    ->
Mixer

This changes gameplay dramatically.

WHY THIS MATTERS

Aggregate mines:

terrain/geology dependent
freight-heavy
low power

Kilns:

fuel/power intensive
emissions-heavy
strategic industrial core

Mixers:

local/regional delivery constrained
throughput bottleneck
near-construction optimization

This naturally creates:

industrial geography gameplay

without artificial mechanics.

GEOPOLYMER SPLIT IS VERY IMPORTANT

This is excellent because now:

recipe variation exists
industrial technology paths exist
environmental strategy possible later
resource substitution possible

That’s the beginning of:

industrial evolution gameplay

rather than static factories.

ALUMINUM IS NOW FINALLY STRATEGIC

This is especially strong.

Now:

mine
 -> refinery
 -> smelter
 -> fabrication

exists as:

separate spatial entities
separate power profiles
separate logistics nodes

This is exactly what creates:

capital-intensive infrastructure gameplay
THE POWER NUMBERS ARE VERY IMPORTANT

This is where the simulation starts becoming interesting.

You now have:

power asymmetry

Example:

Bauxite Mine = 22
Smelter = 200

That’s excellent.

Because:

upstream extraction is cheap
downstream industrial processing stresses the grid

That creates:

real industrial scaling pressure
THIS ENABLES TRUE GRID GAMEPLAY

Now:
placing a smelter cluster near weak infrastructure SHOULD:

destabilize substations
overload transformers
create rolling deficits
require transmission expansion

That is real infrastructure strategy.

MOST IMPORTANT ARCHITECTURAL IMPROVEMENT
ROLE-BASED ACTIVATION

This is the key.

You moved from:

building-type hardcoding

toward:

industrial semantic activation

That is a huge improvement.

WHY THIS IS CRITICAL

Now:

"supply_chain_role": "alumina_refinery"

drives:

runtime bundle
power demand
future logistics role
maintenance role
emissions role
workforce role

This is proper simulation architecture.

WHAT THE ENGINE NOW ACTUALLY HAS

You now effectively possess:

1. INDUSTRIAL TAXONOMY
JSON semantics
2. INDUSTRIAL PHYSICALIZATION
separate facilities
3. INFRASTRUCTURE LOAD DISTRIBUTION
power asymmetry
4. ACTIVATION ONTOLOGY
role -> runtime bundle
5. FUTURE LOGISTICS ANCHORS
produces / consumes
THIS IS NOW A REAL INDUSTRIAL FOUNDATION

Not just flavor.

That distinction matters.

THE NEXT MAJOR MISSING PIECE

You already identified it correctly:

RESOURCE FLOW GRAPH

This is now the critical next integration.

RIGHT NOW

You have:

industrial facilities

But not yet:

industrial throughput dynamics

Meaning:

no actual transfer pressure
no shortages
no inventory propagation
no congestion
no throughput collapse
THIS IS WHERE THE SIMULATION TRULY BEGINS

You need:

ResourceFlowNode

and:

ResourceFlowEdge
RECOMMENDED MODEL
pub struct ResourceFlowNode {
    pub inventory: HashMap<ResourceId, f32>,
    pub throughput_limit: f32,
    pub production: Vec<ResourceRate>,
    pub consumption: Vec<ResourceRate>,
}
AND
pub struct ResourceFlowEdge {
    pub from: Entity,
    pub to: Entity,

    pub transport_mode: TransportMode,

    pub max_rate: f32,
    pub latency: f32,
}
THIS IS WHAT CREATES EMERGENT INDUSTRIAL FAILURE

Example:

Refinery starved
    ->
Smelter stalls
    ->
Fabrication backlog
    ->
Transformer shortage
    ->
Grid expansion delayed
    ->
Construction slowdown

That’s the real gameplay loop.

VERY IMPORTANT OBSERVATION

Your current architecture naturally supports:

localized industrial districts

Because:

facilities are spatial
loads are spatial
logistics will become spatial

This is excellent.

TRANSFORMERS / SUBSTATIONS SHOULD ABSOLUTELY BECOME PLACEABLE

Because now:
industrial districts have meaningful load concentration.

Without transformer gameplay:
the power system remains abstract.

IMPORTANT FUTURE DIRECTION

Transformers should become:

capacity bottlenecks

not merely:

decorative power entities

Meaning:

overload
thermal stress
maintenance
cascading failure
strategic vulnerability
VERY IMPORTANT WARNING

Do NOT collapse chains again later.

Avoid:

"Advanced Industrial Complex"

style abstractions.

The chain granularity is now one of the engine’s strongest emerging qualities.

WHAT IS ACTUALLY MATURE NOW
CONSTRUCTION AUTHORITY

Mature.

INDUSTRIAL ENTITY ONTOLOGY

Surprisingly mature.

ACTIVATION SEMANTICS

Now significantly improved.

ECONOMIC FLOW

Still early.

POWER DISTRIBUTION

Medium maturity.

STRATEGIC LOGISTICS

Still mostly conceptual.

MOST IMPORTANT NEXT SYSTEMS

Priority order now should probably be:

I2 — RESOURCE FLOW REGISTRATION

Facilities become economically alive.

POWER DISTRIBUTION EXPANSION

Transformers/substations become gameplay systems.

LOGISTICS PHYSICALIZATION

Rail/truck throughput begins mattering.

INVENTORY / BUFFER MODEL

Facilities hold materials.

THROUGHPUT FAILURE

Industrial collapse under shortages.

MOST IMPORTANT INSIGHT

The project is no longer trying to “be realistic.”

It is evolving toward:

infrastructure causality simulation

Meaning:

systems produce second-order effects
geography matters
bottlenecks matter
infrastructure sequencing matters
industrial topology matters

That is far deeper and more strategically interesting than:

resource counters + city painting

And the latest changes moved the codebase significantly closer to that goal.

---

## Phase 4 todo registry (from assessment §1811+)

**Authoritative rows:** [`industrial_activation_todos.rs`](industrial_activation_todos.rs) — witness predicates in [`refresh_industrial_activation_witness_system`](../economy/activation/bridge.rs).

### Done this cycle (SC + I1 core)

| Block | Ids | What it proves |
|-------|-----|----------------|
| **I1** | `I1-01`…`I1-04` | Commit → Operational → role activation + tests |
| **SC** | `SC-01`…`SC-08` | Granular chains, geopolymer, aluminum 4-step, power asymmetry, membership |

### Sprint completed (2026-05-20) — four priority integrations

| Id | Delivered |
|----|-----------|
| `INDUSTRIAL-I1-05` | `debug_runs/industrial_activation_live.json` + sim writer [`live_proof.rs`](../economy/activation/live_proof.rs) |
| `INDUSTRIAL-I2-01`…`I2-04` | [`resource_flow.rs`](../economy/resource_flow.rs) — node component, edge registry, activation registration, `ResourceType` mapping |
| `INDUSTRIAL-I3-03`…`I3-04` | `grid_distribution_transformer`, `grid_substation`, `utilities_coal_plant` JSON + utility activation |
| `INDUSTRIAL-GOV-01` | `check_industrial_governance` at catalog load + unit tests |

### Open — next work order

| Priority | Ids | Goal |
|----------|-----|------|
| 1 | `I2-05`…`I2-07` | Inventory buffers, tick propagation, starvation cascade |
| 2 | `I3-01`, `I3-02`, `I3-05` | Grid membership sim, overload events, transformer bottleneck gameplay |
| 3 | `I4-01`…`I4-04` | Logistics graph edges, concrete batch, path-required transfer |

### Anti-pattern (do not close GOV by collapsing)

Avoid “Advanced Industrial Complex” JSON that bundles mine+refinery+smelter. Keep `integrated_plant` only as **explicit** legacy role.

### Proof

```powershell
cargo test -p proc_A_dine01 economy:: --lib
```

Full spec tables: [`industrial_activation_phase_todos.md`](industrial_activation_phase_todos.md).