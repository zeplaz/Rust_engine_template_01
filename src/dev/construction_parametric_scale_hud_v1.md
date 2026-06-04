# DESIGN-PARAM-SCALE-HUD-001 — Parametric scale economy readout `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PARAM-SCALE-HUD-001** |
| **Coder lane** | **B-C5** · **CONSTRUCTION-PARAM-CODER-006** (P4-A economy) |
| **Tray baseline** | [`construction_parametric_tray_mock_v1.md`](construction_parametric_tray_mock_v1.md) § Active ghost readout |
| **Curves** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) § Economy scaling |
| **Plan** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) · Phase 4 |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | `economy_scales_at_activation` · HUD preview before commit |
| **No Rust** | Display contract + formatting only |

---

## Purpose

Define **exact HUD copy, units, and color rules** for economy preview driven by `effective_scale` (`s_eff`) and catalog `placement_scaling` curves. Tray mock v1 showed one line; this doc is authoritative for **B-C5** and tray readout wiring.

**Display-only v1:** numbers preview commit; activation must apply same formulas at `activate_industrial_facilities_system`.

---

## Symbols (match planner)

| Symbol | Formula | HUD label |
|:---|:---|:---|
| `s` | `effective_scale` from weighted raster | (internal) |
| `prod_mult` | `s ^ k_prod` | drives **Prod** % |
| `expense_mult` | `s ^ k_exp` | drives **Expense** % |
| `capex_mult` | `s ^ k_capex` | **Build cost** % |
| `risk_mult` | `s ^ k_risk` | **Risk** index |
| `detect_mult` | `s ^ k_detect` | **Detect** index (secondary line) |
| `fixed_overhead` | catalog flat | included in expense tooltip |

Defaults when catalog omits `placement_scaling`: use plan family defaults (`k_prod=0.90`, `k_exp=1.00`, `k_capex=1.10`, `k_risk=1.35`, `k_detect=0.70`).

---

## Primary readout block (active ghost)

Visible when `origin` is `Some`. Layout: **two lines + badge** (fits 280px tray).

### Line 1 — identity + geometry

```text
{catalog_short} · Scale {scale_factor:.2}× · Rot {rot_deg}° · Mass {mass:.1} tiles
```

| Field | Source |
|:---|:---|
| `catalog_short` | Truncate 18 chars + ellipsis |
| `scale_factor` | Player drag clamped value (show **2** decimals) |
| `rot_deg` | `rotation_quarter_turns * 90` |
| `mass` | `sum(weights)` from raster (1 decimal) |

### Line 2 — economy preview

```text
Prod ~{prod_pct:.0}% · Expense ~{exp_pct:.0}% · Build ~{capex_pct:.0}% · Risk {risk:.2}
```

| Display | Computation |
|:---|:---|
| `prod_pct` | `100 * prod_mult(s)` |
| `exp_pct` | `100 * (base_expense * expense_mult(s) + fixed) / base_expense` — if `base_expense == 0`, show `—` |
| `capex_pct` | `100 * capex_mult(s)` |
| `risk` | `risk_mult(s)` unitless, **2** decimals |

### Line 3 — secondary (optional collapse)

Show when tray expanded **or** `risk_mult(s) > 1.25` **or** user hovers Line 2:

```text
Detect {detect:.2} · Power ~{power_pct:.0}% · Capex curve k={k_capex:.2}
```

`power_pct` uses same exponent as production unless catalog defines `k_power` later — until then mirror `k_prod`.

---

## Color + emphasis rules

| Metric | Normal | Elevated | Suppressed |
|:---|:---|:---|:---|
| **Prod %** | `label_default` | ≥120% `accent_warm` | ≤85% muted |
| **Expense %** | `label_default` | ≥130% `badge_warn` | — |
| **Risk** | `label_default` | ≥1.35 `badge_warn` | ≤0.90 muted |
| **Build %** | `label_muted` | ≥150% `badge_warn` | — |

**Do not** color-only encode risk — always show numeric `Risk 1.31`.

---

## Tooltips (required)

| Target | Tooltip text |
|:---|:---|
| **Scale** | `Effective scale {s_eff:.2} from footprint mass. Drag vertically to resize.` |
| **Prod / Expense line** | `Larger sites: more output per scale, higher running cost. Overhead is per site.` |
| **Risk** | `Single-site exposure index. Many small sites spread risk; one large hub concentrates it.` |
| **Build %** | `Construction time and material cost multiplier before placement.` |

One-line tradeoff (title hover, v1): *Larger: more output, higher exposure.*

---

## Staged row economy (optional column — B-C4/B-C5)

If tray width allows, add **narrow** column after **Scale**:

| Column | Width | Format |
|:---|:---:|:---|
| **Econ** | **64px** | `{prod_pct:.0}/{exp_pct:.0}` muted |

Example: `118/124` = prod%/expense% preview at snapshot time. If omitted for layout, tooltip on **Scale** column shows full Line 2 for that row.

---

## Staging vs single commit

| Mode | Economy source |
|:---|:---|
| Active ghost | Live curves from current ghost + registry |
| Staged row | **Frozen** at LMB-add snapshot (`s_eff`, exponents cached on entry) |
| After registry hot-reload | Staged row shows `Stale` (see staging v2) |

---

## Comparison affordance (v1 light)

When **≥2** staged rows with same `catalog_id`:

- Footer link **Compare selected** (11px) disabled v1 — **defer**.
- Instead: highlight lowest **Risk** row with `*` in Validity column tooltip `Lowest exposure in list`.

---

## Invalid / edge states

| State | HUD behavior |
|:---|:---|
| `s_eff < min_occupied_mass` | Badge **Invalid**; Line 2 hidden; error `Footprint too small` |
| `s_eff` clamped to `scale_max` | Scale shows `2.75×` + muted `(max)` |
| Missing catalog scaling block | Use family defaults; tooltip `Using family defaults` |
| `base_production == 0` | `Prod —` |

---

## ASCII mock (expanded tray)

```text
┌ Active ghost ──────────────────────────────────────────────────┐
│  Solar Array · Scale 1.24× · Rot 90° · Mass 3.7 tiles             │
│  Prod ~118% · Expense ~124% · Build ~113% · Risk 1.31            │
│  Detect 1.09 · Power ~118% · Capex curve k=1.10                  │
│  [ Valid ]                                                        │
└──────────────────────────────────────────────────────────────────┘
```

---

## Witness alignment

| Flag | Criterion |
|:---|:---|
| `economy_scales_at_activation` | Committed site `s_eff ≠ 1` → runtime production ≠ base catalog (test) |
| HUD | Preview Line 2 updates continuously on Shift+drag without integer snap |

---

## Acceptance (designer)

1. Line 1–2 strings match exact templates above (punctuation · spacing).
2. Risk always numeric; warn color at ≥1.35.
3. Tooltips present for Scale, economy line, Risk, Build.
4. Formulas use `s_eff` for mults, `scale_factor` for display label (drag value).
5. Staged snapshot freezes economy numbers at add time.

---

## Coder mapping

| Lane | Deliverable |
|:---|:---|
| **CONSTRUCTION-PARAM-CODER-006** | `placement_scaling.rs` + activation hooks |
| **CONSTRUCTION-PARAM-CODER-004** | Optional Econ column on staged rows |
| **CONSTRUCTION-PARAM-CODER-002** | Live refresh on scale drag |

---

## Sign-off

| Role | Status | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-26 |
| `@coder` | **Unblocked** for B-C5 | — |
