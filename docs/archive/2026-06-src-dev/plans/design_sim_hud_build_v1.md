# SIM-HUD-SLICE-BUILD — Build rail / context tray `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **SIM-HUD-SLICE-BUILD** |
| **Program** | SIM-HUD-PRODUCT-001 |
| **Owner** | `@designer` → `@coder` |
| **Verdict** | **PASS (qualified)** |
| **Date** | 2026-06-03 |
| **Code** | `in_game_hud.rs`, `simulation_shell_phase2.rs`, `simulation_session.rs` |
| **Prereq** | SIM-HUD-SLICE-PLAY01 |
| **Related** | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) |

---

## Problem

Build rail + context tray carry **active tool** and **construction feedback**. In Simulation, context tray should start **collapsed** (PLAY-01) while build rail stays visible for tool selection. Labels and ghost contrast must read at tactical zoom without opening egui shells.

---

## Target layout (left stack)

```text
┌ Build rail ┐ ┌ Context tray (collapsed default) ─────────────┐
│  [icon]    │ │ [Alerts][Logistics][Build]  ← tabs only       │
│  [icon]    │ │ (body hidden until expanded)                  │
│  [icon]    │ └───────────────────────────────────────────────┘
│  52px      │
└────────────┘
```

Constants: `BUILD_RAIL_W_PX = 52`, `CONTEXT_TRAY_TAB_H_PX = 32`, body `96px` when expanded.

---

## Build rail requirements

| # | Requirement | Acceptance |
|:---:|:---|:---|
| 1 | Selected tool slot: **gold border** + label visible (icon + text or tooltip on focus) | Meets construction ghost spec contrast |
| 2 | Inactive slots: muted border; hover brightens — not color-only (label or aria text on slot) | |
| 3 | Rail width fixed **52px** — no wrap | `BUILD_RAIL_W_PX` contract test |
| 4 | Active `ToolContext` syncs to construction ghost palette | Ghost readable on grass + dirt tiles |

---

## Context tray requirements

| # | Requirement | Acceptance |
|:---:|:---|:---|
| 1 | `context_tray.panel_state = Collapsed` on sim enter | `apply_simulation_hud_defaults` |
| 2 | Default tab **Alerts** when user expands (matches `ContextTrayState::default`) | |
| 3 | Tab row always visible when tray Expanded; **Peek** mode uses `CONTEXT_TRAY_PEEK_BODY_H_PX` (48px) max one line | |
| 4 | Body text ≥ **11px** Segoe/mono per zone | |
| 5 | Escape collapses tray (`collapse_context_tray_on_escape`) without hiding build rail | |

---

## Ghost readability (sim construction)

When build tool active:

| Element | Spec |
|:---|:---|
| Valid ghost | Green/cyan outline ≥ 2px; fill alpha ≤ 0.35 |
| Invalid ghost | Red outline + **Invalid placement** one-line in context body when tray expanded |
| Parametric scale | HUD economy readout in tray Build tab when applicable (PARAM scale HUD — read-only) |

Reference construction designer docs — do not change ghost sim authority in this slice.

---

## PLAY-01 checklist

| Check | Pass |
|:---|:---:|
| Enter Sim → context tray collapsed (tabs only if peek) | ✓ |
| Build rail visible + selectable | ✓ |
| Select build tool → ghost visible on map | ✓ |
| Escape → tray collapses | ✓ |

---

## Witness

`debug_runs/sim_hud_slice_build_live.json`:

```json
{
  "program_id": "SIM-HUD-SLICE-BUILD",
  "green": true,
  "context_tray_collapsed_on_sim_enter": true,
  "build_rail_width_px": 52,
  "ghost_readability_wired": true
}
```

---

## Out of scope

- New build tools · construction queue panel (egui pending construction — editor biased)
- Build rail icon atlas redesign (P4-ART done — use existing atlas)

---

## Handoff paste (@coder)

```text
SIM-HUD-SLICE-BUILD — docs/archive/2026-06-src-dev/plans/design_sim_hud_build_v1.md
Touch: in_game_hud.rs spawn/update, simulation_session context_tray collapse
Verify ghost contrast with active ToolContext — no construction sim logic changes
```
