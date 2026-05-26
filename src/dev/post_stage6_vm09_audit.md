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
| Camera visual / FX band | `view_representation.rs` | **Migrated** (2026-05-25, TRIAGE-VM-09-CODER-B) — `resolve_world_main_camera_scale` + `sync_camera_visual_state_from_map_camera` |
| Minimap intent | `view_representation.rs` | **Infrastructure risk** — documented in status snapshot (pose only; not slice 2) |

## TRIAGE-VM-09-CODER-B — steward sign-off (2026-05-25)

| Item | Evidence |
|:---|:---|
| Resolver | `resolve_world_main_camera_scale` in `view_representation.rs` |
| Consumer | `sync_camera_visual_state_from_map_camera` |
| Unit test | `vm09_slice2_resolve_world_main_scale_prefers_view_manager` |
| Witness | `infrastructure_view_isolation_live.json` → `vm_09.triage_vm09_coder_b_green: true` |
| Steward | **STEWARD-VM-09-001** slice 2+ **CLOSED** — handoff **INFRA-PROJ2-001** |

## Next sweep (incremental)

1. ~~`gpu_particles.rs` zoom~~ — **done** (slice 1 / INFRA-VM09-001).
2. ~~`view_representation.rs` camera visual zoom~~ — **done** (**TRIAGE-VM-09-CODER-B**).
3. ~~`tile_world_fallback.rs` minimap click~~ — **done** (**INFRA-PROJ2-001**).
4. ~~`world_preview/interaction.rs` hover~~ — **done** (**INFRA-PROJ2-001**).
5. Grep remaining `map_surface_screen_to_world` call sites — `map_camera.rs` legacy sim HUD (review).
5. Refresh `infrastructure_view_isolation_live.json` after each slice.

## PROJ-2 — bypass inventory (STEWARD-VM-09-001)

**Planner rollup:** [`infra_proj2_sole_writer_plan_v1.md`](infra_proj2_sole_writer_plan_v1.md) (**PLAN-INFRA-PROJ2-001** · **INFRA-PROJ2-CODER-B**).

Projection helpers live in `view_projection_authority.rs`. Prefer `view_surface_world_to_screen` / `view_surface_screen_to_world` with `ViewId`.

| File | API | View context | Steward verdict |
|:---|:---|:---|:---|
| `view_projection_authority.rs` | `view_surface_*` | Per `ViewId` | ✅ **canonical** |
| `map_view_projection.rs` | `map_surface_*` | Low-level math | ✅ keep (called by authority) |
| `tile_world_fallback.rs` | `view_surface_screen_to_world` | Minimap egui click | ☑ **INFRA-PROJ2-001** — `ViewId::Minimap` (+ presentation fallback) |
| `editor/world_preview/interaction.rs` | `view_surface_screen_to_world` | World Preview panel | ☑ **INFRA-PROJ2-001** — `ViewId::WorldPreview` |
| `map_camera.rs` | `sim_map_screen_to_world_xy` | Legacy sim HUD | ☐ review — may stay until sim HUD uses `MapViewInteractionByView` only |

New UI hit-test code must call view authority entry points, not rebuild matrices from `MapCameraDesired` alone.
