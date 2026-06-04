# DESIGN-R4-UX-001 — R4 product slice UX (corridor + HUD) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-R4-UX-001** |
| **Coder lane** | **CONSTRUCTION-R4-PRODUCT-001** |
| **Plan** | [`plan_construction_r4_product_open_001_v1.md`](plan_construction_r4_product_open_001_v1.md) |
| **Visual baseline** | [`construction_r4_corridor_map_ux_v1.md`](construction_r4_corridor_map_ux_v1.md) · [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) |
| **MV baseline** | [`construction_r4_mv_post_param_001.md`](construction_r4_mv_post_param_001.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Witness** | `debug_runs/construction_stage_live.json` |
| **Unblocks** | **CONSTRUCTION-R4-PRODUCT-001** |
| **No Rust** | Wireframe + copy only |

---

## Purpose

One **vertical product slice** for Round 4: operator can read **corridor phase** on the sim map, interpret legend, and save transport without new execute funnels.

**Scope:** corridor phase overlay + legend + build-rail microcopy. **Out:** new tools, minimap polylines, parametric staging changes.

---

## Wireframe — sim map chrome (ASCII)

```text
┌─────────────────────────────────────────────────────────────┐
│ [Command tray · collapsed]                    [Minimap float] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ═══════  Planned corridor (amber dash)                    │
│   ███████  In progress (blue solid, partial α)              │
│   ───────  Completed road (committed palette)               │
│                                                             │
│              (tactical map — WorldMain)                     │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Build rail: Rd │ Rl │ …    │ Corridor: Planned ▾           │
│ Legend ┌──────────────────┐                                 │
│        │ ┄ Planned        │  ← amber dashed swatch         │
│        │ ▬ In progress    │  ← blue + progress hint        │
│        │ ■ Completed      │  ← road committed color        │
│        └──────────────────┘                                 │
└─────────────────────────────────────────────────────────────┘
```

Legend: **collapsible** panel anchored near build rail (egui or Bevy HUD — coder choice); default **open** first session, then obeys PLAY-01 collapsed policy.

---

## Map overlay (locked — from R4 corridor UX)

| Phase | Stroke | Color | Pattern |
|:---|:---:|:---|:---|
| **Planned** | 3px | `#E8B040` | dash 8/4 |
| **InProgress** | 4px | `#50A0E8` | solid; α ∝ `progress` |
| **Completed** | — | `road_committed_color()` | no overlay pass |

**Accessibility:** pattern + hue (not color-only).

---

## HUD copy — build rail

| Control / state | String |
|:---|:---|
| Road tool active | `Place corridor — starts Planned` |
| After first commit | `Corridor: Planned — sim will advance phases` |
| In progress on selection | `Corridor: In progress ({progress}%)*` | *dev numeric OK in rail; player sees bar |
| Completed | `Corridor: Open to traffic` |
| Save reminder | `Map save includes corridor phases` |

**Forbidden:** “Execute”, “R8”, “book row”, internal enum names in player strings.

---

## Minimap policy (locked)

| Allowed | Forbidden |
|:---|:---|
| Heat dim when `traffic_factor < 1` | Edge polylines on minimap |
| Logistics rows above terrain | Phase-colored corridor lines on minimap |

---

## Multiview (MV-001 family)

Corridor tokens on **SimulationMap** only — same tokens as [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md). Parametric ghosts **below** corridor overlay z-order.

---

## Acceptance (designer)

1. ☑ Operator distinguishes Planned vs InProgress without tooltip-only color.
2. ☑ Legend fits collapsed sim chrome (does not reopen full Construction egui window).
3. ☑ Completed segments drop overlay; committed road visible.
4. ☑ Witness greens preserved: `construction_r4_corridor_001`, `construction_r4_mv_ghost_001`, parametric placement.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-28 |

**Unblocks:** **CONSTRUCTION-R4-PRODUCT-001** (coder B).
