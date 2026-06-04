# DESIGN-INFRA-NETWORK-OVERLAY-001 — Infrastructure network overlay UX `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-INFRA-NETWORK-OVERLAY-001** |
| **Parent** | [`world_layer_infrastructure_model_v1.md`](world_layer_infrastructure_model_v1.md) |
| **Plan** | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) · **INFRA-E6** |
| **R4 corridor** | [`construction_r4_corridor_map_ux_v1.md`](construction_r4_corridor_map_ux_v1.md) (construction phase — separate layer) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | **INFRA-E6-003**, **INFRA-E6-004** readability |
| **No Rust** | Colors, legend, chrome |

---

## Purpose

Operators read **graph networks** on the tactical map — roads, rail, utilities — without tile `road: bool` confusion. Aligns with [`world_layer_infrastructure_model_v1.md`](world_layer_infrastructure_model_v1.md).

---

## Overlay colors (tactical map)

| Network | Stroke | Weight | Color | Pattern |
|:---|:---|:---:|:---|:---|
| **Road** (local) | centerline | 3px | `#c8c8c8` | solid |
| **Road** (arterial+) | centerline | 5px | `#f0f0f0` | solid |
| **Rail** | centerline | 4px | `#404040` | dash 6/4 |
| **Power (MV/HV)** | centerline | 2px | `#e8c040` | solid |
| **Water utility** | centerline | 2px | `#4080c0` | solid |
| **Sewer** | centerline | 2px | `#605040` | dash 2/4 |
| **Canal** | centerline | 3px | `#3080a0` | solid |

**Under construction corridor** (R4): use [`construction_r4_corridor_map_ux_v1.md`](construction_r4_corridor_map_ux_v1.md) — **not** road palette.

---

## Legend (collapsible)

```text
Infrastructure
  ─── Road    ╍╍╍ Rail    ··· Power
  ─── Water   ╍╍╍ Sewer
```

Placement: command tray **Overlays** sub-menu or map chrome footer.

---

## Editor vs Simulation chrome

| Session | Overlay toggles | Tray |
|:---|:---|:---|
| **Editor** | full overlay matrix visible | expanded tools OK |
| **Simulation** (PLAY-01) | toggles in collapsed command tray | no WorldGen / full infra editor |
| **WorldGen** | preview tint only — no graph edit | preview chrome |

Default sim: **Road + Rail on**; utilities **off** until player opts in (reduce clutter).

---

## Minimap policy

| Network | Minimap |
|:---|:---|
| Road/rail | **heat / dim** only — no 3px tactical strokes |
| Utilities | off by default |
| R4 corridor phase | heat dim per R4 UX |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
