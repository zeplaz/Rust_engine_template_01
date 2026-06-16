# Strategic platforms — taxonomy `01`

## Classes (v1 checklist)

| Class | Movement / nav | Notes |
|:---|:---|:---|
| **Fixed building** | None | Turret, launcher, radar face, C2 node |
| **Road/rail vehicle** | `navigation/spec/` | trucks, trains |
| **Supply drone / UAS** | air corridor graph 📎 | low-altitude cargo vs strike variant |
| **Ship / boat** | water mesh + ports | couple hydrology / ports (`terrain_world/spec/02_terrain_hydrology_worldgen.md`) |
| **Cargo skiff / barge** | same | unarmed or light arm |
| **Manned aircraft** | high-alt LOD 📎 | optional phase 2 |

## Serializable

- **PlatformBlueprint** / catalog row: role, hardpoints, sensor slots, cargo volume, fuel 📎.

## Overlap terrain

- **Parking / apron / berth** slots attach to **city graph** or manual markers (`terrain_world/spec/04_cities_ai_settlements.md`).
