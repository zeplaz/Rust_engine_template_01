# Minimap widget interaction `v1` (P0-MINIMAP-WIDGET-001)

| Field | Value |
|:---|:---|
| **Program** | **P0-MINIMAP-WIDGET-001** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer` (charter) · `@coder` **MINIMAP-WIDGET-IMPL-001** |
| **Verdict** | **PASS** |
| **Parent** | [`plan_product_polish_exec_001_v1.md`](plan_product_polish_exec_001_v1.md) · G-PLAY-01 |
| **Baseline** | [`design_sim_hud_minimap_v1.md`](../docs/archive/2026-06-src-dev/plans/design_sim_hud_minimap_v1.md) |
| **Code today** | `minimap_bevy_interaction.rs` · `pin_minimap_centered_fit_system` |
| **Witness** | [`debug_runs/design_minimap_widget_live.json`](../debug_runs/design_minimap_widget_live.json) |

**No Rust in this doc.** Interaction contract only.

---

## Problem (operator)

Dragging on the **map image** moves the whole minimap widget instead of navigating the world. Texture pan feels wrong; widget should stay **centered+fit** on resize; only **chrome** drags the panel.

**Acceptance test:** *Drag title bar → widget moves. Click map (no drag) → main camera jumps. Drag on map texture → widget does **not** move.*

---

## 1. Hit regions (GPU Bevy minimap — Simulation)

```text
┌─ Title bar (DRAG) ─────────────── [□] ─┐
│ ┌─ Edge rails (features) ─────────────┐│
│ │ Top=viewport frame toggle           ││
│ │ Left/Right=zoom · Bottom=recenter   ││
│ │ ┌─ MAP IMAGE (NO DRAG) ───────────┐ ││
│ │ │  tap → camera jump             │ ││
│ │ │  wheel → minimap zoom only     │ ││
│ │ │  dbl-tap → tactical zoom bump  │ ││
│ │ └────────────────────────────────┘ ││
│ └─────────────────────────────────────┘│
│                              [resize]  │
└────────────────────────────────────────┘
```

| Region | Rect source | LMB drag | LMB tap | Wheel |
|:---|:---|:---|:---|:---|
| **Title bar** | `shell.title_bar_rect` (new) or top 24px of `last_window_rect` excluding rails | **Move widget** | — | — |
| **Map image** | `last_image_rect` ⊂ body | **Forbidden** — no `panel_drag` | **Jump main camera** to world under cursor | Minimap zoom |
| **Edge rails** | `top/bottom/left/right_rail_rect` | — | Toggle / zoom / recenter | — |
| **Resize grip** | `resize_grip_rect` | Scale square body | — | — |
| **Outside widget** | — | — | — | Overworld zoom |

**Bug fix target:** today `on_map` sets `panel_drag = true` — **invert**: map image never starts panel drag.

---

## 2. Centered + fit (texture)

| Event | Behavior |
|:---|:---|
| Panel resize (corner grip) | Recompute `map_fit_zoom_for_panel(panel, tex_w, tex_h, 0.92)`; **center** on world texture center |
| Sim enter / GPU compositor on | `camera_center = (tex_w/2, tex_h/2)`; fit zoom to body |
| Bottom rail recenter | Same fit — **no** content pan offset stored |
| Resize delta > 2px | `pin_minimap_centered_fit_system` refreshes zoom (existing) |

**Rule:** Minimap **never** accumulates a content pan offset in v0 — world texture stays centered in image rect.

---

## 3. Main camera jump (tap map)

| Input | Result |
|:---|:---|
| Single LMB on map image, movement < 3px | Set `MapCameraDesired.translation` to picked world XY |
| Double LMB within 450ms | Also bump tactical zoom ×1.15 (clamped) |
| During panel drag | No camera jump |

Pick uses `map_surface_screen_to_world` with **image rect** + minimap fit zoom — not widget origin alone.

---

## 4. Overworld interaction boundary

| Condition | Overworld wheel / pick |
|:---|:---|
| Cursor in minimap chrome (any sub-rect) | **Blocked** for overworld — `ActiveMapViewInput = Minimap` |
| Cursor in play area | Normal map camera |

Minimap wheel must **not** scroll the tactical map when hovered (existing — verify no regression).

---

## 5. Witness fields (@coder)

```json
{
  "map_image_drag_moves_widget": true,
  "title_bar_drag_moves_widget": true,
  "tap_map_jumps_camera": true,
  "texture_centered_on_resize": true,
  "content_pan_offset": 0
}
```

Lib fixture: `minimap_bevy_interaction::tests::map_image_drag_moves_panel`

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-13 |
| `@coder` | pending MINIMAP-WIDGET-IMPL-001 | — |

```text
P0-MINIMAP-WIDGET-001 complete
Unblocks: G-PLAY-01 minimap row · MINIMAP-WIDGET-IMPL-001
```
