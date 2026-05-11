# UI / UX design language plan `v1`

> **STATUS:** Plan only — codification in Rust follows phased rollout below.  
> **Does not** change external static sites; those remain **visual inspiration** only.

Version: `v1.0.1`  
Audience: anyone touching `src/gui/**`, editor egui, or future Bevy UI HUDs.

**Parents:** [`gui_runbook_v1.md`](gui_runbook_v1.md) (Bevy UI target vs **`TEMP-EGUI`**), [`scenario_campaign_scripted_tools_runbook_v1.md`](scenario_campaign_scripted_tools_runbook_v1.md) (scenario panel), [`map_editor_runbook_v1.md`](map_editor_runbook_v1.md) (editor shell).

---

## 1. Intent

Unify **look and behavior** across:

- **egui** (`bevy_egui`) — diagnostics, faction tools, world gen, keybindings, map editor, scenario script, logistics panels, etc.
- **Bevy UI** — in-game HUD, menus, and runtime chrome per gui_runbook **when** widgets exist or are built.

**Dual inspiration (references, not copies):**

| Source | What to borrow |
|:---|:---|
| **Orgburo / SAPIP “CMD” surfaces** ([`DATA_SYS_CMD~MODELZ`](https://orgburo.org/sapip/data-sys-cmd-modelz/index.html) tone) | Black field, acid **terminal green** accents, **burnt-orange** interaction heat, pipe/`>`/tilde **section cues**, dense “instrument” feel vs consumer gloss. |
| **Modern Rust web** (e.g. [rust-lang.org](https://www.rust-lang.org/), [docs.rs](https://docs.rs/), crates.io patterns) | **Clear hierarchy** (H1/tool title vs body vs muted meta), comfortable **line length**, **focus rings**, **consistent spacing scale**, **accessible contrast**, purposeful **one accent** for primary actions (Rust’s orange is a **reference** for “primary CTA”; we map it to our token, not a logo clone). |

**Goal:** One **recognizable** engine tooling identity — not default egui gray, not random per-panel colors — while staying **readable** and **keyboard-friendly**.

---

## 2. Design principles

1. **Instrument, not brochure** — Panels read as control surfaces; prefer monospace or mixed **mono for data**, sans for chrome (egui: `TextStyle` / `RichText::monospace` where numbers, IDs, paths appear).
2. **Token-first** — No magic `Color32::from_rgb` scattered across files; one **`AppTheme`** / `UiTokens` resource + helpers (`style::section_title`, `style::cmd_label`).
3. **CMD semantics (optional)** — Tooling sections may prefix with cues like `>`, `~`, `|` in **titles only**, not in every sentence; avoid cluttering HUD players see for hours.
4. **Contrast & focus** — All interactive widgets meet **WCAG AA** for text on background in both default-dark states; visible **focus** (egui handles much of this if `Visuals` are set coherently).
5. **Motion & effect restraint** — Subtle hover/selection only; **no** decorative glitch/noise in core tooling unless behind a **debug flair** flag. “Effects” = clarity: separators, striping, progress states.
6. **TEMP-EGUI parity** — Every `TEMP-EGUI` window uses the same tokens until replaced by Bevy UI; migrating a panel **does not** change its design language — only the renderer.
7. **Per gui_runbook authority** — Styling never bypasses mutation / data-source rules; theme code is **presentation-only**.

---

## 3. Token system (target)

Implement as **`UiPalette` + `UiSpacing`** (Resources) or `constants` module under `src/gui/style/` (exact shape TBD in P0).

### 3.1 Color roles (dark-first)

| Token | Role | Example hex (initial proposal) |
|:---|:---|:---|
| `bg_app` | Main panel/window background | near `#0f0f0f` – `#141414` (slightly lifted from pure black for readability) |
| `bg_elevated` | Cards, inset zones | step +8–12% luminance |
| `fg_primary` | Primary text | high-contrast off-white |
| `fg_muted` | Help text, secondary labels | ~60–70% of primary luminance |
| `accent_terminal` | “System OK”, links, positive rails | `#5dca31` (orgburo green — use **sparingly**) |
| `accent_action` | Primary buttons, committed actions | `#d97706`–`#f97316` range (Rust-web **reference** for warmth; tune for contrast on `bg_app`) |
| `accent_hot` | Hover / attention on nav (optional) | `#c64600` (orgburo burnt orange — **secondary** emphasis) |
| `border_subtle` | Separators | low-alpha white or green at ~15% |
| `danger` / `warn` | errors / sim warnings | distinct from `accent_hot` (e.g. red / amber families) |

**Rule:** **`accent_terminal`** = state / telemetry / “pipe” identity; **`accent_action`** = “do something”; **`accent_hot`** = transient hover or map-editor emphasis — don’t overload one hue for all three.

### 3.2 Typography roles

- **Title** — panel title, major collapsible headers.
- **Section** — `CollapsingHeader`, group labels; optional **CMD prefix** via helper.
- **Body** — default widgets.
- **Data** — entity IDs, ticks, file paths: **monospace**.
- **Caption** — tooltips, runbook path hints: **small + muted**.

### 3.3 Spacing & layout

- Single **spacing scale** (e.g. 4 / 8 / 12 / 16 px) for margins between logical groups.
- **Editor / map tools:** slightly **denser** default than player HUD (same tokens, tighter `UiSpacing::editor`).

### 3.4 Effects

- **Separators** — `ui.separator()` with token-colored stroke where it helps scanability.
- **Selection** — `SelectableLabel` / lists: clear selected `bg_elevated` + `accent_terminal` caret or left border.
- **No** full-screen shaders on UI; **optional** thin “scanline” or border glow **only** for Editor brand strip — gated behind a **single** `experimental_ui_flair` flag if ever added.

---

## 4. egui application plan

### 4.1 Global `Visuals`

- On startup (e.g. `AppShellPlugin` or dedicated `UiThemePlugin`), apply **`egui::Visuals { dark_mode: true, ... }`** from tokens:
  - `panel_fill`, `window_fill`, `widgets`, `selection`, `hyperlink_color` (map to `accent_terminal` or `accent_action` per context).
- Keep **light mode** out of scope until dark is stable; then duplicate token lane.

### 4.2 Central helpers

- `fn apply_app_style(ctx: &egui::Context, theme: &UiPalette)` — one call per frame or on change.
- `fn section_heading(ui: &mut Ui, text: &str)` — optional `>` / `~` prefix from a **`CmdUiStyle`** bitfield.
- `fn path_hint(ui: &mut Ui, path: &str)` — monospace muted (scenario panel, save paths).

### 4.3 Panel audit (inventory)

| Area | Typical files | Phase |
|:---|:---|:---|
| Diagnostics | `diagnostics_ui.rs` | P1 |
| Map editor + scenario | `map_editor/mod.rs`, `scenario_script_panel.rs` | P1 |
| World generator | `editor/world_gen_ui.rs` | P1 |
| Keybindings | `options_keybindings_ui.rs` | P2 |
| Faction / production / vehicle tools | `faction_tools_ui.rs`, `production/tools_ui.rs`, `vehicles/tools_ui.rs` | P2 |
| Logistics / HUD adjacent egui | `logistics_targets_panel.rs`, `in_game_hud.rs` (hybrid) | P2 |
| Agent permissions | `agent_permissions_ui.rs` | P3 |

Each phase: replace hardcoded colors with tokens; verify **focus** and **contrast** on Windows at 100% and 125% display scaling.

---

## 5. Bevy UI application plan

Per [`gui_runbook_v1.md`](gui_runbook_v1.md), Bevy UI is the **runtime goal** for HUD/menus.

1. **Define parallel tokens** — `struct BevyUiTheme` mirroring `UiPalette` (Color → `Srgba`).
2. **Theme plugin** — inserts resources consumed by HUD systems when building `TextStyle`, `BackgroundColor`, `BorderColor`.
3. **Migration rule** — When a `TEMP-EGUI` surface moves to Bevy UI, **copy token references**, not one-off colors; add a **screenshot or test** note in the relevant sub-pack row.
4. **Layout** — Prefer flexible containers; match **spacing scale** numerically where possible for continuity.

---

## 6. Phased rollout

| Phase | Deliverable | Exit criteria |
|:---|:---|:---|
| **P0** | `src/gui/style/` — `UiPalette`, `UiSpacing`, `UiThemePlugin`, `apply_egui_theme_system` (`PreUpdate` after `EguiPreUpdateSet::BeginPass`); Bevy main menu reads `Res<UiPalette>` for backdrop / text / buttons | `cargo test -p proc_A_dine01` green; all egui windows pick up `Visuals` |
| **P1** | Diagnostics + map editor + scenario + world gen on tokens; remove scattered `Color32::…` in those files | Visual review on one 1080p display; focus visible on all buttons |
| **P2** | Options, faction, production, vehicle, logistics panels | Sub-pack checklist updated in `g3*_steps` |
| **P3** | Agent permissions + remaining egui | grep for raw `Color32::from_rgb` outside `style/` near zero |
| **P4** | Bevy UI HUD consumes `BevyUiTheme`; document in `g3b` / `g3d` | HUD matches palette; no duplicate hex in HUD builders |

Optional **P5:** User-selectable **“pure CMD”** preset (purer black + stronger green) vs **“balanced”** (current default) — both from same token struct.

### 6.1 P1 addendum — implementation audit (reviewed)

| Suggestion | Status | Notes |
|:---|:---|:---|
| **P0** `UiPalette`, `UiSpacing`, `UiThemePlugin`, `apply_egui_theme_system` after `EguiPreUpdateSet::BeginPass` | **Implemented** | `src/gui/style/palette.rs`, `theme.rs`, `mod.rs`; uses `init_resource` (equivalent to insert + default). Bevy main-menu shell uses `Res<UiPalette>` where wired. |
| **Expected file split:** `spacing.rs`, `egui_theme.rs`, `bevy_theme.rs`, `helpers.rs` | **Deferred (consolidation)** | **Justification:** same behavior with fewer files — `UiSpacing` + helpers live in `mod.rs`, theme application in `theme.rs`, token values in `palette.rs`. Split later if modules grow. |
| **`helpers.rs` with `section_heading(ui, palette: &UiPalette, …)`** | **Implemented (in `style/mod.rs`)** | `section_heading` / `path_hint` take `&UiPalette` + `impl AsRef<str>`; `CmdHeadingStyle` preserved. Split into `helpers.rs` deferred until module size warrants it. |
| **`status_badge`, `warning_text`, `danger_text`** | **Partial** | `warning_text`, `error_text`, `success_text`, `muted_text`, `StatusTone`, `status_badge`, `scenario_execution_badge`, `framed_group` added; `danger_text` exposed as **`error_text`** (maps to `palette.danger`). |
| **P1 surfaces: diagnostics** | **Migrated (REC / errors)** | Recording indicator uses **`error_text`** + `palette.danger` (replaces `Color32::RED`). |
| **P1 surfaces: scenario** | **Migrated (badges + validation)** | Warnings → `warning_text`; errors → `error_text`; status line → `fg_primary`; **State** row uses `scenario_execution_badge` + `ScenarioExecutionState`. |
| **P1 surfaces: world gen, map editor** | **No raw `Color32::*` found** | Mostly default egui / `.weak()` text; no claim of full spacing/visual parity until helpers adopt tokens. |
| **P1 surfaces: logistics, in-game HUD (egui)** | **No raw `Color32` in grep** | Plan §4.3 originally listed logistics / hybrid HUD as **P2** for some rows; `in_game_hud` may be Bevy UI–heavy — verify when touching. |
| **`BevyUiTheme` (`Srgba` mirror)** | **Deferred** | **Justification:** plan §6 phase **P4**; no duplicate theme system until Bevy UI migration needs it. |
| **Grep cleanliness** (`Color32::`, `from_rgb` in `src/gui` outside `style/`) | **Partial** | Remaining: `diagnostics_ui`, `scenario_script_panel`, `options_keybindings_ui` (P2), `agent_permissions_ui`, editor `world_preview/window` (`WHITE` for image tint — acceptable semantic). `style/` intentionally defines tokens with `from_rgb`. |

**Bottom line:** P0 is landed. **PR-A** (palette-parameterized headings/paths + semantic helpers) is **landed**; **diagnostics + scenario** no longer use raw `Color32::*` for the migrated lines. **P1 exit criteria** (all listed surfaces + spacing/focus parity) still **partial** — extend migration to remaining egui panels per §4.3 / grep.

---

## 7. Governance & backlog

- New panel / major widget: add a row to **§4.3** (this doc) in the same PR or open **BQ-###** in [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4 if ownership is unclear.
- **ASK:** before introducing a **second** parallel theme system (e.g. IMGui, second egui context).

---

## 8. Cross-links

| Doc | Role |
|:---|:---|
| [`gui_runbook_v1.md`](gui_runbook_v1.md) | Bevy vs egui policy |
| [`map_editor_runbook_v1.md`](map_editor_runbook_v1.md) | Editor UX |
| [`scenario_campaign_scripted_tools_runbook_v1.md`](scenario_campaign_scripted_tools_runbook_v1.md) | Scenario tooling |
| Gap remediation `g3a`–`g3d` | Per-surface steps |

---

## 9. Prompt fragment for executing agents

> Implement **one phase** of §6. Add or extend `src/gui/style/` tokens; wire `egui::Visuals` in one central plugin; migrate only the files listed for that phase. Do not change gameplay logic. Run `cargo check -p proc_A_dine01`. Update §4.3 status in this doc or the active G3 sub-pack. External websites are **not** modified.
