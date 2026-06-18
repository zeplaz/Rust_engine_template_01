# PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 — in-game HUD Phase 2 `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-SIM-HUD-PROFESSIONAL-POLISH-001
Date: 2026-06-18
Status: **SIGNED** (@planner)
Owner: @designer (reflection + IA) · @coder (Bevy + egui wire)
Prior: SIM-HUD-PRODUCT-001 **CLOSED** 2026-06-03
Audit: $ref:src/dev/design_sim_hud_reflection_audit_v1.md
```

**Headline:** Machine green ≠ player polish. Phase 2 delivers **one professional simulation instrument** — preserving `UiPalette` aesthetic while fixing build menus, popup discipline, and Bevy/egui cohesion.

**North star:** Build rail → **picker sheet** → context strip → tray Build tab → map ghost — **no sloppy floating panels**.

---

## 0. Why Phase 2 exists

[`design_sim_hud_product_signoff_v1.md`](../docs/archive/2026-06-src-dev/plans/design_sim_hud_product_signoff_v1.md) closed **collapse defaults, font floors, rail width, witness JSON**. [`operator_playtest_report_20260612_v1.md`](operator_playtest_report_20260612_v1.md) and ongoing play still report:

- Build submenus feel **debug-grade**
- **Multiple floating popups** with inconsistent chrome
- **Copy drift** between rail, toolbox, strip, tray
- Cursor / placement friction (coder lane separate)

Phase 2 is **design-led reflection + radical UX polish** — not a witness re-run of Phase 1.

---

## 1. Tracks

```text
Track 1 — Cohesion & tokens      designer charter → coder egui theme enforcement
Track 2 — Build & construction   build picker · tray Build tab · popup migration  ★ P0
Track 3 — Ops & overlays         ops strip v2 · info panel IA · minimap chrome
Track 4 — Modal & session          pause menu · Esc cascade · sim defaults
Track 5 — Copy & a11y            single registry · WCAG pass
```

---

## 2. Track 2 — Build & construction (P0)

**Audit:** [`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md) §3

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-SIM-HUD-BUILD-PICKER-001** | @designer | `design_sim_hud_build_picker_v1.md` — rail-anchored sheet, category tabs, chain cards |
| **DES-SIM-HUD-TRAY-BUILD-001** | @designer | `design_sim_hud_tray_build_v1.md` — legend, staging rows, queue in tray body |
| **DES-SIM-HUD-POPUP-TIERS-001** | @designer | Popup tier migration map (staged panel, road popup, tool hints) |
| **DES-SIM-HUD-COPY-REGISTRY-001** | @designer | `sim_hud_copy_registry_v1.md` — locked strings all build surfaces |
| **COD-SIM-HUD-BUILD-PICKER-001** | @coder | Implement picker; retire raw submenu list UI |
| **COD-SIM-HUD-TRAY-BUILD-001** | @coder | Tray Build tab body per spec |
| **COD-SIM-HUD-POPUP-MIGRATE-001** | @coder | Remove ad-hoc anchors; tray/sheet only |
| **COD-SIM-HUD-EGUI-THEME-001** | @coder | All sim egui passes use `UiPalette` + shell chrome |

**Interaction authority (unchanged):** [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`construction_invariants.md`](construction_invariants.md)

---

## 3. Track 1 — Cohesion

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-SIM-HUD-COHESION-001** | @designer | Bevy/egui parity charter — spacing, borders, selection, focus |
| **COD-SIM-HUD-EGUI-THEME-001** | @coder | Theme enforcement + lint note in `ui_boundary_guide` |

**Token authority:** [`ui_design_language_plan_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/ui_design_language_plan_v1.md) · `src/gui/style/palette.rs`

---

## 4. Track 3 — Ops, overlays, minimap

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-SIM-HUD-OPS-002** | @designer | Ops strip alert tiers, overflow, sort |
| **DES-SIM-HUD-OVERLAY-002** | @designer | Info panel tab IA + overlay legend placement |
| **DES-SIM-HUD-MINIMAP-002** | @designer | Minimap legend dock + chrome polish |
| **COD-SIM-HUD-OPS-002** | @coder | Ops strip wire |
| **COD-SIM-HUD-OVERLAY-002** | @coder | Overlay shell wire |

Links: [`design_minimap_veg_legend_v1.md`](design_minimap_veg_legend_v1.md) · Track D in [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md)

---

## 5. Track 4 — Modal & session

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-SIM-HUD-PAUSE-002** | @designer | Pause menu focus + keyboard map |
| **DES-SIM-HUD-ESC-CASCADE-001** | @designer | Esc order: sheet → tray → pause |
| **COD-SIM-HUD-CURSOR-001** | @coder | Unified cursor (TRIAGE-CURSOR-UNIFY-001) |

---

## 6. Priority order

```text
Week 1   DES-SIM-HUD-COHESION-001 + DES-SIM-HUD-BUILD-PICKER-001 + COPY-REGISTRY
Week 2   DES-SIM-HUD-TRAY-BUILD-001 + DES-SIM-HUD-POPUP-TIERS-001
Week 3   COD-SIM-HUD-BUILD-PICKER-001 + EGUI-THEME-001
Week 4   COD-SIM-HUD-TRAY-BUILD-001 + POPUP-MIGRATE-001
Week 5   DES-SIM-HUD-OPS-002 + MINIMAP-002
Week 6   Operator G-PLAY rubric v2 + pixel walk
```

---

## 7. Success metrics

| Metric | Target |
|:---|:---|
| Operator professional score | **9/10** |
| Ad-hoc build popups in sim | **0** |
| Sim egui on UiPalette | **100%** |
| Build copy drift instances | **0** |
| G-PLAY build checklist | green |

---

## 8. Agent routing

| Agent | Pick | Do not pick |
|:---|:---|:---|
| **@designer** | Tracks 1–5 specs, copy registry, operator rubric | Rust |
| **@coder** | Picker, tray, theme, popup migration, cursor | Design authority |
| **@designer-mcp** | Icons for build cards (optional) | HUD layout |
| **Operator** | Pixel walk + G-PLAY-OPERATOR-01 |

**Prompt:** [`designer_sim_hud_prompt_v1.md`](designer_sim_hud_prompt_v1.md)

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-18 | Phase 2 after SIM-HUD-PRODUCT close |

```text
⟦/PLAN-SIM-HUD-PROFESSIONAL-POLISH-001⟧  ΔWF→@designer BUILD-PICKER first
```
