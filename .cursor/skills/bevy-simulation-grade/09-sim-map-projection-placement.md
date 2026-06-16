# Sim Map Projection + Construction Placement

**SYMLANG packet:** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` §6 `⟨MAP-PICK⟩` — compress live overlay metrics, not cargo perf logs.

**Repo:** `src/gui/map_camera.rs` · `src/construction/map_egui_projection.rs` · `src/construction/placement_debug.rs`  
**Invariants:** `src/dev/construction_invariants.md` (preview ≠ commit)

## Purpose

Tactical map **pick** (world tile under cursor) and **ghost draw** (egui overlay) must share one projection contract. When they diverge, placement looks “detached” from the OS cursor even though the camera roundtrips cleanly.

## Authority spine (who owns what)

```text
⊚ViewProjectionAuthority ◂⊳ commit_pose (input / minimap shell)
  ═▶ ⊚MapCameraDesired (mirror — compat read surface only)
  ═▶ ⊚MainWorldCamera Transform + Projection + Camera::viewport
  ═▶ ⊚SimMapProjectionFrame (per-frame snapshot for manual egui math)
```

| Surface | Writer | Readers |
|---------|--------|---------|
| `ViewProjectionAuthority` | `commit_pose` / map input | `ConstructionMapProjection::resolve` |
| `MapCameraDesired` | `derive_map_camera_desired_from_view_authority` | Legacy egui hole math, presentation mirror |
| `Camera::viewport` + ortho | `sync_main_world_camera_viewport_and_projection` | GPU render, `primary_cursor_world_xy` |
| `MainWorldCameraOrthoTrace` | same sync system | `sim_map_projection_frame`, debug probe |
| `ConstructionPlacementDebugProbe` | `sync_construction_placement_debug_probe` | Debug overlay only (PostUpdate) |

**Schedule:** construction pick + footprint + debug probe run **after** `SimulationViewportSyncSet::ApplyCameraScissor` (`src/construction/mod.rs` PostUpdate chain).

## Dual projection paths (do not mix spans)

| Path | When | API | World span used |
|------|------|-----|-----------------|
| **Camera authoritative** | `SimMapProjectionFrame::camera_authoritative` | `primary_cursor_world_xy` / `world_to_viewport` | Bevy ortho + live viewport |
| **Manual egui in-frame** | fallback or debug “egui inverse” line | `sim_map_screen_to_world_xy_in_frame` | **`visible_w/h`** (= `ortho.view_pixels` world span) |
| **Legacy hole** | no live camera | `sim_map_screen_to_world_xy` | `sim_map_visible_world_span` on map hole rect |

### Critical nuance: `fixed_w/h` ≠ `visible_w/h`

```text
fixed_w/h   = ScalingMode::Fixed params written to camera  (= view_px / zoom)
visible_w/h = world units across screen_rect               (= view_px ≈ ortho.view_pixels)
```

Manual `sim_map_*_in_frame` **must** use `visible_w/h`. Using `fixed_w/h` causes ~zoom× world error (e.g. Pick Δ ≈ 77–374 at zoom ~1.66–4).

### Critical nuance: `view_px` vs `latch_hole`

`MainWorldCameraViewportLatch.using_hole` can stay true while scissor **heals** (`camera.viewport = None`, full-window GPU).

```text
view_px = camera.viewport.physical_size / scale_factor   if viewport Some
        = window logical size                            if viewport None (scissor healed)
```

**Never** size ortho from `sim.logical_size()` when the GPU draws full window.

## `SimMapProjectionFrame` fields

| Field | Meaning |
|-------|---------|
| `screen_rect` | egui rect: map hole if scissor matches hole, else full window |
| `fixed_w`, `fixed_h` | ortho trace fixed dimensions (debug / Bevy Fixed params) |
| `visible_w`, `visible_h` | world span for manual projection math |
| `camera_authoritative` | prefer camera unproject/project when true |

`camera_authoritative` when: camera active AND (hole scissor matches OR full window OR no viewport).

## Presentation pose

`map_camera_pose_for_presentation` — live `GlobalTransform` translation/rotation + `MapCameraDesired` scale. Use for manual path when transform has caught up; camera path uses live transform directly.

## Construction consumers

| Consumer | Pick | Draw |
|----------|------|------|
| `build_pick_ghost_tile_system` | `ConstructionMapProjection::cursor_world_xy_rendered` | — |
| `draw_construction_visual_requests_egui` | — | `world_to_egui_rendered` → fallback `sim_map_world_vec3_to_egui` |
| Debug overlay | probe compares camera vs manual | crosshairs: white cursor, magenta camera, green egui |

## Debug witness (operator + agents)

**Overlay:** `--test vfx` / `--test visual` / `CONSTRUCTION_PLACEMENT_DEBUG=1`  
**Resource:** `ConstructionPlacementDebugProbe` (`src/construction/placement_debug.rs`)

| Metric | Green | Indicates |
|--------|-------|-----------|
| `Pick Δ world` (cam vs manual) | **< 1** | in-frame span + screen_rect correct |
| `Ghost screen Δ` (camera vs egui) | **< 4px** | footprint draw aligned |
| `Pick roundtrip screen Δ (cam)` | **< 4px** | camera path self-consistent |
| `camera_authoritative` | true + full viewport | using live camera pick |
| `Ortho fixed world` | view_px/zoom | not the manual span |
| `Projection visible` | ≈ view_px | manual span source |

**Symptom → likely cause**

| Symptom | Check |
|---------|-------|
| Large Pick Δ, roundtrip ok | manual path using `fixed_w` instead of `visible_w` |
| Large Pick Δ + wrong tile | `view_px` still hole-sized while viewport full window |
| Ghost Δ large, Pick ok | footprint still on legacy hole rect vs full-window frame |
| Everything wrong after “fix” | **`map_camera.rs` empty/truncated** — rebuild; stale binary |

## Anti-patterns

- Using `MapCameraDesired` alone for pick while GPU uses healed full-window scissor
- Sizing manual projection from `sim.logical_size()` when `camera.viewport` is `None`
- Treating `ortho.fixed_width` as visible world span in egui inverse math
- Pick in Update before `ApplyCameraScissor` / camera sync
- Second placement writer outside `src/construction/`

## Escalation

| Role | When |
|------|------|
| `@coder` | fix projection math, schedule, or consumer wiring |
| `@designer` | footprint readability, hatch, crosshair UX (not pick authority) |
| debug-intelligence | compress probe + witness into routing YAML |
