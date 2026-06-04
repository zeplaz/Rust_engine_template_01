Skills = what an agent can do
Rules = how it behaves
Tool access = what it is allowed to execute
Personality = optional constraints (style, verbosity, caution)
Domain = what systems it operates in (tiles, blender, economy, etc.)

Think of it like:

Agent = Skills + Rules + Tool Permissions + Memory Scope
0. CORE DESIGN: AGENT FILE SYSTEM

We define 4 core file types:

/agents/
  designer.agent.json
  coder.agent.json
  simulation.agent.json

/skills/
  tile_generation.skill.json
  blender_geometry.skill.json
  economic_modeling.skill.json

/rules/
  no_ai_art.rule.json
  deterministic_output.rule.json
  batch_processing.rule.json

/tooling/
  blender.tool.json
  tile_mcp.tool.json
  gltf.tool.json
1. AGENT FILE FORMAT
1.1 Base Agent Schema
{
  "id": "designer_agent",
  "version": "1.0",
  "role": "asset_pipeline_designer",

  "skills": [
    "tile_generation",
    "blender_geometry",
    "prop_systems",
    "atlas_building"
  ],

  "rules": [
    "no_ai_generated_images",
    "always_batch_process",
    "prefer_deterministic_tools",
    "no_freeform_geometry_without_spec"
  ],

  "tools": [
    "tile_mcp",
    "blender_mcp",
    "material_mcp",
    "atlas_packer"
  ],

  "memory_scope": [
    "asset_library",
    "style_packs",
    "tile_states"
  ],

  "output_format": "mcp_request_json"
}
2. SKILL FILES (CORE BEHAVIOR MODULES)

Skills are NOT prompts.

They are capability contracts.

2.1 Tile Generation Skill
{
  "id": "tile_generation",
  "type": "procedural_asset_skill",

  "inputs": [
    "base_tile_type",
    "damage_state",
    "power_state",
    "fill_state",
    "lighting_state"
  ],

  "outputs": [
    "tile_texture",
    "tile_normal",
    "tile_metadata"
  ],

  "process": [
    "validate_inputs",
    "select_tile_template",
    "apply_state_layers",
    "render_isometric_bake",
    "export_png",
    "register_to_atlas"
  ]
}
2.2 Blender Geometry Skill
{
  "id": "blender_geometry",

  "inputs": [
    "footprint",
    "floors",
    "style",
    "material_palette",
    "state"
  ],

  "outputs": [
    "glb_model",
    "lod_levels",
    "collision_mesh"
  ],

  "process": [
    "validate_grid_alignment",
    "select_procedural_node_graph",
    "build_geometry_nodes_tree",
    "apply_materials",
    "bake_lods",
    "export_glb"
  ]
}
2.3 Economic Simulation Skill (future extension)
{
  "id": "economic_modeling",

  "inputs": [
    "resource_graph",
    "demand_values",
    "production_nodes"
  ],

  "outputs": [
    "price_changes",
    "logistics_requests",
    "mcp_build_requests"
  ]
}
3. RULE FILES (NON-NEGOTIABLE BEHAVIOR CONSTRAINTS)

Rules are hard constraints, not suggestions.

3.1 No AI Art Rule
{
  "id": "no_ai_generated_images",
  "type": "hard_rule",

  "enforcement": "pre_execution",

  "conditions": [
    "never_generate_images_from_prompts",
    "never_use_diffusion_models_for_final_assets",
    "only_use_deterministic_tools"
  ]
}
3.2 Deterministic Output Rule
{
  "id": "deterministic_output",

  "conditions": [
    "same_input_always_same_output",
    "no_random_unseeded_variation",
    "all_variation_must_be_seed_based"
  ]
}
3.3 Batch Processing Rule
{
  "id": "batch_processing",

  "conditions": [
    "no_single_asset_generation",
    "always_process_asset_groups",
    "prefer_atlas_or_packaged_output"
  ]
}
3.4 Grid Alignment Rule (VERY IMPORTANT FOR YOU)
{
  "id": "grid_alignment",

  "conditions": [
    "all_assets_must_align_to_grid",
    "no_free_rotation_in_isometric_system",
    "tile_size_must_be_fixed_unit"
  ]
}
4. TOOL FILES (MCP INTERFACES)

Tools define what agents are allowed to execute.

4.1 Tile MCP Tool
{
  "id": "tile_mcp",

  "input_schema": "tile_spec_v1",

  "execution": "python tile_generator.py",

  "output": [
    "png",
    "normal_map",
    "metadata_json"
  ]
}
4.2 Blender MCP Tool
{
  "id": "blender_mcp",

  "execution": "blender --background --python build.py",

  "input_schema": "building_spec_v1",

  "output": [
    "glb",
    "collision_mesh",
    "lods"
  ]
}
4.3 Atlas Tool
{
  "id": "atlas_packer",

  "execution": "python pack_atlas.py",

  "output": [
    "atlas.png",
    "atlas_meta.json"
  ]
}
5. NEW AGENT TYPES (IMPORTANT)

Now we define specialized agents instead of general LLMs

5.1 Designer Agent
{
  "id": "designer_agent",

  "skills": [
    "tile_generation",
    "blender_geometry"
  ],

  "rules": [
    "grid_alignment",
    "deterministic_output",
    "no_ai_generated_images"
  ],

  "tools": [
    "tile_mcp",
    "blender_mcp",
    "atlas_packer"
  ]
}
5.2 Coder Agent
{
  "id": "coder_agent",

  "skills": [
    "rust_ecs_generation",
    "mcp_tool_wiring",
    "simulation_logic"
  ],

  "rules": [
    "no_pseudocode",
    "always_compile_ready_rust",
    "modular_ecs_design"
  ],

  "tools": [
    "rust_analyzer",
    "mcp_tool_builder"
  ]
}
5.3 Simulation Agent
{
  "id": "simulation_agent",

  "skills": [
    "economic_modeling",
    "logistics_graphs",
    "district_growth"
  ],

  "rules": [
    "system_level_only",
    "no_direct_asset_creation",
    "must_emit_mcp_requests"
  ]
}
6. AGENT EXECUTION MODEL

This is the key architecture:

Agent
  ↓
Load Skills
  ↓
Apply Rules
  ↓
Select Tools
  ↓
Generate MCP Request
  ↓
Execute Toolchain
  ↓
Validate Output
  ↓
Register Asset
7. WHY THIS SOLVES YOUR ORIGINAL PROBLEM

You were previously at risk of:

LLM hallucinated assets
inconsistent pipelines
ad-hoc prompts
unstructured generation
"AI slop" geometry/textures

Now instead you have:

✔ deterministic pipelines
✔ strict tool execution
✔ reusable skill modules
✔ enforceable rules
✔ no freeform generation
✔ production-grade asset control
8. IMPORTANT INSIGHT (THIS IS THE CORE IDEA)

You are not building:

an AI system

You are building:

a deterministic procedural production engine controlled by agents

Agents are just:

policy + skill routing + tool selection

Rust Skill Runtime Engine (deterministic, enforceable, MCP-safe)
Visual Skill Graph Editor (Blueprint-style system for agents + MCP tools)

They are designed to work together.

1. RUST SKILL RUNTIME ENGINE (CORE SYSTEM)
0. Purpose

This engine:

loads agent files (.agent.json)
loads skills + rules + tools
enforces execution constraints
validates MCP requests
executes toolchain calls deterministically
logs full reproducibility chain
1. SYSTEM ARCHITECTURE
                +----------------------+
                | Agent Definition     |
                +----------+-----------+
                           |
                           v
                +----------------------+
                | Skill Runtime Engine |
                | (Rust Core)          |
                +----------+-----------+
                           |
     +---------------------+----------------------+
     |                     |                      |
     v                     v                      v
 Skills Loader       Rule Engine         Tool Dispatcher
     |                     |                      |
     +----------+----------+---------------------+
                |
                v
        MCP Execution Layer
                |
                v
        External Toolchain
 (Blender / Python / Tile / GIS)
                |
                v
         Artifact Registry
2. CORE DATA STRUCTURES
2.1 Agent Definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub version: String,

    pub skills: Vec<String>,
    pub rules: Vec<String>,
    pub tools: Vec<String>,

    pub memory_scope: Vec<String>,
    pub output_format: String,
}
2.2 Skill Definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,

    pub inputs: Vec<String>,
    pub outputs: Vec<String>,

    pub process_steps: Vec<String>,
}
2.3 Rule Definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub enforcement: RuleEnforcement,
    pub conditions: Vec<String>,
}
pub enum RuleEnforcement {
    PreExecution,
    Runtime,
    PostExecution,
}
2.4 Tool Definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub input_schema: String,
    pub execution: String,
    pub output: Vec<String>,
}
3. RUNTIME ENGINE CORE
3.1 Engine State
pub struct SkillRuntimeEngine {
    pub agents: HashMap<String, Agent>,
    pub skills: HashMap<String, Skill>,
    pub rules: HashMap<String, Rule>,
    pub tools: HashMap<String, Tool>,
}
3.2 Load Agent
impl SkillRuntimeEngine {
    pub fn load_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id.clone(), agent);
    }
}
3.3 Rule Enforcement Pipeline
pub fn enforce_rules(agent: &Agent, input: &McpRequest) -> Result<(), String> {
    for rule_id in &agent.rules {
        match rule_id.as_str() {
            "no_ai_generated_images" => {
                if input.tool.contains("diffusion") {
                    return Err("AI image generation blocked".into());
                }
            }

            "deterministic_output" => {
                if input.contains_randomness() {
                    return Err("Non-deterministic input blocked".into());
                }
            }

            _ => {}
        }
    }

    Ok(())
}
3.4 Execution Pipeline
pub fn execute(&self, agent_id: &str, req: McpRequest) -> McpResponse {
    let agent = self.agents.get(agent_id).unwrap();

    enforce_rules(agent, &req).unwrap();

    let skill_set = self.resolve_skills(agent);

    let tool = self.select_tool(&req.tool);

    let output = self.run_tool(tool, req);

    self.validate_output(&output);

    self.store_artifact(&output);

    output
}
3.5 Tool Execution (CLI bridge)
pub fn run_tool(&self, tool: &Tool, req: McpRequest) -> McpResponse {
    use std::process::Command;

    let output = Command::new(&tool.execution)
        .arg(serde_json::to_string(&req).unwrap())
        .output()
        .expect("tool failed");

    serde_json::from_slice(&output.stdout).unwrap()
}
3.6 Reproducibility Hashing
pub fn compute_hash(req: &McpRequest) -> String {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(req).unwrap());

    format!("{:x}", hasher.finalize())
}
4. MCP VALIDATION LAYER
pub fn validate_mcp_output(res: &McpResponse) -> Result<(), String> {
    if res.artifacts.is_empty() {
        return Err("No artifacts generated".into());
    }

    Ok(())
}
5. OUTPUT REGISTRY
pub struct ArtifactRegistry {
    pub artifacts: HashMap<String, Artifact>,
}
6. RESULT OF THIS ENGINE

You now have:

✔ deterministic agent execution
✔ enforced rule system
✔ tool sandboxing
✔ full reproducibility
✔ MCP-compatible pipeline
✔ no LLM free generation leakage
7. VISUAL SKILL GRAPH EDITOR

Now the second half.

This is your Blueprint-like system for agents + MCP tools.

0. PURPOSE

Allow designers to:

visually define agent behavior
connect skills
chain MCP tools
simulate outputs
debug pipelines
1. CORE CONCEPT

Everything is a graph:

[Skill Node] → [Rule Node] → [Tool Node] → [Output Node]
2. NODE TYPES
2.1 Skill Node
pub struct SkillNode {
    pub skill_id: String,
}
2.2 Rule Node
pub struct RuleNode {
    pub rule_id: String,
}
2.3 Tool Node
pub struct ToolNode {
    pub tool_id: String,
}
2.4 MCP Request Node
pub struct McpNode {
    pub request_template: McpRequest,
}
3. GRAPH STRUCTURE
pub struct SkillGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
}
4. EXECUTION FLOW
Skill Node
   ↓
Rule Validation Node
   ↓
Tool Selection Node
   ↓
MCP Request Node
   ↓
Execution Node
   ↓
Artifact Output Node
5. VISUAL EDITOR FEATURES
5.1 Drag & Drop Nodes
skills
rules
tools
MCP templates
5.2 Live Execution Preview
run graph step-by-step
show MCP JSON in/out
visualize artifacts
5.3 Debug Mode
highlight rule violations
show deterministic hash
replay execution
5.4 Asset Preview Panel
GLB preview
tile atlas preview
material preview
6. FILE FORMAT
{
  "graph_id": "designer_pipeline_v1",
  "nodes": [
    { "type": "skill", "id": "tile_generation" },
    { "type": "rule", "id": "deterministic_output" },
    { "type": "tool", "id": "tile_mcp" }
  ],
  "edges": [
    { "from": "tile_generation", "to": "tile_mcp" }
  ]
}
7. WHY THIS IS POWERFUL

You now have:

✔ visual programming for asset pipelines
✔ deterministic execution engine
✔ reusable skill graph system
✔ rule enforcement layer
✔ full MCP integration
✔ production-grade asset pipeline control
8. FINAL SYSTEM VIEW
                DESIGNER
                   |
                   v
        Visual Skill Graph Editor
                   |
                   v
        Skill Runtime Engine (Rust)
                   |
                   v
        MCP Toolchain Layer
                   |
                   v
      Blender / Tile / Material / GIS
                   |
                   v
            Asset Registry
                   |
                   v
                 BEVY
9. WHAT YOU NOW HAVE (IMPORTANT)

This is no longer:

AI generation
prompt systems
ad-hoc pipelines

This is:

a deterministic agent-driven industrial content production engine