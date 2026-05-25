# VM-09 / PROJ-2 audit (IN-C01, IN-C02)

**Date:** 2026-05-23  
**Goal:** Route pose/projection through view authority; reduce raw `MapCameraDesired` in render consumers.

## `MapCameraDesired` readers (render / gui — sample)

| Area | File | Status |
|------|------|--------|
| View bridge | `view_authority.rs`, `view_runtime/bridge.rs` | **Authoritative shim** — keep |
| RTS input | `map_camera.rs` | **Input writer** — keep |
| Tile fallback focus | `tile_world_fallback.rs` | **Review** — should use view focus API |
| Diagnostics | `full_render_diagnostic.rs`, `visual_diagnostics.rs` | **Read-only** — OK |
| GPU particles | `gpu_particles.rs` | **Migrated** (2026-05-24, INFRA-VM09-001) — `sync_fire_particle_camera_scale` uses `camera_zoom(ViewId::WorldMain)` with `MapCameraDesired` fallback |
| Minimap intent | `view_representation.rs` | **Infrastructure risk** — documented in status snapshot |

## Next sweep (incremental)

1. ~~`gpu_particles.rs` zoom~~ — **done** (`sync_fire_particle_camera_scale`).
2. `view_representation.rs` `sync_camera_visual_state_from_map_camera` — still reads `MapCameraDesired` (presentation FX band).
3. `tile_world_fallback.rs` — focus API review per audit table.
4. Grep `world_to_screen` outside `view_projection_authority.rs` — **INFRA-PROJ2-001**.
5. Refresh `infrastructure_view_isolation_live.json` after each slice.

## PROJ-2

Projection helpers live in `view_projection_authority.rs`. New UI hit-test code must call those entry points, not rebuild matrices from `MapCameraDesired` alone.
