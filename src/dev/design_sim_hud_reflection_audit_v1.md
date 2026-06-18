# DESIGN-SIM-HUD-REFLECTION-001 — In-game HUD audit & reflection `v1`

| Field | Value |
|:---|:---|
| **Program** | **PLAN-SIM-HUD-PROFESSIONAL-POLISH-001** |
| **ID** | **DESIGN-SIM-HUD-REFLECTION-001** |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (audit)** — opens Phase 2 polish program |
| **Parent close** | [`design_sim_hud_product_signoff_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_sim_hud_product_signoff_v1.md) (**SIM-HUD-PRODUCT-001 CLOSED 2026-06-03**) |
| **Aesthetic authority** | [`ui_design_language_plan_v1.md`](../../docs/archive/2026-06-prompts-guides/runbooks/guides/ui_design_language_plan_v1.md) · [`src/gui/style/palette.rs`](../../src/gui/style/palette.rs) (`UiPalette`) |
| **Boundary** | [`ui_boundary_guide_v1.md`](../../prompts/guides/ui_boundary_guide_v1.md) |
| **Build interaction** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`design_build_toolbox_hud_v1.md`](design_build_toolbox_hud_v1.md) |
| **Operator evidence** | [`operator_playtest_report_20260612_v1.md`](operator_playtest_report_20260612_v1.md) |

**Headline:** Machine witnesses say **PASS (full)** — human play says **patchwork**. The sim HUD **keeps the right colours** (black field, cyan labels, green data, gold selection, vellum tray) but **does not play as one professional tool**: build pickers float in default egui chrome beside Bevy rail; popups anchor arbitrarily; copy and interaction models disagree across surfaces.

**North star:** *One instrument* — same tokens, same interaction grammar, same density ladder — whether the widget is Bevy or egui. **Radical polish, not replatform.**

**Rejected:** new colour system · APS Tk patterns in sim · merging all panels into one mega-window · removing egui before Bevy replacements exist.

---

## 0. Executive summary (designer reflection)

| Dimension | Closed program said | Player/operator reality |
|:---|:---|:---|
| **Cohesion** | 5 slices green | **Two renderers** (Bevy shell + egui satellites) read as different apps |
| **Build flow** | Rail + collapsed tray | **Submenu popup** feels sloppy; catalog hierarchy dense; power labels raw |
| **Discoverability** | Context strip exists | Modifiers **still easy to miss** if tray collapsed/peek-only |
| **Popups** | Dock framework exists | **Staged panel**, **road popup**, **tool hints** use ad-hoc `egui::Area` anchors |
| **Professional feel** | 1080p checklist pass | **Spacing, hierarchy, focus, motion** not unified — “dev tool” not “ship product” |

**Verdict:** SIM-HUD-PRODUCT-001 closed **operational readiness** (collapse defaults, font floors, rail width). **SIM-HUD-PROFESSIONAL-POLISH-002** must close **product craft** (interaction unity, menu polish, popup discipline).

---

## 1. Aesthetic contract (keep — do not replace)

From `UiPalette` / design language plan:

| Token | Role in sim HUD | Keep? |
|:---|:---|:---:|
| `bg_app` / `bg_paper` | Ops strip, map chrome | ✓ |
| `fg_primary` (cyan) | Labels, tab text | ✓ |
| `fg_data` / `accent_terminal` (green) | Telemetry, OK rails | ✓ |
| `accent_gold` | Selected build rail slot | ✓ |
| `bg_vellum` | Context tray selected tab | ✓ |
| `wire_magenta` / `wire_red` | Panel chrome sparingly | ✓ |
| `accent_action` (amber) | Primary commit actions | ✓ |

**Polish rule:** tighten **spacing, type roles, selection, disabled-why, focus rings** — **not** new hues. egui windows must use `UiPalette::to_egui_visuals()` + shell chrome helpers — never default gray.

---

## 2. Surface inventory (full map)

### Tier 1 — Always-on simulation chrome (Bevy native)

| Surface | Code | Role | Polish grade |
|:---|:---|:---|:---:|
| **Operations strip** | `in_game_hud.rs` | Time, weather, power, alerts, intel | B+ |
| **Dev context strip** | `contextual_tip.rs` | Mode + build modifiers line | B |
| **Dev cause strip** | `cause_chain.rs` | L2 hint row | C+ |
| **Build rail** | `simulation_shell_phase2.rs` | Tool icons 52px | B |
| **Context tray** | `in_game_hud.rs` | Alerts / Logistics / Build tabs | B− |
| **Minimap chrome** | `minimap_shell` + interaction | Corner widget + overlays | B |
| **Map viewport inset** | `MapViewportFrameInset` | Pick hole vs chrome | A− |
| **Pause menu** | `pause_menu_bevy.rs` | Esc overlay | B |

### Tier 2 — Build & construction (mixed — **priority**)

| Surface | Renderer | Code | Problem |
|:---|:---:|:---|:---|
| **Build rail submenus** | egui | `draw_sim_build_rail_submenus_egui` | Default window feel; long button lists; engineer labels (`concrete_portland`) |
| **Industrial submenu** | egui | `industrial_menu.rs` | Power in parentheses only; no chain diagram; generic + chain mixed |
| **Residential / commercial / utilities submenus** | egui | `*_menu.rs` | Inconsistent grouping; no search at scale |
| **Build toolbox** | egui | `build_toolbox.rs` | **Gated in sim** but divergent copy vs rail; “Drag title bar…” meta noise |
| **Staged parametric panel** | egui | `staged_ghost_panel.rs` | **RIGHT_BOTTOM anchor** — overlaps dock; unstyled heading |
| **Road tool popup** | egui | `road_path.rs` | Ad-hoc popup |
| **Tool hints overlay** | egui | `tool_hints.rs` | Floating hints — z-order/focus unclear |
| **Phase labels on map** | egui | `phase_visual.rs` | Debug-adjacent clutter risk |
| **Site overlay legend** | spec | `design_build_toolbox_hud_v1.md` §4 | **Designed — tray footer not wired** |

### Tier 3 — Product shell widgets (egui + dock)

| Widget ID | Visible in sim default | Grade |
|:---|:---|:---:|
| Minimap | yes (corner) | B |
| Transmission | collapsed | — |
| Overlay tray | collapsed | — |
| Command shell | collapsed | — |
| **Overlays / Info panel** | user-opened | C+ |
| **Construction queue** | user-opened | C |
| Build toolbox | **gated off** | — |
| Intel timeline | collapsed | — |
| Explainability | dev-leaning | C |

### Tier 4 — Contextual / selection HUD

| Surface | Code | Note |
|:---|:---|:---|
| Grammar read HUD | `grammar_read_hud.rs` | DNA/β — needs human labels ([`design_build_grammar_read_hud_v1.md`](design_build_grammar_read_hud_v1.md)) |
| Logistics focus chips | `in_game_hud.rs` | Petroleum / vehicle chips |
| Validation feedback | `validation_feedback.rs` | Toast/banner discipline |
| Side status panel | `hud_side_status_panel.rs` | Legacy overlap risk |

---

## 3. Build menu deep reflection (P0 designer focus)

### 3.1 What works

- **52px rail** — tactile, matches ghost readability spec
- **Gold selected border** — clear active tool
- **Two-click charter** signed ([`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md))
- **Supply-chain grouped industrial list** — correct semantics (no mega-factory)

### 3.2 What feels sloppy (operator-facing)

| # | Symptom | Root cause | Design fix |
|:---:|:---|:---|:---|
| B1 | Submenu **looks like a debug panel** | egui default + no shell chrome parity with Bevy rail | **Build Picker Sheet** spec — vellum panel, rail-attached, token borders |
| B2 | **Two sources of truth** — rail vs toolbox copy differ | Parallel UIs (`build_toolbox` vs submenus) | Single copy pack; toolbox sim-dead or mirror rail |
| B3 | **Catalog wall of buttons** | No search, no recents, no favourites | Category → chain → step **card row** with icon + power tier glyph |
| B4 | **Power shown as raw float** | `({:.0} power)` engineer string | Tier band glyph + human label ([`design_power_tier_bands_v1.md`](design_power_tier_bands_v1.md) when ready) |
| B5 | **Submenu placement drifts** | Floating area near rail — DPI/resize | Anchor contract: rail right edge + 8px gap; clamp to viewport |
| B6 | **Context tray Build tab underused** | Legend + staging info spec’d for tray — wired to floating panel | Move site legend + parametric readout to **tray Build tab body** |
| B7 | **Peek vs collapsed confusion** | Tray peek 48px — easy to miss modifiers | Context strip **always** shows build modifiers in Adjust (already spec’d — enforce) |
| B8 | **Road/zone/rail tools** different popup pattern | Each tool own egui area | **Tool sheet** component — one popup framework |

### 3.3 Target build interaction model (signed direction)

```text
Build rail tap
  → Build Picker Sheet (egui, token chrome, rail-anchored)
       Category tabs: Zone · Roads · Industry · Utilities · Shapes
       Industry: chain headers · step cards · power tier · preview intent
  → Context strip (always): mode + modifiers
  → Context tray Build tab (expanded): legend · queue · parametric rows
  → Map: ghost + site overlay
  → Second click / optional Enter: place
```

**No third floating “Parametric placement” window in default sim layout.**

---

## 4. Popup & floating menu discipline

### 4.1 Anti-patterns (forbid in polish program)

| ID | Pattern | Example today |
|:---|:---|:---|
| POP-1 | `egui::Area` with hard-coded `RIGHT_BOTTOM` | staged panel |
| POP-2 | Window title meta (“Drag title bar…”) in product sim | build toolbox |
| POP-3 | `ui.heading` without token style | staged panel |
| POP-4 | Multiple popups same corner | dock + staged + road |
| POP-5 | Engineer IDs in primary label | `concrete_portland`, `builtin:*` |
| POP-6 | Color-only invalid state | ghost without strip reason |

### 4.2 Popup tier system (design)

| Tier | Behaviour | Examples |
|:---|:---|:---|
| **P0 Anchored** | Attached to initiating control; closes on outside click / Esc | Build picker, road tool sheet |
| **P1 Tray** | Lives in context tray body — not floating | Site legend, staging list, build queue |
| **P2 Docked shell** | Product shell widget — persisted layout | Overlays panel, construction queue |
| **P3 Modal** | Blocks map; explicit dismiss | Pause menu, confirm destructive |

---

## 5. Cross-menu consistency checklist

Designer signs each surface against:

| # | Criterion | Pass bar |
|:---:|:---|:---|
| C1 | Uses `UiPalette` tokens only | No raw `Color32` in new specs |
| C2 | Primary action **one per panel** | Amber `accent_action` |
| C3 | Disabled controls show **why** adjacent | Not silent gray |
| C4 | Font roles: title / body / data / caption | Data = mono green |
| C5 | Min touch target **36×36** (rail 52 OK) | Buttons in submenus |
| C6 | Spacing scale 4/8/12/16 | No arbitrary 6px gaps |
| C7 | Esc cascade documented | tray → sheet → pause |
| C8 | Sim enter defaults documented | collapsed vs available |
| C9 | 1920×1080 + 2560×1440 layout notes | No clip at ops strip |
| C10 | Copy from locked packs — no drift | build + sim hud copy registries |

---

## 6. Phase 2 program slices (designer → coder)

| ID | Owner | Deliverable | Priority |
|:---|:---|:---|:---:|
| **DES-SIM-HUD-COHESION-001** | @designer | Cohesion charter — Bevy/egui parity rules | P0 |
| **DES-SIM-HUD-BUILD-PICKER-001** | @designer | Build Picker Sheet wire + copy + card layout | P0 |
| **DES-SIM-HUD-TRAY-BUILD-001** | @designer | Context tray Build tab body — legend, staging, queue | P0 |
| **DES-SIM-HUD-POPUP-TIERS-001** | @designer | Popup tier system + migration map per surface | P0 |
| **DES-SIM-HUD-OPS-002** | @designer | Ops strip v2 — alert sort, tier badges, overflow | P1 |
| **DES-SIM-HUD-OVERLAY-002** | @designer | Info/overlay panel IA — tab merge, legend placement | P1 |
| **DES-SIM-HUD-MINIMAP-002** | @designer | Minimap chrome + legend dock (veg/fire) | P1 |
| **DES-SIM-HUD-PAUSE-002** | @designer | Pause menu polish — focus trap, keyboard nav | P2 |
| **DES-SIM-HUD-COPY-REGISTRY-001** | @designer | Single sim HUD copy registry (build + tray + toast) | P0 |
| **DES-SIM-HUD-A11Y-001** | @designer | WCAG AA pass on tokens + non-color invalid | P1 |

**Coder consumers (after sign-off):**

| ID | Owner | Implements |
|:---|:---|:---|
| **COD-SIM-HUD-BUILD-PICKER-001** | @coder | Rail-anchored picker sheet |
| **COD-SIM-HUD-TRAY-BUILD-001** | @coder | Tray body relocation |
| **COD-SIM-HUD-POPUP-MIGRATE-001** | @coder | Staged panel → tray; road → tool sheet |
| **COD-SIM-HUD-EGUI-THEME-001** | @coder | Enforce palette on all sim egui passes |
| **COD-SIM-HUD-CURSOR-001** | @coder | TRIAGE-CURSOR-UNIFY-001 |

---

## 7. Success metrics

| Metric | Today | Target |
|:---|:---|:---|
| Operator “professional tool” score | ~6/10 (est.) | **9/10** |
| Build surfaces using Build Picker Sheet | 0 | **1** (all categories) |
| Floating ad-hoc `egui::Area` in sim build path | 3+ | **0** (tray/sheet only) |
| Copy registries | fragmented | **1** sim HUD registry |
| egui sim panels on `UiPalette` | partial | **100%** |
| G-PLAY build checklist | partial | green |

---

## 8. Relationship to other programs

| Program | Relationship |
|:---|:---|
| [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md) Track D | Minimap legend, ecology preview — subset of this |
| [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) | Power tier glyphs feed build picker cards |
| [`aps_design_system_v1.md`](aps_design_system_v1.md) | **Separate tool** — share words (Assembly vs Build) not widgets |
| SIM-HUD-PRODUCT-001 | **Closed** — this is Phase 2, not reopening witnesses without regression |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (audit)** — Phase 2 slices authorized | 2026-06-18 |

```text
DESIGN-SIM-HUD-REFLECTION-001 complete
Next: DES-SIM-HUD-BUILD-PICKER-001 → COD-SIM-HUD-BUILD-PICKER-001
```
