MCP ASSET PIPELINE ARCHITECTURE
1.1 Core Flow
Agent (LLM)
   ↓
MCP Request (JSON)
   ↓
Python Toolchain Layer
   ↓
Blender / Material / Image / Tile Generator
   ↓
Asset Pack (GLB + PNG tiles + metadata)
   ↓
Bevy Import + ECS registry
2. MCP TOOL CATEGORIES (CRITICAL)

We split tools into 4 real pipelines:

2.1 Geometry MCP (Blender)
buildings
factories
props
vehicles
destroyed variants

CLI:

mcp_blender generate building.json
2.2 TILE MCP (ISOMETRIC SYSTEM CORE)

This is what you’re missing.

Tiles are NOT textures.

They are state machines.

Tile definition:
pub struct TileVariant {
    pub base: TileBaseType,
    pub state: TileState,
    pub damage: DamageState,
    pub power: PowerState,
    pub fullness: FillState,
    pub lighting: LightState,
}
Tile base types:
pub enum TileBaseType {
    WoodFloor,
    StoneFloor,
    ConcreteFloor,
    Dirt,
    Asphalt,
    MetalPlate,
}
Tile states:
pub enum TileState {
    Clean,
    Dirty,
    Damaged,
    Ruined,
}
Power state:
pub enum PowerState {
    Off,
    Partial,
    On,
}
Fill state (cargo / containers / storage):
pub enum FillState {
    Empty,
    Quarter,
    Half,
    Full,
}
Light state:
pub enum LightState {
    Day,
    NightOff,
    NightOn,
}
2.3 TILE MCP TOOL (CRITICAL)

Python tool:

tile_generator.py

Input:

{
  "tile": "factory_floor",
  "base": "concrete",
  "damage": 0.3,
  "power": "on",
  "fill": "half",
  "lighting": "night_on",
  "isometric": true
}

Output:

factory_floor_dmg30_power_on_fill_half.png
factory_floor_dmg30_power_on_fill_half_norm.png
2.4 TILE RENDER METHOD (IMPORTANT INSIGHT)

Instead of AI rendering:

We use:

Blender orthographic bake system
Top-down orthographic camera
+ fixed isometric angle
+ material override system
+ batch rendering
Blender Python tile renderer:
def render_tile(spec):
    set_camera_iso()

    apply_material(spec["base"])

    apply_damage_overlay(spec["damage"])

    apply_power_emission(spec["power"])

    apply_fill_mesh(spec["fill"])

    bpy.ops.render.render(write_still=True)
2.5 PROP MCP (SMALL OBJECT SYSTEM)

Used for:

crates
pipes
smoke stacks
wires
lights
cargo boxes
Prop spec:
{
  "type": "crate_stack",
  "count": 12,
  "variation": "industrial",
  "state": "half_loaded"
}
3. REPLICATING “REPUBLIC: THE REVOLUTION” STYLE SYSTEM

That game succeeded because:

buildings were modular + state-based
districts had visual “condition”
propaganda / economy changed visuals
limited asset set → many states

We replicate that with:

3.1 Building State System
pub struct BuildingVisualState {
    pub condition: f32,
    pub occupancy: f32,
    pub production: f32,
    pub power: PowerState,
}
3.2 Visual Layers (VERY IMPORTANT)

Every building = layered rendering:

Base structure
+ Damage overlay
+ Light overlay
+ Smoke overlay
+ Cargo overlay
+ Power emission
3.3 Example: Factory
Concrete shell
+ rust texture variation
+ smoke stack animation
+ glowing windows (power on/off)
+ cargo crates outside
+ trucks loading state
4. SMOKE / LIGHT / ANIMATION MCP

This is where “life” comes from.

4.1 Smoke system
{
  "type": "smoke_stack",
  "intensity": 0.7,
  "color": "dark_gray",
  "wind": [1.0, 0.2]
}
4.2 Light system
{
  "type": "building_light",
  "power": "on",
  "flicker": 0.1,
  "color": "warm_white"
}
4.3 Cargo animation system
{
  "type": "loading_dock",
  "state": "loading",
  "progress": 0.6
}
5. PYTHON TOOLCHAIN LAYER

This is your real production backbone.

5.1 Tools structure
tools/
  blender/
    build_building.py
    render_tile.py
    generate_props.py

  tile/
    tile_generator.py
    tile_batch.py

  props/
    cargo_generator.py

  utils/
    image_batch.py
    atlas_packer.py
5.2 Batch tile generator
for spec in load_specs():
    render_tile(spec)
5.3 Atlas packing (critical for Bevy)
python pack_atlas.py tiles/ output_atlas.png
6. MCP TOOL DEFINITIONS
6.1 Tile MCP
{
  "tool": "tile.generate",
  "input": {
    "base": "concrete",
    "damage": 0.4,
    "power": "on",
    "fill": "full",
    "lighting": "night_on"
  }
}
6.2 Building MCP
{
  "tool": "blender.generate_building",
  "input": {
    "type": "factory",
    "floors": 3,
    "style": "industrial",
    "state": "operational"
  }
}
6.3 Prop MCP
{
  "tool": "prop.generate",
  "input": {
    "type": "crate_stack",
    "amount": 10,
    "state": "half_loaded"
  }
}
7. BEVY INTEGRATION MODEL
7.1 Tile rendering system
#[derive(Component)]
pub struct Tile {
    pub variant: TileVariant,
    pub sprite: Handle<Image>,
}
7.2 State-driven swap system
fn update_tile_visuals() {
    if power_changed {
        swap_texture();
    }
}
8. WHY THIS SYSTEM WORKS

You now have:

✔ Republic-style visual state system
✔ Deterministic tile generation (no AI art)
✔ Fully batchable production pipeline
✔ Modular props + animation system
✔ MCP-controlled asset generation
✔ Bevy-ready atlas system
9. KEY INSIGHT (MOST IMPORTANT)

You are NOT generating assets.

You are generating:

STATE → TOOL → BATCHED ART OUTPUT

LLM only controls:

parameters
variation
requests

Everything else is deterministic.