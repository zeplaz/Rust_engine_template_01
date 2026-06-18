# Sim HUD Build Picker Sheet `v1` — rail-anchored

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-BUILD-PICKER-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 2 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) · [`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md) |
| **Interaction** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) — two-click unchanged |
| **Handoff** | COD-SIM-HUD-BUILD-PICKER-001 |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-BUILD-PICKER-001 Q✓
Replaces wall-of-buttons submenu — one sheet, rail-attached, token chrome
```

---

## 0. Purpose

**Highest-leverage P0 fix:** retire `draw_sim_build_rail_submenus_egui` debug-window feel. One **Build Picker Sheet** — vellum panel anchored to the 52px rail — for all build categories.

---

## 1. Open / close

| Action | Result |
|:---|:---|
| Tap rail category slot (Zone, Roads, Industry, …) | Open sheet anchored to that slot; gold rail border |
| Tap same slot again | Close sheet |
| Tap other category slot | Switch sheet tab |
| Click outside sheet | Close |
| **Esc** | Close sheet (first step of cascade) |
| Pick card | Close sheet · load intent · enter Preview |

**Only one sheet** open at a time.

---

## 2. Anchor geometry

```text
┌Rail 52px┐←8px→┌ Build Picker Sheet ──────────────┐
│ [Zone]  │      │ Zone · Roads · Industry · …      │
│ [Roads] │      │ ┌ card ─┐ ┌ card ─┐              │
│ [Ind] ◀─┼──────│ │       │ │       │              │
│ [Util]  │      │ └───────┘ └───────┘              │
│ [Shape] │      └──────────────────────────────────┘
```

| Rule | Value |
|:---|:---|
| Horizontal gap rail → sheet | **8px** (`S8`) |
| Vertical align | Sheet top = **active rail slot** top (clamp if overflow) |
| Sheet width | **320px** default · max **400px** @ 2560×1440 |
| Sheet max height | `min(480px, viewport_h - ops_strip - 32px)` |
| Viewport clamp | Sheet stays fully on-screen — shift up if bottom clip |

**Ban:** `RIGHT_BOTTOM` anchor · floating center screen · DPI drift without clamp.

---

## 3. Chrome

| Element | Spec |
|:---|:---|
| Background | `bg_vellum` header · `bg_elevated` body |
| Border | 1px `wire_magenta` |
| Title row | Category name + `✕` close (36×36 hit target) |
| Tabs | **Category tabs** inside sheet (see §4) — not duplicate rail |

**Typography:** cohesion charter §4.

---

## 4. Category tabs

| Tab | Content source | Replaces |
|:---|:---|:---|
| **Zone** | zone tools list | zone submenu |
| **Roads** | road/path tools | road popup (migrate P2) |
| **Industry** | chain-grouped cards | `industrial_menu.rs` |
| **Utilities** | utility defs | utilities submenu |
| **Shapes** | parametric / pilot shapes | shapes submenu |

Tab order fixed left-to-right. Selected tab = cyan underline + vellum header wash.

**Empty tab:** `○ No tools in this category` (registry `picker.empty_category`).

---

## 5. Industry tab (reference layout)

```text
┌─ Industry ─────────────────────────────────────────────┐
│ Place each step separately — power adds on the grid.   │
├─ Concrete (Portland) ──────────────────────────────────┤
│ ┌─────────────────────┐ ┌─────────────────────┐      │
│ │ Aggregate quarry    │ │ Cement kiln         │      │
│ │ ⚡ light · mine     │ │ ⚡⚡ medium · kiln   │      │
│ └─────────────────────┘ └─────────────────────┘      │
│ ┌─────────────────────┐                              │
│ │ Concrete batching   │                              │
│ │ ⚡ light · mixer     │                              │
│ └─────────────────────┘                              │
├─ Aluminum primary ─────────────────────────────────────┤
│ … step cards …                                         │
├─ Other industry ───────────────────────────────────────┤
│ Generic factory · Generic depot                        │
└────────────────────────────────────────────────────────┘
```

### Chain header
Human `display_name` from [`industrial_supply_chains.json`](../../assets/configs/industrial_supply_chains.json) — **not** `concrete_portland`.

### Step card (36px min height row → 56px card)

| Field | Source | Display |
|:---|:---|:---|
| Title | `def.display_name` | body |
| Power | `power_consumption` | `power_tier_atom` compact — **single** `⚡` + word for HUD ([`design_power_tier_bands_v1.md`](design_power_tier_bands_v1.md) §5 Build HUD) |
| Role | `supply_chain_role` | caption · human label from registry |
| Selected | intent match | gold left bar |

**Click card:** set `building_intent` · close sheet · Preview mode.

**Generic factory/depot:** two compact rows at bottom — not mixed into chain cards.

---

## 6. Other tabs (summary)

| Tab | Card content |
|:---|:---|
| **Zone** | zone type name + one-line effect |
| **Roads** | tool name + `Draw path` caption |
| **Utilities** | display name + `⊞ grid` if `utility_role` |
| **Shapes** | pilot label + footprint hint (`L footprint · 11 tiles`) |

---

## 7. Search & scale (P1 hook)

v1: scroll only. Reserve top row for **P1** search field (`filter` icon) — hidden in v1, layout must not break when added.

Industry @ 30+ defs: vertical `ScrollArea` inside sheet body max height.

---

## 8. States

| State | Sheet shows |
|:---|:---|
| Loading catalog | `⟳ Loading build catalog…` |
| Registry empty | `✗ Catalog unavailable` |
| Rail idle (no build strip) | sheet does not open |

Use status glyph + word per APS pattern adapted to `UiPalette` colours.

---

## 9. Retire list (coder migrate)

| Remove / replace |
|:---|
| `draw_sim_build_rail_submenus_egui` floating window chrome |
| `({:.0} power)` engineer strings in `industrial_menu.rs` |
| Engineer `chain_id` as header |
| Default egui window title bar styling |

Logic (`chain_groups`, intent pick) **reuses** — presentation only changes.

---

## 10. Witness fields

```json
{
  "build_picker_sheet_open": true,
  "anchor_gap_px": 8,
  "sheet_width_px": 320,
  "active_category": "Industry",
  "ad_hoc_submenu_windows": 0
}
```

---

## 11. Verification

| Check | Method |
|:---|:---|
| Rail attach @ 1080p + 1440p | layout screenshot |
| Esc closes sheet | input test |
| Portland chain human header | copy registry |
| Smelter card shows heavy tier | catalog fixture |
| Two-click unchanged | playtest G-PLAY |

---

## 12. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** COD-SIM-HUD-BUILD-PICKER-001
