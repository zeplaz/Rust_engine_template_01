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

- **Vehicles**: Create and configure vehicle assets
- **Buildings**: Create and configure building assets
- **Power**: Create and configure power infrastructure
- **Textures**: Browse, preview, and map textures to assets

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