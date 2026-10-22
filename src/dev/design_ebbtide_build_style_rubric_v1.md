# Ebbtide / ProjectAliveDev — build style + UI rubric `v1`

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-EBBTIDE-BUILD-STYLE-001** |
| **Date** | 2026-10-17 |
| **Owner** | `@designer` (charter) · `@planner` (occupation lock) · `@coder` (presentation) · `@designer-mcp` / `@coder-mcp` (bake) |
| **Verdict** | **PASS** |
| **Refs** | [x.com/ProjectAliveDev](https://x.com/ProjectAliveDev) · [youtube.com/@EbbtideGame](https://www.youtube.com/@EbbtideGame) |
| **Parent readability** | [`design_build_readability_v1.md`](design_build_readability_v1.md) |
| **Massing contract** | [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) §15–18 |
| **Witness** | [`debug_runs/design_ebbtide_build_style_live.json`](../debug_runs/design_ebbtide_build_style_live.json) |
| **Deferral** | **DR-VOXEL-OCCUPATION** — freeform voxel **sim** authority deferred |

**No Rust required in this charter.** Presentation attach lives in `src/construction/` under the massing contract.

---

## Authority lock (Slice 0 — signed)

| Layer | Authority | Role |
|:---|:---|:---|
| **XY occupation** | Keep — `FootprintMatrix` / weighted raster / `FootprintTiles` | Sim: collide, validate, roads-as-tile-strips, zones, commit funnel |
| **Vertical massing** | New — `VoxelMassing` (derived, disposable) | View: column stacks / floor shells for 3D-pixel read |
| **Commit** | Unchanged | `ConstructionPlanQueue` / `CommitConstructionSiteEvent` — never “paint voxel → world” |

**Freeform voxel gameplay is REJECTED** for this program. See **DR-VOXEL-OCCUPATION**.

---

## Reference digest (non-color)

| Source signal | Harvest |
|:---|:---|
| Pixel pass (Oct 5) | 1px outlines + edge highlights; silhouette in clutter; **stable pixel blocks under camera motion**; **2× detail at max zoom** |
| FOB build (Sep 28) | Perimeter → walkways → gates → fittings — phased assembly readability |
| Ops map (Oct 16) | Strategic ↔ tactical dual scale (chrome sparse; world carries identity) |
| Bio | **3D pixel-art** tactical — not 2D city-builder pixel; genre mismatch intentional |

---

## ADOPT

1. Hybrid occupation — XY bits stay sim; VoxelMassing for vertical read.
2. Silhouette-first massing — primary **15–40%** of site stub ([design_build_readability_v1.md](design_build_readability_v1.md)).
3. Edge discipline — ~1px / hard rim on occupied cells + massing stacks (structure, not hue).
4. Camera-stable pixel blocks — no crawling edges on pan/zoom.
5. Zoom LOD — operational = readable mass; max zoom = denser structure detail (`iso_draw_scale` + readability table).
6. Modular phase stack — shell → circulation → aperture → fittings maps onto `SiteConstructionPhase` bands.
7. Sparse chrome — Bevy build rail + collapsed tray; no pixel-sprite HUD panels.

## REJECT

| Temptation | Why |
|:---|:---|
| Freeform voxel sim authority | Breaks tile-strip roads, zones, commit witnesses |
| Delete `FootprintMatrix` this program | Pain is vertical/presentation |
| Parallel pretty-ghost extractor | Invariant §18 → `RepresentationResult` |
| One-click replace two-click place | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) PASS |
| Pixel HUD chrome | [`ui_boundary_guide_v1.md`](../prompts/guides/ui_boundary_guide_v1.md) |
| Palette cloning | Out of scope |

---

## Module mapping

| Concern | Path |
|:---|:---|
| Ghost color / rim tokens | `src/construction/ghost_visual.rs` |
| Phase labels / stack bands | `src/construction/phase_visual.rs` |
| Visual request buffer | `src/construction/visual_authority.rs` |
| VoxelMassing derive + column requests | `src/construction/voxel_massing.rs` |
| Map mesh paint | `src/construction/world_map_frame.rs` |
| Sim picker chrome | `src/gui/hud/sim_build_picker_sheet.rs` |
| Post-commit bake target | `RepresentationResult` (Stage 5 spine) |

---

## Acceptance (non-color)

1. At operational zoom, site primary silhouette readable with rim; site void still visible (15–40%).
2. Pan/zoom: no crawling ghost edges (stable blocks).
3. Phase stack: operator can name shell vs fill vs fittings via shape / outline / fill density (not hue alone).
4. Ghost height reads as **stacked mass**, not a flat tint — column floors ≥ 1 on occupied ghost cells when massing enabled.
5. Commit semantics unchanged — `FootprintMatrix` / weighted raster still sole XY occupation.

---

## L1 ghost decision (locked)

Footprint quads + validity outlines remain occupation preview. VoxelMassing columns may sit **atop** occupied cells only if derived from `BuildGhostState` / weighted footprint and painted solely via `ConstructionVisualRequests` → world-map mesh / later `RepresentationResult`. **No second placement writer.**

**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-VOXEL-OCCUPATION**.
