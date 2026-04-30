# Processor Alpha Dine - Asset Editor

A modern, fluent-design asset editor for the Processor Alpha Dine game engine.

## Features

- Create and edit game assets (vehicles, buildings, power infrastructure)
- Manage textures and asset states
- Modern UI with Fluent design
- Graceful fallback to standard widgets when PyQt-Fluent-Widgets is not available

## Installation

1. Install Python dependencies:

```bash
pip install -r requirements.txt
```

2. Run the asset editor:

```bash
python run.py
```

## Usage

The asset editor is organized into several sections:

- **Vehicles**, **Buildings**, **Textures**, **World generation** (tuning JSON), **Materials**, **Tags**, **Material rules** (terrain JSON/RON), **Transport** (roads & rails), **Power**
- **Content paths**: Python tools resolve `assets/` via `src/repo_paths.py`. See: [`06_…`](../../../prompts/designer_questions/tools_ui/spec/06_asset_content_studio_workflow_v1.md), [`07_…`](../../../prompts/designer_questions/tools_ui/spec/07_asset_editor_dual_chain_audit_v1.md), [`08_world_gen_desktop_tool_v1.md`](../../../prompts/designer_questions/tools_ui/spec/08_world_gen_desktop_tool_v1.md).

## Content workflow (artists & designers)

### Creating a Vehicle

1. Navigate to the Vehicles section
2. Enter a name for the vehicle
3. Configure vehicle properties (type, capacity, mass, max speed)
4. Select a texture in the Textures section
5. Click "Save Vehicle"

### Creating a Building

1. Navigate to the Buildings section
2. Enter a name for the building
3. Configure building properties (size, height, construction cost)
4. Add resources produced by the building
5. Select a texture in the Textures section
6. Click "Save Building"

### Creating Power Infrastructure

1. Navigate to the Power section
2. Enter a name for the power building
3. Configure building properties
4. Configure power properties (generation, consumption, storage)
5. Select a texture in the Textures section
6. Click "Save Power"

### Managing Textures

1. Navigate to the Textures section
2. Browse the game assets or import external textures
3. Preview textures before assigning them to assets
4. Map different texture states (midday, night, destroyed, etc.)

### Roads, rails, and building-type index

1. Open **`assets/configs/buildings/_building_types_index.json`** (button on **Transport** page) to see how tool labels map to `BuildingType` / roads surfaces in Rust.
2. Use the **Transport** section: **Roads** and **Rails** tabs list JSON under `assets/configs/roads` and `assets/configs/rails`; edit, **Save**, or **New from example** (clones the `*_v1` example).
3. Reusable widgets (`ClickableComboBox`, etc.) live in `src/components/editor_widgets.py`.

### Terrain materials, tags, and rules

1. Open **Materials**, **Tags**, or **Rules** in the sidebar (after World generation).
2. **Load example** → **Save active** writes under `assets/config/terrain/` (`material_registry.json`, `tag_registry.json`, `material_rules.ron`). Engine `AssetLoader` integration is **Pending** (matrix **U3**).
3. Docs: [`material_tag_rule_system_v1.md`](../../../prompts/designer_questions/terrain_world/material_tag_rule_system_v1.md), [`08_world_gen_desktop_tool_v1.md`](../../../prompts/designer_questions/tools_ui/spec/08_world_gen_desktop_tool_v1.md).

### World generation tuning

1. Open **World generation** in the sidebar (map/globe icon).
2. Use **Overview** for workflow; **Noise sampling** / **Biome tuning** for split editors (full biome object vs partial F8 UI); **Full JSON** for the whole `WorldGenTuningOverlay`.
3. **Save active** writes `assets/config/world_gen_tuning.json` (same path as **F8** and `WORLD_GEN_TUNING_JSON_PATH`).
4. Docs: [`08_world_gen_desktop_tool_v1.md`](../../../prompts/designer_questions/tools_ui/spec/08_world_gen_desktop_tool_v1.md), designer [`composite_style_worldgen_v1.md`](../../../prompts/designer_questions/terrain_world/composite_style_worldgen_v1.md), matrix [`composite_style_preview_integration_matrix_v1.md`](../../../prompts/matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md).

## Development

This tool is built with:
- PyQt5 for the base UI
- PyQt-Fluent-Widgets for modern styling (optional)
- Custom asset configuration system

The code is organized as follows:

- `src/`: Main source code
  - `config/`: Configuration classes and constants
  - `pages/`: UI pages for different asset types
  - `components/`: Reusable UI components
- `run.py`: Entry point

## License

This project is part of the Processor Alpha Dine game engine.