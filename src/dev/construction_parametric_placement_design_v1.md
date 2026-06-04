# CONSTRUCTION-PARAM-DESIGN-001 — Parametric placement design `v1` (master)

| Field | Value |
|:---|:---|
| **Queue ID** | **CONSTRUCTION-PARAM-DESIGN-001** |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) |
| **Planner plan** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) (**PLAN-CONSTRUCTION-PARAM-001** SIGNED) |
| **Sign-off record** | [`construction_parametric_design_signoff_v1.md`](construction_parametric_design_signoff_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Status** | **SIGNED** |
| **Unblocks** | **CONSTRUCTION-PARAM-CODER-001** … **006** |
| **No Rust** | Tray · staged list · hints · partial-alpha — implementation in coder lanes |

---

## Deliverable index (CONSTRUCTION-PARAM-DESIGN-001)

| # | Deliverable | File | § in this doc |
|:---:|:---|:---|:---:|
| 1 | **Tray toggle mock** | [`construction_parametric_tray_mock_v1.md`](construction_parametric_tray_mock_v1.md) | §1–2, §4 |
| 2 | **Staged-list columns** | [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md) | §3 |
| 3 | **Tool hints** | §4 + tray mock § Tool hint strings | §4 |
| 4 | **Partial-alpha ghost spec** | [`construction_parametric_ghost_visual_v1.md`](construction_parametric_ghost_visual_v1.md) | §5 |

**Follow-on polish (P0, separate queue IDs):** [`construction_parametric_staging_ux_v2.md`](construction_parametric_staging_ux_v2.md) · [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md)

---

## Executive summary

Parametric building placement replaces **Shift+LMB blueprint queue** with an explicit **Stage placements** toggle, a visible **staged ghost list**, and **continuous scale** (Shift+vertical drag) on weighted tile footprints. **Enter** commits the active ghost when staging is OFF (planner-locked). Map feedback uses **partial-alpha tiles** (`α = weight`) while validity **hue** stays on MV-001 / R4 tokens.

---

## Shell placement

| Layer | Region | Parametric UI |
|:---|:---|:---|
| **P4 left** | `BuildRailRoot` 52px | Tool pick only — no scale on rail |
| **P2 tray** | `ContextTrayRoot` | Toggle + readout + staged list + hints |
| **Floating** | `HudWidgetId::ConstructionQueue` | **Retire title** `Pending blueprints` → optional **Staged placements** drawer when tray collapsed; primary host is P2 tray |
| **R4 footer** | 48+52 legend | Corridor swatches **below** parametric block |

**Theme:** post-industrial archive ([`design_theme.md`](../prompts/guides/ui/design_theme.md)) — flat v2 chrome, drafting-ink magenta accent, material badges (stamped valid / overprint invalid).

---

## 1 — Tray toggle

| Property | Value |
|:---|:---|
| Control | Checkbox `[ ]` / `[x]` |
| Label | **`Stage placements`** (exact) |
| Default | **OFF** (`ConstructionStagingMode::Off`) |
| Persistence | Session only (not save v1) |
| Resource | `ConstructionStagingSettings.mode` |

| Mode | Behavior |
|:---|:---|
| **OFF** | Single-ghost flow; Enter commits active ghost; staged panel hidden unless `staged_count > 0` |
| **ON** | LMB on valid ghost **adds snapshot** to list; Enter = **Build approved** when `staged_count > 0` |

### Tray ASCII — staging OFF, ghost active

```text
┌─ Context tray · Build ─────────────────────────────────────────┐
│ [ ] Stage placements          OFF (default)                    │
├──────────────────────────────────────────────────────────────┤
│ Active ghost                                                   │
│  Solar Array · Scale 1.24× · Rot 90° · Mass 3.7 tiles          │
│  Prod ~118% · Expense ~124% · Risk 1.31                        │
│  [ Valid ]                                                     │
├──────────────────────────────────────────────────────────────┤
│ Hints (11px muted)                                             │
│  LMB: anchor / move ghost                                      │
│  Shift+drag: scale size                                        │
│  Ctrl+scroll / R: rotate · Enter: place building               │
├──────────────────────────────────────────────────────────────┤
│ R4 legend (48+52)  [■ Planned] [■ Building] [○ Open]         │
└──────────────────────────────────────────────────────────────┘
```

### Tray ASCII — staging ON

```text
│ [x] Stage placements          ON                               │
│ … Active ghost readout …                                       │
│ … Staged placements panel (see §2) …                           │
│ … Hints + [Stage ON] LMB: add ghost to list …                    │
```

---

## 2 — Active ghost readout

Shown when `BuildGhostState.origin` is `Some`.

| Row | Format |
|:---|:---|
| Title | `{catalog_display_name}` |
| Params | `Scale {scale_factor:.2}×` · `Rot {rotation * 90}°` · `Mass {s_eff_sum:.1} tiles` |
| Economy | `Prod ~{prod_pct:.0}%` · `Expense ~{exp_pct:.0}%` · `Risk {risk_mult:.2}` |
| Badge | **`Valid`** / **`Risky`** / **`Invalid`** |
| Error | First validation error (max 48 chars) when Invalid |

| Badge | Fill token |
|:---|:---|
| Valid | `#308C48` @ 90% |
| Risky | `#C88C28` @ 90% |
| Invalid | `#B43030` @ 90% |

**Tooltip (title hover):** *Larger: more output, higher exposure.*

**No ghost:**

```text
Active ghost
  Pick a site on the map (LMB)
```

---

## 3 — Staged placements panel

**File target:** `src/construction/staged_ghost_panel.rs` (CODER-004). Reuse `PendingConstructionQueue` data; rename UX strings only.

### Visibility

| Condition | Panel |
|:---|:---|
| Toggle OFF + `staged_count == 0` | Hidden |
| Toggle ON | Visible (may be empty) |
| `staged_count > 0` | Visible even if toggle turned OFF |

**Min height:** 120px list + 36px footer = **156px** in tray body.

### Columns

| Col | Width | Content |
|:---:|:---:|:---|
| Approved | 28px | ☑ checkbox |
| Label | flex (min 88px) | Catalog short name |
| Scale | 52px | `1.24×` |
| Rot | 36px | `90°` |
| Validity | 56px | `OK` / `Warn` / `Bad` |
| Remove | 24px | `✕` |

### Footer (exact labels)

| Button | Action |
|:---|:---|
| **Build approved** | Commit checked + valid rows |
| **Build all valid** | Approve all valid, then commit |
| **Clear unapproved** | Drop unchecked or Invalid rows |

**Disabled:** Build approved when no approved valid rows; Build all valid when no valid rows; Clear when nothing to clear.

### States

**Empty:**

```text
┌ Staged placements (0) ─────────────────────────────────────────┐
│  No staged ghosts — adjust active ghost, then LMB on map.      │
└──────────────────────────────────────────────────────────────┘
```

**Overlap invalid row:**

```text
│ ☐  Solar Array      1.80×    0°   Bad   ✕                       │
│    Σ tile weight > 1.0 at (12, 44)                             │
```

**Snapshot rule:** LMB copies ghost; editing active ghost does **not** update existing rows.

**Acceptance:** ≥3 staged, 2 checked → **Build approved** commits exactly 2.

### Legacy panel migration

| Old (`pending_construction_panel.rs`) | New |
|:---|:---|
| Title `Pending blueprints` | Tray section **Staged placements** |
| `Approve all` / `Approve factories` | **Build all valid** / remove factory-only (v1) |
| `Shift+Enter` copy | Remove — use footer buttons |
| Export/import RON | Keep as tertiary row below footer (optional, not blocking parametric v1) |

---

## 4 — Tool hints (`tool_hints.rs`)

**Building tool only.** Roads / rail / zone unchanged.

### Staging OFF

```text
LMB: anchor / move ghost
Shift+drag: scale size
Ctrl+scroll / R: rotate
X: mirror · Enter: place building
RMB / Esc: clear ghost
```

### Staging ON (append one line)

```text
[Stage ON] LMB: add ghost to list · Build approved: commit checked · Build all valid: commit all valid rows
```

### Deprecated (grep must return zero in player-facing strings)

- `Shift+LMB` + `queue` + `blueprint`
- `Shift+Enter` + `approve` (as primary instruction)

**Bottom-left area** (`construction_tool_hints`): keep anchor `LEFT_BOTTOM` (12, -12); confirm line uses bound `confirm_build_placement` key label.

---

## 5 — Partial-alpha map spec (weighted footprint)

**Authority:** same raster as commit ([`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) — 4×4 subcell, 16 samples).

### Active ghost tiles

| Validity | Base hue (MV-001) | Fill α | Outline |
|:---|:---|:---:|:---|
| Valid | `#308C48` | `clamp(w × 0.86, 0.12, 0.86)` | 1px solid @ 90% |
| Risky | `#C88C28` | `clamp(w × 0.80, 0.12, 0.80)` | 1px dashed @ 70% |
| Invalid | `#B43030` | `clamp(w × 0.88, 0.15, 0.88)` | 1px solid @ 100% |

**Partial tile:** `w = 0.25` → ~25% of max fill α — fractional mass visible without new hue tier.

**Envelope:** AABB of weighted tiles; 2px glow `accent` @ 50% (MV-001 build cursor). Draw: fills → outline → glow.

### Staged ghosts (map)

| Property | Value |
|:---|:---|
| Hue | Same validity family |
| Desaturate | RGB × **0.75** |
| Fill α | `active_α × 0.55`, cap **0.45** |
| Bound | **Dashed** 2px `label_muted` @ 60% |
| Z-order | Under active ghost, over terrain |

### Overlap (Σw > 1)

| Layer | Treatment |
|:---|:---|
| Tile hue | Force `#B43030` |
| Fill α | `min(0.92, 0.35 + 0.55 × w_sum)` |
| Optional | 1px hatch @ 25% on top |

### Scale drag feedback

| Cue | Required? |
|:---|:---:|
| Tray readout + tile alphas | **Yes** |
| Vertical AABB bracket during Shift+drag | Optional |
| Scaled catalog icon | **No** |

### Per-surface matrix

| Surface | Weighted fill | Staged | Overlap red |
|:---|:---:|:---:|:---:|
| SimulationMap | ☑ | ☑ | ☑ |
| WorldMain | ☑ | ☑ | ☑ |
| World Preview | ✗ | ✗ | ✗ |
| Minimap | ✗ | ✗ | ✗ |

### Coder constants (suggested)

```text
parametric_fill_alpha_scale   = 0.86
parametric_staged_desat     = 0.75
parametric_staged_alpha_mul = 0.55
parametric_overlap_hue      = #B43030  // footprint_invalid_color
```

**R4:** corridor Planned `#E8B040` / InProgress `#50A0E8` draw **under** parametric footprint edge.

---

## 6 — Typography & spacing

| Token | Value |
|:---|:---|
| Tray body min width | 280px |
| Section gap | 8px |
| Readout line | 14px / labels 11px |
| Toggle row | 24px |
| Hint footer | 11px muted, max 3 lines |

---

## 7 — Coder lane map

| Lane | This design § |
|:---|:---|
| **CODER-001** | §5 overlap + validity column data |
| **CODER-002** | §1 toggle · §4 hints · scale/rotate input |
| **CODER-003** | §2 Enter · §3 Build approved → commit |
| **CODER-004** | §1–3 tray + staged panel |
| **CODER-005** | §5 partial-alpha draw |
| **CODER-006** | §2 economy readout lines |

---

## 8 — Acceptance (designer)

1. Toggle defaults OFF; label `Stage placements`.
2. Enter commits single ghost when staging OFF.
3. Staging ON: LMB adds snapshot; **Build approved** commits only checked valid rows.
4. Scale drag updates readout + tile alphas continuously (no integer snap in copy).
5. Hints never mention Shift+LMB blueprint queue.
6. `w = 0.5` tile visibly half-filled vs `w = 1.0`.
7. Staged ghosts desaturated + dashed; overlap red on map.
8. `construction_mv_001.green` and `construction_r4_corridor_001.green` unchanged.

---

## 9 — Witness (coder rollup — not a design gate)

**File:** `debug_runs/construction_stage_live.json`  
**Block:** `construction_parametric_placement_001`

Designer **PASS** is recorded on **design deliverables** (toggle, columns, hints, partial-alpha). Sim witness `green` is a **coder** acceptance gate — see [`construction_parametric_design_signoff_v1.md`](construction_parametric_design_signoff_v1.md).

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** — CONSTRUCTION-PARAM-DESIGN-001 complete | 2026-05-26 |
| `@planner` | **PASS** — [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) | 2026-05-26 |
| `@coder` | **Unblocked** — implement 001…006 | 2026-05-26 |
