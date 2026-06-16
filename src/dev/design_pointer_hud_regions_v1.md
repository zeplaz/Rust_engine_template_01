# Simulation pointer gate — HUD hit regions `v1` (BUILD-READ-POINTER-HUD-001)

| Field | Value |
|:---|:---|
| **Program** | **BUILD-READ-POINTER-HUD-001** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer` (charter) · `@coder` `SimulationMapPointerGate` |
| **Verdict** | **PASS** |
| **Parent** | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) · BUILD-READ-P0-003 |
| **Code** | `simulation_pointer_gate.rs` · `simulation_shell_phase2.rs` |
| **Witness** | [`debug_runs/design_pointer_hud_regions_live.json`](../debug_runs/design_pointer_hud_regions_live.json) |

---

## Problem

Map hole is **full window**; Bevy chrome overlays without shrinking the hole. Picks and wheel must **not** hit the world under left rail, ops strips, minimap, or context tray — otherwise ghost placement misaligns with visible UI.

**Acceptance test:** *Cursor over Build rail → no map pick · no ghost move · unified cursor hidden over chrome.*

---

## 1. Chrome stack (z-order bottom → top)

```text
[ Map hole — full window ortho pick plane ]
[ Left command stack 52px + build rail ]
[ Top ops strip ]
[ Minimap widget — floating ]
[ Context tray — bottom peek ]
[ egui floating panels — post-pass egui_blocks ]
```

---

## 2. Rect authority (per frame)

| Region | Source | Blocks map pick? | Blocks wheel? | Hide OS cursor? |
|:---|:---|:---:|:---:|:---:|
| **Left command stack** | `command_left_stack_footprint_px` | Yes | Yes | Yes when build active |
| **Build rail** | included in left stack | Yes | Yes | Yes when build active |
| **Top ops strip** | `SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX` | Yes | Yes | No |
| **Minimap widget** | `MinimapShellState.last_window_rect` | Yes | Yes (minimap owns wheel) | No |
| **Context tray** | tab + body height from `ContextTrayState` | Yes | Yes | No |
| **Play area** | `SimulationMapViewport` minus chrome | No | No | Hide OS when build tool active |
| **egui panels** | `finalize_simulation_map_pointer_gate_egui_system` | Yes (`egui_blocks`) | Yes | Per egui want |

---

## 3. Build-tool cursor policy

When `ActiveBuildTool` is **Building** (or any placement tool):

| Location | OS cursor | Game cursor | Map pick |
|:---|:---|:---|:---|
| Play area | **Hidden** | Unified crosshair / anchor | Active |
| Chrome (§2) | Hidden or default | None | **Blocked** |
| egui menu open | Visible if egui wants | — | Blocked |

Pairs with [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) §5 · **TRIAGE-CURSOR-UNIFY-001**.

---

## 4. Debug overlay fields (BUILD-READ-DEBUG-001)

Extend placement debug when `CONSTRUCTION_PLACEMENT_DEBUG=1`:

| Field | Meaning |
|:---|:---|
| `chrome_blocks` | Rect gate — pre-egui |
| `egui_blocks` | Floating panel capture |
| `in_play_area` | Pick allowed |
| `os_cursor_visible` | Witness cursor hide |
| `blocked_region` | `left_stack` \| `top_ops` \| `minimap` \| `context_tray` \| `egui` \| `none` |

---

## 5. Acceptance matrix

| # | Probe |
|:---:|:---|
| 1 | Hover Build rail → `chrome_blocks=true` · no ghost tile change |
| 2 | Hover minimap map image → pick blocked · minimap tap still works |
| 3 | Context tray expanded → picks blocked in tray rect |
| 4 | Play area center → `in_play_area=true` · pick active |
| 5 | Build tool + play area → `os_cursor_visible=false` after unify slice |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-13 |
| `@coder` | pending BUILD-READ-P0-003 verify | — |

```text
BUILD-READ-POINTER-HUD-001 complete
Unblocks: BUILD-READ-P0-003 · BUILD-READ-DEBUG-001 field spec
```
