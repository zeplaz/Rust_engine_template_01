# Phase variance — respond before you code

**Not a program.** Do not register a plan id, a queue, or a second place funnel.  
**Read first:** [`debug_runs/phase_variance_critique_packet.json`](../../debug_runs/phase_variance_critique_packet.json)  
**Date:** 2026-09-25 · branch `master` · planner critique, production code untouched.

Engine, art catalog, and HUD each keep a private idea of “what phase this cell is in.” The pixels agree with whichever painter ran last. This note is the gate those painters have to pass.

## Procedure

Use it on every ghost, kit, witness, AI marker, or icon change. One variance, one pass. Stop after the witness line.

1. **Observe phase.** Name the resource and the pixel. Ghost mode, `SiteConstructionPhase`, module `development_tier`, and HUD icon size are different clocks. Write down which one you touched.
2. **Name the lie.** Say what a player would believe, then the authority that contradicts it. “Looks planned,” “looks built,” “witness says green,” “icon is the footprint.”
3. **One owner.** The system that already writes that authority. A second writer is the bug, not a collaboration.
4. **One fix.** Change that owner so the pixel matches the authority. Reuse `CommitConstructionSiteEvent`. Player tool stays `ActiveBuildTool`. Queue stays `PendingConstructionQueue`.
5. **Witness.** Point at an existing file. If the claim is visual, `green: true` without `pixel_regression_green: true` is a lie. Lib counters are not pixels.

`@designer` lane `5273fb2c` owns place-feel chrome. `@coder` lane `944715db` may draw haul lines and AI intent only under the contract below. Neither lane opens a new commit path.

## Authority (do not invert)

```text
⊚ActiveBuildTool ═▶ player tool
⊚BuildGhostState ═▶ cursor preview          ⛔ gameplay mutation
⊚CommitConstructionSiteEvent ═▶ site birth   phase starts Planned
⊚ConstructionSite.phase ═▶ committed phase
⊚phase pixels ⊰ ⊚ConstructionSite.phase
⊚player footprint fill ⊰ ⊚BuildGhostState ∩ catalog FootprintMatrix
⊚ProceduralModuleRegistry ═▶ mesh tier       atlas stamp = production only
⊚AI marker ⊰ ⊚GrowthProposal                 ⛔ footprint_tiles
```

## PV-GHOST-COMMIT-PHASE

| Step | Response |
|---|---|
| Observe | `BuildGhostState` is Place/Adjust, not a construction phase. Commit in `strategic/site/systems.rs` writes `SiteConstructionPhase::Planned`. `phase_visual.rs` paints that phase in egui. `site_phase_tile_instances.rs` paints it again and stores the phase in `TileDebugInstance.lod` (Planned = 0). |
| Lie | Place-settle blue is the Planned family (`ghost_visual.rs`). A valid ghost is almost operational green. The GPU “lod” is a construction phase. The egui painter bails out when the instanced path is on, so one of the two drawings disappears without a phase change. |
| Owner | `@coder` construction visuals: `visual_authority`, `phase_visual`, `site_phase_tile_instances`. |
| Fix | Ghost colors stay valid / risky / invalid and stop at commit. Committed cells use phase color only. Stop stuffing phase into `lod`. No new painter. The scaffold exit is already `overlay_matrix.construction_phase`. |
| Witness | `debug_runs/construction_stage_live.json` can be schema-green and still show this lie. Do not close on it. Record `ghost_fill != planned_fill` before calling the map honest. |

## PV-LOD0-VS-PRODUCTION

| Step | Response |
|---|---|
| Observe | Live `_module_index.json` and `_module_index.ron`: **54 production, 0 lod0, 0 smoke.** `DevelopmentTier::atlas_runtime_stamp_allowed` is production only. `prefer_stylepack_tier` still accepts lod0 when production is missing. `visible_in_stylepack` hides smoke and shows lod0. |
| Lie | `_fragments/plan_program_registry_v1.md` still forbids re-litigating “30 prod / 50 lod0 / 31 smoke.” `parametric_commit.rs` still calls BUILD-READ-VISUAL-001 a lod0 extract. Retired lod0 and greybox kits sit untracked under `assets/archive/`. Agents will “finish” a kit phase the catalog has already left, or ship lod0 the moment a `kit_lod0_*` row returns. |
| Owner | Index contents: `@designer-mcp`. Tier filter in `procedural/module_index.rs`: `@coder`. |
| Fix | Visibility matches the atlas rule: production, or no mesh. Leave lod0 and greybox in the archive. Correct the registry counts. Make the visual witness name `source_tier` of the glb it assembled. |
| Witness | Recount `development_tier` on the ron/json index. `build_read_visual_001_witness_green` is a lib check until that tier is in the body. |

## PV-WITNESS-VS-PIXELS

| Step | Response |
|---|---|
| Observe | `WIT-PIXELS-DISHONEST` fails only on an **explicit** `pixel_regression_green: false` or `lod_chrome_leak: true`. Omitted pixel fields pass. `proposal_ghost_witness_green` asserts a vec length inside a headless `App`. |
| Lie | Green means the ghost was drawn, the kit is on the map, or the phase tile is the right color. It does not. |
| Owner | `@coder-mcp`, existing `witness_honesty.py` rule. |
| Fix | A visual claim with `green: true` requires `pixel_regression_green` present and true. Same rule, missing field included. Until that ships, set `green: false` on visual claims. |
| Witness | `tools/mcp/schemas/witness_integrity_rules_v1.json` · `WIT-PIXELS-DISHONEST`. This critique packet is `green: false` on purpose. |

## PV-AI-INTENT-VS-PLAYER-GHOST

| Step | Response |
|---|---|
| Observe | Construction Update clears `ConstructionVisualRequests` and refills footprint tiles from the player ghost. `push_proposal_ghosts_to_visual_requests` appends those same tiles as `FootprintTileColorKind::Risky` (weight 0.45) from the growth queue. The two Update chains are not ordered. `944715db` may add more draw. |
| Lie | An AI idea looks like the player’s risky ghost, or like a road preview, and it may commit if the draw system also writes the queue. |
| Owner | The illegal append already in tree: `src/strategic/settlement/policy.rs`. Draw added by `944715db` follows the contract and does not become a second owner of the player buffer. |
| Fix | Proposals stop writing `footprint_tiles`. Markers and haul lines live on a list the construction `clear()` does not own. Colors are not ghost, phase, or road tokens. Overlap: player fill wins; AI drops to a tick outside the cell. |
| Witness | Player `footprint_tiles` contain no proposal anchors. The draw system’s `CommitConstructionSiteEvent` count stays zero. `proposal_ghost_witness_green` does not count. |

### Contract for AI intent draw (`944715db`)

May draw:

- Freight polylines that already exist on the logistics authority.
- An unfilled mark on a `GrowthProposal` anchor.

Must not:

- Write `BuildGhostState`, `ActiveBuildTool`, or `BuildPlacementPreview`.
- Append to `ConstructionVisualRequests` (`footprint_tiles`, `paths`, `zone_tiles`, `corridor_paths`).
- Reuse valid / risky / invalid fills, place-settle blue, `phase_color`, or road preview / committed road colors.
- Emit `CommitConstructionSiteEvent` or push `PendingConstructionQueue` from a painter.
- Spawn `ConstructionSite` or a road entity.
- Store intent in `TileDebugInstance.lod`.

If an AI building is actually placed, it already has a door: `approve_growth_proposal_into_pending` → `PendingEntryKind::BuildSite` → the existing commit. Painters do not call that door.

## PV-HUD-ICON-VS-FOOTPRINT

| Step | Response |
|---|---|
| Observe | HUD `icon_size` is 16px. Power icons are an egui atlas. The map ghost uses catalog width×depth and `tile_screen_extent`. `sync_strategic_icon_instances_scaffold` emits one icon at world origin, size 12, in the macro band. |
| Lie | The button’s pixel box is the plot. The origin icon is a building. |
| Owner | Chrome: `@designer` (`5273fb2c`). Occupied cells: catalog `FootprintMatrix` on `BuildGhostState` when the tool is armed. Macro scaffold: `@coder` render, not a footprint owner. |
| Fix | Keep icon pixels in the tray. Ghost and commit share the armed footprint. Macro icons draw at committed sites or draw nothing. Do not scale tiles to `icon_size`. |
| Witness | Ghost tile count equals catalog occupied cells. `ConstructionPlacementDebugProbe` screen delta is about the footprint, not the tray icon. A single instance at `(0,0)` is not a site count. |

## Before you type

- Walk the matching **PV-*** row. If none match, you are inventing a phase. Stop.
- Do not add a painter “so it shows up.”
- Do not set queue `done` from a lib `*_witness_green` that never sampled a pixel.
- Do not revive archived lod0 or greybox into the player catalog to satisfy a stale 30/50/31 sentence.
