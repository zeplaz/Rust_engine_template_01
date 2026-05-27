# CONSTRUCTION-PARAM-DESIGN-001 — Parametric ghost visual language `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **CONSTRUCTION-PARAM-DESIGN-001** (map visual slice) |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) § Visual language |
| **MV baseline** | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) (**DESIGN-CONSTRUCTION-MV-001**) |
| **R4 tokens** | [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) — hue unchanged |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Authority** | `src/construction/visual_authority.rs` (**CODER-005**) |
| **No Rust** | Tokens + alpha rules only |

---

## Design rule (locked)

| Principle | Spec |
|:---|:---|
| **Hue** | Valid / Risky / Invalid hues from MV-001 `footprint_*_color()` — **unchanged** by scale |
| **Mass** | Communicated via **per-tile fill alpha** = weight `w ∈ [0,1]` |
| **Scale drag** | No hue shift; optional vertical bounds cue is **non-authoritative** (readout primary) |
| **R4 corridor** | Corridor edge overlay draws **above** terrain, **below** parametric footprint fill |

---

## Active ghost — weighted tiles

| Validity | Base hue (MV-001) | Fill alpha | Outline |
|:---|:---|:---:|:---|
| **Valid** | `#308C48` | `clamp(w * 0.86, 0.12, 0.86)` | 1px @ 90% same hue |
| **Risky** | `#C88C28` | `clamp(w * 0.80, 0.12, 0.80)` | 1px dashed @ 70% |
| **Invalid** | `#B43030` | `clamp(w * 0.88, 0.15, 0.88)` | 1px @ 100% |

**Partial occupation:** tile with `w = 0.25` shows **25%** of max valid alpha — operator sees fractional footprint without new color.

**Gutter:** retain MV-001 **1px** inset between adjacent full tiles; fractional edge tiles may touch — no double-fill bleed.

### Active ghost — envelope

| Element | Spec |
|:---|:---|
| Outer bound | Axis-aligned AABB of weighted footprint — 2px glow `accent` @ 50% (MV-001 § Build site cursor) |
| Rotation | Bounds rotate with ghost; draw order: fill tiles → outline → glow |

---

## Staged ghosts (snapshot, not built)

| Property | Value |
|:---|:---|
| Palette | Same hue family as active |
| Desaturation | Multiply RGB by **0.75** (25% desaturated) |
| Fill alpha | `active_alpha * 0.55` cap 0.45 max |
| Outer bound | **Dashed** 2px @ 60% label_muted |
| Z-order | Staged **under** active ghost; **over** terrain |

**Do not** use a second neon overlay pass — desat + dashed bound only.

---

## Invalid overlap (Σw > 1)

| Layer | Treatment |
|:---|:---|
| Conflicting tiles | Force hue `#B43030` (invalid) regardless of preview validity |
| Fill alpha | `min(0.92, 0.35 + 0.55 * w_conflict)` where `w_conflict` is summed weight over 1.0 |
| Diagnostic | Optional 1px hatch on **top** of red fill @ 25% (coder optional) |

Applies to **active** ghost preview and **staged** row map preview when overlap detected.

---

## Scale drag feedback (optional polish)

| Cue | Priority |
|:---|:---|
| Tray readout + tile alphas | **Required** |
| Vertical bracket on AABB | Optional — 2px `accent` lines at ghost top/bottom during Shift+drag |
| Catalog silhouette scale | **Rejected** — do not scale icon separately from weighted raster |

---

## Per-view matrix (extends MV-001)

| Surface | Parametric weighted fill | Staged desaturated | Overlap red |
|:---|:---:|:---:|:---:|
| **SimulationMap** | ☑ | ☑ | ☑ |
| **WorldMain** (map hole) | ☑ | ☑ | ☑ |
| **World Preview** | ✗ | ✗ | ✗ |
| **Minimap** | ✗ (heat only) | ✗ | ✗ |

---

## Token table (coder constants)

Suggested names in `visual_authority.rs` / `ghost_visual.rs`:

| Token | Value |
|:---|:---|
| `parametric_fill_alpha_scale` | `0.86` (valid max) |
| `parametric_staged_desat` | `0.75` |
| `parametric_staged_alpha_mul` | `0.55` |
| `parametric_overlap_hue` | `#B43030` (same as `footprint_invalid`) |
| `parametric_dashed_bound` | 2px dashed, `label_muted` @ 60% |

---

## Compatibility checklist

| # | Requirement |
|:---:|:---|
| 1 | `construction_mv_001.green` remains true after parametric draw path |
| 2 | R4 corridor Planned `#E8B040` / InProgress `#50A0E8` visible under ghost AABB edge |
| 3 | Valid ghost at `w=1` matches legacy MV-001 solid footprint appearance ±5% alpha |
| 4 | Two staged ghosts at overlap show red on shared tiles before commit blocked |

---

## Acceptance (designer)

1. Partial-alpha tile weights visible at non-unity scale.
2. Staged ghosts read as “pending” via desaturation + dashed bound, not hue swap.
3. Invalid overlap uses red tile weights, not tray-only error.
4. No new hue family introduced for parametric scale tiers.

---

## Coder mapping

| Lane | Deliverable |
|:---|:---|
| **CONSTRUCTION-PARAM-CODER-005** | Weighted tile draw in `visual_authority.rs` |
| **CONSTRUCTION-PARAM-CODER-001** | Raster weights feed alpha |
| **CONSTRUCTION-PARAM-CODER-002** | Active vs staged painter layers |
