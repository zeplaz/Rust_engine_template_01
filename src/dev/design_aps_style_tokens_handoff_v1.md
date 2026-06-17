# DES-APS-STYLE-TOKENS-001 — Typography + lane token handoff `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-STYLE-TOKENS-001** |
| **Parent** | [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md) §1 |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` → `@coder-mcp` |
| **Verdict** | **PASS** |

---

## Add to `aps_theme.py` (NEW tokens)

| Token | Value | Use |
|:---|:---|:---|
| `FONT_TITLE` | `("Segoe UI", 12, "bold")` | One H1 per tab |
| `FONT_CAPTION` | `("Segoe UI", 8)` | Decorative only — allowlisted in `test_aps_font_floor` |
| `PAD_XS` | `2` | intra-control |
| `PAD_SM` | `4` | control gap |
| `PAD_MD` | `8` | section padding |
| `PAD_LG` | `12` | inter-group |
| `COLOR_LANE_BUILDING` | `#0a4a7a` | reuse `COLOR_ACCENT` |
| `COLOR_LANE_LANDSCAPE` | `#1f6b54` | teal-green — **≠** `COLOR_PASS` `#0a6b0a` |

## Ramp (locked)

`8 caption-only → 9 body → 9bold section → 10 mono → 12 title`

**Rule:** panels import tokens; no literal `("Segoe UI", N)` tuples.

## Grayscale check (lane landscape)

| Pair | Requirement |
|:---|:---|
| `COLOR_LANE_LANDSCAPE` on `COLOR_PANEL_BG` | ≥ 4.5:1 for chip text |
| vs `COLOR_PASS` | visually distinct in grayscale sim |

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
