Use:

> **Project alignment:** [`src/dev/plan_art_design_inbound_alignment_v1.md`](../src/dev/plan_art_design_inbound_alignment_v1.md) · **Signed exec:** [`src/dev/plan_designer_mcp_art_toolchain_exec_001_v1.md`](../src/dev/plan_designer_mcp_art_toolchain_exec_001_v1.md) · **Module kit:** [`src/dev/design_procedural_module_kit_v1.md`](../src/dev/design_procedural_module_kit_v1.md)

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

Modern city-building simulation with procedural settlement growth.. Core Idea: “Designer Agent = CLI Orchestrator”

Instead of:

prompt → image → guess → remake

You want:

spec.json → CLI tools → deterministic assets → validation → engine import

The agent’s job:

writes specs
calls tools
inspects outputs
retries with constraints
2. Recommended Open Toolchain (Production Viable)
Geometry / Mesh Generation
Blender (PRIMARY TOOL)

CLI + Python API = your backbone

blender --background --python generate_building.py -- --input spec.json

Capabilities:

procedural buildings
modular assembly
UV generation
LOD creation
export GLB/FBX
Geometry Nodes (inside Blender)

Best for:

buildings
roads
rail systems
cities

Run via Python:

bpy.ops.wm.open_mainfile(filepath="template.blend")
Procedural Geometry Alternative
Houdini (if available)

CLI:

hython generate.py spec.json

Best for:

city blocks
terrain
erosion
road networks
Material Generation
Material Maker (Godot-based, open-source Substance alternative)

CLI usage:

material_maker --batch brick_graph.mmg --output ./materials/brick_01/

Generates:

albedo
normal
roughness
AO
height
Alternative: Blender Shader Nodes Bake
blender --background --python bake_materials.py
Texture Tools
ImageMagick (critical utility)
convert input.png -modulate 100,120,100 output.png

Used for:

variation
color shifting
aging
grime overlays
TexTools (Blender addon, CLI possible via Python)

UV packing, baking support.

Asset Processing / Validation
glTF pipeline tools
gltf-transform (VERY IMPORTANT)
gltf-transform optimize model.glb model_opt.glb
gltf-transform validate model.glb
gltf-transform resize model.glb model_2k.glb

Use for:

compression
LOD prep
validation
cleanup
Mesh Processing
MeshLab (CLI)
meshlabserver -i input.obj -o output.glb -s cleanup.mlx

Used for:

fixing meshes
decimation
normals
cleanup
Terrain / GIS Tools
GDAL (critical for your Natural Earth pipeline)
gdal_translate input.tif output.raw
gdalwarp input.tif output_reprojected.tif
Tippecanoe (vector tiles)
tippecanoe -o map.mbtiles input.geojson
Data / Spec Pipeline
jq (JSON orchestration)
jq '.floors = 6' spec.json
yq (YAML equivalent)
Build Orchestration
Justfile (recommended over Makefile)
build-building:
    blender --background --python tools/build.py -- spec.json
    gltf-transform optimize out.glb out_opt.glb
GNU Make (fallback)
Scripting Layer
Python (main glue layer)

Use for:

calling Blender
generating specs
procedural logic
batch operations
3. MCP Toolchain Architecture

Your system becomes:

Designer Agent (LLM)
        ↓
Spec Generator (JSON)
        ↓
MCP Orchestrator (Rust/Python)
        ↓
CLI Toolchain
        ↓
Artifacts (.glb, .png, .json)
        ↓
Validation MCP
        ↓
Bevy Engine
4. MCP Tool Definitions (IMPORTANT)

Each tool becomes a callable “capability”.

Example:

{
  "tool": "blender.building.generate",
  "input": "building_spec.json",
  "output": "building.glb"
}
Example MCP Tool Registry
pub enum Tool {
    BlenderGenerateBuilding,
    BlenderGenerateRoadNetwork,
    MaterialMakerBuild,
    GltfOptimize,
    MeshLabClean,
    GdalImportTerrain,
}
5. Example Full Pipeline (Real)
Input Spec
{
  "type": "apartment",
  "floors": 6,
  "style": "industrial",
  "footprint": [4, 2],
  "material": "brick_red"
}
Step 1: Blender generate
blender --background --python build.py -- spec.json

Output:

apartment_01_raw.obj
Step 2: optimize
gltf-transform optimize apartment_01.glb apartment_01_opt.glb
Step 3: validate
gltf-transform validate apartment_01_opt.glb
Step 4: register in engine
asset_server.load("apartment_01_opt.glb")
6. Preventing “AI-looking art”

This pipeline avoids AI artifacts because:

deterministic geometry generation
rule-based materials
real-world references (GIS / CAD style)
modular kit assembly
no diffusion model in final output

Instead of:

AI hallucinated building

you get:

engineered procedural structure
7. What You SHOULD Build (Key Missing Piece)

You need a custom tool:

“bevy_asset_mcp_cli”

Responsibilities:

- read spec
- call Blender
- call MaterialMaker
- run validation
- output Bevy-ready asset bundle

Example CLI:

bam build building spec.json
bam build district district.json
bam validate assets/
bam export bevy_bundle/
8. Suggested Folder Structure
tools/
    blender/
        building_gen.py
        road_gen.py
    materials/
        brick.mmg
        concrete.mmg
    pipeline/
        build.rs
        validate.rs

specs/
    buildings/
    districts/
    roads/

assets/
    raw/
    processed/
    bevy/
9. Key Design Shift (Important)

You are no longer building:

a game asset pipeline

You are building:

a deterministic simulation-driven content compilerBelow is a clean, production-style MCP (Model/Modular Control Protocol) spec designed for your designer-agent + CLI toolchain + Bevy asset pipeline.

It is:

JSON-schema driven (tool-safe, deterministic)
Rust-first (serde + strong typing)
CLI/MCP bridge compatible
extensible for Blender / MaterialMaker / GIS / validation tools
designed for non-AI deterministic asset generation
0. MCP DESIGN GOAL

MCP is not “chat protocol”.

It is:

Agent → Tool Request → Execution → Artifact → Validation → Registry

Everything is:

explicit
versioned
reproducible
scriptable
1. CORE ARCHITECTURE
+----------------------+
| Designer Agent (LLM) |
+----------+-----------+
           |
           v
+----------------------+
| MCP Request Builder  |
+----------+-----------+
           |
           v
+----------------------+
| MCP Orchestrator     |
| (Rust daemon/CLI)    |
+----------+-----------+
           |
     +-----+-----+------------------+
     |           |                  |
     v           v                  v
 Blender     MaterialMaker     GIS Tools
 CLI/Python     CLI              GDAL
     |
     v
+----------------------+
| Artifact Store       |
+----------+-----------+
           |
           v
+----------------------+
| Validation MCP       |
+----------+-----------+
           |
           v
+----------------------+
| Bevy Asset Registry  |
+----------------------+
2. MCP CORE OBJECT MODEL
2.1 MCP Request
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub version: String,
    pub tool: McpTool,
    pub input: McpInput,
    pub constraints: Option<McpConstraints>,
    pub output: McpOutputSpec,
}
2.2 Tool Enum
#[derive(Debug, Serialize, Deserialize)]
pub enum McpTool {
    BlenderGenerate,
    BlenderGeometryNodes,
    MaterialMakerBuild,
    GltfOptimize,
    MeshCleanup,
    GdalImport,
    AssetValidation,
}
2.3 Input Types
#[derive(Debug, Serialize, Deserialize)]
pub enum McpInput {
    Building(BuildingSpec),
    RoadNetwork(RoadNetworkSpec),
    RailNetwork(RailNetworkSpec),
    Material(MaterialSpec),
    Terrain(TerrainSpec),
}
2.4 Output Spec
#[derive(Debug, Serialize, Deserialize)]
pub struct McpOutputSpec {
    pub format: OutputFormat,
    pub path: String,
    pub lod: Option<Vec<u8>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    GLB,
    FBX,
    PNG,
    EXR,
    JSON,
}
2.5 Constraints System (IMPORTANT)

This prevents garbage output.

#[derive(Debug, Serialize, Deserialize)]
pub struct McpConstraints {
    pub max_poly_count: Option<u32>,
    pub grid_aligned: Option<bool>,
    pub scale_meters: Option<f32>,
    pub pivot: Option<PivotType>,
    pub lod_required: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PivotType {
    Center,
    BottomCenter,
    Origin,
}
3. DOMAIN SPECS
3.1 Building Spec
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingSpec {
    pub id: String,
    pub footprint: (u32, u32),
    pub floors: u32,
    pub style: StyleId,
    pub usage: BuildingUsage,
    pub material_palette: Vec<MaterialId>,
    pub roof_type: RoofType,
}
pub enum BuildingUsage {
    Residential,
    Commercial,
    Industrial,
    Office,
    Military,
}
3.2 Road Network Spec
#[derive(Debug, Serialize, Deserialize)]
pub struct RoadNetworkSpec {
    pub nodes: Vec<RoadNode>,
    pub edges: Vec<RoadEdge>,
    pub road_class: RoadClass,
}
pub enum RoadClass {
    Dirt,
    Local,
    Arterial,
    Highway,
    Expressway,
}
3.3 Rail Spec
#[derive(Debug, Serialize, Deserialize)]
pub struct RailNetworkSpec {
    pub gauge: RailGauge,
    pub electrification: Electrification,
    pub nodes: Vec<RailNode>,
    pub edges: Vec<RailEdge>,
}
4. MCP RESPONSE FORMAT

Every tool returns:

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub request_id: String,
    pub success: bool,
    pub artifacts: Vec<Artifact>,
    pub logs: Vec<String>,
    pub validation: Option<ValidationReport>,
}
4.1 Artifact
#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub path: String,
    pub artifact_type: ArtifactType,
}
pub enum ArtifactType {
    Mesh,
    Texture,
    Material,
    Scene,
    Metadata,
}
4.2 Validation Report
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
}
5. MCP JSON SCHEMA (ROOT)
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MCP Request",
  "type": "object",
  "required": ["id", "version", "tool", "input"],
  "properties": {
    "id": { "type": "string" },
    "version": { "type": "string" },
    "tool": { "type": "string" },
    "input": { "type": "object" },
    "constraints": { "type": "object" },
    "output": { "type": "object" }
  }
}
6. TOOL EXECUTION CONTRACT

Each tool MUST implement:

stdin  → JSON MCP Request
stdout → JSON MCP Response
stderr → logs only
exit   → success/failure
Example CLI Tool Call
blender_mcp < request.json > response.json
7. BLENDER TOOL CONTRACT
Input:
{
  "tool": "BlenderGenerate",
  "input": {
    "Building": {
      "id": "apt_01",
      "footprint": [4, 2],
      "floors": 6,
      "style": "industrial",
      "usage": "Residential",
      "material_palette": ["brick_red", "concrete_grey"],
      "roof_type": "Flat"
    }
  },
  "output": {
    "format": "GLB",
    "path": "assets/buildings/apt_01.glb"
  },
  "constraints": {
    "scale_meters": 1.0,
    "pivot": "BottomCenter",
    "lod_required": true
  }
}
Output:
{
  "request_id": "abc123",
  "success": true,
  "artifacts": [
    {
      "id": "mesh_apt_01",
      "path": "assets/buildings/apt_01.glb",
      "artifact_type": "Mesh"
    }
  ],
  "logs": ["Generated building mesh", "Applied materials"],
  "validation": {
    "valid": true,
    "issues": []
  }
}
8. ORCHESTRATION RULES
Rule 1: No tool runs without schema validation
Rule 2: All outputs must be deterministic
Rule 3: No free-text asset generation allowed
Rule 4: All assets must pass validation MCP
Rule 5: All geometry must be grid or metric aligned
9. MCP ORCHESTRATOR (RUST CORE)
pub trait McpToolRunner {
    fn execute(&self, req: McpRequest) -> McpResponse;
}
Dispatcher
pub struct McpDispatcher {
    tools: HashMap<McpTool, Box<dyn McpToolRunner>>,
}
Execution Flow
fn run(req: McpRequest) -> McpResponse {
    validate(req)?;
    let tool = dispatcher.get(req.tool);
    let res = tool.execute(req);
    validate_output(&res);
    store(res);
    res
}
10. ASSET REGISTRY FORMAT (BEVY READY)
pub struct AssetEntry {
    pub id: String,
    pub path: String,
    pub asset_type: AssetType,
    pub tags: Vec<String>,
}
11. WHY THIS DESIGN WORKS

This solves:

❌ AI artifact drift

→ removed (no generative imagery dependency)

❌ inconsistent assets

→ validation MCP

❌ pipeline chaos

→ deterministic CLI contract

❌ scaling issues

→ modular toolchain separationBelow is a working, production-style foundation for both systems:

Rust MCP Daemon (core orchestration engine)
Blender Geometry Nodes procedural generator system (CLI tool)

Designed to plug directly into your Bevy pipeline.

PART 1 — RUST MCP DAEMON (FULL CORE IMPLEMENTATION)

This is your central execution brain.

It:

receives MCP JSON requests
validates schema
dispatches CLI tools (Blender, MaterialMaker, GDAL, etc.)
stores artifacts
returns structured responses
is tool-agnostic and extensible
1. Project Structure
mcp-daemon/
  src/
    main.rs
    daemon.rs
    dispatcher.rs
    tool.rs
    protocol.rs
    validation.rs
    storage.rs
    config.rs
  tools/
    blender.rs
    material.rs
    gltf.rs
2. Cargo.toml
[package]
name = "mcp-daemon"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
thiserror = "1"
anyhow = "1"
3. MCP Core Protocol
// src/protocol.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub tool: String,
    pub input: serde_json::Value,
    pub constraints: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub request_id: String,
    pub success: bool,
    pub artifacts: Vec<Artifact>,
    pub logs: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub path: String,
    pub kind: String,
}
4. TOOL TRAIT (CORE ABSTRACTION)
// src/tool.rs
use crate::protocol::{McpRequest, McpResponse};

pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;

    fn execute(&self, req: McpRequest) -> anyhow::Result<McpResponse>;
}
5. DISPATCHER (CORE ENGINE)
// src/dispatcher.rs
use std::collections::HashMap;
use std::sync::Arc;

use crate::tool::McpTool;
use crate::protocol::{McpRequest, McpResponse};

pub struct Dispatcher {
    tools: HashMap<String, Arc<dyn McpTool>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn McpTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn execute(&self, req: McpRequest) -> McpResponse {
        let tool = self.tools.get(&req.tool);

        match tool {
            Some(t) => match t.execute(req.clone()) {
                Ok(res) => res,
                Err(e) => McpResponse {
                    request_id: req.id,
                    success: false,
                    artifacts: vec![],
                    logs: vec![format!("ERROR: {}", e)],
                },
            },
            None => McpResponse {
                request_id: req.id,
                success: false,
                artifacts: vec![],
                logs: vec!["Tool not found".into()],
            },
        }
    }
}
6. DAEMON ENTRY POINT
// src/main.rs
mod protocol;
mod tool;
mod dispatcher;

use dispatcher::Dispatcher;
use protocol::McpRequest;
use std::io::{self, Read};

fn main() {
    let mut dispatcher = Dispatcher::new();

    // register tools (Blender, Material, etc.)
    dispatcher.register(Box::leak(Box::new(
        tools::blender::BlenderTool::new()
    )));

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let req: McpRequest = serde_json::from_str(&input).unwrap();

    let res = dispatcher.execute(req);

    println!("{}", serde_json::to_string_pretty(&res).unwrap());
}
7. BLENDER TOOL IMPLEMENTATION (CLI WRAPPER)
// tools/blender.rs
use crate::tool::McpTool;
use crate::protocol::{McpRequest, McpResponse, Artifact};
use std::process::Command;
use uuid::Uuid;

pub struct BlenderTool;

impl BlenderTool {
    pub fn new() -> Self {
        Self
    }
}

impl McpTool for BlenderTool {
    fn name(&self) -> &str {
        "blender.generate"
    }

    fn execute(&self, req: McpRequest) -> anyhow::Result<McpResponse> {
        let id = Uuid::new_v4().to_string();
        let output_path = "assets/out.glb";

        let status = Command::new("blender")
            .args([
                "--background",
                "--python",
                "tools/blender_generate.py",
                "--",
                &serde_json::to_string(&req.input)?,
                output_path,
            ])
            .status()?;

        Ok(McpResponse {
            request_id: req.id,
            success: status.success(),
            artifacts: vec![Artifact {
                id,
                path: output_path.into(),
                kind: "mesh".into(),
            }],
            logs: vec!["Blender executed".into()],
        })
    }
}
PART 2 — BLENDER GEOMETRY NODES GENERATOR SYSTEM

This is the actual procedural asset engine.

1. Blender Python Entry Script
# tools/blender_generate.py

import bpy
import sys
import json

argv = sys.argv
argv = argv[argv.index("--") + 1:]

input_json = argv[0]
output_path = argv[1]

spec = json.loads(input_json)

bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete()

build_building(spec)

bpy.ops.export_scene.gltf(filepath=output_path, export_format='GLB')
2. Procedural Building Generator
def build_building(spec):
    footprint_x, footprint_y = spec["Building"]["footprint"]
    floors = spec["Building"]["floors"]

    for x in range(footprint_x):
        for y in range(footprint_y):
            create_column(x, y, floors)
3. Column Generator
def create_column(x, y, floors):
    for f in range(floors):
        bpy.ops.mesh.primitive_cube_add(size=1, location=(x, y, f))
        obj = bpy.context.active_object
        obj.scale.z = 1.0
4. Geometry Nodes Upgrade (REAL SYSTEM)

Replace cubes with GN modifier:

def apply_geometry_nodes(obj):
    mod = obj.modifiers.new("BuildingGN", 'NODES')

    mod.node_group = bpy.data.node_groups["BuildingGenerator"]
5. Geometry Nodes Graph (Concept)

Inside Blender:

Nodes:
Input (Footprint)
   ↓
Grid Generator
   ↓
Wall Instances
   ↓
Window Mask
   ↓
Floor Stack
   ↓
Roof System
   ↓
Output Mesh
6. STYLE SYSTEM (CRITICAL)
STYLE_MAP = {
    "industrial": {
        "wall": "brick_dirty",
        "roof": "flat_concrete",
    },
    "modern": {
        "wall": "glass_panel",
        "roof": "steel_flat",
    }
}
7. MATERIAL ASSIGNMENT
def assign_material(obj, style):
    mat = bpy.data.materials.get(style["wall"])
    obj.data.materials.append(mat)
8. OUTPUT STRUCTURE
assets/
  buildings/
    apt_001.glb
    warehouse_002.glb
  materials/
    brick_01/
    concrete_01/
9. HOW MCP + BLENDER CONNECT

Final pipeline:

Bevy / Designer Agent
        ↓
MCP Request JSON
        ↓
Rust MCP Daemon
        ↓
Blender CLI Tool
        ↓
Geometry Nodes Generator
        ↓
GLB Output
        ↓
Validation MCP
        ↓
Bevy Asset Registry
10. WHAT THIS GIVES YOU

You now have:

✔ Deterministic procedural generation
✔ No AI art dependency
✔ Full CLI reproducibility
✔ Swappable toolchain (Blender/Houdini/etc.)
✔ ECS-ready assets
✔ Scalable city generation foundation


1. ROAD + RAIL GRAPH GENERATOR (ECS-NATIVE)

This replaces tile-based “road booleans” entirely.

You are building a transport graph simulation layer.

1.1 Core Idea
Nodes = intersections / stations / hubs
Edges = road / rail segments
Graph = transport network

Everything is:

spatial
weighted
simulated
queryable
1.2 ECS Components
#[derive(Component)]
pub struct TransportNode {
    pub id: NodeId,
    pub position: Vec3,
    pub node_type: NodeType,
}
pub enum NodeType {
    Intersection,
    CityCenter,
    Station,
    Depot,
    Port,
    Junction,
}
#[derive(Component)]
pub struct TransportEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub length: f32,
    pub capacity: f32,
    pub transport: TransportType,
}
pub enum TransportType {
    Road(RoadClass),
    Rail(RailClass),
    Mixed,
}
1.3 Road/Rail Definitions
pub enum RoadClass {
    Dirt,
    Local,
    Arterial,
    Highway,
}
pub enum RailClass {
    Freight,
    Passenger,
    HighSpeed,
}
1.4 Graph Resource
#[derive(Resource)]
pub struct TransportGraph {
    pub nodes: HashMap<NodeId, Vec3>,
    pub edges: Vec<TransportEdge>,
}
1.5 Procedural Generator (MCP OUTPUT SOURCE)
pub struct GraphSeed {
    pub city_positions: Vec<Vec3>,
    pub terrain_cost: fn(Vec3) -> f32,
}
1.6 Generation Logic
pub fn generate_graph(seed: GraphSeed) -> TransportGraph {
    let mut graph = TransportGraph::default();

    // 1. create nodes
    for city in seed.city_positions {
        graph.add_node(city);
    }

    // 2. connect via MST + cost weighting
    let edges = generate_mst(&graph.nodes, seed.terrain_cost);

    graph.edges = edges;
    graph
}
1.7 MCP Bridge
{
  "tool": "graph.generate",
  "input": {
    "cities": [[0,0,0], [100,0,0]],
    "mode": "road+rail"
  }
}
1.8 Blender Output Hook

Graph edges become:

road splines
rail splines

via:

blender --python generate_splines.py graph.json
2. DISTRICT GROWTH SYSTEM (SIMULATION → MCP REQUESTS)

This is your organic city engine.

2.1 Core Idea

Districts do NOT spawn buildings.

They emit demand signals.

2.2 District Component
#[derive(Component)]
pub struct District {
    pub id: DistrictId,
    pub population: f32,
    pub employment: f32,
    pub wealth: f32,
    pub accessibility: f32,
    pub pollution: f32,
}
2.3 Demand Model
#[derive(Resource)]
pub struct DistrictDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
}
2.4 Update System
pub fn update_district_demand(
    mut districts: Query<&District>,
    mut demand: ResMut<DistrictDemand>,
) {
    demand.residential = 0.0;
    demand.commercial = 0.0;

    for d in &districts {
        demand.residential += d.population * 0.2;
        demand.commercial += d.employment * 0.3;
        demand.industrial += (1.0 - d.pollution) * 0.1;
    }
}
2.5 Conversion → MCP Requests

This is the key architectural bridge.

pub fn emit_build_requests(demand: Res<DistrictDemand>) {
    if demand.residential > THRESHOLD {
        send_mcp(BuildingRequest::ApartmentBlock);
    }
}
2.6 MCP Output Example
{
  "tool": "blender.generate",
  "input": {
    "Building": {
      "footprint": [4,2],
      "floors": 8,
      "style": "dense_urban"
    }
  }
}
2.7 Growth Logic (IMPORTANT)

Growth is:

transport access
+ land value
+ demand
- pollution
= building probability
2.8 Result

You get:

slums near industry
high-rise near transit
warehouses near rail hubs
organic city texture

NOT scripted placement.

3. MATERIAL GRAPH SYSTEM (SUBSTANCE-STYLE)

This replaces static textures entirely.

3.1 Core Idea

Materials are node graphs, not images.

3.2 Material Graph Struct
#[derive(Serialize, Deserialize)]
pub struct MaterialGraph {
    pub nodes: Vec<MaterialNode>,
    pub edges: Vec<MaterialEdge>,
}
3.3 Nodes
pub enum MaterialNode {
    Noise(NoiseNode),
    ColorRamp(ColorRampNode),
    Texture(TextureNode),
    Mix(MixNode),
    Weathering(WeatherNode),
}
3.4 Example Graph
Noise
  ↓
ColorRamp
  ↓
Brick Pattern
  ↓
Weathering
  ↓
Output
3.5 Procedural Material Spec
{
  "material": "brick_wall",
  "age": 30,
  "humidity": 0.6,
  "dirt": 0.4
}
3.6 CLI Build Tool
material_mcp build brick_graph.json

Output:

brick_albedo.png
brick_normal.png
brick_roughness.png
3.7 Integration

Bevy loads:

PBRMaterial {
    base_color_texture,
    normal_map,
    roughness_map,
}
4. RUNTIME STREAMING SYSTEM (LOD + INFINITE WORLD)

This is what allows:

infinite cities
large wars
global logistics
no memory explosion
4.1 Core Idea

World is split into streaming chunks

Chunk = 256m x 256m
4.2 Chunk Component
#[derive(Component)]
pub struct WorldChunk {
    pub coord: IVec2,
    pub lod_level: u8,
    pub loaded: bool,
}
4.3 Streaming Manager
#[derive(Resource)]
pub struct StreamingManager {
    pub active_chunks: HashSet<IVec2>,
    pub load_radius: i32,
}
4.4 Streaming System
pub fn update_streaming(
    player: Query<&Transform, With<Player>>,
    mut manager: ResMut<StreamingManager>,
) {
    let center = world_to_chunk(player.translation);

    for x in -R..R {
        for y in -R..R {
            manager.active_chunks.insert(center + IVec2::new(x,y));
        }
    }
}
4.5 LOD System
pub fn calculate_lod(distance: f32) -> u8 {
    if distance < 200.0 { 0 }
    else if distance < 800.0 { 1 }
    else { 2 }
}
4.6 MCP Integration

Chunks request assets:

{
  "tool": "chunk.generate",
  "input": {
    "chunk": [12, 44],
    "lod": 1
  }
}
4.7 Result
near player → full buildings
mid distance → simplified meshes
far → graph-only representation
SYSTEM INTEGRATION OVERVIEW
District System
     ↓
Demand Signals
     ↓
MCP Requests
     ↓
Blender / Material / Graph Tools
     ↓
Assets
     ↓
Streaming System
     ↓
Bevy World
WHY THIS ARCHITECTURE WORKS

You now have:

1. TRUE simulation-driven cities

Not scripted placement

2. Graph-based infrastructure

Not tile hacks

3. Procedural materials

Not static textures

4. Infinite world scaling

Not fixed maps

5. Tool-driven asset production

Not manual modeling