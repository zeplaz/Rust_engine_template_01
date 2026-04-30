# Logistics & unit coupling (surface to theatre) `05`

**Platforms detail:** `strategic_platforms/spec/01_platform_taxonomy.md`, `strategic_platforms/spec/04_supply_cargo_magazines.md`.

## Categories on the world map

| Category | World anchor | Notes |
|:---|:---|:---|
| **Buildings** | City cell / tile footprint | factories, depots, turrets |
| **Road vehicles** | `navigation` graph | trucks, haulers |
| **Supply drones / UAS** | air corridors + pads | cargo vs armed variants |
| **Ships & boats** | **ports** from §04 + water cells | hydrology coupling |
| **Cargo** | convoys, pallets, containers | production I/O |
| **Weapons** | magazines, hardpoints | EW/sensors — strategic `spec/02–03` |

## Correct placement rules

- **Berth depth / draft** vs bathymetry 📎.
- **Runway length** vs terrain slope 📎.
- **Clearance** for drone corridors above terrain + structures.

## Supply flows

- **Hub-and-spoke** from cities to front lines; damage or closed borders trigger `navigation` replan.

## LOD

- Entity promotion when **fires** or **radar** matter — pair `strategic_platforms` with `simulation_lod_v1.md`.
