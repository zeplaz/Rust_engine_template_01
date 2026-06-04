# Blender Geometry — Reference

Merged from [`prompts/MCP/mcp_drafts.md`](../../prompts/MCP/mcp_drafts.md) §2.1, §3, §5–6 and shipped [`tools/mcp/`](../../tools/mcp/).

## Geometry MCP (Blender)

**Use for:** buildings · factories · props · vehicles · destroyed variants

**CLI (draft):** `mcp_blender generate building.json`

**Shipped:** `python -m rust_engine_mcp.cli run-geometry <job.json>`

## Building MCP (draft)

```json
{
  "tool": "blender.generate_building",
  "input": {
    "type": "factory",
    "floors": 3,
    "style": "industrial",
    "state": "operational"
  }
}
```

**Shipped equivalent:** Compose from module ops (`module_wall`, `module_roof`, `module_door`) via separate jobs or future composite job type.

## BuildingVisualState (sim → art bridge)

```rust
pub struct BuildingVisualState {
    pub condition: f32,
    pub occupancy: f32,
    pub production: f32,
    pub power: PowerState,
}
```

## Visual layers (Republic-style)

Every building = layered rendering:

1. Base structure
2. Damage overlay
3. Light overlay
4. Smoke overlay
5. Cargo overlay
6. Power emission

### Example: Factory

- Concrete shell
- Rust texture variation
- Smoke stack animation
- Glowing windows (power on/off)
- Cargo crates outside
- Trucks loading state

## Prop MCP (draft)

```json
{
  "tool": "prop.generate",
  "input": {
    "type": "crate_stack",
    "count": 12,
    "variation": "industrial",
    "state": "half_loaded"
  }
}
```

**Use for:** crates · pipes · smoke stacks · wires · lights · cargo boxes

## Smoke / light / animation MCP (draft)

### Smoke

```json
{
  "type": "smoke_stack",
  "intensity": 0.7,
  "color": "dark_gray",
  "wind": [1.0, 0.2]
}
```

### Light

```json
{
  "type": "building_light",
  "power": "on",
  "flicker": 0.1,
  "color": "warm_white"
}
```

### Cargo animation

```json
{
  "type": "loading_dock",
  "state": "loading",
  "progress": 0.6
}
```

## Blender geometry skill process (draft contract)

From `rules_skills_draft.md`:

1. validate_grid_alignment
2. select_procedural_node_graph
3. build_geometry_nodes_tree
4. apply_materials
5. bake_lods
6. export_glb

## Shipped headless layout

```text
tools/mcp/blender/
  headless_run.ps1
  scripts/
    run_job.py
    ops/
      module_wall.py
      module_roof.py
      module_door.py
      export_glb.py
```

## Adding a new operation

1. Create `tools/mcp/blender/scripts/ops/<op>.py`
2. Register in `run_job.py`
3. Add schema params to `geometry_job_v1` or op-local validation
4. Document in `MICRO_TOOLS_REGISTRY_v1.md`
5. Add example job under `tools/mcp/schemas/examples/`

## Validation + promotion

```powershell
python -m rust_engine_mcp.cli validate-glb assets/staging/<job_id>/model.glb
python -m rust_engine_mcp.cli promote <job_id>
```

Checks: header · vertex budget · naming (extend per exec plan).

## Skill inputs/outputs (draft contract)

**Inputs:** footprint · floors · style · material_palette · state

**Outputs:** glb_model · lod_levels · collision_mesh

LOD/collision: **planned** (Tier 3 gltfpack / art_validator).

## Grid alignment

- Fixed module unit from procedural module kit
- Bottom-center pivot on vertical modules
- No free rotation in isometric presentation layer

## Related

- Orchestration: [mcp-asset-pipeline](../mcp-asset-pipeline/SKILL.md)
- Rules: [mcp-production-rules](../mcp-production-rules/SKILL.md)
- Module kit: [`design_procedural_module_kit_v1.md`](../../src/dev/design_procedural_module_kit_v1.md)
