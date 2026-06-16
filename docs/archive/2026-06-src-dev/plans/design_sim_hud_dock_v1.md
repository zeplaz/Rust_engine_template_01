# SIM-HUD-SLICE-DOCK — Collapsed command tray default `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **SIM-HUD-SLICE-DOCK** |
| **Program** | SIM-HUD-PRODUCT-001 |
| **Owner** | `@designer` → `@coder` |
| **Verdict** | **PASS (qualified)** — designer sign-off 2026-06-03 |
| **Date** | 2026-06-03 |
| **Coder handoff** | `SIM-HUD-SLICE-DOCK-CODER` |
| **Code** | `simulation_session.rs`, `dock_shell.rs`, `hud_root_tick.rs`, `shell_framework.rs` |
| **Prereq** | SIM-HUD-SLICE-PLAY01 |

---

## Before / after

| State | Before (bug) | After (target) |
|:---|:---|:---|
| Editor → Sim with Command window **pinned expanded** | Tray stays open, covers map | `apply_simulation_hud_defaults` forces **Collapsed** |
| Sim entry | WorldGen / scenario script may linger | Dismissed per PLAY-01 witness |
| Ops strip TRAY | N/A | Still expands overlay **on purpose** — not a regression |
| Minimap | Sometimes minimized from editor session | `visible=true`, `minimized=false` on sim enter |

```text
BEFORE (sim enter — bad)          AFTER (sim enter — good)
┌ Command window (egui) ────┐   ┌ map + ops strip only ─────────┐
│ full overlay tray open      │   │ ▼ TRAY  (collapsed)           │
└─────────────────────────────┘   │ [minimap] [build rail]          │
                                  └─────────────────────────────────┘
```

---

PLAY-01 requires **collapsed command tray** on Simulation enter. Code paths exist (`collapse_simulation_floating_shell_layout`, `command_tray_state = Collapsed`) but operators still see expanded egui command windows when layout was pinned expanded in editor session.

---

## Target behavior

```text
OnEnter(Simulation):
  command_tray_state      → Collapsed
  overlay_tray_state      → Collapsed
  intel_timeline_state    → Collapsed
  command_table_state     → Collapsed
  transmission.panel      → Collapsed
  floating egui shells    → gated (witness.floating_egui_shells_gated)

Player affordances still available:
  Ops strip TRAY chevron  → expands overlay tray (explicit)
  Minimized shell strip   → chips for pinned widgets only (sim-safe set)
  Minimap dock slot       → visible (not minimized)
```

**Mental model:** Simulation opens **clean map + ops strip + build rail + minimap** — not editor command window.

---

## Requirements (@coder)

| # | Requirement | Acceptance |
|:---:|:---|:---|
| 1 | `apply_simulation_hud_defaults` forces **Collapsed** even if editor had Expanded + pinned | `witness.sim_hud_product_play01_wired == true` |
| 2 | Re-entering Simulation resets tray unless user expanded **during same sim session** via TRAY affordance | Document in witness |
| 3 | `draw_hud_command_shell_egui` skips body when `!command_tray_state.shows_content()` | No flash on first frame |
| 4 | Exit Simulation → editor (`OnExit` or state transition) **does not** require restoring tray — optional nice-to-have | P2 |
| 5 | Minimized strip (`draw_hud_dock_minimized_strip_egui`) shows only sim-allowed widgets | No WorldGen / scenario script chips in sim |

---

## Wireframe — minimized strip (sim)

```text
[Map] [Minimap] [Build]     ← allowed chips when dock minimized
```

Editor-only chips hidden via `suppress_simulation_floating_shell_slots` + `ui_gates`.

---

## PLAY-01 checklist

| Step | Expected |
|:---|:---|
| Editor: expand Command window | Expanded |
| Enter Simulation | Command window **closed**; TRAY shows `▼ TRAY` |
| Click TRAY | Overlay tray expands (egui) — intentional |
| F3 diagnostics | Collapsed sections default (`sections_default_open = false`) |

---

## Witness

`debug_runs/sim_hud_slice_dock_live.json` or extend `ui_shell_migration_live.json`:

```json
{
  "program_id": "SIM-HUD-SLICE-DOCK",
  "green": true,
  "command_tray_collapsed_on_sim_enter": true,
  "play01_wired": true,
  "sim_hud_product_play01_wired": true
}
```

Test: `cargo test -p proc_A_dine01 simulation_session --lib` (existing PLAY-01 assertions).

---

## Out of scope

- Save/restore layout bundle on sim exit (Wave S — separate)
- Retiring egui command shell entirely (Phase 2 migration)
