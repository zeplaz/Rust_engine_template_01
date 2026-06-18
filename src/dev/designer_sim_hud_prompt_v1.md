# @designer — in-game HUD Phase 2 prompt `v1`

**Program:** [`plan_sim_hud_professional_polish_v1.md`](plan_sim_hud_professional_polish_v1.md)  
**Audit (read first):** [`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md)  
**Aesthetic (keep):** [`ui_design_language_plan_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/ui_design_language_plan_v1.md) · `UiPalette` in `src/gui/style/palette.rs`  
**Build interaction (unchanged):** [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md)

---

## Situation

SIM-HUD-PRODUCT-001 is **closed** at witness level — but playtest shows **sloppy menus**: build submenus float in default egui beside the Bevy rail; staged parametric panel anchors bottom-right; copy drifts across surfaces. **Keep the colour story** (black/cyan/green/gold/vellum). **Fix craft** — spacing, hierarchy, popup discipline, one build picker pattern.

**No Rust.** Specs + wireframes + copy registry only.

---

## P0 — This week (build menus first)

```
1. DES-SIM-HUD-COHESION-001
   design_sim_hud_cohesion_charter_v1.md
   Rules: Bevy shell vs egui satellite parity (borders, spacing, selection, focus)
   Ref: design_sim_hud_reflection_audit_v1.md §1 aesthetic contract

2. DES-SIM-HUD-BUILD-PICKER-001  ★ highest leverage
   design_sim_hud_build_picker_v1.md
   Rail-anchored Build Picker Sheet:
     - Category tabs: Zone · Roads · Industry · Utilities · Shapes
     - Industry: chain header · step card (name, power tier glyph, role)
     - Rail-attached anchor (8px gap), vellum bg, token chrome
     - Replaces wall-of-buttons submenu pattern
   Ref: industrial_menu.rs behaviour · design_build_toolbox_hud_v1.md copy

3. DES-SIM-HUD-COPY-REGISTRY-001
   sim_hud_copy_registry_v1.md
   Single locked string table: context strip, picker, tray, toasts
   Merge design_build_ux_redesign + design_build_toolbox_hud — no drift
```

---

## P0 — Next (tray + popups)

```
4. DES-SIM-HUD-TRAY-BUILD-001
   design_sim_hud_tray_build_v1.md
   Context tray Build tab body:
     - Site stub legend (from design_build_toolbox_hud §4)
     - Staged placement rows (move off floating RIGHT_BOTTOM panel)
     - Pending queue summary
   Default: collapsed sim enter; expandable; peek shows one modifier line only

5. DES-SIM-HUD-POPUP-TIERS-001
   design_sim_hud_popup_tiers_v1.md
   Migration map: staged_ghost_panel → tray; road popup → tool sheet;
   tool_hints → strip or tray footer
   Ref: audit §4 anti-patterns POP-1..POP-6
```

---

## P1 — Ops, overlays, minimap

```
6. DES-SIM-HUD-OPS-002        — ops strip alert tiers + overflow
7. DES-SIM-HUD-OVERLAY-002    — info panel tab IA
8. DES-SIM-HUD-MINIMAP-002    — legend dock + chrome (links design_minimap_veg_legend_v1)
9. DES-SIM-HUD-A11Y-001       — WCAG AA on tokens + invalid states
```

---

## P2 — Session chrome

```
10. DES-SIM-HUD-PAUSE-002       — pause menu keyboard + focus
11. DES-SIM-HUD-ESC-CASCADE-001 — Esc order doc for @coder
12. DES-G-PLAY-OPERATOR-V2-001  — operator checklist incl. build picker walk
```

---

## @coder handoff (after P0 specs signed)

| Spec | Coder slice |
|:---|:---|
| build_picker_v1 | COD-SIM-HUD-BUILD-PICKER-001 |
| tray_build_v1 | COD-SIM-HUD-TRAY-BUILD-001 |
| popup_tiers_v1 | COD-SIM-HUD-POPUP-MIGRATE-001 |
| cohesion_charter | COD-SIM-HUD-EGUI-THEME-001 |

---

## Rules

- **Keep UiPalette** — polish layout/interaction, not colours
- **No APS Tk patterns** — sim HUD is Bevy + egui product shell
- **One build picker** — rail submenus retire visually (coder migrates)
- **No engineer IDs** in primary labels (`concrete_portland` → “Concrete (Portland)”)
- **Invalid = text reason** — never color-only (construction invariants)
- **Two-click placement** authority unchanged

```text
ΔWF→ DES-SIM-HUD-BUILD-PICKER-001 after DES-SIM-HUD-COHESION-001
```
