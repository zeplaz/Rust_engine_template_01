# Construction Ghost Overlay

**Repo:** `src/construction/` · **Invariants:** `src/dev/construction_invariants.md`

## Purpose

Preview placement and ghosts are **presentation** — single execute funnel commits sim.

## BuildGhostState (pattern)

Ghosts show:
- footprint, blocked tiles, orientation, validity
- must not merge into terrain or hide occupancy

## Hard rules (construction invariants)

| Rule | |
|------|--|
| Preview **never** mutates gameplay | |
| **Single** execute funnel for commit | |
| Logic stays in `src/construction/` | |
| Not part of `STAGE5_TODOS` exit | separate witness `debug_runs/construction_stage_live.json` |

## Layer boundary

| Layer | May |
|-------|-----|
| UI / ghost render | Draw overlay, read sim validity |
| Construction systems | Commit on execute |
| View authority | Camera/viewport only — not placement commit |

## Pick + draw projection (must align)

Ghosts and pick share **`ConstructionMapProjection`** + live camera frame when authoritative.

| Path | API |
|------|-----|
| Pick (production) | `cursor_world_xy_rendered` → `sim_map_cursor_world_xy_rendered` |
| Footprint draw | `world_to_egui_rendered` with same frame |
| Debug compare | `ConstructionPlacementDebugProbe` — Pick Δ world **< 1**, ghost screen Δ **< 4px** |

Full contract (fixed vs visible span, scissor heal, schedule): **[09-sim-map-projection-placement.md](09-sim-map-projection-placement.md)**.

## Anti-patterns

- Ghost system writing transport or building entities
- Overlay-driven “fake” placement validity
- Skipping construction book / corridor hydrate on load (see transport R8 + construction slice)
- Manual egui projection using `ortho.fixed_width` as world span (use `visible_w/h` / `view_pixels`)
- Pick/footprint before `SimulationViewportSyncSet::ApplyCameraScissor`

## Designer vs coder

- Readability / hatch / footprint UX → `@designer`
- Execute pipeline, ECS commit → `@coder` (this agent)
