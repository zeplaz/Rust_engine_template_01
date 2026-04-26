# Processor Alpha Dine

A game engine written in Rust using Bevy for simulation with production, vehicles, and terrain systems.

## Project Structure

This codebase has been reorganized for better maintainability and clarity:

- **core**: Core utilities and ID generation
- **engine**: Game engine, state management, and logic models
- **entities**: Entity component system with specialized types
  - **production**: Production entities and resources
  - **structure**: Building structures and placement
  - **vehicles**: Vehicle entities and movement
- **gui**: User interface components and screens
- **io**: Input/output systems and serialization
- **render**: Rendering systems and shaders
- **systems**: Game systems for various mechanics
  - **collision**: Collision detection
  - **damage**: Damage handling
  - **navigation**: Movement and pathfinding
  - **production**: Resource production and consumption
- **terrain**: Terrain generation and management
- **traits**: Shared traits for game objects
- **utils**: Utility functions and events

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Dependencies as listed in Cargo.toml

### Building the Project

```bash
cargo build
```

### Running the Game

```bash
cargo run
```

## Development Notes

- Uses Bevy ECS for entity management
- Implements custom systems for production, navigation, and damage
- See REORGANIZATION.md for details on the codebase structure

## Future Improvements

- Complete implementation of production chains
- Enhance vehicle navigation
- Improve terrain generation
- Add additional building types
- Implement proper multiplayer support
- Improve asset loading and management