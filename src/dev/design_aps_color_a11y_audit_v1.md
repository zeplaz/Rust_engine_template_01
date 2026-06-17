# DES-APS-COLOR-A11Y-001 — Color-not-alone audit `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-COLOR-A11Y-001** |
| **Parent** | [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md) §3 |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Audit matrix

| Surface | Current risk | Required fix | Owner |
|:---|:---|:---|:---:|
| **Footprint heatmap** | Cell fill color = role | Draw **role glyph** center at all cell sizes ≥12px; hatch pattern fallback <12px | C |
| **Material swatch** | `bg=#hex` only | Print **profile_id** (trunc 18) beside swatch; first 2 chars in swatch if narrow | C |
| **Atlas inline status** | Green/red fg | Prefix `PASS:` / `FAIL:` / `WARN:` on every validation line | C |
| **Pipeline bar** | `✓`/`○` | Append word: `✓ Catalog done` / `○ Atlas pending` | C |
| **Lane chip** | Tint only | Word `Buildings`/`Landscape` + tint underline | C |
| **Material tree ●◐○** | Glyph only | `Ready · id` / `Partial · id` / `Missing · id` | C |

## Hatch spec (footprint fallback)

| Role | Hatch | Glyph |
|:---|:---|:---:|
| wall | `///` | `W` |
| door | `+++` | `D` |
| window | `xxx` | `n` |
| roof | `---` | `R` |
| prop | `...` | `P` |

## Guard extension

Extend non-color DoD row 4: grep for `foreground=#` status without adjacent `PASS|FAIL|pending|valid` word in same StringVar setter.

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
