# PLAN-UI-THEME-MERGE-001 — single theme implementation spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-THEME-MERGE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — coders use **this doc** + code anchors below |
| **Design direction** | [`design_theme.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/design_theme.md) (vision / tokens) |
| **Phase 0 mocks** | [`ui_phase0_panel_mocks_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase0_panel_mocks_v1.md) |

**No new Rust in this deliverable.** Merges scattered theme guidance into one **coder-facing** spec.

---

## Executive summary

| Source | Role after merge |
|:---|:---|
| **`design_theme.md`** | Vision, collage direction, emotional tone — **designer** |
| **`ui_theme_merge_impl_spec_v1.md` (this)** | Tokens, resources, systems, forbidden edits — **@coder** |
| **`src/gui/style/theme.rs`** | Runtime **sole** egui theme applier (editor/diagnostics only in sim) |
| **Bevy HUD chrome** | Panel materials in dock shell / zones — **not** egui |

**Sim session rule:** **PLAY-01** + **UI-P2B** — product HUD is **Bevy**; egui theme applies only where **2B audit** allows (`Diagnostics_F3`, `Editor_tools`).

---

## Token table (canonical)

| Token | Value | Used by |
|:---|:---|:---|
| `--bg-primary` | `#0d0f12` | panels, tray |
| `--bg-elevated` | `#161a20` | floating shells |
| `--text-primary` | `#e8eaed` | labels |
| `--text-muted` | `#8b9298` | secondary |
| `--accent` | `#c4a574` | selection / intel (warm) |
| `--border` | `#2a3038` | 1px separators |
| Font mono | **Fira Mono** / CMD mono | egui + HUD numeric |
| Density | `HudDensityProfile` | scale factor |

**Designer deltas** (collage, decay, handwriting) stay in **`design_theme.md`** until mock § updated — do not hard-code in Rust without designer sign-off.

---

## Code authority map

| Concern | Sole owner | Path |
|:---|:---|:---|
| egui palette resource | `UiPalette` | `src/gui/style/mod.rs` |
| egui theme apply | `UiThemePlugin` | `src/gui/style/theme.rs` |
| Font load | `load_cmd_ui_mono_font` | `src/gui/style/fonts.rs` |
| Density / scale | `HudDensityProfile`, `UiScaleApplicationGate` | `src/gui/style/` |
| Bevy panel chrome | dock / zones | `src/gui/dock_shell.rs`, zone plugins |

**Apply order (egui):**

```text
Startup: load_cmd_ui_mono_font
PreUpdate: reset_ui_scale_application_gate → sync_egui_density_scale
PreUpdate (after BeginPass): apply_egui_theme_system
```

---

## Coder checklist (theme touch)

| # | Rule |
|:---:|:---|
| 1 | Change tokens in **`UiPalette`** / density profile — not ad-hoc `Color32` in feature code |
| 2 | New egui UI only in **editor-allowed** systems per **UI-P2B** audit |
| 3 | Simulation HUD colors → Bevy materials / styles on zone nodes |
| 4 | Match § P1–P4 mock spacing when touching panel chrome |
| 5 | Run **2B** witness after egui edits |

```powershell
cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle
```

---

## PASS gate (planner)

| # | Criterion | Evidence |
|:---:|:---|:---|
| T1 | Single spec exists | this file **SIGNED** |
| T2 | Code plugin wired | `UiThemePlugin` registered |
| T3 | Sim egui gated | `egui_pass_count_in_sim: 0` |
| T4 | Design doc linked | `design_theme.md` referenced, not duplicated |

---

## Forbidden

| Wrong | Correct |
|:---|:---|
| Parallel theme plugins | extend `UiThemePlugin` |
| egui theme in sim product panels | Bevy chrome |
| Copy full `design_theme.md` into Rust comments | link spec |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **PLAN-UI-THEME-MERGE-001** — merged coder spec |
