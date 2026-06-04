# Tile Generation — Reference

Merged from [`prompts/MCP/mcp_drafts.md`](../../prompts/MCP/mcp_drafts.md) §2.2–2.5, §7–8.

## Core insight

Tiles are **NOT** textures. They are **state machines**.

## TileVariant (target Rust)

```rust
pub struct TileVariant {
    pub base: TileBaseType,
    pub state: TileState,
    pub damage: DamageState,  // or f32 scalar 0.0–1.0 in MCP JSON
    pub power: PowerState,
    pub fullness: FillState,
    pub lighting: LightState,
}
```

## Enums

### TileBaseType

```rust
pub enum TileBaseType {
    WoodFloor,
    StoneFloor,
    ConcreteFloor,
    Dirt,
    Asphalt,
    MetalPlate,
}
```

### TileState

```rust
pub enum TileState {
    Clean,
    Dirty,
    Damaged,
    Ruined,
}
```

### PowerState

```rust
pub enum PowerState {
    Off,
    Partial,
    On,
}
```

### FillState

```rust
pub enum FillState {
    Empty,
    Quarter,
    Half,
    Full,
}
```

### LightState

```rust
pub enum LightState {
    Day,
    NightOff,
    NightOn,
}
```

## MCP JSON example

```json
{
  "tool": "tile.generate",
  "input": {
    "tile": "factory_floor",
    "base": "concrete",
    "damage": 0.3,
    "power": "on",
    "fill": "half",
    "lighting": "night_on",
    "isometric": true,
    "seed": 42
  }
}
```

## Outputs

```text
factory_floor_dmg30_power_on_fill_half.png
factory_floor_dmg30_power_on_fill_half_norm.png
```

Naming: encode state axes in filename for deterministic lookup.

## Render method (Blender orthographic bake)

```python
def render_tile(spec):
    set_camera_iso()
    apply_material(spec["base"])
    apply_damage_overlay(spec["damage"])
    apply_power_emission(spec["power"])
    apply_fill_mesh(spec["fill"])
    bpy.ops.render.render(write_still=True)
```

- Top-down orthographic + fixed isometric angle
- Material override system
- Batch rendering loop — no diffusion

## Batch loop

```python
for spec in load_specs():
    render_tile(spec)
```

Then:

```bash
python pack_atlas.py tiles/ output_atlas.png
```

## Republic-style replication

Limited asset set → many states via:

- Modular buildings + district condition
- Economy/power changing visuals
- State axes (damage, power, fill, lighting)

## Bevy integration (target)

```rust
#[derive(Component)]
pub struct Tile {
    pub variant: TileVariant,
    pub sprite: Handle<Image>,
}

// On sim state change:
// if power_changed { swap_texture_from_atlas(variant_key); }
```

Atlas UV metadata in `atlas_meta.json` for O(1) handle swap.

## Skill process steps (draft contract)

From `rules_skills_draft.md`:

1. validate_inputs
2. select_tile_template
3. apply_state_layers
4. render_isometric_bake
5. export_png
6. register_to_atlas

## Planned toolchain paths

```text
tools/tile/tile_generator.py   # single variant
tools/tile/tile_batch.py       # spec list runner
tools/utils/atlas_packer.py    # Bevy-ready atlas
```

**Status:** Spec only — implement per exec plan Phase 2+.

## Related

- Rules: [mcp-production-rules](../mcp-production-rules/SKILL.md) — batch + grid required
- Orchestration: [mcp-asset-pipeline](../mcp-asset-pipeline/SKILL.md)
