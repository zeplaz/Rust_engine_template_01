# SIM-HUD-SLICE-OPS — Ops strip readability `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **SIM-HUD-SLICE-OPS** |
| **Program** | SIM-HUD-PRODUCT-001 |
| **Owner** | `@designer` → `@coder` |
| **Verdict** | **PASS (qualified)** — spec ready for implementation |
| **Date** | 2026-06-03 |
| **Code** | `src/gui/in_game_hud.rs`, `src/gui/hud/simulation_shell_phase2.rs` |
| **Prereq** | SIM-HUD-SLICE-PLAY01 (session entry hides editor chrome) |
| **Related** | [`ind_e03_ops_strip_polish_v1.md`](ind_e03_ops_strip_polish_v1.md) (PWR overload — already signed) |

---

## Problem

At tactical zoom the ops strip is the **primary status readout**. Today zones use small mono text, muted secondary color on INTEL/WX, and alert badge **◆0** without always pairing glyph + count in one scan line.

---

## Target layout (Bevy native — top strip)

```text
┌─ Operations strip (full width, z=1200) ─────────────────────────────────────┐
│ [TIME  T+01234  RUN  ×1.0]  [◆2  ALERTS  2]  [INTEL  routes on]  [WX  r0.2 s0.1]  [PWR  78%]  [▼ TRAY] │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Strip height:** keep `OPS_STRIP_H_PX` — bump **minimum body font** to **11px** (from 9–10 on badge-only nodes).

---

## Zone copy rules (locked)

Prefix + **two spaces** + body — align with [`OpsStripZone`](../../src/gui/hud/simulation_shell_phase2.rs):

| Zone | Idle (sim) | Notes |
|:---|:---|:---|
| **TIME** | `T+{tick:05}  {PAUSE\|RUN}  ×{speed:.1}` | Pause state word uppercase |
| **ALERTS** | `{badge}  ALERTS  {n}` | Badge text **always** `{n}` numeric — not color-only ◆ |
| **INTEL** | `INTEL  routes {on\|off}` | Drop em-dash placeholder when wired |
| **WX** | `WX  r {precip}  s {wind}` | Two decimals max |
| **PWR** | `PWR  {pct}%` | Overload: `PWR  ⚠ Grid overload — …` per IND-E03 doc |
| **TRAY** | `▼ TRAY` collapsed · `▲ TRAY` expanded | Text label required beside chevron |

---

## Readability requirements (@coder)

| # | Requirement | Test |
|:---:|:---|:---|
| 1 | All zone bodies ≥ **11px** mono (`CmdUiMonoFont`) | Visual / layout test constant |
| 2 | ALERTS: badge + label + count on **one line** | `ALERTS  0` when zero, not hidden |
| 3 | Muted zones (INTEL, WX) still ≥ **4.5:1** contrast on paper fill | Use `bevy_fg_data` not `bevy_text_muted` for body when sim active |
| 4 | Strip never clips PWR on 1280×720 | Flex: ALERTS center grow; TIME/PWR fixed min-width |
| 5 | Overload toast does not truncate body mid-word | Ellipsis only after 48 chars |

---

## PLAY-01 regression

| Check | Pass |
|:---|:---:|
| Enter Simulation → ops strip visible | ✓ |
| WorldGen / scenario script not covering strip | ✓ |
| PWR overload toast reverts to idle % after window | ✓ (IND-E03) |

---

## Witness (@coder)

Extend `debug_runs/ui_shell_migration_live.json` or add `debug_runs/sim_hud_slice_ops_live.json`:

```json
{
  "program_id": "SIM-HUD-SLICE-OPS",
  "green": true,
  "ops_strip_font_min_px": 11,
  "alerts_text_pairing": true,
  "play01_regression": true
}
```

Lib hook: optional test on `format_sim_tick_line` + ops strip update systems if present.

---

## Out of scope

- New ops zones · ALERTS tray body (deferred IND-E03 secondary)
- egui overlay tray contents · multiview parity

---

## Handoff paste (@coder)

```text
SIM-HUD-SLICE-OPS — implement per docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md
Files: in_game_hud.rs (spawn + update), simulation_shell_phase2.rs if constants
Do not merge with assembly_snapshot_qc_ui (egui dev lane)
```
