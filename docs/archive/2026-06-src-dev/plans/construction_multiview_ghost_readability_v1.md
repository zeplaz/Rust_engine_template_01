# Construction multiview ghost readability `v1` (DESIGN-CONSTRUCTION-MV-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-CONSTRUCTION-MV-001** |
| **Planner spec** | [`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md) (**SIGNED**) |
| **Coder lane** | **CONSTRUCTION-MV-SIM-001** — witness **green** on disk |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Policy** | **DQ-POST-04** — ghosts on **SimulationMap** via `ViewProjectionAuthority`, not egui-only camera |
| **Code anchors** | [`ghost_visual.rs`](../construction/ghost_visual.rs) · [`visual_authority.rs`](../construction/visual_authority.rs) · [`map_egui_projection.rs`](../construction/map_egui_projection.rs) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) — preview never executes; single commit funnel |
| **Witness** | `debug_runs/construction_stage_live.json` → `construction_mv_001` |

**No Rust in this doc.** Visual contract for construction ghosts across views — coders align polish to these tokens; wiring exit is separate ([`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md)).

---

## Executive summary

| Surface | Construction ghosts | Authority |
|:---|:---|:---|
| **SimulationMap** | **Full** — roads, zones, rail, building footprint | `ConstructionMapProjection` + `ViewProjectionAuthority` |
| **WorldMain** | Same presentation path when map hole shows tactical view | Shared projection; no duplicate execute |
| **World Preview** | **None** (archive / generator) | Preview chrome only — **no** site commit |
| **Minimap** | **Heat / markers** only | No road polyline ghosts on minimap |

**North star:** Operator sees **valid vs invalid** placement in one glance on the **sim map hole**; ghosts track camera pan/zoom without “swimming” off the terrain.

---

## Ghost families

### 1 — Road / rail path (polyline)

| State | Color (RGBA) | Stroke | Source |
|:---|:---|:---:|:---|
| **Valid segment** | `#50DCB4` @ 140/255 (~55%) | 2px screen; scale with `map_zoom_screen_scale` | `road_segment_color(true)` |
| **Invalid segment** | `#F05A5A` @ 160/255 | 2px | `road_segment_color(false)` |
| **Control point** | `#FFE68C` @ 200/255 | 6px disc | `road_control_point_color()` |
| **Committed road** | `#2A2C34` @ 230/255 | 3px (executed) | `road_committed_color()` |

**Readability rules:**

- Valid green-teal must read on **biome mid-tones** and under light FoW veil.
- Invalid red shifts **hue** on the same polyline — **no** second full-width overlay pass.
- Minimum contrast vs local terrain: **≥ 3:1** for centerline (target **4.5:1** on default grass).

### 2 — Zone paint (tile fill)

| Token | Value |
|:---|:---|
| Fill | Tool-selected color @ **15–25%** alpha over terrain |
| Edge | 1px outline @ **40%** alpha same hue |
| Invalid | Replace fill with `footprint_invalid` hue; keep alpha band |

**Rejected:** solid neon fill @ >40% alpha (obscures logistics / fire underlay).

### 3 — Building footprint (matrix)

| Kind | Color (RGBA) | Meaning |
|:---|:---|:---|
| **Valid** | `#308C48` @ 220/255 | `footprint_valid_color()` |
| **Risky** | `#C88C28` @ 230/255 | slope / soft validation |
| **Invalid** | `#B43030` @ 240/255 | `footprint_invalid_color()` |

| Rule | Spec |
|:---|:---|
| Tile size | 1 world tile = 1 egui rect; inset **1px** gutter between tiles |
| Rotation | Mirror/rotate updates footprint **before** draw — no lagging ghost mesh |
| Occupation grid | Optional per-tile grid when `show_occupation_tiles` — `label_muted` @ 30% |

### 4 — Build site cursor (entity ghost)

| Element | Spec |
|:---|:---|
| Silhouette | Catalog footprint outline + 2px outer glow `accent` @ 50% |
| Snap | Tile-center snap; invalid = `footprint_invalid` only (no hidden execute) |

---

## Per-view matrix

| View / surface | Roads | Zones | Footprint | Rail | Notes |
|:---|:---:|:---:|:---:|:---:|:---|
| **SimulationMap** | ☑ | ☑ | ☑ | ☑ | Primary **DQ-POST-04** target |
| **WorldMain** (map visible) | ☑ | ☑ | ☑ | ☑ | Same draw pass as sim map |
| **World Preview** | ✗ | ✗ | ✗ | ✗ | Generator preview ≠ construction execute |
| **Minimap** | ✗ | ✗ | ✗ | ✗ | Construction **heat** channel only (M2) |
| **Editor egui shell** | Tray/tools only | — | — | — | No ghost authority in product egui sim |

---

## Motion & zoom

| Behavior | Spec |
|:---|:---|
| Pan/zoom | Ghost vertices reproject every frame from `world_to_sim_map_egui` |
| Zoom scale | Stroke widths multiply by `map_zoom_screen_scale` — lines stay visible, not hairline |
| Multiview switch | Ghosts **only** on active map surface; switching views clears stale painter ids |
| Sim entry | `OnEnter(Simulation)` — ghosts visible when construction tool active; collapsed when tray dismissed |

---

## Interaction feedback (no new systems)

| Action | Visual |
|:---|:---|
| Pick tile | Footprint origin snap flash (1 frame accent outline) |
| Invalid commit attempt | Footprint → `invalid` color; **no** toast required for v1 |
| Cancel ghost | Clear paths + footprint within same frame |
| Queue intent (BQ-128 apply) | Ghost updates from preset — same valid/invalid language |

---

## Acceptance (designer / operator)

| # | Pass | Fail |
|:---:|:---|:---|
| 1 | Road valid/invalid distinguishable at default sim zoom | Monochrome lines |
| 2 | Footprint valid/risky/invalid obvious on forest and desert tiles | Lost in terrain |
| 3 | Ghosts stick to map when panning SimulationMap | Detached “floating” lines |
| 4 | No construction ghosts on World Preview window | Ghosts in archive table |
| 5 | `construction_mv_001.green: true` | Witness red while ghosts look wrong |

```powershell
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json_operational_green
cargo test -p proc_A_dine01 --lib coder_b_s7p_construction_mv_proof
```

**Optional visual:** `--test visual` in **Simulation** with road + zone tools — capture to `assets/ui/construction/mv_ghost_readability_target_v1.png` (not required for SIGNED).

---

## Coder polish (post-wiring)

| ID | When | Touch |
|:---|:---|:---|
| **CONSTRUCTION-MV-SIM-001** | **CLOSED** — wiring + witness | — |
| **CONSTRUCTION-R4-PREP-001** | Round 4 prep | ☑ |
| **R4-PLAN-002** | Round 4 product | [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) **SIGNED** |
| **UX-E02-APPLY-POLISH-001** | BQ-128 apply | Ghost apply affordance only |

Align any token drift to [`ghost_visual.rs`](../construction/ghost_visual.rs) — **single source** for RGBA constants.

---

## Forbidden

| Pattern | Why |
|:---|:---|
| Second ghost draw path bypassing `map_egui_projection` | Breaks MV authority |
| Execute from preview / WorldGen | [`construction_invariants.md`](construction_invariants.md) |
| Full-opacity zone fill | Hides operational overlays |
| Minimap polylines for roads | Wrong scale; use construction heat |
| egui-only `MapCameraDesired` when authority committed | DQ-POST-04 violation |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **SIGNED** — readability contract for MV ghosts |
| Planner | 2026-05-26 | **SIGNED** — [`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md) |
| Coder | 2026-05-26 | **CLOSED** — `construction_mv_001.green: true` (sim writer) |

**Unblocks:** Visual QA for **DQ-POST-04**; Round 4 polish; does **not** reopen **CONSTRUCTION-MV-SIM-001** witness.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v0.1.0 | 2026-05-26 | Stub |
| v1.0.0 | 2026-05-26 | **DESIGN-CONSTRUCTION-MV-001** SIGNED — full per-view + token table |
