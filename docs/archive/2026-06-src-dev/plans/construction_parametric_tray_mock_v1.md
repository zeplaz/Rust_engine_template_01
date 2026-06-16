# CONSTRUCTION-PARAM-DESIGN-001 — Parametric build tray mock `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **CONSTRUCTION-PARAM-DESIGN-001** (tray slice) |
| **Plan** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) (**PLAN-CONSTRUCTION-PARAM-001** SIGNED) |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) § HUD/tray |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** · **PASS** (CONSTRUCTION-PARAM-DESIGN-001) |
| **Host** | Bevy `ContextTrayRoot` · **Build** context (when `ActiveBuildTool` = building) |
| **Rail reference** | [`construction_r4_tray_legend_v1.md`](construction_r4_tray_legend_v1.md) — **48+52** legend footer unchanged; parametric block sits **above** R4 corridor legend |
| **Mock style** | [`ui_phase0_panel_mocks_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase0_panel_mocks_v1.md) (flat v2 chrome, wire labels) |
| **No Rust** | Layout + copy + token names only |

---

## Placement on shell

| Layer | Region | Notes |
|:---|:---|:---|
| **P4 left** | `BuildRailRoot` 52px | Tool pick unchanged — no parametric controls on rail |
| **P2 tray** | `ContextTrayRoot` body | Parametric block when building tool active |
| **R4 legend** | Footer 48+52 | Corridor swatches **below** parametric block; never occluded |

**Collapsed PLAY-01:** tray may stay collapsed; **readout one-liner** still visible on ops strip secondary line (optional coder polish) — primary contract is expanded tray.

---

## Tray block — ASCII mock (staging OFF, ghost active)

```text
┌─ Context tray · Build ─────────────────────────────────────────┐
│ [ ] Stage placements          OFF (default)                    │
├──────────────────────────────────────────────────────────────┤
│ Active ghost                                                   │
│  Solar Array · Scale 1.24× · Rot 90° · Mass 3.7 tiles          │
│  Prod ~118% · Expense ~124% · Risk 1.31                        │
│  [ Valid ]                                                     │
├──────────────────────────────────────────────────────────────┤
│ Hints (footer, 11px muted)                                     │
│  LMB: anchor / move ghost                                      │
│  Shift+drag: scale size                                        │
│  Ctrl+scroll / R: rotate · Enter: place building               │
├──────────────────────────────────────────────────────────────┤
│ R4 legend (48+52)  [■ Planned] [■ Building] [○ Open]         │
└──────────────────────────────────────────────────────────────┘
```

---

## Tray block — ASCII mock (staging ON, 0 staged)

```text
┌─ Context tray · Build ─────────────────────────────────────────┐
│ [x] Stage placements          ON                               │
├──────────────────────────────────────────────────────────────┤
│ Active ghost  (same readout as above)                          │
├──────────────────────────────────────────────────────────────┤
│ Staged placements (0)                                          │
│  (empty state — see staged panel spec)                       │
├──────────────────────────────────────────────────────────────┤
│ Hints — staging branch appended:                               │
│  … · [Stage ON] LMB: add ghost to list · Build approved: …    │
└──────────────────────────────────────────────────────────────┘
```

---

## Control spec

### 1 — Stage placements toggle

| Property | Value |
|:---|:---|
| **Control** | Checkbox `[ ]` / `[x]` |
| **Label** | `Stage placements` |
| **Default** | **OFF** (`ConstructionStagingMode::Off`) |
| **Persistence** | Per session only (not save game v1) |
| **Binding** | `ConstructionStagingSettings.mode` (coder 004) |
| **When OFF** | Staged list panel hidden unless `staged_count > 0` (reminder rows) |
| **When ON** | Staged panel visible; Enter routes to **Build approved** if `staged_count > 0` |

### 2 — Active ghost readout

Visible when `BuildGhostState.origin` is `Some`.

| Row | Format | Source |
|:---|:---|:---|
| **Title line** | `{catalog_display_name}` | registry |
| **Params** | `Scale {scale_factor:.2}×` · `Rot {rotation * 90}°` | ghost state |
| **Mass** | `Mass {s_eff_sum:.1} tiles` | weighted raster sum |
| **Economy** | `Prod ~{prod_pct:.0}%` · `Expense ~{exp_pct:.0}%` · `Risk {risk_mult:.2}` | curve preview (display-only v1) |
| **Validity badge** | `Valid` / `Risky` / `Invalid` | `BuildPlacementPreview` |
| **Error line** | First validation error (trunc 48 chars) | only when Invalid |

**Badge colors (flat v2):**

| State | Token | Fill |
|:---|:---|:---|
| Valid | `badge_ok` | `#308C48` @ 90% |
| Risky | `badge_warn` | `#C88C28` @ 90% |
| Invalid | `badge_err` | `#B43030` @ 90% |

**Tooltip (title line hover):** *Larger: more output, higher exposure.*

### 3 — Readout when no ghost

```text
Active ghost
  Pick a site on the map (LMB)
```

Muted `label_muted` — no economy numbers.

---

## Tool hint strings (`tool_hints.rs`)

Replace all Shift+LMB blueprint queue copy. **Building tool only** — roads/rail/zone hints unchanged.

### Staging OFF (default)

```text
LMB: anchor / move ghost
Shift+drag: scale size
Ctrl+scroll / R: rotate
X: mirror · Enter: place building
RMB / Esc: clear ghost
```

### Staging ON (append second line)

```text
[Stage ON] LMB: add ghost to list · Build approved: commit checked · Build all valid: commit all valid rows
```

### Deprecated strings (must not appear)

- `Shift+LMB` + `queue` + `blueprint`
- `Shift+Enter` + `approve`

---

## Typography & spacing (mock tokens)

| Token | Value |
|:---|:---|
| Tray body min width | **280px** (expanded) |
| Section gap | **8px** |
| Readout line height | **14px** / 11px labels |
| Toggle row height | **24px** |
| Hint footer | **11px** `label_muted`, max **3 lines** |

---

## Acceptance (designer)

1. Toggle defaults OFF; label exact match `Stage placements`.
2. Readout updates continuously during Shift+drag scale (no integer snap in copy).
3. Hints never reference Shift+LMB queue.
4. R4 48+52 legend remains visible and below parametric block when corridor legend applies.
5. Staging ON shows staged panel host region (see [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md)).

---

## Coder mapping

| Coder lane | Uses this doc |
|:---|:---|
| **CONSTRUCTION-PARAM-CODER-002** | Hint strings + toggle resource |
| **CONSTRUCTION-PARAM-CODER-004** | Tray readout egui |
| **CONSTRUCTION-PARAM-CODER-006** | Economy preview — full contract [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md) |
