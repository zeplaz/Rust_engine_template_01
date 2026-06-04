Right now the pipeline is:

PG Assembly
    ->
Greybox GLB
    ->
Single Orthographic Snapshot
    ->
Atlas Pack
    ->
Registry

when it should be:

Building Definition
    ->
Procedural Assembly
    ->
Variant Generation
    ->
Material Resolution
    ->
Blender Scene Assembly
    ->
Keyframe Generation
    ->
Render
    ->
Atlas Build
    ->
Registry
    ->
Runtime State Selection
Planner Phase 1: Fix Asset Architecture

The biggest mistake currently is that variants are generated AFTER rendering.

Variants should exist before rendering.

Current:

warehouse
    ->
render
    ->
fake damage
    ->
fake night

Should be:

warehouse
    ->
variant generator

        clean_day
        clean_night
        damage_1
        damage_2
        damage_3
        fire
        abandoned
        construction

    ->
render each variant

Example:

BuildingDefinition(
    id: "warehouse_industrial",

    modules: [
        RoofLarge,
        WallIndustrial,
        DoorRollup,
        WindowFactory,
        StackIndustrial,
    ],

    variants: [
        CleanDay,
        CleanNight,
        DamageLight,
        DamageHeavy,
        Fire,
        Construction,
    ]
)
Planner Phase 2: Geometry Must Become Real Modules

Current:

module_wall = cube
module_roof = cube
module_stack = cube

This is only useful for:

pathfinding
placement
validation

not rendering.

Need:

module_wall_industrial_01.glb
module_wall_industrial_02.glb

module_roof_flat_01.glb
module_roof_sawtooth_01.glb

module_stack_small.glb
module_stack_large.glb

module_window_factory.glb

module_door_rollup.glb

MCP should assemble:

warehouse

  wall x12
  roof x4
  windows x24
  stacks x3

        ↓

assembled.blend

not

warehouse_cube.glb
Planner Phase 3: Material Resolution Layer

Currently:

material profile exists

brick_red_01

but texture doesn't

Need hard validation.

Planner rule:

Material Definition
    MUST resolve to:

albedo.png
normal.png
roughness.png

OR

explicit fallback

Example:

MaterialProfile(
    id: "brick_red_01",

    albedo: "...",
    normal: "...",
    roughness: "...",
)

Validator:

FAIL BUILD

if missing textures

not

auto grey cube
Planner Phase 4: Procedural Variant Generation

This is where the real power comes from.

Do NOT manually author:

clean
night
damage
fire

Generate them.

Example:

pub enum VariantLayer {
    Lighting,
    Damage,
    Occupancy,
    Fire,
    Construction,
}

Then:

pub struct BuildingState {
    lighting: LightingState,
    damage: DamageState,
    occupancy: OccupancyState,
    fire: FireState,
}

Generated combinations:

clean_day

clean_night

damage_light_day

damage_light_night

damage_heavy_day

damage_heavy_night

fire_small

fire_medium

fire_large

The renderer receives:

VariantRecipe

instead of

PNG
Planner Phase 5: Rotation System

Current building pipeline completely misses this.

The planner should establish:

Every renderable asset
must define

facings

Example:

RenderContract(
    facings: 8,
)

or

RenderContract(
    facings: 4,
)

Render matrix becomes:

State x Facing

instead of

State only

Warehouse:

8 facings

6 states

48 renders

Fire:

8 facings
8 fire frames

64 renders

Total:

State × Facing × Animation
Planner Phase 6: Unified Atlas Schema

Current:

variant_key -> uv

Too simple.

Need:

AtlasLookup {
    variant,
    facing,
    frame,
}

Example:

lookup(
    "damage_light",
    facing = 5,
    frame = 0
)

returns

uv

Atlas metadata:

{
  "variant":"damage_light",
  "facing":5,
  "frame":0,
  "uv":[...]
}
Planner Phase 7: Runtime State Resolver

Current:

variant -> uv

Need:

pub struct VisualState {
    variant: VariantId,
    facing: u8,
    frame: u8,
}

Resolver:

atlas.resolve(
    visual_state.variant,
    visual_state.facing,
    visual_state.frame
)

This aligns perfectly with:

Buildings
Vehicles
Power Plants
Turrets
Units
Props

Same system.

Planner Phase 8: Blender Integration

MCP should never render directly from a greybox export.

Instead:

Definition
    ->
Assembly
    ->
Blender Scene
    ->
Apply Materials
    ->
Apply Variant
    ->
Apply Facing
    ->
Render

Pseudo flow:

for variant in variants:

    apply_variant(variant)

    for facing in facings:

        set_rotation(facing)

        for frame in animation_frames:

            render()
Planner Phase 9: Tilemap Output Contract

Output should become:

warehouse/
warehouse/
    atlas.png
    atlas_meta.json
    visual_config.ron
    assembly.blend
    manifest.ron

visual_config.ron

BuildingVisual(
    facings: 8,

    states: [
        "clean_day",
        "clean_night",
        "damage_light",
        "damage_heavy",
        "fire"
    ]
)
Planner Phase 10: Promotion Gates

Current gate:

PNG exists

Worthless.

Require:

Geometry
module count > minimum
Materials
100% material resolution
Variants
required states generated
Facings
all facings generated
Atlas
lookup validation passes
Runtime
spawn test passes

Only then:

ship = true
Recommended Immediate Roadmap

Do these in order:

1. Freeze current building atlases
2. Define new atlas schema:
       variant × facing × frame
3. Build runtime resolver
4. Replace greybox-only modules
       with production module library
5. Add material validation
6. Build variant generation layer
7. Generate Blender assemblies
8. Run keyframe_render pipeline
9. Pack atlas
10. Promote only after full validation

The most important architectural correction is: the procedural system should generate a complete building asset graph (geometry + materials + variants), and rendering should be the final compilation stage. The current pipeline renders too early, which is why you're getting gray slabs and state-only atlases instead of proper state × facing × animation production assets.


What the Planner Should Adopt

The authoritative workflow should be:

Designer
    ->
Art Pipeline Suite

        Catalog
        Assembly
        Variants
        Atlas

    ->
MCP Commands

    ->
Headless Blender

    ->
Output Assets

NOT:

Designer
    ->
Open Blender

    ->
Fix Stuff

    ->
Render

    ->
Export

Blender should be treated exactly like:

Rust Compiler
Asset Compiler
Texture Baker

A backend tool.

Not the primary user interface.

Fix the Authority Chain

Currently there are multiple competing truths:

_module_index.json

assembly_snapshot

stylepack

variant catalog

atlas registry

The planner should define one source of truth.

I would recommend:

Asset Definition
        ↓
Assembly Snapshot
        ↓
Variant Graph
        ↓
Build Graph
        ↓
Atlas

Everything else becomes derived data.

Missing System: Assembly Graph

Right now snapshots are too shallow.

You need something closer to:

AssemblyNode(
    id: "wall_001",

    module: "industrial_wall_large",

    material_profile: "steel_panel_01",

    transform: (...),

    tags: [
        "industrial",
        "weathered",
    ]
)

Current snapshots mostly capture geometry placement.

The planner should extend snapshots to carry:

module
material profile
variant tags
lod policy
damage rules
emission rules

per node.

Missing System: Variant Graph

This is probably the biggest architectural gap.

Currently:

Variant Bake

acts like:

Generate PNG

which is far too late.

Instead:

BuildingDefinition

should generate:

VariantGraph

Example:

Warehouse
    ├── clean_day
    ├── clean_night
    ├── damaged_light
    ├── damaged_heavy
    ├── abandoned
    ├── burning
    └── construction

Each variant contains modifications.

VariantNode {
    material_overrides,
    visibility_overrides,
    emission_overrides,
    decal_overrides,
}

Then MCP compiles:

Warehouse
    ->
Variant Graph
    ->
Render Graph

instead of

Warehouse
    ->
PNG
Material System Needs Promotion

The planner is still treating materials as decoration.

They should be first-class assets.

Current:

Module
    +
Material

Desired:

AssetAssemblyNode {
    geometry,
    material_profile,
    variant_overrides,
}

Example:

(
    module: "wall_industrial_a",
    material_profile: "steel_panel_01",
)

Not:

(
    module: "wall_industrial_a"
)

and later hoping Blender figures it out.

APS-UI-003b Should Become Much Larger

Current description:

slot grid
material picker

That is too narrow.

The planner should redefine it as:

Assembly Editor

with:

Module Placement
Material Assignment
Tag Assignment
Variant Assignment
LOD Assignment
Validation

Something like:

Assembly Tab

+--------------------------------+
| Footprint Grid                 |
+--------------------------------+

Selected Slot:

Module:
    [industrial_wall_large]

Material:
    [steel_panel_01]

Tags:
    [industrial]
    [weathered]

Variants:
    [clean]
    [damaged]
    [night]

LOD:
    [production]
Build Graph Should Be Explicit

Currently many commands appear disconnected.

Planner should formalize:

build_assembly
build_variants
build_blend
render_frames
pack_atlas
register_atlas

as nodes in a dependency graph.

Example:

Assembly Snapshot
        ↓
Variant Graph
        ↓
Blend Build
        ↓
Frame Render
        ↓
Atlas Pack
        ↓
Registry Update

Every node produces:

artifact
manifest
witness
validation
Headless Blender Should Become a Worker

Do not think of Blender as an editor.

Think of it as:

Worker Type:
    blender-worker

MCP submits jobs:

{
  "job":"render_variant",
  "asset":"warehouse_industrial",
  "variant":"damage_heavy",
  "facing":3
}

Worker returns:

{
  "png":"..."
}
Warehouse Fix Should Be Reframed

Instead of:

Fix warehouse

Planner should create:

BUILDING-TILE-SPINE-001

Goal:

State × Facing × Frame

for every building asset.

Warehouse becomes the pilot.

Revised Backlog Order

I would tell the planner to reorder work like this:

ARCH-001
Formal Assembly Graph schema
ARCH-002
Formal Variant Graph schema
ARCH-003
Material profile per placement
APS-UI-003b
Assembly Editor
BUILD-001
Build dependency graph
RENDER-001
Headless Blender worker contract
ATLAS-001
State × Facing × Frame atlas schema
RUNTIME-001
Variant/Facing resolver
PILOT-001
Warehouse through new pipeline

Only after those are complete should the planner allow:

warehouse_tile_ship_workflow_v1

to exist as a shipping workflow.

At the moment, that document should be classified as:

Debug / Recovery Workflow

because it bypasses the intended Art Pipeline Suite → MCP → Headless Build architecture instead of validating it.


better improvment to our design inspreration which 

It was a hierarchical grammar.

More like:

District Style
    ->
Building Archetype
    ->
Massing Grammar
    ->
Facade Grammar
    ->
Detail Grammar
    ->
Damage/Age Grammar

which produced much richer variation.

First: Why Are There So Many Files?

Honestly?

Most of these are not generation files.

You have:

UI
Schemas
Validators
Witnesses
Tests
Atlas
Tile pipeline
Render extraction
Runtime lookup

mixed into the same list.

The actual procedural generation core is much smaller.

Files Planner Should Focus On
Tier 1 - Core Generator

These are the real heart.

src/construction/procedural/footprint_grid.rs

src/construction/procedural/assembly_snapshot.rs

src/construction/procedural/types.rs

src/construction/procedural/load.rs

src/construction/procedural/module_index.rs

src/construction/procedural/variant_recipe.rs
Tier 2 - Definition Layer

These define what can be built.

src/construction/building_definitions.rs

assets/configs/buildings/style_packs/*.ron

assets/configs/buildings/_module_index.ron
Tier 3 - Build Compiler

These assemble the scene.

tools/mcp/python/rust_engine_mcp/assembly.py

tools/mcp/blender/scripts/ops/assembly_import.py

tools/mcp/python/rust_engine_mcp/building_definition.py

Everything else is mostly downstream.

Biggest Problem I See

Current generator appears to be:

Footprint
    ->
Module Placement

That is not really a building grammar.

It's a placement grammar.

Current likely model:

4x3 footprint

place wall

place roof

place door

What Republic-style systems do:

Building Type

Industrial Warehouse

↓

Massing Strategy

Long Hall

↓

Roof Strategy

Sawtooth

↓

Facade Strategy

Factory Windows

↓

Detail Strategy

Stacks
Ventilation
Pipe Runs

↓

Age Strategy

Clean
Weathered
Abandoned

That hierarchy is largely missing.

Planner Order 1

Create a Formal Building Grammar

New doc:

ARCH-BUILD-GRAMMAR-001

Define:

pub struct BuildingGrammar {
    archetype: ArchetypeRule,
    massing: MassingRule,
    roof: RoofRule,
    facade: FacadeRule,
    detail: DetailRule,
}

Instead of:

generate(width, depth, floors)

Use:

generate(
    archetype,
    district_style,
    seed
)

Example:

IndustrialWarehouse

can choose:

Long Hall

Double Hall

L Shape

Yard Complex

Already a huge gain.

Planner Order 2

Assembly Graph Needs More Data

Current slot:

{
  "module":"wall_a"
}

Too little.

Target:

{
  "module":"wall_a",

  "role":"primary_wall",

  "material":"brick_red",

  "style":"industrial",

  "weathering":"medium",

  "tags":[
      "street_facing"
  ]
}

The grammar should produce semantic meaning.

Not just geometry.

Planner Order 3

Assembly Editor Needs Massive Expansion

Current panel feels like:

JSON editor with checkboxes

Should become:

Building Authoring Tool

Add:

Footprint Preview

Not text.

Actual grid.

□ □ □ □

□ □ □ □

□ □ □ □

Clickable.

Placement Heatmap

Show:

Wall
Roof
Window
Door
Stack

visually.

Archetype Selection
Industrial Warehouse

Factory

Office

Power Plant

Apartment
Grammar Inspector

Show:

Massing:
    Long Hall

Roof:
    Sawtooth

Facade:
    Factory

Detail:
    Heavy Industry
Planner Order 4

Add Preview Everywhere

You asked about preview windows.

Yes.

Almost every tab should have one.

Catalog

Preview GLB

Assembly

Preview assembled building

Variants

Preview state

Atlas

Preview atlas
Recommendation

Use Open Source Viewer

Don't build your own renderer.

Use:

pygltflib
+
pyglet

or

moderngl

or

three.js embedded view

Long-term:

Because you're already using Bevy:

bevy viewer mode

is probably the best solution.

Run a small Bevy preview process.

APS communicates:

Load Snapshot

Render Preview

Return Thumbnail

Then you're previewing with the same renderer as the game.

Planner Order 5

Tag System Is Wrong

Current tags:

industrial
weathered
damaged

are too generic.

Need tag categories.

Example:

LocationTag

StreetFacing
Corner
Rear
Interior
ArchitecturalTag

Industrial
Commercial
Residential
DetailTag

Pipework
Stack
Ventilation
LoadingDock
ConditionTag

Clean
Weathered
Damaged
Abandoned

This lets grammars reason about tags.

Planner Order 6

Create Grammar Debug Window

Most important tool.

Add panel:

Grammar Inspector

Example:

Warehouse_001

Archetype:
    Industrial Warehouse

Massing:
    Long Hall

Roof:
    Sawtooth

Facade:
    Factory Window Grid

Detail:
    Pipe Cluster

Seed:
    847282

Show generated rule chain.

Without this you'll never understand why a building was generated.

Planner Order 7

Module Library Audit

Current module kit likely too small.

Need categories.

Walls

Roofs

Corners

Windows

Doors

Stacks

Vents

Pipe Runs

Platforms

Signs

Lights

Air Conditioners

Cranes

A grammar cannot create variety if modules don't exist.

Orders I'd Send To Planner
ARCH-BUILD-GRAMMAR-001
Create hierarchical building grammar system.
ARCH-ASSEMBLY-GRAPH-002
Extend assembly nodes with semantic metadata.
APS-UI-003b-EXPANDED
Convert Assembly Editor into Building Authoring Tool.
APS-PREVIEW-001
Add live preview panel to Catalog, Assembly, Variants, Atlas.
APS-GRAMMAR-INSPECTOR-001
Display generation rule chain.
APS-TAGS-001
Replace flat tags with categorized semantic tags.
PG-MODULE-AUDIT-001
Audit module kit coverage and identify missing architectural categories.
PG-QUALITY-001
Measure generator diversity:
    silhouette count
    roof diversity
    facade diversity
    detail diversity

The biggest strategic improvement is not another atlas fix or bake fix. It's evolving the generator from a footprint → module placement system into a hierarchical architectural grammar that can generate distinct building families, expose its decisions in the UI, and preview the results immediately. That is the foundation



The document you pasted is effectively assuming:

Assembly Generated
    ->
Open Blender
    ->
Assign Materials
    ->
Preview
    ->
Render

But the architecture you've been describing for a long time is:

Assembly Generated
    ->
Art Pipeline Suite
        Assign Materials
        Preview Materials
        Preview Variants
        Preview Assembly
    ->
Headless Build Worker
        (Blender hidden)
    ->
Render

Those are fundamentally different systems.

The Planner Needs To Stop Treating Blender As The Material Editor

This section is the biggest red flag:

Phase 2 — Materials on assembly

Assign PBR materials to all visible modules

followed by

Open blend
Assign materials
Viewport render sanity

That is a DCC workflow.

Not an asset-pipeline workflow.

Your actual target is:

Assembly Editor

select slot

wall_industrial_a

Material:
    steel_panel_01

Preview updates immediately

Then:

Save Snapshot

Then:

MCP
    ->
Build Blend

Then:

Headless Blender

Then:

Render

Nobody manually opens Blender.

Material Assignment Must Become Part Of The Assembly Graph

Right now I suspect materials are still being treated as a Blender concern.

Planner should change authority.

Current:

Assembly Snapshot
    ->
Build Blend
        ->
Assign Materials

Desired:

Assembly Snapshot

node:
    module
    material
    tags
    lod

Example:

{
  "node_id":"wall_001",

  "module":"industrial_wall_large",

  "material_profile":"steel_panel_01"
}

The blend should simply inherit this.

APS Should Become The Material Editor

This is probably the single highest ROI UI improvement.

Current Assembly Editor:

Placement List

Selected Slot

Module
Material
Tags
LOD

but it's basically a property editor.

Target:

+----------------------------------+
| Assembly Preview                 |
|                                  |
|   [3D View]                      |
|                                  |
+----------------------------------+

Selected Module

Module:
    industrial_wall_large

Material:
    steel_panel_01

Variant:
    clean

LOD:
    production

When material changes:

Preview updates immediately
Preview Is The Missing Piece

Without preview:

steel_panel_01

steel_panel_02

steel_panel_03

are just words.

Nobody knows what they mean.

Planner should add:

APS-PREVIEW-001

Material browser:

Steel Panel 01

[thumbnail]

Albedo
Normal
Roughness

Click:

Apply

Preview updates.

I Would Go Further

Material assignment shouldn't even be done from a combobox.

Current:

ttk.Combobox(
    slot,
    textvariable=self.material_var,
    values=self._material_profiles
)

This doesn't scale.

Use:

Material Library

[thumb]
Steel Panel 01

[thumb]
Brick Red 01

[thumb]
Concrete Weathered

Click material.

Apply to selected slot.

Why Blender Preview Is Not Needed

Because Blender is solving a problem you don't actually have.

You need:

View GLB

Apply Material

View Result

not:

Rigging
Animation
Modifiers
Geometry Nodes
Cycles
Node Editor

95% of Blender UI is irrelevant.

Better Architecture
APS

owns:

Assembly Editing
Material Assignment
Variant Editing
Preview
Blender Worker

owns:

Import GLB

Apply Materials

Render

Export

Designer never sees Blender.

The Procedural Generator Also Needs This

Current generator likely outputs:

{
  "module":"wall_industrial"
}

Planner should change generation output.

Generator should emit:

{
  "module":"wall_industrial",

  "material_profile":"steel_panel_01",

  "weathering":"medium",

  "detail_set":"industrial_heavy"
}

Now generated buildings already know their materials.

Then APS allows override.

Generator Suggests

Steel Panel 01

User changes to:

Brick Red 01

without touching Blender.

Preview Architecture I'd Recommend

For this project specifically:

Don't build another renderer.

You already have Bevy.

Create:

APS Preview Service

Process:

aps

sends:

{
  "assembly_snapshot":"..."
}

Preview process:

bevy_preview

loads:

modules
materials
variants

renders:

thumbnail

returns:

preview.png

Then:

Assembly Tab
Variant Tab
Atlas Tab
Catalog Tab

all use the same preview backend.

Planner Order I'd Issue Immediately

Before any more warehouse work:

ARCH-MATERIAL-AUTHORITY-001

Material assignment authority moves from Blender
to Assembly Snapshot.
APS-UI-003b

Add material browser with thumbnails.
APS-PREVIEW-001

Live assembly preview panel.
APS-PREVIEW-002

Material preview updates selected module.
APS-PREVIEW-003

Variant preview updates assembly state.
PG-MATERIAL-GENERATION-001

Procedural generator emits material_profile
per placement.
BUILD-WORKER-001

Blender becomes headless render worker only.

I would honestly pause almost all "manual keyframe warehouse" planning until this is addressed, because otherwise you're building more workflow around a Blender-centered material pipeline that you already know you don't want as the long-term production path. The material assignment authority should be moved into the Assembly/APS layer first, then the rest of the rendering pipeline can consume that data automatically.


Artist
    ->
Art Pipeline Suite

    Catalog
    Assembly
    Materials
    Variants
    Atlas
    Grammar
    Validation

    ->
Build Pipeline

    ->
Blender Worker

The artist never needs to know Blender exists for normal production.

Missing Major System: Material Studio

Right now APS appears to have:

Material:
    steel_panel_01

as a dropdown.

That's not really a material workflow.

You need something closer to:

Material Studio

Materials
--------------------------------

Steel Panel 01
Brick Red 01
Concrete Weathered
Asphalt Dark
Factory Roof Metal

Preview
--------------------------------

3D Sphere
3D Wall
3D Building Section

Properties
--------------------------------

Albedo
Normal
Roughness
AO
Height
Metallic
Emission
Material Profiles Need To Become Real Assets

Current likely state:

steel_panel_01

is mostly just a name.

Target:

MaterialProfile(
    id: "steel_panel_01",

    albedo: "textures/steel/albedo.png",

    normal: "textures/steel/normal.png",

    roughness: "textures/steel/roughness.png",

    ao: "textures/steel/ao.png",

    height: "textures/steel/height.png",

    metallic: "textures/steel/metallic.png",
)
Procedural Material Generation

This is a huge opportunity.

Rather than storing every material manually:

Steel Panel Clean
Steel Panel Dirty
Steel Panel Rusted
Steel Panel Burned

Store:

Base Material
+
Material Layers

Example:

Steel Panel

plus

Weathering Layer

plus

Rust Layer

plus

Damage Layer

creates:

Steel Panel Dirty
Steel Panel Heavy Rust
Steel Panel Fire Damaged

automatically.

Material Grammar

Just like buildings need a grammar, materials need one too.

Example:

MaterialRecipe {
    base: SteelPanel,

    weathering: Medium,

    rust: Heavy,

    dirt: Light,

    damage: None,
}

Generated maps:

albedo
normal
roughness
ao
height

from one recipe.

Material Variant System

Instead of:

Building Variant

only affecting geometry:

lights_on
lights_off
damaged

it should also affect materials.

Example:

clean_day

uses:

steel_panel_clean
damaged

uses:

steel_panel_rusted
abandoned

uses:

steel_panel_rusted
broken_windows
heavy_grime
Material Editor Should Support Layer Stacks

Think:

Base Material
--------------------------------

Steel Panel

Layers
--------------------------------

[✓] Dirt
[✓] Rust
[ ] Snow
[ ] Burn
[ ] Moss

Parameters
--------------------------------

Rust Amount: 0.42
Dirt Amount: 0.18

This gives far more variation than maintaining hundreds of static materials.

Preview Requirements

Material editor should support:

Sphere Preview
Material

test.

Wall Preview
Facade

test.

Building Preview
Warehouse

test.

Changing:

Rust Amount

should update all previews.

Procedural Texture Generation

Later you can generate maps automatically.

Input:

Steel Panel

Generator creates:

Albedo
Normal
Roughness
AO
Height

maps.

Then artist tweaks.

Material Replacement Workflow

You mentioned:

replace from a ref

That's another major feature.

Example:

Import Reference Image

APS attempts:

Material Analysis

extracting:

Color
Pattern
Wear
Metallicity

and proposes:

Material Recipe

Artist adjusts.

Material Library Needs Categories

Not:

200 materials

in one list.

Instead:

Industrial

    Steel
    Corrugated
    Factory Glass
    Concrete

Residential

    Brick
    Plaster
    Roofing

Infrastructure

    Asphalt
    Gravel
    Sidewalk
Asset Pipeline Visibility

Another thing you're asking for indirectly:

You want artists to understand where an asset is in the pipeline.

Example:

Warehouse_Industrial_A

Status
--------------------------------

Grammar
    ✓

Assembly
    ✓

Materials
    ✓

Variants
    ✓

Preview
    ✓

Atlas
    Pending

Validation
    Pending

Instead of hunting through folders.

New Planner Orders

I'd add a whole material track.

APS-MAT-001
Material Studio
APS-MAT-002
Material Preview System
APS-MAT-003
Layer-Based Material Recipes
APS-MAT-004
Procedural Texture Generation
APS-MAT-005
Reference Image Material Extraction
APS-MAT-006
Material Library Browser
APS-MAT-007
Building Material Assignment UI
APS-MAT-008
Material Validation Gates
APS-MAT-009
Material Variant Integration
ARCH-BLENDER-001
Blender Worker Contract

Explicitly define Blender as:

Render Worker
Bake Worker
Conversion Worker

and not as the primary authoring environment.

The end goal should be that an artist can:

Create Building
Assign Materials
Generate Variants
Preview Results
Generate Atlas
Validate Asset

entirely inside APS, while Blender runs invisibly in the background only when a bake, render, conversion, or specialized repair operation is needed. That aligns much better with the custom-engine workflow you're building than trying to force a traditional DCC-centered pipeline.