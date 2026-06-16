> **Implementation:** [`docs/archive/2026-06-src-dev/plans/plan_designer_mcp_art_toolchain_exec_001_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_designer_mcp_art_toolchain_exec_001_v1.md) · **Tooling:** [`tools/mcp/README.md`](../../tools/mcp/README.md) · **Extensions:** [`art_extend.md`](art_extend.md)

Use:

Designer Agent
    ↓
Asset Specification
    ↓
Toolchain
    ↓
Generated Asset
    ↓
Validation
    ↓
Game Asset

The agent creates instructions and parameters. The tools create the assets.

MCP Structure

I'd separate it into several MCP services.

Art Director MCP
Asset Library MCP
Material MCP
Geometry MCP
Validation MCP
Reference MCP
Art Director MCP

Purpose:

Define style
Define standards
Create asset requests
Review outputs

Example request:

{
  "asset_type": "residential_wall",
  "era": "modern",
  "wealth_level": "middle",
  "material": "brick",
  "module_size": "4m"
}

This MCP never creates art.

It creates specifications.

Reference MCP

Purpose:

Gather real-world references.

Sources:

Natural Earth
OpenStreetMap
Wikipedia
Government GIS
Library of Congress
Historic England
USGS
Rail infrastructure manuals
Military engineering manuals

Output:

{
  "reference_set": [
    "row houses",
    "mid-rise apartments",
    "industrial warehouses"
  ]
}

No AI imagery.

Only real references.

Material MCP

Instead of generating textures from prompts.

Generate materials procedurally.

Tools:

Substance Designer
Material Maker
Quixel Mixer

Example:

{
  "material_type": "brick",
  "age": 30,
  "weathering": 0.4,
  "color_family": "red"
}

Produces:

albedo
normal
roughness
ao
height
Geometry MCP

This is likely the most important component.

Backend:

Blender Geometry Nodes
Blender Python
Houdini

Agent sends:

{
  "building_type": "apartment",
  "width": 24,
  "depth": 12,
  "floors": 6,
  "roof": "flat"
}

Tool generates mesh.

Asset Library MCP

Stores reusable modules.

Example:

Wall_A
Wall_B
Wall_C

Window_A
Window_B

Roof_A
Roof_B

Door_A
Door_B

The agent never asks for:

create apartment building

Instead:

assemble apartment using:
wall_b
window_a
roof_b
door_a
Validation MCP

One of the biggest missing pieces in most AI pipelines.

Checks:

Polygon count
Scale
UVs
PBR compliance
Naming
LODs
Collision meshes
Grid alignment

Example:

{
  "asset": "warehouse_04"
}

Returns:

{
  "valid": true,
  "issues": []
}

or

{
  "valid": false,
  "issues": [
    "pivot incorrect",
    "mesh non-manifold"
  ]
}
Settlement Style System

The designer agent should define style packs.

Example:

pub struct StylePack {
    pub wall_set: Vec<AssetId>,
    pub roof_set: Vec<AssetId>,
    pub window_set: Vec<AssetId>,
    pub door_set: Vec<AssetId>,
    pub props: Vec<AssetId>,
}

Example styles:

Victorian
Modern Urban
Rural
Industrial
Military
Soviet
Colonial
Port District
Railway District
Procedural District Generation

Instead of generating buildings directly.

Generate district rules.

{
  "district": "industrial",
  "allowed_buildings": [
    "warehouse",
    "factory",
    "storage"
  ],
  "roof_types": [
    "flat",
    "sawtooth"
  ]
}

Then building generation uses those rules.

Designer Agent Workflow
Request New Asset
        ↓
Reference MCP
        ↓
Specification MCP
        ↓
Geometry MCP
        ↓
Material MCP
        ↓
Validation MCP
        ↓
Asset Library MCP
Token Efficiency

The biggest token saver is to stop describing assets in natural language repeatedly.

Instead use structured specifications.

For example, instead of:

Generate a six-floor red brick apartment with white trim...

Use:

{
  "archetype": "apartment_midrise",
  "style": "industrial_urban",
  "floors": 6,
  "material": "brick_red_01"
}

The agent only exchanges compact metadata.

For Your Bevy Project

I would eventually have the MCP produce artifacts directly consumable by the engine:

.glb
.ron
.riv
.json
.png
.webp

and generate accompanying definitions:

BuildingArchetype
StylePack
MaterialDefinition
DistrictDefinition

so the same pipeline can feed:

Organic settlement growth
Player construction
Procedural cities
Military bases
Industrial complexes
Rail networks
Ports
Airfields

from one unified asset-authoring system.Designer Deliverables

Instead of:

House01
House02
House03
House04

Request:

Wall Modules
Roof Modules
Door Modules
Window Modules
Foundation Modules
Industrial Props
Utility Props
Road Props
Texture Requirements

Prefer:

Tileable PBR textures

over:

Unique baked textures

For example:

brick_wall_01
brick_wall_02
concrete_01
concrete_02
steel_panel_01
glass_01
wood_01

Each should include:

Base Color
Normal
Roughness
Ambient Occlusion
Style Guide

To avoid the common AI-generated look:

Do not ask for:

beautiful
cinematic
ultra detailed
photoreal
concept art

Those phrases tend to produce generic AI aesthetics.

Instead specify:

functional
architectural
engineering drawing reference
modular asset
game-ready
real-world construction methods
Reference Sources

Give references such as:

USGS
Natural Earth
OpenStreetMap
Historical building surveys
Industrial architecture references
Railway engineering manuals
Military construction manuals

rather than AI art references.

Preferred Workflow

I would recommend the designer use:

Textures

Tools like:

Material Maker
Substance Designer
ArmorPaint
Quixel Mixer

Procedural textures age much better than generated image textures.

Models

Tools:

Blender Geometry Nodes
Houdini Indie
Blender Python Tools

For example:

Generate apartment block
Generate warehouse
Generate factory
Generate rail station

from parameters.

Building Generator Inputs

Example designer-created asset package:

Walls:
  Brick A
  Brick B
  Concrete A
  Concrete B

Windows:
  Residential A
  Residential B
  Commercial A

Doors:
  Residential
  Commercial
  Industrial

Roofs:
  Flat
  Pitched
  Industrial
Asset Specification Document

Give the designer something like:

Module Size:

1m
2m
4m

Pivot:
Bottom Center

Grid:
1 meter

Scale:
Real World

Coordinate System:
Z Up (or whatever engine standard)
Texture Resolution

For a large simulation:

512x512
1024x1024

is usually enough.

Reserve:

2048+

for landmarks.

Procedural Variation

Rather than making:

50 brick textures

make:

5 brick textures

and vary in engine:

pub struct MaterialVariation {
    pub hue_shift: f32,
    pub roughness_shift: f32,
    pub dirt_level: f32,
    pub wear_level: f32,
}

This produces thousands of appearances from a handful of textures.

AI Usage Guidance

If using AI at all:

Use it primarily for:

Reference gathering
Moodboards
Architectural exploration
Material references

Avoid using AI outputs directly as final textures or final models.

Instead:

AI Reference
→ Designer Cleanup
→ Procedural Material Creation
→ Engine Integration
Prompt Template for Designers

Something like:

Create modular architectural assets for a procedural city builder.

Requirements:

- Real-world proportions
- Grid aligned
- Reusable modules
- No baked lighting
- No stylistic AI artifacts
- Engineering-focused construction
- Suitable for procedural assembly

Deliver:

- Wall modules
- Roof modules
- Window modules
- Door modules
- Industrial props
- Utility props

Include:

- Albedo
- Normal
- Roughness
- AO

Target:

Modern city-building simulation with procedural settlement growth.